// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Unit tests for the daemon API client (`src/api/daemon.ts`).
 *
 * The daemon is called directly (no proxy envelope). Tests stub
 * `globalThis.fetch` and exercise the `DaemonResult<T>` union:
 *
 * - 200 + valid JSON → `{ kind: "data" }`
 * - 503 → `{ kind: "unavailable" }`
 * - fetch throws → `{ kind: "unavailable" }`
 * - 4xx → `{ kind: "error" }`
 * - Zod schema enforcement on the response body
 */

import {
  afterEach,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from "vitest";

import {
  addAnchor,
  blobServeUrl,
  daemonBaseUrlFromInfo,
  getDaemonInfo,
  isValidCuratorPubkey,
  listBrowse,
  listCurators,
  listNodes,
  NodesResponseSchema,
  searchBrowse,
  SearchResponseSchema,
  seedCount,
  seedVoluntary,
  subscribeCurator,
  triggerPanicWipe,
  unsubscribeCurator,
  type DaemonInfo,
} from "@/api/daemon";

const BASE = "http://127.0.0.1:18765";

function mockFetchResponse(args: {
  status: number;
  body: unknown | string;
}): Response {
  const body =
    typeof args.body === "string" ? args.body : JSON.stringify(args.body);
  return new Response(body, {
    status: args.status,
    headers: { "content-type": "application/json" },
  });
}

function mockFetchOk<T>(body: T): void {
  vi.stubGlobal(
    "fetch",
    vi.fn(async () =>
      mockFetchResponse({
        status: 200,
        body,
      }),
    ),
  );
}

beforeEach(() => {
  vi.restoreAllMocks();
});
afterEach(() => {
  vi.unstubAllGlobals();
});

// ---------------------------------------------------------------
// isValidCuratorPubkey
// ---------------------------------------------------------------

describe("isValidCuratorPubkey", () => {
  it("accepts 64 lowercase hex chars", () => {
    expect(isValidCuratorPubkey("a".repeat(64))).toBe(true);
    expect(
      isValidCuratorPubkey(
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      ),
    ).toBe(true);
  });

  it("rejects uppercase hex", () => {
    expect(isValidCuratorPubkey("A".repeat(64))).toBe(false);
  });

  it("rejects wrong length", () => {
    expect(isValidCuratorPubkey("")).toBe(false);
    expect(isValidCuratorPubkey("a".repeat(63))).toBe(false);
    expect(isValidCuratorPubkey("a".repeat(65))).toBe(false);
  });

  it("rejects non-hex chars", () => {
    expect(isValidCuratorPubkey("z".repeat(64))).toBe(false);
    expect(
      isValidCuratorPubkey("abcd efgh".padEnd(64, "a")),
    ).toBe(false);
  });
});

// ---------------------------------------------------------------
// getDaemonInfo
// ---------------------------------------------------------------

describe("getDaemonInfo", () => {
  it("returns kind=data + parsed body on 200", async () => {
    mockFetchOk({
      schema_version: 1,
      node_id: "aa".repeat(32),
      daemon_version: "0.1.0",
      uptime_secs: 10,
      started_at: "2026-04-11T12:00:00Z",
      last_updated_at: "2026-04-11T12:00:10Z",
      api_host: "127.0.0.1",
      api_port: 18765,
      subscribed_curators: [],
      known_lists: 0,
      known_browse_entries: 0,
    });
    const result = await getDaemonInfo(BASE);
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.status).toBe(200);
    expect(result.body.schema_version).toBe(1);
    expect(result.body.api_port).toBe(18765);
  });

  it("returns kind=unavailable on 503", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        mockFetchResponse({
          status: 503,
          body: { error: "maintenance" },
        }),
      ),
    );
    const result = await getDaemonInfo(BASE);
    expect(result.kind).toBe("unavailable");
  });

  it("returns kind=unavailable when fetch throws (network error)", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        throw new TypeError("Failed to fetch");
      }),
    );
    const result = await getDaemonInfo(BASE);
    expect(result.kind).toBe("unavailable");
    if (result.kind !== "unavailable") throw new Error("unreachable");
    expect(result.reason).toContain("Failed to fetch");
  });

  it("throws ApiProtocolError when body fails Zod", async () => {
    mockFetchOk({
      schema_version: 999,
      node_id: "aa".repeat(32),
      daemon_version: "0.1.0",
      uptime_secs: 0,
      started_at: "x",
      last_updated_at: "x",
      api_host: "127.0.0.1",
      api_port: 18765,
      subscribed_curators: [],
      known_lists: 0,
      known_browse_entries: 0,
    });
    await expect(getDaemonInfo(BASE)).rejects.toThrow(/protocol error/);
  });
});

