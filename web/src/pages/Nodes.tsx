// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/nodes` — Sprint 75 Phase F — la lentille « par nœud » de la découverte
 * PULL (modèle F-Droid : l'écran « Dépôts »).
 *
 * Liste les nœuds-catalogues : ceux dont un annuaire signé a été ingéré
 * (`GET /api/daemon/nodes`) + les identités abonnées qui n'ont pas encore
 * annoncé (lignes « en attente », croisées depuis la liste des
 * subscriptions — un seul attention set, kickoff Q3/DQ3).
 *
 * Verrou 2 : cette page est ADDITIVE — la grille `/browse` reste l'index
 * par défaut et le sur-ensemble honnête ; ici on regarde les mêmes apps
 * par éditeur. Verrou 5 : le cold-start (« ajoute une ancre ») n'apparaît
 * que déclenché par l'état observé (aucun nœud), jamais poussé au publish.
 */

import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import {
  Anchor,
  ChevronRight,
  Globe,
  Radio,
  RefreshCw,
  Server,
} from "lucide-react";

import {
  listCurators,
  listNodes,
  type NodeSummary,
  type ObservedNode,
} from "@/api/daemon";
import { AddAnchorDialog } from "@/components/AddAnchorDialog";
import { DaemonOfflineBanner } from "@/pages/Browse";
import {
  selectActiveCoordinator,
  useProjectStore,
} from "@/stores/projectStore";

export default function Nodes() {
  const active = useProjectStore(selectActiveCoordinator);

  if (!active) {
    return (
      <div className="space-y-6">
        <PageHeader />
        <div className="glass-card max-w-md p-6">
          <h3 className="mb-2 font-bold">Aucun noeud actif</h3>
          <p className="text-sm text-white/50">
            Connecte-toi a un noeud depuis l&apos;en-tete pour explorer les
            catalogues du reseau.
          </p>
        </div>
      </div>
    );
  }

  return <NodesContent coordUrl={active.url} />;
}

function PageHeader() {
  return (
    <div>
      <h1 className="text-3xl font-extrabold tracking-tight">Noeuds</h1>
      <p className="mt-1 text-sm text-white/50">
        Les noeuds qui publient un catalogue d&apos;apps. Un noeud est une
        source de decouverte — jamais une autorite : la preuve d&apos;une app
        reste la signature de son auteur.
      </p>
    </div>
  );
}

function NodesContent({ coordUrl }: { coordUrl: string }) {
  const [addOpen, setAddOpen] = useState(false);
  // UX-ARRIVAL : pré-remplissage du dialog déclenché UNIQUEMENT par le clic
  // « S'abonner » d'une ligne observée (intention explicite, verrou 3 : le
  // dialog ouvert à la main garde son placeholder inerte). La `key` du dialog
  // force un remount au changement, donc l'état du champ suit.
  const [anchorPrefill, setAnchorPrefill] = useState<string | null>(null);

  const nodesQuery = useQuery({
    queryKey: ["daemon-nodes", coordUrl],
    queryFn: () => listNodes(coordUrl),
    staleTime: 30_000,
    refetchOnWindowFocus: false,
  });

  // Les subscriptions sans annuaire ingéré rendent une ligne « en attente »
  // (cold-start honnête : subscribe n'est pas une ingestion synchrone).
  const curatorsQuery = useQuery({
    queryKey: ["daemon-curators", coordUrl],
    queryFn: () => listCurators(coordUrl),
    staleTime: 30_000,
    refetchOnWindowFocus: false,
  });

  if (nodesQuery.isLoading) {
    return (
      <div className="space-y-6">
        <PageHeader />
        <div className="glass-card p-6 text-sm text-white/50">Chargement...</div>
      </div>
    );
  }

  if (nodesQuery.isError) {
    return (
      <div className="space-y-6">
        <PageHeader />
        <div className="glass-card p-6">
          <h3 className="mb-1 font-bold text-red-300">Erreur reseau</h3>
          <p className="text-sm text-white/50">
            {nodesQuery.error instanceof Error
              ? nodesQuery.error.message
              : "erreur inconnue"}
          </p>
        </div>
      </div>
    );
  }

  const result = nodesQuery.data;
  if (result?.kind === "unavailable") {
    return (
      <div className="space-y-6">
        <PageHeader />
        <DaemonOfflineBanner reason={result.reason} />
      </div>
    );
  }
  if (result?.kind === "error") {
    return (
      <div className="space-y-6">
        <PageHeader />
        <div className="glass-card p-6">
          <h3 className="mb-1 font-bold text-red-300">Noeud refuse</h3>
          <p className="text-sm text-white/50">{result.reason}</p>
        </div>
      </div>
    );
  }

  const nodes: NodeSummary[] = result?.kind === "data" ? result.body.nodes : [];
  // UX-ARRIVAL : les éditeurs d'annuaire entendus sur le gossip SANS
  // abonnement (métadonnées cheap-envelope, le daemon ne fetch jamais leur
  // catalogue). `?? []` = tolérance pour un daemon antérieur à la clé.
  const observed: ObservedNode[] =
    result?.kind === "data" ? (result.body.observed ?? []) : [];
  const knownIds = new Set(nodes.map((n) => n.node_id));
  // Le cold-start exige des subscriptions CONNUES-vides : tant que la query
  // charge OU répond non-data (daemon indisponible, erreur), l'état des
  // abonnements est INCONNU — afficher « aucun noeud connu » serait un
  // mensonge transitoire (review F) ou carrément faux (GAP Codex R2 : un
  // /curators en échec collapsait à [] et déclenchait le CTA).
  const curatorsResult = curatorsQuery.data;
  const subsKnown = curatorsResult?.kind === "data";
  const subscribed =
    curatorsResult?.kind === "data"
      ? curatorsResult.body.subscribed_curators
      : [];
  // B6 (discriminateur curateur/ancre) : l'attention set est UNIQUE (Q3/DQ3),
  // donc une ligne « en attente » peut être soit une ancre dont on n'a pas
  // encore ingéré l'annuaire de nœud, soit un curateur dont on a déjà la liste
  // signée mais qui ne publiera peut-être JAMAIS d'annuaire. On les distingue :
  // une identité présente dans les `entries` (listes de curation ingérées)
  // curate déjà — sinon c'est une ancre en attente de sa première annonce.
  const curatingHexes = new Set(
    curatorsResult?.kind === "data"
      ? curatorsResult.body.entries.map((e) => bytesToHex(e.curator_pubkey))
      : [],
  );
  const waiting = subscribed.filter((hex) => !knownIds.has(hex));
  // Un nœud observé n'est pas « rien » : le cold-start ne s'affiche que si
  // les trois familles (catalogues, en-attente, observés) sont vides.
  const isEmpty =
    nodes.length === 0 &&
    waiting.length === 0 &&
    observed.length === 0 &&
    subsKnown;

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-3">
        <PageHeader />
        <div className="flex items-center gap-2">
          <button
            onClick={() => nodesQuery.refetch()}
            disabled={nodesQuery.isFetching}
            className="glass-pill flex items-center gap-2 text-xs"
            data-testid="nodes-refresh"
          >
            <RefreshCw
              className={`h-3.5 w-3.5 ${nodesQuery.isFetching ? "animate-spin" : ""}`}
            />
            Rafraichir
          </button>
          <button
            onClick={() => setAddOpen(true)}
            className="flex items-center gap-2 rounded-lg bg-purple-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-purple-500"
            data-testid="nodes-add-anchor"
          >
            <Anchor className="h-4 w-4" />
            Ajouter une ancre
          </button>
        </div>
      </div>

      {isEmpty ? (
        <ColdStart onAdd={() => setAddOpen(true)} />
      ) : (
        <>
          {(nodes.length > 0 || waiting.length > 0) && (
            <div className="space-y-3" data-testid="nodes-list">
              {nodes.map((node) => (
                <NodeRow key={node.node_id} node={node} />
              ))}
              {waiting.map((hex) => (
                <WaitingRow
                  key={hex}
                  pubkeyHex={hex}
                  isCurator={curatingHexes.has(hex)}
                />
              ))}
            </div>
          )}
          {observed.length > 0 && (
            <ObservedSection
              observed={observed}
              onSubscribe={(nodeId) => {
                setAnchorPrefill(nodeId);
                setAddOpen(true);
              }}
            />
          )}
        </>
      )}

      <AddAnchorDialog
        key={anchorPrefill ?? "manual"}
        open={addOpen}
        onOpenChange={(open) => {
          setAddOpen(open);
          if (!open) setAnchorPrefill(null);
        }}
        coordUrl={coordUrl}
        initialPubkey={anchorPrefill ?? undefined}
      />
    </div>
  );
}

