// TypstCompiler — Cloudflare Container that runs quick-tool serve.
//
// The Container base class from @cloudflare/containers proxies HTTP requests
// to the Docker container running Dockerfile.compiler.  One shared instance
// ("shared") handles all compile requests; it wakes on demand and sleeps after
// inactivity.
//
// Endpoints (implemented by quick-tool serve inside the container):
//   POST /compile  { name: string, content: string } → application/pdf
//   GET  /health                                     → 200 "ok"

import { Container } from "@cloudflare/containers";

export class TypstCompiler extends Container<Env> {
  /** Port that quick-tool serve listens on inside the container. */
  defaultPort = 8080;

  /** Sleep the container after 10 minutes of no requests. */
  sleepAfter = "10m";

  /**
   * Override fetch to extend the cold-start wait to 90 s.
   *
   * The @cloudflare/containers base class only polls for 8 s after start().
   * CF Container cold starts (first provisioning, slot scheduling) can take
   * much longer, so we implement our own start + poll loop before handing
   * off to containerFetch (which then waits for port 8080 to open).
   */
  override async fetch(request: Request): Promise<Response> {
    const COLD_START_TIMEOUT_MS = 90_000;
    const POLL_INTERVAL_MS = 1_000;

    const ctr = this.ctx.container!;
    console.log("[TypstCompiler] fetch called, ctx.container:", typeof ctr, "running:", ctr.running);

    if (!ctr.running) {
      console.log("[TypstCompiler] starting container…");
      ctr.start();
      const deadline = Date.now() + COLD_START_TIMEOUT_MS;
      while (!ctr.running) {
        if (Date.now() > deadline) {
          return new Response(
            "TypstCompiler container failed to start within 90 s",
            { status: 503 }
          );
        }
        await scheduler.wait(POLL_INTERVAL_MS);
        console.log("[TypstCompiler] polling… running:", ctr.running, "elapsed:", Date.now() - (deadline - COLD_START_TIMEOUT_MS), "ms");
      }
    }

    console.log("[TypstCompiler] container running, forwarding request");
    // Container is running — proxy to quick-tool serve on port 8080.
    return this.containerFetch(request, 8080);
  }
}
