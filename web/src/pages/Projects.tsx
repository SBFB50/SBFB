// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/my-projects` — list of the user's local coordinators.
 *
 * Phase A is an intentionally small stub that redirects to
 * onboarding when no coordinators are known, and otherwise
 * shows a one-card-per-entry list with no live data. Phase B
 * replaces this with the full rich view.
 */

import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { getHealth } from "@/api/coordinator";
import { useProjectStore } from "@/stores/projectStore";
import OnboardingEmpty from "@/pages/OnboardingEmpty";

export default function Projects() {
  const knownCoordinators = useProjectStore((s) => s.knownCoordinators);

  if (knownCoordinators.length === 0) {
    return <OnboardingEmpty />;
  }

  return (
    <div className="space-y-4">
      <div>
        <h1 className="text-2xl font-bold">Mes projets</h1>
        <p className="text-sm text-muted-foreground">
          Chaque carte correspond à un coordinateur actif que tu as ajouté.
          Phase A affiche les informations de base ; Phase B ajoute la vue
          détaillée par projet.
        </p>
      </div>

      <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
        {knownCoordinators.map((coord) => (
          <CoordinatorCard
            key={coord.url}
            url={coord.url}
            nickname={coord.nickname}
          />
        ))}
      </div>
    </div>
  );
}

function CoordinatorCard({
  url,
  nickname,
}: {
  url: string;
  nickname: string;
}) {
  const navigate = useNavigate();
  const healthQuery = useQuery({
    queryKey: ["health", url],
    queryFn: () => getHealth(url),
    refetchInterval: 5000,
    retry: 0,
  });

  const healthy = healthQuery.isSuccess && healthQuery.data.status === "ok";
  const label = nickname || healthQuery.data?.project || url;

  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between gap-2">
          <div className="min-w-0 flex-1">
            <CardTitle className="truncate">{label}</CardTitle>
            <CardDescription className="truncate font-mono text-[11px]">
              {url}
            </CardDescription>
          </div>
          <Badge
            variant={healthy ? "outline" : "secondary"}
            className={healthy ? "border-emerald-500/40 text-emerald-500" : ""}
          >
            {healthQuery.isLoading
              ? "…"
              : healthy
                ? "En ligne"
                : "Hors ligne"}
          </Badge>
        </div>
      </CardHeader>
      <CardContent>
        {healthQuery.data && (
          <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-[11px]">
            <dt className="text-muted-foreground">Projet</dt>
            <dd className="font-mono">{healthQuery.data.project}</dd>
            <dt className="text-muted-foreground">Node</dt>
            <dd className="font-mono">
              {healthQuery.data.node_id
                ? `${healthQuery.data.node_id.slice(0, 12)}…`
                : "—"}
            </dd>
          </dl>
        )}
        <div className="mt-4 flex justify-end">
          <Button
            size="sm"
            variant="outline"
            onClick={() => navigate(`/project/${encodeURIComponent(label)}`)}
          >
            Ouvrir
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = Projects;
