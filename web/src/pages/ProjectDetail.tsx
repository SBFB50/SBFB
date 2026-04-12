/**
 * `/project/:name` — Sprint 5 Phase B rich view.
 *
 * Five tabs per `.planning/sprint5_plan.md` §5.2:
 * - Overview:  health + project metadata + derived counters
 * - Tasks:     paginated task list via /tasks
 * - Kudos:     ledger table + integrity badge
 * - Invites:   list + create + revoke
 * - Apps:      manifest browser with sync/async tab descriptors
 *
 * The page resolves the coordinator URL from the project store
 * by name. If the `:name` route param does not match any known
 * coordinator's nickname / project_name, the page renders a
 * card offering to add one via the dialog.
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
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
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

  // Resolve the coordinator by nickname first (set by the
  // AddCoordinatorDialog from the health payload's project name),
  // falling back to an exact URL match.
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
      <div className="space-y-4">
        <div>
          <h1 className="text-2xl font-bold">Projet introuvable</h1>
          <p className="text-sm text-muted-foreground">
            Aucun coordinateur connu ne porte le nom{" "}
            <code className="font-mono">{name ?? "—"}</code>.
          </p>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>Que faire&nbsp;?</CardTitle>
            <CardDescription>
              Ajoute le coordinateur via le bouton « Ajouter un
              coordinateur » dans le header, ou vérifie que son
              nickname correspond à ce que tu as entré.
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  return <ProjectDetailContent url={coordinator.url} />;
}

function ProjectDetailContent({ url }: { url: string }) {
  // Preload every tab's data in parallel. React Query dedupes
  // these so child tabs that `useQuery` the same keys get the
  // already-fetched data immediately.
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
    <div className="space-y-4">
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div className="min-w-0">
          <h1 className="truncate text-2xl font-bold">
            {project?.name ?? "Projet"}
          </h1>
          <p className="truncate text-sm text-muted-foreground">
            {project?.description ?? "…"}
          </p>
          <p className="mt-1 truncate font-mono text-[11px] text-muted-foreground">
            {url}
          </p>
        </div>
        <div className="flex items-center gap-2">
          {project && (
            <Badge
              variant="outline"
              className={
                project.visibility === "public"
                  ? "border-emerald-500/40 text-emerald-500"
                  : "border-border text-muted-foreground"
              }
            >
              {project.visibility === "public" ? "Public" : "Privé"}
            </Badge>
          )}
          <Badge
            variant="outline"
            className={
              health?.status === "ok"
                ? "border-emerald-500/40 text-emerald-500"
                : "border-destructive/40 text-destructive"
            }
          >
            {health ? health.status : "…"}
          </Badge>
        </div>
      </header>

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
                  (e) => e.worker_pubkey_hex,
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
