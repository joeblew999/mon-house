// CF Workers AI backend — uses the AI binding declared in wrangler.toml.
// Model is controlled by QUICK_CF_MODEL (default: llama-3.3-70b-instruct-fp8-fast).
// No external API key required — runs entirely within Cloudflare.

import { generateText } from "ai";
import { createWorkersAI } from "workers-ai-provider";
import type { TranslateBackend } from "./index";
import { promptFor, userPromptFor, cleanOutput, type TranslateMode } from "../prompt";

export class WorkersAIBackend implements TranslateBackend {
  async translate(content: string, env: Env, mode: TranslateMode = "spec"): Promise<string> {
    const workersai = createWorkersAI({ binding: env.AI });
    const model = (env.QUICK_CF_MODEL as string) ?? "@cf/meta/llama-3.3-70b-instruct-fp8-fast";
    const { text } = await generateText({
      model: workersai(model),
      system: promptFor(mode),
      prompt: userPromptFor(mode, content),
      // Labels are single short phrases — keep cap small to avoid run-on output.
      // Specs need more headroom but the underlying CF model still has its own cap.
      maxOutputTokens: mode === "label" ? 256 : 4096,
    });
    return cleanOutput(text);
  }
}
