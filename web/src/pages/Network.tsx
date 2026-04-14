// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/my-network` — glassmorphism live view of the local nexus-worker.
 *
 * Polls `GET /worker-state` on the active coordinator every 2 s.
 */

import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Cpu, Activity, HardDrive, Settings, Timer } from "lucide-react";

import { Progress } from "@/components/ui/progress";
import { Button } from "@/components/ui/button";
import { GpuConsentDialog } from "@/components/GpuConsentDialog";
import {
  type GpuSnapshot,
  type LastTask,
  type ProjectServed,
  type WorkerStateV1,
  getWorkerState,
} from "@/api/coordinator";
import { type ConsentLevel, getConsent } from "@/api/consent";
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
      <div className="space-y-6">
        <div>
          <h1 className="text-3xl font-extrabold tracking-tight">
            Mon réseau
          </h1>
          <p className="mt-1 text-sm text-white/50">
            État live du worker nexus-grid qui tourne sur ta machine.
          </p>
        </div>
        <div className="glass-card max-w-md p-6">
          <h3 className="mb-2 font-bold">
            Aucun coordinateur sélectionné
          </h3>
          <p className="text-sm text-white/50">
            Ajoute un coordinateur depuis l'en-tete pour lire
            l'etat du worker via son endpoint{" "}
            <code className="font-mono text-white/60">/worker-state</code>.
          </p>
        </div>
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
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-3xl font-extrabold tracking-tight">
            Mon réseau
          </h1>
          <p className="mt-1 text-sm text-white/50">
            État live du worker nexus-grid — polling 2 s via{" "}
            <code className="font-mono text-white/60">{url}/worker-state</code>.
          </p>
        </div>
        <ConsentBadge coordUrl={url} />
      </div>

      {query.isLoading && (
        <div className="glass-card p-6 text-sm text-white/50">
          Lecture du snapshot...
        </div>
      )}

      {query.isError && (
        <div className="glass-card border-red-500/20 p-6 text-sm text-red-300">
          Erreur fetch /worker-state : {query.error.message}
        </div>
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
    <div className="glass-card p-6">
      <h3 className="mb-2 font-bold">Worker non détecté</h3>
      <p className="mb-3 text-sm text-white/50">
        Lance le worker dans un terminal pour que le shell puisse
        lire son etat. Le worker ecrit un snapshot JSON toutes
        les 5 s dans{" "}
        <code className="font-mono text-white/60">
          ~/.nexus-grid/worker/state.json
        </code>
        .
      </p>
      <pre className="rounded-lg bg-white/[0.04] p-3 font-mono text-[11px] text-white/60">
        cargo run -p nexus-worker -- start
      </pre>
      {error && (
        <p className="mt-2 text-[11px] text-white/40">
          Dernière erreur protocole : {error}
        </p>
      )}
    </div>
  );
}

function StaleBanner() {
  return (
    <div className="glass-card flex items-center gap-3 border-amber-500/20 p-4">
      <Timer className="h-4 w-4 shrink-0 text-amber-400" />
      <div>
        <p className="text-sm font-medium text-amber-400">
          Snapshot obsolète
        </p>
        <p className="text-xs text-white/50">
          Le worker n'a pas rafraichi son etat depuis plus de
          15 s. Les valeurs ci-dessous peuvent etre figees —
          verifie que le process tourne encore.
        </p>
      </div>
    </div>
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
    <div className="glass-card p-5">
      <div className="mb-4 flex items-center gap-2 text-base font-bold">
        <Activity className="h-4 w-4 text-white/40" />
        Identité du worker
      </div>
      <dl className="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-2 text-xs">
        <dt className="text-white/40">Node id</dt>
        <dd className="truncate font-mono text-white/70">
          {formatHash(state.node_id, 20)}
        </dd>
        <dt className="text-white/40">Version</dt>
        <dd className="text-white/70">{state.worker_version}</dd>
        <dt className="text-white/40">Uptime</dt>
        <dd className="text-white/70">{formatUptime(state.uptime_secs)}</dd>
        <dt className="text-white/40">Démarré</dt>
        <dd className="text-white/40">
          {formatRelativeTime(state.started_at)}
        </dd>
        <dt className="text-white/40">Dernier flush</dt>
        <dd className="text-white/40">
          {formatRelativeTime(state.last_updated_at)}
        </dd>
      </dl>
    </div>
  );
}

