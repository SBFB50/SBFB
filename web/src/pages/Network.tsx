// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/my-network` — glassmorphism live view of the local nexus-worker.
 *
 * Polls `GET /api/v1/worker/state` on the active coordinator every 2 s
 * and surfaces the "offer my power" panel (Sprint 76 Phase A, D1): the
 * active sharing level + a live caps gauge fed by the snapshot's
 * `consent` field, plus the CTA that opens `GpuConsentDialog`.
 */

import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Cpu,
  Activity,
  HardDrive,
  Heart,
  Settings,
  Timer,
} from "lucide-react";

import { Progress } from "@/components/ui/progress";
import { Button } from "@/components/ui/button";
import { GpuConsentDialog } from "@/components/GpuConsentDialog";
import {
  type ConsentSnapshot,
  type GpuSnapshot,
  type LastTask,
  type ProjectServed,
  type WorkerStateV1,
  getWorkerState,
} from "@/api/coordinator";
import {
  CONSENT_LEVEL,
  type ConsentLevel,
  PUBLIC_SHARING_LEVELS,
  getConsent,
} from "@/api/consent";
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
            Aucun noeud actif
          </h3>
          <p className="text-sm text-white/50">
            Connecte-toi a un noeud depuis l'en-tete pour lire
            l'etat du worker qui tourne sur ta machine.
          </p>
        </div>
      </div>
    );
  }

  return <NetworkContent url={active.url} />;
}

const CONSENT_SEEN_KEY = "sbfb-consent-seen-v1";

