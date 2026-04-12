// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/browse` — Sprint 7 Phase E live view of every project any
 * subscribed curator vouches for, with a reachability dot next
 * to each row.
 *
 * Flow:
 *   shell  ─ GET /daemon/browse ─▶  coordinator proxy
 *                                    │
 *                                    ▼
 *                            nexus-shell-daemon
 *                            (aggregates curator lists +
 *                             probes endpoint reachability
 *                             through iroh pkarr dial)
 *
 * Rendering states (discriminated via the `DaemonResult<T>`
 * union from `@/api/daemon`):
 *
 * 1. No active coordinator      → onboarding pointer card
 * 2. Active coord, daemon off   → DaemonOfflineBanner + CTA
 * 3. Proxy error (400)          → inline error card
 * 4. Data, 0 entries            → "no curators yet" empty state
 * 5. Data, N entries            → grid of browse cards
 */

import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { BookmarkPlus, RefreshCw } from "lucide-react";

import { listBrowse, type BrowseEntry, type BrowseStatus } from "@/api/daemon";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  selectActiveCoordinator,
  useProjectStore,
} from "@/stores/projectStore";

export default function Browse() {
  const active = useProjectStore(selectActiveCoordinator);

  if (!active) {
    return (
      <div className="space-y-4">
        <PageHeader />
        <Card>
          <CardHeader>
            <CardTitle>Aucun coordinateur sélectionné</CardTitle>
            <CardDescription>
              Ajoute un coordinateur depuis l'en-tête pour
              interroger son daemon via{" "}
              <code className="font-mono">/daemon/browse</code>.
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  return <BrowseContent coordUrl={active.url} />;
}

function PageHeader() {
  return (
    <div>
      <h1 className="text-2xl font-bold">Explorer</h1>
      <p className="text-sm text-muted-foreground">
        Projets publics vouchés par les curators auxquels tu es
        abonné, résolus via la DHT iroh-pkarr par
        nexus-shell-daemon.
      </p>
    </div>
  );
}

function BrowseContent({ coordUrl }: { coordUrl: string }) {
  const query = useQuery({
    queryKey: ["daemon-browse", coordUrl],
    queryFn: () => listBrowse(coordUrl),
    // 30s stale time — the daemon has its own 60s TTL cache on
    // reachability probes, so polling here more often wastes
    // coordinator+daemon cycles without faster UI updates.
    staleTime: 30_000,
    refetchOnWindowFocus: false,
  });

  return (
    <div className="space-y-4">
      <div className="flex items-start justify-between gap-4">
        <PageHeader />
        <Button
          variant="outline"
          size="sm"
          onClick={() => query.refetch()}
          disabled={query.isFetching}
          data-testid="browse-refresh"
        >
          <RefreshCw className="mr-2 h-4 w-4" />
          Rafraîchir
        </Button>
      </div>

      {query.isLoading ? (
        <Card>
          <CardContent className="py-6 text-sm text-muted-foreground">
            Chargement…
          </CardContent>
        </Card>
      ) : query.isError ? (
        <Card>
          <CardHeader>
            <CardTitle>Erreur réseau</CardTitle>
            <CardDescription>
              {query.error instanceof Error
                ? query.error.message
                : "erreur inconnue"}
            </CardDescription>
          </CardHeader>
        </Card>
      ) : (
        <BrowseResultView result={query.data!} />
      )}
    </div>
  );
}

function BrowseResultView({
  result,
}: {
  result: Awaited<ReturnType<typeof listBrowse>>;
}) {
  if (result.kind === "unavailable") {
    return <DaemonOfflineBanner reason={result.reason} />;
  }
  if (result.kind === "error") {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Proxy daemon refusé</CardTitle>
          <CardDescription>{result.reason}</CardDescription>
        </CardHeader>
      </Card>
    );
  }

  const entries = result.body.entries;
  if (entries.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Aucun projet à explorer pour l'instant</CardTitle>
          <CardDescription>
            Abonne-toi à un curator via la page{" "}
            <strong>Curators</strong> pour que ses projets
            vouchés apparaissent ici.
          </CardDescription>
        </CardHeader>
      </Card>
    );
  }

  return (
    <div
      className="grid gap-3 md:grid-cols-2 xl:grid-cols-3"
      data-testid="browse-grid"
    >
      {entries.map((entry) => (
        <BrowseCard
          key={`${entry.project_id}-${entry.curator_pubkey}`}
          entry={entry}
        />
      ))}
    </div>
  );
}

function BrowseCard({ entry }: { entry: BrowseEntry }) {
  const navigate = useNavigate();
  return (
    <Card
      data-testid="browse-card"
      className="cursor-pointer transition-colors hover:border-foreground/20"
      onClick={() => navigate(`/browse/${entry.project_id}`)}
    >
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between gap-2">
          <div className="flex items-center gap-2">
            <BookmarkPlus className="h-4 w-4 text-muted-foreground" />
            <CardTitle className="text-base">
              {entry.project_name}
            </CardTitle>
          </div>
          <StatusBadge status={entry.status} />
        </div>
        <CardDescription className="font-mono text-xs">
          {truncateHex(entry.project_id)}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-2 text-sm">
        <p className="text-muted-foreground">
          {entry.description || "—"}
        </p>
        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <span className="flex items-center gap-1">
            <Badge variant="outline">{entry.category}</Badge>
            {(entry.source ?? "curator") === "direct" && (
              <Badge variant="secondary" className="text-[10px]" data-testid="source-badge-direct">
                Auto-publié
              </Badge>
            )}
          </span>
          <span>
            {(entry.source ?? "curator") === "direct" ? (
              "auto-publié"
            ) : (
              <>
                vouché par{" "}
                <strong className="text-foreground">
                  {entry.curator_name}
                </strong>
              </>
            )}
          </span>
        </div>
      </CardContent>
    </Card>
  );
}

function StatusBadge({ status }: { status: BrowseStatus }) {
  // The Rust enum ships three values; map each to a French
  // label + a semantic variant the design system already
  // carries.
  const label =
    status === "reachable"
      ? "Accessible"
      : status === "unreachable"
        ? "Injoignable"
        : "Inconnu";
  const variant =
    status === "reachable"
      ? "default"
      : status === "unreachable"
        ? "destructive"
        : "outline";
  return (
    <Badge variant={variant} data-testid={`browse-status-${status}`}>
      {label}
    </Badge>
  );
}

export function DaemonOfflineBanner({ reason }: { reason: string }) {
  return (
    <Card data-testid="daemon-offline-banner">
      <CardHeader>
        <CardTitle>Daemon indisponible</CardTitle>
        <CardDescription>
          Le coordinateur ne peut pas joindre{" "}
          <code className="font-mono">nexus-shell-daemon</code>.
          Démarre-le avec{" "}
          <code className="font-mono">nexus-shell-daemon start</code>{" "}
          puis rafraîchis la page.
        </CardDescription>
      </CardHeader>
      <CardContent className="text-xs text-muted-foreground">
        <p>
          Détail technique : <span className="font-mono">{reason}</span>
        </p>
      </CardContent>
    </Card>
  );
}

function truncateHex(hex: string): string {
  if (hex.length <= 16) return hex;
  return `${hex.slice(0, 8)}…${hex.slice(-8)}`;
}

// Sprint 9 Phase A (D6) — react-router lazy() looks up a named
// `Component` export when a route uses `lazy: () => import(...)`.
export const Component = Browse;
