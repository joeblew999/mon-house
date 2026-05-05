// Worker entry point — Hono router.
//
// Routes:
//   GET  /health          — health check
//   POST /translate       — translate EN → Thai. JSON in: {content, mode?},
//                           JSON out: {thai}. Used by the Rust CLI's
//                           QUICK_TRANSLATE_URL backend
//                           (see cli/src/translate.rs::call_worker).
//   ALL  /agents/*        — delegated to routeAgentRequest. Currently hosts
//                           the spec-copilot ChatAgent (see ./chat.ts);
//                           AIChatAgent gives us a WebSocket chat UI that
//                           the React SPA's useAgentChat hook drives.
//   *                     — wrangler assets serves the React SPA

import { Hono } from "hono";
import { routeAgentRequest } from "agents";
import { ChatAgent } from "./chat";
import { translate } from "./translate";

export { ChatAgent };

const app = new Hono<{ Bindings: Env }>();

app.get("/health", (c) => c.json({ ok: true, ts: Date.now() }));

/**
 * Per-IP rate-limit middleware for AI surfaces. Looks up the caller via
 * CF-Connecting-IP, checks the RATE_LIMIT binding (configured in
 * wrangler.toml at 60/min/IP), returns 429 if exceeded. Defends the
 * Workers AI budget against unbounded public access — see COSTS.md.
 */
async function checkRateLimit(c: { req: { raw: Request }; env: Env }) {
  const ip = c.req.raw.headers.get("CF-Connecting-IP") ?? "unknown";
  const { success } = await c.env.RATE_LIMIT.limit({ key: ip });
  return success;
}

// Plain HTTP translate endpoint — what the Rust CLI's QUICK_TRANSLATE_URL hits.
// Body shape is fixed by cli/src/translate.rs::WorkerRequest / WorkerResponse:
//   request:  { "content": "<en markdown>", "mode"?: "spec" | "label" }
//   response: { "thai":    "<th markdown>" }
app.post("/translate", async (c) => {
  if (!(await checkRateLimit(c))) {
    return c.json({ error: "rate limit exceeded — see COSTS.md" }, 429);
  }
  const body = await c.req.json<{ content?: string; mode?: "spec" | "label" }>();
  if (typeof body?.content !== "string" || body.content.length === 0) {
    return c.json({ error: "missing 'content' string in request body" }, 400);
  }
  const mode = body.mode === "label" ? "label" : "spec";
  try {
    const thai = await translate(body.content, c.env, mode);
    return c.json({ thai });
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    return c.json({ error: msg }, 500);
  }
});

// All /agents/* routes go to the SDK's routeAgentRequest, which dispatches
// WebSocket upgrades to the right Durable Object. Today: ChatAgent.
// Rate-limit on the upgrade hop so a flood of new connections can't burn
// the AI budget; the WS itself stays cheap once established.
app.all("/agents/*", async (c) => {
  if (!(await checkRateLimit(c))) {
    return c.json({ error: "rate limit exceeded — see COSTS.md" }, 429);
  }
  const response = await routeAgentRequest(c.req.raw, c.env);
  return response ?? c.notFound();
});

export default app;
