// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Typed client for daemon core routes under `/api/daemon/*`
 * (info, curators, browse, panic wipe). Calls the daemon
 * directly (same-origin) without any proxy envelope.
 *
 * Returns a `DaemonResult<T>` discriminated union so the React
 * layer can render "daemon offline" as a normal UX state rather
 * than as an error boundary trip.
 */

import { z } from "zod";

import { authFetch } from "@/api/auth";
import {
  ApiProtocolError,
} from "@/api/coordinator";

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
 * Mirrors `nexus_shell_daemon_core::browse::BrowseSource`.
 *
 * Sprint 11 Phase A: entries can be discovered via a signed
 * curator list (`"curator"`) or directly via a gossip project
 * announcement (`"direct"`). Defaults to `"curator"` for
 * backward compat with daemons that predate Sprint 11.
 */
export const BrowseSourceSchema = z.enum(["curator", "direct"]);

export type BrowseSource = z.infer<typeof BrowseSourceSchema>;

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
    source: BrowseSourceSchema.optional(),
    status: BrowseStatusSchema,
    last_probed_at: z.string().nullable(),
    archive_ticket: z.string().optional(),
    archive_hash: z.string().optional(),
    repo_url: z.string().optional(),
    provenance_hash: z.string().optional(),
    /**
     * True iff the originating ProjectAnnouncement carried the
     * `is_open_source` flag (i.e. the project was deployed via
     * `deploy-from-repo` with a signed provenance chain). The
     * Rust daemon always serializes this field, but the Zod
     * `.optional()` guard is kept as runtime tolerance — a test
     * fixture or a future minimal-JSON client can skip it and
     * the parser stays forgiving rather than 422-erroring.
     */
    is_open_source: z.boolean().optional(),
    /**
     * KEEP-ONLINE-READ-PATH (Sprint 74 Phase G): true iff THIS node hosts the
     * app (the entry's hosting node_id == our node_id), derived daemon-side in
     * `list_browse`. The shell uses it to show the owner "Garder en ligne"
     * toggle for per-app deploys (project_id = blake3(name) != node_id, where
     * the old node_id===project_id heuristic was always false). `.optional()`
     * for runtime tolerance with daemons that predate the field.
     */
    is_own: z.boolean().optional(),
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
// Internal: call the daemon directly (no proxy envelope)
// =================================================================

async function callDaemon<T>(
  baseUrl: string,
  path: string,
  bodySchema: z.ZodType<T>,
  init?: RequestInit,
): Promise<DaemonResult<T>> {
  const url = `${baseUrl}${path}`;
  let res: Response;
  try {
    res = await authFetch(url, {
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

  if (res.status === 503) {
    return { kind: "unavailable", reason: "daemon unavailable" };
  }

  if (!res.ok) {
    return {
      kind: "error",
      reason: `HTTP ${res.status} ${res.statusText}`,
    };
  }

  let raw: unknown;
  try {
    raw = await res.json();
  } catch (e) {
    return {
      kind: "error",
      reason: `non-json response (status ${res.status}): ${
        e instanceof Error ? e.message : "parse error"
      }`,
    };
  }

  const parsed = bodySchema.safeParse(raw);
  if (!parsed.success) {
    throw new ApiProtocolError(path, parsed.error.issues, raw);
  }
  return {
    kind: "data",
    status: res.status,
    body: parsed.data,
  };
}

// =================================================================
// Public helpers — one per proxy route
// =================================================================

export function getDaemonInfo(baseUrl: string): Promise<DaemonResult<DaemonInfo>> {
  return callDaemon(baseUrl, "/api/daemon/info", DaemonInfoSchema);
}

export function listCurators(
  baseUrl: string,
): Promise<DaemonResult<DaemonCuratorsResponse>> {
  return callDaemon(baseUrl, "/api/daemon/curators", DaemonCuratorsResponseSchema);
}

export function subscribeCurator(
  baseUrl: string,
  curatorPubkeyHex: string,
): Promise<DaemonResult<SubscriptionsResponse>> {
  return callDaemon(baseUrl, "/api/daemon/curators/subscribe", SubscriptionsResponseSchema, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ curator_pubkey_hex: curatorPubkeyHex }),
  });
}

