// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/browse` — Netflix-style glassmorphism app browser.
 */

import { useState } from "react";
import { keepPreviousData, useQuery, type UseQueryResult } from "@tanstack/react-query";
import { Link, useNavigate } from "react-router-dom";
import { ExternalLink, FileCheck, Globe, Play, RefreshCw, Search, Server, Signal, SignalZero, Sparkles } from "lucide-react";

import {
  browsePull,
  listBrowse,
  searchBrowse,
  type BrowseEntry,
  type BrowseStatus,
  type DaemonResult,
  type SearchResponse,
  type SearchResult,
} from "@/api/daemon";
import {
  selectActiveCoordinator,
  useProjectStore,
} from "@/stores/projectStore";

export default function Browse() {
  const active = useProjectStore(selectActiveCoordinator);

  if (!active) {
    return (
      <div className="flex min-h-[80vh] items-center justify-center">
        <div className="glass-card max-w-md p-8 text-center">
          <Globe className="mx-auto mb-4 h-12 w-12 text-purple-400" />
          <h2 className="mb-2 text-xl font-bold">
            Aucun noeud actif
          </h2>
          <p className="text-sm text-white/60">
            Connecte-toi a un noeud depuis l'en-tete pour explorer le
            reseau P2P.
          </p>
        </div>
      </div>
    );
  }

  return <BrowseContent coordUrl={active.url} />;
}

function BrowseContent({ coordUrl }: { coordUrl: string }) {
  const [searchTerm, setSearchTerm] = useState("");
  const trimmed = searchTerm.trim();
  const isSearching = trimmed.length > 0;

  const browseQuery = useQuery({
    queryKey: ["daemon-browse", coordUrl],
    queryFn: () => listBrowse(coordUrl),
    staleTime: 30_000,
    refetchOnWindowFocus: false,
  });

  // Sprint 73 Phase E (D4) — the dedicated search field hits the
  // daemon FTS5 index only while there's a non-empty query;
  // `keepPreviousData` keeps the last hits visible while the next
  // keystroke's query resolves so the grid doesn't flash empty.
  const searchQuery = useQuery({
    queryKey: ["daemon-search", coordUrl, trimmed],
    queryFn: () => searchBrowse(coordUrl, trimmed),
    enabled: isSearching,
    placeholderData: keepPreviousData,
    staleTime: 10_000,
    refetchOnWindowFocus: false,
  });

  const result = browseQuery.data;
  const entries =
    result?.kind === "data" ? result.body.entries : [];

  // Daemon offline or proxy error — show banner before grid
  const daemonOffline = result?.kind === "unavailable";
  const proxyError = result?.kind === "error";

  return (
    <div className="space-y-8">
      {/* ---- Hero (browse mode only) ---- */}
      {!isSearching && entries.length > 0 && <HeroSection entry={entries[0]} />}

      {/* ---- Top bar ---- */}
      <div className="flex items-end justify-between gap-4 px-2">
        <div>
          <h1 className="text-3xl font-extrabold tracking-tight">
            Explorer
          </h1>
          <p className="mt-1 text-sm text-white/50">
            Apps distribuees sur le reseau SBFB
          </p>
        </div>
        <div className="flex items-center gap-2">
          {/* Sprint 75 Phase F (Q6: additive cohabitation, verrou 2) — the
              by-node lens NEVER replaces this grid (the honest superset);
              it is an extra surface, like F-Droid's Repositories screen. */}
          <Link
            to="/nodes"
            className="glass-pill flex items-center gap-2 text-xs"
            data-testid="browse-by-node-link"
          >
            <Server className="h-3.5 w-3.5" />
            Parcourir par noeud
          </Link>
          <button
            onClick={async () => {
              await browsePull(coordUrl);
              setTimeout(() => browseQuery.refetch(), 2000);
            }}
            disabled={browseQuery.isFetching}
            className="glass-pill flex items-center gap-2 text-xs"
            data-testid="browse-refresh"
          >
            <RefreshCw
              className={`h-3.5 w-3.5 ${browseQuery.isFetching ? "animate-spin" : ""}`}
            />
            Rafraichir
          </button>
        </div>
      </div>

      {/* ---- Search bar ---- */}
      <SearchBar value={searchTerm} onChange={setSearchTerm} />

      {/* ---- Content ---- */}
      {isSearching ? (
        <SearchResultsView query={searchQuery} term={trimmed} />
      ) : browseQuery.isLoading ? (
        <LoadingSkeleton />
      ) : browseQuery.isError ? (
        <ErrorCard
          message={
            browseQuery.error instanceof Error
              ? browseQuery.error.message
              : "erreur inconnue"
          }
        />
      ) : daemonOffline ? (
        <DaemonOfflineBanner reason={result.reason} />
      ) : proxyError ? (
        <ErrorCard message={result.reason} />
      ) : entries.length === 0 ? (
        <EmptyState />
      ) : (
        <AppGrid entries={entries} />
      )}
    </div>
  );
}

