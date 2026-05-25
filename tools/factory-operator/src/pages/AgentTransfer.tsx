// SPDX-License-Identifier: AGPL-3.0-or-later

import { useState } from "react";
import { useTranslation } from "react-i18next";
import { postApi } from "@/hooks/useApi";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  Tooltip, TooltipTrigger, TooltipContent, TooltipProvider,
} from "@/components/ui/tooltip";
import { TechnicalDetails } from "@/components/TechnicalDetails";

const PROVIDERS = ["claude", "codex", "gpt", "local"] as const;

export function AgentTransfer() {
  const { t } = useTranslation();
  const [provider, setProvider] = useState("claude");
  const [role, setRole] = useState("driver");
  const [result, setResult] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);
  const [lastAction, setLastAction] = useState<"handoff" | "context-pack" | null>(null);

  async function handleGenerate() {
    setLoading(true);
    setResult(null);
    setLastAction("handoff");
    try {
      const res = await fetch("/api/prompt/handoff");
      if (!res.ok) throw new Error(`${res.status}`);
      const json = await res.json();
      setResult(typeof json.prompt === "string" ? json.prompt : JSON.stringify(json, null, 2));
    } catch {
      setResult(t("transfer.errorHandoff"));
    } finally {
      setLoading(false);
    }
  }

  async function handleContextPack() {
    setLoading(true);
    setResult(null);
    setLastAction("context-pack");
    try {
      const json = await postApi<{ pack: string }>("/context-pack", { provider, role });
      setResult(typeof json.pack === "string" ? json.pack : JSON.stringify(json, null, 2));
    } catch {
      setResult(t("transfer.errorContextPack"));
    } finally {
      setLoading(false);
    }
  }

  function handleCopy() {
    if (result) {
      navigator.clipboard.writeText(result);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  }

  return (
    <TooltipProvider>
      <div className="space-y-6">
        <div>
          <h1 className="text-xl font-bold">{t("transfer.title")}</h1>
          <p className="text-sm text-muted-foreground">{t("transfer.description")}</p>
        </div>

        <div className="grid gap-4 sm:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle className="text-sm">{t("transfer.targetProvider")}</CardTitle>
              <CardDescription>{t("transfer.providerHint")}</CardDescription>
            </CardHeader>
            <CardContent>
              <Select value={provider} onValueChange={(v) => v && setProvider(v)}>
                <SelectTrigger aria-label={t("transfer.targetProvider")}>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {PROVIDERS.map((p) => (
                    <SelectItem key={p} value={p}>{t(`agents.${p}`)}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-sm">{t("transfer.role")}</CardTitle>
              <CardDescription>{t("transfer.roleHint")}</CardDescription>
            </CardHeader>
            <CardContent>
              <Select value={role} onValueChange={(v) => v && setRole(v)}>
                <SelectTrigger aria-label={t("transfer.role")}>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="driver">{t("transfer.roleDriver")}</SelectItem>
                  <SelectItem value="verifier">{t("transfer.roleVerifier")}</SelectItem>
                </SelectContent>
              </Select>
            </CardContent>
          </Card>
        </div>

        <Separator />

        <div className="flex flex-wrap gap-3">
          <Tooltip>
            <TooltipTrigger render={<span />}>
              <Button
                onClick={handleGenerate}
                disabled={loading}
                aria-label={t("transfer.generateHandoff")}
              >
                {loading && lastAction === "handoff" ? t("status.loading") : t("transfer.generateHandoff")}
              </Button>
            </TooltipTrigger>
            <TooltipContent>{t("transfer.handoffTooltip")}</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger render={<span />}>
              <Button
                variant="outline"
                onClick={handleContextPack}
                disabled={loading}
                aria-label={t("transfer.newContextPack")}
              >
                {loading && lastAction === "context-pack" ? t("status.loading") : t("transfer.newContextPack")}
              </Button>
            </TooltipTrigger>
            <TooltipContent>{t("transfer.contextPackTooltip")}</TooltipContent>
          </Tooltip>
        </div>

        <TechnicalDetails command={`sbfb-factory process prompt --kind handoff --provider ${provider}`} />

        {loading && !result && (
          <Card>
            <CardContent className="py-8">
              <div className="space-y-3" aria-busy="true" aria-label="Loading">
                <div className="h-4 w-3/4 animate-pulse rounded bg-muted/50" />
                <div className="h-4 w-full animate-pulse rounded bg-muted/50" />
                <div className="h-4 w-5/6 animate-pulse rounded bg-muted/50" />
                <div className="h-4 w-2/3 animate-pulse rounded bg-muted/50" />
              </div>
            </CardContent>
          </Card>
        )}

        {!result && !loading && (
          <Card>
            <CardContent className="py-12 text-center">
              <div className="mx-auto mb-3 flex size-12 items-center justify-center rounded-full bg-muted/30">
                <span className="text-lg text-muted-foreground" aria-hidden="true">
                  {"→"}
                </span>
              </div>
              <p className="text-sm text-muted-foreground">{t("transfer.emptyState")}</p>
            </CardContent>
          </Card>
        )}

        {result && (
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <CardTitle className="text-sm">{t("transfer.result")}</CardTitle>
                  {lastAction && (
                    <Badge variant="secondary">
                      {lastAction === "handoff" ? t("transfer.typeHandoff") : t("transfer.typeContextPack")}
                    </Badge>
                  )}
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleCopy}
                  aria-label={t("transfer.copy")}
                >
                  {copied ? t("transfer.copied") : t("transfer.copy")}
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <ScrollArea className="max-h-96">
                <pre className="whitespace-pre-wrap font-mono text-xs leading-relaxed text-muted-foreground">
                  {result}
                </pre>
              </ScrollArea>
            </CardContent>
          </Card>
        )}
      </div>
    </TooltipProvider>
  );
}