// ---------------------------------------------------------------
// listCurators / subscribeCurator / unsubscribeCurator
// ---------------------------------------------------------------

describe("listCurators", () => {
  it("parses entries[] + subscribed_curators", async () => {
    mockFetchOk({
      entries: [],
      subscribed_curators: ["aa".repeat(32)],
    });
    const result = await listCurators(BASE);
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.subscribed_curators).toEqual(["aa".repeat(32)]);
    expect(result.body.entries).toEqual([]);
  });
});

describe("subscribeCurator", () => {
  it("serializes the pubkey body and returns the new set", async () => {
    const spy = vi.fn(async () =>
      mockFetchResponse({
        status: 200,
        body: { subscribed_curators: ["bb".repeat(32)] },
      }),
    );
    vi.stubGlobal("fetch", spy);
    const result = await subscribeCurator(BASE, "bb".repeat(32));
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.subscribed_curators).toEqual(["bb".repeat(32)]);

    expect(spy).toHaveBeenCalledOnce();
    const calls = spy.mock.calls as unknown as [
      RequestInfo | URL,
      RequestInit | undefined,
    ][];
    const [urlArg, initArg] = calls[0];
    expect(String(urlArg)).toBe(`${BASE}/api/daemon/curators/subscribe`);
    expect(initArg?.method).toBe("POST");
    expect(initArg?.body).toBe(
      JSON.stringify({ curator_pubkey_hex: "bb".repeat(32) }),
    );
  });

  it("returns kind=error on 400", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        mockFetchResponse({
          status: 400,
          body: { error: "bad request" },
        }),
      ),
    );
    const result = await subscribeCurator(BASE, "bb".repeat(32));
    expect(result.kind).toBe("error");
  });
});

describe("unsubscribeCurator", () => {
  it("URL-encodes the pubkey path param", async () => {
    const spy = vi.fn(async () =>
      mockFetchResponse({
        status: 200,
        body: { subscribed_curators: [] },
      }),
    );
    vi.stubGlobal("fetch", spy);
    const result = await unsubscribeCurator(BASE, "cc".repeat(32));
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.subscribed_curators).toEqual([]);

    const calls = spy.mock.calls as unknown as [
      RequestInfo | URL,
      RequestInit | undefined,
    ][];
    const [urlArg, initArg] = calls[0];
    expect(String(urlArg)).toBe(`${BASE}/api/daemon/curators/${"cc".repeat(32)}`);
    expect(initArg?.method).toBe("DELETE");
  });
});

// ---------------------------------------------------------------
// listBrowse
// ---------------------------------------------------------------

describe("listBrowse", () => {
  it("parses a mix of reachable / unreachable / unknown entries", async () => {
    mockFetchOk({
      entries: [
        {
          project_id: "aa".repeat(32),
          project_name: "gov",
          category: "gov",
          description: "desc",
          curator_pubkey: "bb".repeat(32),
          curator_name: "FlowUP",
          status: "reachable",
          last_probed_at: "2026-04-11T12:00:00Z",
        },
        {
          project_id: "cc".repeat(32),
          project_name: "coldcase",
          category: "invest",
          description: "desc",
          curator_pubkey: "bb".repeat(32),
          curator_name: "FlowUP",
          status: "unreachable",
          last_probed_at: "2026-04-11T12:00:00Z",
        },
        {
          project_id: "dd".repeat(32),
          project_name: "forensics",
          category: "tool",
          description: "desc",
          curator_pubkey: "bb".repeat(32),
          curator_name: "FlowUP",
          status: "unknown",
          last_probed_at: null,
        },
      ],
    });
    const result = await listBrowse(BASE);
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.entries.length).toBe(3);
    expect(result.body.entries.map((e) => e.status)).toEqual([
      "reachable",
      "unreachable",
      "unknown",
    ]);
    expect(result.body.entries[2].last_probed_at).toBe(null);
  });

  it("rejects invalid status discriminator via Zod", async () => {
    mockFetchOk({
      entries: [
        {
          project_id: "aa".repeat(32),
          project_name: "p",
          category: "c",
          description: "d",
          curator_pubkey: "bb".repeat(32),
          curator_name: "n",
          status: "banana",
          last_probed_at: null,
        },
      ],
    });
    await expect(listBrowse(BASE)).rejects.toThrow(/protocol error/);
  });
});

