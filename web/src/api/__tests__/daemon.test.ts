// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 7 Phase E — unit tests for the coordinator `/daemon/*`
 * proxy client (`src/api/daemon.ts`).
 *
 * These tests stub `globalThis.fetch` so we exercise every
 * branch of the discriminated-union result path without
 * spinning up a real coordinator:
 *
 * - envelope parsing for `kind: "data"` / `"unavailable"` /
 *   `"error"`
 * - transport failure (fetch throws) → `{ kind: "unavailable" }`
 * - non-JSON body → `{ kind: "error" }`
 * - 503 with a readable envelope → reason propagated
 * - 503 with garbage envelope → fallback reason
 * - Zod schema enforcement on the daemon's data payload
 *
 * The proxy layer also guards the POST body validation at
 * 400, which we cover too.
 *
 * The `isValidCuratorPubkey` helper gets its own focused
 * tests because it runs client-side before every subscribe
 * call and a regression there would hide bad user input.
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
  getDaemonInfo,
  isValidCuratorPubkey,
  listBrowse,
  listCurators,
  subscribeCurator,
  unsubscribeCurator,
} from "@/api/daemon";

const COORD = "http://127.0.0.1:8765";

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
    // Canonical form is lowercase — uppercase would give the
    // same key two identities in a sorted set.
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
// getDaemonInfo — happy path
// ---------------------------------------------------------------

describe("getDaemonInfo", () => {
  it("returns kind=data + parsed body when proxy forwards 200", async () => {
    mockFetchOk({
      kind: "data",
      status: 200,
      body: {
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
      },
    });
    const result = await getDaemonInfo(COORD);
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.status).toBe(200);
    expect(result.body.schema_version).toBe(1);
    expect(result.body.api_port).toBe(18765);
  });

  it("returns kind=unavailable when proxy answers 503", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        mockFetchResponse({
          status: 503,
          body: { kind: "unavailable", reason: "shell-daemon not running" },
        }),
      ),
    );
    const result = await getDaemonInfo(COORD);
    expect(result.kind).toBe("unavailable");
    if (result.kind !== "unavailable") throw new Error("unreachable");
    expect(result.reason).toContain("not running");
  });

  it("returns kind=unavailable when fetch throws (network error)", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        throw new TypeError("Failed to fetch");
      }),
    );
    const result = await getDaemonInfo(COORD);
    expect(result.kind).toBe("unavailable");
    if (result.kind !== "unavailable") throw new Error("unreachable");
    expect(result.reason).toContain("Failed to fetch");
  });

  it("returns kind=unavailable when 503 body is unreadable", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        mockFetchResponse({
          status: 503,
          body: { kind: "wrong-shape" },
        }),
      ),
    );
    const result = await getDaemonInfo(COORD);
    expect(result.kind).toBe("unavailable");
  });

  it("throws CoordinatorProtocolError when data body fails Zod", async () => {
    mockFetchOk({
      kind: "data",
      status: 200,
      body: {
        // schema_version must be literal 1 — deliberate
        // regression catcher
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
      },
    });
    await expect(getDaemonInfo(COORD)).rejects.toThrow(/protocol error/);
  });
});

// ---------------------------------------------------------------
// listCurators / subscribeCurator / unsubscribeCurator
// ---------------------------------------------------------------

describe("listCurators", () => {
  it("parses entries[] + subscribed_curators", async () => {
    mockFetchOk({
      kind: "data",
      status: 200,
      body: {
        entries: [],
        subscribed_curators: ["aa".repeat(32)],
      },
    });
    const result = await listCurators(COORD);
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
        body: {
          kind: "data",
          status: 200,
          body: { subscribed_curators: ["bb".repeat(32)] },
        },
      }),
    );
    vi.stubGlobal("fetch", spy);
    const result = await subscribeCurator(COORD, "bb".repeat(32));
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.subscribed_curators).toEqual(["bb".repeat(32)]);

    // Assert the body the fetch wrapper actually sent.
    expect(spy).toHaveBeenCalledOnce();
    const calls = spy.mock.calls as unknown as [
      RequestInfo | URL,
      RequestInit | undefined,
    ][];
    const [urlArg, initArg] = calls[0];
    expect(String(urlArg)).toBe(`${COORD}/daemon/curators/subscribe`);
    expect(initArg?.method).toBe("POST");
    expect(initArg?.body).toBe(
      JSON.stringify({ curator_pubkey_hex: "bb".repeat(32) }),
    );
  });

  it("returns kind=error when proxy rejects with 400", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        mockFetchResponse({
          status: 400,
          body: { kind: "error", reason: "request body must be a JSON object" },
        }),
      ),
    );
    const result = await subscribeCurator(COORD, "bb".repeat(32));
    expect(result.kind).toBe("error");
    if (result.kind !== "error") throw new Error("unreachable");
    expect(result.reason).toContain("JSON object");
  });
});

