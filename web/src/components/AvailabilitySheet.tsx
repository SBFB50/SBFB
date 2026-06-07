// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 74 Phase A (D-DISPO) — "Disponibilite" side panel for an app fiche.
 *
 * Renders the three SBFB availability invariants as three SEALED-SEPARATELY
 * sections (design `s74_disponibilite_ux_design.md` §4/§5/§6):
 *
 *   1. AUTEUR    — immutable. Who signed the app. "Garder en ligne ne change
 *                  jamais l'auteur." (verrou anti-recentralisation §8(4)).
 *   2. ETAT      — the live reachability probe (`browse.rs` status), humanised
 *                  + a "Reverifier" action that re-runs `browsePull`.
 *   3. QUI LA GARDE EN LIGNE — mutable. The seeder set. For the user's own app
 *                  the "Garder en ligne" toggle is shown ON but DISABLED (honestly
 *                  non-interactive, "Bientôt configurable") in Phase A — never a
 *                  control that looks clickable yet silently no-ops (verrou §8(5));
 *                  the local pin lands Phase D. For a remote app the
 *                  voluntary "soutenir ce projet" action is presented inert
 *                  ("Bientot") until D+F wire it (amendement PO §13).
 *   4. COPIES DE SECOURS — additive redundancy, never substitutive. The
 *                  cross-node invite is inert ("Bientot") in Phase A (Phase E).
 *
 * Phase A is 100% front on existing S73 primitives. It introduces ZERO host
 * field (verrou §8(1)) and ZERO faux active button (verrou §8(5)): every
 * un-wired cross-node CTA is an inert "Bientot" element, never a button that
 * silently no-ops.
 */

import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  CheckCircle2,
  FileCheck,
  HeartHandshake,
  Loader2,
  RefreshCw,
  Signal,
  SignalZero,
  UserPlus,
} from "lucide-react";

import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Toggle } from "@/components/ui/toggle";
import { browsePull, setKeepOnline, type BrowseEntry } from "@/api/daemon";
import { formatRelativeTime } from "@/lib/format";

export interface AvailabilitySheetProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  entry: BrowseEntry;
  /**
   * True iff this node is the app's author/host. Phase A derives this from
   * the local-node check (`daemon.node_id === project_id`); deploy-from-repo
   * apps (`project_id = blake3(name)`) get a precise ownership signal in
   * Phase D via the `keep_online` table. We deliberately UNDER-claim rather
   * than risk a false authorship claim (verrou §8(4)).
   */
  isOwn: boolean;
  coordUrl: string;
}

type AvailabilityState = "online" | "offline" | "checking";

function mapStatus(status: BrowseEntry["status"]): AvailabilityState {
  if (status === "reachable") return "online";
  if (status === "unreachable") return "offline";
  return "checking";
}

