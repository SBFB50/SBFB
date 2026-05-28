// SPDX-License-Identifier: AGPL-3.0-or-later

import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { postApi } from "@/hooks/useApi";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card, CardContent,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  MessageCircleIcon, SendIcon, LoaderIcon, BotIcon, UserIcon,
  PlusCircleIcon, ZapIcon, MessageSquareIcon, BugIcon, ChevronDownIcon,
  ChevronRightIcon, TerminalIcon,
} from "lucide-react";

interface Message {
  role: "user" | "agent";
  content: string;
  action?: string;
  thinking?: string;
  timestamp: string;
  cost?: number;
  duration?: number;
}

interface DebugEntry {
  time: string;
  label: string;
  content: string;
}

interface StreamEvent {
  type: "delta" | "thinking" | "done" | "error" | "debug";
  text?: string;
  message?: string;
  cost_usd?: number;
  duration_ms?: number;
  result?: string;
  label?: string;
  content?: string;
}

export function AgentChat() {
  const { t } = useTranslation();
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [streamingText, setStreamingText] = useState("");
  const [thinkingText, setThinkingText] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const [debugMode, setDebugMode] = useState(() =>
    localStorage.getItem("factory-debug") === "true",
  );
  const [debugLog, setDebugLog] = useState<DebugEntry[]>([]);
  const [debugOpen, setDebugOpen] = useState(true);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const debugScrollRef = useRef<HTMLDivElement>(null);
  const eventSourceRef = useRef<EventSource | null>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages, streamingText]);

  useEffect(() => {
    if (debugScrollRef.current) {
      debugScrollRef.current.scrollTop = debugScrollRef.current.scrollHeight;
    }
  }, [debugLog]);

  useEffect(() => {
    if (sessionId && inputRef.current) {
      inputRef.current.focus();
    }
  }, [sessionId]);

  useEffect(() => {
    localStorage.setItem("factory-debug", String(debugMode));
  }, [debugMode]);

  useEffect(() => {
    return () => {
      eventSourceRef.current?.close();
    };
  }, []);

  const addDebug = useCallback((label: string, content: string) => {
    setDebugLog((prev) => [
      ...prev,
      { time: new Date().toLocaleTimeString("fr-FR", { hour12: false, fractionalSecondDigits: 3 }), label, content },
    ]);
  }, []);

  const startSession = useCallback(async () => {
    setLoading(true);
    setDebugLog([]);
    try {
      addDebug("api", "POST /api/chat/session");
      const json = await postApi<{ id: string; context_pack: unknown }>("/chat/session", {
        context_pack: {},
      });
      setSessionId(json.id);
      setMessages([]);
      addDebug("session", json.id);
      addDebug("context_pack", JSON.stringify(json.context_pack, null, 2));
    } catch (e) {
      addDebug("error", String(e));
      setMessages([{
        role: "agent",
        content: t("chat.sessionError"),
        timestamp: new Date().toLocaleTimeString("fr-FR"),
      }]);
    } finally {
      setLoading(false);
    }
  }, [t, addDebug]);

  const sendMessage = useCallback(async () => {
    if (!input.trim() || !sessionId || isStreaming) return;
    const userMsg = input.trim();
    setInput("");
    const now = new Date().toLocaleTimeString("fr-FR");
    setMessages((prev) => [...prev, { role: "user", content: userMsg, timestamp: now }]);
    setLoading(true);

    const provider = localStorage.getItem("factory-driver") ?? "claude";
    const model = provider === "claude" ? "sonnet" : "";

    addDebug("user_input", userMsg);
    addDebug("provider", `${provider} / model=${model}`);

    try {
      addDebug("api", `POST /api/chat/${sessionId}/send`);
      const sendResult = await postApi<{ ok: boolean; requires_gate?: boolean }>(`/chat/${sessionId}/send`, {
        message: userMsg,
        provider,
        model,
      });

      addDebug("send_result", JSON.stringify(sendResult));

      if (!sendResult.ok) {
        setMessages((prev) => [
          ...prev,
          {
            role: "agent",
            content: t("chat.gateRequired"),
            action: "requires_gate",
            timestamp: new Date().toLocaleTimeString("fr-FR"),
          },
        ]);
        setLoading(false);
        return;
      }

      if (provider !== "claude") {
        setMessages((prev) => [
          ...prev,
          {
            role: "agent",
            content: t("chat.providerNotConnected"),
            timestamp: new Date().toLocaleTimeString("fr-FR"),
          },
        ]);
        setLoading(false);
        return;
      }

      setIsStreaming(true);
      setStreamingText("");
      setThinkingText("");

      const sseUrl = `/api/chat/${sessionId}/stream`;
      addDebug("sse", `EventSource → ${sseUrl}`);

      eventSourceRef.current?.close();
      const es = new EventSource(sseUrl);
      eventSourceRef.current = es;

      let accumulated = "";
      let accumulatedThinking = "";

      es.onmessage = (event) => {
        try {
          const data: StreamEvent = JSON.parse(event.data);

          switch (data.type) {
            case "debug":
              addDebug(data.label ?? "debug", data.content ?? "");
              break;

            case "delta":
              accumulated += data.text ?? "";
              setStreamingText(accumulated);
              break;

            case "thinking":
              accumulatedThinking += data.text ?? "";
              setThinkingText(accumulatedThinking);
              addDebug("thinking", data.text ?? "");
              break;

            case "done":
              es.close();
              eventSourceRef.current = null;
              setIsStreaming(false);
              setStreamingText("");
              setThinkingText("");
              setLoading(false);
              addDebug("done", `cost=$${data.cost_usd?.toFixed(4)} duration=${data.duration_ms}ms`);
              setMessages((prev) => [
                ...prev,
                {
                  role: "agent",
                  content: data.result ?? accumulated,
                  timestamp: new Date().toLocaleTimeString("fr-FR"),
                  cost: data.cost_usd,
                  duration: data.duration_ms,
                  thinking: accumulatedThinking || undefined,
                },
              ]);
              break;

            case "error":
              es.close();
              eventSourceRef.current = null;
              setIsStreaming(false);
              setStreamingText("");
              setLoading(false);
              addDebug("error", data.message ?? "unknown");
              setMessages((prev) => [
                ...prev,
                {
                  role: "agent",
                  content: data.message ?? t("chat.commError"),
                  action: "error",
                  timestamp: new Date().toLocaleTimeString("fr-FR"),
                },
              ]);
              break;
          }
        } catch {
          // ignore unparseable events
        }
      };

      es.onerror = () => {
        es.close();
        eventSourceRef.current = null;
        addDebug("sse_error", "EventSource connection closed");
        if (isStreaming) {
          setIsStreaming(false);
          setLoading(false);
          if (accumulated) {
            setMessages((prev) => [
              ...prev,
              {
                role: "agent",
                content: accumulated,
                timestamp: new Date().toLocaleTimeString("fr-FR"),
              },
            ]);
          }
          setStreamingText("");
        }
      };

    } catch (e) {
      addDebug("error", String(e));
      setMessages((prev) => [
        ...prev,
        {
          role: "agent",
          content: t("chat.commError"),
          timestamp: new Date().toLocaleTimeString("fr-FR"),
        },
      ]);
      setLoading(false);
    }
  }, [input, sessionId, isStreaming, t, addDebug]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }, [sendMessage]);

  if (!sessionId) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-6 py-20">
        <div className="flex size-16 items-center justify-center rounded-2xl bg-primary/10">
          <MessageCircleIcon className="size-8 text-primary" />
        </div>
        <div className="max-w-sm text-center">
          <h1 className="text-xl font-bold">{t("chat.title")}</h1>
          <p className="mt-2 text-sm text-muted-foreground">{t("chat.noSession")}</p>
        </div>
        <div className="flex items-center gap-3">
          <Button onClick={startSession} disabled={loading} size="lg">
            {loading
              ? <LoaderIcon className="size-4 animate-spin" />
              : <PlusCircleIcon className="size-4" />
            }
            {t("chat.startSession")}
          </Button>
          <Button
            variant={debugMode ? "default" : "outline"}
            size="sm"
            onClick={() => setDebugMode((v) => !v)}
            title="Mode développeur"
          >
            <BugIcon className="size-4" />
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-[calc(100vh-8rem)] flex-col">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <MessageCircleIcon className="size-5 text-primary" />
          <h1 className="text-xl font-bold">{t("chat.title")}</h1>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant={debugMode ? "default" : "ghost"}
            size="sm"
            onClick={() => setDebugMode((v) => !v)}
            title="Mode développeur"
            className="size-8 p-0"
          >
            <BugIcon className="size-3.5" />
          </Button>
          <Badge variant="outline" className="font-mono text-xs">{sessionId.slice(0, 8)}</Badge>
          <Badge variant="secondary">
            {messages.filter((m) => m.role === "user").length} {t("chat.messagesCount")}
          </Badge>
        </div>
      </div>

      <Separator className="my-3" />

      <div className={`flex flex-1 gap-3 overflow-hidden ${debugMode ? "" : "flex-col"}`}>
        <ScrollArea className={`rounded-lg border border-border bg-background ${debugMode ? "w-1/2" : "flex-1"}`}>
          <div ref={scrollRef} className="space-y-1 p-4">
            {messages.length === 0 && !isStreaming && (
              <div className="flex flex-col items-center justify-center py-16">
                <MessageSquareIcon className="mb-3 size-10 text-muted-foreground/30" />
                <p className="text-sm text-muted-foreground">{t("chat.emptyConversation")}</p>
              </div>
            )}

            {messages.map((msg, i) => (
              <div
                key={i}
                className={`flex gap-3 ${msg.role === "user" ? "flex-row-reverse" : "flex-row"}`}
              >
                <div className={`flex size-8 shrink-0 items-center justify-center rounded-full ${
                  msg.role === "user"
                    ? "bg-primary/15 text-primary"
                    : "bg-muted text-muted-foreground"
                }`}>
                  {msg.role === "user"
                    ? <UserIcon className="size-4" />
                    : <BotIcon className="size-4" />
                  }
                </div>

                <div className={`max-w-[75%] space-y-1 ${msg.role === "user" ? "items-end text-right" : ""}`}>
                  <div className={`flex items-center gap-2 ${msg.role === "user" ? "flex-row-reverse" : ""}`}>
                    <span className="text-xs font-semibold">
                      {msg.role === "user" ? t("chat.operator") : t("chat.agent")}
                    </span>
                    <span className="text-xs text-muted-foreground">{msg.timestamp}</span>
                    {msg.cost != null && (
                      <span className="text-xs text-muted-foreground">
                        ${msg.cost.toFixed(4)} · {((msg.duration ?? 0) / 1000).toFixed(1)}s
                      </span>
                    )}
                  </div>

                  {msg.thinking && (
                    <details className="rounded-lg bg-zinc-900/50 text-xs ring-1 ring-zinc-700/50">
                      <summary className="cursor-pointer px-3 py-1.5 font-mono text-zinc-500 hover:text-zinc-300">
                        thinking ({msg.thinking.length} chars)
                      </summary>
                      <pre className="max-h-48 overflow-auto whitespace-pre-wrap px-3 pb-2 font-mono text-zinc-500">
                        {msg.thinking}
                      </pre>
                    </details>
                  )}

                  <div className={`rounded-xl px-3.5 py-2.5 text-sm ${
                    msg.role === "user"
                      ? "rounded-tr-sm bg-primary/15 text-foreground"
                      : "rounded-tl-sm bg-card text-foreground ring-1 ring-border"
                  }`}>
                    <p className="whitespace-pre-wrap text-left">{msg.content}</p>
                  </div>

                  {msg.action && (
                    <Card className="mt-1.5">
                      <CardContent className="flex items-center gap-2 p-2">
                        <ZapIcon className="size-3.5 text-[var(--yellow)]" />
                        <span className="font-mono text-xs text-muted-foreground">{msg.action}</span>
                      </CardContent>
                    </Card>
                  )}
                </div>
              </div>
            ))}

            {isStreaming && (thinkingText || streamingText) && (
              <div className="flex gap-3">
                <div className="flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground">
                  <BotIcon className="size-4" />
                </div>
                <div className="max-w-[75%] space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-semibold">{t("chat.agent")}</span>
                    <LoaderIcon className="size-3 animate-spin text-muted-foreground" />
                  </div>

                  {thinkingText && (
                    <div className="rounded-lg bg-zinc-900/50 px-3 py-2 text-xs ring-1 ring-zinc-700/50">
                      <div className="mb-1 font-mono text-[10px] text-zinc-500">thinking...</div>
                      <pre className="max-h-32 overflow-auto whitespace-pre-wrap font-mono text-zinc-400">
                        {thinkingText}
                      </pre>
                    </div>
                  )}

                  {streamingText && (
                    <div className="rounded-xl rounded-tl-sm bg-card px-3.5 py-2.5 text-sm text-foreground ring-1 ring-border">
                      <p className="whitespace-pre-wrap text-left">{streamingText}</p>
                    </div>
                  )}
                </div>
              </div>
            )}

            {loading && !isStreaming && (
              <div className="flex gap-3">
                <div className="flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground">
                  <BotIcon className="size-4" />
                </div>
                <div className="flex items-center gap-2 rounded-xl rounded-tl-sm bg-card px-3.5 py-2.5 ring-1 ring-border">
                  <LoaderIcon className="size-3.5 animate-spin text-muted-foreground" />
                  <span className="text-xs text-muted-foreground">{t("chat.thinking")}</span>
                </div>
              </div>
            )}
          </div>
        </ScrollArea>

        {debugMode && (
          <div className="flex w-1/2 flex-col rounded-lg border border-border bg-zinc-950">
            <button
              type="button"
              className="flex items-center gap-2 px-3 py-2 text-left text-xs font-semibold text-zinc-400 hover:text-zinc-200"
              onClick={() => setDebugOpen((v) => !v)}
            >
              {debugOpen ? <ChevronDownIcon className="size-3" /> : <ChevronRightIcon className="size-3" />}
              <TerminalIcon className="size-3" />
              Debug ({debugLog.length} events)
              {debugLog.length > 0 && (
                <button
                  type="button"
                  className="ml-auto text-xs text-zinc-600 hover:text-zinc-400"
                  onClick={(e) => { e.stopPropagation(); setDebugLog([]); }}
                >
                  clear
                </button>
              )}
            </button>
            {debugOpen && (
              <div ref={debugScrollRef} className="flex-1 overflow-auto px-3 pb-3">
                {debugLog.map((entry, i) => (
                  <div key={i} className="mb-1 font-mono text-[11px] leading-tight">
                    <span className="text-zinc-600">{entry.time}</span>
                    {" "}
                    <span className={
                      entry.label === "error" || entry.label === "sse_error"
                        ? "text-red-400"
                        : entry.label === "prompt"
                          ? "text-blue-400"
                          : entry.label.startsWith("ndjson")
                            ? "text-zinc-500"
                            : entry.label === "done"
                              ? "text-green-400"
                              : "text-amber-400"
                    }>
                      [{entry.label}]
                    </span>
                    {" "}
                    <span className={
                      entry.label.startsWith("ndjson") ? "text-zinc-600" : "text-zinc-300"
                    }>
                      {entry.content.length > 500 && entry.label.startsWith("ndjson")
                        ? entry.content.slice(0, 500) + "…"
                        : entry.content
                      }
                    </span>
                  </div>
                ))}
                {debugLog.length === 0 && (
                  <p className="text-xs text-zinc-600">En attente d&apos;events...</p>
                )}
              </div>
            )}
          </div>
        )}
      </div>

      <div className="mt-3 flex gap-2">
        <Input
          ref={inputRef}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={t("chat.placeholder")}
          onKeyDown={handleKeyDown}
          disabled={loading || isStreaming}
          aria-label={t("chat.placeholder")}
          className="flex-1"
        />
        <Button
          onClick={sendMessage}
          disabled={loading || isStreaming || !input.trim()}
          aria-label={t("chat.send")}
        >
          {loading
            ? <LoaderIcon className="size-4 animate-spin" />
            : <SendIcon className="size-4" />
          }
          {t("chat.send")}
        </Button>
      </div>
    </div>
  );
}