// ---------------------------------------------------------------
// BrowseEntry source field
// ---------------------------------------------------------------

describe("BrowseEntry source field", () => {
  it("accepts entries with source='direct'", async () => {
    mockFetchOk({
      entries: [
        {
          project_id: "aa".repeat(32),
          project_name: "gov",
          category: "gov",
          description: "desc",
          curator_pubkey: "",
          curator_name: "Self-published",
          source: "direct",
          status: "reachable",
          last_probed_at: "2026-04-12T12:00:00Z",
        },
      ],
    });
    const result = await listBrowse(BASE);
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.entries[0].source).toBe("direct");
  });

  it("accepts entries with source='nodedirectory' (Sprint 75 Phase C PULL discovery)", async () => {
    // The daemon emits "nodedirectory" for an app discovered through a
    // subscribed node's signed directory; the Zod enum must accept it so a
    // mixed-version /browse response still parses instead of erroring the list.
    mockFetchOk({
      entries: [
        {
          project_id: "cc".repeat(32),
          project_name: "Babel",
          category: "translation",
          description: "community translation",
          curator_pubkey: "dd".repeat(32),
          curator_name: "Node catalog",
          source: "nodedirectory",
          status: "reachable",
          last_probed_at: "2026-06-09T12:00:00Z",
          archive_hash: "ee".repeat(32),
        },
      ],
    });
    const result = await listBrowse(BASE);
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.entries[0].source).toBe("nodedirectory");
  });

  it("accepts entries without source (backward compat, defaults to undefined)", async () => {
    mockFetchOk({
      entries: [
        {
          project_id: "aa".repeat(32),
          project_name: "gov",
          category: "gov",
          description: "desc",
          curator_pubkey: "bb".repeat(32),
          curator_name: "FlowUP",
          status: "reachable",
          last_probed_at: "2026-04-12T12:00:00Z",
        },
      ],
    });
    const result = await listBrowse(BASE);
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.entries[0].source).toBeUndefined();
  });
});

// ---------------------------------------------------------------
// BrowseEntry is_own (KEEP-ONLINE-READ-PATH, Sprint 74 Phase G)
// ---------------------------------------------------------------

describe("BrowseEntry is_own", () => {
  it("parses the daemon-derived is_own flag (true for a self-hosted app)", async () => {
    // The daemon's list_browse serializes is_own=true when the entry's
    // hosting node_id == our node_id (a self-deployed/published app), so the
    // shell shows the owner toggle even for per-app deploys whose
    // project_id = blake3(name) != node_id.
    mockFetchOk({
      entries: [
        {
          project_id: "aa".repeat(32),
          project_name: "gov",
          category: "gov",
          description: "desc",
          curator_pubkey: "",
          curator_name: "Self-published",
          source: "direct",
          status: "reachable",
          last_probed_at: "2026-04-12T12:00:00Z",
          is_own: true,
        },
      ],
    });
    const result = await listBrowse(BASE);
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.entries[0].is_own).toBe(true);
  });

  it("tolerates a daemon that predates is_own (undefined, not a parse error)", async () => {
    mockFetchOk({
      entries: [
        {
          project_id: "aa".repeat(32),
          project_name: "gov",
          category: "gov",
          description: "desc",
          curator_pubkey: "bb".repeat(32),
          curator_name: "FlowUP",
          status: "reachable",
          last_probed_at: "2026-04-12T12:00:00Z",
        },
      ],
    });
    const result = await listBrowse(BASE);
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.entries[0].is_own).toBeUndefined();
  });
});

// ---------------------------------------------------------------
// BrowseEntry archive fields + helpers
// ---------------------------------------------------------------

