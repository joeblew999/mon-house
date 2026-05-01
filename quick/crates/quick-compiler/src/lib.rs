// quick-compiler — CF Worker (workers-rs + typst-pdf)
//
// Routes:
//   GET  /health   → "ok"
//   POST /compile  → request body is the typst source (text/plain), response is PDF bytes
//
// Leptos UI is a separate crate (planned: crates/quick-app) that calls this
// Worker via service binding. Splitting them keeps each Worker comfortably
// under the CF 10 MiB limit; the all-in-one Leptos+typst Worker hits a
// rustc-emits-custom-descriptors bug that V8 rejects.

mod compile;

use worker::*;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    Router::new()
        .get("/health", |_req, _ctx| Response::ok("ok"))
        .post_async("/compile", |mut req, _ctx| async move {
            let source = req.text().await?;
            match compile::compile_pdf(&source) {
                Ok(pdf) => {
                    let headers = Headers::new();
                    headers.set("Content-Type", "application/pdf")?;
                    headers.set("Content-Length", &pdf.len().to_string())?;
                    Ok(Response::from_bytes(pdf)?.with_headers(headers))
                }
                Err(msg) => Response::error(format!("compile error:\n{msg}"), 400),
            }
        })
        .run(req, env)
        .await
}