/**
 * UX-ARRIVAL — « Nœuds découverts sur le réseau » : les éditeurs entendus
 * par gossip sans abonnement. Le daemon n'a JAMAIS fetché leur catalogue
 * (anti-amplification) — on n'affiche donc ni compte d'apps ni révision,
 * seulement l'identité et un CTA d'abonnement explicite (verrou 5 :
 * l'utilisateur choisit ses sources, rien ne s'auto-abonne).
 */
function ObservedSection({
  observed,
  onSubscribe,
}: {
  observed: ObservedNode[];
  onSubscribe: (nodeId: string) => void;
}) {
  return (
    <div className="space-y-3" data-testid="nodes-observed-section">
      <div>
        <h2 className="text-lg font-bold text-white/80">
          Nœuds découverts sur le réseau
        </h2>
        <p className="mt-1 text-xs text-white/40">
          Entendus sur le réseau sans abonnement — identité seulement, leur
          catalogue n&apos;est jamais téléchargé tant que tu ne les suis pas.
        </p>
      </div>
      {observed.map((node) => (
        <ObservedRow
          key={node.node_id}
          node={node}
          onSubscribe={onSubscribe}
        />
      ))}
    </div>
  );
}

function ObservedRow({
  node,
  onSubscribe,
}: {
  node: ObservedNode;
  onSubscribe: (nodeId: string) => void;
}) {
  return (
    <div
      className="glass-card flex items-center justify-between gap-4 p-5"
      data-testid="node-observed-row"
    >
      <div className="flex items-center gap-3">
        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-white/[0.04]">
          <Radio className="h-5 w-5 text-white/30" />
        </div>
        <div>
          <p className="font-mono text-sm font-bold text-white/70">
            {truncateHex(node.node_id)}
          </p>
          {/* Copy prudente (review SEC-UXARR-3) : on décrit NOTRE observation
              (une annonce entendue), pas une agence prouvée du nœud — le champ
              identité d'une annonce gossip n'est pas authentifié. */}
          <p className="mt-1 text-xs text-white/40">
            Annonce entendue sur le réseau — abonne-toi pour voir son
            catalogue.
          </p>
        </div>
      </div>
      <button
        onClick={() => onSubscribe(node.node_id)}
        className="glass-pill flex items-center gap-2 text-xs"
        data-testid="observed-subscribe-cta"
      >
        <Anchor className="h-3.5 w-3.5" />
        S&apos;abonner
      </button>
    </div>
  );
}

