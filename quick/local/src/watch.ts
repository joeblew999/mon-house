// Local file watcher + CF PipelineAgent client.
//
// Watches specs/*.md for changes, sends each changed file to the PipelineAgent
// over WebSocket, and writes results back to disk:
//   translated → specs/{name}.th.md + specs/{name}.th.md.hash
//   pdf        → out/{name}.th.pdf
//
// Uses AgentClient (from agents/client, built on PartySocket) for:
//   - Auto-reconnection when the DO hibernates or wrangler restarts
//   - Message buffering while disconnected
//   - Protocol frame handling (cf_agent_identity, cf_agent_state, etc.)
//
// Config (env vars, matching mise.toml / mise.local.toml):
//   QUICK_AGENT_HOST   host[:port] of the Worker  (default: localhost:8787)
//   QUICK_AGENT_NAME   DO instance name            (default: default)
//   QUICK_SPECS_DIR    specs directory             (default: specs)
//   QUICK_OUT_DIR      PDF output directory        (default: out)

import { AgentClient } from "agents/client";
import chokidar from "chokidar";
import { createHash } from "crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "fs";
import { basename, join } from "path";

// ── Config ─────────────────────────────────────────────────────────────────────

const HOST      = process.env.QUICK_AGENT_HOST ?? "localhost:8787";
const NAME      = process.env.QUICK_AGENT_NAME ?? "default";
const SPECS_DIR = process.env.QUICK_SPECS_DIR  ?? "specs";
const OUT_DIR   = process.env.QUICK_OUT_DIR    ?? "out";

// ── AgentClient ────────────────────────────────────────────────────────────────

const client = new AgentClient({
  agent: "PipelineAgent",   // → kebab: pipeline-agent → /agents/pipeline-agent/{name}
  name: NAME,
  host: HOST,
});

client.addEventListener("open",  () => console.log(`[watch] connected  → ws://${HOST}/agents/pipeline-agent/${NAME}`));
client.addEventListener("close", () => console.log("[watch] disconnected (will auto-reconnect)"));
client.addEventListener("error", (e) => console.error("[watch] error:", e));

// ── Incoming events ────────────────────────────────────────────────────────────

client.addEventListener("message", (e) => {
  let msg: Record<string, string>;
  try { msg = JSON.parse((e as MessageEvent).data as string); } catch { return; }

  switch (msg.type) {
    case "progress":
      console.log(`  agent: ${msg.message}`);
      break;

    case "translated": {
      const thPath   = join(SPECS_DIR, `${msg.name}.th.md`);
      const hashPath = join(SPECS_DIR, `${msg.name}.th.md.hash`);
      writeFileSync(thPath, msg.content, "utf-8");
      writeFileSync(hashPath, sha256(msg.content), "utf-8");
      console.log(`  ✓ ${msg.name}.th.md written`);
      break;
    }

    case "skipped":
      console.log(`  skip ${msg.name} (agent cache hit)`);
      break;

    case "pdf": {
      mkdirSync(OUT_DIR, { recursive: true });
      const pdfPath = join(OUT_DIR, `${msg.name}.th.pdf`);
      writeFileSync(pdfPath, Buffer.from(msg.bytes, "base64"));
      console.log(`  ✓ ${msg.name}.th.pdf saved`);
      break;
    }

    case "error":
      console.error(`  agent error: ${msg.message}`);
      break;
  }
});

// ── File watcher ───────────────────────────────────────────────────────────────

if (!existsSync(SPECS_DIR)) {
  console.error(`[watch] specs dir not found: ${SPECS_DIR}`);
  process.exit(1);
}

console.log(`[watch] watching ${SPECS_DIR}/*.md`);
console.log(`[watch] PDFs → ${OUT_DIR}/`);
console.log("[watch] (Ctrl+C to stop)\n");

const watcher = chokidar.watch(`${SPECS_DIR}/[A-Z]*.md`, {
  ignoreInitial: true,
  ignored: /\.th\.md$/,
});

watcher.on("change", (filePath: string) => {
  const name    = basename(filePath, ".md");
  const content = readFileSync(filePath, "utf-8");

  // Local hash check — skip before hitting the network
  const hashPath = join(SPECS_DIR, `${name}.th.md.hash`);
  const thPath   = join(SPECS_DIR, `${name}.th.md`);
  if (existsSync(hashPath) && existsSync(thPath)) {
    const stored = readFileSync(hashPath, "utf-8").trim();
    if (stored === sha256(content)) {
      console.log(`↺  ${name}.md  skip (unchanged)`);
      return;
    }
  }

  console.log(`↺  ${name}.md  → pipeline`);
  client.send(JSON.stringify({ type: "pipeline", name, content }));
});

watcher.on("add", (filePath: string) => {
  const name = basename(filePath, ".md");
  if (!filePath.endsWith(".th.md")) {
    console.log(`[watch] new spec detected: ${name}.md`);
  }
});

// ── Helpers ────────────────────────────────────────────────────────────────────

function sha256(text: string): string {
  return createHash("sha256").update(text, "utf-8").digest("hex");
}
