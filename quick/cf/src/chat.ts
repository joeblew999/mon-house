// ChatAgent — Cloudflare AIChatAgent with Workers AI as the model backend.
//
// Provides a per-user chat session over WebSocket: message history persists
// in the Durable Object's SQLite storage automatically (the SDK handles it),
// streaming completions stream back to the React client, reconnect-on-close
// is handled by `useAgentChat`.
//
// ## Dep-graph event awareness
//
// The SPA's WASM-hosted Rust engine emits structured events when the user
// (or AI) queries the dependency graph — e.g. "what specs include
// _partials/paint-metal.md?". Those events arrive as custom WebSocket
// messages of type `dep-graph-event`. We persist them in a SQLite table and
// inject the most recent ones into the system prompt so the AI has live
// awareness of "what's the user looking at right now?" without needing to
// be told explicitly. The AI is a passive subscriber — it reads the events
// to ground its replies, never edits files autonomously.

// Direct import from @cloudflare/ai-chat — `agents/ai-chat-agent` is a
// deprecated re-export shim in agents 0.12+ that points here anyway.
import { AIChatAgent } from "@cloudflare/ai-chat";
import type { Connection, WSMessage } from "agents";
import { convertToModelMessages, streamText } from "ai";
import { createWorkersAI } from "workers-ai-provider";

/** Maximum number of dep-graph events kept in DO state per agent instance. */
const MAX_EVENTS = 20;

/** Custom WebSocket message type emitted by the SPA's WASM engine. */
type DepGraphEventMessage = {
  type: "dep-graph-event";
  event: DepGraphEvent;
};

/** Shape of a single dep-graph event as stored and reported to the AI. */
type DepGraphEvent = {
  /** What kind of engine action produced this — for now, only `partial-queried`. */
  kind: "partial-queried";
  /** Project-relative path of the file in question (e.g. `specs/_partials/paint-metal.md`). */
  path: string;
  /** Specs that include the queried partial. Empty list = no dependents. */
  dependents: string[];
  /** ISO 8601 timestamp set by the SPA at emit time. */
  ts: string;
};

const BASE_SYSTEM_PROMPT = `You are a helpful assistant for the quick/ construction-spec project.

The user is editing markdown specs for a renovation project in Thailand. You help them:
- draft and edit spec sections in plain English
- refine product picks (basin, toilet, tiles, etc.) with attention to dimensions and fit
- understand existing specs they paste in
- spot inconsistencies or ambiguities a builder would catch

Style:
- direct, concise, conversational
- markdown-friendly when the user is editing markdown
- ask one focused clarifying question if intent is genuinely ambiguous (otherwise just answer)
- short responses by default; expand only when asked
`;

export class ChatAgent extends AIChatAgent<Env> {
  // Lazy schema setup — runs once on first SQL access. The AIChatAgent base
  // class already creates its own tables in onStart; we add ours alongside.
  private schemaReady = false;
  private ensureSchema() {
    if (this.schemaReady) return;
    this.sql`create table if not exists dep_events (
      id integer primary key autoincrement,
      kind text not null,
      path text not null,
      dependents text not null,
      ts text not null
    )`;
    this.schemaReady = true;
  }

  /**
   * Custom WebSocket messages bypass the chat protocol and land here. We
   * handle `dep-graph-event` for engine-emitted notifications; everything
   * else is forwarded to the parent class's default handling.
   */
  async onMessage(connection: Connection, message: WSMessage) {
    if (typeof message === "string") {
      try {
        const parsed = JSON.parse(message) as { type?: string };
        if (parsed.type === "dep-graph-event") {
          const { event } = parsed as DepGraphEventMessage;
          this.ensureSchema();
          this.sql`insert into dep_events (kind, path, dependents, ts) values (
            ${event.kind}, ${event.path}, ${JSON.stringify(event.dependents)}, ${event.ts}
          )`;
          // Trim ring buffer — keep only the most recent MAX_EVENTS rows.
          this.sql`delete from dep_events where id not in (
            select id from dep_events order by id desc limit ${MAX_EVENTS}
          )`;
          console.log("[chat] dep-graph event recorded:", event.kind, event.path);
          return; // Don't forward to parent — this is our message type.
        }
      } catch {
        // Not JSON or not our shape — fall through to default handling.
      }
    }
    return super.onMessage(connection, message);
  }

  /**
   * Read recent dep-graph events from SQLite for system-prompt injection.
   * Newest first; defaults to 5 since older events get progressively less
   * relevant to the current chat turn.
   */
  private recentEvents(limit = 5): DepGraphEvent[] {
    this.ensureSchema();
    type Row = { kind: string; path: string; dependents: string; ts: string };
    const rows = this.sql<Row>`
      select kind, path, dependents, ts
      from dep_events
      order by id desc
      limit ${limit}
    `;
    return rows.map((r) => ({
      kind: r.kind as DepGraphEvent["kind"],
      path: r.path,
      dependents: JSON.parse(r.dependents) as string[],
      ts: r.ts,
    }));
  }

  /** Build the per-turn system prompt: base prompt + live event context. */
  private buildSystemPrompt(): string {
    const events = this.recentEvents();
    if (events.length === 0) return BASE_SYSTEM_PROMPT;

    const lines = events.map((e) => {
      if (e.kind === "partial-queried") {
        const deps = e.dependents.length > 0 ? e.dependents.join(", ") : "(none)";
        return `- ${e.ts}: user looked up ${e.path} → dependents: ${deps}`;
      }
      return `- ${e.ts}: ${e.kind} on ${e.path}`;
    });

    return `${BASE_SYSTEM_PROMPT}

## Live engine context

The user has been working with these files recently (newest first):

${lines.join("\n")}

Use this context to ground your answers — if the user asks "what did I just look at?" or "what should I review?", refer to these events. Don't invent files outside this list.
`;
  }

  /**
   * Called by the SDK on every new user message. We stream the assistant
   * response from Workers AI back to the React client; the AIChatAgent base
   * class handles persistence (DO SQLite), reconnection replay, and
   * dispatch to all connected clients of this agent instance.
   *
   * Model selection priority:
   *   1. `body.model` from the client (SPA dropdown override) — wins
   *   2. `QUICK_CF_MODEL_CHAT` from wrangler.toml [vars]
   *   3. Hard-coded fallback (gemma-3-12b-it — cheap, fluent enough)
   *
   * The SPA passes the user's pick via the AI SDK's `body` option in
   * `useAgentChat`; we surface it here through `options.body`.
   */
  async onChatMessage(_onFinish: unknown, options?: { body?: Record<string, unknown> }) {
    const workersai = createWorkersAI({ binding: this.env.AI });
    const requested = typeof options?.body?.model === "string" ? options.body.model : undefined;
    const model =
      requested ??
      this.env.QUICK_CF_MODEL_CHAT ??
      "@cf/google/gemma-3-12b-it";

    const systemPrompt = this.buildSystemPrompt();

    console.log("[chat] model:", model, "requested:", requested ?? "<none>",
      "events:", this.recentEvents().length);

    const result = streamText({
      model: workersai(model),
      system: systemPrompt,
      messages: await convertToModelMessages(this.messages),
    });

    return result.toUIMessageStreamResponse();
  }
}
