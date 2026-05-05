// ChatAgent — Cloudflare AIChatAgent with Workers AI as the model backend.
//
// Provides a per-user chat session over WebSocket: message history persists
// in the Durable Object's SQLite storage automatically (the SDK handles it),
// streaming completions stream back to the React client, reconnect-on-close
// is handled by `useAgentChat`.
//
// Use case in this repo: a "spec copilot" attached to the React SPA that
// helps users draft / edit construction-spec markdown in plain English. The
// AI sees the conversation history and (eventually) the spec content to
// answer questions like "rewrite the bidet section more concisely" or
// "what's still TBC in this spec?".
//
// Today the system prompt is generic ("you are a helpful assistant for the
// quick/ construction-spec project"). Future work: pipe in the active spec
// content + parts list as context for grounded suggestions.

// Direct import from @cloudflare/ai-chat — `agents/ai-chat-agent` is a
// deprecated re-export shim in agents 0.12+ that points here anyway.
import { AIChatAgent } from "@cloudflare/ai-chat";
import { generateText } from "ai";
import { createWorkersAI } from "workers-ai-provider";

const SYSTEM_PROMPT = `You are a helpful assistant for the quick/ construction-spec project.

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
  /**
   * Called by the SDK on every new user message. We send the full message
   * history (already maintained for us) to Workers AI and stream the reply
   * back to the React client.
   */
  async onChatMessage() {
    const workersai = createWorkersAI({ binding: this.env.AI });
    const model = (this.env.QUICK_CF_MODEL as string) ?? "@cf/meta/llama-3.3-70b-instruct-fp8-fast";

    const { text } = await generateText({
      model: workersai(model),
      system: SYSTEM_PROMPT,
      messages: this.messages.map((m) => ({ role: m.role, content: m.content })),
      maxOutputTokens: 4096,
    });

    return text;
  }
}
