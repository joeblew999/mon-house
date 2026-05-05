// Spec copilot — chat UI backed by the ChatAgent on the Worker side.
//
// Layout: side-pane chat + main work area. Today the work area shows a
// placeholder pointing at the published PDFs; tomorrow (per ADR 007) it'll
// host the typst-WASM markdown editor + PDF preview, side-by-side with the
// copilot. Layout pattern lifted from cloudflare/agents/examples/playground.
//
// Hooks:
//   - useAgent — opens WebSocket to /agents/chat-agent/<name>
//   - useAgentChat — message history + streaming reply (AI SDK 6 shape:
//     messages, sendMessage, status; no more input/handleSubmit)
//
// Per-message model override: sendMessage accepts a ChatRequestOptions
// second argument with `body`. The agent reads `options.body.model` and
// uses that model instead of QUICK_CF_MODEL_CHAT for that one turn — see
// cf/COSTS.md for pricing per model.

import "./styles.css";
import { createRoot } from "react-dom/client";
import { useAgent } from "agents/react";
import { useAgentChat } from "@cloudflare/ai-chat/react";
import { type FormEvent, useEffect, useRef, useState } from "react";

// Curated model picks, ordered cheapest → priciest.
// $/M numbers track cf/COSTS.md — keep both files in sync.
const MODELS: { id: string; label: string; hint: string }[] = [
  { id: "@cf/google/gemma-3-12b-it",                 label: "Gemma 3 (cheap)",     hint: "default · ~$0.10 in / $0.20 out" },
  { id: "@cf/aisingapore/gemma-sea-lion-v4-27b-it",  label: "Sea-Lion (Thai)",     hint: "Thai-aware · ~$0.35 in / $0.56 out" },
  { id: "@cf/meta/llama-4-scout-17b-16e-instruct",   label: "Llama 4 Scout",       hint: "newer mid · ~$0.20 in / $0.85 out" },
  { id: "@cf/meta/llama-3.3-70b-instruct-fp8-fast",  label: "Llama 70B (smart)",   hint: "smarter · ~$0.29 in / $2.25 out" },
];

