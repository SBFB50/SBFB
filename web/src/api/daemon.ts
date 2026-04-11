/**
 * Typed client for the Sprint 7 Phase E coordinator
 * `/daemon/*` proxy. Every call in this module lives on top of
 * the shared `getJson` / `postJson` / `deleteJson` helpers from
 * `./coordinator.ts` and returns a **discriminated union** so
 * the React layer can render "daemon offline" as a normal UX
 * state rather than as an error boundary trip.
 *
 * Sprint 7 D1 freezes the path: the shell never talks to the
 * `nexus-shell-daemon` binary directly. Every call goes
 * `shell → coordinator → daemon`, and the coordinator proxy
 * wraps the upstream response in one of:
 *
 *   - `{ kind: "data",        status, body }` — the daemon
 *     answered (any HTTP status, not necessarily 2xx)
 *   - `{ kind: "unavailable", reason }`      — daemon offline
 *     or proxy transport failure
 *   - `{ kind: "error",       reason }`      — proxy-level
 *     400 (e.g. malformed request body caught before forward)
 *
 * This module mirrors that envelope in Zod and exposes helper
 * `*Result` unions so callers see the three outcomes as a
 * closed set. No `as any`, no raw fetches, no thrown errors
 * for "daemon not running" — that's a first-class return
 * value.
 */

import { z } from "zod";

import {
  CoordinatorHttpError,
  CoordinatorProtocolError,
} from "@/api/coordinator";

// =================================================================
// Proxy envelope
// =================================================================

/**
 * Shape-only validation for the proxy envelope: we check the
 * `kind` / `status` / shape-of-body, then hand the body off to
 * the caller's Zod schema separately. This split avoids a
 * generic-inference headache where `z.object({ body: bodySchema })`
 * spreads an `addQuestionMarks<...>` type back over the caller's
 * `T` — Zod's input/output asymmetry leaks through the generic
 * boundary otherwise.
 */
const ProxyDataEnvelopeRaw = z.object({
  kind: z.literal("data"),
  status: z.number().int(),
  body: z.unknown(),
});

const ProxyUnavailableEnvelope = z.object({
  kind: z.literal("unavailable"),
  reason: z.string(),
});

const ProxyErrorEnvelope = z.object({
  kind: z.literal("error"),
  reason: z.string(),
});

// =================================================================
// Payload schemas — mirror the Rust daemon's wire types
// =================================================================

/**
 * Mirrors `nexus_shell_daemon_core::state::DaemonStateSnapshot`
 * (schema_version 1). The Rust side always serializes the
 * curator / browse count fields (they're `Vec<_>` / `u32`,
 * never Option), so the schema mirrors them as required — no
 * `.default()` calls, which would asymmetrize the Zod
 * input/output and trip the generic `callProxy` helper below.
 */
export const DaemonInfoSchema = z
  .object({
    schema_version: z.literal(1),
    node_id: z.string().min(1),
    daemon_version: z.string(),
    uptime_secs: z.number().int().min(0),
    started_at: z.string(),
    last_updated_at: z.string(),
    api_host: z.string(),
    api_port: z.number().int().min(1).max(65535),
    subscribed_curators: z.array(z.string()),
    known_lists: z.number().int().min(0),
    known_browse_entries: z.number().int().min(0),
  })
  .strict();

export type DaemonInfo = z.infer<typeof DaemonInfoSchema>;

/**
 * Mirrors `nexus_core_rs::curator::CuratorProjectRef`.
 *
 * Sprint 8 audit A-4: per-field length caps mirror the Rust
 * `CURATOR_*_MAX` constants so a curator pushing a pathological
 * entry is rejected by the shell layer even before it hits the
 * renderer. The values match the Rust side byte-for-byte:
 * project_id ≤ 128, project_name ≤ 128, category ≤ 64,
 * description ≤ 280.
 */
export const CuratorProjectRefSchema = z
  .object({
    project_id: z.string().min(1).max(128),
    project_name: z.string().max(128),
    category: z.string().max(64),
    description: z.string().max(280),
  })
  .strict();

/**
 * Mirrors `nexus_core_rs::curator::CuratorList`.
 *
 * `curator_pubkey` + `signature` arrive as JSON arrays of bytes
 * (Rust `[u8; 32]` / `[u8; 64]` serialized by serde) — we
 * validate the lengths but keep them as numeric arrays so the
 * shell can re-serialize them verbatim if needed.
 */
export const CuratorListSchema = z
  .object({
    version: z.literal(1),
    curator_pubkey: z.array(z.number().int().min(0).max(255)).length(32),
    curator_name: z.string(),
    created_at: z.number().int().min(0),
    revision: z.number().int().min(0),
    entries: z.array(CuratorProjectRefSchema).max(256),
  })
  .strict();

/**
 * Mirrors `nexus_core_rs::curator::CuratorListEntry`.
 */
export const CuratorListEntrySchema = z
  .object({
    list: CuratorListSchema,
    curator_pubkey: z.array(z.number().int().min(0).max(255)).length(32),
    signature: z.array(z.number().int().min(0).max(255)).length(64),
  })
  .strict();

export type CuratorListEntry = z.infer<typeof CuratorListEntrySchema>;

/**
 * Mirrors the daemon's `GET /curators` response
 * (`CuratorsListResponse` in `nexus-shell-daemon/src/http.rs`).
 */
export const DaemonCuratorsResponseSchema = z
  .object({
    entries: z.array(CuratorListEntrySchema),
    subscribed_curators: z.array(z.string()),
  })
  .strict();

export type DaemonCuratorsResponse = z.infer<typeof DaemonCuratorsResponseSchema>;

/**
 * Mirrors `POST /curators/subscribe` + `DELETE /curators/{pubkey}`
 * success responses.
 */
