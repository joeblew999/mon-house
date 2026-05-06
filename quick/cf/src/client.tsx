// Spec copilot — chat UI backed by the ChatAgent on the Worker side.
//
// Layout: main work area + chat side-pane. Desktop: side-by-side. Mobile:
// main is full-screen, chat slides in as an overlay drawer from the right
// (lifted from cloudflare-agents/examples/playground/src/layout/Layout.tsx,
// mirrored — playground sidebars on the left, our chat is on the right).
//
// Styling: @cloudflare/kumo (semantic tokens like bg-kumo-base; theme is
// Cloudflare orange). Light/dark mode toggled via document.data-mode and
// resolved automatically by Kumo's CSS light-dark() pairs.
//
// Hooks:
//   - useAgent — opens WebSocket to /agents/chat-agent/<name>
//   - useAgentChat — message history + streaming reply (AI SDK 6 shape)
//
// Per-message model override: sendMessage's second arg `body` is read by the
// agent on the server and used for that one turn — see cf/COSTS.md.

import "./styles.css";
import "@cloudflare/kumo/styles";
import { createRoot } from "react-dom/client";
import { useAgent } from "agents/react";
import { useAgentChat } from "@cloudflare/ai-chat/react";
import { type FormEvent, useEffect, useRef, useState } from "react";
import { WasmVfs, WasmVfsRO } from "./wasm-vfs";
import {
  Button,
  InputArea,
  Empty,
  Text,
} from "@cloudflare/kumo";
import {
  ListIcon,
  XIcon,
  PaperPlaneRightIcon,
  MoonIcon,
  SunIcon,
} from "@phosphor-icons/react";

// Curated model picks, ordered cheapest → priciest.
// $/M numbers track cf/COSTS.md — keep both files in sync.
const MODELS: { id: string; label: string; hint: string }[] = [
  { id: "@cf/google/gemma-3-12b-it",                 label: "Gemma 3 (cheap)",     hint: "default · ~$0.10 in / $0.20 out" },
  { id: "@cf/aisingapore/gemma-sea-lion-v4-27b-it",  label: "Sea-Lion (Thai)",     hint: "Thai-aware · ~$0.35 in / $0.56 out" },
  { id: "@cf/meta/llama-4-scout-17b-16e-instruct",   label: "Llama 4 Scout",       hint: "newer mid · ~$0.20 in / $0.85 out" },
  { id: "@cf/meta/llama-3.3-70b-instruct-fp8-fast",  label: "Llama 70B (smart)",   hint: "smarter · ~$0.29 in / $2.25 out" },
];

function ModeToggle() {
  const [mode, setMode] = useState<"light" | "dark">(
    () => (localStorage.getItem("theme") as "light" | "dark") || "light",
  );
  useEffect(() => {
    document.documentElement.setAttribute("data-mode", mode);
    document.documentElement.style.colorScheme = mode;
    localStorage.setItem("theme", mode);
  }, [mode]);
  return (
    <Button
      variant="ghost"
      shape="square"
      size="sm"
      aria-label="Toggle theme"
      onClick={() => setMode((m) => (m === "light" ? "dark" : "light"))}
      icon={mode === "light" ? <MoonIcon size={16} /> : <SunIcon size={16} />}
    />
  );
}

