// Spec copilot — chat UI backed by the ChatAgent on the Worker side.
//
// Uses `useAgent` + `useAgentChat` from the agents SDK:
//   - opens a WebSocket to /agents/chat-agent/<name>
//   - persists message history in the DO's SQLite storage
//   - reconnects automatically on close
//
// AI SDK 6 hook shape: { messages, sendMessage, status, error, regenerate,
// stop }. We drive the input ourselves with React state because the SDK no
// longer ships `input` / `handleInputChange` / `handleSubmit`.

import "./styles.css";
import { createRoot } from "react-dom/client";
import { useAgent } from "agents/react";
import { useAgentChat } from "@cloudflare/ai-chat/react";
import { type FormEvent, useEffect, useRef, useState } from "react";

function App() {
  const agent = useAgent({ agent: "ChatAgent" });
  const { messages, sendMessage, status, error } = useAgentChat({ agent });

  const [input, setInput] = useState("");
  const formRef = useRef<HTMLFormElement>(null);
  const logEndRef = useRef<HTMLDivElement>(null);

  const busy = status === "submitted" || status === "streaming";

  // Auto-scroll on new messages.
  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, busy]);

  const onSubmit = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const text = input.trim();
    if (busy || !text) return;
    sendMessage({ role: "user", parts: [{ type: "text", text }] });
    setInput("");
    requestAnimationFrame(() => {
      formRef.current?.querySelector("textarea")?.focus();
    });
  };

  // Pull the visible text out of an AI SDK 6 UIMessage. UIMessage.parts is an
  // array of `{ type: "text", text: "..." }` (and other variants we ignore).
  const textOf = (msg: { parts?: Array<{ type: string; text?: string }> }) =>
    (msg.parts ?? [])
      .filter((p) => p.type === "text" && typeof p.text === "string")
      .map((p) => p.text)
      .join("");

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
              {textOf(m)}
            </div>
          ))}

          {busy && (
            <div className="mr-auto bg-white border border-gray-200 rounded-2xl rounded-bl-sm px-4 py-2.5 text-sm text-gray-500">
              <span className="inline-block animate-pulse">…thinking</span>
            </div>
          )}

          {error && (
            <div className="mr-auto bg-red-50 border border-red-200 text-red-700 rounded-lg px-4 py-2 text-sm">
              {error.message}
            </div>
          )}

          <div ref={logEndRef} />
        </div>

        {/* Composer */}
        <form ref={formRef} onSubmit={onSubmit} className="flex flex-col gap-2">
          <textarea
            className="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm font-mono h-28 resize-y focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-60"
            placeholder="Type your message or paste a markdown section…"
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
