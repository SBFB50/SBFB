# Factory app-authoring — agent wiring spec (contract-dense, source-anchored)

> **Audience: an LLM / agent that must author, wire, or review a sealed-iframe
> SBFB app (anime.js + daisyUI) without hallucinating.** This is the
> machine-actionable contract layer above the human docs
> ([`EXPLANATION.md`](./EXPLANATION.md), [`HOW_TO_WIRE.md`](./HOW_TO_WIRE.md),
> [`REFERENCE.md`](./REFERENCE.md)) and the gate spec
> ([`FACTORY_GATES.md`](./FACTORY_GATES.md)). Every contract clause carries a
> **source_ref** of the form `path:Symbol` to a rank-1 repo file; verify the ref
> (grep the symbol) before you act on the claim.

## 1. Authority — Truth Stack

When this spec and any other source disagree, trust the higher rank:

```
repo files > .planning/active/ > commits > prompts > chat
```

- **Rank-1 = repo files** (`crates/`, `docs/`, `web/`, `scripts/`). The only
  authority. Every contract clause here cites one as `path:Symbol`.
- **Rank-2 = `.planning/active/`** is an *in-flight pointer* — archived when the
  sprint closes, so never treat an `.planning/active/...` path as a durable
  anchor. (The source-ref-check resolves only rank-1 paths.)
- Lower ranks (commit bodies, prompts, chat) are context, not contract.
- **Rule: a fact absent from rank-1 is `Not evidenced`** — do not assert it. If
  you cannot point at a repo file, say so rather than inventing a symbol.

> **Status: PROVISIONAL where it counts.** The CSP authoring gate, the two
> knowledge packs, and the daisyUI starter template are **LIVE and hermetically
> tested**. What is **PROVISIONAL / `Not evidenced`**: the end-to-end in-vivo
> journey (a real author → publish → cross-peer render) and the *generative
> efficacy* of the prompt-kind / Ollama copilot — the plumbing is wired and
> tested, but no real LLM-authored app has been measured in-vivo. Never document
> this path as "shipped" / "LIVE in production".

> **Caveat cardinal — lint statique ≠ garantie runtime ; connaissance consommée,
> jamais autoritaire.** The CSP gate is a deterministic static scan of *delivered*
> assets (assumed false-negatives: `fetch` via `atob`, `form.action`/`base.href`
> built at runtime); the runtime net is the self-check viewer (Sprint 79 Phase H).
> The knowledge (packs, prompt-kind) is *consumed and displayed*, **never
> authoritative**: **0 verdict PASS**, `crates/sbfb-factory/src/operator_server.rs:chat_history_authoritative`
> stays `false`. The code (`nexus_core_rs::csp`) and the deterministic gates
> decide. See [`../security/THREAT_MODEL.md`](../security/THREAT_MODEL.md).

## 2. The sealed-iframe contract (single source of truth)

An SBFB app renders inside a sealed iframe: `sandbox="allow-scripts"` **without**
`allow-same-origin` (opaque/null origin), under the daemon-injected CSP/COOP/COEP.
The **single, canonical** policy string is
`crates/nexus-core-rs/src/csp.rs:BLOB_SERVE_CSP` — re-exported from
`crates/nexus-core-rs/src/lib.rs` and mirrored, verified-by-test, in
`crates/nexus-core-rs/csp-contract.json`. **Never re-hardcode it; import it.**

The exfiltration-critical directives (value `'none'`) are extracted
deterministically by `crates/nexus-core-rs/src/csp.rs:none_directives`, which for
`BLOB_SERVE_CSP` returns exactly:

```
connect-src · worker-src · frame-src · object-src · base-uri · form-action
```

The only absolute URLs a scanned asset may carry (never fetched: XML namespaces +
the MIT license banner) are `crates/nexus-core-rs/src/csp.rs:CSS_URL_ALLOW`.

Runnable proof of this contract:
[`examples/csp_contract.rs`](./examples/csp_contract.rs), compiled+run by
`crates/nexus-core-rs/tests/factory_csp_contract.rs` (a value drift fails the
test, an API drift fails the build). It targets the **library** API
`nexus_core_rs::csp`, not the gate: the
gate lives in the binary-only `sbfb-factory` crate (no lib target) and cannot be
lifted into a `use`.

## 3. Per-primitive contract

Each row lists **source_ref** (`path:Symbol`, grep-resolvable) · what it does ·
caps / preconditions.

### GATE — deterministic static CSP scan (publish-time, non-delegable)
- source_ref `crates/sbfb-factory/src/gates.rs:run_gate_csp_authoring`.
- Scans the authored workspace in **three tiers** (scanned source / vendored /
  skipped): zero network primitive in every tier; every absolute URL in a scanned
  asset ∈ the imported `CSS_URL_ALLOW`; no `<script type=module>`. The detection
  rules (`CSP_RULES`) are hand-written, but the policy is **not re-hardcoded**: a
  cross-crate `#[cfg(test)]` anti-drift test asserts the scanner covers every
  `'none'` directive of `BLOB_SERVE_CSP` (via `none_directives`), so a new blocked
  directive fails the build until a matching rule is added.
- **Non-delegable**: it runs **outside** the `--skip-gates` block (Day-0
  "scellage 100% Factory"); no CSP exemption is possible, the knowledge grants
  none. Detail + assumed false-negatives:
  [`FACTORY_GATES.md`](./FACTORY_GATES.md) (FG-CSP-authoring).
