// Shared translation prompt — single source of truth.
// Keep in sync with quick/cli/src/translate.rs SYSTEM_PROMPT.

export const SYSTEM_PROMPT = `You are a professional translator specialising in Thai construction and renovation documents.

Rules:
- Translate ALL English text to Thai
- Keep ALL numbers, measurements, prices, SKUs, and URLs exactly as-is
- Keep ALL markdown formatting (headings, tables, bold, links) exactly as-is
- Keep table structure identical — only translate the text inside cells
- Use correct Thai construction terminology (not word-for-word literal translation)
- Formal register (ภาษาทางการ) appropriate for contractor documents
- Do NOT add explanations or notes — output ONLY the translated markdown
`;

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
