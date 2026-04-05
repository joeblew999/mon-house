/// POST /translate — translate EN construction-spec markdown to Thai.
///
/// Reuses from `quick-tool` (cli/):
///   - `SYSTEM_PROMPT`        — shared prompt text, single source of truth
///   - `ApiRequest/Message`   — Serde types for Claude Messages API body
///   - `ApiResponse/Content`  — Serde types for response parsing
///   - `clean_output`         — strips code fences, trims whitespace
///
/// HTTP is NOT shared — the CLI uses sync `ureq`; here we use async `worker::Fetch`.
///
/// ## Request
/// ```json
/// { "content": "# Gate Spec\n..." }
/// ```
///
/// ## Response (200)
/// ```json
/// { "thai": "# ข้อมูลประตู\n..." }
/// ```
use quick_tool::translate::{ApiContent, ApiMessage, ApiRequest, ApiResponse, clean_output, SYSTEM_PROMPT};
use serde::{Deserialize, Serialize};
use worker::{Env, Fetch, Headers, Method, Request, RequestInit, Response, RouteContext};

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const DEFAULT_MODEL: &str = "claude-opus-4-6";

// ── Worker-local request/response shapes ──────────────────────────────────────

#[derive(Deserialize)]
struct TranslateRequest {
    content: String,
}

#[derive(Serialize)]
struct TranslateResponse {
    thai: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// ── Handler ────────────────────────────────────────────────────────────────────

pub async fn handle(mut req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let body: TranslateRequest = match req.json().await {
        Ok(b) => b,
        Err(e) => return error_response(400, &format!("invalid request body: {e}")),
    };
    if body.content.trim().is_empty() {
        return error_response(400, "content must not be empty");
    }

    let api_key = match ctx.env.secret("ANTHROPIC_API_KEY") {
        Ok(s) => s.to_string(),
        Err(_) => return error_response(500, "ANTHROPIC_API_KEY secret not configured"),
    };
    let model = ctx
        .env
        .var("QUICK_CLAUDE_MODEL")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| DEFAULT_MODEL.to_string());

    match call_claude(&api_key, &model, &body.content).await {
        Ok(thai) => Response::from_json(&TranslateResponse { thai }),
        Err(e) => error_response(502, &format!("Claude API error: {e}")),
    }
}

// ── Claude API call (async, worker::Fetch) ────────────────────────────────────

async fn call_claude(api_key: &str, model: &str, content: &str) -> worker::Result<String> {
    let prompt = format!("Translate this construction spec to Thai:\n\n{content}");

    // Reuse shared Serde types from quick-tool — single source of truth for the API contract.
    let body = ApiRequest {
        model,
        max_tokens: 8096,
        system: SYSTEM_PROMPT,
        messages: vec![ApiMessage { role: "user", content: &prompt }],
    };
    let body_str = serde_json::to_string(&body)
        .map_err(|e| worker::Error::RustError(e.to_string()))?;

    let mut headers = Headers::new();
    headers.set("content-type", "application/json")?;
    headers.set("x-api-key", api_key)?;
    headers.set("anthropic-version", "2023-06-01")?;

    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_headers(headers)
        .with_body(Some(body_str.into()));

    let api_req = Request::new_with_init(API_URL, &init)?;
    let mut resp = Fetch::Request(api_req).send().await?;

    if resp.status_code() != 200 {
        let text = resp.text().await.unwrap_or_default();
        return Err(worker::Error::RustError(format!(
            "HTTP {}: {text}",
            resp.status_code()
        )));
    }

    // Reuse shared response types and output cleaner from quick-tool.
    let api_resp: ApiResponse = resp
        .json()
        .await
        .map_err(|e| worker::Error::RustError(format!("deserialising response: {e}")))?;

    let text = api_resp
        .content
        .into_iter()
        .find(|c: &ApiContent| c.kind == "text")
        .and_then(|c| c.text)
        .ok_or_else(|| worker::Error::RustError("no text block in Claude response".into()))?;

    Ok(clean_output(text))
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn error_response(status: u16, msg: &str) -> worker::Result<Response> {
    Response::from_json(&ErrorResponse { error: msg.to_string() })
        .map(|r| r.with_status(status))
}