export function unsubscribeCurator(
  baseUrl: string,
  curatorPubkeyHex: string,
): Promise<DaemonResult<SubscriptionsResponse>> {
  return callDaemon(
    baseUrl,
    `/api/daemon/curators/${encodeURIComponent(curatorPubkeyHex)}`,
    SubscriptionsResponseSchema,
    { method: "DELETE" },
  );
}

export function listBrowse(
  baseUrl: string,
): Promise<DaemonResult<BrowseListResponse>> {
  return callDaemon(baseUrl, "/api/daemon/browse", BrowseListResponseSchema);
}

const BrowsePullResponseSchema = z.object({ requested: z.boolean() }).strict();

export function browsePull(
  baseUrl: string,
): Promise<DaemonResult<{ requested: boolean }>> {
  return callDaemon(baseUrl, "/api/daemon/browse/pull", BrowsePullResponseSchema, {
    method: "POST",
  });
}

// Sprint 74 Phase D — toggle a self-deployed app's local keep-online pin.
const KeepOnlineResponseSchema = z
  .object({ ok: z.boolean(), enabled: z.boolean() })
  .strict();

export function setKeepOnline(
  baseUrl: string,
  projectId: string,
  enabled: boolean,
): Promise<DaemonResult<{ ok: boolean; enabled: boolean }>> {
  return callDaemon(baseUrl, "/api/daemon/keep-online", KeepOnlineResponseSchema, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ project_id: projectId, enabled }),
  });
}

// Sprint 74 Phase E — VOLUNTARY community seed of a DISTANT public app. This
// node fetches + pins the app's archive (by the ticket already learned via
// gossip) and keeps it online to support the project. No author approval is
// needed: the content is already public and content-addressed (blake3), and
// the supporter never re-signs provenance (the author stays the author).
const SeedVoluntaryResponseSchema = z
  .object({ ok: z.boolean(), seeding: z.string() })
  .strict();

export function seedVoluntary(
  baseUrl: string,
  projectId: string,
): Promise<DaemonResult<{ ok: boolean; seeding: string }>> {
  return callDaemon(baseUrl, "/api/daemon/seed", SeedVoluntaryResponseSchema, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ project_id: projectId }),
  });
}

// Sprint 74 Phase F — best-effort multi-seed availability count for an app.
// `peer_count` is the number of distinct REMOTE seeders the daemon has seen
// re-announce within the TTL (its in-memory SeedRegistry); `self_seeding` is
// whether THIS node actively keeps the app online. The shell renders the pair
// as "Toi + N pairs (vus recemment)". Both keys are ALWAYS present (the Rust
// `seed_count` handler serialises them unconditionally), hence non-optional
// under the `.strict()` parse — matching the S73 Phase E always-present rule.
const SeedCountResponseSchema = z
  .object({ peer_count: z.number().int().min(0), self_seeding: z.boolean() })
  .strict();

export type SeedCountResponse = z.infer<typeof SeedCountResponseSchema>;

export function seedCount(
  baseUrl: string,
  projectId: string,
): Promise<DaemonResult<SeedCountResponse>> {
  return callDaemon(
    baseUrl,
    `/api/daemon/seed-count/${encodeURIComponent(projectId)}`,
    SeedCountResponseSchema,
  );
}

// =================================================================
// Search — FTS5 full-text index (Sprint 67 endpoint, Sprint 73 Phase E)
// =================================================================

/**
 * Mirrors `nexus_coordinator_rs::search::SearchResult`.
 *
 * The seven base columns plus the Sprint 73 Phase D provenance
 * triplet. The Rust `search_handler` (`nexus-shell-daemon/src/http.rs`)
 * serialises every key unconditionally — the four provenance fields
 * come through as JSON `null` (NOT absent) for non-release ops and
 * pre-M17 index rows. They are therefore modelled as `.nullable()`,
 * not `.optional()`: `callDaemon` runs the schema with `.strict()`,
 * which would reject a hit that simply omitted a key. `score` is the
 * raw bm25 rank (can be any finite number) and `is_open_source` is a
 * plain `bool`, always present.
 */
export const SearchResultSchema = z
  .object({
    project_id: z.string(),
    project_name: z.string(),
    category: z.string(),
    description: z.string(),
    op_type: z.string(),
    source_type: z.string(),
    score: z.number(),
    repo_url: z.string().nullable(),
    commit_sha: z.string().nullable(),
    archive_hash: z.string().nullable(),
    provenance_hash: z.string().nullable(),
    is_open_source: z.boolean(),
  })
  .strict();

