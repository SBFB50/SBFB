// SPDX-License-Identifier: AGPL-3.0-or-later

import { useTranslation } from "react-i18next";
import { useApi } from "@/hooks/useApi";
import {
  Card, CardContent, CardHeader, CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  ScrollTextIcon, ClockIcon, InboxIcon, TerminalIcon,
} from "lucide-react";

interface LogEntry {
  timestamp: string;
  action: string;
  args: Record<string, unknown>;
  result: string;
}

function LogEntrySkeleton() {
  return (
    <Card size="sm">
      <CardContent className="p-3">
        <div className="mb-2 flex items-center gap-3">
          <div className="h-3.5 w-20 animate-pulse rounded bg-muted" />
          <div className="h-4 w-28 animate-pulse rounded bg-muted" />
        </div>
        <div className="space-y-1.5">
          <div className="h-3 w-full animate-pulse rounded bg-muted" />
          <div className="h-3 w-3/4 animate-pulse rounded bg-muted" />
        </div>
      </CardContent>
    </Card>
  );
}

export function ActionLog() {
  const { t } = useTranslation();
  const { data, loading, error } = useApi<LogEntry[]>("/actions/log");

  const entries = data ?? [];

  return (
    <div className="space-y-6">
      <div>
        <div className="flex items-center gap-2">
          <ScrollTextIcon className="size-5 text-primary" />
          <h1 className="text-xl font-bold">{t("actionLog.title")}</h1>
          {!loading && entries.length > 0 && (
            <Badge variant="secondary">{entries.length}</Badge>
          )}
        </div>
        <p className="mt-1 text-sm text-muted-foreground">{t("actionLog.subtitle")}</p>
      </div>

      <Separator />

      {error && (
        <Card className="border-destructive/30 bg-destructive/5">
          <CardContent className="p-4 text-sm text-destructive">
            {error}
          </CardContent>
        </Card>
      )}

      {loading ? (
        <div className="space-y-2">
          {Array.from({ length: 4 }).map((_, i) => (
            <LogEntrySkeleton key={i} />
          ))}
        </div>
      ) : entries.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-16">
            <InboxIcon className="mb-3 size-12 text-muted-foreground/30" />
            <p className="text-sm font-medium text-muted-foreground">{t("actionLog.empty")}</p>
            <p className="mt-1 text-xs text-muted-foreground/70">{t("actionLog.emptyHint")}</p>
          </CardContent>
        </Card>
      ) : (
        <ScrollArea className="max-h-[70vh]">
          <div className="space-y-2">
            {entries.map((entry, i) => (
              <Card key={i} size="sm" className="transition-colors hover:ring-1 hover:ring-border">
                <CardContent className="p-3">
                  <div className="mb-2 flex items-center gap-2">
                    <TerminalIcon className="size-3.5 text-muted-foreground" />
                    <Badge variant="outline" className="font-mono text-xs">{entry.action}</Badge>
                    <div className="ml-auto flex items-center gap-1 text-xs text-muted-foreground">
                      <ClockIcon className="size-3" />
                      <time>{entry.timestamp}</time>
                    </div>
                  </div>
                  {Object.keys(entry.args).length > 0 && (
                    <div className="mb-2 flex flex-wrap gap-1">
                      {Object.entries(entry.args).map(([key, val]) => (
                        <Badge key={key} variant="secondary" className="font-mono text-xs">
                          {key}={String(val)}
                        </Badge>
                      ))}
                    </div>
                  )}
                  <ScrollArea className="max-h-48">
                    <pre className="whitespace-pre-wrap rounded-md bg-background p-2 font-mono text-xs text-muted-foreground">{entry.result}</pre>
                  </ScrollArea>
                </CardContent>
              </Card>
            ))}
          </div>
        </ScrollArea>
      )}

      {!loading && entries.length > 0 && (
        <Card size="sm">
          <CardHeader>
            <CardTitle className="text-xs text-muted-foreground">
              {t("actionLog.totalEntries", { count: entries.length })}
            </CardTitle>
          </CardHeader>
        </Card>
      )}
    </div>
  );
}
