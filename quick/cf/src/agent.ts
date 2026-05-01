// PipelineAgent — Durable Object backed by @cloudflare/agents Workspace.
//
// One DO instance per named workspace (e.g. per project).
// WebSocket connections carry ClientMessage commands from the local Rust CLI,
// CI, web UI, or any other client.
//
// All pipeline events are BROADCAST to every connected client so that multiple
// watchers, a web UI, and CI all see the same progress in real time.
//
// Pipeline:
//   upload spec → hash check → translate → store .th.md → TODO: Typst WASM → PDF

import { Agent, callable, type Connection } from "agents";
// import { getContainer } from "@cloudflare/containers"; // disabled — see runCompile
import { Workspace, type FileInfo } from "@cloudflare/shell";
import { translate } from "./translate";
import type { ClientMessage, AgentEvent } from "./types";

export class PipelineAgent extends Agent<Env> {
  workspace = new Workspace({
    sql: this.ctx.storage.sql,
    r2:  this.env.SPECS_BUCKET,
    name: () => this.name,
  });

  // ── WebSocket: messages from any connected client ───────────────────────────

  async onMessage(conn: Connection, raw: string) {
    let msg: ClientMessage;
    try {
      msg = JSON.parse(raw) as ClientMessage;
    } catch {
      this.send(conn, { type: "error", message: "invalid JSON" });
      return;
    }

    switch (msg.type) {
      case "translate":
        await this.runTranslate(msg.name, msg.content);
        break;
      case "pipeline":
        await this.runTranslate(msg.name, msg.content);
        await this.runCompile(msg.name);
        break;
      case "fetch-pdf":
        await this.servePdf(conn, msg.name);
        break;
      default:
        this.send(conn, { type: "error", message: `unknown message type` });
    }
  }

  // ── Core pipeline steps ─────────────────────────────────────────────────────

  private async runTranslate(name: string, content: string) {
    this.emit({ type: "progress", message: `translating ${name}.md…` });

    const specPath  = `/specs/${name}.md`;
    const thPath    = `/specs/${name}.th.md`;
    const hashPath  = `/specs/${name}.th.md.hash`;

    // Write incoming spec into workspace
    await this.workspace.writeFile(specPath, content);

    // Hash check — skip if unchanged
    const currentHash = await sha256(content);
    const storedHash  = await this.workspace.readFile(hashPath);
    if (storedHash?.trim() === currentHash) {
      const cached = await this.workspace.readFile(thPath);
      if (cached) {
        this.emit({ type: "skipped", name });
        this.emit({ type: "translated", name, content: cached });
        return;
      }
    }

    // Translate
    try {
      const thai = await translate(content, this.env);
      await this.workspace.writeFile(thPath, thai);
      await this.workspace.writeFile(hashPath, currentHash);
      this.emit({ type: "translated", name, content: thai });
    } catch (e) {
      this.emit({ type: "error", message: String(e) });
    }
  }

  private async runCompile(name: string) {
    // PDF compile not implemented on the Worker. PDFs are built by GitHub
    // Actions on push to main and published to the specs-latest release.
    // Future direction (see CLAUDE.md): typst-WASM on client (and possibly
    // also on the Worker) — wasm-typst-studio-rs as the reference.
    this.emit({
      type: "error",
      message: `PDF compile not available on this Worker (${name}). PDFs come from GitHub Actions specs-latest release.`,
    });
  }
  // (former Container-based runCompile removed — see git history for the body.)

  private async servePdf(conn: Connection, name: string) {
    const pdfPath = `/pdfs/${name}.pdf`;
    const bytes = await this.workspace.readFileBytes(pdfPath);
    if (!bytes) {
      this.send(conn, { type: "error", message: `PDF not found: ${name}` });
      return;
    }
    const b64 = btoa(String.fromCharCode(...new Uint8Array(bytes as unknown as ArrayBuffer)));
    // PDF bytes are large and specific to the requester — direct send only
    this.send(conn, { type: "pdf", name, bytes: b64 });
  }

  // ── RPC (@callable) — usable from wrangler / tests / CLI ──────────────────

  @callable()
  async listSpecs(): Promise<FileInfo[]> {
    return this.workspace.glob("/specs/[A-Z]*.md");
  }

  @callable()
  async getSpec(name: string): Promise<string | null> {
    return this.workspace.readFile(`/specs/${name}.md`);
  }

  @callable()
  async getTranslation(name: string): Promise<string | null> {
    return this.workspace.readFile(`/specs/${name}.th.md`);
  }

  @callable()
  async deleteSpec(name: string): Promise<void> {
    await this.workspace.deleteFile(`/specs/${name}.md`);
    await this.workspace.deleteFile(`/specs/${name}.th.md`);
    await this.workspace.deleteFile(`/specs/${name}.th.md.hash`);
  }

  // ── Helpers ─────────────────────────────────────────────────────────────────

  /** Send an event to ONE specific connection (e.g. large PDF bytes). */
  private send(conn: Connection, event: AgentEvent) {
    conn.send(JSON.stringify(event));
  }

  /** Broadcast an event to ALL connected clients — watchers, web UI, CI. */
  private emit(event: AgentEvent) {
    this.broadcast(JSON.stringify(event));
  }
}

// SHA-256 via Web Crypto — available in all Workers runtimes
async function sha256(text: string): Promise<string> {
  const data = new TextEncoder().encode(text);
  const buf  = await crypto.subtle.digest("SHA-256", data);
  return Array.from(new Uint8Array(buf))
    .map(b => b.toString(16).padStart(2, "0"))
    .join("");
}