function App() {
  const [model, setModel] = useState<string>(MODELS[0].id);
  const [input, setInput] = useState("");
  const formRef = useRef<HTMLFormElement>(null);
  const logEndRef = useRef<HTMLDivElement>(null);

  const agent = useAgent({ agent: "ChatAgent" });
  const { messages, sendMessage, status, error } = useAgentChat({ agent });

  const busy = status === "submitted" || status === "streaming";

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, busy]);

  const onSubmit = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const text = input.trim();
    if (busy || !text) return;
    // Per-call body override: the chat agent reads options.body.model and
    // uses it for THIS turn. Switching the dropdown changes the next send.
    sendMessage(
      { role: "user", parts: [{ type: "text", text }] },
      { body: { model } },
    );
    setInput("");
    requestAnimationFrame(() => {
      formRef.current?.querySelector("textarea")?.focus();
    });
  };

  const textOf = (msg: { parts?: Array<{ type: string; text?: string }> }) =>
    (msg.parts ?? [])
      .filter((p) => p.type === "text" && typeof p.text === "string")
      .map((p) => p.text)
      .join("");

  const activeModel = MODELS.find((m) => m.id === model) ?? MODELS[0];

  return (
    <div className="h-screen flex flex-col md:flex-row bg-gray-50">
      {/* MAIN WORK AREA — where the markdown editor + typst-WASM PDF preview
          will land per ADR 007. Today it's just a placeholder. */}
      <main className="flex-1 min-h-0 overflow-y-auto p-8">
        <div className="max-w-3xl mx-auto space-y-4">
          <h1 className="text-2xl font-semibold text-gray-800">Quick Specs</h1>
          <p className="text-sm text-gray-600">
            Construction-spec workspace. Use the copilot on the right to draft,
            revise, or ask questions about a section. Built PDFs are published
            to the{" "}
            <a
              href="https://github.com/joeblew999/mon-house/releases/tag/specs-latest"
              target="_blank"
              rel="noreferrer"
              className="text-blue-600 hover:underline"
            >
              specs-latest release ↗
            </a>
            . An in-browser editor + PDF preview lands later (ADR 007).
          </p>
          <div className="rounded-lg border border-dashed border-gray-300 bg-white p-6 text-sm text-gray-500">
            Editor &amp; PDF preview placeholder — see{" "}
            <code className="font-mono text-xs">adr/007-typst-wasm-in-browser.md</code>.
          </div>
        </div>
      </main>

      {/* CHAT SIDE-PANE — fixed-width on desktop, full-bleed on mobile. */}
      <aside className="md:w-96 md:border-l border-t md:border-t-0 border-gray-200 bg-white flex flex-col min-h-0">
        <header className="px-4 py-3 border-b border-gray-200 flex items-center justify-between">
          <div>
            <div className="text-sm font-semibold text-gray-800">Spec Copilot</div>
            <div className="text-xs text-gray-400">{activeModel.label}</div>
          </div>
          <select
            className="text-xs border border-gray-300 rounded-md px-2 py-1 bg-white max-w-[10rem] truncate"
            value={model}
            onChange={(e) => setModel(e.target.value)}
            title={activeModel.hint}
          >
            {MODELS.map((m) => (
              <option key={m.id} value={m.id} title={m.hint}>
                {m.label}
              </option>
            ))}
          </select>
        </header>

        <div className="flex-1 overflow-y-auto px-4 py-3 space-y-2.5">
          {messages.length === 0 && (
            <div className="text-xs text-gray-500 bg-gray-50 border border-gray-200 rounded-lg px-3 py-4 text-center">
              Drop a markdown section in below, or ask a question. The active
              model burns ~{activeModel.hint.split("·")[1]?.trim() ?? "?"}.
            </div>
          )}

          {messages.map((m) => (
            <div
              key={m.id}
              className={
                m.role === "user"
                  ? "ml-auto max-w-[90%] bg-blue-600 text-white rounded-2xl rounded-br-sm px-3 py-2 text-sm whitespace-pre-wrap"
                  : "mr-auto max-w-[90%] bg-gray-100 border border-gray-200 rounded-2xl rounded-bl-sm px-3 py-2 text-sm whitespace-pre-wrap"
              }
            >
              {textOf(m)}
            </div>
          ))}

          {busy && (
            <div className="mr-auto bg-gray-100 border border-gray-200 rounded-2xl rounded-bl-sm px-3 py-2 text-xs text-gray-500">
              <span className="inline-block animate-pulse">…thinking</span>
            </div>
          )}

          {error && (
            <div className="bg-red-50 border border-red-200 text-red-700 rounded-lg px-3 py-2 text-xs">
              {error.message}
            </div>
          )}

          <div ref={logEndRef} />
        </div>

        <form ref={formRef} onSubmit={onSubmit} className="p-3 border-t border-gray-200 flex flex-col gap-2">
          <textarea
            className="w-full border border-gray-300 rounded-lg px-2.5 py-2 text-sm font-mono h-24 resize-y focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-60"
            placeholder="Type or paste markdown…"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            disabled={busy}
            onKeyDown={(e) => {
              if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
                e.preventDefault();
                formRef.current?.requestSubmit();
              }
            }}
          />
          <div className="flex items-center justify-between">
            <span className="text-[11px] text-gray-400">⌘+Enter to send</span>
            <button
              type="submit"
              className="bg-blue-600 hover:bg-blue-700 text-white rounded-md px-3 py-1.5 text-xs font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
              disabled={busy || !input.trim()}
            >
              {busy ? "Sending…" : "Send"}
            </button>
          </div>
        </form>
      </aside>
    </div>
  );
}

const root = document.getElementById("root")!;
createRoot(root).render(<App />);
