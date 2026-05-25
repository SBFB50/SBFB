// SPDX-License-Identifier: AGPL-3.0-or-later

import { useTranslation } from "react-i18next";
import { useApi } from "@/hooks/useApi";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  Tooltip, TooltipTrigger, TooltipContent, TooltipProvider,
} from "@/components/ui/tooltip";

interface LintResult {
  level: string;
  message: string;
  file: string;
  line?: number;
}

interface LintData {
  results: LintResult[];
}

function LintSkeleton() {
  return (
    <div className="space-y-3" aria-busy="true" aria-label="Loading">
      {Array.from({ length: 4 }, (_, i) => (
        <div key={i} className="h-20 animate-pulse rounded-xl bg-muted/50" />
      ))}
    </div>
  );
}

export function LintOperator() {
  const { t } = useTranslation();
  const { data, error, loading } = useApi<LintData>("/lint");

  const results = data?.results ?? [];
  const errorCount = results.filter((r) => r.level === "error").length;
  const warningCount = results.filter((r) => r.level === "warning").length;

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold">{t("lint.title")}</h1>
        <p className="text-sm text-muted-foreground">{t("lint.description")}</p>
      </div>

      {loading ? (
        <LintSkeleton />
      ) : error ? (
        <Card>
          <CardContent className="py-10 text-center">
            <p className="text-sm text-destructive">{error}</p>
            <p className="mt-1 text-xs text-muted-foreground">{t("lint.errorHint")}</p>
          </CardContent>
        </Card>
      ) : results.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center">
            <div className="mx-auto mb-3 flex size-12 items-center justify-center rounded-full bg-[var(--green)]/10">
              <span className="text-lg text-[var(--green)]" aria-hidden="true">
                {"✓"}
              </span>
            </div>
            <p className="text-sm font-medium text-[var(--green)]">{t("lint.noIssues")}</p>
            <p className="mt-1 text-xs text-muted-foreground">{t("lint.noIssuesHint")}</p>
          </CardContent>
        </Card>
      ) : (
        <>
          <div className="flex items-center gap-3">
            <Badge variant="destructive" aria-label={t("lint.errorCount", { count: errorCount })}>
              {errorCount} {t("lint.errorLevel")}
            </Badge>
            <Badge variant="outline" aria-label={t("lint.warningCount", { count: warningCount })}>
              {warningCount} {t("lint.warningLevel")}
            </Badge>
            <span className="text-xs text-muted-foreground">
              {t("lint.totalIssues", { count: results.length })}
            </span>
          </div>

          <Separator />

          <TooltipProvider>
            <div className="space-y-3">
              {results.map((r, i) => (
                <Card
                  key={i}
                  size="sm"
                  className={
                    r.level === "error"
                      ? "border-destructive/30 bg-destructive/5"
                      : "border-[var(--yellow)]/30 bg-[var(--yellow)]/5"
                  }
                >
                  <CardHeader>
                    <div className="flex items-center gap-2">
                      <Badge
                        variant={r.level === "error" ? "destructive" : "outline"}
                      >
                        {r.level === "error" ? t("lint.errorLevel") : t("lint.warningLevel")}
                      </Badge>
                      <Tooltip>
                        <TooltipTrigger
                          className="cursor-default font-mono text-xs text-muted-foreground transition-colors hover:text-foreground"
                        >
                          {r.file}{r.line != null ? `:${r.line}` : ""}
                        </TooltipTrigger>
                        <TooltipContent>
                          {t("lint.fileTooltip", { path: r.file, line: r.line ?? "-" })}
                        </TooltipContent>
                      </Tooltip>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <p className="text-sm leading-relaxed">{r.message}</p>
                  </CardContent>
                </Card>
              ))}
            </div>
          </TooltipProvider>
        </>
      )}
    </div>
  );
}
