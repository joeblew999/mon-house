/// Cloudflare Worker entry point for the quick-tool translation service.
///
/// ## Routes
///
/// | Method | Path         | Description                              |
/// |--------|--------------|------------------------------------------|
/// | GET    | /health      | Liveness check — returns `{"ok":true}`   |
/// | POST   | /translate   | Translate EN markdown to Thai via Claude |
///
/// ## Environment
///
/// | Name                | Type   | Source              |
/// |---------------------|--------|---------------------|
/// | `ANTHROPIC_API_KEY` | secret | `wrangler secret`   |
/// | `QUICK_CLAUDE_MODEL`| var    | `wrangler.toml`     |
use worker::*;

mod translate;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/health", |_, _| Response::ok("{\"ok\":true}"))
        .post_async("/translate", translate::handle)
        .run(req, env)
        .await
}
