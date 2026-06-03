// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 72 Phase E — provider-routed execution chat (D4/D5).
//
// PLAN-ADAPT (preflight `sprint72_phase_e_preflight.md`): plan §8 assumed
// an existing chat to extend, but the chat-SSE front consumer was removed
// in `c3f4813` and the `/chat` terminal bypasses `ExecutionTarget`. This
// page BUILDS the consumer: a three-intention selector (the execution
// axis, distinct from the prompt-adaptation `AgentSelector`) wired to
// `ChatSendRequest.provider`, plus the `StreamChunk` SSE reader with a
// "running on the network" state for the async (poll) target (PO-14).

import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Cloud, Cpu, Loader, Network, Play, Send } from "lucide-react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  createSession,
  openStream,
  sendMessage,
  type ExecutionIntent,
  type StreamChunk,
} from "@/lib/executionChat";

const INTENTS = ["claude", "ollama", "network"] as const;

const INTENT_ICONS: Record<ExecutionIntent, typeof Cloud> = {
  claude: Cloud,
  ollama: Cpu,
  network: Network,
};

const STORAGE_KEY = "factory-execution-intent";

interface Turn {
  role: "user" | "assistant" | "system";
  content: string;
}

interface Streaming {
  text: string;
  networkStatus: string | null;
}

function isIntent(v: string): v is ExecutionIntent {
  return v === "claude" || v === "ollama" || v === "network";
}

function MessageBubble({ role, content }: Turn) {
  const isUser = role === "user";
  const isSystem = role === "system";
  const tone = isUser
    ? "bg-primary/20 text-foreground"
    : isSystem
      ? "border border-[var(--yellow)]/40 bg-[var(--yellow)]/5 text-foreground"
      : "bg-muted text-foreground";
  return (
    <div className={`flex ${isUser ? "justify-end" : "justify-start"}`}>
      <div
        className={`max-w-[85%] rounded-lg px-3 py-2 text-sm whitespace-pre-wrap ${tone}`}
      >
        {content}
      </div>
    </div>
  );
}

