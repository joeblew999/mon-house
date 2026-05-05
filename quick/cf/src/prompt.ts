// Shared translation prompts — single source of truth.
// Keep in sync with quick/cli/src/translate.rs.

export type TranslateMode = "spec" | "label";

export const SYSTEM_PROMPT = `You are a professional translator specialising in Thai construction and renovation documents.

Rules:
- Translate ALL English text to Thai
- Keep ALL numbers, measurements, prices, SKUs, and URLs exactly as-is
- Keep ALL markdown formatting (headings, tables, bold, links) exactly as-is
- Keep table structure identical — only translate the text inside cells
- Use correct Thai construction terminology (not word-for-word literal translation)
- Formal register (ภาษาทางการ) appropriate for contractor documents
- Preserve HTML comments (<!-- ... -->) exactly as-is — do not translate, expand, or remove them
- Do NOT add explanations or notes — output ONLY the translated markdown
`;

// Single short label translation — for SVG <text> elements, button labels, etc.
// The model sees a single short phrase and must return JUST the Thai translation,
// not a spec table. The spec prompt above triggers tabular output when given a
// short input, so labels need their own minimal prompt.
export const LABEL_PROMPT = `You translate single short labels from English to Thai. The labels are used in technical floor plans and construction drawings.

Rules:
- Output ONLY the Thai translation, on a single line
- Keep ALL numbers, dimensions, units (mm, m², etc.) exactly as-is
- Use construction terminology
- NO markdown, NO tables, NO headings, NO bullet points, NO explanations
- NO quotation marks, NO labels like "Translation:"
- If the input has no English words (numbers / units only), output it unchanged
`;

export function promptFor(mode: TranslateMode): string {
  return mode === "label" ? LABEL_PROMPT : SYSTEM_PROMPT;
}

export function userPromptFor(mode: TranslateMode, content: string): string {
  return mode === "label"
    ? `Translate this label to Thai: ${content}`
    : `Translate this construction spec to Thai:\n\n${content}`;
}

// Strip markdown code fences if the model wrapped its output.
export function cleanOutput(raw: string): string {
  const trimmed = raw.trim();
  if (trimmed.startsWith("```")) {
    const lines = trimmed.split("\n");
    if (lines.at(-1)?.trim() === "```") {
      return lines.slice(1, -1).join("\n");
    }
  }
  return trimmed;
}