describe("BrowseEntry archive_ticket + archive_hash", () => {
  it("accepts entries with archive_ticket and archive_hash", async () => {
    mockFetchOk({
      entries: [
        {
          project_id: "aa".repeat(32),
          project_name: "web-app",
          category: "misc",
          description: "has archive",
          curator_pubkey: "",
          curator_name: "Self-published",
          source: "direct",
          status: "reachable",
          last_probed_at: null,
          archive_ticket: "blobticket_abc123",
          archive_hash: "ab".repeat(32),
        },
      ],
    });
    const result = await listBrowse(BASE);
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.entries[0].archive_ticket).toBe("blobticket_abc123");
    expect(result.body.entries[0].archive_hash).toBe("ab".repeat(32));
  });

  it("accepts entries without archive fields (backward compat)", async () => {
    mockFetchOk({
      entries: [
        {
          project_id: "aa".repeat(32),
          project_name: "old",
          category: "misc",
          description: "no archive",
          curator_pubkey: "bb".repeat(32),
          curator_name: "FlowUP",
          status: "reachable",
          last_probed_at: null,
        },
      ],
    });
    const result = await listBrowse(BASE);
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.entries[0].archive_ticket).toBeUndefined();
    expect(result.body.entries[0].archive_hash).toBeUndefined();
  });
});

// ---------------------------------------------------------------
// BrowseEntry is_open_source flag
// ---------------------------------------------------------------

describe("BrowseEntry is_open_source", () => {
  it("accepts entries with is_open_source=true", async () => {
    mockFetchOk({
      entries: [
        {
          project_id: "aa".repeat(32),
          project_name: "verified-app",
          category: "misc",
          description: "deploy-from-repo",
          curator_pubkey: "",
          curator_name: "Self-published",
          source: "direct",
          status: "reachable",
          last_probed_at: null,
          archive_ticket: "blobticket_abc",
          archive_hash: "ab".repeat(32),
          repo_url: "https://github.com/test/verified-app",
          provenance_hash: "cd".repeat(32),
          is_open_source: true,
        },
      ],
    });
    const result = await listBrowse(BASE);
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.entries[0].is_open_source).toBe(true);
  });

  it("accepts entries with is_open_source=false (private deploy)", async () => {
    mockFetchOk({
      entries: [
        {
          project_id: "aa".repeat(32),
          project_name: "private-app",
          category: "misc",
          description: "zip upload",
          curator_pubkey: "",
          curator_name: "Self-published",
          source: "direct",
          status: "reachable",
          last_probed_at: null,
          is_open_source: false,
        },
      ],
    });
    const result = await listBrowse(BASE);
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.entries[0].is_open_source).toBe(false);
  });
});

// ---------------------------------------------------------------
// searchBrowse + SearchResponseSchema (Sprint 73 Phase E)
// ---------------------------------------------------------------

function makeSearchHit(overrides: Record<string, unknown> = {}) {
  return {
    project_id: "aa".repeat(32),
    project_name: "verified-app",
    category: "outils",
    description: "App deployee depuis sa source",
    op_type: "ReleasePublished",
    source_type: "feed",
    score: 1.42,
    repo_url: "https://github.com/test/verified-app",
    commit_sha: "ab".repeat(20),
    archive_hash: "cd".repeat(32),
    provenance_hash: "ef".repeat(32),
    is_open_source: true,
    ...overrides,
  };
}

describe("searchBrowse", () => {
  it("searchBrowse_calls_daemon_search_endpoint", async () => {
    const spy = vi.fn(async () =>
      mockFetchResponse({
        status: 200,
        body: { results: [], total: 0, took_ms: 1 },
      }),
    );
    vi.stubGlobal("fetch", spy);

    const result = await searchBrowse(BASE, "react");
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.total).toBe(0);

    expect(spy).toHaveBeenCalledOnce();
    const calls = spy.mock.calls as unknown as [
      RequestInfo | URL,
      RequestInit | undefined,
    ][];
    const [urlArg] = calls[0];
    // Default limit/offset are appended; q is percent-encoded via
    // URLSearchParams so it can never break out of the query string.
    expect(String(urlArg)).toBe(
      `${BASE}/api/daemon/search?q=react&limit=20&offset=0`,
    );
  });

  it("percent-encodes a pathological query", async () => {
    const spy = vi.fn(async () =>
      mockFetchResponse({
        status: 200,
        body: { results: [], total: 0, took_ms: 0 },
      }),
    );
    vi.stubGlobal("fetch", spy);

    await searchBrowse(BASE, "a&b=c d", 5, 10);
    const calls = spy.mock.calls as unknown as [RequestInfo | URL][];
    expect(String(calls[0][0])).toBe(
      `${BASE}/api/daemon/search?q=a%26b%3Dc+d&limit=5&offset=10`,
    );
  });

  it("returns kind=unavailable on 503", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        mockFetchResponse({ status: 503, body: { error: "busy" } }),
      ),
    );
    const result = await searchBrowse(BASE, "react");
    expect(result.kind).toBe("unavailable");
  });
});