export function ExecutionChat() {
  const { t } = useTranslation();

  const [intent, setIntent] = useState<ExecutionIntent>(() => {
    const saved = localStorage.getItem(STORAGE_KEY);
    return saved && isIntent(saved) ? saved : "claude";
  });
  const [messages, setMessages] = useState<Turn[]>([]);
  const [streaming, setStreaming] = useState<Streaming | null>(null);
  const [input, setInput] = useState("");
  const [busy, setBusy] = useState(false);

  const esRef = useRef<EventSource | null>(null);
  const sessionIdRef = useRef<string | null>(null);
  const accumRef = useRef("");

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, intent);
  }, [intent]);

  // StrictMode double-mounts effects in dev (`main.tsx` is StrictMode):
  // close any open stream on unmount so a navigation never leaks an
  // EventSource (which would also keep auto-reconnecting).
  useEffect(() => {
    return () => {
      esRef.current?.close();
      esRef.current = null;
    };
  }, []);

  const handleSend = useCallback(async () => {
    const text = input.trim();
    if (!text || busy) return;

    // Defuse any stale stream before starting a new turn.
    esRef.current?.close();
    esRef.current = null;

    setBusy(true);
    setInput("");
    setMessages((prev) => [...prev, { role: "user", content: text }]);

    let sessionId: string;
    try {
      if (!sessionIdRef.current) {
        sessionIdRef.current = await createSession(intent);
      }
      sessionId = sessionIdRef.current;
      const res = await sendMessage(sessionId, text, intent);
      if (res.requires_gate) {
        setMessages((prev) => [
          ...prev,
          { role: "system", content: t("execute.gateRequired") },
        ]);
        setBusy(false);
        return;
      }
    } catch {
      // Failed to create the session or queue the turn — reset so the next
      // attempt starts a fresh session.
      sessionIdRef.current = null;
      setMessages((prev) => [
        ...prev,
        { role: "system", content: t("execute.sessionError") },
      ]);
      setBusy(false);
      return;
    }

    accumRef.current = "";
    setStreaming({ text: "", networkStatus: null });

    const es = openStream(sessionId);
    esRef.current = es;
    let finished = false;

    const close = () => {
      finished = true;
      es.close();
      if (esRef.current === es) esRef.current = null;
    };

    es.onmessage = (ev: MessageEvent) => {
      let chunk: StreamChunk;
      try {
        chunk = JSON.parse(ev.data) as StreamChunk;
      } catch {
        return;
      }
      switch (chunk.type) {
        case "delta":
          accumRef.current += chunk.text;
          setStreaming((prev) =>
            prev ? { ...prev, text: accumRef.current } : prev,
          );
          break;
        // `thinking` (reasoning) chunks are not surfaced as text; the
        // empty-state spinner already covers the pre-first-token wait.
        case "thinking":
          break;
        case "debug":
          // The network arm emits one `network-poll` Debug per tick (no
          // token Delta over the WAN, PO-14): surface it as progress.
          if (chunk.label === "network-poll") {
            const status = chunk.content.replace(/^status:\s*/, "").trim();
            setStreaming((prev) =>
              prev ? { ...prev, networkStatus: status } : prev,
            );
          }
          break;
        case "done": {
          const finalText = chunk.result || accumRef.current;
          close();
          setStreaming(null);
          setBusy(false);
          setMessages((prev) => [
            ...prev,
            { role: "assistant", content: finalText },
          ]);
          break;
        }
        case "error":
          close();
          setStreaming(null);
          setBusy(false);
          setMessages((prev) => [
            ...prev,
            {
              role: "system",
              content: t("execute.streamError", { message: chunk.message }),
            },
          ]);
          break;
        case "requires_gate":
          close();
          setStreaming(null);
          setBusy(false);
          setMessages((prev) => [
            ...prev,
            { role: "system", content: t("execute.gateRequired") },
          ]);
          break;
      }
    };

    es.onerror = () => {
      if (finished) return; // the terminal chunk already closed the stream
      close();
      setStreaming(null);
      setBusy(false);
      setMessages((prev) => [
        ...prev,
        { role: "system", content: t("execute.connectionLost") },
      ]);
    };
  }, [input, busy, intent, t]);

  const SelectedIcon = INTENT_ICONS[intent];

  return (
    <div className="space-y-6">
      <div>
        <div className="flex items-center gap-2">
          <Play className="size-5 text-primary" />
          <h1 className="text-xl font-bold">{t("execute.title")}</h1>
        </div>
        <p className="mt-1 text-sm text-muted-foreground">
          {t("execute.description")}
        </p>
      </div>

      <Separator />

      <section aria-label={t("execute.targetLabel")}>
        <h2 className="mb-3 text-sm font-semibold text-muted-foreground">
          {t("execute.targetLabel")}
        </h2>
        <div className="grid gap-3 sm:grid-cols-3">
          {INTENTS.map((it) => {
            const Icon = INTENT_ICONS[it];
            const active = it === intent;
            return (
              <button
                key={it}
                type="button"
                onClick={() => setIntent(it)}
                aria-pressed={active}
                className={`flex flex-col items-start gap-1 rounded-lg border p-4 text-left transition-colors ${
                  active
                    ? "border-primary bg-primary/10 ring-1 ring-primary/30"
                    : "border-border hover:border-primary/40 hover:bg-accent"
                }`}
              >
                <span className="flex items-center gap-2 text-sm font-medium">
                  <Icon className="size-4 text-primary" />
                  {t(`execute.intent.${it}`)}
                </span>
                <span className="text-xs text-muted-foreground">
                  {t(`execute.intentDesc.${it}`)}
                </span>
              </button>
            );
          })}
        </div>
      </section>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-sm">
            <SelectedIcon className="size-4 text-primary" />
            {t(`execute.intent.${intent}`)}
          </CardTitle>
          <CardDescription>{t("execute.conversationHint")}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <ScrollArea className="h-96 rounded-md border border-border bg-background p-3">
            {messages.length === 0 && !streaming ? (
              <p className="p-4 text-center text-sm text-muted-foreground">
                {t("execute.empty")}
              </p>
            ) : (
              <div className="space-y-3">
                {messages.map((m, i) => (
                  <MessageBubble key={i} role={m.role} content={m.content} />
                ))}
                {streaming && (
                  <div className="flex justify-start">
                    <div className="max-w-[85%] space-y-1 rounded-lg bg-muted px-3 py-2 text-sm text-foreground">
                      {streaming.networkStatus !== null && (
                        <div className="flex items-center gap-2 text-xs text-primary">
                          <Loader className="size-3 animate-spin" />
                          {t("execute.networkInProgress")} ·{" "}
                          {t(`execute.networkStatus.${streaming.networkStatus}`, {
                            defaultValue: streaming.networkStatus,
                          })}
                        </div>
                      )}
                      {streaming.text && (
                        <div className="whitespace-pre-wrap">
                          {streaming.text}
                        </div>
                      )}
                      {!streaming.text && streaming.networkStatus === null && (
                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                          <Loader className="size-3 animate-spin" />
                          {t("execute.thinking")}
                        </div>
                      )}
                    </div>
                  </div>
                )}
              </div>
            )}
          </ScrollArea>

          <div className="flex items-center gap-2">
            <Input
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) {
                  e.preventDefault();
                  void handleSend();
                }
              }}
              placeholder={t("execute.placeholder")}
              aria-label={t("execute.placeholder")}
              disabled={busy}
            />
            <Button
              onClick={() => void handleSend()}
              disabled={busy || !input.trim()}
              aria-label={t("execute.send")}
            >
              {busy ? (
                <Loader className="size-4 animate-spin" />
              ) : (
                <Send className="size-4" />
              )}
              {t("execute.send")}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
