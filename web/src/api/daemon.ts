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
 * (`CuratorsListResponse` in `nexus-shell-daemon/src/curators_api.rs`).
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
 *
 * Sprint 75 Phase C: `"nodedirectory"` — an app discovered through a
 * subscribed node's signed directory (the PULL discovery substrate). Additive:
 * a daemon that predates Phase C never emits it, and the value is accepted
 * here so a mixed-version `/browse` response still parses.
 */
export const BrowseSourceSchema = z.enum(["curator", "direct", "nodedirectory"]);

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
    /**
     * UX-ARRIVAL (post-S75) : true ssi l'app est à nous OU si sa paire
     * (project_id, archive_hash) figure dans le catalogue Ed25519-VÉRIFIÉ de
     * l'annuaire signé du nœud qu'elle revendique (dérivé daemon-side dans
     * `list_browse` — CATALOG-BACKED, jamais la simple appartenance du
     * node_id réclamé à l'attention set : une annonce direct n'est pas
     * signée, son node_id est spoofable). Le shell s'en sert pour séparer la
     * grille (MES sources) de la section « Découvert sur le réseau » — le
     * flag n'est DÉCISIF que pour les entrées `direct` (les rows
     * `curator`/`nodedirectory` sont déjà subscription-gated à l'ingest et
     * classées par `source`). `.optional()` pour la tolérance runtime avec
     * un daemon antérieur au champ.
     */
    from_subscribed: z.boolean().optional(),
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
    // Sprint 75 Phase F (GAP Codex R2) : remonter la raison {"error": ...}
    // que le daemon Rust met dans le body — le statusText générique seul
    // cache la cause actionnable (ex. « bad key » d'un subscribe refusé).
    // Best-effort : un body non-JSON garde la raison générique.
    let detail = "";
    try {
      const raw = (await res.json()) as { error?: unknown };
      if (typeof raw?.error === "string" && raw.error.length > 0) {
        detail = ` — ${raw.error}`;
      }
    } catch {
      /* body non-JSON — raison générique conservée */
    }
    return {
      kind: "error",
      reason: `HTTP ${res.status} ${res.statusText}${detail}`,
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

/**
 * Sprint 75 Phase F (review-D deferral): `archiveHash` pins the EXACT version
 * the user was shown. Without it, two subscribed anchors listing the same
 * project id with different versions resolve first-match — the daemon could
 * pin bytes the user did not ask for. Callers that have a BrowseEntry or a
 * catalog row always pass its `archive_hash` (mirrors `seedCount`).
 */
export function seedVoluntary(
  baseUrl: string,
  projectId: string,
  archiveHash?: string | null,
): Promise<DaemonResult<{ ok: boolean; seeding: string }>> {
  return callDaemon(baseUrl, "/api/daemon/seed", SeedVoluntaryResponseSchema, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      project_id: projectId,
      ...(archiveHash ? { archive_hash: archiveHash } : {}),
    }),
  });
}

// Sprint 74 Phase F — best-effort multi-seed availability count for an app.
// `peer_count` is the number of distinct REMOTE seeders the daemon has seen
// re-announce within the TTL (its in-memory SeedRegistry); `self_seeding` is
// whether THIS node actively keeps the app online. The shell renders the pair
// as "Toi + N pairs (vus recemment)". Both keys are ALWAYS present (the Rust
// `seed_count` handler serialises them unconditionally), hence non-optional
// under the `.strict()` parse — matching the S73 Phase E always-present rule.
// Sprint 75 Phase F (WEB-1): `self_pin_enabled` is the operator's PERSISTED
// keep-online intent, three-valued — `null` = never toggled (no keep_online
// row; the app still rebroadcasts by default), `true`/`false` = the explicit
// toggle state. The Rust handler serialises the key unconditionally (an
// Option maps to JSON null, never an absent key) → `.nullable()`, NOT
// `.optional()` (S73-E rule). The shell's "Garder en ligne" toggle reconciles
// from THIS field; `self_seeding` stays the version-scoped serving truth and
// must not drive the toggle (a fresh never-toggled own app would render a
// false OFF).
const SeedCountResponseSchema = z
  .object({
    peer_count: z.number().int().min(0),
    self_seeding: z.boolean(),
    self_pin_enabled: z.boolean().nullable(),
  })
  .strict();

