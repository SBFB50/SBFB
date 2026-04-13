// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/browse/:projectId` — Full-screen immersive app viewer.
 *
 * The app fills the entire viewport. A glassmorphism top bar
 * auto-hides and reveals when the mouse approaches the top edge.
 */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { useBridge } from "@/bridge/useBridge";
import {
  ArrowLeft,
  ExternalLink,
  Globe,
  Shield,
} from "lucide-react";

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
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";

export default function BrowsedProject() {
  const { projectId } = useParams<{ projectId: string }>();
  const active = useProjectStore(selectActiveCoordinator);

  if (!active || !projectId) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-[#0a0a0f]">
        <div className="glass-card max-w-md p-8 text-center">
          <Globe className="mx-auto mb-4 h-12 w-12 text-white/20" />
          <h2 className="mb-2 text-xl font-bold text-white">
            Aucun coordinateur
          </h2>
          <p className="text-sm text-white/50">
            Ajoute un coordinateur depuis l'en-tete.
          </p>
        </div>
      </div>
    );
  }

  return <ProjectView coordUrl={active.url} projectId={projectId} />;
}

// ================================================================
// Main view — full screen
// ================================================================

function ProjectView({
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

  const daemonInfo =
    daemonInfoQuery.data?.kind === "data"
      ? daemonInfoQuery.data.body
      : null;
  const isLocal = daemonInfo !== null && daemonInfo.node_id === projectId;

  if (browseQuery.isLoading || daemonInfoQuery.isLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-[#0a0a0f]">
        <div className="h-8 w-8 animate-spin rounded-full border-2 border-purple-400 border-t-transparent" />
      </div>
    );
  }

  if (!entry) {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center bg-[#0a0a0f]">
        <div className="glass-card max-w-sm p-8 text-center" data-testid="project-not-found">
          <h3 className="mb-2 font-bold text-white">Projet introuvable</h3>
          <p className="mb-4 text-sm text-white/50">
            <code className="font-mono">{truncateHex(projectId)}</code>
          </p>
          <Link
            to="/browse"
            className="inline-flex items-center gap-1 text-sm text-purple-400 hover:text-purple-300"
            data-testid="back-to-browse"
          >
            <ArrowLeft className="h-4 w-4" />
            Retour
          </Link>
        </div>
      </div>
    );
  }

  const hasArchive = !!entry.archive_hash && !!daemonInfo;

  return (
    <FullScreenApp
      entry={entry}
      daemonInfo={daemonInfo}
      isLocal={isLocal}
      hasArchive={hasArchive}
      coordUrl={coordUrl}
    />
  );
}

// ================================================================
// Full screen layout with auto-hide top bar
// ================================================================

function FullScreenApp({
  entry,
  daemonInfo,
  isLocal,
  hasArchive,
  coordUrl,
}: {
  entry: BrowseEntry;
  daemonInfo: DaemonInfo | null;
  isLocal: boolean;
  hasArchive: boolean;
  coordUrl: string;
}) {
  const [barVisible, setBarVisible] = useState(true);
  const hideTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const iframeRef = useRef<HTMLIFrameElement | null>(null);

  // Sprint 13 Phase C: bridge listener for iframe ↔ coordinator.
  useBridge(coordUrl, entry.project_name, iframeRef);

  // Auto-hide after 3s
  useEffect(() => {
    hideTimer.current = setTimeout(() => setBarVisible(false), 3000);
    return () => {
      if (hideTimer.current) clearTimeout(hideTimer.current);
    };
  }, []);

  // Show bar when mouse near top (within 48px)
  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (e.clientY < 48) {
        setBarVisible(true);
        if (hideTimer.current) clearTimeout(hideTimer.current);
      } else if (e.clientY > 80 && barVisible) {
        hideTimer.current = setTimeout(() => setBarVisible(false), 1500);
      }
    },
    [barVisible],
  );

  const daemonUrl = daemonInfo
    ? daemonBaseUrlFromInfo(daemonInfo)
    : null;

  return (
    <div
      className="fixed inset-0 z-50 flex flex-col bg-[#0a0a0f]"
      onMouseMove={handleMouseMove}
      data-testid="browsed-project"
    >
      {/* ---- Auto-hide top bar ---- */}
      <div
        className={`absolute left-0 right-0 top-0 z-50 transition-all duration-500 ${
          barVisible
            ? "translate-y-0 opacity-100"
            : "-translate-y-full opacity-0"
        }`}
      >
        <div className="flex items-center justify-between gap-4 border-b border-white/[0.06] bg-black/60 px-5 py-3 backdrop-blur-xl">
          {/* Left side */}
          <div className="flex items-center gap-3">
            <Link
              to="/browse"
              className="flex items-center gap-1.5 rounded-full bg-white/[0.06] px-3 py-1.5 text-xs text-white/70 transition-colors hover:bg-white/10 hover:text-white"
              data-testid="back-to-browse"
            >
              <ArrowLeft className="h-3.5 w-3.5" />
              Explorer
            </Link>

            <div className="h-4 w-px bg-white/10" />

            <h1 className="text-sm font-bold text-white">
              {entry.project_name}
            </h1>

            <StatusDot status={entry.status} />

            {(entry.source ?? "curator") === "direct" && (
              <span className="rounded-full bg-purple-500/20 px-2 py-0.5 text-[10px] font-medium text-purple-300">
                Auto-publie
              </span>
            )}
          </div>

          {/* Right side */}
          <div className="flex items-center gap-2">
            {entry.archive_hash && (
              <span className="rounded-full bg-emerald-500/10 px-2 py-0.5 text-[10px] font-medium text-emerald-400/70">
                blob:{truncateHex(entry.archive_hash)}
              </span>
            )}

            {entry.archive_hash && daemonUrl && (
              <a
                href={blobServeUrl(daemonUrl, entry.archive_hash)}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-1 rounded-full bg-white/[0.06] px-3 py-1.5 text-[11px] text-white/60 transition-colors hover:bg-white/10 hover:text-white"
              >
                <ExternalLink className="h-3 w-3" />
                Nouvel onglet
              </a>
            )}

            {entry.repo_url && (
              <a
                href={entry.repo_url}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-1 rounded-full bg-white/[0.06] px-3 py-1.5 text-[11px] text-white/60 transition-colors hover:bg-white/10 hover:text-white"
                data-testid="repo-link"
              >
                <ExternalLink className="h-3 w-3" />
                Source
              </a>
            )}

            <span className="flex items-center gap-1 text-[11px] text-white/30">
              <Shield className="h-3 w-3" />
              sandbox
            </span>
          </div>
        </div>
      </div>

      {/* ---- Content — fills entire screen ---- */}
      <div className="flex-1">
        {hasArchive ? (
          <iframe
            src={blobServeUrl(
              daemonBaseUrlFromInfo(daemonInfo!),
              entry.archive_hash!,
            )}
            ref={iframeRef}
            sandbox="allow-scripts"
            className="h-full w-full border-0"
            title={entry.project_name}
            data-testid="remote-iframe-element"
          />
        ) : isLocal ? (
          <div className="mx-auto max-w-5xl p-8 pt-16">
            <LocalProjectApps coordUrl={coordUrl} />
          </div>
        ) : (
          <div className="flex h-full items-center justify-center" data-testid="remote-placeholder">
            <div className="glass-card max-w-sm p-8 text-center">
              <Globe className="mx-auto mb-4 h-10 w-10 text-white/15" />
              <h3 className="mb-2 font-bold text-white">Projet distant</h3>
              <p className="text-sm text-white/50">
                Aucune archive P2P disponible.
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// ================================================================
// Status dot
// ================================================================

function StatusDot({ status }: { status: string }) {
  const isReachable = status === "reachable";
  return (
    <span
      className={`inline-block h-2 w-2 rounded-full ${
        isReachable
          ? "bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.5)]"
          : "bg-white/20"
      }`}
      title={isReachable ? "En ligne" : "Hors ligne"}
    />
  );
}

// ================================================================
// Local SDK apps (legacy path for TabView apps)
// ================================================================

function LocalProjectApps({ coordUrl }: { coordUrl: string }) {
  const appsQuery = useQuery({
    queryKey: ["apps", coordUrl],
    queryFn: () => listApps(coordUrl),
    staleTime: 10_000,
  });

  if (appsQuery.isLoading) {
    return <p className="text-sm text-white/50">Chargement des applications...</p>;
  }
  if (appsQuery.isError) {
    return <p className="text-sm text-red-300">Erreur : {appsQuery.error.message}</p>;
  }

  const apps = appsQuery.data?.apps ?? [];
  if (apps.length === 0) {
    return (
      <div className="py-12 text-center" data-testid="no-apps">
        <Globe className="mx-auto mb-4 h-10 w-10 text-white/15" />
        <p className="text-sm text-white/50">
          Aucune application SDK installee.
        </p>
      </div>
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
    return <p className="text-sm text-white/50">Chargement du manifest...</p>;
  }
  if (manifestQuery.isError) {
    return <p className="text-sm text-red-300">Erreur : {manifestQuery.error.message}</p>;
  }

  const tabs = manifestQuery.data?.tabs ?? [];
  if (tabs.length === 0) {
    return <p className="text-sm text-white/50">Aucun onglet disponible.</p>;
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
    return <p className="text-sm text-white/50">Chargement...</p>;
  }
  if (descriptorQuery.isError) {
    return <p className="text-sm text-red-300">Erreur : {descriptorQuery.error.message}</p>;
  }

  const result = descriptorQuery.data;
  if (!result) return null;

  if (result.kind === "error") {
    return <p className="text-sm text-red-300">Erreur : {result.message}</p>;
  }

  return (
    <TabAppContext.Provider value={{ coordinatorUrl: coordUrl, appName }}>
      <TabViewRenderer tabView={result.tabView} />
    </TabAppContext.Provider>
  );
}

// ================================================================
// Helpers
// ================================================================

function truncateHex(hex: string): string {
  if (hex.length <= 16) return hex;
  return `${hex.slice(0, 8)}...${hex.slice(-8)}`;
}

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = BrowsedProject;