describe("SearchResponseSchema", () => {
  it("search_response_schema_parses_triplet", async () => {
    mockFetchOk({
      results: [makeSearchHit()],
      total: 1,
      took_ms: 3,
    });
    const result = await searchBrowse(BASE, "verified");
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    const hit = result.body.results[0];
    expect(hit.repo_url).toBe("https://github.com/test/verified-app");
    expect(hit.commit_sha).toBe("ab".repeat(20));
    expect(hit.archive_hash).toBe("cd".repeat(32));
    expect(hit.provenance_hash).toBe("ef".repeat(32));
    expect(hit.is_open_source).toBe(true);
  });

  it("parses a hit whose triplet is null (non-release op)", () => {
    const parsed = SearchResponseSchema.safeParse({
      results: [
        makeSearchHit({
          op_type: "CuratorVouched",
          repo_url: null,
          commit_sha: null,
          archive_hash: null,
          provenance_hash: null,
          is_open_source: false,
        }),
      ],
      total: 1,
      took_ms: 0,
    });
    expect(parsed.success).toBe(true);
    if (!parsed.success) throw new Error("unreachable");
    expect(parsed.data.results[0].repo_url).toBeNull();
    expect(parsed.data.results[0].is_open_source).toBe(false);
  });

  it("rejects a hit that omits a provenance key (strict, not optional)", async () => {
    // The Rust handler always serialises the four provenance keys as
    // `null` when absent — never omits them. Modelling them as
    // `.nullable()` (not `.optional()`) keeps this strict so a future
    // wire drift surfaces as a protocol error instead of silently
    // dropping the field.
    const withoutRepoUrl = makeSearchHit();
    delete (withoutRepoUrl as Record<string, unknown>).repo_url;
    mockFetchOk({ results: [withoutRepoUrl], total: 1, took_ms: 0 });
    await expect(searchBrowse(BASE, "x")).rejects.toThrow(/protocol error/);
  });
});

describe("blobServeUrl", () => {
  it("constructs the correct URL with default path", () => {
    const url = blobServeUrl("http://127.0.0.1:7000", "ab".repeat(32));
    expect(url).toBe(`http://127.0.0.1:7000/blob-serve/${"ab".repeat(32)}/index.html`);
  });

  it("constructs the correct URL with custom path", () => {
    const url = blobServeUrl("http://127.0.0.1:7000", "cd".repeat(32), "assets/main.js");
    expect(url).toBe(`http://127.0.0.1:7000/blob-serve/${"cd".repeat(32)}/assets/main.js`);
  });
});

describe("daemonBaseUrlFromInfo", () => {
  it("builds http URL from host and port", () => {
    const info = {
      api_host: "127.0.0.1",
      api_port: 7000,
    } as DaemonInfo;
    expect(daemonBaseUrlFromInfo(info)).toBe("http://127.0.0.1:7000");
  });
});

// ---------------------------------------------------------------
// seedCount (Sprint 74 Phase F)
// ---------------------------------------------------------------