// ================================================================
// Search bar + results (Sprint 73 Phase E)
// ================================================================

/**
 * `repo_url` is feed-sourced and React does not sanitize an anchor
 * `href`, so a `javascript:`/`data:` scheme would otherwise reach
 * the DOM. Only treat an explicit `https://` origin as a link
 * target (the forges SBFB clones apps from). Cheap hardening ahead
 * of S74 making the provenance triplet action-driving.
 */
function isHttpsUrl(url: string | null | undefined): url is string {
  return typeof url === "string" && url.startsWith("https://");
}

function SearchBar({
  value,
  onChange,
}: {
  value: string;
  onChange: (next: string) => void;
}) {
  return (
    <div className="px-2">
      <div className="glass-card flex items-center gap-3 px-4 py-3">
        <Search className="h-4 w-4 shrink-0 text-white/40" />
        <input
          type="search"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder="Rechercher une app par nom, catégorie ou description"
          aria-label="Rechercher une app"
          data-testid="browse-search-input"
          className="w-full bg-transparent text-sm text-white placeholder:text-white/30 focus:outline-none"
        />
        {value.length > 0 && (
          <button
            type="button"
            onClick={() => onChange("")}
            aria-label="Effacer la recherche"
            data-testid="browse-search-clear"
            className="shrink-0 text-xs text-white/40 hover:text-white/70"
          >
            Effacer
          </button>
        )}
      </div>
    </div>
  );
}

function SearchResultsView({
  query,
  term,
}: {
  query: UseQueryResult<DaemonResult<SearchResponse>>;
  term: string;
}) {
  const result = query.data;

  // SEARCH-VIEW-THROW-SKELETON (carry S73): callDaemon THROWS an
  // ApiProtocolError on a Zod drift, which React Query surfaces as `isError`
  // with `data === undefined`. Without this branch the view falls through to
  // the loading skeleton forever. Render an error card instead.
  if (query.isError) {
    return (
      <ErrorCard
        message={
          query.error instanceof Error
            ? query.error.message
            : "Erreur de recherche"
        }
      />
    );
  }
  if (query.isLoading || result === undefined) {
    return <LoadingSkeleton />;
  }
  if (result.kind === "unavailable") {
    return <DaemonOfflineBanner reason={result.reason} />;
  }
  if (result.kind === "error") {
    return <ErrorCard message={result.reason} />;
  }

  const hits = result.body.results;
  if (hits.length === 0) {
    return <SearchEmptyState term={term} />;
  }

  return (
    <div>
      <h3 className="mb-4 px-2 text-lg font-bold text-white/80">
        Résultats
        <span className="ml-2 text-sm font-normal text-white/40">
          {result.body.total}
        </span>
      </h3>
      <div
        className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
        data-testid="browse-search-grid"
      >
        {hits.map((hit, idx) => (
          <SearchHitCard key={`${hit.project_id}-${idx}`} hit={hit} />
        ))}
      </div>
    </div>
  );
}

