/**
 * `/my-network` — live view of the local nexus-worker.
 *
 * Polls `GET /worker-state` on the active coordinator every
 * 2 s (sprint5_plan.md §6.2). Rendering has three states:
 *
 * 1. No active coordinator — onboarding pointer.
 * 2. Coordinator reachable but worker not running — explanation
 *    card with the CLI command to start the worker.
 * 3. Coordinator reachable AND worker flushing a state snapshot
 *    — four cards: identity, GPU, projects served, last task.
 *
 * A `stale: true` payload (worker last flushed > 15 s ago)
 * keeps the four cards visible but paints a warning banner
 * above them so the user knows the numbers may be frozen.
 */

import { useQuery } from "@tanstack/react-query";
import { Cpu, Activity, HardDrive, Timer } from "lucide-react";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import {
  type GpuSnapshot,
  type LastTask,
  type ProjectServed,
  type WorkerStateV1,
  getWorkerState,
} from "@/api/coordinator";
import {
  selectActiveCoordinator,
  useProjectStore,
} from "@/stores/projectStore";
import {
  formatHash,
  formatMemoryMb,
  formatRelativeTime,
  formatUptime,
} from "@/lib/format";

export default function Network() {
  const active = useProjectStore(selectActiveCoordinator);

  if (!active) {
    return (
      <div className="space-y-4">
        <div>
          <h1 className="text-2xl font-bold">Mon réseau</h1>
          <p className="text-sm text-muted-foreground">
            État live du worker nexus-grid qui tourne sur ta machine.
          </p>
        </div>
        <Card>
          <CardHeader>
            <CardTitle>Aucun coordinateur sélectionné</CardTitle>
            <CardDescription>
              Ajoute un coordinateur depuis l'en-tête pour lire
              l'état du worker via son endpoint{" "}
              <code className="font-mono">/worker-state</code>.
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  return <NetworkContent url={active.url} />;
}

function NetworkContent({ url }: { url: string }) {
  const query = useQuery({
    queryKey: ["worker-state", url],
    queryFn: () => getWorkerState(url),
    refetchInterval: 2000,
    retry: 0,
  });

  return (
    <div className="space-y-4">
      <div>
        <h1 className="text-2xl font-bold">Mon réseau</h1>
        <p className="text-sm text-muted-foreground">
          État live du worker nexus-grid — polling 2 s via{" "}
          <code className="font-mono">{url}/worker-state</code>.
        </p>
      </div>

      {query.isLoading && (
        <Card>
          <CardContent className="p-6 text-sm text-muted-foreground">
            Lecture du snapshot…
          </CardContent>
        </Card>
      )}

      {query.isError && (
        <Card className="border-destructive/40">
          <CardContent className="p-6 text-sm text-destructive">
            Erreur fetch /worker-state : {query.error.message}
          </CardContent>
        </Card>
      )}

      {query.data && query.data.running === false && (
        <WorkerOfflineCard error={query.data.error} />
      )}

      {query.data && query.data.running === true && (
        <>
          {query.data.stale && <StaleBanner />}
          <WorkerCards state={query.data.state} />
        </>
      )}
    </div>
  );
}

function WorkerOfflineCard({ error }: { error?: string }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Worker non détecté</CardTitle>
        <CardDescription>
          Lance le worker dans un terminal pour que le shell puisse
          lire son état. Le worker écrit un snapshot JSON toutes
          les 5 s dans{" "}
          <code className="font-mono">
            ~/.nexus-grid/worker/state.json
          </code>
          .
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-2">
        <pre className="rounded-md border border-border bg-muted/30 p-3 text-[11px]">
          cargo run -p nexus-worker -- start
        </pre>
        {error && (
          <p className="text-[11px] text-muted-foreground">
            Dernière erreur protocole : {error}
          </p>
        )}
      </CardContent>
    </Card>
  );
}

function StaleBanner() {
  return (
    <Card className="border-yellow-500/40">
      <CardContent className="flex items-center gap-3 p-4">
        <Timer className="h-4 w-4 text-yellow-500" />
        <div>
          <p className="text-sm font-medium text-yellow-500">
            Snapshot obsolète
          </p>
          <p className="text-xs text-muted-foreground">
            Le worker n'a pas rafraîchi son état depuis plus de
            15 s. Les valeurs ci-dessous peuvent être figées —
            vérifie que le process tourne encore.
          </p>
        </div>
      </CardContent>
    </Card>
  );
}

function WorkerCards({ state }: { state: WorkerStateV1 }) {
  return (
    <div className="grid gap-4 lg:grid-cols-2">
      <IdentityCard state={state} />
      <GpuCard gpu={state.gpu} />
      <ProjectsServedCard projects={state.projects_served} />
      <LastTaskCard task={state.last_task} />
    </div>
  );
}

function IdentityCard({ state }: { state: WorkerStateV1 }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-base">
          <Activity className="h-4 w-4 text-muted-foreground" />
          Identité du worker
        </CardTitle>
      </CardHeader>
      <CardContent>
        <dl className="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-2 text-xs">
          <dt className="text-muted-foreground">Node id</dt>
          <dd className="truncate font-mono">
            {formatHash(state.node_id, 20)}
          </dd>
          <dt className="text-muted-foreground">Version</dt>
          <dd>{state.worker_version}</dd>
          <dt className="text-muted-foreground">Uptime</dt>
          <dd>{formatUptime(state.uptime_secs)}</dd>
          <dt className="text-muted-foreground">Démarré</dt>
          <dd className="text-muted-foreground">
            {formatRelativeTime(state.started_at)}
          </dd>
          <dt className="text-muted-foreground">Dernier flush</dt>
          <dd className="text-muted-foreground">
            {formatRelativeTime(state.last_updated_at)}
          </dd>
        </dl>
      </CardContent>
    </Card>
  );
}

