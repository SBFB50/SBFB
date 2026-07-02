# Factory app-authoring — human reference

*Reference document (Diátaxis). This is the **human-readable twin** of the rank-1
sources. For the narrative, see [`EXPLANATION.md`](./EXPLANATION.md); for the
role-by-role guide, see [`HOW_TO_WIRE.md`](./HOW_TO_WIRE.md).*

> **Status: PROVISIONAL** for the in-vivo end-to-end journey (see
> [`README.md`](./README.md)). The body is intentionally in English: this page is
> reference material consumed by external contributors and agents. A claim absent
> from a rank-1 repo file is **`Not evidenced`** — the gate / packs are LIVE, the
> generative efficacy of the prompt-kind / copilot is not.

## Single source of truth

The CSP policy is one Rust constant. The machine mirror `csp-contract.json` and
the JS lint `check-csp.mjs` are **derived** from it and guarded by a drift test.
**This page is a convenience mirror** — if it ever disagrees with the const, the
const wins. The policy lives in
[`csp.rs`](../../crates/nexus-core-rs/src/csp.rs); the gate lives in
[`gates.rs`](../../crates/sbfb-factory/src/gates.rs).

## The CSP contract

`BLOB_SERVE_CSP` is injected on every blob-serve response (even 404). The
exfiltration-critical directives (value `'none'`), extracted by `none_directives`,
are exactly six:

| Directive | What it blocks for an authored app |
|---|---|
| `connect-src` | `fetch` / XHR / WebSocket / EventSource / `sendBeacon` — seed your RNG, ship assets locally |
| `worker-src` | `Worker` / `SharedWorker` / `importScripts` / ServiceWorker |
| `frame-src` | nested iframes (the `/auth/token` exfiltration vector) |
| `object-src` | `<object>` / `<embed>` |
| `base-uri` | `<base href>` hijack of relative URLs |
| `form-action` | `<form action>` exfiltration (blocked even where `allow-forms` is set) |

The only absolute URLs a scanned asset may carry (never fetched) are
`CSS_URL_ALLOW`: `http://www.w3.org/2000/svg`, `http://www.w3.org/1999/xlink`,
`https://tailwindcss.com` (the MIT license banner). Any other absolute http(s)
URL in a scanned asset trips the gate.

## Gate tiers

The gate `run_gate_csp_authoring` classifies each asset into one of three tiers
(mirrors the JS lint `check-csp.mjs`):

| Tier | Files | Rule |
|---|---|---|
| scanned source | `*.html`/`*.js`/`*.css`, not vendored, not a bundle | 0 network primitive + every absolute URL ∈ `CSS_URL_ALLOW` + no `<script type=module>` |
| vendored | `vendor/*` or `*.umd.js`/`*.min.js` | 0 network primitive only |
| skipped | everything else | — |

The gate is **non-delegable**: it runs outside the `--skip-gates` block (Day-0
"scellage 100% Factory"). The knowledge grants no exemption.

## Symbols (rank-1)

| Symbol | source_ref | Role |
|---|---|---|
| CSP policy | `crates/nexus-core-rs/src/csp.rs:BLOB_SERVE_CSP` | single source of truth |
| `'none'` extractor | `crates/nexus-core-rs/src/csp.rs:none_directives` | anti-drift directive set |
| URL allowlist | `crates/nexus-core-rs/src/csp.rs:CSS_URL_ALLOW` | non-fetched absolute URLs |
| CSP gate | `crates/sbfb-factory/src/gates.rs:run_gate_csp_authoring` | deterministic static scan |
| prompt-kind registry | `crates/sbfb-factory/src/process.rs:PROMPT_KINDS` | closed enum; entry `app-authoring` |
| context-pack handler | `crates/sbfb-factory/src/operator_server.rs:handle_context_pack` | emits `authoring_knowledge` |
| knowledge builder | `crates/sbfb-factory/src/operator_server.rs:authoring_knowledge` | hashed path refs, never inlined |
| starter template | `crates/sbfb-factory/src/template_engine.rs:DAISYUI_TEMPLATE` | vendored UMD + compiled `app.css` |

## Knowledge packs

| Pack | Version | Manifest | Integrity test |
|---|---|---|---|
| anime.js | 4.5.0 (snapshot 2026-06-23) | `docs/factory/knowledge/animejs/MANIFEST.json` | `crates/sbfb-factory/tests/animejs_manifest.rs` |
| daisyUI | 5.5.23 × Tailwind 4.3.1 | `docs/factory/knowledge/daisyui/MANIFEST.json` | `crates/sbfb-factory/tests/daisyui_manifest.rs` |

Each `MANIFEST.json` self-records per-layer blake3 16-hex digests; the integrity
test recomputes and asserts equality (a silent byte drift fails the build).
Freshness is **manual re-extraction at a version bump** — no auto-fetch
(`connect-src 'none'`).

## Runtime net (not a gate of record)

The static gate cannot see runtime-assembled code; the self-check viewer (Sprint
79 Phase H, [`../rust/PATTERNS.md`](../rust/PATTERNS.md) §P71) replays the app
under the real served CSP and captures browser-level violations. **Static lint ≠
runtime guarantee.** The self-check `status` is a *test* verdict, never a publish
authority (Day-0 "0 verdict PASS").

## Operator control-plane API (loopback)