function NodeRow({ node }: { node: NodeSummary }) {
  return (
    <Link
      to={`/node/${encodeURIComponent(node.node_id)}`}
      className="glass-card group flex items-center justify-between gap-4 p-5 transition-all duration-300 hover:-translate-y-0.5 hover:shadow-[0_8px_40px_rgba(120,80,255,0.15)]"
      data-testid="node-row"
    >
      <div className="flex items-center gap-3">
        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-purple-500/10">
          <Server className="h-5 w-5 text-purple-300" />
        </div>
        <div>
          <p className="font-mono text-sm font-bold text-white/90">
            {truncateHex(node.node_id)}
          </p>
          <div className="mt-1 flex items-center gap-2">
            <span className="glass-pill py-0.5 text-[10px] text-white/60">
              {node.app_count} app{node.app_count > 1 ? "s" : ""}
            </span>
            <span className="glass-pill py-0.5 text-[10px] text-white/60">
              rev. {node.revision}
            </span>
          </div>
        </div>
      </div>
      <ChevronRight className="h-4 w-4 text-white/30 transition-transform group-hover:translate-x-0.5" />
    </Link>
  );
}

function WaitingRow({
  pubkeyHex,
  isCurator,
}: {
  pubkeyHex: string;
  isCurator: boolean;
}) {
  return (
    <div
      className="glass-card p-5"
      data-testid="node-waiting-row"
      data-kind={isCurator ? "curator" : "anchor"}
    >
      <div className="flex items-center gap-3">
        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-white/[0.04]">
          <Server className="h-5 w-5 text-white/30" />
        </div>
        <div>
          <p className="font-mono text-sm font-bold text-white/50">
            {truncateHex(pubkeyHex)}
          </p>
          {/* B6 (discriminateur curateur/ancre) : copy honnête — l'attention
              set est UNIQUE (Q3/DQ3). Une identité dont on a déjà ingéré une
              liste de curation signée curate ; elle ne publiera peut-être
              JAMAIS d'annuaire de nœud. Une ancre sans rien d'ingéré est en
              attente de sa première annonce. On décrit l'état observé sans
              promettre une annonce future (review F). */}
          {isCurator ? (
            <p className="mt-1 text-xs text-white/40">
              Curateur — listes de curation signées suivies ; aucun annuaire de
              nœud publié.
            </p>
          ) : (
            <p className="mt-1 text-xs text-white/40">
              Ancre abonnée — en attente de sa première annonce d&apos;annuaire.
            </p>
          )}
        </div>
      </div>
    </div>
  );
}

