// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/browse/:projectId` — Full-screen immersive app viewer.
 *
 * The app fills the entire viewport. A glassmorphism top bar
 * auto-hides and reveals when the mouse approaches the top edge.
 */

import {
  useCallback,
  useEffect,
  useMemo,
  useReducer,
  useRef,
  useState,
} from "react";
import { Link, useParams } from "react-router-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useBridge } from "@/bridge/useBridge";
import {
  AlertTriangle,
  ArrowLeft,
  ExternalLink,
  GitFork,
  Globe,
  Heart,
  HeartOff,
  Loader2,
  Rocket,
  Shield,
  Signal,
  SignalZero,
  FileCheck,
  X,
} from "lucide-react";
import { VerificationDetail } from "@/components/VerificationDetail";
import { AvailabilitySheet } from "@/components/AvailabilitySheet";
import { ProofCard, type ProofCardData } from "@/components/ProofCard";

import {
  addToWhitelist,
  getConsent,
  removeFromWhitelist,
} from "@/api/consent";
import { authFetch } from "@/api/auth";

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
            Aucun noeud actif
          </h2>
          <p className="text-sm text-white/50">
            Connecte-toi a un noeud depuis l'en-tete.
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
  // KEEP-ONLINE-READ-PATH (carry S74): prefer the daemon-derived `is_own`
  // (entry's hosting node_id == our node_id) — accurate for per-app deploys
  // whose project_id = blake3(name) != node_id. Fall back to the old
  // node_id===projectId heuristic only when the daemon predates the field.
  const isLocal =
    entry?.is_own ??
    (daemonInfo !== null && daemonInfo.node_id === projectId);

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
  const [verifyOpen, setVerifyOpen] = useState(false);
  const [availabilityOpen, setAvailabilityOpen] = useState(false);
  // Sprint 74 Phase C — "Forker dans l'atelier" intention panel. The fork itself
  // runs in the local atelier tool (a separate privileged process), so the shell
  // surfaces the INTENTION + the steps rather than faking a one-click action it
  // cannot perform (verrou "0 faux bouton actif").
  const [forkOpen, setForkOpen] = useState(false);
  // Forkable when there is a verifiable forge source OR a published archive the
  // atelier can reconstruct from — matches the backend fork_from_search_hit
  // (forge clone OR archive reconstruction). The panel is an explainer, so a
  // broad-but-honest gate is right.
  const canFork = isHttpsUrl(entry.repo_url) || !!entry.archive_hash;
  const hideTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const iframeRef = useRef<HTMLIFrameElement | null>(null);

  // Sprint 74 Phase A (D-DISPO) — availability surfacing.
  // Ownership is the local-node check in Phase A (precise signal lands Phase D).
  const isOwn = isLocal;
  const isOffline = entry.status === "unreachable";
  // Greffe A — the offline reminder is state-triggered (not pushed at publish),
  // shown only for the user's OWN apps, and dismissible 1x/session/app.
  const reminderKey = `sbfb:offline-reminder-dismissed:${entry.project_id}`;
  // Derive dismissal from sessionStorage DURING render (a cheap, side-effect-free
  // read) instead of mirroring it into state via an effect. The route is
  // `lazy()` without a `key`, so this component is NOT remounted on a
  // `/browse/:projectId` change — reading per render keeps the dismissal per-app
  // (a project change re-reads the new key) with no setState-in-effect. The
  // dismiss action writes sessionStorage then forces a re-read.
  const [, forceReminderRecheck] = useReducer((n: number) => n + 1, 0);
  let reminderDismissed = false;
  try {
    reminderDismissed = sessionStorage.getItem(reminderKey) === "1";
  } catch {
    /* sessionStorage can be unavailable (private mode / SSR) — treat as not dismissed. */
  }
  const dismissReminder = () => {
    try {
      sessionStorage.setItem(reminderKey, "1");
    } catch {
      /* sessionStorage can be unavailable (private mode / SSR) — ignore. */
    }
    forceReminderRecheck();
  };
  const showOfflineReminder = isOwn && isOffline && !reminderDismissed;

  const proofCardQuery = useQuery({
    queryKey: ["proof-card", coordUrl, entry.project_id],
    queryFn: async () => {
      const resp = await authFetch(
        `${coordUrl}/api/daemon/proof-card/${encodeURIComponent(entry.project_id)}`,
      );
      if (resp.status === 404) return null;
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      return (await resp.json()) as ProofCardData;
    },
    staleTime: 60_000,
    retry: 1,
  });

  const verifyQuery = useQuery({
    queryKey: ["provenance-verify", coordUrl, entry.project_id],
    queryFn: async () => {
      const resp = await authFetch(
        `${coordUrl}/api/v1/project/${encodeURIComponent(entry.project_id)}/provenance`,
      );
      if (resp.status === 404) return { verified: false, status: "absent" as const };
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const data = (await resp.json()) as {
        verified: boolean;
        provenance_hash: string;
        status: string;
      };
      return {
        verified: data.verified,
        status: data.status as "verified" | "failed" | "absent",
      };
    },
    enabled: !!entry.provenance_hash,
    staleTime: 5 * 60_000,
    retry: 1,
  });

  // Sprint 13 Phase C: bridge listener for iframe ↔ coordinator.
  // Sprint 15 Phase B: also exposes the CPU watchdog state.
  const { watchdogState, resetWatchdog } = useBridge(
    coordUrl,
    entry.project_name,
    iframeRef,
  );

  // Sprint 15 Phase B: reload a stalled iframe by resetting its src.
  // Going through about:blank first avoids browser caches and forces
  // the internal document to be torn down before the new load.
  const reloadIframe = useCallback(() => {
    const frame = iframeRef.current;
    if (!frame || !daemonInfo || !entry.archive_hash) return;
    const url = blobServeUrl(daemonBaseUrlFromInfo(daemonInfo), entry.archive_hash);
    frame.src = "about:blank";
    window.setTimeout(() => {
      if (iframeRef.current) iframeRef.current.src = url;
      resetWatchdog();
    }, 50);
  }, [daemonInfo, entry.archive_hash, resetWatchdog]);

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
                Upload direct
              </span>
            )}
          </div>

          {/* Right side */}
          <div className="flex items-center gap-2">
            {/* Sprint 74 Phase A — "Disponibilite" opens the availability panel
                (replaces the raw blob:<hash> badge). The hash is now folded
                inside the panel's "Details" rather than shown as jargon. The
                button is TRI-state so a never-probed (unknown) app is not
                mis-styled as online (it shares `mapAvailabilityCopy` with the
                panel so the two surfaces cannot drift). */}
            <button
              type="button"
              onClick={() => setAvailabilityOpen(true)}
              className={`flex items-center gap-1 rounded-full px-3 py-1.5 text-[11px] font-medium transition-colors ${
                entry.status === "reachable"
                  ? "bg-emerald-500/10 text-emerald-300 hover:bg-emerald-500/20"
                  : "bg-white/[0.08] text-white/50 hover:bg-white/[0.12]"
              }`}
              data-testid="availability-button"
              aria-label={`Disponibilite — ${availabilityShortLabel(entry.status)}`}
              title={`Disponibilite — ${availabilityShortLabel(entry.status)}`}
            >
              {entry.status === "reachable" ? (
                <Signal className="h-3 w-3" />
              ) : entry.status === "unreachable" ? (
                <SignalZero className="h-3 w-3" />
              ) : (
                <Loader2 className="h-3 w-3 animate-spin" />
              )}
              Disponibilite
            </button>

            {entry.provenance_hash && (
              <button
                type="button"
                className={`flex items-center gap-1 rounded-full px-3 py-1.5 text-[11px] font-medium transition-colors ${
                  verifyQuery.isLoading
                    ? "bg-white/[0.08] text-white/50"
                    : verifyQuery.isSuccess && verifyQuery.data.status === "verified"
                      ? "bg-emerald-500/15 text-emerald-400 hover:bg-emerald-500/25"
                      : verifyQuery.isSuccess && verifyQuery.data.status === "failed"
                        ? "bg-red-500/15 text-red-400 hover:bg-red-500/25"
                        : verifyQuery.isError
                          ? "bg-red-500/15 text-red-400 hover:bg-red-500/25"
                          : "bg-white/[0.08] text-white/50"
                }`}
                data-testid="verified-badge"
                onClick={() => setVerifyOpen(true)}
                title="Provenance auto-attestee (SLSA L1)"
              >
                {verifyQuery.isLoading ? (
                  <>
                    <Loader2 className="h-3 w-3 animate-spin" />
                    Verification...
                  </>
                ) : verifyQuery.isSuccess && verifyQuery.data.status === "verified" ? (
                  <>
                    <FileCheck className="h-3 w-3" />
                    Signature verifiee
                  </>
                ) : verifyQuery.isSuccess && verifyQuery.data.status === "failed" ? (
                  <>
                    <AlertTriangle className="h-3 w-3" />
                    Verification echouee
                  </>
                ) : verifyQuery.isError ? (
                  <>
                    <AlertTriangle className="h-3 w-3" />
                    Verification echouee
                  </>
                ) : (
                  <>
                    <FileCheck className="h-3 w-3" />
                    Provenance
                  </>
                )}
              </button>
            )}

            <ProofCard
              card={proofCardQuery.data ?? null}
              loading={proofCardQuery.isLoading}
            />

            {/* Scheme guard (carry B.5): React does not sanitize an anchor href,
                so only render the Source link for an explicit https origin —
                never a javascript:/data: feed-sourced repo_url. The remaining two
                anchors (Browse, VerificationDetail) are normalised in Phase G. */}
            {isHttpsUrl(entry.repo_url) && (
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

            {canFork && (
              <button
                type="button"
                onClick={() => setForkOpen(true)}
                className="flex items-center gap-1 rounded-full bg-white/[0.06] px-3 py-1.5 text-[11px] text-white/60 transition-colors hover:bg-white/10 hover:text-white"
                data-testid="fork-atelier-cta"
                title="Forker cette app dans ton atelier pour publier ta propre version"
              >
                <GitFork className="h-3 w-3" />
                Forker dans l'atelier
              </button>
            )}

            <ContributeGpuButton
              coordUrl={coordUrl}
              projectId={entry.project_id}
            />

            <span className="flex items-center gap-1 text-[11px] text-white/40">
              <Shield className="h-3 w-3" />
              sandbox
            </span>
          </div>
        </div>
      </div>

      {/* ---- Content — fills entire screen ---- */}
      <div className="relative flex-1">
        {/* Sprint 74 Phase A — greffe A : offline reminder, state-triggered,
            own apps only, dismissible 1x/session/app. */}
        {showOfflineReminder && (
          <div
            className="absolute left-1/2 top-16 z-40 w-[min(92%,32rem)] -translate-x-1/2"
            data-testid="offline-reminder"
          >
            <div className="flex items-start gap-3 rounded-lg border border-amber-500/25 bg-amber-950/70 p-3 shadow-lg backdrop-blur-md">
              <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-amber-400" />
              <p className="flex-1 text-xs leading-relaxed text-amber-100/90">
                Cette app est hors ligne : ce noeud est ferme. Elle redeviendra
                joignable au prochain demarrage. Pour la garder en ligne meme PC
                eteint, ajoute une copie de secours.
              </p>
              <button
                type="button"
                onClick={dismissReminder}
                aria-label="Masquer le rappel"
                data-testid="offline-reminder-dismiss"
                className="shrink-0 rounded p-0.5 text-amber-200/60 hover:bg-white/10 hover:text-amber-100"
              >
                <X className="h-3.5 w-3.5" />
              </button>
            </div>
          </div>
        )}

        {/* Sprint 74 Phase C — "Forker dans l'atelier" intention panel. Plain
            language, no jargon, no fake action: it explains the atelier-fork
            flow (the fork + redeploy run in the local atelier tool). */}
        {forkOpen && (
          <div
            className="absolute inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md"
            data-testid="fork-atelier-panel"
          >
            <div className="glass-card max-w-md p-8 text-left">
              <div className="mb-3 flex items-center gap-2">
                <GitFork className="h-5 w-5 text-purple-300" />
                <h3 className="text-lg font-bold text-white">
                  Forker dans l'atelier
                </h3>
              </div>
              <p className="mb-4 text-sm leading-relaxed text-white/70">
                Recupere le code de cette app dans ton atelier pour la modifier
                et publier ta propre version. Ta version sera signee par ton
                noeud — l'app d'origine reste celle de son auteur.
              </p>
              <ol className="mb-6 list-decimal space-y-1.5 pl-5 text-sm text-white/60">
                <li>Ouvre l'atelier sur ton ordinateur.</li>
                <li>Recupere cette app (son code source).</li>
                <li>Modifie-la a ta facon.</li>
                <li>Publie ta version sous ton identite.</li>
              </ol>
              <div className="flex justify-end">
                <button
                  type="button"
                  onClick={() => setForkOpen(false)}
                  className="rounded-full bg-white/[0.08] px-4 py-1.5 text-xs text-white hover:bg-white/[0.15]"
                  data-testid="fork-atelier-close"
                >
                  Compris
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Sprint 15 Phase B: stalled overlay. Absent when healthy
            or unknown so the first few seconds of an app's life
            stay clean. */}
        {hasArchive && watchdogState === "stalled" && (
          <div
            className="absolute inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md"
            data-testid="watchdog-overlay"
          >
            <div className="glass-card max-w-sm p-8 text-center">
              <AlertTriangle className="mx-auto mb-4 h-10 w-10 text-amber-400" />
              <h3 className="mb-2 text-lg font-bold text-white">
                Application ne repond plus
              </h3>
              <p className="mb-6 text-sm text-white/70">
                L'app n'a pas envoye de signal depuis plusieurs secondes.
                Son script est peut-etre bloque ou en boucle.
              </p>
              <div className="flex justify-center gap-3">
                <button
                  onClick={reloadIframe}
                  className="rounded-full bg-white/[0.08] px-4 py-1.5 text-xs text-white hover:bg-white/[0.15]"
                  data-testid="watchdog-reload"
                >
                  Recharger
                </button>
                <Link
                  to="/browse"
                  className="rounded-full bg-white/[0.04] px-4 py-1.5 text-xs text-white/70 hover:bg-white/[0.08] hover:text-white"
                  data-testid="watchdog-close"
                >
                  Fermer
                </Link>
              </div>
            </div>
          </div>
        )}

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
        ) : !entry.archive_hash && isHttpsUrl(entry.repo_url) ? (
          /* Sprint 74 Phase A — greffe D : app tombee. Nobody is currently
             serving the blob (no archive_hash), but the source is verifiable —
             offer a one-click "remettre en ligne" that prefills /deploy. Phase C
             turns this into a true fork→redeploy under the local node identity.
             Gated on `!archive_hash` so a transient daemon-info error on an
             ARCHIVED app does not mis-present a redeploy CTA (the archive exists;
             it just could not be served this render). */
          <div className="flex h-full items-center justify-center" data-testid="fallen-app">
            <div className="glass-card max-w-sm p-8 text-center">
              <SignalZero className="mx-auto mb-4 h-10 w-10 text-amber-400/70" />
              <h3 className="mb-2 font-bold text-white">
                Personne ne garde cette app en ligne en ce moment.
              </h3>
              <p className="mb-5 text-sm text-white/50">
                Tu as le code source — remets-la en ligne en un clic.
              </p>
              <Link
                to={`/deploy?repo_url=${encodeURIComponent(entry.repo_url)}&project_name=${encodeURIComponent(entry.project_name)}`}
                className="inline-flex items-center gap-2 rounded-lg bg-emerald-500/15 px-4 py-2 text-sm font-medium text-emerald-300 transition-colors hover:bg-emerald-500/25"
                data-testid="redeploy-fallen-app"
              >
                <Rocket className="h-4 w-4" />
                La remettre en ligne
              </Link>
            </div>
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

      <VerificationDetail
        open={verifyOpen}
        onOpenChange={setVerifyOpen}
        coordUrl={coordUrl}
        projectId={entry.project_id}
        provenanceHash={entry.provenance_hash ?? null}
      />

      <AvailabilitySheet
        open={availabilityOpen}
        onOpenChange={setAvailabilityOpen}
        entry={entry}
        isOwn={isOwn}
        coordUrl={coordUrl}
      />
    </div>
  );
}

// ================================================================
// Sprint 16 Phase C — "Contribuer mon GPU" button
// ================================================================
//
// Visible only when the user picked L3 (manual whitelist) in
// `GpuConsentDialog`. Toggle: adds the project to / removes the
// project from `consent.json::allowed_project_ids`. The worker's
// `notify` watcher applies the change before the next claim.

function ContributeGpuButton({
  coordUrl,
  projectId,
}: {
  coordUrl: string;
  projectId: string;
}) {
  const queryClient = useQueryClient();
  const consentQuery = useQuery({
    queryKey: ["consent", coordUrl],
    queryFn: () => getConsent(coordUrl),
    staleTime: 30_000,
    retry: 0,
  });

  const isHexNodeId = useMemo(
    () => /^[0-9a-fA-F]{64}$/.test(projectId),
    [projectId],
  );

  const mutation = useMutation({
    mutationFn: async (action: "add" | "remove") =>
      action === "add"
        ? addToWhitelist(coordUrl, projectId)
        : removeFromWhitelist(coordUrl, projectId),
    onSuccess: (cfg) => {
      queryClient.setQueryData(["consent", coordUrl], cfg);
    },
  });

  if (consentQuery.data?.level !== 3) return null;
  if (!isHexNodeId) return null;

  const inWhitelist =
    consentQuery.data?.allowed_project_ids.includes(projectId) ?? false;

  return (
    <button
      type="button"
      onClick={() => mutation.mutate(inWhitelist ? "remove" : "add")}
      disabled={mutation.isPending}
      className={`flex items-center gap-1 rounded-full px-3 py-1.5 text-[11px] transition-colors disabled:opacity-50 ${
        inWhitelist
          ? "bg-pink-500/15 text-pink-300 hover:bg-pink-500/25"
          : "bg-white/[0.06] text-white/60 hover:bg-white/10 hover:text-white"
      }`}
      data-testid="contribute-gpu"
      aria-pressed={inWhitelist}
      title={
        inWhitelist
          ? "Tu contribues actuellement ton GPU à ce projet — clique pour retirer."
          : "Ajoute ce projet à ta whitelist L3 pour partager ton GPU avec lui."
      }
    >
      {inWhitelist ? (
        <>
          <HeartOff className="h-3 w-3" />
          Contribution active
        </>
      ) : (
        <>
          <Heart className="h-3 w-3" />
          Contribuer mon GPU
        </>
      )}
    </button>
  );
}

// ================================================================
// Status dot
// ================================================================

function StatusDot({ status }: { status: string }) {
  const isReachable = status === "reachable";
  // Tri-state title: a never-probed (unknown) app must not assert "Hors ligne".
  const title =
    status === "reachable"
      ? "En ligne"
      : status === "unreachable"
        ? "Hors ligne"
        : "Verification…";
  return (
    <span
      className={`inline-block h-2 w-2 rounded-full ${
        isReachable
          ? "bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.5)]"
          : "bg-white/20"
      }`}
      title={title}
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

/** Short at-a-glance availability label, shared by the button + tooltip. */
function availabilityShortLabel(status: string): string {
  if (status === "reachable") return "en ligne";
  if (status === "unreachable") return "hors ligne";
  return "verification";
}

/**
 * Scheme guard for the feed-sourced `repo_url`. React does not sanitize an
 * anchor `href`, and the "remettre en ligne" prefill only makes sense for a
 * cloneable forge URL — so we only treat an explicit `https://` origin as a
 * redeployable source (mirrors `Browse.tsx::isHttpsUrl`; the 3 pre-existing
 * unguarded anchors are normalised in Phase G / carry B.5).
 */
function isHttpsUrl(url: string | null | undefined): url is string {
  return typeof url === "string" && url.startsWith("https://");
}

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = BrowsedProject;
