// SPDX-License-Identifier: AGPL-3.0-or-later

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
import { Separator } from "@/components/ui/separator";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  GitCommitHorizontal,
  FlaskConical,
  CheckCircle2,
  XCircle,
  ServerCrash,
  Layers,
} from "lucide-react";

interface ApiPhase {
  letter: string;
  has_preflight: boolean;
  has_review: boolean;
  has_codex: boolean;
  review_verdict: string | null;
}

interface PhaseData {
  id: string;
  label: string;
  status: string;
  artifacts: { preflight: boolean; review: boolean; codex: boolean };
}

interface ApiStatusData {
  sprint: number;
  branch: string;
  head: string;
  current_phase: string;
  has_kickoff: boolean;
  has_plan: boolean;
  has_design_review: boolean;
  has_audit_plan: boolean;
  phases: ApiPhase[];
}

interface StatusData {
  sprint: number;
  title: string;
  head: string;
  phases: PhaseData[];
  test_counts?: { rust: number; vitest: number; size_limit: number };
}

function mapApiToStatus(api: ApiStatusData): StatusData {
  return {
    sprint: api.sprint,
    title: `Sprint ${api.sprint} (${api.branch})`,
    head: api.head,
    phases: api.phases.map((p) => ({
      id: p.letter,
      label: `Phase ${p.letter}`,
      status:
        p.review_verdict === "PASS"
          ? "done"
          : p.has_review
            ? "active"
            : "pending",
      artifacts: {
        preflight: p.has_preflight,
        review: p.has_review,
        codex: p.has_codex,
      },
    })),
  };
}

const STATUS_VARIANT: Record<
  string,
  "default" | "secondary" | "destructive" | "outline"
> = {
  done: "default",
  active: "secondary",
  pending: "outline",
  error: "destructive",
};

