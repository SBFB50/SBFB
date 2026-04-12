// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/browse/:projectId` — Sprint 11 Phase C full-screen project view.
 *
 * Reached by clicking a card on the `/browse` page. Shows a sidebar
 * with project metadata (resolved from the daemon browse list) and
 * a main area that renders the project's apps full-screen via
 * `TabViewRenderer` when the project is hosted on the local
 * coordinator.
 *
 * Remote projects (cross-node) display a placeholder message — P2P
 * cross-node fetch is scope-cut to Sprint 12+.
 */

import { useMemo } from "react";
import { Link, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { ArrowLeft } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";
import {
  blobServeUrl,
  daemonBaseUrlFromInfo,
  getDaemonInfo,
  listBrowse,
  type BrowseEntry,
  type DaemonInfo,
} from "@/api/daemon";
import {
  getAppManifest,
  getAppTabDescriptor,
  listApps,
} from "@/api/coordinator";
import {
  selectActiveCoordinator,
  useProjectStore,
} from "@/stores/projectStore";
import { TabViewRenderer } from "@/components/app/tabview/TabViewRenderer";
import { TabAppContext } from "@/components/app/tabview/TabAppContext";

export default function BrowsedProject() {
  const { projectId } = useParams<{ projectId: string }>();
  const active = useProjectStore(selectActiveCoordinator);

  if (!active || !projectId) {
    return (
      <div className="space-y-4">
        <BackLink />
        <Card>
          <CardHeader>
            <CardTitle>Aucun coordinateur sélectionné</CardTitle>
          </CardHeader>
          <CardContent className="text-sm text-muted-foreground">
            Ajoute un coordinateur depuis l'en-tête pour explorer les projets.
          </CardContent>
        </Card>
      </div>
    );
  }

  return <BrowsedProjectContent coordUrl={active.url} projectId={projectId} />;
}

// =================================================================
// Main content — resolves browse entry + locality check
// =================================================================

function BrowsedProjectContent({
  coordUrl,
  projectId,
}: {
  coordUrl: string;
  projectId: string;
}) {
  const browseQuery = useQuery({
    queryKey: ["daemon-browse", coordUrl],
    queryFn: () => listBrowse(coordUrl),
    staleTime: 30_000,
  });

  const daemonInfoQuery = useQuery({
    queryKey: ["daemon-info", coordUrl],
    queryFn: () => getDaemonInfo(coordUrl),
    staleTime: 10_000,
  });

  const entry = useMemo(() => {
    if (browseQuery.data?.kind !== "data") return null;
    return (
      browseQuery.data.body.entries.find((e) => e.project_id === projectId) ??
      null
    );
  }, [browseQuery.data, projectId]);

  // Locality check: compare the browsed project_id (which is the
  // announcing daemon's node_id) against the daemon's own node_id
  // returned by GET /daemon/info.  The previous check compared
  // against the coordinator's node_id (GET /health) which is a
  // *different* iroh node — always false in production.
  const daemonInfo =
    daemonInfoQuery.data?.kind === "data"
      ? daemonInfoQuery.data.body
      : null;
  const isLocal = daemonInfo !== null && daemonInfo.node_id === projectId;

  if (browseQuery.isLoading || daemonInfoQuery.isLoading) {
    return (
      <div className="space-y-4">
        <BackLink />
        <Card>
          <CardContent className="py-6 text-sm text-muted-foreground">
            Chargement…
          </CardContent>
        </Card>
      </div>
    );
  }

  if (!entry) {
    return (
      <div className="space-y-4">
        <BackLink />
        <Card data-testid="project-not-found">
          <CardHeader>
            <CardTitle>Projet introuvable</CardTitle>
          </CardHeader>
          <CardContent className="text-sm text-muted-foreground">
            Aucun projet avec l'identifiant{" "}
            <code className="font-mono">{truncateHex(projectId)}</code>{" "}
            n'a été trouvé dans la liste de navigation.
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-4" data-testid="browsed-project">
      <BackLink projectName={entry.project_name} />
      <div className="grid gap-6 lg:grid-cols-[280px_1fr]">
        <ProjectSidebar entry={entry} isLocal={isLocal} />
        <div>
          {isLocal ? (
            <LocalProjectApps coordUrl={coordUrl} />
          ) : (
            <RemoteProjectFrame entry={entry} daemonInfo={daemonInfo} />
          )}
        </div>
      </div>
    </div>
  );
}

// =================================================================
// Sub-components
// =================================================================

function BackLink({ projectName }: { projectName?: string }) {
  return (
    <div className="flex items-center gap-3">
      <Link
        to="/browse"
        className="flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
        data-testid="back-to-browse"
      >
        <ArrowLeft className="h-4 w-4" />
        Retour
      </Link>
      {projectName && (
        <h1 className="text-2xl font-bold">{projectName}</h1>
      )}
    </div>
  );
}

function ProjectSidebar({
  entry,
  isLocal,
}: {
  entry: BrowseEntry;
  isLocal: boolean;
}) {
  return (
    <Card data-testid="project-sidebar">
      <CardHeader>
        <CardTitle className="text-base">{entry.project_name}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3 text-sm">
        <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-2">
          <dt className="text-muted-foreground">Catégorie</dt>
          <dd>
            <Badge variant="outline">{entry.category}</Badge>
          </dd>

          <dt className="text-muted-foreground">Description</dt>
          <dd className="text-muted-foreground">
            {entry.description || "—"}
          </dd>

          <dt className="text-muted-foreground">Source</dt>
          <dd>
            <Badge
              variant={(entry.source ?? "curator") === "direct" ? "default" : "outline"}
            >
              {(entry.source ?? "curator") === "direct" ? "Auto-publié" : "Curator"}
            </Badge>
          </dd>

          <dt className="text-muted-foreground">Curator</dt>
          <dd>{entry.curator_name}</dd>

          <dt className="text-muted-foreground">Accès</dt>
          <dd>
            <Badge variant={isLocal ? "default" : "outline"}>
              {isLocal ? "Local" : "Distant"}
            </Badge>
          </dd>

          <dt className="text-muted-foreground">Node ID</dt>
          <dd className="break-all font-mono text-xs">
            {truncateHex(entry.project_id)}
          </dd>
        </dl>
      </CardContent>
    </Card>
  );
}

// =================================================================
// Local project — full-screen app/tab rendering
// =================================================================

function LocalProjectApps({ coordUrl }: { coordUrl: string }) {
  const appsQuery = useQuery({
    queryKey: ["apps", coordUrl],
    queryFn: () => listApps(coordUrl),
    staleTime: 10_000,
  });

  if (appsQuery.isLoading) {
    return (
      <Card>
        <CardContent className="py-6 text-sm text-muted-foreground">
          Chargement des applications…
        </CardContent>
      </Card>
    );
  }
  if (appsQuery.isError) {
    return (
      <Card>
        <CardContent className="py-6 text-sm text-destructive">
          Erreur : {appsQuery.error.message}
        </CardContent>
      </Card>
    );
  }

  const apps = appsQuery.data?.apps ?? [];
  if (apps.length === 0) {
    return (
      <Card data-testid="no-apps">
        <CardContent className="py-6 text-sm text-muted-foreground">
          Aucune application installée sur ce projet.
        </CardContent>
      </Card>
    );
  }

  if (apps.length === 1) {
    return <SingleAppTabs coordUrl={coordUrl} appName={apps[0].name} />;
  }

  return (
    <Tabs defaultValue={apps[0].name}>
      <TabsList>
        {apps.map((app) => (
          <TabsTrigger key={app.name} value={app.name}>
            {app.name}
          </TabsTrigger>
        ))}
      </TabsList>
      {apps.map((app) => (
        <TabsContent key={app.name} value={app.name} className="pt-4">
          <SingleAppTabs coordUrl={coordUrl} appName={app.name} />
        </TabsContent>
      ))}
    </Tabs>
  );
}

function SingleAppTabs({
  coordUrl,
  appName,
}: {
  coordUrl: string;
  appName: string;
}) {
  const manifestQuery = useQuery({
    queryKey: ["app-manifest", coordUrl, appName],
    queryFn: () => getAppManifest(coordUrl, appName),
    staleTime: 30_000,
  });

  if (manifestQuery.isLoading) {
    return (
      <p className="text-sm text-muted-foreground">
        Chargement du manifest…
      </p>
    );
  }
  if (manifestQuery.isError) {
    return (
      <p className="text-sm text-destructive">
        Erreur : {manifestQuery.error.message}
      </p>
    );
  }

  const tabs = manifestQuery.data?.tabs ?? [];
  if (tabs.length === 0) {
    return (
      <p className="text-sm text-muted-foreground">
        Aucun onglet disponible.
      </p>
    );
  }

  return (
    <Tabs defaultValue={tabs[0].name}>
      <TabsList>
        {tabs.map((tab) => (
          <TabsTrigger key={tab.name} value={tab.name}>
            {tab.icon && <span className="mr-1">{tab.icon}</span>}
            {tab.name}
          </TabsTrigger>
        ))}
      </TabsList>
      {tabs.map((tab) => (
        <TabsContent key={tab.name} value={tab.name} className="pt-4">
          <AppTabContent
            coordUrl={coordUrl}
            appName={appName}
            tabName={tab.name}
          />
        </TabsContent>
      ))}
    </Tabs>
  );
}

function AppTabContent({
  coordUrl,
  appName,
  tabName,
}: {
  coordUrl: string;
  appName: string;
  tabName: string;
}) {
  const descriptorQuery = useQuery({
    queryKey: ["app-tab-descriptor", coordUrl, appName, tabName],
    queryFn: () => getAppTabDescriptor(coordUrl, appName, tabName),
    staleTime: 30_000,
  });

  if (descriptorQuery.isLoading) {
    return <p className="text-sm text-muted-foreground">Chargement…</p>;
  }
  if (descriptorQuery.isError) {
    return (
      <p className="text-sm text-destructive">
        Erreur : {descriptorQuery.error.message}
      </p>
    );
  }

  const result = descriptorQuery.data;
  if (!result) return null;

  if (result.kind === "error") {
    return (
      <p className="text-sm text-destructive">Erreur : {result.message}</p>
    );
  }

  return (
    <TabAppContext.Provider value={{ coordinatorUrl: coordUrl, appName }}>
      <TabViewRenderer tabView={result.tabView} />
    </TabAppContext.Provider>
  );
}

// =================================================================
// Remote project — iframe rendering or placeholder
// =================================================================

function RemoteProjectFrame({
  entry,
  daemonInfo,
}: {
  entry: BrowseEntry;
  daemonInfo: DaemonInfo | null;
}) {
  if (!entry.archive_hash || !daemonInfo) {
    return (
      <Card data-testid="remote-placeholder">
        <CardHeader>
          <CardTitle>Projet distant</CardTitle>
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          Ce projet est hébergé sur un noeud distant. Aucune archive
          n'est disponible pour le rendu local. Connectez-vous
          directement au coordinateur du projet pour y accéder.
        </CardContent>
      </Card>
    );
  }

  const daemonUrl = daemonBaseUrlFromInfo(daemonInfo);
  const iframeSrc = blobServeUrl(daemonUrl, entry.archive_hash);

  return (
    <div className="flex flex-col" data-testid="remote-iframe">
      <div className="rounded-t-lg border border-b-0 border-amber-500/30 bg-amber-900/20 px-4 py-2 text-sm text-amber-200">
        Contenu publié par un tiers — non vérifié par SBFB
      </div>
      <iframe
        src={iframeSrc}
        sandbox="allow-scripts"
        className="min-h-[600px] w-full rounded-b-lg border border-border"
        title={entry.project_name}
        data-testid="remote-iframe-element"
      />
    </div>
  );
}

// =================================================================
// Helpers
// =================================================================

function truncateHex(hex: string): string {
  if (hex.length <= 16) return hex;
  return `${hex.slice(0, 8)}…${hex.slice(-8)}`;
}

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = BrowsedProject;