- **Runtime net** (complement, not replacement): the self-check viewer replays
  the app in the real prod iframe-host under the served CSP (Sprint 79 Phase H,
  [`../rust/PATTERNS.md`](../rust/PATTERNS.md) §P71). Static lint ≠ runtime proof.

### KIND — the portable `app-authoring` prompt-kind
- source_ref `crates/sbfb-factory/src/process.rs:PROMPT_KINDS` — the closed
  registry of prompt-kinds; the 9th entry is the string
  `crates/sbfb-factory/src/process.rs:app-authoring`, which resolves through the
  generic filename arm to [`../../prompts/agent/app-authoring.md`](../../prompts/agent/app-authoring.md)
  (no alias, no special case).
- Drift gate: `crates/sbfb-factory/src/process.rs:prompt_kinds_resolve_to_existing_files`
  fails the build if the fiche file is absent.
- The fiche is **consumed and displayed, never authoritative** (§1 caveat).

### CONTEXT-PACK — surfacing the knowledge to a fresh session
- source_ref `crates/sbfb-factory/src/operator_server.rs:handle_context_pack`
  (route registered at `/api/context-pack`) emits an `authoring_knowledge` array
  built by `crates/sbfb-factory/src/operator_server.rs:authoring_knowledge` from
  the single edit-point `crates/sbfb-factory/src/operator_server.rs:AUTHORING_KNOWLEDGE_MANIFESTS`.
- The pack manifest is surfaced as a **hashed path reference** (model
  `process_docs`), **never inlined**. The single edit-point
  `AUTHORING_KNOWLEDGE_MANIFESTS` lists the **animejs MANIFEST only at this
  revision** (`docs/factory/knowledge/animejs/MANIFEST.json`); the daisyUI pack
  (`docs/factory/knowledge/daisyui/MANIFEST.json`) lives in the same tree and is
  hashed by its own test (§KNOWLEDGE) but is **not yet emitted** in the
  `authoring_knowledge` array. The packs live outside any app workspace ⇒ 0 impact
  on the app provenance/FG6.
- The same context-pack carries `chat_history_authoritative` = `false`.

### TEMPLATE — the daisyUI + anime.js starter
- source_ref `crates/sbfb-factory/src/template_engine.rs:TemplateConfig`; the
  daisyUI entry is `crates/sbfb-factory/src/template_engine.rs:DAISYUI_TEMPLATE`
  (vendored UMD + build-time compiled `app.css`, 0 runtime dependency).
- Vendorization doctrine: **UMD classic `<script>`, never `type=module`** —
  `connect-src 'none'` + an opaque origin under COEP `require-corp` make every
  remote/ESM import impossible.

### KNOWLEDGE — the two versioned packs
- `docs/factory/knowledge/animejs/MANIFEST.json` (anime.js 4.5.0) +
  `docs/factory/knowledge/daisyui/MANIFEST.json` (daisyUI 5.5.23 × Tailwind
  4.3.1). Each self-records per-layer blake3 16-hex digests.
- Integrity is a verified-by-recompute test:
  `crates/sbfb-factory/tests/animejs_manifest.rs:animejs_manifest_hashes_match_promoted_layers`
  + `crates/sbfb-factory/tests/daisyui_manifest.rs` — a silent byte drift fails
  the build. Freshness is **manual re-extraction at a version bump** (no
  auto-fetch; `connect-src 'none'`).

## 4. INVIOLABLE invariants

Violating any of these is a security/contract defect, not a style nit:

1. **Single CSP source of truth.** Import `BLOB_SERVE_CSP`
   (`crates/nexus-core-rs/src/csp.rs:BLOB_SERVE_CSP`); never re-hardcode the
   policy nor read a stale code comment.
2. **The gate is non-delegable.** `run_gate_csp_authoring` runs outside
   `--skip-gates`; the knowledge accords no CSP/COEP/COOP/Ed25519 exemption.
3. **Knowledge is consumed, never authoritative.** 0 verdict PASS;
   `chat_history_authoritative` stays `false`; the artifact-draft is anti-PASS.
4. **Vendored same-origin, never networked.** UMD classic script, no ESM/CDN, no
   remote `url()` outside `CSS_URL_ALLOW`. Zero outgoing request at runtime.
5. **Process asset, never in the app archive.** The packs live under `docs/`
   (hashed by the manifest tests), never inside a published app workspace.
6. **Each claim carries a source_ref.** This document, and any agent acting on
   it, cites a rank-1 file for every contract clause; an unanchored claim is
   `Not evidenced` (§1).

## See also

- Gate spec (the 11 Factory gates + FG-CSP-authoring detail):
  [`FACTORY_GATES.md`](./FACTORY_GATES.md).
- Human docs (Diátaxis): [`README.md`](./README.md),
  [`EXPLANATION.md`](./EXPLANATION.md), [`HOW_TO_WIRE.md`](./HOW_TO_WIRE.md),
  [`REFERENCE.md`](./REFERENCE.md).
- Index for agents: [`llms.txt`](./llms.txt).
- The authoring fiche an agent actually receives:
  [`../../prompts/agent/app-authoring.md`](../../prompts/agent/app-authoring.md).
- CSP source of truth: [`csp.rs`](../../crates/nexus-core-rs/src/csp.rs).
- Threat model + runtime patterns:
  [`../security/THREAT_MODEL.md`](../security/THREAT_MODEL.md),
  [`../rust/PATTERNS.md`](../rust/PATTERNS.md) §P71.
