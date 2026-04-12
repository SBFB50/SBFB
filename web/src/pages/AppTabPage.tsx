/**
 * `/app/:appName/tabs/:tabName` — deep-link target for
 * Sprint 8 Phase E command palette navigation.
 *
 * The shell's Command Palette fetches `@nexus_command`
 * descriptors per enrolled app, and on invoke forwards the
 * handler's `{navigation: {path}}` payload to React Router.
 * This page is where those paths land: it resolves the active
 * coordinator from the Zustand store and renders the requested
 * tab descriptor via `TabViewRenderer`.
 *
 * Polish states (Phase E §8.4):
 *  - **Skeleton** : pulsing blocks while `getAppTabDescriptor`
 *    is in flight, so the tab-switch never shows a bare white
 *    page.
 *  - **Empty** : coordinator reachable but the handler returned
 *    an empty TabView (a `heading` + an `empty` block is the
 *    gov app's canonical shape — handled by the renderer itself
 *    so we only guard on the `blocks.length === 0` edge case).
 *  - **Error** : 422 / HTTP / Zod failure surfaced as a banner
 *    with the raw message + a retry button.
 *  - **No active coordinator** : banner explaining the user
 *    must pick a project from the header picker first.
 */

import { useQuery } from "@tanstack/react-query";
import { useNavigate, useParams } from "react-router-dom";
import { RefreshCw, ArrowLeft } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { getAppTabDescriptor } from "@/api/coordinator";
import { TabViewRenderer } from "@/components/app/tabview/TabViewRenderer";
import { TabAppContext } from "@/components/app/tabview/TabAppContext";
import { useAppEvents } from "@/hooks/useAppEvents";
import {
  selectActiveCoordinator,
  useProjectStore,
} from "@/stores/projectStore";

export default function AppTabPage() {
  const { appName, tabName } = useParams<{
    appName: string;
    tabName: string;
  }>();
  const navigate = useNavigate();
  const active = useProjectStore(selectActiveCoordinator);

  const descriptorQuery = useQuery({
    queryKey: ["app-tab", active?.url, appName, tabName],
    queryFn: () => {
      if (!active || !appName || !tabName) {
        throw new Error("missing active coordinator or route params");
      }
      return getAppTabDescriptor(active.url, appName, tabName);
    },
    enabled: Boolean(active && appName && tabName),
    staleTime: 5_000,
    retry: 0,
  });

  // Sprint 9 Phase C (D2 consumer): the gov Politiciens tab
  // subscribes to ``party.refreshed`` so a worker that updates
  // the party cache invalidates the descriptor query and
  // re-renders the grid without a manual refresh. Other tabs
  // are unaffected — the hook is a no-op when ``appName`` or
  // ``tabName`` does not match the gov Politiciens combination.
  const isGovPoliticiensTab = appName === "gov" && tabName === "Politiciens";
  useAppEvents({
    coordinatorUrl: isGovPoliticiensTab ? (active?.url ?? null) : null,
    appName: isGovPoliticiensTab ? appName : null,
    pattern: "party.refreshed",
    invalidateQueryKey: ["app-tab", active?.url, appName, tabName],
  });

  if (!appName || !tabName) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Paramètres manquants</CardTitle>
          <CardDescription>
            Le chemin doit être <code>/app/&lt;app&gt;/tabs/&lt;tab&gt;</code>.
          </CardDescription>
        </CardHeader>
      </Card>
    );
  }

  if (!active) {
    return (
      <div className="space-y-4">
        <header className="min-w-0">
          <h1 className="truncate text-2xl font-bold">
            {appName} — {tabName}
          </h1>
          <p className="text-sm text-muted-foreground">
            Aucun coordinateur actif.
          </p>
        </header>
        <Card>
          <CardHeader>
            <CardTitle>Sélectionner un projet d'abord</CardTitle>
            <CardDescription>
              Pour consulter un onglet d'une app, choisis un
              coordinateur actif dans le header (ou ajoute-en un
              via « Ajouter un coordinateur »).
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Button
              variant="outline"
              size="sm"
              onClick={() => navigate("/my-projects")}
            >
              <ArrowLeft className="size-4" />
              Voir mes projets
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div className="min-w-0">
          <h1 className="truncate text-2xl font-bold">
            {appName} — {tabName}
          </h1>
          <p className="truncate font-mono text-[11px] text-muted-foreground">
            {active.nickname || active.url}
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => descriptorQuery.refetch()}
          disabled={descriptorQuery.isFetching}
          data-testid="app-tab-refresh"
        >
          <RefreshCw
            className={
              descriptorQuery.isFetching ? "size-4 animate-spin" : "size-4"
            }
          />
          {descriptorQuery.isFetching ? "Chargement…" : "Recharger"}
        </Button>
      </header>

      {descriptorQuery.isLoading && <AppTabSkeleton />}

      {descriptorQuery.isError && (
        <Card>
          <CardHeader>
            <CardTitle>Erreur de chargement</CardTitle>
            <CardDescription>
              Impossible de contacter le coordinateur pour l'onglet{" "}
              <code>{tabName}</code>.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="mb-2 text-xs text-destructive">
              {descriptorQuery.error instanceof Error
                ? descriptorQuery.error.message
                : "erreur inconnue"}
            </p>
            <Button
              variant="outline"
              size="sm"
              onClick={() => descriptorQuery.refetch()}
            >
              Réessayer
            </Button>
          </CardContent>
        </Card>
      )}

      {descriptorQuery.data && descriptorQuery.data.kind === "error" && (
        <Card>
          <CardHeader>
            <CardTitle>Onglet indisponible</CardTitle>
            <CardDescription>
              Le coordinateur a répondu mais l'onglet ne peut pas
              être rendu.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="mb-2 text-xs text-destructive">
              {descriptorQuery.data.message}
            </p>
            <Button
              variant="outline"
              size="sm"
              onClick={() => descriptorQuery.refetch()}
            >
              Réessayer
            </Button>
          </CardContent>
        </Card>
      )}

      {descriptorQuery.data && descriptorQuery.data.kind === "schema" && (
        <TabAppContext.Provider
          value={{ coordinatorUrl: active.url, appName }}
        >
          {descriptorQuery.data.tabView.blocks.length === 0 ? (
            <Card>
              <CardContent className="p-6 text-sm text-muted-foreground">
                Onglet vide — l'app n'a pas encore de données à
                afficher. Lance un scan via la palette
                (<kbd className="font-mono">Ctrl + K</kbd>) pour
                peupler la base.
              </CardContent>
            </Card>
          ) : (
            <TabViewRenderer tabView={descriptorQuery.data.tabView} />
          )}
        </TabAppContext.Provider>
      )}
    </div>
  );
}

function AppTabSkeleton() {
  return (
    <div className="space-y-3" data-testid="app-tab-skeleton">
      <div className="h-6 w-48 animate-pulse rounded bg-muted" />
      <div className="h-20 w-full animate-pulse rounded bg-muted" />
      <div className="h-20 w-full animate-pulse rounded bg-muted" />
      <div className="h-32 w-full animate-pulse rounded bg-muted" />
    </div>
  );
}

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = AppTabPage;
