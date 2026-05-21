# Sprint 67 Phase D — preflight G8

Date : 2026-05-21 | HEAD : `e8ee13f` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- feedback_approach.md : pick deepest, research before code,
  G8 = procedural mechanism for pick-deepest, OSS prior art
  obligatoire — SATISFIED (Phase D is a continuation of Phase C
  research grounding, provenance design from roadmap v3/v4 FG8).
- feedback_context7_systematic.md : context7 obligatoire avant
  code touchant lib/API — SATISFIED (blake3 context7 queried
  below, deps verified against lockfile).
- sprint14_keyoxide_decision.md : deploy from source, coordinator
  signs provenance.json SLSA L1 — N/A (factory.provenance.json is
  a DISTINCT file for local template lineage, not the daemon's
  SLSA provenance). No naming collision: daemon's provenance is in
  `nexus-coordinator-rs/src/provenance.rs`, factory's will be in
  `sbfb-factory/src/provenance.rs`.
- vision_model.md : OpenBSD solo maintainer, no startup — N/A
  (Phase D is pure internal tooling).
- nexus_grid_pivot.md : Factory hors daemon (D2 v4) — PRESERVED
  (provenance.rs is inside sbfb-factory crate, no daemon dep).
- Tensions plan vs memory : aucune.

## Scans (all clean)

- S1a OSS prior art : 5 projets recherches (Copier, cargo-generate,
  Backstage scaffolder, SLSA/in-toto, Gitleaks/Betterleaks),
  APPROACH-ALIGNED + APPROACH-NOVEL justified — clean
- S1b deps : 8 libs scannees, 0 delta — clean
- S2 historiques : 3 fichiers cibles, 4 commits bodies lus — clean
- S3 threat model : FULL, 5 vectors analyses — clean
- S4 wire format : FULL / VERSION=1, Day 0 preserved — clean

---

## S1a — OSS prior art deep analysis

### Projets analyses en profondeur

#### [Copier] — copier-org/copier (https://github.com/copier-org/copier)
- Fichiers source lus : docs/configuring.md (~500 LOC),
  docs/generating.md (~200 LOC), docs/faq.md (~100 LOC)
- Pattern architectural extrait : `.copier-answers.yml` records
  template version (`_commit: v1.0.0`) and all input variables.
  No hash-based integrity verification of generated files. No
  lockfile. Focus is on enabling template updates, not provenance
  auditing.
- Edge cases geres : template version tracking for update diffs,
  `vcs_ref` pinning for reproducible generation.