export type SearchResult = z.infer<typeof SearchResultSchema>;

/**
 * Mirrors the `search_handler` envelope `{ results, total, took_ms }`.
 * `total` is the full match count before `limit`/`offset` paging;
 * `took_ms` is the server-side query duration in milliseconds.
 */
export const SearchResponseSchema = z
  .object({
    results: z.array(SearchResultSchema),
    total: z.number().int().min(0),
    took_ms: z.number().int().min(0),
  })
  .strict();

export type SearchResponse = z.infer<typeof SearchResponseSchema>;

/**
 * Sprint 73 Phase E (D4) — full-text search over the daemon's FTS5
 * index of browse/feed entries. Mirrors {@link listBrowse}: routes
 * through `callDaemon` for the loopback bearer + the
 * `DaemonResult<T>` offline/error union the shell renders as a normal
 * UX state. The query parameters are built with `URLSearchParams` so
 * a pathological `q` is percent-encoded and cannot break out of the
 * query string (the daemon decodes it via `serde_urlencoded`).
 */
export function searchBrowse(
  baseUrl: string,
  q: string,
  limit = 20,
  offset = 0,
): Promise<DaemonResult<SearchResponse>> {
  const params = new URLSearchParams({
    q,
    limit: String(limit),
    offset: String(offset),
  });
  return callDaemon(
    baseUrl,
    `/api/daemon/search?${params.toString()}`,
    SearchResponseSchema,
  );
}

/**
 * Mirrors the Rust handler's `{ "wiped": true }` success envelope.
 */
export const PanicWipeResponseSchema = z
  .object({ wiped: z.literal(true) })
  .strict();
export type PanicWipeResponse = z.infer<typeof PanicWipeResponseSchema>;

/**
 * Sprint 20 Phase B — trigger the irreversible panic wipe on the
 * daemon. Invoked by the `PanicWipeKeybind` component after a
 * confirmed 5-tap gesture.
 *
 * This call is **destructive** — the daemon zeroes its identity
 * blobs + OS keyring entries + on-disk state before exiting the
 * process. The returned envelope only arrives if the handler's
 * `tokio::spawn` sleep delays the exit enough for axum to flush
 * the response; callers MUST assume the daemon is gone after
 * the call returns regardless of envelope shape.
 */
export function triggerPanicWipe(
  baseUrl: string,
): Promise<DaemonResult<PanicWipeResponse>> {
  return callDaemon(baseUrl, "/api/daemon/panic/wipe", PanicWipeResponseSchema, {
    method: "POST",
  });
}

// =================================================================
// Deploy
// =================================================================

export const DeployResponseSchema = z
  .object({
    deployed: z.boolean(),
    hash: z.string(),
    provenance_hash: z.string().optional(),
    commit_sha: z.string().optional(),
  })
  .strict();

export type DeployResponse = z.infer<typeof DeployResponseSchema>;

export interface DeployFromRepoRequest {
  repo_url: string;
  project_name: string;
  description?: string;
  category?: string;
  commit_sha?: string;
}

export function deployFromRepo(
  baseUrl: string,
  req: DeployFromRepoRequest,
): Promise<DaemonResult<DeployResponse>> {
  return callDaemon(baseUrl, "/api/v1/deploy-from-repo", DeployResponseSchema, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(req),
  });
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

/**
 * Build the daemon blob-serve URL for a file inside an archived
 * web app. The daemon serves these at a separate origin (port 7000)
 * so that `sandbox="allow-scripts"` without `allow-same-origin`
 * gives the iframe an opaque origin.
 *
 * Sprint 12 Phase C.
 */
export function blobServeUrl(
  daemonBaseUrl: string,
  hash: string,
  path: string = "index.html",
): string {
  return `${daemonBaseUrl}/blob-serve/${hash}/${path}`;
}

/**
 * Extract the daemon's base URL from a `DaemonInfo` payload.
 *
 * Sprint 12 Phase C.
 */
export function daemonBaseUrlFromInfo(info: DaemonInfo): string {
  return `http://${info.api_host}:${info.api_port}`;
}