function GpuCard({ gpu }: { gpu: GpuSnapshot | null }) {
  if (!gpu) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <Cpu className="h-4 w-4 text-muted-foreground" />
            GPU
          </CardTitle>
          <CardDescription>Aucun GPU détecté (mode CPU only).</CardDescription>
        </CardHeader>
      </Card>
    );
  }
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-base">
          <Cpu className="h-4 w-4 text-muted-foreground" />
          GPU
        </CardTitle>
        <CardDescription className="truncate">{gpu.name}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <div>
          <div className="mb-1 flex items-center justify-between text-[11px]">
            <span className="text-muted-foreground">VRAM</span>
            <span className="font-mono">
              {formatMemoryMb(gpu.memory_used_mb)} /{" "}
              {formatMemoryMb(gpu.memory_total_mb)}
            </span>
          </div>
          <Progress
            value={Math.round(
              (gpu.memory_used_mb / Math.max(1, gpu.memory_total_mb)) * 100,
            )}
          />
        </div>
        <div>
          <div className="mb-1 flex items-center justify-between text-[11px]">
            <span className="text-muted-foreground">Utilisation</span>
            <span className="font-mono">{gpu.utilization_pct}%</span>
          </div>
          <Progress value={gpu.utilization_pct} />
        </div>
        <div className="grid grid-cols-2 gap-3 text-[11px]">
          <div>
            <span className="text-muted-foreground">Température</span>
            <p className="font-mono">{gpu.temperature_c}°C</p>
          </div>
          <div>
            <span className="text-muted-foreground">Puissance</span>
            <p className="font-mono">{gpu.power_draw_w.toFixed(0)} W</p>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function ProjectsServedCard({ projects }: { projects: ProjectServed[] }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-base">
          <HardDrive className="h-4 w-4 text-muted-foreground" />
          Projets enrôlés
        </CardTitle>
        <CardDescription>
          {projects.length} projet(s) déclarés par la allowlist du worker.
        </CardDescription>
      </CardHeader>
      <CardContent>
        {projects.length === 0 ? (
          <p className="text-xs text-muted-foreground">
            Aucun projet enrôlé. Utilise{" "}
            <code className="font-mono">nexus-worker join &lt;invite&gt;</code>{" "}
            pour en ajouter.
          </p>
        ) : (
          <ul className="space-y-2">
            {projects.map((p) => (
              <li
                key={p.doc_id || p.project_name}
                className="flex items-start justify-between gap-2 rounded-md border border-border bg-muted/20 p-2"
              >
                <div className="min-w-0">
                  <p className="truncate text-xs font-medium">
                    {p.project_name}
                  </p>
                  <p className="truncate font-mono text-[10px] text-muted-foreground">
                    {formatHash(p.doc_id, 16)}
                  </p>
                </div>
                <div className="flex items-center gap-2 text-[10px]">
                  <Badge variant="outline" className="text-[10px]">
                    {p.tasks_completed} tâches
                  </Badge>
                  <Badge variant="outline" className="text-[10px]">
                    {p.kudos_total} kudos
                  </Badge>
                </div>
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}

function LastTaskCard({ task }: { task: LastTask | null }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-base">
          <Timer className="h-4 w-4 text-muted-foreground" />
          Dernière tâche
        </CardTitle>
        <CardDescription>
          {task
            ? `Projet ${task.project_name}`
            : "Aucune tâche traitée depuis le boot."}
        </CardDescription>
      </CardHeader>
      {task && (
        <CardContent>
          <dl className="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-2 text-xs">
            <dt className="text-muted-foreground">Task id</dt>
            <dd className="truncate font-mono">
              {formatHash(task.task_id, 16)}
            </dd>
            <dt className="text-muted-foreground">État</dt>
            <dd>
              <Badge variant="outline">{task.status}</Badge>
            </dd>
            <dt className="text-muted-foreground">Terminée</dt>
            <dd className="text-muted-foreground">
              {formatRelativeTime(task.completed_at)}
            </dd>
            <dt className="text-muted-foreground">Prompt</dt>
            <dd className="italic text-muted-foreground">
              {task.prompt_preview}
            </dd>
          </dl>
        </CardContent>
      )}
    </Card>
  );
}