function SearchHitCard({ hit }: { hit: SearchResult }) {
  const navigate = useNavigate();
  const repoUrl = isHttpsUrl(hit.repo_url) ? hit.repo_url : null;

  return (
    <div
      data-testid="browse-search-card"
      className="glass-card group relative cursor-pointer overflow-hidden p-4 transition-all duration-300 hover:-translate-y-1 hover:shadow-[0_8px_40px_rgba(120,80,255,0.15)]"
      onClick={() => navigate(`/browse/${hit.project_id}`)}
    >
      <div className="space-y-3">
        <div className="flex items-start justify-between gap-2">
          <h3 className="font-bold leading-tight">{hit.project_name}</h3>
          {hit.is_open_source && (
            <span className="shrink-0 rounded-full bg-emerald-500/15 px-2 py-0.5 text-[10px] font-medium text-emerald-400">
              Source vérifiable
            </span>
          )}
        </div>

        <p className="line-clamp-2 text-xs text-white/50">
          {hit.description || "Application P2P"}
        </p>

        <div className="flex flex-wrap items-center gap-1.5">
          {hit.category && (
            <span className="rounded-full bg-white/[0.06] px-2.5 py-0.5 text-[10px] font-medium text-white/60">
              {hit.category}
            </span>
          )}
          {hit.archive_hash && (
            <span className="rounded-full bg-emerald-500/15 px-2.5 py-0.5 text-[10px] font-medium text-emerald-400">
              P2P
            </span>
          )}
          {hit.provenance_hash && (
            <span
              className="inline-flex items-center gap-1 rounded-full bg-emerald-500/15 px-2.5 py-0.5 text-[10px] font-medium text-emerald-400"
              data-testid="search-verified-badge"
            >
              <FileCheck className="h-2.5 w-2.5" />
              Provenance
            </span>
          )}
          {repoUrl && (
            <a
              href={repoUrl}
              target="_blank"
              rel="noopener noreferrer"
              onClick={(e) => e.stopPropagation()}
              className="inline-flex items-center gap-1 rounded-full bg-white/[0.06] px-2.5 py-0.5 text-[10px] font-medium text-white/60 hover:bg-white/10 hover:text-white"
              data-testid="search-repo-link"
            >
              <ExternalLink className="h-2.5 w-2.5" />
              Source
            </a>
          )}
        </div>
      </div>
    </div>
  );
}

function SearchEmptyState({ term }: { term: string }) {
  return (
    <div
      className="flex min-h-[40vh] items-center justify-center"
      data-testid="browse-search-empty"
    >
      <div className="glass-card max-w-sm p-8 text-center">
        <Search className="mx-auto mb-4 h-10 w-10 text-white/20" />
        <h3 className="mb-2 font-bold">Aucun résultat</h3>
        <p className="text-sm text-white/50">
          Aucune app ne correspond à « {term} ». Essaie d'autres mots-clés.
        </p>
      </div>
    </div>
  );
}

// ================================================================
// Hero — featured app like a Netflix banner
// ================================================================

