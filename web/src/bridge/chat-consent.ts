// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Chat v1 — user consent for the SENSITIVE bridge class (ADR_CHAT_V1).
 *
 * Stored by the HOST shell (never reachable from the sandboxed iframe, whose
 * origin is opaque), keyed by the app's exact archive hash: a new version of an
 * app is a new archive and asks again. Absent or unreadable storage = no consent
 * (fail-closed).
 */

const STORAGE_KEY = "sbfb.chat-consent.v1";

export type ChatConsent = "granted" | "denied";

function readAll(): Record<string, ChatConsent> {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed: unknown = JSON.parse(raw);
    if (parsed === null || typeof parsed !== "object") return {};
    const out: Record<string, ChatConsent> = {};
    for (const [k, v] of Object.entries(parsed as Record<string, unknown>)) {
      if (v === "granted" || v === "denied") out[k] = v;
    }
    return out;
  } catch {
    return {};
  }
}

/** The recorded decision for this archive, or `null` if never asked. */
export function getChatConsent(archiveHash: string): ChatConsent | null {
  return readAll()[archiveHash] ?? null;
}

export function setChatConsent(archiveHash: string, decision: ChatConsent): void {
  try {
    const all = readAll();
    all[archiveHash] = decision;
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(all));
  } catch {
    // Storage unavailable: the decision holds for this page only.
  }
}
