// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/project/:name` — glassmorphism project detail view.
 *
 * Five tabs: Overview, Tasks, Kudos, Invites, Apps.
 */

import { useMemo } from "react";
import { useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";

import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";
import {
  getHealth,
  getProject,
  listApps,
  listInvites,
  listKudos,
  listTasks,
  verifyKudos,
} from "@/api/coordinator";
import { useProjectStore } from "@/stores/projectStore";

import { OverviewTab } from "@/components/project/OverviewTab";
import { TasksTab } from "@/components/project/TasksTab";
import { KudosTab } from "@/components/project/KudosTab";
import { InvitesTab } from "@/components/project/InvitesTab";
import { AppsTab } from "@/components/project/AppsTab";

export default function ProjectDetail() {
  const { name } = useParams<{ name: string }>();
  const knownCoordinators = useProjectStore((s) => s.knownCoordinators);

  const coordinator = useMemo(() => {
    if (!name) return null;
    return (
      knownCoordinators.find((c) => c.nickname === name) ??
      knownCoordinators.find((c) => c.url === name) ??
      null
    );
  }, [name, knownCoordinators]);

  if (!coordinator) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-3xl font-extrabold tracking-tight">
            Projet introuvable
          </h1>
          <p className="mt-1 text-sm text-white/50">
            Aucun coordinateur connu ne porte le nom{" "}
            <code className="font-mono text-white/70">{name ?? "\u2014"}</code>.
          </p>
        </div>

        <div className="glass-card max-w-md p-6">
          <h3 className="mb-2 font-bold">Que faire ?</h3>
          <p className="text-sm text-white/50">
            Ajoute le coordinateur via le bouton « Ajouter un
            coordinateur » dans l'en-tete, ou verifie que son
            nickname correspond a ce que tu as entre.
          </p>
        </div>
      </div>
    );
  }

  return <ProjectDetailContent url={coordinator.url} />;
}

function ProjectDetailContent({ url }: { url: string }) {
  const projectQuery = useQuery({
    queryKey: ["project", url],
    queryFn: () => getProject(url),
    staleTime: 10_000,
  });
  const healthQuery = useQuery({
    queryKey: ["health", url],
    queryFn: () => getHealth(url),
    refetchInterval: 5000,
  });
  const tasksQuery = useQuery({
    queryKey: ["tasks", url],
    queryFn: () => listTasks(url, { limit: 100 }),
    refetchInterval: 3000,
  });
  const kudosQuery = useQuery({
    queryKey: ["kudos", url],
    queryFn: () => listKudos(url),
    refetchInterval: 5000,
  });
  const kudosVerifyQuery = useQuery({
    queryKey: ["kudos-verify", url],
    queryFn: () => verifyKudos(url),
    refetchInterval: 5000,
  });
  const invitesQuery = useQuery({
    queryKey: ["invites", url],
    queryFn: () => listInvites(url),
    staleTime: 3000,
  });
  const appsQuery = useQuery({
    queryKey: ["apps", url],
    queryFn: () => listApps(url),
    staleTime: 10_000,
  });

  const project = projectQuery.data;
  const health = healthQuery.data;

  return (
    <div className="space-y-6">
      {/* ---- Hero header ---- */}
      <div className="relative -mx-6 -mt-6 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-indigo-900/40 via-purple-950/30 to-transparent" />
        <div className="absolute inset-0 bg-gradient-to-t from-[#0a0a0f] via-transparent to-transparent" />
        <div className="relative px-8 pb-8 pt-12">
          <div className="flex flex-wrap items-end justify-between gap-4">
            <div className="min-w-0">
              <h1 className="truncate text-3xl font-extrabold tracking-tight">
                {project?.name ?? "Projet"}
              </h1>
              <p className="mt-1 truncate text-sm text-white/60">
                {project?.description ?? "..."}
              </p>
              <p className="mt-1 truncate font-mono text-[11px] text-white/40">
                {url}
              </p>
            </div>
            <div className="flex items-center gap-2">
              {project && (
                <span
                  className={`glass-pill text-[11px] font-medium ${
                    project.visibility === "public"
                      ? "text-emerald-400"
                      : "text-white/50"
                  }`}
                >
                  {project.visibility === "public" ? "Public" : "Prive"}
                </span>
              )}
              <span
                className={`glass-pill text-[11px] font-medium ${
                  health?.status === "ok"
                    ? "text-emerald-400"
                    : "text-red-400"
                }`}
              >
                {health ? health.status : "..."}
              </span>
            </div>
          </div>
        </div>
      </div>

      <Tabs defaultValue="overview">
        <TabsList>
          <TabsTrigger value="overview">Vue d'ensemble</TabsTrigger>
          <TabsTrigger value="tasks">Tâches</TabsTrigger>
          <TabsTrigger value="kudos">Kudos</TabsTrigger>
          <TabsTrigger value="invites">Invites</TabsTrigger>
          <TabsTrigger value="apps">Apps</TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="pt-4">
          <OverviewTab
            project={project}
            health={health}
            tasksCount={tasksQuery.data?.count ?? 0}
            kudosCount={kudosQuery.data?.count ?? 0}
            workerCount={
              new Set(
                (kudosQuery.data?.entries ?? []).map(
                  (e) => e.worker_node_id,
                ),
              ).size
            }
            appsCount={appsQuery.data?.count ?? 0}
          />
        </TabsContent>

        <TabsContent value="tasks" className="pt-4">
          <TasksTab
            query={tasksQuery}
          />
        </TabsContent>

        <TabsContent value="kudos" className="pt-4">
          <KudosTab
            query={kudosQuery}
            verifyQuery={kudosVerifyQuery}
          />
        </TabsContent>

        <TabsContent value="invites" className="pt-4">
          <InvitesTab url={url} query={invitesQuery} />
        </TabsContent>

        <TabsContent value="apps" className="pt-4">
          <AppsTab url={url} query={appsQuery} />
        </TabsContent>
      </Tabs>
    </div>
  );
}

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = ProjectDetail;
