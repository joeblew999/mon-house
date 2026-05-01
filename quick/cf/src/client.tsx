// Pipeline monitor — React web UI.
//
// Connects to PipelineAgent over WebSocket via useAgent (agents/react).
// Lets the user paste spec markdown, run the pipeline, and download the
// translated .th.md content and compiled PDF — all from the browser.

import "./styles.css";
import { createRoot } from "react-dom/client";
import { useAgent } from "agents/react";
import { useState, useCallback, useEffect, useRef } from "react";

type Status = "connecting" | "connected" | "disconnected";

interface LogEntry {
  type: "info" | "success" | "error" | "skip";
  text: string;
}

interface PipelineResult {
  name: string;
  thMd: string | null;
  pdfBase64: string | null;
}

function App() {
  const [status, setStatus] = useState<Status>("connecting");
  const [specName, setSpecName] = useState("");
  const [content, setContent] = useState("");
  const [log, setLog] = useState<LogEntry[]>([]);
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<PipelineResult | null>(null);
  const logEndRef = useRef<HTMLDivElement>(null);

  const agent = useAgent({
    agent: "PipelineAgent",
    onOpen: useCallback(() => setStatus("connected"), []),
    onClose: useCallback(() => setStatus("disconnected"), []),
    onError: useCallback(() => setStatus("disconnected"), []),
  });

  // Handle raw JSON messages from the agent
  useEffect(() => {
    function onMessage(e: MessageEvent) {
      let msg: Record<string, string>;
      try { msg = JSON.parse(e.data as string); } catch { return; }

      switch (msg.type) {
        case "progress":
          setLog((l) => [...l, { type: "info", text: msg.message }]);
          break;
        case "translated":
          setResult((r) => ({ name: msg.name, thMd: msg.content, pdfBase64: r?.pdfBase64 ?? null }));
          setLog((l) => [...l, { type: "success", text: `✓ ${msg.name}.th.md translated` }]);
          break;
        case "skipped":
          setLog((l) => [...l, { type: "skip", text: `skip ${msg.name} (unchanged — no PDF needed)` }]);
          setRunning(false);
          break;
        case "pdf":
          setResult((r) => ({ name: msg.name, thMd: r?.thMd ?? null, pdfBase64: msg.bytes }));
          setLog((l) => [...l, { type: "success", text: `✓ ${msg.name}.th.pdf compiled` }]);
          setRunning(false);
          break;
        case "error":
          setLog((l) => [...l, { type: "error", text: `error: ${msg.message}` }]);
          setRunning(false);
          break;
      }
    }
    agent.addEventListener("message", onMessage);
    return () => agent.removeEventListener("message", onMessage);
  }, [agent]);

  // Auto-scroll log
  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [log]);

  const runPipeline = useCallback(() => {
    if (status !== "connected" || running || !specName.trim() || !content.trim()) return;
    setLog([]);
    setResult(null);
    setRunning(true);
    agent.send(JSON.stringify({ type: "pipeline", name: specName.trim(), content }));
  }, [status, running, specName, content, agent]);

  const downloadPdf = useCallback(() => {
    if (!result?.pdfBase64 || !result.name) return;
    const bytes = Uint8Array.from(atob(result.pdfBase64), (c) => c.charCodeAt(0));
    const blob = new Blob([bytes], { type: "application/pdf" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${result.name}.th.pdf`;
    a.click();
    URL.revokeObjectURL(url);
  }, [result]);

  const statusColor =
    status === "connected" ? "bg-green-500" :
    status === "connecting" ? "bg-yellow-400 animate-pulse" :
    "bg-red-500";

  const logColor = (type: LogEntry["type"]) =>
    type === "error" ? "text-red-400" :
    type === "success" ? "text-green-400" :
    type === "skip" ? "text-yellow-400" :
    "text-gray-300";

  return (
    <div className="min-h-screen bg-gray-50 flex flex-col">
      {/* Header */}
      <header className="bg-white border-b border-gray-200 px-6 py-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <span className="text-lg font-semibold text-gray-800">Quick Pipeline</span>
          <span className="text-xs text-gray-400">construction spec → Thai PDF</span>
        </div>
        <div className="flex items-center gap-2">
          <span className={`w-2 h-2 rounded-full ${statusColor}`} />
          <span className="text-xs text-gray-500 capitalize">{status}</span>
        </div>
      </header>

      {/* Body */}
      <main className="flex-1 p-6">
        <div className="max-w-2xl mx-auto space-y-4">

          {/* Spec name */}
          <div>
            <label className="block text-xs font-medium text-gray-600 mb-1">Spec name</label>
            <input
              className="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="e.g. GATE"
              value={specName}
              onChange={(e) => setSpecName(e.target.value)}
            />
          </div>

          {/* Markdown content */}
          <div>
            <label className="block text-xs font-medium text-gray-600 mb-1">Spec content (markdown)</label>
            <textarea
              className="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm font-mono h-52 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-y"
              placeholder="Paste your spec markdown here..."
              value={content}
              onChange={(e) => setContent(e.target.value)}
            />
          </div>

          {/* Run button */}
          <button
            className="w-full bg-blue-600 hover:bg-blue-700 text-white rounded-lg px-4 py-2.5 text-sm font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            disabled={status !== "connected" || running || !specName.trim() || !content.trim()}
            onClick={runPipeline}
          >
            {running ? "Running pipeline…" : "Run Pipeline"}
          </button>

          {/* Log */}
          {log.length > 0 && (
            <div className="bg-gray-900 rounded-lg p-4 text-xs font-mono space-y-0.5 max-h-56 overflow-y-auto">
              {log.map((entry, i) => (
                <div key={i} className={logColor(entry.type)}>{entry.text}</div>
              ))}
              <div ref={logEndRef} />
            </div>
          )}

          {/* Results */}
          {result && (
            <div className="border border-gray-200 rounded-lg divide-y divide-gray-100">
              {/* PDF download */}
              {result.pdfBase64 && (
                <div className="p-4 flex items-center justify-between">
                  <div>
                    <div className="text-sm font-medium text-gray-800">{result.name}.th.pdf</div>
                    <div className="text-xs text-gray-400 mt-0.5">Compiled Thai PDF</div>
                  </div>
                  <button
                    className="bg-green-600 hover:bg-green-700 text-white rounded-lg px-4 py-1.5 text-sm font-medium transition-colors"
                    onClick={downloadPdf}
                  >
                    Download
                  </button>
                </div>
              )}

              {/* Translation preview */}
              {result.thMd && (
                <details className="p-4">
                  <summary className="cursor-pointer text-sm text-gray-600 select-none">
                    {result.name}.th.md — preview translation
                  </summary>
                  <pre className="mt-3 text-xs text-gray-700 overflow-auto max-h-64 whitespace-pre-wrap">
                    {result.thMd}
                  </pre>
                </details>
              )}
            </div>
          )}
        </div>
      </main>
    </div>
  );
}

const root = document.getElementById("root")!;
createRoot(root).render(<App />);