describe("unsubscribeCurator", () => {
  it("URL-encodes the pubkey path param", async () => {
    const spy = vi.fn(async () =>
      mockFetchResponse({
        status: 200,
        body: {
          kind: "data",
          status: 200,
          body: { subscribed_curators: [] },
        },
      }),
    );
    vi.stubGlobal("fetch", spy);
    const result = await unsubscribeCurator(COORD, "cc".repeat(32));
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.subscribed_curators).toEqual([]);

    const calls = spy.mock.calls as unknown as [
      RequestInfo | URL,
      RequestInit | undefined,
    ][];
    const [urlArg, initArg] = calls[0];
    expect(String(urlArg)).toBe(`${COORD}/daemon/curators/${"cc".repeat(32)}`);
    expect(initArg?.method).toBe("DELETE");
  });
});

// ---------------------------------------------------------------
// listBrowse
// ---------------------------------------------------------------

describe("listBrowse", () => {
  it("parses a mix of reachable / unreachable / unknown entries", async () => {
    mockFetchOk({
      kind: "data",
      status: 200,
      body: {
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
      },
    });
    const result = await listBrowse(COORD);
    expect(result.kind).toBe("data");
    if (result.kind !== "data") throw new Error("unreachable");
    expect(result.body.entries.length).toBe(3);
    expect(result.body.entries.map((e) => e.status)).toEqual([
      "reachable",
      "unreachable",
      "unknown",
    ]);
    // Nullable last_probed_at survives JSON null
    expect(result.body.entries[2].last_probed_at).toBe(null);
  });

  it("rejects invalid status discriminator via Zod", async () => {
    mockFetchOk({
      kind: "data",
      status: 200,
      body: {
        entries: [
          {
            project_id: "aa".repeat(32),
            project_name: "p",
            category: "c",
            description: "d",
            curator_pubkey: "bb".repeat(32),
            curator_name: "n",
            status: "banana", // invalid
            last_probed_at: null,
          },
        ],
      },
    });
    await expect(listBrowse(COORD)).rejects.toThrow(/protocol error/);
  });
});

// ---------------------------------------------------------------------------
// Sprint 8 audit A-3 — cross-language canonical fixture
// ---------------------------------------------------------------------------
//
// This block consumes the exact JSON file the Python side writes
// in `packages/nexus-sdk/tests/snapshots/curator_canonical.json`.
// The file is signed by a deterministic keypair derived from a
// fixed 32-byte seed; any drift on either side fails one of:
//
// - the Python `test_canonical_fixture_roundtrip` (signature or
//   keypair derivation regression), or
// - this Vitest block (Zod shape regression or a silent schema
//   drift the Rust audit wouldn't catch).

import { readFileSync } from "fs";
import { resolve } from "path";

import { CuratorListEntrySchema } from "@/api/daemon";

// Resolved at test import time. The snapshot lives under
// `packages/nexus-sdk/tests/snapshots/curator_canonical.json` —
// a sibling of the SDK test suite that writes the file via
// Python sign_curator_list. Loading through `readFileSync`
// (rather than a bundler-resolved `import ... from "*.json"`)
// keeps the `web/tsconfig.app.json:include = ["src"]` scope
// clean without adding the monorepo's SDK tests to the TS
// project. Vitest runs tests from `web/` by default, so the
// path resolves from there.
const CURATOR_CANONICAL_PATH = resolve(
  "../packages/nexus-sdk/tests/snapshots/curator_canonical.json",
);
const curatorCanonical = JSON.parse(
  readFileSync(CURATOR_CANONICAL_PATH, "utf-8"),
);

describe("CuratorListEntrySchema cross-language fixture (A-3)", () => {
  it("parses the Python-signed canonical fixture", () => {
    // `resolveJsonModule` imports the JSON as a typed const;
    // the schema accepts the numeric arrays verbatim.
    const parsed = CuratorListEntrySchema.safeParse(curatorCanonical);
    if (!parsed.success) {
      // Print the Zod issues so a regression is diagnosable
      // directly from the CI log without rerunning locally.
      throw new Error(
        "cross-lang fixture failed Zod: " +
          JSON.stringify(parsed.error.issues, null, 2),
      );
    }
    expect(parsed.data.list.version).toBe(1);
    expect(parsed.data.curator_pubkey.length).toBe(32);
    expect(parsed.data.signature.length).toBe(64);
    expect(parsed.data.list.entries.length).toBe(2);
    // The entry data must survive Zod verbatim.
    expect(parsed.data.list.entries[0].project_name).toBe("gov");
    expect(parsed.data.list.entries[1].project_name).toBe("coldcase");
  });

  it("enforces Sprint 8 A-4 project field length caps", () => {
    // A clone that bumps `description` past the 280-char cap
    // must fail Zod — mirrors the Rust verify_rejects_oversized_fields
    // regression guard. The cap is the shell's second line of
    // defense behind the Rust verifier.
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
