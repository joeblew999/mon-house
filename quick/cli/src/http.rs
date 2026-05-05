/// Outbound HTTP — single abstraction layer for all network I/O.
///
/// ## Implementations
///
/// | Feature         | Backend          | Target          |
/// |-----------------|------------------|-----------------|
/// | `native-http`   | `ureq` (sync)    | native (CLI)    |
/// | *(none)*        | stubs (panic)    | WASM / CF       |
///
/// When targeting Cloudflare Workers, swap this file for one backed by
/// `worker::Fetch` (async). No other file changes required — all callers
/// go through this module.
///
/// ## What belongs here
/// - GET (bytes, JSON)
/// - POST JSON with custom headers
///
/// ## What does NOT belong here
/// - URL construction — caller's responsibility
/// - Response-body logging — caller prints what it wants to show the user

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

#[cfg(feature = "native-http")]
use anyhow::Context;

#[cfg(feature = "native-http")]
const USER_AGENT: &str = "quick-tool/0.2";

// ── GET ────────────────────────────────────────────────────────────────────────

/// Download a URL as raw bytes.
#[cfg(feature = "native-http")]
pub fn get_bytes(url: &str) -> Result<Vec<u8>> {
    use std::io::Read;
    let resp = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .with_context(|| format!("GET {url}"))?;
    let mut bytes = Vec::new();
    resp.into_reader().read_to_end(&mut bytes)?;
    Ok(bytes)
}

#[cfg(not(feature = "native-http"))]
pub fn get_bytes(_url: &str) -> Result<Vec<u8>> {
    anyhow::bail!("native-http feature required for HTTP calls (not available in WASM builds)")
}

/// GET a URL and deserialise the JSON response body.
#[cfg(feature = "native-http")]
pub fn get_json<T: DeserializeOwned>(url: &str) -> Result<T> {
    let resp = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .with_context(|| format!("GET {url}"))?;
    resp.into_json::<T>().context("deserialising JSON response")
}

#[cfg(not(feature = "native-http"))]
pub fn get_json<T: DeserializeOwned>(_url: &str) -> Result<T> {
    anyhow::bail!("native-http feature required for HTTP calls (not available in WASM builds)")
}

// ── POST ───────────────────────────────────────────────────────────────────────

/// POST a JSON body, set extra headers, and deserialise the JSON response.
///
/// `headers` is a slice of `(name, value)` pairs added after `User-Agent`.
/// Intended for API calls that need auth headers (e.g. `x-api-key`).
///
/// **Transient errors are retried** with exponential backoff (2s → 4s → 8s,
/// up to MAX_RETRIES total attempts). Transient = HTTP 408/429/5xx or any
/// transport-level failure (timeout, connection reset). Non-transient errors
/// (4xx other than 408/429, malformed responses) bail immediately.
#[cfg(feature = "native-http")]
pub fn post_json<B: Serialize, R: DeserializeOwned>(
    url: &str,
    headers: &[(&str, &str)],
    body: &B,
) -> Result<R> {
    const MAX_ATTEMPTS: u32 = 3;
    let mut delay = std::time::Duration::from_secs(2);

    for attempt in 1..=MAX_ATTEMPTS {
        let mut req = ureq::post(url).set("User-Agent", USER_AGENT);
        for (k, v) in headers {
            req = req.set(k, v);
        }
        match req.send_json(body) {
            Ok(resp) => return resp.into_json::<R>().context("deserialising JSON response"),
            Err(e) if attempt < MAX_ATTEMPTS && is_transient(&e) => {
                eprintln!(
                    "  http {url}: transient error ({e}) on attempt {attempt}/{MAX_ATTEMPTS}, retrying in {delay:?}..."
                );
                std::thread::sleep(delay);
                delay *= 2;
            }
            Err(e) => {
                return Err(anyhow::Error::from(e).context(format!("POST {url}")));
            }
        }
    }
    unreachable!("loop exits via Ok or final Err");
}

/// Classify a ureq error as worth retrying.
#[cfg(feature = "native-http")]
fn is_transient(err: &ureq::Error) -> bool {
    match err {
        ureq::Error::Status(code, _) => matches!(*code, 408 | 425 | 429 | 500 | 502 | 503 | 504),
        ureq::Error::Transport(_) => true,
    }
}

#[cfg(not(feature = "native-http"))]
pub fn post_json<B: Serialize, R: DeserializeOwned>(
    _url: &str,
    _headers: &[(&str, &str)],
    _body: &B,
) -> Result<R> {
    anyhow::bail!("native-http feature required for HTTP calls (not available in WASM builds)")
}