/** Convert a `[u8; 32]`-style byte array (as sent by the daemon) to lowercase
 *  hex so a curator entry's `curator_pubkey` can be matched against the
 *  hex-encoded subscribed-curator list (B6 discriminator). */
function bytesToHex(bytes: number[]): string {
  return bytes.map((b) => b.toString(16).padStart(2, "0")).join("");
}

function ColdStart({ onAdd }: { onAdd: () => void }) {
  return (
    <div
      className="flex min-h-[40vh] items-center justify-center"
      data-testid="nodes-cold-start"
    >
      <div className="glass-card max-w-md p-8 text-center">
        <Globe className="mx-auto mb-4 h-10 w-10 text-white/20" />
        <h3 className="mb-2 font-bold">Aucun noeud-catalogue connu</h3>
        <p className="mb-5 text-sm text-white/50">
          Abonne-toi a l&apos;identite d&apos;un noeud (la tienne sur un autre
          ordinateur, celle d&apos;un ami, ou une ancre communautaire) pour
          decouvrir les apps qu&apos;il garde en ligne. N&apos;importe qui
          peut monter une ancre — il n&apos;y a pas de serveur central.
        </p>
        <button
          onClick={onAdd}
          className="inline-flex items-center gap-2 rounded-lg bg-purple-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-purple-500"
          data-testid="cold-start-add-anchor"
        >
          <Anchor className="h-4 w-4" />
          Ajouter une ancre
        </button>
      </div>
    </div>
  );
}

function truncateHex(hex: string): string {
  if (hex.length <= 16) return hex;
  return `${hex.slice(0, 8)}...${hex.slice(-8)}`;
}

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = Nodes;
