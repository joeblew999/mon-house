/// Outbound HTTP — single abstraction layer for all network I/O.
///
/// Currently backed by `ureq` (sync). When targeting Cloudflare Workers,
/// swap the implementations here — no other file changes required.
///
/// ## What belongs here
/// - GET (bytes, JSON)
/// - POST JSON with custom headers
///
/// ## What does NOT belong here
/// - URL construction — caller's responsibility
/// - Response-body logging — caller prints what it wants to show the user
use std::io::Read;

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};

const USER_AGENT: &str = "quick-tool/0.2";

// ── GET ────────────────────────────────────────────────────────────────────────

/// Download a URL as raw bytes.
pub fn get_bytes(url: &str) -> Result<Vec<u8>> {
    let resp = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .with_context(|| format!("GET {url}"))?;
    let mut bytes = Vec::new();
    resp.into_reader().read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// GET a URL and deserialise the JSON response body.
pub fn get_json<T: DeserializeOwned>(url: &str) -> Result<T> {
    let resp = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .with_context(|| format!("GET {url}"))?;
    resp.into_json::<T>().context("deserialising JSON response")
}

// ── POST ───────────────────────────────────────────────────────────────────────

/// POST a JSON body, set extra headers, and deserialise the JSON response.
///
/// `headers` is a slice of `(name, value)` pairs added after `User-Agent`.
/// Intended for API calls that need auth headers (e.g. `x-api-key`).
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
