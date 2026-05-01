/// Serve subcommand — HTTP compile server for the CF Container.
///
/// Runs inside the Cloudflare Container that sits alongside the PipelineAgent
/// Durable Object.  The Agent calls `POST /compile` after translating a spec;
/// the server compiles the Thai PDF and returns the bytes.
///
/// ## Endpoints
///
/// | Method | Path       | Request body              | Response               |
/// |--------|------------|---------------------------|------------------------|
/// | GET    | /health    | —                         | 200 "ok"               |
/// | POST   | /compile   | JSON `CompileRequest`     | 200 application/pdf    |
///
/// ## CompileRequest
///
/// ```json
/// { "name": "GATE", "content": "# Gate\n\n..." }
/// ```
///
/// `content` is the Thai-translated markdown (`.th.md`).
///
/// ## Concurrency
///
/// Typst writes a temporary `_tmp.typ` wrapper to the current working directory.
/// A global `Mutex` serialises requests so concurrent calls don't race on that
/// file.  One compile at a time is fine for a PDF compiler container.
use std::sync::Mutex;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::{build, vfs, Config};

#[derive(Deserialize)]
struct CompileRequest {
    /// Spec name without extension (e.g. "GATE").
    name: String,
    /// Thai-translated markdown content (`.th.md` body).
    content: String,
}

// Serialize all compile calls — typst writes _tmp.typ to cwd.
static COMPILE_LOCK: Mutex<()> = Mutex::new(());

pub fn cmd_serve(cfg: &Config, port: u16) -> Result<()> {
    use tiny_http::Server;

    let addr = format!("0.0.0.0:{port}");
    let server = Server::http(&addr)
        .map_err(|e| anyhow::anyhow!("failed to start HTTP server on {addr}: {e}"))?;

    println!("quick-tool compile server on :{port}");
    println!("  POST /compile  — Thai spec → PDF");
    println!("  GET  /health   — liveness probe");
    println!("  scripts : {}", cfg.scripts_dir.display());
    println!("  fonts   : {}", cfg.resolved_font_dir().display());

    for mut request in server.incoming_requests() {
        let method = request.method().to_string();
        let url    = request.url().to_owned();

        let result = match (method.as_str(), url.as_str()) {
            ("GET",  "/health")  => health(),
            ("POST", "/compile") => compile(&mut request, cfg),
            _                    => not_found(),
        };

        match result {
            Ok(resp) => { let _ = request.respond(resp); }
            Err(e) => {
                eprintln!("  error: {e:#}");
                let body = format!("{e:#}");
                let _ = request.respond(
                    tiny_http::Response::from_string(body).with_status_code(500),
                );
            }
        }
    }
    Ok(())
}

// ── handlers ──────────────────────────────────────────────────────────────────

type Resp = tiny_http::Response<std::io::Cursor<Vec<u8>>>;

fn health() -> Result<Resp> {
    let body = b"ok".to_vec();
    Ok(tiny_http::Response::new(
        tiny_http::StatusCode(200),
        vec![],
        std::io::Cursor::new(body),
        Some(2),
        None,
    ))
}

fn not_found() -> Result<Resp> {
    let body = b"not found".to_vec();
    let len = body.len();
    Ok(tiny_http::Response::new(
        tiny_http::StatusCode(404),
        vec![],
        std::io::Cursor::new(body),
        Some(len),
        None,
    ))
}

fn compile(request: &mut tiny_http::Request, cfg: &Config) -> Result<Resp> {
    // Read JSON body
    let mut body = String::new();
    request
        .as_reader()
        .read_to_string(&mut body)
        .context("reading request body")?;
    let req: CompileRequest =
        serde_json::from_str(&body).context("parsing CompileRequest JSON")?;

    // Per-request scratch directory (absolute paths → no cwd conflicts)
    let tmp = std::env::temp_dir().join(format!("quick-{}", req.name));
    let specs_dir = tmp.join("specs");
    let out_dir   = tmp.join("out");
    vfs::create_dir_all(&specs_dir)?;
    vfs::create_dir_all(&out_dir)?;

    // Write Thai spec
    let th_path = specs_dir.join(format!("{}.th.md", req.name));
    vfs::write(&th_path, req.content.as_bytes())?;

    // Build config pointing at temp dirs; scripts/fonts stay at container paths
    let mut build_cfg = cfg.clone();
    build_cfg.specs_dir = specs_dir;
    build_cfg.out_dir   = out_dir.clone();

    // Compile — serialised because typst writes _tmp.typ to cwd
    {
        let _lock = COMPILE_LOCK
            .lock()
            .map_err(|_| anyhow::anyhow!("compile mutex poisoned"))?;
        build::compile_th(&req.name, &build_cfg)
            .with_context(|| format!("typst compile failed for {}", req.name))?;
    }

    // Read PDF and return
    let pdf_path  = out_dir.join(format!("{}.th.pdf", req.name));
    let pdf_bytes = std::fs::read(&pdf_path)
        .with_context(|| format!("reading {}", pdf_path.display()))?;

    // Cleanup scratch dir (best-effort)
    let _ = std::fs::remove_dir_all(&tmp);

    let len = pdf_bytes.len();
    println!("  compiled {} → {len} bytes", req.name);

    Ok(tiny_http::Response::new(
        tiny_http::StatusCode(200),
        vec![
            tiny_http::Header::from_bytes(b"Content-Type", b"application/pdf").unwrap(),
            tiny_http::Header::from_bytes(
                b"Content-Length",
                len.to_string().as_bytes(),
            )
            .unwrap(),
        ],
        std::io::Cursor::new(pdf_bytes),
        Some(len),
        None,
    ))
}
