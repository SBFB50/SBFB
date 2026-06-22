// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/compute` — Sprint 77 Phase J — « Calcul en réseau » : le panneau d'un
 * groupe de calcul privé (sharding LLM cross-machine, le modèle réparti sur
 * plusieurs machines qui ne le tiendraient pas seules).
 *
 * Read-only sur le statut (control-plane) : il interroge
 * `GET /api/daemon/shard-session/{id}` et n'expose qu'un AGRÉGAT (nombre de
 * membres), jamais les identités du groupe privé (THREAT_MODEL §16 SI-3/SI-4).
 * En Phase J le daemon n'a pas encore de session vivante (le plan de données
 * `sbfb/shard/1` n'est pas câblé à un registre lisible — Phase K), donc l'état
 * par défaut est « Aucune session active » : c'est honnête, pas un bug.
 *
 * UX (PO-9) : intentions utilisateur (« Lancer un gros modèle en réseau » /
 * « Rejoindre un groupe de calcul »), zéro jargon shard/ALPN/ComputeGroup en
 * surface. Les `data-testid` restent en anglais (identifiants techniques).
 */

import { useState } from "react";
import { useQuery, type UseQueryResult } from "@tanstack/react-query";
import { Boxes, Network, Search, Users } from "lucide-react";

import {
  getShardSession,
  type DaemonResult,
  type ShardSessionStatusResponse,
} from "@/api/daemon";
import { DaemonOfflineBanner } from "@/pages/Browse";
import {
  selectActiveCoordinator,
  useProjectStore,
} from "@/stores/projectStore";

export default function ShardSessionPanel() {
  const active = useProjectStore(selectActiveCoordinator);

  if (!active) {
    return (
      <div className="space-y-6">
        <PageHeader />
        <div className="glass-card max-w-md p-6">
          <h3 className="mb-2 font-bold">Aucun noeud actif</h3>
          <p className="text-sm text-white/50">
            Connecte-toi à un noeud depuis l&apos;en-tête pour rejoindre ou
            lancer un calcul en réseau.
          </p>
        </div>
      </div>
    );
  }

  return <ComputeContent coordUrl={active.url} />;
}

function PageHeader() {
  return (
    <div>
      <h1 className="text-3xl font-extrabold tracking-tight">
        Calcul en réseau
      </h1>
      <p className="mt-1 max-w-2xl text-sm text-white/50">
        Mets en commun la puissance de plusieurs machines pour faire tourner un
        gros modèle qui ne tiendrait sur aucune d&apos;elles seule. Le groupe
        est privé : seules les machines invitées y participent, et il n&apos;y a
        pas de serveur central.
      </p>
    </div>
  );
}

type Mode = "idle" | "join" | "launch";

function ComputeContent({ coordUrl }: { coordUrl: string }) {
  const [mode, setMode] = useState<Mode>("idle");
  const [groupIdInput, setGroupIdInput] = useState("");
  // The session id actually looked up — set only when the user submits a join
  // code. `null` keeps the status query disabled and the empty state showing.
  const [sessionId, setSessionId] = useState<string | null>(null);

  const sessionQuery = useQuery({
    queryKey: ["shard-session", coordUrl, sessionId],
    queryFn: () => getShardSession(coordUrl, sessionId as string),
    enabled: sessionId !== null && sessionId.length > 0,
    staleTime: 5_000,
    refetchOnWindowFocus: false,
  });

  const looking = sessionId !== null && sessionId.length > 0;

  return (
    <div className="space-y-6" data-testid="shard-session-panel">
      <PageHeader />

      <div className="flex flex-wrap gap-3">
        <button
          type="button"
          onClick={() => {
            setMode("launch");
            setSessionId(null);
          }}
          className="flex items-center gap-2 rounded-lg bg-purple-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-purple-500"
          data-testid="cta-launch-large-model"
        >
          <Network className="h-4 w-4" />
          Lancer un gros modèle en réseau
        </button>
        <button
          type="button"
          onClick={() => setMode("join")}
          className="glass-pill flex items-center gap-2 text-sm"
          data-testid="cta-join-compute-group"
        >
          <Boxes className="h-4 w-4" />
          Rejoindre un groupe de calcul
        </button>
      </div>

      {mode === "join" && (
        <JoinForm
          value={groupIdInput}
          onChange={setGroupIdInput}
          onSubmit={() => setSessionId(groupIdInput.trim())}
        />
      )}

      {mode === "launch" && <LaunchIntent />}

      {looking ? <SessionStatus query={sessionQuery} /> : <EmptyState />}
    </div>
  );
}

/**
 * « Rejoindre un groupe de calcul » : on rejoint avec l'identifiant que
 * l'initiateur a partagé hors-bande. La soumission interroge le statut
 * (read-only) ; le plan de données réel arrive en Phase K.
 */
