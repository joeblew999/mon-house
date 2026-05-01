// Vercel AI SDK backend — original implementation.
// Picks Anthropic Claude if ANTHROPIC_API_KEY is set, otherwise CF Workers AI.

import { generateText } from "ai";
import { createWorkersAI } from "workers-ai-provider";
import { createAnthropic } from "@ai-sdk/anthropic";
import type { TranslateBackend } from "./index";
import { SYSTEM_PROMPT, cleanOutput } from "../prompt";

export class VercelAIBackend implements TranslateBackend {
  async translate(content: string, env: Env): Promise<string> {
    const prompt = `Translate this construction spec to Thai:\n\n${content}`;

    if (env.ANTHROPIC_API_KEY) {
      const anthropic = createAnthropic({ apiKey: env.ANTHROPIC_API_KEY });
      const { text } = await generateText({
        model: anthropic("claude-opus-4-6"),
        system: SYSTEM_PROMPT,
        prompt,
      });
      return cleanOutput(text);
    }

    const workersai = createWorkersAI({ binding: env.AI });
    const model = (env.QUICK_CF_MODEL as string) ?? "@cf/meta/llama-3.3-70b-instruct-fp8-fast";
    const { text } = await generateText({
      model: workersai(model),
      system: SYSTEM_PROMPT,
      prompt,
    });
    return cleanOutput(text);
  }
}
