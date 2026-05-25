// SPDX-License-Identifier: AGPL-3.0-or-later

import { useState, useId } from "react";
import { useTranslation } from "react-i18next";
import { useApi } from "@/hooks/useApi";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { TechnicalDetails } from "@/components/TechnicalDetails";
import {
  Compass,
  Loader2,
  Scan,
  FileSearch,
  ShieldCheck,
  FileText,
  ArrowRightLeft,
  ClipboardCheck,
  Sparkles,
  Copy,
  Check,
  AlertCircle,
} from "lucide-react";

interface StatusData {
  sprint: number;
  phases: Array<{ id: string; label: string; status: string }>;
}

const INTENTION_KINDS = [
  "preflight",
  "phase-review",
  "phase-auditor",
  "commit-body",
  "handoff",
  "audit-gate",
] as const;

const INTENTION_ICONS: Record<(typeof INTENTION_KINDS)[number], typeof Scan> = {
  preflight: Scan,
  "phase-review": FileSearch,
  "phase-auditor": ShieldCheck,
  "commit-body": FileText,
  handoff: ArrowRightLeft,
  "audit-gate": ClipboardCheck,
};

function IntentionButton({
  kind,
  disabled,
  active,
  onClick,
}: {
  kind: (typeof INTENTION_KINDS)[number];
  disabled: boolean;
  active: boolean;
  onClick: () => void;
}) {
  const { t } = useTranslation();
  const Icon = INTENTION_ICONS[kind];

  return (
    <button
      type="button"
      disabled={disabled}
      onClick={onClick}
      aria-pressed={active}
      className="group flex w-full items-start gap-3 rounded-lg border border-border bg-card p-4 text-left transition-all duration-200 hover:border-primary/50 hover:ring-1 hover:ring-primary/20 focus-visible:border-ring focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-pressed:border-primary/40 aria-pressed:bg-primary/5"
    >
      <div className="flex size-8 shrink-0 items-center justify-center rounded-md bg-muted transition-colors group-hover:bg-primary/10">
        <Icon className="size-4 text-muted-foreground transition-colors group-hover:text-primary" />
      </div>
      <div className="min-w-0 flex-1">
        <span className="text-sm font-medium">
          {t(`phase.intentions.${kind}`)}
        </span>
        <p className="mt-0.5 text-xs text-muted-foreground">
          {t(`phase.intentions.${kind}_desc`)}
        </p>
      </div>
    </button>
  );
}

function ResultPanel({
  result,
  onCopy,
  copied,
}: {
  result: string;
  onCopy: () => void;
  copied: boolean;
}) {
  const { t } = useTranslation();

  return (
    <Card className="border-primary/20">
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2 text-sm text-primary">
            <Sparkles className="size-4" />
            {t("phase.result")}
          </CardTitle>
          <Button
            variant="outline"
            size="sm"
            onClick={onCopy}
            className="gap-1.5"
          >
            {copied ? (
              <>
                <Check className="size-3.5" />
                {t("phase.copied")}
              </>
            ) : (
              <>
                <Copy className="size-3.5" />
                {t("phase.copy")}
              </>
            )}
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <ScrollArea className="max-h-96">
          <pre className="whitespace-pre-wrap rounded-md bg-muted/50 p-3 font-mono text-xs text-muted-foreground">
            {result}
          </pre>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}

function ErrorResult({ message }: { message: string }) {
  return (
    <Card className="border-destructive/30">
      <CardContent className="flex items-center gap-3 p-4">
        <AlertCircle className="size-5 shrink-0 text-destructive" />
        <p className="text-sm text-destructive">{message}</p>
      </CardContent>
    </Card>
  );
}

function PhaseSkeleton() {
  return (
    <div className="space-y-6">
      <div className="space-y-2">
        <div className="h-7 w-48 animate-pulse rounded bg-muted" />
        <div className="h-4 w-64 animate-pulse rounded bg-muted" />
      </div>
      <div className="h-8 w-48 animate-pulse rounded bg-muted" />
      <div className="space-y-2">
        {Array.from({ length: 3 }, (_, i) => (
          <div key={i} className="h-20 animate-pulse rounded-lg bg-muted" />
        ))}
      </div>
    </div>
  );
}

export function PhaseAssistant() {
  const { t } = useTranslation();
  const { data, loading: dataLoading } = useApi<StatusData>("/status");
  const [selectedPhase, setSelectedPhase] = useState("__auto__");
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [activeKind, setActiveKind] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const selectId = useId();

  const sprint = data?.sprint ?? 0;
  const phases = data?.phases ?? [];
  const activePhase = phases.find((p) => p.status === "active");
  const currentPhase =
    selectedPhase === "__auto__" ? (activePhase?.id ?? "") : selectedPhase;

  async function handleAction(kind: string) {
    setLoading(true);
    setResult(null);
    setError(null);
    setActiveKind(kind);
    setCopied(false);
    try {
      const res = await fetch(`/api/prompt/${encodeURIComponent(kind)}`);
      if (!res.ok) throw new Error(`${res.status}`);
      const json = await res.json();
      setResult(
        typeof json.prompt === "string"
          ? json.prompt
          : JSON.stringify(json, null, 2),
      );
    } catch {
      setError(t("phase.error"));
    } finally {
      setLoading(false);
    }
  }

  function handleCopy() {
    if (!result) return;
    navigator.clipboard.writeText(result).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }

  if (dataLoading) return <PhaseSkeleton />;

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-lg">
            <Compass className="size-5 text-primary" />
            {t("phase.title")}
          </CardTitle>
          <CardDescription>
            {currentPhase
              ? t("phase.currentPhase", {
                  sprint,
                  phase: currentPhase.toUpperCase(),
                })
              : t("sprint.title", { number: sprint })}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-3">
            <label
              htmlFor={selectId}
              className="text-sm font-medium text-muted-foreground"
            >
              {t("phase.selectPhase")}
            </label>
            <Select
              value={selectedPhase}
              onValueChange={(v) => v && setSelectedPhase(v)}
            >
              <SelectTrigger id={selectId} className="w-48">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="__auto__">
                  {t("phase.autoDetect")}
                  {activePhase && (
                    <Badge variant="secondary" className="ml-2 text-[10px]">
                      {activePhase.label}
                    </Badge>
                  )}
                </SelectItem>
                {phases.map((p) => (
                  <SelectItem key={p.id} value={p.id}>
                    Phase {p.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </CardContent>
      </Card>

      <section aria-label={t("phase.whatToDo")}>
        <h2 className="mb-3 text-sm font-semibold">{t("phase.whatToDo")}</h2>
        <div className="grid gap-2 sm:grid-cols-2">
          {INTENTION_KINDS.map((kind) => (
            <div key={kind}>
              <IntentionButton
                kind={kind}
                disabled={loading}
                active={activeKind === kind}
                onClick={() => handleAction(kind)}
              />
              <TechnicalDetails
                command={`sbfb-factory process prompt --kind ${kind} --sprint ${sprint} --phase ${currentPhase} --provider local`}
              />
            </div>
          ))}
        </div>
      </section>

      {loading && (
        <Card>
          <CardContent className="flex items-center justify-center gap-3 py-8">
            <Loader2 className="size-5 animate-spin text-primary" />
            <span className="text-sm text-muted-foreground">
              {t("phase.generating")}
            </span>
          </CardContent>
        </Card>
      )}

      {error && <ErrorResult message={error} />}

      {result && !loading && (
        <>
          <Separator />
          <ResultPanel
            result={result}
            onCopy={handleCopy}
            copied={copied}
          />
        </>
      )}
    </div>
  );
}
