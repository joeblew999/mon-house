// typst → PDF compile, extracted from lib.rs so it can be reused.
//
// Spike font strategy: embed ONE font (Inter 400) so typst initialises and
// renders Latin text. Thai/Sarabun + alternative weights move to R2 lazy-load
// next iteration — that's what frees enough headroom to ship the full Leptos
// stack alongside typst on the same Worker.
//
// Until R2 fonts land, /compile only renders correctly with Latin source.

const INTER_400: &[u8] = include_bytes!("../../../resources/fonts/inter_400.ttf");

fn fonts() -> [&'static [u8]; 1] {
    [INTER_400]
}

pub fn compile_pdf(source: &str) -> Result<Vec<u8>, String> {
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
