// Vercel AI SDK backend — original implementation.
// Picks Anthropic Claude if ANTHROPIC_API_KEY is set, otherwise CF Workers AI.

import { generateText } from "ai";
import { createWorkersAI } from "workers-ai-provider";
import { createAnthropic } from "@ai-sdk/anthropic";
import type { TranslateBackend } from "./index";
import { promptFor, userPromptFor, cleanOutput, type TranslateMode } from "../prompt";

export class VercelAIBackend implements TranslateBackend {
  async translate(content: string, env: Env, mode: TranslateMode = "spec"): Promise<string> {
    const system = promptFor(mode);
    const prompt = userPromptFor(mode, content);

    if (env.ANTHROPIC_API_KEY) {
      const anthropic = createAnthropic({ apiKey: env.ANTHROPIC_API_KEY });
      const { text } = await generateText({
        model: anthropic("claude-opus-4-6"),
        system,
        prompt,
      });
      return cleanOutput(text);
    }

    const workersai = createWorkersAI({ binding: env.AI });
    const model = (env.QUICK_CF_MODEL_TRANSLATE as string) ?? "@cf/aisingapore/gemma-sea-lion-v4-27b-it";
    const { text } = await generateText({
      model: workersai(model),
      system,
      prompt,
    });
    return cleanOutput(text);
  }
}
