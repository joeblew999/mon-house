// CF Workers AI backend — uses the AI binding declared in wrangler.toml.
// Model is controlled by QUICK_CF_MODEL_TRANSLATE (set in wrangler.toml).
// No external API key required — runs entirely within Cloudflare.

import { generateText } from "ai";
import { createWorkersAI } from "workers-ai-provider";
import type { TranslateBackend } from "./index";
import { promptFor, userPromptFor, cleanOutput, type TranslateMode } from "../prompt";

export class WorkersAIBackend implements TranslateBackend {
  async translate(content: string, env: Env, mode: TranslateMode = "spec"): Promise<string> {
    const workersai = createWorkersAI({ binding: env.AI });
    const model = (env.QUICK_CF_MODEL_TRANSLATE as string) ?? "@cf/aisingapore/gemma-sea-lion-v4-27b-it";
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
