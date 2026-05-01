// CF Workers AI backend — uses the AI binding declared in wrangler.toml.
// Model is controlled by QUICK_CF_MODEL (default: llama-3.3-70b-instruct-fp8-fast).
// No external API key required — runs entirely within Cloudflare.

import { generateText } from "ai";
import { createWorkersAI } from "workers-ai-provider";
import type { TranslateBackend } from "./index";
import { SYSTEM_PROMPT, cleanOutput } from "../prompt";

export class WorkersAIBackend implements TranslateBackend {
  async translate(content: string, env: Env): Promise<string> {
    const workersai = createWorkersAI({ binding: env.AI });
    const model = (env.QUICK_CF_MODEL as string) ?? "@cf/meta/llama-3.3-70b-instruct-fp8-fast";
    const { text } = await generateText({
      model: workersai(model),
      system: SYSTEM_PROMPT,
      prompt: `Translate this construction spec to Thai:\n\n${content}`,
    });
    return cleanOutput(text);
  }
}
