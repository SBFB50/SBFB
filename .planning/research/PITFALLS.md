# Domain Pitfalls

**Domain:** P2P app factory, broker sandbox, domain-specific app generation
**Researched:** 2026-05-18

## Critical Pitfalls

### Pitfall 1: Factory Becomes Protocol Business Logic
**What goes wrong:** Factory-specific methods (factory_shell_exec, factory_write_file)
added to bridge protocol or coordinator crate.
**Why it happens:** Convenience — bridge already dispatches to coordinator.
**Consequences:** Protocol loses neutrality. Every non-Factory app inherits Factory deps.
**Prevention:** Factory = daemon module or crate, never in coordinator or bridge. Uses HTTP
routes /api/v1/factory/*, never bridge methods. Code review gate: any PR adding "factory"
to bridge/protocol.ts or coordinator-rs is flagged.
**Detection:** `rg "factory" web/src/bridge/protocol.ts` or
`rg "factory" crates/nexus-coordinator-rs/` finding non-comment matches.

### Pitfall 2: SBFB.json v2 Breaks Existing Apps
**What goes wrong:** Enriched manifest v2 made mandatory. Explorer and Ideas Hub fail deploy.
**Why it happens:** Forgetting backward compat when schema_version is absent in old manifests.
**Consequences:** Deploy pipeline breaks for every existing app.
**Prevention:** schema_version absent = v1. schema_version: 1 = v1. schema_version: 2 = new.
deploy.rs parses both. Migrate existing apps to v2 in Phase A, not as prerequisite.
**Detection:** Deploy E2E test with old SBFB.json (3 fields only) must still pass.

### Pitfall 3: Broker Path Traversal
**What goes wrong:** Bug in workspace path validation allows writing outside workspace.
**Why it happens:** String comparison instead of canonicalize(). Symlinks not handled.
Windows backslash not normalized.
**Consequences:** Arbitrary file write on host. Security incident.
**Prevention:** std::fs::canonicalize() + prefix check BEFORE any write. Same rigor as
validate_zip_path() in blob_serve.rs. Tests: ../etc/passwd, ..\\Windows\\System32, symlinks.
**Detection:** Tests must reject all traversal patterns. CI gate.

### Pitfall 4: Preview Diverges from Production
**What goes wrong:** Factory preview serves app differently than production (different CSP,
sandbox, base path). Apps pass preview but fail in production.
**Why it happens:** Custom preview server or relaxed sandbox for "convenience."
**Consequences:** False sense of security. Trust model undermined.
**Prevention:** Preview MUST use exact same pipeline: zip -> blob-serve -> iframe
sandbox="allow-scripts" with CSP connect-src 'none'. Zero exceptions.
**Detection:** E2E test: preview headers == production headers.

## Moderate Pitfalls

### Pitfall 5: Templates Too Rigid
**What goes wrong:** Templates impose heavy structure (linting, CI, monorepo) that doesn't
fit user needs. User spends more time removing boilerplate than coding.
**Prevention:** Minimal templates: index.html + app.js + style.css + sbfb-bridge.js +
SBFB.json. The simplest template (static-minimal) should be <100 LOC.

### Pitfall 6: NLLB-200 Backend Not Ready for S75
**What goes wrong:** S75 depends on functional NLLB-200 worker not yet built.
**Prevention:** Babel MVP uses fixture translations (pre-translated JSON). Traduction via
task_submit is stretch goal. Factory -> app -> deploy pipeline validated without NLP infra.

### Pitfall 7: Domain Pack Scope Creep
**What goes wrong:** Babel domain pack tries full vision (corpus pipeline, multi-source,
1500+ languages) instead of minimal reader.
**Prevention:** S75 Babel scope: reader + 3 fixture texts + 5 languages + storage progression
+ traduction mock. Full Babel is post-S75.

### Pitfall 8: Audit Log as Security Theater
**What goes wrong:** Factory writes audit logs nobody reads. No query tool.
**Prevention:** Start minimal JSONL. Add `sbfb factory audit` CLI. Show last 5 actions
on /factory page.

## Minor Pitfalls

### Pitfall 9: CLI vs UI Confusion
**What goes wrong:** `sbfb create` (CLI) and /factory page (UI) have different capabilities.
**Prevention:** Both call same Template Engine function. CLI = thin wrapper. UI adds diff
preview. Same engine, same output.

### Pitfall 10: Template Git Drift
**What goes wrong:** Templates updated in Git but factory.template.lock in projects still
references old hash.
**Prevention:** factory.template.lock records template id, version, BLAKE3 hash.
`sbfb factory check-template` compares against current version.

### Pitfall 11: French-Only Templates
**What goes wrong:** Templates generate French UI (per CLAUDE.md) but external contributors
expect English.
**Prevention:** Documented in template.json metadata. i18n = template variant later.

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| S73 Phase A (SBFB.json v2) | P2: break existing apps | schema_version compat, deploy E2E test |
| S73 Phase B (Template engine) | P5: templates too rigid | Minimal templates, <100 LOC static-minimal |
| S73 Phase C (CLI) | P9: CLI vs UI divergence | Same engine function called by both |
| S74 Phase A (Broker routes) | P3: path traversal | canonicalize + prefix + traversal tests |
| S74 Phase C (Review UI) | No major pitfall | Standard React page pattern |
| S74 Phase D (Publish gate) | P4: preview != production | Same blob-serve pipeline, header E2E |
| S75 Phase A (Domain packs) | P7: scope creep | Babel = reader + fixtures only |
| S75 Phase C (Bridge) | P6: NLLB not ready | Fixture translations as fallback |
| S75 Phase D (Deploy) | No major pitfall | Uses existing deploy-from-repo |

## Sources

- Path traversal in blob_serve.rs: validate_zip_path function
- SBFB.json parsing in deploy.rs
- VS Code trust bypass: https://www.ox.security/blog/can-you-trust-that-verified-symbol-exploiting-ide-extensions-is-easier-than-it-should-be/
- Developer trust in AI code: https://edmondscommerce.co.uk/research/ai/developer-trust/
- Internal: `.planning/research/sbfb_project_factory_rrv_oss_research.md` (section 13)