export type SeedCountResponse = z.infer<typeof SeedCountResponseSchema>;

// Sprint 75 Phase C (WIRE-2): an optional `archiveHash` scopes the count to the
// seeders of that EXACT version (the bytes the caller is about to pull), and
// makes `self_seeding` honest about that version. Omitting it preserves the
// pre-WIRE-2 version-agnostic behaviour. Callers that have a BrowseEntry pass its
// `archive_hash` so the availability count is not silently version-agnostic.
export function seedCount(
  baseUrl: string,
  projectId: string,
  archiveHash?: string | null,
): Promise<DaemonResult<SeedCountResponse>> {
  const query = archiveHash
    ? `?archive_hash=${encodeURIComponent(archiveHash)}`
    : "";
  return callDaemon(
    baseUrl,
    `/api/daemon/seed-count/${encodeURIComponent(projectId)}${query}`,
    SeedCountResponseSchema,
  );
}

// =================================================================
// Node directories — PULL discovery (Sprint 75 Phase F)
// =================================================================

/**
 * Mirrors `nexus_core_rs::CatalogApp` — one app a subscribed node advertises
 * in its signed directory.
 *
 * Deliberately NOT `.strict()` (review-D deferral): the catalog rows travel
 * inside the signed directory and the pre-launch policy adds fields
 * additively with 0 bump — a `.strict()` row schema would 422 the whole
 * `/nodes` page on the FIRST additive Rust field. Unknown keys are stripped
 * (Zod default), known keys are all always-present Rust `String`s.
 */
export const CatalogAppSchema = z.object({
  project_id: z.string(),
  /** Empty string for a placeholder row with no archive (a puller skips it). */
  archive_hash: z.string(),
  project_name: z.string(),
  category: z.string(),
  description: z.string(),
});

export type CatalogApp = z.infer<typeof CatalogAppSchema>;

/**
 * Mirrors the daemon's `NodeSummary` — one catalog-publishing node. The node
 * is a DISCOVERY source, never an authority (verrou 4): provenance comes from
 * the author-signed provenance.json at pull time, not from this row. Like the
 * catalog rows, not `.strict()` so a future additive field cannot brick the
 * page.
 */
export const NodeSummarySchema = z.object({
  /** Lowercase hex Ed25519 pubkey — dialable identity AND directory signer. */
  node_id: z.string().min(1),
  revision: z.number().int().min(0),
  app_count: z.number().int().min(0),
  catalog: z.array(CatalogAppSchema),
});

export type NodeSummary = z.infer<typeof NodeSummarySchema>;

/**
 * UX-ARRIVAL (post-S75) — un nœud OBSERVÉ : un éditeur d'annuaire entendu sur
 * le gossip SANS abonnement. Métadonnée cheap-envelope UNIQUEMENT (le blob
 * signé n'est jamais fetché pour un non-abonné — pas de `revision` ni
 * `app_count` ici) : l'identité est adossée au PoW gossip, pas à une
 * vérification Ed25519 du catalogue. Row tolérante (règle P37, pas de
 * `.strict()`) : un champ additif futur ne doit pas briquer la page.
 */
export const ObservedNodeSchema = z.object({
  /** Hex Ed25519 minuscule annoncé par l'enveloppe gossip. */
  node_id: z.string().min(1),
  /** Unix secondes (horloge LOCALE de réception) de la dernière annonce. */
  last_seen: z.number().int().min(0),
});

export type ObservedNode = z.infer<typeof ObservedNodeSchema>;

/**
 * Mirrors `GET /api/daemon/nodes` — `{ nodes: [...], observed: [...] }`. The
 * ENVELOPE is `.strict()` (pinned by the Rust test
 * `nodes_response_pins_envelope_and_grouping`); only the rows stay
 * additive-tolerant. `observed` is `.optional()` as runtime tolerance for a
 * daemon that predates UX-ARRIVAL (the current daemon ALWAYS serializes it).
 */
export const NodesResponseSchema = z
  .object({
    nodes: z.array(NodeSummarySchema),
    observed: z.array(ObservedNodeSchema).optional(),
  })
  .strict();