function SkeletonCard() {
  return (
    <Card>
      <CardContent className="p-4">
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <div className="h-4 w-20 animate-pulse rounded bg-muted" />
            <div className="h-5 w-16 animate-pulse rounded-full bg-muted" />
          </div>
          <div className="flex gap-3">
            <div className="h-3 w-14 animate-pulse rounded bg-muted" />
            <div className="h-3 w-14 animate-pulse rounded bg-muted" />
            <div className="h-3 w-14 animate-pulse rounded bg-muted" />
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function SprintSkeleton() {
  return (
    <div className="space-y-6">
      <div className="space-y-2">
        <div className="h-7 w-40 animate-pulse rounded bg-muted" />
        <div className="h-4 w-64 animate-pulse rounded bg-muted" />
        <div className="h-3 w-32 animate-pulse rounded bg-muted" />
      </div>
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
        {Array.from({ length: 4 }, (_, i) => (
          <SkeletonCard key={i} />
        ))}
      </div>
      <Card>
        <CardContent className="p-4">
          <div className="flex gap-8">
            <div className="h-4 w-20 animate-pulse rounded bg-muted" />
            <div className="h-4 w-20 animate-pulse rounded bg-muted" />
            <div className="h-4 w-20 animate-pulse rounded bg-muted" />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function ArtifactIndicator({
  present,
  label,
}: {
  present: boolean;
  label: string;
}) {
  const { t } = useTranslation();

  return (
    <Tooltip>
      <TooltipTrigger
        className="inline-flex items-center gap-1 text-xs"
        aria-label={`${label}: ${present ? t("sprint.artifactPresent") : t("sprint.artifactMissing")}`}
      >
        {present ? (
          <CheckCircle2 className="size-3 text-[var(--green)]" />
        ) : (
          <XCircle className="size-3 text-muted-foreground" />
        )}
        <span
          className={
            present ? "text-[var(--green)]" : "text-muted-foreground"
          }
        >
          {label}
        </span>
      </TooltipTrigger>
      <TooltipContent>
        {present ? t("sprint.artifactPresent") : t("sprint.artifactMissing")}
      </TooltipContent>
    </Tooltip>
  );
}

function TestStat({ value, label }: { value: number; label: string }) {
  return (
    <div className="flex flex-col items-center gap-1">
      <span className="font-mono text-lg font-semibold tabular-nums">
        {value.toLocaleString()}
      </span>
      <span className="text-xs text-muted-foreground">{label}</span>
    </div>
  );
}

function PhaseCard({ phase }: { phase: PhaseData }) {
  const { t } = useTranslation();

  return (
    <Card className="transition-shadow duration-200 hover:ring-2 hover:ring-primary/20">
      <CardContent className="p-4">
        <div className="mb-3 flex items-center justify-between gap-2">
          <span className="truncate text-sm font-medium">{phase.label}</span>
          <Badge variant={STATUS_VARIANT[phase.status] ?? "outline"}>
            {t(`status.${phase.status}`)}
          </Badge>
        </div>
        <Separator className="mb-3" />
        <div className="flex gap-3">
          <ArtifactIndicator
            present={phase.artifacts.preflight}
            label={t("sprint.artifactPreflight")}
          />
          <ArtifactIndicator
            present={phase.artifacts.review}
            label={t("sprint.artifactReview")}
          />
          <ArtifactIndicator
            present={phase.artifacts.codex}
            label={t("sprint.artifactCodex")}
          />
        </div>
      </CardContent>
    </Card>
  );
}

function EmptyPhases() {
  const { t } = useTranslation();

  return (
    <Card>
      <CardContent className="flex flex-col items-center justify-center gap-3 py-12">
        <Layers className="size-10 text-muted-foreground/50" />
        <p className="text-sm text-muted-foreground">
          {t("sprint.noPhases")}
        </p>
      </CardContent>
    </Card>
  );
}

function ErrorState() {
  const { t } = useTranslation();

  return (
    <Card className="border-destructive/30">
      <CardContent className="flex flex-col items-center justify-center gap-3 py-12">
        <ServerCrash className="size-10 text-destructive/60" />
        <div className="text-center">
          <p className="text-sm font-medium text-destructive">
            {t("sprint.serverError")}
          </p>
          <p className="mt-1 text-xs text-muted-foreground">
            {t("sprint.serverHint")}
          </p>
        </div>
      </CardContent>
    </Card>
  );
}

export function SprintOverview() {
  const { t } = useTranslation();
  const { data: raw, error, loading } = useApi<ApiStatusData>("/status");

  if (loading) return <SprintSkeleton />;
  if (error || !raw) return <ErrorState />;

  const data = mapApiToStatus(raw);

  const completedCount = data.phases.filter(
    (p) => p.status === "done",
  ).length;

  return (
    <TooltipProvider>
      <div className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-lg">
              {t("sprint.title", { number: data.sprint })}
              <Badge variant="secondary" className="text-xs font-normal">
                {t("sprint.phaseProgress", {
                  done: completedCount,
                  total: data.phases.length,
                })}
              </Badge>
            </CardTitle>
            <CardDescription>{data.title}</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <GitCommitHorizontal className="size-3.5" />
              <code className="rounded bg-muted px-1.5 py-0.5 font-mono">
                {data.head}
              </code>
            </div>
          </CardContent>
        </Card>

        <section aria-label={t("sprint.phasesLabel")}>
          <h2 className="mb-3 flex items-center gap-2 text-sm font-semibold">
            <Layers className="size-4 text-muted-foreground" />
            {t("sprint.phasesLabel")}
          </h2>
          {data.phases.length > 0 ? (
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
              {data.phases.map((phase) => (
                <PhaseCard key={phase.id} phase={phase} />
              ))}
            </div>
          ) : (
            <EmptyPhases />
          )}
        </section>

        {data.test_counts && (
          <section aria-label={t("sprint.tests")}>
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-sm">
                  <FlaskConical className="size-4 text-muted-foreground" />
                  {t("sprint.tests")}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex justify-around gap-6">
                  <TestStat value={data.test_counts.rust} label="Rust" />
                  <Separator orientation="vertical" className="h-10" />
                  <TestStat value={data.test_counts.vitest} label="Vitest" />
                  <Separator orientation="vertical" className="h-10" />
                  <TestStat
                    value={data.test_counts.size_limit}
                    label="size-limit"
                  />
                </div>
              </CardContent>
            </Card>
          </section>
        )}
      </div>
    </TooltipProvider>
  );
}
