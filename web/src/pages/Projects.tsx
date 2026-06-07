// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/my-projects` — glassmorphism list of the user's local coordinators.
 */

import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";

import { getHealth } from "@/api/coordinator";
import { useProjectStore } from "@/stores/projectStore";
import OnboardingEmpty from "@/pages/OnboardingEmpty";

export default function Projects() {
  const knownCoordinators = useProjectStore((s) => s.knownCoordinators);

  if (knownCoordinators.length === 0) {
    return <OnboardingEmpty />;
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-extrabold tracking-tight">
          Mes projets
        </h1>
        <p className="mt-1 text-sm text-white/50">
          Chaque carte correspond a un noeud actif que tu as
          ajoute.
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
    <div className="glass-card group overflow-hidden transition-all duration-300 hover:-translate-y-1 hover:shadow-[0_8px_40px_rgba(120,80,255,0.12)]">
      <div className="p-5">
        <div className="flex items-start justify-between gap-2">
          <div className="min-w-0 flex-1">
            <h3 className="truncate text-base font-bold">{label}</h3>
            <p className="truncate font-mono text-[11px] text-white/40">
              {url}
            </p>
          </div>
          <span
            className={`mt-1 inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-[10px] font-medium ${
              healthy
                ? "bg-emerald-500/15 text-emerald-400"
                : healthQuery.isLoading
                  ? "bg-white/[0.06] text-white/40"
                  : "bg-red-500/15 text-red-400"
            }`}
          >
            <span
              className={`h-1.5 w-1.5 rounded-full ${
                healthy
                  ? "bg-emerald-400 shadow-[0_0_4px_rgba(52,211,153,0.5)]"
                  : healthQuery.isLoading
                    ? "bg-white/30"
                    : "bg-red-400"
              }`}
            />
            {healthQuery.isLoading
              ? "..."
              : healthy
                ? "En ligne"
                : "Hors ligne"}
          </span>
        </div>

        {healthQuery.data && (
          <dl className="mt-4 grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-[11px]">
            <dt className="text-white/40">Projet</dt>
            <dd className="font-mono text-white/70">
              {healthQuery.data.project}
            </dd>
            <dt className="text-white/40">Node</dt>
            <dd className="font-mono text-white/70">
              {healthQuery.data.node_id
                ? `${healthQuery.data.node_id.slice(0, 12)}...`
                : "\u2014"}
            </dd>
          </dl>
        )}

        <div className="mt-4 flex justify-end">
          <button
            onClick={() => navigate(`/project/${encodeURIComponent(label)}`)}
            className="rounded-lg bg-white/[0.06] px-4 py-1.5 text-xs font-medium text-white/70 transition-colors hover:bg-white/10 hover:text-white"
          >
            Ouvrir
          </button>
        </div>
      </div>
    </div>
  );
}

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = Projects;