export function AvailabilitySheet({
  open,
  onOpenChange,
  entry,
  isOwn,
  coordUrl,
}: AvailabilitySheetProps) {
  const queryClient = useQueryClient();
  const [reverifying, setReverifying] = useState(false);
  const state = mapStatus(entry.status);

  // Sprint 74 Phase D — functional "Garder en ligne" pin (replaces the Phase A
  // disabled-ON). No fetch on mount: a freshly deployed own app is ON by default;
  // the toggle POSTs on click. The daemon echoes the persisted state back.
  const [keepOnline, setKeepOnlineLocal] = useState(true);
  const keepOnlineMutation = useMutation({
    mutationFn: (next: boolean) => setKeepOnline(coordUrl, entry.project_id, next),
    onSuccess: (res) => {
      // Only reflect a state the daemon actually persisted. On error/unavailable
      // leave the toggle at its prior position — never show a state the daemon
      // did not echo (no failure-state lie).
      if (res.kind === "data") {
        setKeepOnlineLocal(res.body.enabled);
        void queryClient.invalidateQueries({
          queryKey: ["daemon-browse", coordUrl],
        });
      }
    },
  });

  const onReverify = async () => {
    setReverifying(true);
    try {
      await browsePull(coordUrl);
    } finally {
      // The daemon re-probes asynchronously; invalidate so the next render
      // picks up the refreshed `last_probed_at` / `status`.
      await queryClient.invalidateQueries({
        queryKey: ["daemon-browse", coordUrl],
      });
      setReverifying(false);
    }
  };

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent
        side="right"
        data-testid="availability-sheet"
        className="border-white/10 bg-[#0c0c12]/95 text-white backdrop-blur-xl"
      >
        <SheetHeader>
          <SheetTitle className="text-white">Disponibilite</SheetTitle>
        </SheetHeader>

        <div className="flex flex-col gap-5 overflow-y-auto px-4 pb-6 text-sm">
          {/* ---- AUTEUR (sealed, immutable) ---- */}
          <section data-testid="availability-section-author" className="space-y-2">
            <h3 className="text-[10px] font-semibold uppercase tracking-wider text-white/40">
              Auteur
            </h3>
            <div className="flex items-center gap-2 text-emerald-300">
              <CheckCircle2 className="h-4 w-4 shrink-0" />
              <span className="font-medium">
                {isOwn ? "Publiee par ton noeud" : "Publiee par un autre noeud"}
              </span>
            </div>
            {entry.provenance_hash && (
              <div className="flex items-center gap-1.5 text-xs text-emerald-400/80">
                <FileCheck className="h-3.5 w-3.5 shrink-0" />
                Signature verifiee
              </div>
            )}
            <p className="text-xs leading-relaxed text-white/40">
              L&apos;auteur est fige par la signature. Garder une app en ligne
              ne change jamais son auteur.
            </p>
          </section>

          <div className="h-px bg-white/5" />

          {/* ---- ETAT (live probe) ---- */}
          <section data-testid="availability-section-state" className="space-y-2">
            <h3 className="text-[10px] font-semibold uppercase tracking-wider text-white/40">
              Etat
            </h3>
            <div className="flex items-center gap-2" data-testid="availability-state">
              <StateDot state={state} />
              <span
                className={
                  state === "online"
                    ? "font-medium text-emerald-300"
                    : state === "offline"
                      ? "font-medium text-red-300"
                      : "font-medium text-white/60"
                }
              >
                {state === "online"
                  ? isOwn
                    ? "En ligne (vu de ton noeud)"
                    : "En ligne — joignable par tous"
                  : state === "offline"
                    ? "Hors ligne — relance ton noeud pour la rediffuser"
                    : "Verification…"}
              </span>
            </div>
            <div className="flex items-center gap-3 text-xs text-white/40">
              <span data-testid="availability-freshness">
                {entry.last_probed_at
                  ? `Verifie ${formatRelativeTime(entry.last_probed_at)}`
                  : "Pas encore verifie"}
              </span>
              <button
                type="button"
                onClick={onReverify}
                disabled={reverifying}
                data-testid="availability-reverify"
                className="inline-flex items-center gap-1 rounded-full bg-white/[0.06] px-2.5 py-1 text-[11px] text-white/60 transition-colors hover:bg-white/10 hover:text-white disabled:opacity-50"
              >
                {reverifying ? (
                  <Loader2 className="h-3 w-3 animate-spin" />
                ) : (
                  <RefreshCw className="h-3 w-3" />
                )}
                Reverifier
              </button>
            </div>
          </section>

          <div className="h-px bg-white/5" />

          {/* ---- QUI LA GARDE EN LIGNE (mutable) ---- */}
          <section data-testid="availability-section-seeders" className="space-y-3">
            <h3 className="text-[10px] font-semibold uppercase tracking-wider text-white/40">
              Qui la garde en ligne
            </h3>

            {isOwn ? (
              <div className="space-y-2">
                <div className="flex items-center justify-between gap-3">
                  <div className="flex items-center gap-2">
                    <Signal className="h-3.5 w-3.5 text-emerald-300" />
                    <span className="text-white/80">Ton noeud</span>
                    <span
                      className={
                        state === "online"
                          ? "text-[11px] text-emerald-300"
                          : "text-[11px] text-white/40"
                      }
                    >
                      {state === "online" ? "En ligne" : "Hors ligne"}
                    </span>
                  </div>
                  {/*
                    Sprint 74 Phase D — FUNCTIONAL toggle (POST /api/daemon/
                    keep-online). ON: blob pinned + diffused; OFF: tag removed +
                    no longer re-broadcast ("stockee, plus diffusee" — no disk is
                    freed today, no GC reaper yet). Disabled only WHILE the
                    request is in flight (never a silent no-op, verrou §8(5)).
                  */}
                  <div className="flex flex-col items-end gap-1">
                    <Toggle
                      pressed={keepOnline}
                      disabled={keepOnlineMutation.isPending}
                      onPressedChange={(next) => keepOnlineMutation.mutate(next)}
                      data-testid="keep-online-toggle"
                      aria-label="Garder en ligne"
                      className="border border-emerald-500/30 text-emerald-300 disabled:opacity-60 data-[state=on]:bg-emerald-500/15 data-[state=off]:bg-white/[0.04] data-[state=off]:text-white/50"
                    >
                      {keepOnlineMutation.isPending && (
                        <Loader2 className="mr-1 h-3 w-3 animate-spin" />
                      )}
                      Garder en ligne
                      <span className="ml-1 text-[10px] font-semibold">
                        {keepOnline ? "ON" : "OFF"}
                      </span>
                    </Toggle>
                    <span className="text-[10px] text-white/40">
                      {keepOnline ? "Diffusee tant que ton noeud tourne" : "Stockee, plus diffusee"}
                    </span>
                  </div>
                </div>
                <p className="text-xs leading-relaxed text-white/40">
                  {keepOnline
                    ? "Ton noeud diffuse l'app tant qu'il tourne."
                    : "L'app reste stockee mais ton noeud ne la diffuse plus."}
                </p>
              </div>
            ) : (
              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <Signal className="h-3.5 w-3.5 text-white/40" />
                  <span className="text-white/60">Un autre noeud</span>
                </div>
                {/*
                  Seed VOLONTAIRE communautaire (amendement PO §13): a node
                  consulting a public app may keep it online to support it,
                  with NO author approval (safe by blake3 content-addressing,
                  seeder != author). Inert "Bientot" in Phase A; functional
                  from D+F. NEVER a faux active button (verrou §8(5)).
                */}
                <div
                  data-testid="support-seed-cta"
                  className="flex items-center justify-between gap-2 rounded-lg border border-white/10 bg-white/[0.03] px-3 py-2 text-white/50"
                >
                  <span className="flex items-center gap-2">
                    <HeartHandshake className="h-4 w-4 shrink-0" />
                    Garder en ligne — soutenir ce projet
                  </span>
                  <span className="shrink-0 rounded-full bg-white/[0.06] px-2 py-0.5 text-[10px] font-medium text-white/40">
                    Bientôt
                  </span>
                </div>
              </div>
            )}
          </section>

          <div className="h-px bg-white/5" />

          {/* ---- COPIES DE SECOURS (additive redundancy) ---- */}
          <section data-testid="availability-section-backups" className="space-y-3">
            <h3 className="text-[10px] font-semibold uppercase tracking-wider text-white/40">
              Copies de secours
            </h3>
            <p className="text-xs leading-relaxed text-white/40">
              Aucune copie de secours. Si ton noeud s&apos;eteint, l&apos;app
              devient hors ligne.
            </p>
            <div
              data-testid="invite-peer-cta"
              className="flex items-center justify-between gap-2 rounded-lg border border-white/10 bg-white/[0.03] px-3 py-2 text-white/50"
            >
              <span className="flex items-center gap-2">
                <UserPlus className="h-4 w-4 shrink-0" />
                Inviter un pair de confiance
              </span>
              <span className="shrink-0 rounded-full bg-white/[0.06] px-2 py-0.5 text-[10px] font-medium text-white/40">
                Bientôt
              </span>
            </div>
          </section>
        </div>
      </SheetContent>
    </Sheet>
  );
}

function StateDot({ state }: { state: AvailabilityState }) {
  if (state === "online") {
    return (
      <span className="inline-block h-2.5 w-2.5 rounded-full bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.5)]" />
    );
  }
  if (state === "offline") {
    return (
      <SignalZero className="h-3.5 w-3.5 text-red-400" aria-hidden="true" />
    );
  }
  return <Loader2 className="h-3.5 w-3.5 animate-spin text-white/40" />;
}