function HeroSection({ entry }: { entry: BrowseEntry }) {
  const navigate = useNavigate();
  return (
    <div
      className="group relative -mx-6 -mt-6 cursor-pointer overflow-hidden"
      onClick={() => navigate(`/browse/${entry.project_id}`)}
    >
      {/* Gradient background */}
      <div className="absolute inset-0 bg-gradient-to-br from-purple-900/80 via-indigo-950/60 to-transparent" />
      <div className="absolute inset-0 bg-gradient-to-t from-[#0a0a0f] via-transparent to-transparent" />

      {/* Animated glow */}
      <div className="absolute -left-20 -top-20 h-60 w-60 rounded-full bg-purple-600/20 blur-[80px] transition-all duration-700 group-hover:bg-purple-500/30" />
      <div className="absolute -bottom-10 -right-10 h-40 w-40 rounded-full bg-indigo-600/15 blur-[60px]" />

      <div className="relative flex min-h-[320px] items-end px-10 pb-10 pt-20">
        <div className="max-w-xl space-y-4">
          <div className="flex items-center gap-2">
            <Sparkles className="h-4 w-4 text-purple-400" />
            <span className="text-xs font-medium uppercase tracking-widest text-purple-300">
              En vedette
            </span>
          </div>
          <h2 className="text-4xl font-black tracking-tight">
            {entry.project_name}
          </h2>
          <p className="text-base text-white/70">
            {entry.description || "Application distribuee sur le reseau SBFB"}
          </p>
          <div className="flex items-center gap-3 pt-2">
            <button className="flex items-center gap-2 rounded-lg bg-white px-6 py-2.5 text-sm font-bold text-black transition-transform hover:scale-105">
              <Play className="h-4 w-4 fill-current" />
              Ouvrir
            </button>
            <StatusPill status={entry.status} />
            {entry.archive_hash && (
              <span className="glass-pill text-[11px] text-emerald-300">
                Archive P2P
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

// ================================================================
// App Grid — glass cards with hover effects
// ================================================================

function AppGrid({ entries }: { entries: BrowseEntry[] }) {
  return (
    <div>
      <h3 className="mb-4 px-2 text-lg font-bold text-white/80">
        Toutes les apps
        <span className="ml-2 text-sm font-normal text-white/40">
          {entries.length}
        </span>
      </h3>
      <div
        className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
        data-testid="browse-grid"
      >
        {entries.map((entry) => (
          <AppCard
            key={`${entry.project_id}-${entry.curator_pubkey}`}
            entry={entry}
          />
        ))}
      </div>
    </div>
  );
}

function AppCard({ entry }: { entry: BrowseEntry }) {
  const navigate = useNavigate();

  // Deterministic color from project name
  const hue = [...entry.project_name].reduce(
    (h, c) => h + c.charCodeAt(0),
    0,
  ) % 360;

  return (
    <div
      data-testid="browse-card"
      className="glass-card group relative cursor-pointer overflow-hidden transition-all duration-300 hover:-translate-y-1 hover:shadow-[0_8px_40px_rgba(120,80,255,0.15)]"
      onClick={() => navigate(`/browse/${entry.project_id}`)}
    >
      {/* Card header gradient */}
      <div
        className="h-28 transition-all duration-500 group-hover:scale-105"
        style={{
          background: `linear-gradient(135deg, hsl(${hue}, 60%, 20%) 0%, hsl(${hue + 40}, 50%, 12%) 100%)`,
        }}
      >
        <div className="flex h-full items-center justify-center">
          <span className="text-4xl font-black uppercase tracking-wider text-white/20">
            {entry.project_name.slice(0, 2)}
          </span>
        </div>
        {/* Glow on hover */}
        <div
          className="absolute left-1/2 top-12 h-16 w-16 -translate-x-1/2 rounded-full opacity-0 blur-[30px] transition-opacity duration-500 group-hover:opacity-100"
          style={{ background: `hsl(${hue}, 70%, 50%)` }}
        />
      </div>

      {/* Card body */}
      <div className="space-y-3 p-4">
        <div className="flex items-start justify-between gap-2">
          <h3 className="font-bold leading-tight">
            {entry.project_name}
          </h3>
          <StatusDot status={entry.status} />
        </div>

        <p className="line-clamp-2 text-xs text-white/50">
          {entry.description || "Application P2P"}
        </p>

        <div className="flex flex-wrap items-center gap-1.5">
          <span className="rounded-full bg-white/[0.06] px-2.5 py-0.5 text-[10px] font-medium text-white/60">
            {entry.category}
          </span>
          {(entry.source ?? "curator") === "direct" && (
            <span
              className="rounded-full bg-purple-500/20 px-2.5 py-0.5 text-[10px] font-medium text-purple-300"
              data-testid="source-badge-direct"
            >
              Upload direct
            </span>
          )}
          {entry.archive_hash && (
            <span className="rounded-full bg-emerald-500/15 px-2.5 py-0.5 text-[10px] font-medium text-emerald-400">
              P2P
            </span>
          )}
          {entry.provenance_hash && (
            <span
              className="inline-flex items-center gap-1 rounded-full bg-emerald-500/15 px-2.5 py-0.5 text-[10px] font-medium text-emerald-400"
              data-testid="verified-badge"
            >
              <FileCheck className="h-2.5 w-2.5" />
              Provenance
            </span>
          )}
          {isHttpsUrl(entry.repo_url) && (
            <a
              href={entry.repo_url}
              target="_blank"
              rel="noopener noreferrer"
              onClick={(e) => e.stopPropagation()}
              className="inline-flex items-center gap-1 rounded-full bg-white/[0.06] px-2.5 py-0.5 text-[10px] font-medium text-white/60 hover:bg-white/10 hover:text-white"
              data-testid="repo-link"
            >
              <ExternalLink className="h-2.5 w-2.5" />
              Source
            </a>
          )}
        </div>
      </div>

      {/* Play overlay on hover */}
      <div className="absolute inset-0 flex items-center justify-center bg-black/40 opacity-0 backdrop-blur-sm transition-all duration-300 group-hover:opacity-100">
        <div className="flex h-14 w-14 items-center justify-center rounded-full bg-white/20 backdrop-blur-md transition-transform duration-300 group-hover:scale-110">
          <Play className="h-6 w-6 fill-white text-white" />
        </div>
      </div>
    </div>
  );
}

// ================================================================
// Status indicators
// ================================================================

function StatusPill({ status }: { status: BrowseStatus }) {
  const isReachable = status === "reachable";
  return (
    <span
      className={`glass-pill flex items-center gap-1.5 text-[11px] ${
        isReachable ? "text-emerald-300" : "text-white/40"
      }`}
    >
      {isReachable ? (
        <Signal className="h-3 w-3" />
      ) : (
        <SignalZero className="h-3 w-3" />
      )}
      {isReachable ? "En ligne" : "Hors ligne"}
    </span>
  );
}

function StatusDot({ status }: { status: BrowseStatus }) {
  return (
    <span
      className={`mt-1.5 inline-block h-2 w-2 rounded-full ${
        status === "reachable"
          ? "bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.5)]"
          : status === "unreachable"
            ? "bg-red-400"
            : "bg-white/20"
      }`}
      title={status}
    />
  );
}

// ================================================================
// States
// ================================================================

function LoadingSkeleton() {
  return (
    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
      {Array.from({ length: 4 }).map((_, i) => (
        <div key={i} className="glass-card animate-pulse overflow-hidden">
          <div className="h-28 bg-white/[0.03]" />
          <div className="space-y-3 p-4">
            <div className="h-4 w-2/3 rounded bg-white/[0.06]" />
            <div className="h-3 w-full rounded bg-white/[0.04]" />
            <div className="h-3 w-1/2 rounded bg-white/[0.04]" />
          </div>
        </div>
      ))}
    </div>
  );
}

function EmptyState() {
  return (
    <div className="flex min-h-[40vh] items-center justify-center">
      <div className="glass-card max-w-sm p-8 text-center">
        <Globe className="mx-auto mb-4 h-10 w-10 text-white/20" />
        <h3 className="mb-2 font-bold">Aucune app</h3>
        <p className="text-sm text-white/50">
          Abonne-toi a un curator ou publie ta premiere app pour la
          voir apparaitre ici.
        </p>
      </div>
    </div>
  );
}

function ErrorCard({ message }: { message: string }) {
  return (
    <div className="glass-card border-red-500/20 p-6">
      <h3 className="mb-1 font-bold text-red-300">Erreur reseau</h3>
      <p className="text-sm text-white/50">{message}</p>
    </div>
  );
}

export function DaemonOfflineBanner({ reason }: { reason: string }) {
  return (
    <div className="glass-card border-amber-500/20 p-6" data-testid="daemon-offline-banner">
      <h3 className="mb-1 font-bold text-amber-300">
        Noeud indisponible
      </h3>
      <p className="text-sm text-white/50">
        Impossible de joindre le noeud{" "}
        <code className="font-mono text-white/70">nexus-shell-daemon</code>.
        Lance{" "}
        <code className="font-mono text-white/70">nexus-shell-daemon start</code>{" "}
        puis rafraichis.
      </p>
      <p className="mt-2 text-xs text-white/40">
        Détail technique : <span className="font-mono">{reason}</span>
      </p>
    </div>
  );
}

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = Browse;