export type NodesResponse = z.infer<typeof NodesResponseSchema>;

/** Sprint 75 Phase F — list the subscribed catalogue-publishing nodes. */
export function listNodes(
  baseUrl: string,
): Promise<DaemonResult<NodesResponse>> {
  return callDaemon(baseUrl, "/api/daemon/nodes", NodesResponseSchema);
}

// =================================================================
// Shard session — Sprint 77 Phase J
// =================================================================

/**
 * Mirrors the daemon's `ShardSessionView` — an AGGREGATE status of one private
 * compute-group shard session. Privacy-whitelisted by the producer: it carries
 * a `member_count`, NEVER the worker/initiator pubkeys of the private group
 * (THREAT_MODEL §16 SI-3/SI-4).
 *
 * Row tolerant (NOT `.strict()`, S73/S75 rule): the parser stays tolerant if a
 * later producer adds runtime fields (e.g. `pipeline_status`,
 * `verification_level`) to the session row — an unknown/extra field must not
 * brick this panel.
 *
 * Sprint 81 Phase I: the live session registry landed daemon-side —
 * `rtt_frontier_ms` is the worst frontier RTT measured at the session's
 * readiness barrier (an aggregate transport measurement, still no identity).
 * The CURRENT producer always serializes it (`null` until sampled) →
 * `.nullable()`; `.optional()` on top keeps this consumer tolerant of an
 * OLDER daemon that predates the field (shell/daemon version skew must
 * degrade to "no RTT shown", never brick the panel).
 */
export const ShardSessionViewSchema = z.object({
  session_id: z.string(),
  member_count: z.number().int().nonnegative(),
  rtt_frontier_ms: z.number().int().nonnegative().nullable().optional(),
});

export type ShardSessionView = z.infer<typeof ShardSessionViewSchema>;

/**
 * Mirrors `GET /api/daemon/shard-session/{id}` — `{ found, session }`. The
 * ENVELOPE is `.strict()` (pinned by the Rust test
 * `shard_session_response_pins_empty_envelope`); `session` is `.nullable()`
 * (NOT `.optional()`) because the producer ALWAYS serializes the key — `null`
 * on a miss. Since S81 Phase I the daemon reads a LIVE session registry
 * (populated by the operator mount tool): an unmounted id still answers
 * `{found:false, session:null}` (the panel's empty state), a mounted one
 * answers `found:true` with the aggregate view.
 */
export const ShardSessionStatusResponseSchema = z
  .object({
    found: z.boolean(),
    session: ShardSessionViewSchema.nullable(),
  })
  .strict();

export type ShardSessionStatusResponse = z.infer<
  typeof ShardSessionStatusResponseSchema
>;

/**
 * Sprint 77 Phase J — read-only status of a private compute-group shard
 * session. Control-plane only: an aggregate status, never member identities.
 */
export function getShardSession(
  baseUrl: string,
  id: string,
): Promise<DaemonResult<ShardSessionStatusResponse>> {
  return callDaemon(
    baseUrl,
    `/api/daemon/shard-session/${encodeURIComponent(id)}`,
    ShardSessionStatusResponseSchema,
  );
}

/**
 * Sprint 75 Phase F — "ajouter une ancre". An anchor IS a subscription in the
 * SAME attention set as curators (kickoff D1/Q3/DQ3: one attention set, no
 * separate `[directory]` section) — so adding an anchor is exactly the
 * existing `POST /api/daemon/curators/subscribe`. The alias keeps the
 * node-Browse vocabulary honest without inventing a second route. The
 * directory then arrives via gossip or the boot re-pull (subscribe is NOT
 * synchronous-ingest), hence the "waiting for the first announcement"
 * cold-start affordance in the UI.
 */
export const addAnchor = subscribeCurator;

// =================================================================
// Search — FTS5 full-text index (Sprint 67 endpoint, Sprint 73 Phase E)
// =================================================================

/**
 * Mirrors `nexus_coordinator_rs::search::SearchResult`.
 *
 * The seven base columns plus the Sprint 73 Phase D provenance
 * triplet. The Rust `search_handler` (`nexus-shell-daemon/src/search_api.rs`)
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

