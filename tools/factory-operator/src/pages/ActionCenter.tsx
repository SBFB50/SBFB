// SPDX-License-Identifier: AGPL-3.0-or-later

import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { postApi } from "@/hooks/useApi";
import { Button } from "@/components/ui/button";
import {
  Card, CardContent, CardDescription, CardHeader, CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Tooltip, TooltipContent, TooltipProvider, TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  ZapIcon, ActivityIcon, FileSearchIcon, GitCommitHorizontalIcon,
  TerminalIcon, ShieldAlertIcon, LoaderIcon, CheckCircleIcon,
  XCircleIcon, ClockIcon, PlayIcon, InboxIcon,
} from "lucide-react";

const ACTION_IDS = ["status-sprint", "lint-planning", "audit-commit", "prompt"] as const;

const ACTION_ICONS: Record<string, typeof ActivityIcon> = {
  "status-sprint": ActivityIcon,
  "lint-planning": FileSearchIcon,
  "audit-commit": GitCommitHorizontalIcon,
  "prompt": TerminalIcon,
};

interface ActionResult {
  action: string;
  result: string;
  timestamp: string;
  success: boolean;
}

export function ActionCenter() {
  const { t } = useTranslation();
  const [results, setResults] = useState<ActionResult[]>([]);
  const [loading, setLoading] = useState<string | null>(null);

  const handleRun = useCallback(async (actionId: string) => {
    setLoading(actionId);
    try {
      const json = await postApi<{ result: string }>("/actions/run", { command: actionId });
      setResults((prev) => [{
        action: actionId,
        result: typeof json.result === "string" ? json.result : JSON.stringify(json),
        timestamp: new Date().toLocaleTimeString("fr-FR"),
        success: true,
      }, ...prev]);
    } catch (err) {
      setResults((prev) => [{
        action: actionId,
        result: err instanceof Error ? err.message : t("status.error"),
        timestamp: new Date().toLocaleTimeString("fr-FR"),
        success: false,
      }, ...prev]);
    } finally {
      setLoading(null);
    }
  }, [t]);

  return (
    <div className="space-y-6">
      <div>
        <div className="flex items-center gap-2">
          <ZapIcon className="size-5 text-primary" />
          <h1 className="text-xl font-bold">{t("actionCenter.title")}</h1>
        </div>
        <p className="mt-1 text-sm text-muted-foreground">{t("actionCenter.subtitle")}</p>
      </div>

      <Separator />

      <div className="grid gap-3 sm:grid-cols-2">
        {ACTION_IDS.map((id) => {
          const Icon = ACTION_ICONS[id];
          const isRunning = loading === id;
          return (
            <Card
              key={id}
              className="group cursor-pointer transition-all hover:ring-1 hover:ring-primary/30"
            >
              <button
                type="button"
                onClick={() => handleRun(id)}
                disabled={loading !== null}
                className="w-full p-4 text-left disabled:opacity-50"
                aria-label={t(`actionCenter.actions.${id}`)}
              >
                <div className="flex items-start gap-3">
                  <div className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary transition-colors group-hover:bg-primary/20">
                    {isRunning
                      ? <LoaderIcon className="size-4 animate-spin" />
                      : Icon && <Icon className="size-4" />
                    }
                  </div>
                  <div className="min-w-0 flex-1">
                    <span className="text-sm font-medium">{t(`actionCenter.actions.${id}`)}</span>
                    <p className="mt-0.5 text-xs text-muted-foreground">{t(`actionCenter.actions.${id}_desc`)}</p>
                  </div>
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger
                        render={
                          <Button
                            variant="ghost"
                            size="icon-xs"
                            className="shrink-0 opacity-0 transition-opacity group-hover:opacity-100"
                            tabIndex={-1}
                          />
                        }
                      >
                        <PlayIcon className="size-3" />
                      </TooltipTrigger>
                      <TooltipContent>{t("actionCenter.runAction")}</TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                </div>
              </button>
            </Card>
          );
        })}
      </div>

      <Card className="border-[var(--yellow)]/20 bg-[var(--yellow)]/5">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-sm">
            <ShieldAlertIcon className="size-4 text-[var(--yellow)]" />
            {t("actionCenter.sensitiveTitle")}
          </CardTitle>
          <CardDescription>{t("actionCenter.sensitiveHint")}</CardDescription>
        </CardHeader>
      </Card>

      <Separator />

      <div className="space-y-3">
        <h2 className="flex items-center gap-2 text-sm font-semibold">
          <ClockIcon className="size-4 text-muted-foreground" />
          {t("actionCenter.results")}
          {results.length > 0 && (
            <Badge variant="secondary">{results.length}</Badge>
          )}
        </h2>

        {results.length === 0 ? (
          <Card>
            <CardContent className="flex flex-col items-center justify-center py-10">
              <InboxIcon className="mb-3 size-10 text-muted-foreground/40" />
              <p className="text-sm text-muted-foreground">{t("actionCenter.noResults")}</p>
            </CardContent>
          </Card>
        ) : (
          <ScrollArea className="max-h-[50vh]">
            <div className="space-y-2">
              {results.map((r, i) => (
                <Card key={i} size="sm">
                  <CardContent className="p-3">
                    <div className="mb-2 flex items-center gap-2">
                      {r.success
                        ? <CheckCircleIcon className="size-3.5 text-[var(--green)]" />
                        : <XCircleIcon className="size-3.5 text-destructive" />
                      }
                      <Badge variant="outline" className="font-mono text-xs">{r.action}</Badge>
                      <span className="ml-auto text-xs text-muted-foreground">{r.timestamp}</span>
                    </div>
                    <ScrollArea className="max-h-48">
                      <pre className="whitespace-pre-wrap rounded-md bg-background p-2 font-mono text-xs text-muted-foreground">{r.result}</pre>
                    </ScrollArea>
                  </CardContent>
                </Card>
              ))}
            </div>
          </ScrollArea>
        )}
      </div>
    </div>
  );
}