function ChatPane({ onClose }: { onClose?: () => void }) {
  const [model, setModel] = useState<string>(MODELS[0].id);
  const [input, setInput] = useState("");
  const formRef = useRef<HTMLFormElement>(null);
  const logEndRef = useRef<HTMLDivElement>(null);

  // Ref so the body function always captures the latest model without
  // needing to re-initialise the hook when the dropdown changes.
  const modelRef = useRef(model);
  modelRef.current = model;

  // useAgent dedupes connections by (agent, name) — calling it with the same
  // args from EnginePanel and here yields the same underlying WebSocket. So
  // dep-graph events sent from EnginePanel arrive at the same ChatAgent DO
  // instance that's serving this chat session.
  const agent = useAgent({ agent: "ChatAgent" });
  const { messages, sendMessage, status, error } = useAgentChat({
    agent,
    // body is called on each send — server reads options.body.model in onChatMessage.
    body: () => ({ model: modelRef.current }),
  });

  const busy = status === "submitted" || status === "streaming";

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, busy]);

  const onSubmit = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const text = input.trim();
    if (busy || !text) return;
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
    <div className="flex flex-col h-full bg-kumo-base">
      <header className="px-4 py-3 border-b border-kumo-line flex items-center justify-between gap-2 shrink-0">
        <div className="min-w-0">
          <Text variant="secondary" className="block text-sm font-semibold truncate">
            Spec Copilot
          </Text>
          <Text variant="secondary" className="block text-xs">
            {activeModel.label}
          </Text>
        </div>
        <div className="flex items-center gap-1">
          <select
            className="text-xs border border-kumo-line rounded-md px-2 py-1 bg-kumo-base text-kumo-default max-w-[10rem] truncate"
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
          {onClose && (
            <Button
              variant="ghost"
              shape="square"
              size="sm"
              icon={<XIcon size={18} />}
              onClick={onClose}
              aria-label="Close chat"
              className="md:hidden"
            />
          )}
        </div>
      </header>

      <div className="flex-1 overflow-y-auto px-4 py-3 space-y-2.5">
        {messages.length === 0 && (
          <Empty
            title="Start a conversation"
            description={`Drop a markdown section in below, or ask a question. The active model burns ~${activeModel.hint.split("·")[1]?.trim() ?? "?"}.`}
          />
        )}

        {messages.map((m) => (
          <div
            key={m.id}
            className={
              m.role === "user"
                ? "ml-auto max-w-[90%] bg-kumo-accent text-kumo-on-accent rounded-2xl rounded-br-sm px-3 py-2 text-sm whitespace-pre-wrap"
                : "mr-auto max-w-[90%] bg-kumo-control border border-kumo-line text-kumo-default rounded-2xl rounded-bl-sm px-3 py-2 text-sm whitespace-pre-wrap"
            }
          >
            {textOf(m)}
          </div>
        ))}

        {busy && (
          <div className="mr-auto bg-kumo-control border border-kumo-line text-kumo-subtle rounded-2xl rounded-bl-sm px-3 py-2 text-xs">
            <span className="inline-block animate-pulse">…thinking</span>
          </div>
        )}

        {error && (
          <div className="bg-kumo-danger-tint border border-kumo-danger text-kumo-danger rounded-md px-3 py-2 text-xs">
            {error.message}
          </div>
        )}

        <div ref={logEndRef} />
      </div>

      <form ref={formRef} onSubmit={onSubmit} className="p-3 border-t border-kumo-line flex flex-col gap-2 shrink-0">
        <InputArea
          size="sm"
          placeholder="Type or paste markdown…"
          value={input}
          onValueChange={setInput}
          disabled={busy}
          rows={4}
          onKeyDown={(e) => {
            if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
              e.preventDefault();
              formRef.current?.requestSubmit();
            }
          }}
          className="font-mono"
        />
        <div className="flex items-center justify-between">
          <Text variant="secondary" className="text-[11px]">
            ⌘+Enter to send
          </Text>
          <Button
            type="submit"
            size="sm"
            icon={<PaperPlaneRightIcon size={14} />}
            disabled={busy || !input.trim()}
          >
            {busy ? "Sending…" : "Send"}
          </Button>
        </div>
      </form>

    </div>
  );
}

