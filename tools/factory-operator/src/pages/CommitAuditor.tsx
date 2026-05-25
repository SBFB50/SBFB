// SPDX-License-Identifier: AGPL-3.0-or-later

import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  Tooltip, TooltipTrigger, TooltipContent, TooltipProvider,
} from "@/components/ui/tooltip";

interface AuditData {
  rev: string;
  sections_present: string[];
  sections_missing: string[];
  review_check: boolean;
  codex_check: boolean;
}

function AuditSkeleton() {
  return (
    <div className="space-y-4" aria-busy="true" aria-label="Loading">
      <div className="h-5 w-64 animate-pulse rounded bg-muted/50" />
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="h-40 animate-pulse rounded-xl bg-muted/50" />
        <div className="h-40 animate-pulse rounded-xl bg-muted/50" />
      </div>
      <div className="flex gap-4">
        <div className="h-6 w-24 animate-pulse rounded bg-muted/50" />
        <div className="h-6 w-24 animate-pulse rounded bg-muted/50" />
      </div>
    </div>
  );
}

export function CommitAuditor() {
  const { t } = useTranslation();
  const [rev, setRev] = useState("");
  const [data, setData] = useState<AuditData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleAudit() {
    if (!rev.trim()) return;
    setLoading(true);
    setError(null);
    setData(null);
    try {
      const res = await fetch(`/api/audit/${encodeURIComponent(rev.trim())}`);
      if (!res.ok) throw new Error(`${res.status}`);
      setData(await res.json());
    } catch (err) {
      setError(err instanceof Error ? err.message : "Error");
    } finally {
      setLoading(false);
    }
  }

  const allPresent = data && data.sections_missing.length === 0 && data.review_check && data.codex_check;

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold">{t("audit.title")}</h1>
        <p className="text-sm text-muted-foreground">{t("audit.description")}</p>
      </div>

      <Card>
        <CardContent className="flex gap-2 pt-1">
          <Input
            value={rev}
            onChange={(e) => setRev(e.target.value)}
            placeholder={t("audit.placeholder")}
            className="flex-1 font-mono"
            aria-label={t("audit.placeholder")}
            onKeyDown={(e) => e.key === "Enter" && handleAudit()}
          />
          <Button
            onClick={handleAudit}
            disabled={loading || !rev.trim()}
            aria-label={t("audit.button")}
          >
            {loading ? t("status.loading") : t("audit.button")}
          </Button>
        </CardContent>
      </Card>

      {error && (
        <Card>
          <CardContent className="py-6 text-center">
            <p className="text-sm text-destructive">{t("audit.errorPrefix")}: {error}</p>
            <p className="mt-1 text-xs text-muted-foreground">{t("audit.errorHint")}</p>
          </CardContent>
        </Card>
      )}

      {loading && !data && <AuditSkeleton />}

      {!data && !loading && !error && (
        <Card>
          <CardContent className="py-12 text-center">
            <div className="mx-auto mb-3 flex size-12 items-center justify-center rounded-full bg-muted/30">
              <span className="text-lg text-muted-foreground" aria-hidden="true">
                {"#"}
              </span>
            </div>
            <p className="text-sm text-muted-foreground">{t("audit.emptyState")}</p>
          </CardContent>
        </Card>
      )}

      {data && (
        <TooltipProvider>
          <div className="space-y-4">
            <div className="flex items-center gap-3">
              <Tooltip>
                <TooltipTrigger className="cursor-default font-mono text-sm text-muted-foreground transition-colors hover:text-foreground">
                  {data.rev.slice(0, 12)}
                </TooltipTrigger>
                <TooltipContent>{data.rev}</TooltipContent>
              </Tooltip>
              {allPresent ? (
                <Badge variant="default">{t("verdict.PASS")}</Badge>
              ) : (
                <Badge variant="destructive">{t("verdict.FAIL")}</Badge>
              )}
            </div>

            <Separator />

            <div className="grid gap-4 sm:grid-cols-2">
              <Card>
                <CardHeader>
                  <CardTitle className="text-sm text-[var(--green)]">
                    {t("audit.present")}
                  </CardTitle>
                  <CardDescription>
                    {t("audit.sectionCount", { count: data.sections_present.length })}
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  {data.sections_present.length > 0 ? (
                    <ul className="space-y-1.5">
                      {data.sections_present.map((s) => (
                        <li key={s} className="flex items-center gap-2 text-xs">
                          <span className="text-[var(--green)]" aria-hidden="true">{"✓"}</span>
                          <span className="font-mono">{s}</span>
                        </li>
                      ))}
                    </ul>
                  ) : (
                    <p className="text-xs text-muted-foreground">{t("audit.none")}</p>
                  )}
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="text-sm text-destructive">
                    {t("audit.missing")}
                  </CardTitle>
                  <CardDescription>
                    {t("audit.sectionCount", { count: data.sections_missing.length })}
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  {data.sections_missing.length > 0 ? (
                    <ul className="space-y-1.5">
                      {data.sections_missing.map((s) => (
                        <li key={s} className="flex items-center gap-2 text-xs">
                          <span className="text-destructive" aria-hidden="true">{"✗"}</span>
                          <span className="font-mono">{s}</span>
                        </li>
                      ))}
                    </ul>
                  ) : (
                    <p className="text-xs text-[var(--green)]">{t("audit.allPresent")}</p>
                  )}
                </CardContent>
              </Card>
            </div>

            <Card size="sm">
              <CardHeader>
                <CardTitle className="text-sm">{t("audit.gateChecks")}</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex gap-6">
                  <Tooltip>
                    <TooltipTrigger className="flex cursor-default items-center gap-2 text-sm transition-colors hover:text-foreground">
                      <Badge variant={data.review_check ? "default" : "destructive"}>
                        {data.review_check ? t("verdict.PASS") : t("verdict.FAIL")}
                      </Badge>
                      <span className="text-muted-foreground">{t("audit.review")}</span>
                    </TooltipTrigger>
                    <TooltipContent>{t("audit.reviewTooltip")}</TooltipContent>
                  </Tooltip>
                  <Tooltip>
                    <TooltipTrigger className="flex cursor-default items-center gap-2 text-sm transition-colors hover:text-foreground">
                      <Badge variant={data.codex_check ? "default" : "destructive"}>
                        {data.codex_check ? t("verdict.PASS") : t("verdict.FAIL")}
                      </Badge>
                      <span className="text-muted-foreground">{t("audit.codexCheck")}</span>
                    </TooltipTrigger>
                    <TooltipContent>{t("audit.codexTooltip")}</TooltipContent>
                  </Tooltip>
                </div>
              </CardContent>
            </Card>
          </div>
        </TooltipProvider>
      )}
    </div>
  );
}