describe("seedCount", () => {
  it("parses {peer_count, self_seeding} and URL-encodes the project_id", async () => {
    const spy = vi.fn(async () =>
      mockFetchResponse({
        status: 200,
        body: { peer_count: 3, self_seeding: true, self_pin_enabled: true },
      }),
    );
    vi.stubGlobal("fetch", spy);

    const result = await seedCount(BASE, "ab".repeat(32));
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.peer_count).toBe(3);
    expect(result.body.self_seeding).toBe(true);

    const calls = spy.mock.calls as unknown as [
      RequestInfo | URL,
      RequestInit | undefined,
    ][];
    expect(String(calls[0][0])).toBe(
      `${BASE}/api/daemon/seed-count/${"ab".repeat(32)}`,
    );
  });

  it("returns kind=unavailable on 503", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => mockFetchResponse({ status: 503, body: {} })),
    );
    const result = await seedCount(BASE, "ab".repeat(32));
    expect(result.kind).toBe("unavailable");
  });

  it("throws ApiProtocolError when a key is missing (strict schema)", async () => {
    // The Rust handler ALWAYS serialises both keys; a body omitting one is a
    // protocol drift, not a tolerated shape (.strict()).
    mockFetchOk({ peer_count: 1 });
    await expect(seedCount(BASE, "ab".repeat(32))).rejects.toThrow();
  });

  it("appends ?archive_hash= for version-scoped counts (Sprint 75 Phase C WIRE-2)", async () => {
    // Without this query the count would be version-agnostic even when the caller
    // knows the exact archive_hash — the WIRE-2 read-side that closes the carry.
    const spy = vi.fn(async () =>
      mockFetchResponse({
        status: 200,
        body: { peer_count: 2, self_seeding: false, self_pin_enabled: null },
      }),
    );
    vi.stubGlobal("fetch", spy);
    const hash = "cd".repeat(32);
    const result = await seedCount(BASE, "ab".repeat(32), hash);
    expect(result.kind).toBe("data");
    const calls = spy.mock.calls as unknown as [
      RequestInfo | URL,
      RequestInit | undefined,
    ][];
    expect(String(calls[0][0])).toBe(
      `${BASE}/api/daemon/seed-count/${"ab".repeat(32)}?archive_hash=${hash}`,
    );
  });

  it("omits the query when no archiveHash is given (version-agnostic, backward compatible)", async () => {
    const spy = vi.fn(async () =>
      mockFetchResponse({
        status: 200,
        body: { peer_count: 1, self_seeding: false, self_pin_enabled: null },
      }),
    );
    vi.stubGlobal("fetch", spy);
    await seedCount(BASE, "ab".repeat(32), null);
    const calls = spy.mock.calls as unknown as [
      RequestInfo | URL,
      RequestInit | undefined,
    ][];
    expect(String(calls[0][0])).toBe(
      `${BASE}/api/daemon/seed-count/${"ab".repeat(32)}`,
    );
  });

  it("parses the three-valued self_pin_enabled (WEB-1, Sprint 75 Phase F)", async () => {
    // null = never toggled (row absent), true/false = explicit intent. The
    // Rust handler ALWAYS serialises the key (Option -> JSON null), so the
    // schema is .nullable(), never .optional() — an ABSENT key is a drift,
    // rejected because the key is REQUIRED (non-optional) in the schema
    // (`.strict()` only rejects EXTRA keys, not missing ones).
    for (const pin of [null, true, false]) {
      mockFetchOk({ peer_count: 0, self_seeding: false, self_pin_enabled: pin });
      const result = await seedCount(BASE, "ab".repeat(32));
      expect(result.kind).toBe("data");
      if (result.kind !== "data") throw new Error("unreachable");
      expect(result.body.self_pin_enabled).toBe(pin);
      vi.unstubAllGlobals();
    }
    // An absent key is a protocol drift under .strict() — throws.
    mockFetchOk({ peer_count: 0, self_seeding: false });
    await expect(seedCount(BASE, "ab".repeat(32))).rejects.toThrow();
  });
});

// ---------------------------------------------------------------
// listNodes + NodesResponseSchema (Sprint 75 Phase F)
// ---------------------------------------------------------------