- Patterns abandonnes : none visible in releases.
- Verdict : ALIGNED (variable tracking pattern) / APPROACH-NOVEL
  justified (SBFB adds BLAKE3 output hash for P2P provenance
  chain that Copier doesn't need).

#### [cargo-generate] — cargo-generate/cargo-generate (https://github.com/cargo-generate/cargo-generate)
- Fichiers source lus : README.md (~300 LOC), templates/ docs
  (~200 LOC), cargo-generate.toml format (~100 LOC)
- Pattern architectural extrait : Uses Liquid template engine
  (Shopify), Rhai hooks, regex placeholders. `.genignore` for
  exclusions. Records `cargo-generate` version in Cargo.toml
  metadata for compatibility tracking. No hash of generated output.
  No provenance file.
- Edge cases geres : version compatibility warnings between
  template and cargo-generate version.
- Verdict : ALIGNED (template metadata tracking)

#### [Backstage scaffolder] — backstage/backstage (https://github.com/backstage/backstage)
- Fichiers source lus : scaffolder plugin CHANGELOG (~500 LOC),
  scaffolder backend router test (~200 LOC), authorizing template
  docs (~100 LOC)
- Pattern architectural extrait : Scaffolder emits audit log events
  when tasks are created (user ID, template ref, parameters).
  `commitHash` returned as output. Permission system for template
  actions. No hash-based output verification, no determinism test.
- Edge cases geres : secret leakage in logs (issue #30876),
  permission-based access control.
- Verdict : ALIGNED (audit log pattern, commitHash output)

#### [SLSA/in-toto provenance] — SLSA framework (https://slsa.dev)
- Fichiers source lus : SLSA provenance spec v1.0 (~500 LOC),
  in-toto bundle spec (~200 LOC), distributing provenance guide
  (~200 LOC)
- Pattern architectural extrait : SLSA provenance = in-toto
  statement with DSSE envelope. Subject = artifact + digest (SHA256).
  Predicate = buildType, builder, invocation, materials. Signed by
  build platform. Level 1 = self-attestation (local builder signs).
  Level 2+ requires hosted build platform.
- Key insight : factory.provenance.json is equivalent to SLSA L1
  self-attestation: the local tool (sbfb-factory) attests what it
  produced. The hash is the digest, the template+variables are the
  materials. No CI platform signature — that's S68+ publish path.
- Verdict : APPROACH-ALIGNED (self-attestation pattern matches
  SLSA L1 model)

#### [Gitleaks/Betterleaks] — gitleaks/gitleaks (https://github.com/gitleaks/gitleaks)
- Fichiers source lus : README (~200 LOC), patterns reference
  (regex-based detection, 150+ patterns)
- Pattern architectural extrait : regex-based secret detection,
  entropy analysis, allowlist system. Betterleaks (by same author)
  adds ahocorasick keyword filters and contextual rule filters.
- Key insight : sbfb-factory's secret_scanner.rs (3 patterns) is
  aligned with Gitleaks pattern but minimal. Adequate for MVP.
  Phase D does not modify secret_scanner.rs.
- Verdict : N/A (Phase D doesn't touch this)

### Tableau comparatif

| Aspect | Plan Phase D | Copier | cargo-generate | Backstage | SLSA L1 |
|--------|-------------|--------|---------------|-----------|---------|
| Template version tracking | factory.template.lock | .copier-answers.yml | Cargo.toml metadata | commitHash output | materials[] |
| Output hash | BLAKE3 output_hash | none | none | none | subject.digest |
| Variables hash | variables_hash | recorded in answers | via parameters | via audit log | invocation.parameters |
| Template hash | template_hash BLAKE3 | _commit version | version in toml | template ref | materials[].digest |
| Signature | optional (S68+) | none | none | audit log auth | DSSE envelope |
| Determinism test | explicit test plan | none | none | none | not applicable |

### Finding S1a

- Classification : APPROACH-ALIGNED + APPROACH-NOVEL justified
- Evidence : Copier tracks template version without hash
  (configuring.md), cargo-generate tracks version compatibility
  (README), SLSA L1 defines self-attestation with digest
  (slsa.dev/spec/v1.0/provenance). SBFB's factory.provenance.json
  combines these patterns with BLAKE3 workspace hash — novel but
  justified by the P2P publish chain (S68+ publish path needs a
  local hash as the first link in the provenance chain).
- Impact sur le plan : aucun

---

## S1b — Deps/libs versions + CVE

| Dep | Version lockfile | CVE/Advisory | Status |
|-----|-----------------|--------------|--------|
| blake3 | 1.8.3 | None found 2025-2026 | clean |
| serde_json | 1.0.149 | Fedora advisory patched at 1.0.145 | clean (1.0.149 > 1.0.145) |
| clap | 4.6.1 | None found 2025-2026 | clean |
| time | 0.3.47 | RUSTSEC-2026-0009 (DoS stack exhaustion RFC2822 parse, affects 0.3.6-0.3.46) | **PATCHED** (0.3.47) + usage is RFC3339 **formatting** only (template_lock.rs:23-24), not RFC2822 parsing |
| walkdir | 2.5.0 | None found 2025-2026 | clean |
| regex | 1.12.3 | None found 2025-2026 | clean |
| thiserror | 2.0.18 | None found 2025-2026 | clean |
| tempfile | 3.27.0 (dev-dep) | None found 2025-2026 | clean |

No new deps added by Phase D. BLAKE3 already in workspace. serde
already in sbfb-factory. No dep bump required.

Finding S1b : 0 delta — clean.

---

## S2 — Decision chain reconstruction

### Fichiers scannes

- `crates/sbfb-factory/src/main.rs` : 1 commit (49d6bcd Phase C)
- `crates/sbfb-factory/src/template_engine.rs` : 1 commit (49d6bcd)
- `crates/sbfb-factory/src/template_lock.rs` : 1 commit (49d6bcd)
- `docs/rust/PATTERNS.md` : 15 commits scanned, 3 bodies read
  in full (ea87547 S66B, cfa3c3c S60B, f3ea1c3 S66A)

### Decisions historiques trouvees

#### Decision 1 : BlobStore enum pattern (P2-66-2 carry)
- Sprint 66, sha `f3ea1c3` : introduced BlobStore enum (Mem/Fs)
  with Deref<Target=Store> in node.rs
  Body extrait : "enum BlobStore (Mem/Fs) avec Deref<Target=Store>.
  create_node_with_config cree un FsStore quand data_dir est set,
  MemStore sinon."
- Sprint 66, sha `3821508` : audit found P2-66-2 BlobStore
  pattern undocumented
  Body extrait : "P2-66-2 BlobStore pattern undocumented"
- Sprint 67, kickoff S6 carry table : P2-66-2 scheduled Phase D
- Reverse-commit check : N/A (no rejection to revert — this is
  an explicit carry request)
- Status : active carry, Phase D absorbs it
- Impact phase : aucun (plan explicitly includes P52 BlobStore
  documentation)

#### Decision 2 : Feed republish test gap (P2-66-1 carry)
- Sprint 66, sha `6467082` : feed republish at boot implemented
  Body extrait : "Republish des entries feed SQLite vers iroh-docs
  au boot pour garantir la coherence apres restart."
- Sprint 66, sha `3821508` : audit found P2-66-1 test limitation
  Body extrait : "test_feed_republish_at_boot verifie que le
  daemon boot sans panic et que feed_handle.is_some() mais ne
  verifie PAS que l'entry est effectivement presente dans iroh-docs"
- Sprint 67, kickoff S6 carry table : P2-66-1 scheduled Phase D
- Reverse-commit check : N/A (documentation carry, not a decision
  reversal)
- Status : active carry, Phase D absorbs it
- Impact phase : aucun (plan explicitly includes note in
  PATTERNS.md)

#### Decision 3 : Factory hors daemon (D2 v4 gelee)
- Roadmap v4 D2 : "Factory = outil client externe (crate
  sbfb-factory), hors daemon"
- Sprint 67 Phase C, sha `49d6bcd` : confirmed Factory as
  independent crate
  Body extrait : "Decision D2 v4 : Factory hors daemon, crate
  independant."
- Reverse-commit check : N/A (frozen Day 0)
- Status : active, frozen
- Impact phase : aucun (provenance.rs is inside sbfb-factory,
  zero daemon dep)

### Memory constraints

- feedback_approach.md : "Viser le long terme, pas le sprint.
  Documenter AVANT de coder." — Phase D is documentation-heavy
  (P52 pattern + provenance design), research pre-code satisfied
  by kickoff context7 + WebSearch.
- feedback_context7_systematic.md : "context7 obligatoire avant
  code touchant lib/API" — blake3 API confirmed via context7
  (/websites/rs_blake3), Hasher::new+update+finalize pattern
  matches existing template_lock.rs usage.
- sprint14_keyoxide_decision.md : "deploy from source" — factory
  provenance is a distinct concept (local lineage, not deploy
  attestation). No collision with daemon provenance.

---

## S3 — Threat model analysis

### Primitive analysee : factory.provenance.json

Local JSON file generated by `sbfb-factory create`, containing
BLAKE3 hashes of template content, input variables, and generated
workspace. Unsigned in S67 (Ed25519 signature deferred S68+).

### Assets en jeu

- A1 factory.provenance.json integrity : LOW (local file, no
  network exposure)
- A2 output_hash correctness : MEDIUM (used in S68+ publish chain
  as first link in provenance)
- A3 template_hash : LOW (verifies template version used)

### Threat actors

- TA1 local file tamperer : user or malware modifies provenance
  file post-generation
- TA2 path traversal attacker : crafted template variables to
  escape output directory (already mitigated by Phase C validate)

### Attack vectors identifies

1. V1 Provenance forgery (TA1) : attacker modifies output_hash
   in factory.provenance.json to match a different workspace.
   Asset : A2. Coverage : NOT covered by T0-T5 (local file, not
   network-exposed). Severity : LOW pre-S68 (local only), MEDIUM
   S68+ (when publish path uses this hash). Mitigation S68+ :
   Ed25519 signature by daemon keypair at publish time.

2. V2 Determinism violation : non-deterministic output causes
   different hashes for same inputs, breaking idempotence claim.
   Asset : A2. Coverage : mitigated by test plan
   (test_provenance_hash_deterministic). Severity : LOW.
   **Implementation note** : `factory.template.lock` contains
   `generated_at` timestamp. If this file is included in
   output_hash computation, determinism breaks. The hash must
   exclude timestamp-bearing files (lock + provenance itself) or
   hash only the content files.

3. V3 DoS via large workspace : attacker provides template that
   generates very large output, slow BLAKE3 hash. Asset : N/A
   (local CLI). Severity : negligible (user runs on own machine).

4. V4 Supply chain (dep compromise) : BLAKE3 crate compromised.
   Asset : A2. Coverage : Cargo.lock pinned, blake3 1.8.3 no
   known CVE. Severity : LOW.

5. V5 Replay/reorder : N/A (no network message, local file only).

### Mitigations existantes

- Path traversal : validate() rejects `..` components (Phase C)
- Symlink : validate() rejects symlinks (Phase C)
- Secret scanner : validate() detects AWS/GitHub/PEM patterns
- BLAKE3 hash-chain : template_lock already hashes template
  content deterministically (sorted by filename)

### Gaps identifies

- GAP1 V1 unsigned provenance : severity LOW (S67 local-only).
  Documented in plan: "Signature Ed25519 optionnelle S67, S68+
  publish path." Acceptable pre-S68.
- GAP2 V2 determinism depends on implementation : severity LOW.
  Implementation note surfaced below. Not a gap in the plan, but
  a detail the coder must handle.

### Regression check

- Phase D does NOT diminish any existing T0-T5 mitigation
- Phase D does NOT create a new network-exposed attack vector
- No new T section needed in THREAT_MODEL.md for S67

### Verdict S3 : clean (0 regression, 2 gaps LOW severity both
documented/expected)

---

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui (296 lines)

Phase D does NOT touch canonical.rs. No new struct is added to the
canonical signing surface. factory.provenance.json is a local file,
not a signed wire format payload.

### Structs verifiees

Phase D creates a new `Provenance` struct in sbfb-factory crate.
This struct is NOT in canonical.rs, NOT signed via
`canonical_bytes()`, NOT part of the wire format. It is a local
JSON file generated by the CLI tool.

No existing struct is modified by Phase D.

### Version constants verified

| Constant | Value | Touched by Phase D |
|----------|-------|--------------------|
| FEED_FORMAT_VERSION | 1 | NO |
| TASK_FORMAT_VERSION | (in task.rs) | NO |
| CURATOR_LIST_FORMAT_VERSION | 1 | NO |
| KEY_ROTATION_FORMAT_VERSION | 1 | NO |
| INVITE_FORMAT_VERSION | 2 | NO |
| PROVENANCE_SCHEMA_VERSION | 1 | NO |
| SCHEMA_VERSION (state_writer) | 1 | NO |

### Day 0 check

- D1 FTS5 search : not touched by Phase D — OK
- D2 sbfb-manifest crate : not touched by Phase D — OK
- D3 CuratorVouched/Disendorsed : not touched by Phase D — OK
- D4 Feed entries read paginee : not touched by Phase D — OK
- D5 sbfb-factory CLI create + validate : Phase D EXTENDS D5
  with provenance generation — OK (additive, not contradictory)

Decisions actees pivot.md :
- Factory hors daemon (D2 v4) : PRESERVED (provenance.rs in
  sbfb-factory, no daemon dep)
- Feed raw-op extensible (D4 v4) : not touched — OK
- Pre-launch protocol : not touched — OK

### Pre-launch policy

- No *_VERSION bump
- No tolerant decoder multi-version
- No tests "legacy decode"
- factory.provenance.json is a local file, not a wire format

---

## Implementation note (non-blocking)

The `output_hash` determinism test requires careful scoping of
which files are hashed. The `factory.template.lock` file contains
a `generated_at` RFC3339 timestamp that changes between runs. If
`factory.template.lock` and/or `factory.provenance.json` are
included in the workspace hash, `output_hash` will NOT be
deterministic across invocations with the same inputs.

**Recommendation** : compute `output_hash` over the content files
only (index.html, sbfb-bridge.js, SBFB.json, README.md,
.gitignore) EXCLUDING factory.template.lock and
factory.provenance.json. Alternatively, compute output_hash BEFORE
writing the lock/provenance files. The test
`test_provenance_hash_deterministic` should call `create()` twice
with the same inputs and assert `output_hash` equality.

This is a standard implementation detail, not a plan deviation.

---

## Telemetrie preflight (agent deep)

- Duree totale : ~15 minutes
- S1a : 5 projets OSS analyses (Copier, cargo-generate, Backstage,
  SLSA/in-toto, Gitleaks) / 12 fichiers source lus / ~2500 LOC
  reviewees / 2 context7 queries (blake3 resolve + query) / 8
  WebSearch queries / finding : APPROACH-ALIGNED + APPROACH-NOVEL
- S1b : 8 libs scannees / 5 CVE searches (blake3, walkdir, clap,
  serde_json, time) / finding : 0 delta, RUSTSEC-2026-0009 patched
- S2 : 4 commits bodies lus in full / 1 archive file (S66 audit
  findings) / 5 memory files (approach, context7, pivot, vision,
  keyoxide) / finding : clean (3 decisions tracked, all aligned)
- S3 : FULL / 5 vectors analyses / 2 gaps LOW (both documented/
  expected)
- S4 : FULL / 0 structs to verify (Phase D creates new struct in
  sbfb-factory, not in canonical.rs) / canonical.rs lu
  integralement : oui / 7 version constants verified : all = 1,
  none touched

## Action

Proceder code phase D.
