// Translation entry point — delegates to the configured backend.
//
// Backend is selected by QUICK_AI_BACKEND (wrangler.toml [vars]):
//   "vercel"        (default) — Vercel AI SDK; Claude or CF Workers AI
//   "openai-agents"           — @openai/agents SDK; supports handoffs + tools

import { resolveBackend } from "./backends/index";
import type { TranslateMode } from "./prompt";

export async function translate(content: string, env: Env, mode: TranslateMode = "spec"): Promise<string> {
  const backend = await resolveBackend(env);
  return backend.translate(content, env, mode);
}
