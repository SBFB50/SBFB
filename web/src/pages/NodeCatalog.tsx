// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/node/:nodeId` — Sprint 75 Phase F — le catalogue d'un nœud-catalogue
 * (modèle F-Droid : la liste d'apps d'un dépôt).
 *
 * Verrou 4 (exigence PO) : le nœud est une SOURCE DE DÉCOUVERTE, jamais une
 * autorité. Chaque carte ouvre la preuve de provenance via le composant
 * `VerificationDetail` (fetch par projectId — la signature AUTEUR est la
 * seule autorité) ; une version non source-vérifiée (`is_open_source=false`
 * appris de l'annonce de son éditeur dans `/browse`) porte le marqueur
 * « Version dérivée ou modifiée », jamais le badge de l'original. Une carte
 * sans annonce d'éditeur croisable n'affiche AUCUN claim — la provenance se
 * prouve au clic, pas par le catalogue.
 *
 * Badge Q7 (PLAN-ADAPT preflight F) : « joignable via un seeder » est
 * CALCULÉ côté front depuis la paire de signaux existante — statut d'ancre
 * `unreachable` (sonde `/browse`) + `peer_count > 0` version-exacte
 * (`/seed-count?archive_hash=`). Aucun variant wire : Phase D (`0010450`)
 * a gardé `/browse` byte-identique, le test Rust
 * `reachable_via_seeder_status` pinne la paire.
 */

import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useNavigate, useParams } from "react-router-dom";
import {
  ArrowLeft,
  FileCheck,
  GitBranch,
  HeartHandshake,
  Loader2,
  Play,
  Radio,
  Server,
  ShieldQuestion,
} from "lucide-react";

import {
  listBrowse,
  listNodes,
  seedCount,
  seedVoluntary,
  type BrowseEntry,
  type CatalogApp,
  type NodeSummary,
} from "@/api/daemon";
import { VerificationDetail } from "@/components/VerificationDetail";
import { DaemonOfflineBanner } from "@/pages/Browse";
import {
  selectActiveCoordinator,
  useProjectStore,
} from "@/stores/projectStore";

export default function NodeCatalog() {
  const { nodeId } = useParams<{ nodeId: string }>();
  const active = useProjectStore(selectActiveCoordinator);

  if (!active || !nodeId) {
    return (
      <div className="space-y-6">
        <div className="glass-card max-w-md p-6">
          <h3 className="mb-2 font-bold">Aucun noeud actif</h3>
          <p className="text-sm text-white/50">
            Connecte-toi a un noeud depuis l&apos;en-tete.
          </p>
        </div>
      </div>
    );
  }

  return <CatalogContent coordUrl={active.url} nodeId={nodeId} />;
}

