// AI backend abstraction — swap by setting QUICK_AI_BACKEND in wrangler.toml [vars].
//
// Backends:
//   "workers-ai"   (default) — CF Workers AI binding; no external key needed.
//   "anthropic"              — Anthropic Claude; requires ANTHROPIC_API_KEY secret.
//   "openai-agents"          — @openai/agents SDK; supports handoffs, tools, multi-agent.
//                              Requires OPENAI_API_KEY secret.
//   "vercel"                 — Vercel AI SDK (legacy); auto-picks anthropic or workers-ai.

import type { TranslateMode } from "../prompt";

export interface TranslateBackend {
  translate(content: string, env: Env, mode?: TranslateMode): Promise<string>;
}

export async function resolveBackend(env: Env): Promise<TranslateBackend> {
  const name = (env as unknown as Record<string, string>).QUICK_AI_BACKEND ?? "workers-ai";
  switch (name) {
    case "workers-ai": {
      const { WorkersAIBackend } = await import("./workers-ai");
      return new WorkersAIBackend();
    }
    case "anthropic": {
      const { AnthropicBackend } = await import("./anthropic");
      return new AnthropicBackend();
    }
    case "openai-agents": {
      const { OpenAIAgentsBackend } = await import("./openai-agents");
      return new OpenAIAgentsBackend();
    }
    default: {
      const { VercelAIBackend } = await import("./vercel");
      return new VercelAIBackend();
    }
  }
}