function GpuCard({ gpu }: { gpu: GpuSnapshot | null }) {
  if (!gpu) {
    return (
      <div className="glass-card p-5">
        <div className="mb-2 flex items-center gap-2 text-base font-bold">
          <Cpu className="h-4 w-4 text-white/40" />
          GPU
        </div>
        <p className="text-sm text-white/40">
          Aucun GPU détecté (mode CPU only).
        </p>
      </div>
    );
  }
  return (
    <div className="glass-card p-5">
      <div className="mb-1 flex items-center gap-2 text-base font-bold">
        <Cpu className="h-4 w-4 text-white/40" />
        GPU
      </div>
      <p className="mb-3 truncate text-sm text-white/50">{gpu.name}</p>
      <div className="space-y-3">
        <div>
          <div className="mb-1 flex items-center justify-between text-[11px]">
            <span className="text-white/40">VRAM</span>
            <span className="font-mono text-white/60">
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
            <span className="text-white/40">Utilisation</span>
            <span className="font-mono text-white/60">{gpu.utilization_pct}%</span>
          </div>
          <Progress value={gpu.utilization_pct} />
        </div>
        <div className="grid grid-cols-2 gap-3 text-[11px]">
          <div>
            <span className="text-white/40">Température</span>
            <p className="font-mono text-white/70">{gpu.temperature_c}°C</p>
          </div>
          <div>
            <span className="text-white/40">Puissance</span>
            <p className="font-mono text-white/70">{gpu.power_draw_w.toFixed(0)} W</p>
          </div>
        </div>
      </div>
    </div>
  );
}