function CatalogContent({
  coordUrl,
  nodeId,
}: {
  coordUrl: string;
  nodeId: string;
}) {
  const nodesQuery = useQuery({
    queryKey: ["daemon-nodes", coordUrl],
    queryFn: () => listNodes(coordUrl),
    staleTime: 30_000,
    refetchOnWindowFocus: false,
  });

  // Croisement /browse : (a) le statut de sonde de CETTE ancre (l'entrée
  // nodedirectory porte `curator_pubkey = node_id` de l'ancre) pour le badge
  // Q7 ; (b) l'annonce de l'ÉDITEUR (source direct/curator) pour le flag
  // `is_open_source` — la 3e boucle aggregate met `is_open_source: false`
  // par DÉFAUT sur les entrées nodedirectory (le catalogue ne porte pas le
  // flag), donc ce canal-là ne prouve rien sur la version.
  const browseQuery = useQuery({
    queryKey: ["daemon-browse", coordUrl],
    queryFn: () => listBrowse(coordUrl),
    staleTime: 30_000,
    refetchOnWindowFocus: false,
  });

  const browseEntries = useMemo(
    () =>
      browseQuery.data?.kind === "data" ? browseQuery.data.body.entries : [],
    [browseQuery.data],
  );

  if (nodesQuery.isLoading) {
    return (
      <div className="glass-card p-6 text-sm text-white/50">Chargement...</div>
    );
  }
  if (nodesQuery.isError) {
    return (
      <div className="glass-card p-6">
        <h3 className="mb-1 font-bold text-red-300">Erreur reseau</h3>
        <p className="text-sm text-white/50">
          {nodesQuery.error instanceof Error
            ? nodesQuery.error.message
            : "erreur inconnue"}
        </p>
      </div>
    );
  }
  const result = nodesQuery.data;
  if (result?.kind === "unavailable") {
    return <DaemonOfflineBanner reason={result.reason} />;
  }
  if (result?.kind === "error") {
    return (
      <div className="glass-card p-6">
        <h3 className="mb-1 font-bold text-red-300">Noeud refuse</h3>
        <p className="text-sm text-white/50">{result.reason}</p>
      </div>
    );
  }

  const node: NodeSummary | undefined =
    result?.kind === "data"
      ? result.body.nodes.find((n) => n.node_id === nodeId)
      : undefined;

  if (!node) {
    return (
      <div className="space-y-4" data-testid="node-not-found">
        <BackLink />
        <div className="glass-card max-w-md p-6">
          <h3 className="mb-2 font-bold">Catalogue introuvable</h3>
          <p className="text-sm text-white/50">
            Aucun annuaire connu pour{" "}
            <code className="font-mono text-white/70">{truncateHex(nodeId)}</code>.
            Si tu viens de t&apos;y abonner, son catalogue apparaitra a sa
            premiere annonce.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <BackLink />

      {/* En-tête : la source de découverte, étiquetée comme telle — jamais
          stylée en autorité (verrou 4c). */}
      <div className="glass-card flex items-center gap-4 p-6" data-testid="node-catalog-header">
        <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-purple-500/10">
          <Server className="h-6 w-6 text-purple-300" />
        </div>
        <div>
          <h1 className="font-mono text-xl font-extrabold tracking-tight">
            {truncateHex(node.node_id)}
          </h1>
          <p className="mt-0.5 text-sm text-white/50" data-testid="node-source-label">
            Catalogue de ce noeud — source de decouverte, pas une autorite.
            La preuve d&apos;une app est la signature de son auteur.
          </p>
          <div className="mt-2 flex items-center gap-2">
            <span className="glass-pill py-0.5 text-[10px] text-white/60">
              {node.app_count} app{node.app_count > 1 ? "s" : ""}
            </span>
            <span className="glass-pill py-0.5 text-[10px] text-white/60">
              rev. {node.revision}
            </span>
          </div>
        </div>
      </div>

      {node.catalog.length === 0 ? (
        <div className="glass-card p-6" data-testid="node-catalog-empty">
          <h3 className="mb-1 font-bold">Catalogue vide</h3>
          <p className="text-sm text-white/50">
            Ce noeud n&apos;annonce aucune app pour l&apos;instant.
          </p>
        </div>
      ) : (
        <div
          className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3"
          data-testid="node-catalog-grid"
        >
          {dedupeCatalog(node.catalog).map((app) => (
            <CatalogCard
              key={`${app.project_id}-${app.archive_hash}`}
              app={app}
              anchorId={node.node_id}
              browseEntries={browseEntries}
              coordUrl={coordUrl}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function BackLink() {
  return (
    <Link
      to="/nodes"
      className="inline-flex items-center gap-1.5 rounded-full bg-white/[0.06] px-3 py-1.5 text-xs text-white/70 transition-colors hover:bg-white/10 hover:text-white"
      data-testid="back-to-nodes"
    >
      <ArrowLeft className="h-3.5 w-3.5" />
      Noeuds
    </Link>
  );
}

// ================================================================
// Carte catalogue — provenance auteur (verrou 4) + badge Q7
// ================================================================

function CatalogCard({
  app,
  anchorId,
  browseEntries,
  coordUrl,
}: {
  app: CatalogApp;
  anchorId: string;
  browseEntries: BrowseEntry[];
  coordUrl: string;
}) {
  const navigate = useNavigate();
  const [verifyOpen, setVerifyOpen] = useState(false);

  // (b) L'annonce de l'ÉDITEUR de CETTE version exacte (pid + hash), seule
  // source fiable du flag is_open_source. SEULES les entrées `direct`
  // (ProjectAnnouncement de l'éditeur) portent le vrai flag : l'aggregateur
  // hardcode is_open_source:false sur les boucles curator ET nodedirectory
  // (browse.rs — ni CuratorProjectRef ni CatalogApp ne transportent le flag),
  // donc tout match non-direct fabriquerait un faux « Version dérivée » sur
  // une app légitime (piège latent, review F).
  const publisherEntry = browseEntries.find(
    (e) =>
      (e.source ?? "curator") === "direct" &&
      e.project_id === app.project_id &&
      !!app.archive_hash &&
      e.archive_hash === app.archive_hash,
  );

  // (a) Le statut de sonde de CETTE ancre pour cette app (l'entrée
  // nodedirectory du même catalogue) — alimente le badge Q7.
  const anchorEntry = browseEntries.find(
    (e) =>
      (e.source ?? "curator") === "nodedirectory" &&
      e.curator_pubkey === anchorId &&
      e.project_id === app.project_id,
  );

  const verified = publisherEntry?.is_open_source === true;
  const derived = publisherEntry !== undefined && publisherEntry.is_open_source === false;

  return (
    <div className="glass-card space-y-3 p-4" data-testid="catalog-card">
      <div className="flex items-start justify-between gap-2">
        <h3 className="font-bold leading-tight">{app.project_name}</h3>
        {verified && (
          <span
            className="shrink-0 rounded-full bg-emerald-500/15 px-2 py-0.5 text-[10px] font-medium text-emerald-400"
            data-testid="catalog-verified-badge"
          >
            Source vérifiable
          </span>
        )}
        {derived && (
          <span
            className="inline-flex shrink-0 items-center gap-1 rounded-full bg-amber-500/15 px-2 py-0.5 text-[10px] font-medium text-amber-300"
            data-testid="catalog-derived-badge"
            title="Re-signée par son éditeur — ce n'est pas la version d'origine vérifiée"
          >
            <GitBranch className="h-2.5 w-2.5" />
            Version dérivée
          </span>
        )}
      </div>

      <p className="line-clamp-2 text-xs text-white/50">
        {app.description || "Application P2P"}
      </p>

      <div className="flex flex-wrap items-center gap-1.5">
        {app.category && (
          <span className="rounded-full bg-white/[0.06] px-2.5 py-0.5 text-[10px] font-medium text-white/60">
            {app.category}
          </span>
        )}
        {anchorEntry?.status === "unreachable" && app.archive_hash && (
          <SeederReachBadge
            coordUrl={coordUrl}
            projectId={app.project_id}
            archiveHash={app.archive_hash}
          />
        )}
      </div>

      <div className="flex items-center gap-2 pt-1">
        <button
          type="button"
          onClick={() =>
            navigate(`/browse/${encodeURIComponent(app.project_id)}`)
          }
          className="flex items-center gap-1.5 rounded-lg bg-white px-3 py-1.5 text-xs font-bold text-black transition-transform hover:scale-105"
          data-testid="catalog-open"
        >
          <Play className="h-3 w-3 fill-current" />
          Ouvrir
        </button>
        <button
          type="button"
          onClick={() => setVerifyOpen(true)}
          className="flex items-center gap-1.5 rounded-full bg-white/[0.06] px-3 py-1.5 text-[11px] text-white/60 transition-colors hover:bg-white/10 hover:text-white"
          data-testid="catalog-provenance"
          title="Voir la preuve de provenance signee par l'auteur"
        >
          {publisherEntry?.provenance_hash ? (
            <FileCheck className="h-3 w-3" />
          ) : (
            <ShieldQuestion className="h-3 w-3" />
          )}
          Provenance
        </button>
        <SupportButton
          coordUrl={coordUrl}
          projectId={app.project_id}
          archiveHash={app.archive_hash || null}
        />
      </div>

      <VerificationDetail
        open={verifyOpen}
        onOpenChange={setVerifyOpen}
        coordUrl={coordUrl}
        projectId={app.project_id}
        provenanceHash={publisherEntry?.provenance_hash ?? null}
        // Verrou 4 (résiduel review F) : le record provenance est keyé par
        // projectId et peut prouver une AUTRE version que cette row — le
        // dialog avertit si record.artifact_hash != le hash listé ici.
        expectedArtifactHash={app.archive_hash || null}
      />
    </div>
  );
}

/**
 * Badge Q7 « joignable via un seeder » — la paire de signaux honnête :
 * l'ancre est `unreachable` (sonde) MAIS un pair a annoncé détenir cette
 * version exacte récemment (`peer_count > 0`, best-effort TTL). Le badge ne
 * promet jamais une joignabilité dure : le content-addressing (BLAKE3) reste
 * la seule vérité au moment du pull.
 */
function SeederReachBadge({
  coordUrl,
  projectId,
  archiveHash,
}: {
  coordUrl: string;
  projectId: string;
  archiveHash: string;
}) {
  const countQuery = useQuery({
    queryKey: ["seed-count", coordUrl, projectId, archiveHash],
    queryFn: () => seedCount(coordUrl, projectId, archiveHash),
    staleTime: 30_000,
    refetchOnWindowFocus: false,
  });
  const peerCount =
    countQuery.data?.kind === "data" ? countQuery.data.body.peer_count : 0;

  if (peerCount <= 0) return null;
  return (
    <span
      className="inline-flex items-center gap-1 rounded-full bg-sky-500/15 px-2.5 py-0.5 text-[10px] font-medium text-sky-300"
      data-testid="seeder-reach-badge"
      title="Le noeud d'origine est hors ligne, mais un pair vu récemment garde cette version en ligne (best-effort)"
    >
      <Radio className="h-2.5 w-2.5" />
      Joignable via un pair
    </span>
  );
}

/**
 * CTA pull/seed : récupère + épingle la version EXACTE affichée
 * (`archive_hash` discriminateur, déféré review-D fermé en F). Volontaire,
 * sans approbation auteur (contenu public content-addressé) — le soutien ne
 * re-signe jamais rien, l'auteur reste l'auteur.
 */
function SupportButton({
  coordUrl,
  projectId,
  archiveHash,
}: {
  coordUrl: string;
  projectId: string;
  archiveHash: string | null;
}) {
  const queryClient = useQueryClient();
  const [supporting, setSupporting] = useState(false);
  const mutation = useMutation({
    mutationFn: () => seedVoluntary(coordUrl, projectId, archiveHash),
    onSuccess: (res) => {
      // Ne basculer que sur un état que le daemon a réellement confirmé
      // (miroir du CTA AvailabilitySheet : jamais d'état-mensonge d'échec).
      // L'état daemon a changé (row keep_online + self_seeding) : invalider
      // les mêmes caches que le miroir pour que /browse et la fiche
      // Disponibilité reflètent le seed sans attendre le staleTime.
      if (res.kind === "data") {
        setSupporting(true);
        void queryClient.invalidateQueries({
          queryKey: ["daemon-browse", coordUrl],
        });
        void queryClient.invalidateQueries({
          queryKey: ["seed-count", coordUrl, projectId],
        });
      }
    },
  });
  const failed =
    mutation.data?.kind === "error" || mutation.data?.kind === "unavailable";

  if (!archiveHash) return null;

  if (supporting) {
    return (
      <span
        className="inline-flex items-center gap-1.5 rounded-full bg-emerald-500/10 px-3 py-1.5 text-[11px] text-emerald-300"
        data-testid="catalog-support-active"
      >
        <HeartHandshake className="h-3 w-3" />
        Gardée en ligne
      </span>
    );
  }
  return (
    <button
      type="button"
      onClick={() => mutation.mutate()}
      disabled={mutation.isPending}
      className={`flex items-center gap-1.5 rounded-full px-3 py-1.5 text-[11px] transition-colors disabled:opacity-50 ${
        failed
          ? "bg-red-500/10 text-red-300 hover:bg-red-500/20"
          : "bg-white/[0.06] text-white/60 hover:bg-emerald-500/10 hover:text-emerald-300"
      }`}
      data-testid="catalog-support"
      title={
        failed
          ? "Impossible de récupérer cette app pour l'instant — réessayer"
          : "Récupérer cette version et la garder en ligne pour la soutenir"
      }
    >
      {mutation.isPending ? (
        <Loader2 className="h-3 w-3 animate-spin" />
      ) : (
        <HeartHandshake className="h-3 w-3" />
      )}
      Garder en ligne
    </button>
  );
}

/**
 * Un annuaire signé (donnée distante) peut lister deux fois la même paire
 * (project_id, archive_hash) — les caps d'ingest bornent la taille, pas
 * l'unicité. La dédup garde la première occurrence pour des clés React
 * stables (pas de collision de rendu sur un catalogue malveillant).
 */
function dedupeCatalog(catalog: CatalogApp[]): CatalogApp[] {
  const seen = new Set<string>();
  return catalog.filter((app) => {
    const key = `${app.project_id}-${app.archive_hash}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function truncateHex(hex: string): string {
  if (hex.length <= 16) return hex;
  return `${hex.slice(0, 8)}...${hex.slice(-8)}`;
}

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = NodeCatalog;
