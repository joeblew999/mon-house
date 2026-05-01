// Worker entry point — Hono router.
//
// Routes:
//   GET  /health          — health check
//   ALL  /agents/*        — delegated to routeAgentRequest (PipelineAgent DO)
//   *                     — wrangler assets serves the React SPA

import { Hono } from "hono";
import { routeAgentRequest } from "agents";
import { PipelineAgent } from "./agent";

export { PipelineAgent };

const app = new Hono<{ Bindings: Env }>();

app.get("/health", (c) => c.json({ ok: true, ts: Date.now() }));

// All /agents/* routes go to the Agent DO via routeAgentRequest.
// routeAgentRequest handles WebSocket upgrades, routing to the right DO class,
// and the cf_agent protocol framing.
app.all("/agents/*", async (c) => {
  const response = await routeAgentRequest(c.req.raw, c.env);
  return response ?? c.notFound();
});

export default app;
