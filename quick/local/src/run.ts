// Run a single spec through the PipelineAgent (translate + compile PDF).
//
// Usage:  QUICK_AGENT_HOST=localhost:8787 npx tsx src/run.ts GATE
//         mise run run -- GATE
//
// Connects to the Agent, sends one pipeline message, waits for translated + pdf
// events, writes results to disk, then exits.

import { AgentClient } from "agents/client";
import { createHash } from "crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "fs";
import { basename, join } from "path";

const HOST      = process.env.QUICK_AGENT_HOST ?? "localhost:8787";
const NAME      = process.env.QUICK_AGENT_NAME ?? "default";
const SPECS_DIR = process.env.QUICK_SPECS_DIR  ?? "specs";
const OUT_DIR   = process.env.QUICK_OUT_DIR    ?? "out";

const specName = process.argv[2];
if (!specName) {
  console.error("Usage: npx tsx src/run.ts SPECNAME");
  process.exit(1);
}

const specFile = join(SPECS_DIR, `${specName}.md`);
if (!existsSync(specFile)) {
  console.error(`Spec not found: ${specFile}`);
  process.exit(1);
}

const content = readFileSync(specFile, "utf-8");
console.log(`[run] ${specName}.md → pipeline-agent @ ${HOST}`);

const client = new AgentClient({
  agent: "PipelineAgent",
  name: NAME,
  host: HOST,
});

let done = false;

client.addEventListener("open", () => {
  console.log("[run] connected, sending pipeline message…");
  client.send(JSON.stringify({ type: "pipeline", name: specName, content }));
});

client.addEventListener("message", (e) => {
  let msg: Record<string, string>;
  try { msg = JSON.parse((e as MessageEvent).data as string); } catch { return; }

  switch (msg.type) {
    case "progress":
      console.log(`  ${msg.message}`);
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
      console.log(`  skip ${msg.name} (unchanged)`);
      done = true;
      break;

    case "pdf": {
      mkdirSync(OUT_DIR, { recursive: true });
      const pdfPath = join(OUT_DIR, `${msg.name}.th.pdf`);
      writeFileSync(pdfPath, Buffer.from(msg.bytes, "base64"));
      console.log(`  ✓ ${msg.name}.th.pdf saved`);
      done = true;
      break;
    }

    case "error":
      console.error(`  error: ${msg.message}`);
      done = true;
      break;
  }

  if (done) {
    client.close();
    process.exit(0);
  }
});

client.addEventListener("error", (e) => {
  console.error("[run] error:", e);
  process.exit(1);
});

// Timeout — SEA-LION 27B can be slow on first call
setTimeout(() => {
  console.error("[run] timeout waiting for agent response");
  process.exit(1);
}, 120_000);

function sha256(text: string): string {
  return createHash("sha256").update(text, "utf-8").digest("hex");
}
