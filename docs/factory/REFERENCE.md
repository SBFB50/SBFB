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

## See also

- Agent wiring spec: [`WIRING_SPEC.md`](./WIRING_SPEC.md).
- Gate spec: [`FACTORY_GATES.md`](./FACTORY_GATES.md).
- The authoring fiche: [`../../prompts/agent/app-authoring.md`](../../prompts/agent/app-authoring.md).
- Threat model: [`../security/THREAT_MODEL.md`](../security/THREAT_MODEL.md).