// ── EnginePanel ───────────────────────────────────────────────────────────────
// Smoke test for the Rust engine running in the browser via WASM. User picks a
// project root (a FileSystemDirectoryHandle); we wrap it in a WasmVfs and call
// the Rust find_dependents() through the wasm-bindgen interop. No file content
// is preloaded — every read is on-demand through the directory handle.
//
// This is the "proves it works" panel; it'll be replaced by real editor UI
// (ADR 007). The pattern stays: WASM loaded once, vfs handle constructed from
// the user's directory pick, Rust calls in async via the JS bridge.
function EnginePanel() {
  // Same args as ChatPane's useAgent — the hook caches connections, so this
  // shares the underlying WebSocket. Lets us emit dep-graph events to the
  // same ChatAgent DO that's hosting the chat session.
  const agent = useAgent({ agent: "ChatAgent" });
  const [vfs, setVfs] = useState<WasmVfs | WasmVfsRO | null>(null);
  const [vfsKind, setVfsKind] = useState<"rw" | "ro" | null>(null);
  const [version, setVersion] = useState<string | null>(null);
  const [target, setTarget] = useState("specs/_partials/paint-metal.md");
  const [deps, setDeps] = useState<string[] | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  // FS Access API is the default-when-supported path. `?usefallback` in the URL
  // forces the webkitdirectory <input> path even on Chromium — needed for
  // Playwright/automation that can't approve native directory pickers.
  const fsAccessSupported =
    typeof window !== "undefined" &&
    "showDirectoryPicker" in window &&
    !new URLSearchParams(window.location.search).has("usefallback");

  // Loaded lazily on first interaction so the WASM (~380 KB gzipped) doesn't
  // block initial page load. Keeps the cached engine module in a ref so
  // multiple find-dependents calls reuse one init.
  const engineRef = useRef<typeof import("./wasm/quick_tool") | null>(null);

  const ensureEngine = async () => {
    if (engineRef.current) return engineRef.current;
    const mod = await import("./wasm/quick_tool");
    await mod.default();
    engineRef.current = mod;
    setVersion(mod.engine_version());
    return mod;
  };

  const onPickFolderFsAccess = async () => {
    setErr(null);
    try {
      // @ts-expect-error — TS lib.dom may not have this yet
      const handle = (await window.showDirectoryPicker()) as FileSystemDirectoryHandle;
      setVfs(new WasmVfs(handle));
      setVfsKind("rw");
      setDeps(null);
    } catch (e) {
      if ((e as DOMException).name !== "AbortError") {
        setErr(`folder pick failed: ${String(e)}`);
      }
    }
  };

  const onPickFolderInput = () => {
    setErr(null);
    fileInputRef.current?.click();
  };

  const onInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files || files.length === 0) return;
    setVfs(new WasmVfsRO(files));
    setVfsKind("ro");
    setDeps(null);
  };

  const onFindDependents = async () => {
    if (!vfs) return;
    setBusy(true);
    setErr(null);
    setDeps(null);
    try {
      const mod = await ensureEngine();
      const partial = target.trim();
      const dir = partial.includes("/") ? partial.split("/")[0] : "";
      const result = (await mod.find_dependents(vfs, dir, partial)) as string[];
      setDeps(result);
      // Emit a dep-graph event over the existing chat WebSocket so the
      // server-side ChatAgent can persist it and the AI sees it as live
      // context on its next reply. Fire-and-forget — non-critical.
      try {
        agent.send(
          JSON.stringify({
            type: "dep-graph-event",
            event: {
              kind: "partial-queried",
              path: partial,
              dependents: result,
              ts: new Date().toISOString(),
            },
          }),
        );
      } catch {
        // WebSocket might be closed; harmless to drop.
      }
    } catch (e) {
      setErr(`find_dependents failed: ${String(e)}`);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="border border-kumo-line rounded-lg p-4 space-y-3 bg-kumo-control">
      <div className="flex items-center justify-between">
        <Text className="text-sm font-semibold">Engine smoke test (WASM)</Text>
        {version && (
          <Text variant="secondary" className="text-xs">
            v{version}
          </Text>
        )}
      </div>

      {!vfs ? (
        <div className="space-y-2">
          <Text variant="secondary" className="text-xs">
            Pick the project root (e.g. <code>quick/</code>). The Rust engine
            will scan <code>specs/*.md</code> on demand — no content is
            uploaded.{" "}
            {fsAccessSupported
              ? "Read+write supported via FS Access API."
              : "Safari/Firefox: read-only via webkitdirectory <input>."}
          </Text>
          {fsAccessSupported ? (
            <Button onClick={onPickFolderFsAccess} size="sm">
              Pick project folder…
            </Button>
          ) : (
            <>
              <Button onClick={onPickFolderInput} size="sm">
                Pick project folder…
              </Button>
              <input
                ref={fileInputRef}
                type="file"
                // @ts-expect-error — webkitdirectory isn't in TS lib.dom yet
                webkitdirectory=""
                multiple
                style={{ display: "none" }}
                onChange={onInputChange}
              />
            </>
          )}
        </div>
      ) : (
        <div className="space-y-2">
          <Text variant="secondary" className="text-xs">
            Folder granted ({vfsKind === "ro" ? "read-only" : "read+write"}).
            Try a partial path; the engine will return every spec that{" "}
            <code>&lt;!-- include: --&gt;</code>s it.
          </Text>
          <div className="flex gap-2">
            <input
              type="text"
              value={target}
              onChange={(e) => setTarget(e.target.value)}
              className="flex-1 text-xs border border-kumo-line rounded-md px-2 py-1 bg-kumo-base text-kumo-default font-mono"
              placeholder="specs/_partials/paint-metal.md"
            />
            <Button onClick={onFindDependents} disabled={busy} size="sm">
              {busy ? "…" : "find dependents"}
            </Button>
          </div>
          {deps && (
            <div className="text-xs space-y-1">
              <Text variant="secondary" className="block">
                {deps.length} dependent{deps.length === 1 ? "" : "s"}:
              </Text>
              {deps.length === 0 ? (
                <Text variant="secondary" className="block italic">
                  none
                </Text>
              ) : (
                <ul className="list-disc pl-5 space-y-0.5 font-mono">
                  {deps.map((d) => (
                    <li key={d}>{d}</li>
                  ))}
                </ul>
              )}
            </div>
          )}
        </div>
      )}

      {err && (
        <Text variant="error" className="text-xs block">
          {err}
        </Text>
      )}
    </div>
  );
}

function App() {
  const [chatOpen, setChatOpen] = useState(false);
  // ChatPane and EnginePanel each call useAgent({ agent: "ChatAgent" })
  // independently — the SDK dedupes connections by (agent, name) so they
  // share the underlying WebSocket. Means dep-graph events from EnginePanel
  // arrive at the same DO instance that's hosting the chat.

  return (
    <div className="h-screen flex flex-col md:flex-row bg-kumo-base text-kumo-default">
      {/* Mobile header — only visible below md */}
      <header className="md:hidden flex items-center justify-between px-4 py-3 border-b border-kumo-line bg-kumo-base shrink-0">
        <Text className="text-sm font-semibold">Quick Specs</Text>
        <div className="flex items-center gap-1">
          <ModeToggle />
          <Button
            variant="ghost"
            shape="square"
            size="sm"
            icon={<ListIcon size={20} />}
            onClick={() => setChatOpen(true)}
            aria-label="Open chat"
          />
        </div>
      </header>

      {/* MAIN WORK AREA — placeholder until ADR 007 (typst-WASM editor + PDF preview) lands. */}
      <main className="flex-1 min-h-0 overflow-y-auto p-6 md:p-8 bg-kumo-base">
        <div className="max-w-3xl mx-auto space-y-4">
          <div className="hidden md:flex items-center justify-between">
            <Text className="text-2xl font-semibold">Quick Specs</Text>
            <ModeToggle />
          </div>
          <Text variant="secondary" className="text-sm block">
            Construction-spec workspace. Use the copilot{" "}
            <span className="md:hidden">(tap the menu icon)</span>
            <span className="hidden md:inline">on the right</span> to draft,
            revise, or ask questions about a section. Built PDFs are published
            to the{" "}
            <a
              href="https://github.com/joeblew999/mon-house/releases/tag/specs-latest"
              target="_blank"
              rel="noreferrer"
              className="text-kumo-accent hover:underline"
            >
              specs-latest release ↗
            </a>
            . An in-browser editor + PDF preview lands later (ADR 007).
          </Text>
          <Empty
            title="Editor & PDF preview"
            description="Coming with ADR 007 — typst-WASM in browser."
          />

          <EnginePanel />
        </div>
      </main>

      {/* CHAT — desktop: static right pane. Mobile: overlay drawer from right. */}
      <aside className="hidden md:flex md:w-96 md:border-l border-kumo-line flex-col shrink-0 min-h-0">
        <ChatPane />
      </aside>

      {chatOpen && (
        <div className="fixed inset-0 z-40 md:hidden">
          <button
            type="button"
            className="absolute inset-0 bg-black/40"
            onClick={() => setChatOpen(false)}
            aria-label="Close chat"
          />
          <aside className="absolute right-0 top-0 h-full w-[90vw] max-w-md bg-kumo-base shadow-xl flex flex-col">
            <ChatPane onClose={() => setChatOpen(false)} />
          </aside>
        </div>
      )}
    </div>
  );
}

const root = document.getElementById("root")!;
createRoot(root).render(<App />);
