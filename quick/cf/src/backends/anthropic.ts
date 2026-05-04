// Anthropic Claude backend — requires ANTHROPIC_API_KEY secret.
// Set via: wrangler secret put ANTHROPIC_API_KEY
// Local dev: add ANTHROPIC_API_KEY=sk-ant-... to cf/.dev.vars (gitignored)

import { generateText } from "ai";
import { createAnthropic } from "@ai-sdk/anthropic";
import type { TranslateBackend } from "./index";
import { promptFor, userPromptFor, cleanOutput, type TranslateMode } from "../prompt";

export class AnthropicBackend implements TranslateBackend {
  async translate(content: string, env: Env, mode: TranslateMode = "spec"): Promise<string> {
    const anthropic = createAnthropic({ apiKey: env.ANTHROPIC_API_KEY });
    const model = (env.QUICK_CLAUDE_MODEL as string) ?? "claude-opus-4-6";
    const { text } = await generateText({
      model: anthropic(model),
      system: promptFor(mode),
      prompt: userPromptFor(mode, content),
    });
    return cleanOutput(text);
  }
}