function JoinForm({
  value,
  onChange,
  onSubmit,
}: {
  value: string;
  onChange: (v: string) => void;
  onSubmit: () => void;
}) {
  return (
    <div className="glass-card max-w-xl space-y-3 p-5" data-testid="shard-join-form">
      <label htmlFor="shard-group-id" className="block text-sm text-white/70">
        Identifiant du groupe
      </label>
      <div className="flex gap-2">
        <input
          id="shard-group-id"
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder="Identifiant transmis par l'initiateur"
          className="flex-1 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-white/90 outline-none placeholder:text-white/30 focus:border-purple-500/60"
          data-testid="shard-group-id-input"
        />
        <button
          type="button"
          onClick={onSubmit}
          disabled={value.trim().length === 0}
          className="flex items-center gap-2 rounded-lg bg-purple-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-purple-500 disabled:cursor-not-allowed disabled:opacity-40"
          data-testid="shard-join-submit"
        >
          <Search className="h-4 w-4" />
          Voir le statut
        </button>
      </div>
    </div>
  );
}

/**
 * « Lancer un gros modèle en réseau » : décrit honnêtement le flux réel
 * (rassembler des machines qui partagent leur puissance, puis répartir le
 * modèle). La session démarrée apparaîtra dans le statut ci-dessous une fois le
 * plan de données câblé (Phase K) — pas de « bientôt » creux, juste l'étape
 * suivante.
 */
function LaunchIntent() {
  return (
    <div className="glass-card max-w-xl p-5" data-testid="shard-launch-intent">
      <h3 className="mb-2 font-bold">Lancer un modèle réparti</h3>
      <p className="text-sm text-white/60">
        Pour lancer un gros modèle, rassemble d&apos;abord des machines qui
        partagent leur puissance, puis répartis le modèle entre elles : chaque
        machine ne charge que sa part. La session apparaîtra ici une fois
        démarrée, avec le nombre de machines qui y participent.
      </p>
    </div>
  );
}

function EmptyState() {
  return (
    <div
      className="glass-card p-8 text-center"
      data-testid="shard-session-empty"
    >
      <Boxes className="mx-auto mb-4 h-10 w-10 text-white/20" />
      <h3 className="mb-2 font-bold">Aucune session active</h3>
      <p className="mx-auto max-w-md text-sm text-white/50">
        Rejoins un groupe de calcul avec l&apos;identifiant que son initiateur
        t&apos;a transmis, ou lance ton propre modèle en réseau.
      </p>
    </div>
  );
}

/**
 * Affiche le résultat du lookup de statut. Read-only : seul un AGRÉGAT (nombre
 * de membres) est exposé, jamais les identités du groupe privé.
 */
function SessionStatus({
  query,
}: {
  query: UseQueryResult<DaemonResult<ShardSessionStatusResponse>, Error>;
}) {
  if (query.isLoading) {
    return (
      <div className="glass-card p-6 text-sm text-white/50">Chargement...</div>
    );
  }

  if (query.isError) {
    return (
      <div className="glass-card p-6">
        <h3 className="mb-1 font-bold text-red-300">Erreur réseau</h3>
        <p className="text-sm text-white/50">
          {query.error instanceof Error ? query.error.message : "erreur inconnue"}
        </p>
      </div>
    );
  }

  const result = query.data;
  if (result?.kind === "unavailable") {
    return <DaemonOfflineBanner reason={result.reason} />;
  }
  if (result?.kind === "error") {
    return (
      <div className="glass-card p-6">
        <h3 className="mb-1 font-bold text-red-300">Noeud refusé</h3>
        <p className="text-sm text-white/50">{result.reason}</p>
      </div>
    );
  }
  if (result?.kind === "data") {
    const { found, session } = result.body;
    if (!found || !session) {
      return (
        <div className="glass-card p-6" data-testid="shard-session-not-found">
          <h3 className="mb-1 font-bold">Aucune session sous cet identifiant</h3>
          <p className="text-sm text-white/50">
            Vérifie l&apos;identifiant transmis par l&apos;initiateur, ou attends
            que la session démarre.
          </p>
        </div>
      );
    }
    return (
      <div className="glass-card space-y-3 p-6" data-testid="shard-session-status">
        <p className="font-mono text-sm font-bold text-white/80">
          {truncate(session.session_id)}
        </p>
        <div className="flex items-center gap-2 text-sm">
          <Users className="h-4 w-4 text-purple-300" />
          <span className="text-white/70">Machines participantes</span>
          <span className="font-bold" data-testid="shard-member-count">
            {session.member_count}
          </span>
        </div>
      </div>
    );
  }
  return null;
}

function truncate(id: string): string {
  if (id.length <= 18) return id;
  return `${id.slice(0, 10)}...${id.slice(-6)}`;
}

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = ShardSessionPanel;
