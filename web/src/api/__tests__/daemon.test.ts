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
  blobServeUrl,
  daemonBaseUrlFromInfo,
  getDaemonInfo,
  isValidCuratorPubkey,
  listBrowse,
  listCurators,
  subscribeCurator,
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
    expect(String(urlArg)).toBe(`${BASE}/curators/subscribe`);
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
    expect(String(urlArg)).toBe(`${BASE}/curators/${"cc".repeat(32)}`);
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

// ---------------------------------------------------------------------------
// Sprint 8 audit A-3 — cross-language canonical fixture
// ---------------------------------------------------------------------------

import { readFileSync } from "fs";
import { resolve } from "path";

import { CuratorListEntrySchema } from "@/api/daemon";

const CURATOR_CANONICAL_PATH = resolve(
  "../packages/nexus-sdk/tests/snapshots/curator_canonical.json",
);
const curatorCanonical = JSON.parse(
  readFileSync(CURATOR_CANONICAL_PATH, "utf-8"),
);

describe("CuratorListEntrySchema cross-language fixture (A-3)", () => {
  it("parses the Python-signed canonical fixture", () => {
    const parsed = CuratorListEntrySchema.safeParse(curatorCanonical);
    if (!parsed.success) {
      throw new Error(
        "cross-lang fixture failed Zod: " +
          JSON.stringify(parsed.error.issues, null, 2),
      );
    }
    expect(parsed.data.list.version).toBe(1);
    expect(parsed.data.curator_pubkey.length).toBe(32);
    expect(parsed.data.signature.length).toBe(64);
    expect(parsed.data.list.entries.length).toBe(2);
    expect(parsed.data.list.entries[0].project_name).toBe("gov");
    expect(parsed.data.list.entries[1].project_name).toBe("coldcase");
  });

  it("enforces Sprint 8 A-4 project field length caps", () => {
    const oversized = JSON.parse(JSON.stringify(curatorCanonical));
    oversized.list.entries[0].description = "x".repeat(281);
    const parsed = CuratorListEntrySchema.safeParse(oversized);
    expect(parsed.success).toBe(false);
  });

  it("enforces category 64-char cap", () => {
    const oversized = JSON.parse(JSON.stringify(curatorCanonical));
    oversized.list.entries[0].category = "c".repeat(65);
    const parsed = CuratorListEntrySchema.safeParse(oversized);
    expect(parsed.success).toBe(false);
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
