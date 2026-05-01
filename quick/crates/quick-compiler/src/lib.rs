// quick-compiler — CF Worker that compiles typst source to PDF.
//
// Routes:
//   GET  /health   → "ok"
//   POST /compile  → request body is the typst source (text/plain), response is PDF bytes
//
// Fonts are embedded via include_bytes! from quick/resources/fonts/.
// First-pass spike: no package resolver, so specs that #import "@preview/..."
// won't compile yet — supply pre-flattened typ source for now.

use worker::*;

// Embedded fonts — same set served by the local typst CLI.
// Path relative to this file: ../../../resources/fonts/
const INTER_400:          &[u8] = include_bytes!("../../../resources/fonts/inter_400.ttf");
const INTER_700:          &[u8] = include_bytes!("../../../resources/fonts/inter_700.ttf");
const NOTO_SANS_400:      &[u8] = include_bytes!("../../../resources/fonts/noto_sans_400.ttf");
const NOTO_SANS_700:      &[u8] = include_bytes!("../../../resources/fonts/noto_sans_700.ttf");
const NOTO_SANS_THAI_400: &[u8] = include_bytes!("../../../resources/fonts/noto_sans_thai_400.ttf");
const NOTO_SANS_THAI_700: &[u8] = include_bytes!("../../../resources/fonts/noto_sans_thai_700.ttf");
const SARABUN_400:        &[u8] = include_bytes!("../../../resources/fonts/sarabun_400.ttf");
const SARABUN_700:        &[u8] = include_bytes!("../../../resources/fonts/sarabun_700.ttf");

fn fonts() -> [&'static [u8]; 8] {
    [
        INTER_400, INTER_700,
        NOTO_SANS_400, NOTO_SANS_700,
        NOTO_SANS_THAI_400, NOTO_SANS_THAI_700,
        SARABUN_400, SARABUN_700,
    ]
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    Router::new()
        .get("/health", |_req, _ctx| {
            Response::ok("ok")
        })
        .post_async("/compile", |mut req, _ctx| async move {
            let source = req.text().await?;
            match compile_pdf(&source) {
                Ok(pdf) => {
                    let mut headers = Headers::new();
                    headers.set("Content-Type", "application/pdf")?;
                    headers.set("Content-Length", &pdf.len().to_string())?;
                    Ok(Response::from_bytes(pdf)?.with_headers(headers))
                }
                Err(msg) => {
                    Response::error(format!("compile error:\n{msg}"), 400)
                }
            }
        })
        .run(req, env)
        .await
}

fn compile_pdf(source: &str) -> std::result::Result<Vec<u8>, String> {
    use typst_as_lib::TypstEngine;

    let engine = TypstEngine::builder()
        .main_file(source)
        .fonts(fonts())
        .build();

    let doc = engine
        .compile()
        .output
        .map_err(|errs| format!("typst compile failed: {errs:?}"))?;

    let options = Default::default();
    typst_pdf::pdf(&doc, &options)
        .map_err(|errs| format!("typst-pdf failed: {errs:?}"))
}