*Sprint 80 frontier closure. The Operator control-center (greenfield front,
`tools/factory-operator/`) is a DISTINCT runtime that reads these loopback
routes over `127.0.0.1` — that actor makes them §6.12 frontier primitives.
They are NOT part of the sealed-iframe contract above: the Operator serves
outside `BLOB_SERVE_CSP` (its own minimal self-origin CSP). Mitigations for
this surface (T-OPERATOR-CSRF and friends) live in the
[threat model](../security/THREAT_MODEL.md) — single source, not duplicated
here. TS consumers:
[`streamChunk.ts`](../../tools/factory-operator/src/lib/streamChunk.ts),
[`useTokenStream.ts`](../../tools/factory-operator/src/lib/useTokenStream.ts),
[`operator.ts`](../../tools/factory-operator/src/api/operator.ts).*

### Auth bootstrap (cookie transport) — shipped `a5ace8d`

GET `/?token=<hex>` sits OUTSIDE `auth_required` (chicken-and-egg: reachable
before any cookie exists). It validates the bearer in constant time, then
mints `Set-Cookie: sbfb_operator=<session secret>; HttpOnly; SameSite=Strict;
Path=/` and answers **303 See Other** to `/` with `Referrer-Policy:
no-referrer` (the token leaves the address bar). The cookie carries a
**per-boot session secret** (`session_secret`) — **never the bearer**: the
root of trust stays the `x-sbfb-token` header (`AUTH_HEADER`). The cookie is
only accepted as a browser fallback transport (SSE/WS cannot set headers)
when `Sec-Fetch-Site: same-origin` is present (cross-port CSRF guard —
cookies are not port-scoped, RFC 6265). The response is IDENTICAL for
absent and wrong tokens (no oracle). The front never sets an auth header
itself (`credentials: 'same-origin'` lets the browser attach the cookie).
Anchors: `handle_bootstrap`, `OPERATOR_COOKIE`.

### GET /api/git/diff (working-tree) — shipped `bb35d39`

Restitutes the working tree computed **IN RUST** — never a JS-side diff
(kickoff invariant: one source of diff truth). Read-only, zero user input.
Envelope `{head, unstaged, staged, truncated}`: `head` = short HEAD sha (the
`run@<rev>` freshness anchor); `unstaged` = `git diff`, `staged` = `git diff
--cached` (a partially staged file legitimately appears in BOTH arrays — git
semantics); `truncated=true` past the line cap (cut at a line boundary).
Each file diff = `{path, insertions, deletions, hunks[]}`; each hunk =
`{header, lines[]}`; each line = `{kind: "add"|"del"|"ctx", content,
old_lineno, new_lineno}` where `old_lineno`/`new_lineno` serialize to
**null** when absent (the front Zod contract is `.nullable()`). Untracked
files are absent (not part of `git diff`). Anchor: `working_tree_diff_data`.

### GET /api/gates (live gate registry) — shipped `ed00b4a`

**1:1 read-only and idempotent** diagnostic: NO publish scan runs on this
GET (a side effect would break idempotence). Envelope `{gates: [...]}` with
**no aggregate root field** — no `overall`, no `all_passed`, no score. Each
entry = `{gate, status, issues[]}`; `status` is the `GateStatus` enum with
**exactly five** snake_case values: `not_run` / `not_applicable` / `passed`
/ `informational` / `blocking`. One gate can appear under SEVERAL statuses
(`lint-planning` splits errors→`blocking` and warnings→`informational`):
index by the **(gate, status)** key, never by gate alone. Cardinal
invariant: the Operator computes **no aggregate verdict** (0 UI-computed
verdict); the front restitutes 1:1 and never fabricates a PASS — acceptance
words (`PROVISIONAL`/`Not evidenced`/`RIG-ABSENT`) are NOT in the enum. Each
issue = `{message, file, line}`; `line` is **null as of S80** (fine-grained
line anchor is tracked debt). Anchors: `gates_live_data`, `GateStatus`.

### Chat SSE contract — shipped `6991d51`

`POST /api/chat/session` → `{id, context_pack}`, then `POST
/api/chat/{id}/send` (persists the turn's provider + model and applies the
MUR), then `GET /api/chat/{id}/stream` (bodyless; auth rides the same-origin
cookie). The stream emits `data: <compact json>\n\n` frames ONLY — no
`event:`/`id:`/heartbeat/keep-alive: **EOF is the end signal**. **Six wire
event types**: the five serde variants of `StreamChunk` (`delta`,
`thinking`, `done`, `error`, `debug`; tag `"type"`) plus **`requires_gate`,
hand-forged outside serde by `sse_gate`**. The MUR (`SENSITIVE_ACTIONS`)
runs BEFORE any dispatch: a sensitive message answers `requires_gate` and
never spawns an agent — a structural refusal (0 spawn), never a button.
PO-14 invariant: **exactly ONE `done`** (the Network arm carries a single
`done`, zero `delta`); the front latches the FIRST terminal event
`{done|error|requires_gate}` and ignores the rest, via `fetch +
ReadableStream + AbortController` — **never EventSource** (which would
reconnect and replay the turn). Seven front statuses (`StreamStatus`:
`idle`, `streaming`, `done`, `aborted`, `error`, `gate`, `ended` — a
front-internal state machine, one hop from the six wire types). The `debug` variant's `content` may carry the
assembled prompt verbatim — this page documents the SHAPE
(`{type:"debug", label, content}`), intentionally without a dump. Anchors:
`handle_chat_stream`, `sse_gate`, `StreamChunk`.

## See also

- Agent wiring spec: [`WIRING_SPEC.md`](./WIRING_SPEC.md).
- Gate spec: [`FACTORY_GATES.md`](./FACTORY_GATES.md).
- The authoring fiche: [`../../prompts/agent/app-authoring.md`](../../prompts/agent/app-authoring.md).
- Threat model: [`../security/THREAT_MODEL.md`](../security/THREAT_MODEL.md).