describe("listNodes", () => {
  const NODE = {
    node_id: "ee".repeat(32),
    revision: 3,
    app_count: 1,
    catalog: [
      {
        project_id: "ab".repeat(32),
        archive_hash: "cd".repeat(32),
        project_name: "Babel",
        category: "tools",
        description: "Translation hub",
      },
    ],
  };

  it("parses the {nodes} envelope from GET /api/daemon/nodes", async () => {
    const spy = vi.fn(async () =>
      mockFetchResponse({ status: 200, body: { nodes: [NODE] } }),
    );
    vi.stubGlobal("fetch", spy);
    const result = await listNodes(BASE);
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.nodes).toHaveLength(1);
    expect(result.body.nodes[0].node_id).toBe("ee".repeat(32));
    expect(result.body.nodes[0].catalog[0].project_name).toBe("Babel");
    const calls = spy.mock.calls as unknown as [RequestInfo | URL][];
    expect(String(calls[0][0])).toBe(`${BASE}/api/daemon/nodes`);
  });

  it("rejects an unknown key on the ENVELOPE (strict)", async () => {
    // The envelope is pinned by the Rust producer test
    // (`nodes_response_pins_envelope_and_grouping`) — an extra top-level key
    // is a protocol drift, not a tolerated shape.
    mockFetchOk({ nodes: [], extra_envelope_key: 1 });
    await expect(listNodes(BASE)).rejects.toThrow();
  });

  it("tolerates an additive field on rows (review-D rule: rows NOT strict)", () => {
    // Pre-launch policy adds CatalogApp/NodeSummary fields additively with
    // 0 bump — the FIRST additive Rust field must not brick the /nodes page.
    const parsed = NodesResponseSchema.parse({
      nodes: [
        {
          ...NODE,
          future_node_field: "ignored",
          catalog: [{ ...NODE.catalog[0], future_app_field: 42 }],
        },
      ],
    });
    expect(parsed.nodes[0].catalog[0].project_id).toBe("ab".repeat(32));
    // Unknown keys are STRIPPED, not kept (Zod default object behaviour).
    expect(
      (parsed.nodes[0] as Record<string, unknown>)["future_node_field"],
    ).toBeUndefined();
  });

  it("addAnchor is the curators/subscribe route (anchor = subscription, Q3/DQ3)", async () => {
    const spy = vi.fn(async () =>
      mockFetchResponse({
        status: 200,
        body: { subscribed_curators: ["ee".repeat(32)] },
      }),
    );
    vi.stubGlobal("fetch", spy);
    const result = await addAnchor(BASE, "ee".repeat(32));
    expect(result.kind).toBe("data");
    const calls = spy.mock.calls as unknown as [
      RequestInfo | URL,
      RequestInit | undefined,
    ][];
    expect(String(calls[0][0])).toBe(`${BASE}/api/daemon/curators/subscribe`);
    expect(addAnchor).toBe(subscribeCurator);
  });
});

// ---------------------------------------------------------------
// seedVoluntary version discriminator (Sprint 75 Phase F)
// ---------------------------------------------------------------

describe("seedVoluntary", () => {
  it("pins the displayed version via archive_hash in the body", async () => {
    const spy = vi.fn(async () =>
      mockFetchResponse({
        status: 200,
        body: { ok: true, seeding: "ab".repeat(32) },
      }),
    );
    vi.stubGlobal("fetch", spy);
    const hash = "cd".repeat(32);
    await seedVoluntary(BASE, "ab".repeat(32), hash);
    const calls = spy.mock.calls as unknown as [
      RequestInfo | URL,
      RequestInit | undefined,
    ][];
    expect(String(calls[0][0])).toBe(`${BASE}/api/daemon/seed`);
    expect(JSON.parse(calls[0][1]?.body as string)).toEqual({
      project_id: "ab".repeat(32),
      archive_hash: hash,
    });
  });

  it("omits archive_hash when unknown (runtime-tolerant pre-F behaviour)", async () => {
    const spy = vi.fn(async () =>
      mockFetchResponse({
        status: 200,
        body: { ok: true, seeding: "ab".repeat(32) },
      }),
    );
    vi.stubGlobal("fetch", spy);
    await seedVoluntary(BASE, "ab".repeat(32));
    const calls = spy.mock.calls as unknown as [
      RequestInfo | URL,
      RequestInit | undefined,
    ][];
    expect(JSON.parse(calls[0][1]?.body as string)).toEqual({
      project_id: "ab".repeat(32),
    });
  });
});

// ---------------------------------------------------------------
// triggerPanicWipe
// ---------------------------------------------------------------

describe("triggerPanicWipe", () => {
  it("POSTs to the panic wipe route and parses {wiped:true}", async () => {
    const spy = vi.fn(async () => mockFetchResponse({ status: 200, body: { wiped: true } }));
    vi.stubGlobal("fetch", spy);
    const result = await triggerPanicWipe(BASE);
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.wiped).toBe(true);
    const calls = spy.mock.calls as unknown as [
      RequestInfo | URL,
      RequestInit | undefined,
    ][];
    expect(String(calls[0][0])).toBe(`${BASE}/api/daemon/panic/wipe`);
    expect(calls[0][1]?.method).toBe("POST");
  });
});
