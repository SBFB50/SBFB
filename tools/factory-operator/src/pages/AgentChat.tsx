// SPDX-License-Identifier: AGPL-3.0-or-later

import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useApi } from "@/hooks/useApi";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  GitBranchIcon, CheckCircle2Icon,
  AlertCircleIcon, LoaderIcon, ActivityIcon, TerminalIcon,
  LayersIcon, ShieldCheckIcon, FileTextIcon,
} from "lucide-react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";
import "@xterm/xterm/css/xterm.css";

interface SprintStatus {
  sprint: number;
  head: string;
  branch: string;
  current_phase: string;
  has_kickoff: boolean;
  has_plan: boolean;
  has_design_review: boolean;
  has_audit_plan: boolean;
  phases: { letter: string; has_preflight: boolean; has_review: boolean; review_verdict: string; has_codex: boolean }[];
}

interface LintResult {
  ok: boolean;
  errors: { file: string; message: string }[];
  warnings: { file: string; message: string }[];
}

function PhaseChip({ phase }: { phase: SprintStatus["phases"][0] }) {
  const allGreen = phase.has_preflight && phase.has_review && phase.review_verdict === "PASS" && phase.has_codex;
  const inProgress = phase.has_preflight && !phase.has_review;

  return (
    <div className={`flex items-center gap-1.5 rounded-md px-2 py-1 text-xs font-mono transition-all duration-300 ${
      allGreen
        ? "bg-emerald-500/15 text-emerald-400 ring-1 ring-emerald-500/30"
        : inProgress
          ? "bg-amber-500/15 text-amber-400 ring-1 ring-amber-500/30 animate-pulse"
          : "bg-zinc-800 text-zinc-500 ring-1 ring-zinc-700"
    }`}>
      <span className="font-bold">{phase.letter}</span>
      {phase.has_preflight && <ShieldCheckIcon className="size-3" />}
      {phase.has_review && <FileTextIcon className="size-3" />}
      {phase.has_codex && <CheckCircle2Icon className="size-3" />}
    </div>
  );
}

function Pulse({ active }: { active: boolean }) {
  if (!active) return null;
  return (
    <span className="relative flex size-2">
      <span className="absolute inline-flex size-full animate-ping rounded-full bg-emerald-400 opacity-75" />
      <span className="relative inline-flex size-2 rounded-full bg-emerald-500" />
    </span>
  );
}

