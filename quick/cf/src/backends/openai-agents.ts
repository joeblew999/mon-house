// OpenAI Agents SDK backend.
//
// Uses @openai/agents Agent + run() so the pipeline can grow into multi-agent
// workflows (handoffs, tools, reviewers) without changing the caller.
//
// Requires OPENAI_API_KEY set as a wrangler secret.
//
// To add a new agent (e.g. a "review" agent after translation):
//   1. Define it below with `new Agent({ ... })`
//   2. Add it to the triageAgent handoffs array — routing is automatic.

import { Agent, run } from "@openai/agents";
import type { TranslateBackend } from "./index";
import { promptFor, userPromptFor, cleanOutput, type TranslateMode } from "../prompt";

export class OpenAIAgentsBackend implements TranslateBackend {
  async translate(content: string, _env: Env, mode: TranslateMode = "spec"): Promise<string> {
    // Translation agent — primary worker
    const translatorAgent = new Agent({
      name: "Thai Translator",
      instructions: promptFor(mode),
    });

    // Triage agent — extend handoffs here to add reviewers, language variants, etc.
    const triageAgent = new Agent({
      name: "Pipeline Triage",
      instructions: "Route construction spec or label translation tasks to the appropriate specialist agent.",
      handoffs: [translatorAgent],
    });

    const result = await run(triageAgent, userPromptFor(mode, content));

    return cleanOutput(result.finalOutput ?? "");
  }
}
