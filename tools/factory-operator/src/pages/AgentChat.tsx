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
  PlusCircleIcon, ZapIcon, MessageSquareIcon,
} from "lucide-react";

interface Message {
  role: "user" | "agent";
  content: string;
  action?: string;
  timestamp: string;
}

export function AgentChat() {
  const { t } = useTranslation();
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages]);

  useEffect(() => {
    if (sessionId && inputRef.current) {
      inputRef.current.focus();
    }
  }, [sessionId]);

  const startSession = useCallback(async () => {
    setLoading(true);
    try {
      const json = await postApi<{ id: string }>("/chat/session", {
        context_pack: {},
      });
      setSessionId(json.id);
      setMessages([]);
    } catch {
      setMessages([{
        role: "agent",
        content: t("chat.sessionError"),
        timestamp: new Date().toLocaleTimeString("fr-FR"),
      }]);
    } finally {
      setLoading(false);
    }
  }, [t]);

  const sendMessage = useCallback(async () => {
    if (!input.trim() || !sessionId) return;
    const userMsg = input.trim();
    setInput("");
    const now = new Date().toLocaleTimeString("fr-FR");
    setMessages((prev) => [...prev, { role: "user", content: userMsg, timestamp: now }]);
    setLoading(true);
    try {
      const json = await postApi<{ response: string; action?: string }>("/chat/message", {
        session_id: sessionId,
        message: userMsg,
      });
      setMessages((prev) => [
        ...prev,
        {
          role: "agent",
          content: json.response,
          action: json.action,
          timestamp: new Date().toLocaleTimeString("fr-FR"),
        },
      ]);
    } catch {
      setMessages((prev) => [
        ...prev,
        {
          role: "agent",
          content: t("chat.commError"),
          timestamp: new Date().toLocaleTimeString("fr-FR"),
        },
      ]);
    } finally {
      setLoading(false);
    }
  }, [input, sessionId, t]);

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
        <Button onClick={startSession} disabled={loading} size="lg">
          {loading
            ? <LoaderIcon className="size-4 animate-spin" />
            : <PlusCircleIcon className="size-4" />
          }
          {t("chat.startSession")}
        </Button>
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
          <Badge variant="outline" className="font-mono text-xs">{sessionId.slice(0, 8)}</Badge>
          <Badge variant="secondary">
            {messages.filter((m) => m.role === "user").length} {t("chat.messagesCount")}
          </Badge>
        </div>
      </div>

      <Separator className="my-3" />

      <ScrollArea className="flex-1 rounded-lg border border-border bg-background">
        <div ref={scrollRef} className="space-y-1 p-4">
          {messages.length === 0 && (
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
                </div>

                <div className={`rounded-xl px-3.5 py-2.5 text-sm ${
                  msg.role === "user"
                    ? "rounded-tr-sm bg-primary/15 text-foreground"
                    : "rounded-tl-sm bg-card text-foreground ring-1 ring-border"
                }`}>
                  <p className="whitespace-pre-wrap text-left">{msg.content}</p>
                </div>

                {msg.action && (
                  <Card size="sm" className="mt-1.5">
                    <CardContent className="flex items-center gap-2 p-2">
                      <ZapIcon className="size-3.5 text-[var(--yellow)]" />
                      <span className="font-mono text-xs text-muted-foreground">{msg.action}</span>
                    </CardContent>
                  </Card>
                )}
              </div>
            </div>
          ))}

          {loading && (
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

      <div className="mt-3 flex gap-2">
        <Input
          ref={inputRef}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={t("chat.placeholder")}
          onKeyDown={handleKeyDown}
          disabled={loading}
          aria-label={t("chat.placeholder")}
          className="flex-1"
        />
        <Button
          onClick={sendMessage}
          disabled={loading || !input.trim()}
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
