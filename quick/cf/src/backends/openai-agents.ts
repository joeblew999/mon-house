// OpenAI Agents SDK backend.
//
// Uses @openai/agents Agent + run() so the pipeline can grow into multi-agent
// workflows (handoffs, tools, reviewers) without changing the caller.
//
// Requires OPENAI_API_KEY set as a wrangler secret.
//
// To add a new agent (e.g. a "review" agent after translation):
//   1. Define it below with `new Agent({ ... })`
//   2. Add it to the triageAgent handoffs array
//   3. The triage agent routes automatically — no changes needed in agent.ts

import { Agent, run } from "@openai/agents";
import type { TranslateBackend } from "./index";
import { SYSTEM_PROMPT, cleanOutput } from "../prompt";

export class OpenAIAgentsBackend implements TranslateBackend {
  async translate(content: string, _env: Env): Promise<string> {
    // Translation agent — primary worker
    const translatorAgent = new Agent({
      name: "Thai Translator",
      instructions: SYSTEM_PROMPT,
    });

    // Triage agent — extend handoffs here to add reviewers, language variants, etc.
    const triageAgent = new Agent({
      name: "Pipeline Triage",
      instructions: "Route construction spec translation tasks to the appropriate specialist agent.",
      handoffs: [translatorAgent],
    });

    const result = await run(
      triageAgent,
      `Translate this construction spec to Thai:\n\n${content}`,
    );

    return cleanOutput(result.finalOutput ?? "");
  }
}