function ProjectsServedCard({ projects }: { projects: ProjectServed[] }) {
  return (
    <div className="glass-card p-5">
      <div className="mb-1 flex items-center gap-2 text-base font-bold">
        <HardDrive className="h-4 w-4 text-white/40" />
        Projets enrôlés
      </div>
      <p className="mb-3 text-sm text-white/40">
        {projects.length} projet(s) déclarés par la allowlist du worker.
      </p>
      {projects.length === 0 ? (
        <p className="text-xs text-white/40">
          Aucun projet enrôlé. Utilise{" "}
          <code className="font-mono text-white/50">nexus-worker join &lt;invite&gt;</code>{" "}
          pour en ajouter.
        </p>
      ) : (
        <ul className="space-y-2">
          {projects.map((p) => (
            <li
              key={p.doc_id || p.project_name}
              className="flex items-start justify-between gap-2 rounded-lg bg-white/[0.04] p-2.5 border border-white/[0.06]"
            >
              <div className="min-w-0">
                <p className="truncate text-xs font-medium">
                  {p.project_name}
                </p>
                <p className="truncate font-mono text-[10px] text-white/40">
                  {formatHash(p.doc_id, 16)}
                </p>
              </div>
              <div className="flex items-center gap-2 text-[10px]">
                <span className="glass-pill py-0.5 text-[10px] text-white/60">
                  {p.tasks_completed} tâches
                </span>
                <span className="glass-pill py-0.5 text-[10px] text-white/60">
                  {p.kudos_total} kudos
                </span>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

function LastTaskCard({ task }: { task: LastTask | null }) {
  return (
    <div className="glass-card p-5">
      <div className="mb-1 flex items-center gap-2 text-base font-bold">
        <Timer className="h-4 w-4 text-white/40" />
        Dernière tâche
      </div>
      <p className="mb-3 text-sm text-white/40">
        {task
          ? `Projet ${task.project_name}`
          : "Aucune tâche traitée depuis le boot."}
      </p>
      {task && (
        <dl className="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-2 text-xs">
          <dt className="text-white/40">Task id</dt>
          <dd className="truncate font-mono text-white/70">
            {formatHash(task.task_id, 16)}
          </dd>
          <dt className="text-white/40">État</dt>
          <dd>
            <span className="glass-pill py-0.5 text-[10px] text-white/60">
              {task.status}
            </span>
          </dd>
          <dt className="text-white/40">Terminée</dt>
          <dd className="text-white/40">
            {formatRelativeTime(task.completed_at)}
          </dd>
          <dt className="text-white/40">Prompt</dt>
          <dd className="italic text-white/40">
            {task.prompt_preview}
          </dd>
        </dl>
      )}
    </div>
  );
}

// ================================================================
// Sprint 16 Phase C — consent badge + dialog wiring
// ================================================================
//
// The badge sits in the page header, shows the current sharing
// level (1..4) with a colour cue, and opens the
// `GpuConsentDialog` on click. The dialog also auto-opens once
// per browser profile (gated by a localStorage flag) so the
// first-boot UX matches the plan's "user must explicitly opt in
// before the worker shares anything" requirement.

const CONSENT_SEEN_KEY = "sbfb-consent-seen-v1";

const LEVEL_LABELS: Record<ConsentLevel, string> = {
  1: "L1 — Mes projets",
  2: "L2 — Open source",
  3: "L3 — Whitelist",
  4: "L4 — Tous publics",
};

const LEVEL_TONES: Record<ConsentLevel, string> = {
  1: "bg-white/[0.06] text-white/70",
  2: "bg-emerald-500/15 text-emerald-300",
  3: "bg-pink-500/15 text-pink-300",
  4: "bg-amber-500/15 text-amber-300",
};

function ConsentBadge({ coordUrl }: { coordUrl: string }) {
  const consentQuery = useQuery({
    queryKey: ["consent", coordUrl],
    queryFn: () => getConsent(coordUrl),
    staleTime: 30_000,
    retry: 0,
  });

  // Auto-open on the first visit ever (gated by the localStorage
  // flag). Computed once via the lazy initializer so the dialog
  // mounts open without a cascading effect re-render. The flag is
  // set immediately so a refresh during the same session doesn't
  // re-open the dialog even if the user closed it without saving.
  const [open, setOpen] = useState<boolean>(() => {
    if (typeof window === "undefined") return false;
    if (window.localStorage.getItem(CONSENT_SEEN_KEY) === "1") return false;
    window.localStorage.setItem(CONSENT_SEEN_KEY, "1");
    return true;
  });

  if (!consentQuery.data) {
    return (
      <div
        className="text-xs text-white/30"
        data-testid="consent-badge-loading"
      >
        Chargement consent…
      </div>
    );
  }

  const level = consentQuery.data.level;
  const whitelist = consentQuery.data.allowed_project_ids;

  return (
    <div className="flex flex-col items-end gap-2">
      <div className="flex items-center gap-2">
        <span
          className={`rounded-full px-2.5 py-1 text-[11px] font-medium ${LEVEL_TONES[level]}`}
          data-testid="consent-level-badge"
        >
          {LEVEL_LABELS[level]}
        </span>
        <Button
          variant="outline"
          size="sm"
          onClick={() => setOpen(true)}
          data-testid="consent-edit"
        >
          <Settings className="h-3 w-3" />
          Modifier
        </Button>
      </div>
      {level === 3 && whitelist.length > 0 && (
        <div className="text-right text-[10px] text-white/40">
          Whitelist : {whitelist.length} projet
          {whitelist.length > 1 ? "s" : ""}
        </div>
      )}
      <GpuConsentDialog
        open={open}
        onOpenChange={setOpen}
        coordinatorUrl={coordUrl}
        initialConfig={consentQuery.data}
      />
    </div>
  );
}

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = Network;
