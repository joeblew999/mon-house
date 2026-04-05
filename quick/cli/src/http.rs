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
#[cfg(feature = "native-http")]
pub fn post_json<B: Serialize, R: DeserializeOwned>(
    url: &str,
    headers: &[(&str, &str)],
    body: &B,
) -> Result<R> {
    let mut req = ureq::post(url).set("User-Agent", USER_AGENT);
    for (k, v) in headers {
        req = req.set(k, v);
    }
    let resp = req
        .send_json(body)
        .with_context(|| format!("POST {url}"))?;
    resp.into_json::<R>().context("deserialising JSON response")
}

#[cfg(not(feature = "native-http"))]
pub fn post_json<B: Serialize, R: DeserializeOwned>(
    _url: &str,
    _headers: &[(&str, &str)],
    _body: &B,
) -> Result<R> {
    anyhow::bail!("native-http feature required for HTTP calls (not available in WASM builds)")
}
