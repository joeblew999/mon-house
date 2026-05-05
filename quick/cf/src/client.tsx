// Spec copilot — a chat UI backed by the ChatAgent on the Worker side.
//
// Uses `useAgentChat` from `agents/react`, which:
//   - opens a WebSocket to /agents/chat-agent/<name>
//   - streams assistant replies token-by-token
//   - persists message history in the DO's SQLite storage
//   - reconnects automatically on close
//
// Today the chat is general-purpose: drop a markdown spec section in,
// ask for revisions, paraphrasing, sanity-checking. Future work: pipe
// the active spec into the system prompt as live context, hook the
// builder-quote workflow from ADR 006, etc.

import "./styles.css";
import { createRoot } from "react-dom/client";
import { useAgent } from "agents/react";
import { useAgentChat } from "agents/ai-react";
import { type FormEvent, useRef } from "react";

function App() {
  const agent = useAgent({ agent: "ChatAgent" });
  const { messages, input, handleInputChange, handleSubmit, status } = useAgentChat({ agent });
  const formRef = useRef<HTMLFormElement>(null);

  const onSubmit = (e: FormEvent<HTMLFormElement>) => {
    handleSubmit(e);
    // Re-focus the textarea after submit
    requestAnimationFrame(() => {
      const ta = formRef.current?.querySelector("textarea");
      ta?.focus();
    });
  };

  const busy = status === "submitted" || status === "streaming";

  return (
    <div className="min-h-screen bg-gray-50 flex flex-col">
      <header className="bg-white border-b border-gray-200 px-6 py-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <span className="text-lg font-semibold text-gray-800">Spec Copilot</span>
          <span className="text-xs text-gray-400">drafting + revising construction specs</span>
        </div>
        <a
          href="https://github.com/joeblew999/mon-house/releases/tag/specs-latest"
          target="_blank"
          rel="noreferrer"
          className="text-xs text-blue-600 hover:underline"
        >
          PDFs ↗
        </a>
      </header>

      <main className="flex-1 flex flex-col max-w-3xl mx-auto w-full p-6 gap-4">
        {/* Conversation */}
        <div className="flex-1 overflow-y-auto space-y-3 pr-1">
          {messages.length === 0 && (
            <div className="text-sm text-gray-500 bg-white border border-gray-200 rounded-lg px-4 py-6 text-center">
              Drop a markdown section in below and ask for a revision, summary, or sanity check.
            </div>
          )}

          {messages.map((m) => (
            <div
              key={m.id}
              className={
                m.role === "user"
                  ? "ml-auto max-w-[85%] bg-blue-600 text-white rounded-2xl rounded-br-sm px-4 py-2.5 text-sm whitespace-pre-wrap"
                  : "mr-auto max-w-[85%] bg-white border border-gray-200 rounded-2xl rounded-bl-sm px-4 py-2.5 text-sm whitespace-pre-wrap"
              }
            >
              {m.content}
            </div>
          ))}

          {busy && (
            <div className="mr-auto bg-white border border-gray-200 rounded-2xl rounded-bl-sm px-4 py-2.5 text-sm text-gray-500">
              <span className="inline-block animate-pulse">…thinking</span>
            </div>
          )}
        </div>

        {/* Composer */}
        <form ref={formRef} onSubmit={onSubmit} className="flex flex-col gap-2">
          <textarea
            className="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm font-mono h-28 resize-y focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-60"
            placeholder="Type your message or paste a markdown section…"
            value={input}
            onChange={handleInputChange}
            disabled={busy}
            onKeyDown={(e) => {
              if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
                e.preventDefault();
                formRef.current?.requestSubmit();
              }
            }}
          />
          <div className="flex items-center justify-between">
            <span className="text-xs text-gray-400">⌘+Enter to send</span>
            <button
              type="submit"
              className="bg-blue-600 hover:bg-blue-700 text-white rounded-lg px-4 py-2 text-sm font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
              disabled={busy || !input.trim()}
            >
              {busy ? "Sending…" : "Send"}
            </button>
          </div>
        </form>
      </main>
    </div>
  );
}

const root = document.getElementById("root")!;
createRoot(root).render(<App />);