export const SubscriptionsResponseSchema = z
  .object({
    subscribed_curators: z.array(z.string()),
  })
  .strict();

export type SubscriptionsResponse = z.infer<typeof SubscriptionsResponseSchema>;

/**
 * Mirrors `nexus_shell_daemon_core::browse::BrowseStatus`.
 */
export const BrowseStatusSchema = z.enum(["reachable", "unreachable", "unknown"]);

export type BrowseStatus = z.infer<typeof BrowseStatusSchema>;

/**
 * Mirrors `nexus_shell_daemon_core::browse::BrowseEntry`.
 */
export const BrowseEntrySchema = z
  .object({
    project_id: z.string(),
    project_name: z.string(),
    category: z.string(),
    description: z.string(),
    curator_pubkey: z.string(),
    curator_name: z.string(),
    status: BrowseStatusSchema,
    last_probed_at: z.string().nullable(),
  })
  .strict();

export type BrowseEntry = z.infer<typeof BrowseEntrySchema>;

export const BrowseListResponseSchema = z
  .object({
    entries: z.array(BrowseEntrySchema),
  })
  .strict();

export type BrowseListResponse = z.infer<typeof BrowseListResponseSchema>;

// =================================================================
// Result unions — what callers actually receive
// =================================================================

/**
 * Generic discriminated union every daemon-proxy helper returns.
 * Mirrors the Python proxy's envelope plus the transport error
 * cases the shell must also render as "daemon offline".
 */
export type DaemonResult<T> =
  | { kind: "data"; status: number; body: T }
  | { kind: "unavailable"; reason: string }
  | { kind: "error"; reason: string };

// =================================================================
// Internal: call the proxy + unwrap the envelope + Zod the body
// =================================================================

async function callProxy<T>(
  baseUrl: string,
  path: string,
  bodySchema: z.ZodType<T>,
  init?: RequestInit,
): Promise<DaemonResult<T>> {
  const url = `${baseUrl}${path}`;
  let res: Response;
  try {
    res = await fetch(url, {
      ...init,
      headers: {
        accept: "application/json",
        ...(init?.headers ?? {}),
      },
    });
  } catch (e) {
    return {
      kind: "unavailable",
      reason: e instanceof Error ? e.message : "network error",
    };
  }

  let raw: unknown;
  try {
    raw = await res.json();
  } catch (e) {
    return {
      kind: "error",
      reason: `non-json body from proxy (status ${res.status}): ${
        e instanceof Error ? e.message : "parse error"
      }`,
    };
  }

  // 503 → always `unavailable`
  if (res.status === 503) {
    const parsed = ProxyUnavailableEnvelope.safeParse(raw);
    if (parsed.success) {
      return { kind: "unavailable", reason: parsed.data.reason };
    }
    return {
      kind: "unavailable",
      reason: "daemon unavailable (and envelope unreadable)",
    };
  }

  // 400 from the proxy itself → `error`
  if (res.status === 400) {
    const parsed = ProxyErrorEnvelope.safeParse(raw);
    if (parsed.success) {
      return { kind: "error", reason: parsed.data.reason };
    }
    return {
      kind: "error",
      reason: `proxy 400 (and envelope unreadable)`,
    };
  }

  // Everything else must be the `data` envelope.
  if (!res.ok) {
    throw new CoordinatorHttpError(path, res.status, res.statusText);
  }
  const envelopeParsed = ProxyDataEnvelopeRaw.safeParse(raw);
  if (!envelopeParsed.success) {
    throw new CoordinatorProtocolError(path, envelopeParsed.error.issues, raw);
  }
  const bodyParsed = bodySchema.safeParse(envelopeParsed.data.body);
  if (!bodyParsed.success) {
    throw new CoordinatorProtocolError(path, bodyParsed.error.issues, raw);
  }
  return {
    kind: "data",
    status: envelopeParsed.data.status,
    body: bodyParsed.data,
  };
}

// =================================================================
// Public helpers — one per proxy route
// =================================================================

export function getDaemonInfo(baseUrl: string): Promise<DaemonResult<DaemonInfo>> {
  return callProxy(baseUrl, "/daemon/info", DaemonInfoSchema);
}

export function listCurators(
  baseUrl: string,
): Promise<DaemonResult<DaemonCuratorsResponse>> {
  return callProxy(baseUrl, "/daemon/curators", DaemonCuratorsResponseSchema);
}

export function subscribeCurator(
  baseUrl: string,
  curatorPubkeyHex: string,
): Promise<DaemonResult<SubscriptionsResponse>> {
  return callProxy(baseUrl, "/daemon/curators/subscribe", SubscriptionsResponseSchema, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ curator_pubkey_hex: curatorPubkeyHex }),
  });
}

export function unsubscribeCurator(
  baseUrl: string,
  curatorPubkeyHex: string,
): Promise<DaemonResult<SubscriptionsResponse>> {
  return callProxy(
    baseUrl,
    `/daemon/curators/${encodeURIComponent(curatorPubkeyHex)}`,
    SubscriptionsResponseSchema,
    { method: "DELETE" },
  );
}

export function listBrowse(
  baseUrl: string,
): Promise<DaemonResult<BrowseListResponse>> {
  return callProxy(baseUrl, "/daemon/browse", BrowseListResponseSchema);
}

// =================================================================
// Helpers
// =================================================================

/**
 * Lowercase-hex sanity check. Used by the Curators page form
 * before we even hit the network — catches a whole class of
 * user paste errors locally so the shell can show an inline
 * validation hint instead of a daemon error toast.
 */
export function isValidCuratorPubkey(candidate: string): boolean {
  return /^[0-9a-f]{64}$/.test(candidate);
}