function NetworkContent({ url }: { url: string }) {
  const query = useQuery({
    queryKey: ["worker-state", url],
    queryFn: () => getWorkerState(url),
    refetchInterval: 2000,
    retry: 0,
  });

  // The consent config (level + caps) the dialog edits. Distinct from
  // the live `consent` snapshot field, which carries today's usage.
  const consentQuery = useQuery({
    queryKey: ["consent", url],
    queryFn: () => getConsent(url),
    staleTime: 30_000,
    retry: 0,
  });

  // Auto-open the dialog on the very first visit (gated by a
  // localStorage flag) so the first-boot UX still nudges an explicit
  // opt-in. The flag is set immediately so a same-session refresh does
  // not re-open it.
  const [dialogOpen, setDialogOpen] = useState<boolean>(() => {
    if (typeof window === "undefined") return false;
    if (window.localStorage.getItem(CONSENT_SEEN_KEY) === "1") return false;
    window.localStorage.setItem(CONSENT_SEEN_KEY, "1");
    return true;
  });

  const liveConsent =
    query.data && query.data.running === true
      ? query.data.state.consent ?? undefined
      : undefined;
  const activeLevel = (liveConsent?.level ?? consentQuery.data?.level) as
    | ConsentLevel
    | undefined;

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-3xl font-extrabold tracking-tight">
            Mon réseau
          </h1>
          <p className="mt-1 text-sm text-white/50">
            État live du worker nexus-grid qui tourne sur ta machine —
            rafraîchi toutes les 2 s.
          </p>
        </div>
        <ConsentLevelPill level={activeLevel} onEdit={() => setDialogOpen(true)} />
      </div>

      <OfferPowerCard
        consent={liveConsent}
        fallbackLevel={consentQuery.data?.level as ConsentLevel | undefined}
        onOffer={() => setDialogOpen(true)}
      />

      {query.isLoading && (
        <div className="glass-card p-6 text-sm text-white/50">
          Lecture du snapshot...
        </div>
      )}

      {query.isError && (
        <div className="glass-card border-red-500/20 p-6 text-sm text-red-300">
          Erreur lecture de l'état du worker : {query.error.message}
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

      {consentQuery.data && (
        <GpuConsentDialog
          key={`${dialogOpen}-${consentQuery.data.level}`}
          open={dialogOpen}
          onOpenChange={setDialogOpen}
          coordinatorUrl={url}
          initialConfig={consentQuery.data}
          onSaved={() => {
            void consentQuery.refetch();
          }}
        />
      )}
    </div>
  );
}

// ================================================================
// Sprint 76 Phase A (D1) — "offer my power" panel
// ================================================================

const LEVEL_LABELS: Record<ConsentLevel, string> = {
  [CONSENT_LEVEL.OWN_PROJECTS]: "Mes projets uniquement",
  [CONSENT_LEVEL.OPEN_SOURCE]: "Apps depuis un depot public",
  [CONSENT_LEVEL.WHITELIST]: "Projets choisis (whitelist)",
  [CONSENT_LEVEL.ALL]: "Tous les projets publics",
};

const LEVEL_TONES: Record<ConsentLevel, string> = {
  [CONSENT_LEVEL.OWN_PROJECTS]: "bg-white/[0.06] text-white/70",
  [CONSENT_LEVEL.OPEN_SOURCE]: "bg-emerald-500/15 text-emerald-300",
  [CONSENT_LEVEL.WHITELIST]: "bg-pink-500/15 text-pink-300",
  [CONSENT_LEVEL.ALL]: "bg-amber-500/15 text-amber-300",
};

function ConsentLevelPill({
  level,
  onEdit,
}: {
  level?: ConsentLevel;
  onEdit: () => void;
}) {
  if (level === undefined) {
    return (
      <div className="text-xs text-white/30" data-testid="consent-pill-loading">
        Chargement du niveau…
      </div>
    );
  }
  return (
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
        onClick={onEdit}
        data-testid="consent-edit"
      >
        <Settings className="h-3 w-3" />
        Modifier
      </Button>
    </div>
  );
}

function OfferPowerCard({
  consent,
  fallbackLevel,
  onOffer,
}: {
  consent?: ConsentSnapshot;
  fallbackLevel?: ConsentLevel;
  onOffer: () => void;
}) {
  const level = (consent?.level ?? fallbackLevel) as ConsentLevel | undefined;
  // Only OpenSource / All actually enroll the co-located worker
  // at-large (D1). OwnProjects / Whitelist keep it least-privilege, so
  // the copy must not claim public contribution.
  const sharing = level !== undefined && PUBLIC_SHARING_LEVELS.includes(level);
  const hoursCap = consent?.max_hours_day ?? null;
  const hoursUsed = consent?.hours_used_today ?? 0;

  return (
    <div className="glass-card p-6" data-testid="offer-power-card">
      <div className="flex items-start justify-between gap-4">
        <div>
          <div className="flex items-center gap-2 text-base font-bold">
            <Heart className="h-4 w-4 text-pink-300" />
            Offrir ma puissance au réseau
          </div>
          <p className="mt-1 text-sm text-white/50">
            {sharing
              ? "Ta machine contribue au calcul du réseau public, dans la limite de tes caps."
              : "Ta puissance reste privée. Choisis ce que ton GPU partage avec le réseau."}
          </p>
        </div>
        <Button
          onClick={onOffer}
          data-testid="offer-power-cta"
          className="shrink-0"
        >
          <Heart className="h-3.5 w-3.5" />
          Offrir ma puissance au réseau
        </Button>
      </div>

      {level !== undefined && (
        <p className="mt-4 text-xs text-white/50" data-testid="offer-power-level">
          Niveau actif :{" "}
          <span className="font-medium text-white/80">
            {LEVEL_LABELS[level]}
          </span>
        </p>
      )}

      {consent && hoursCap !== null && (
        <div className="mt-3" data-testid="offer-power-hours-gauge">
          <div className="mb-1 flex items-center justify-between text-[11px]">
            <span className="text-white/40">Heures données aujourd'hui</span>
            <span className="font-mono text-white/60">
              {hoursUsed.toFixed(1)} h / {hoursCap.toFixed(0)} h
            </span>
          </div>
          <Progress
            value={Math.min(
              100,
              Math.round((hoursUsed / Math.max(0.01, hoursCap)) * 100),
            )}
          />
        </div>
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

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = Network;
