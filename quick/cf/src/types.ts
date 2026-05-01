// ── WebSocket message protocol ─────────────────────────────────────────────────
//
// Generated TypeScript types (TranslateRequest / TranslateResponse / ErrorResponse)
// live in quick/cli/bindings/ — emitted by ts-rs from Rust source of truth.
// WS message envelope types are defined here (CF Worker only concern).

// Messages: local CLI → Agent
export type ClientMessage =
  | { type: "pipeline"; name: string; content: string }  // translate + build PDF
  | { type: "translate"; name: string; content: string } // translate only
  | { type: "fetch-pdf"; name: string }                  // download built PDF

// Messages: Agent → local CLI
export type AgentEvent =
  | { type: "progress";   message: string }
  | { type: "translated"; name: string; content: string }
  | { type: "pdf";        name: string; bytes: string }   // base64-encoded PDF
  | { type: "skipped";    name: string }
  | { type: "error";      message: string }
