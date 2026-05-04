// Worker entry point — Hono router.
//
// Routes:
//   GET  /health          — health check
//   POST /translate       — translate EN → Thai. JSON in: {content}, JSON out: {thai}
//                           Used by the Rust CLI's `QUICK_TRANSLATE_URL` backend
//                           (see cli/src/translate.rs::call_worker).
//   ALL  /agents/*        — delegated to routeAgentRequest (PipelineAgent DO)
//   *                     — wrangler assets serves the React SPA

import { Hono } from "hono";
import { routeAgentRequest } from "agents";
import { PipelineAgent } from "./agent";
import { translate } from "./translate";

export { PipelineAgent };

const app = new Hono<{ Bindings: Env }>();

app.get("/health", (c) => c.json({ ok: true, ts: Date.now() }));

// Plain HTTP translate endpoint — what the Rust CLI's QUICK_TRANSLATE_URL hits.
// Body shape is fixed by cli/src/translate.rs::WorkerRequest / WorkerResponse:
//   request:  { "content": "<en markdown>" }
//   response: { "thai":    "<th markdown>" }
app.post("/translate", async (c) => {
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

// All /agents/* routes go to the Agent DO via routeAgentRequest.
// routeAgentRequest handles WebSocket upgrades, routing to the right DO class,
// and the cf_agent protocol framing.
app.all("/agents/*", async (c) => {
  const response = await routeAgentRequest(c.req.raw, c.env);
  return response ?? c.notFound();
});

export default app;