export function AgentChat() {
  const { t } = useTranslation();
  const termRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const [connected, setConnected] = useState(false);

  const status = useApi<SprintStatus>("/status");
  const lint = useApi<LintResult>("/lint");

  const [refreshKey, setRefreshKey] = useState(0);
  useEffect(() => {
    const interval = setInterval(() => setRefreshKey((k) => k + 1), 15000);
    return () => clearInterval(interval);
  }, []);

  const statusFresh = useApi<SprintStatus>(`/status?_t=${refreshKey}`);
  const lintFresh = useApi<LintResult>(`/lint?_t=${refreshKey}`);
  const sessionsFresh = useApi<{ sessions: { name: string; size_bytes: number }[] }>(`/terminal/sessions?_t=${refreshKey}`);
  const currentStatus = statusFresh.data ?? status.data;
  const currentLint = lintFresh.data ?? lint.data;
  const currentSessions = sessionsFresh.data?.sessions ?? [];

  const connectTerminal = useCallback(() => {
    if (!termRef.current || terminalRef.current) return;

    const term = new Terminal({
      cursorBlink: true,
      fontSize: 13,
      fontFamily: "'Cascadia Code', 'Fira Code', 'JetBrains Mono', monospace",
      theme: {
        background: "#09090b",
        foreground: "#fafafa",
        cursor: "#fafafa",
        selectionBackground: "#27272a",
        black: "#09090b",
        red: "#ef4444",
        green: "#22c55e",
        yellow: "#eab308",
        blue: "#3b82f6",
        magenta: "#a855f7",
        cyan: "#06b6d4",
        white: "#fafafa",
      },
      allowProposedApi: true,
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.loadAddon(new WebLinksAddon());

    term.open(termRef.current);
    fitAddon.fit();
    fitAddonRef.current = fitAddon;
    terminalRef.current = term;

    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const wsUrl = `${protocol}//${window.location.host}/api/terminal/ws`;
    const ws = new WebSocket(wsUrl);
    ws.binaryType = "arraybuffer";
    wsRef.current = ws;

    ws.onopen = () => {
      setConnected(true);
      const dims = fitAddon.proposeDimensions();
      if (dims) {
        ws.send(JSON.stringify({ type: "resize", cols: dims.cols, rows: dims.rows }));
      }
    };

    ws.onmessage = (event) => {
      if (event.data instanceof ArrayBuffer) {
        term.write(new Uint8Array(event.data));
      } else {
        term.write(event.data);
      }
    };

    ws.onclose = () => {
      setConnected(false);
      term.write("\r\n\x1b[90m--- session closed ---\x1b[0m\r\n");
    };

    term.onData((data) => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(data);
      }
    });

    term.onResize(({ cols, rows }) => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: "resize", cols, rows }));
      }
    });

    const resizeObserver = new ResizeObserver(() => {
      fitAddon.fit();
    });
    resizeObserver.observe(termRef.current);

    return () => {
      resizeObserver.disconnect();
      ws.close();
      term.dispose();
      terminalRef.current = null;
      wsRef.current = null;
    };
  }, []);

  useEffect(() => {
    const cleanup = connectTerminal();
    return cleanup;
  }, [connectTerminal]);

  const phaseCount = currentStatus?.phases?.length ?? 0;
  const passCount = currentStatus?.phases?.filter((p) => p.review_verdict === "PASS").length ?? 0;

  return (
    <div className="flex h-[calc(100vh-8rem)] gap-3">
      {/* LEFT: Project Dashboard */}
      <div className="flex w-80 shrink-0 flex-col gap-3 overflow-hidden">
        <div className="flex items-center gap-2">
          <ActivityIcon className="size-4 text-primary" />
          <h2 className="text-sm font-bold">{t("chat.title")}</h2>
          <Pulse active={connected} />
        </div>

        <ScrollArea className="flex-1">
          <div className="space-y-3 pr-2">
            {/* Sprint Status Card */}
            {currentStatus && (
              <div className="rounded-lg border border-border bg-card p-3 transition-all duration-500">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <LayersIcon className="size-4 text-primary" />
                    <span className="text-sm font-semibold">Sprint {currentStatus.sprint}</span>
                  </div>
                  <Badge variant="outline" className="font-mono text-[10px]">
                    <GitBranchIcon className="mr-1 size-3" />
                    {currentStatus.head}
                  </Badge>
                </div>

                <div className="mt-2 flex items-center gap-1.5">
                  <Badge variant={currentStatus.current_phase === "done" ? "default" : "secondary"} className="text-[10px]">
                    {currentStatus.current_phase === "done" ? "Terminé" : `Phase ${currentStatus.current_phase}`}
                  </Badge>
                  <span className="text-[10px] text-muted-foreground">
                    {passCount}/{phaseCount} phases
                  </span>
                </div>

                {/* Phase chips */}
                <div className="mt-3 flex flex-wrap gap-1.5">
                  {currentStatus.phases.map((p) => (
                    <PhaseChip key={p.letter} phase={p} />
                  ))}
                </div>

                {/* Artifacts */}
                <div className="mt-3 grid grid-cols-2 gap-1.5 text-[10px]">
                  {[
                    { label: "Kickoff", ok: currentStatus.has_kickoff },
                    { label: "Plan", ok: currentStatus.has_plan },
                    { label: "Design Review", ok: currentStatus.has_design_review },
                    { label: "Audit Plan", ok: currentStatus.has_audit_plan },
                  ].map((a) => (
                    <div key={a.label} className={`flex items-center gap-1 rounded px-1.5 py-0.5 ${
                      a.ok ? "bg-emerald-500/10 text-emerald-400" : "bg-zinc-800 text-zinc-600"
                    }`}>
                      {a.ok ? <CheckCircle2Icon className="size-3" /> : <AlertCircleIcon className="size-3" />}
                      {a.label}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Lint Card */}
            {currentLint && (
              <div className={`rounded-lg border p-3 transition-all duration-500 ${
                currentLint.ok
                  ? "border-emerald-500/30 bg-emerald-500/5"
                  : "border-red-500/30 bg-red-500/5"
              }`}>
                <div className="flex items-center gap-2">
                  {currentLint.ok
                    ? <CheckCircle2Icon className="size-4 text-emerald-400" />
                    : <AlertCircleIcon className="size-4 text-red-400" />
                  }
                  <span className="text-sm font-semibold">
                    Lint {currentLint.ok ? "CLEAN" : `${currentLint.errors.length}E / ${currentLint.warnings.length}W`}
                  </span>
                </div>
                {!currentLint.ok && currentLint.errors.length > 0 && (
                  <div className="mt-2 space-y-1">
                    {currentLint.errors.slice(0, 3).map((e, i) => (
                      <div key={i} className="text-[10px] text-red-400 font-mono truncate">{e.message}</div>
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* Connection Card */}
            <div className={`rounded-lg border p-3 transition-all duration-500 ${
              connected ? "border-emerald-500/30 bg-emerald-500/5" : "border-zinc-700 bg-zinc-900"
            }`}>
              <div className="flex items-center gap-2">
                <TerminalIcon className="size-4" />
                <span className="text-sm font-semibold">Claude Code</span>
                <Pulse active={connected} />
              </div>
              <p className="mt-1 text-[10px] text-muted-foreground">
                {connected
                  ? "Terminal interactif connecté — toutes les fonctionnalités Claude Code disponibles."
                  : "Connexion au terminal en cours..."
                }
              </p>
            </div>

            {/* Sessions History */}
            {currentSessions.length > 0 && (
              <div className="rounded-lg border border-border bg-card p-3">
                <div className="flex items-center gap-2">
                  <TerminalIcon className="size-4 text-muted-foreground" />
                  <span className="text-sm font-semibold">Sessions ({currentSessions.length})</span>
                </div>
                <div className="mt-2 space-y-1">
                  {currentSessions.slice(0, 10).map((s) => {
                    const parts = s.name.replace("sprint", "S").split("_");
                    const sprint = parts[0] ?? "";
                    const phase = parts.length > 2 ? parts.slice(1, 3).join(" ") : "";
                    return (
                      <div key={s.name} className="flex items-center justify-between rounded px-1.5 py-0.5 text-[10px] font-mono hover:bg-zinc-800/50">
                        <span className="text-zinc-400 truncate">{sprint} {phase}</span>
                        <span className="text-zinc-600 shrink-0 ml-2">{(s.size_bytes / 1024).toFixed(0)}KB</span>
                      </div>
                    );
                  })}
                </div>
              </div>
            )}

            {/* Loading state */}
            {status.loading && (
              <div className="flex items-center justify-center py-8">
                <LoaderIcon className="size-5 animate-spin text-muted-foreground" />
              </div>
            )}
          </div>
        </ScrollArea>
      </div>

      <Separator orientation="vertical" />

      {/* RIGHT: Terminal */}
      <div className="flex flex-1 flex-col overflow-hidden rounded-lg border border-zinc-800 bg-[#09090b]">
        <div className="flex items-center gap-2 border-b border-zinc-800 px-3 py-1.5">
          <div className="flex gap-1.5">
            <span className={`size-3 rounded-full ${connected ? "bg-emerald-500" : "bg-zinc-600"}`} />
            <span className="size-3 rounded-full bg-zinc-600" />
            <span className="size-3 rounded-full bg-zinc-600" />
          </div>
          <span className="ml-2 font-mono text-[11px] text-zinc-500">
            claude — {currentStatus ? `Sprint ${currentStatus.sprint}` : "..."}
          </span>
          {connected && (
            <Badge variant="outline" className="ml-auto border-emerald-500/30 font-mono text-[9px] text-emerald-400">
              LIVE
            </Badge>
          )}
        </div>
        <div ref={termRef} className="flex-1" />
      </div>
    </div>
  );
}
