# Sprint 76 Phase B Review

## Verdict: PASS

> Driver-side deep review (nexus-phase-review-deep, 1M ctx, independent process)
> RECONCILED with Codex (GPT 5.5 cross-model). HEAD `d6dea45`. Phase = dette
> reservee non convertible (sprint PAIR, Regle 1 G7). 0 P0, 0 P1, 2 P2, 2 P3.
> Rigor G4 honored (>= 1 discutable trade-off found).

## Codex reconciliation

Codex `codex exec -o` brut : `sprint76_phase_b_codex_review.md` (output GPT 5.5,
NON reecrit). Verdict : **12/12 livrables CONFIRME, 0 GAP, 0 PARTIEL**.
Chaque livrable corrobore independamment file:line par Codex :
- B1 duress : early-return AVANT mutation/emit confirme (`http.rs:1543`, `:2097`),
  bytes leurre == succes normal (`:1548`==`:2285`), tests asserent zero row +
  zero tag (`:6016-6024`, `:6056-6064`).
- B1 observed : aucun code prod binding PoW, doc ne sur-promet pas (`iroh_runtime.rs:2352`).
- B2 : downgrade AVANT `add_direct_entry` (`runtime.rs:2299`/`:2335`), helper bool.
- B3 : vraie chaine boucle (pas selection), mismatch delete_tag avant tier suivant
  (`:2288-2292`), codes 400/404/502 preserves.
- B4/B5/B6/B7/B8/B10/B11 : CONFIRME (cf. artefact). Invariant pre-launch : 0
  fichier wire/canonical, 0 FORMAT_VERSION change.

**Aucun GAP P0/P1 → aucune boucle de correction requise.** Suites NON re-runnees
apres Codex (0 changement de code post-review : Codex n'a trouve aucun fix a
porter). Findings P2/P3 (review Claude) reportes au commit body, non-bloquants :
- [P2] B10 parite = 2 miroirs hand-maintained (route fixture partagee S77+).
- [P2] B3 directory tier resolu EAGER sur happy-path (trade-off assume).
- [P3] preflight untracked -> committe avec la phase. [P3] decoy set_keep_online
  200 vs rare 500 -> negligeable, erring-benign correct.

## Scope And Staging

Atomic phase commit, coherent. `git status --short` separates cleanly:
- **Phase code/test** : `http.rs`, `runtime.rs`, `iroh_runtime.rs`, `sbfb-manifest/lib.rs`,
  `Nodes.tsx`, `Nodes.test.tsx`, `protocol.test.ts`, 4 new page smoke tests
  (`Curators/OnboardingEmpty/ProjectDetail/Projects.test.tsx`), `vitest.config.ts`,
  `package.json`, `ci.yml`.
- **Phase doc** : `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` (B7), `THREAT_MODEL.md` (B1/B2/B8),
  `sprint76_audit_plan.md` (B7/B11).
- **Preflight artifact** : `sprint76_phase_b_preflight.md` (untracked, G8 evidence — keep
  out of the code commit or include with planning; not a code file).

No `pub mod` added (`git diff --cached -- '*.rs' | rg '^\+pub mod '` = none — and the diff
is unstaged; the auditor should `git add` the phase set before the final gate). No build
output, no cache, no accidental file. The audit_plan + THREAT_MODEL + LOOPBACK edits are
the phase's documented deliverables (B7/B8/B11), not unrelated planning drift, so they
belong in this atomic `fix(daemon+shell)` commit.

## Three-Block Verification

Re-run independently this review (not memory):
- **Rust** `cargo nextest run -p nexus-shell-daemon -p nexus-shell-daemon-core -p sbfb-manifest --locked`
  → **672 passed, 0 skipped** (23.6s). The full-workspace 1775 claim is plausible and the
  touched-crate subset is green; auditor should confirm the full-workspace number + clippy
  `-D warnings` + fmt + doc + release build (the driver claims all green; not independently
  re-run here for the full workspace).
- **Web** `npm run test:unit` → **396 passed (37 files)**, exit 0 (386 → 396, +10 EXACT).
  `bash scripts/scan-en-strings.sh` → "src/ is French-only, clean", exit 0.
- Driver claims coverage 87.2/79.01/85.92/88.52 (>= 85/78/85/85), build/size/lint/tsc green
  — consistent with the vitest.config honesty note (route pages stay OUT of `coverage.include`,
  smoke = regression not measurement). Not independently re-run here; reconcile at Codex.

No `--no-verify`, no `#[ignore]`, no `xfail`, no suite skipped.

## Delta Tests

Plan: +~10 Rust / +~7 Vitest. Actual reconciles EXACTLY to the driver's +8 / +10 claim:
- **Rust +8** : `http.rs` 3 (`set_keep_online_noop_in_duress`, `seed_voluntary_noop_in_duress`,
  `pull_falls_back_across_tiers_when_ticket_dead`) ; `runtime.rs` 3
  (`aggregator_downgrades_open_source_without_provenance`, `gossip_cmd_outbox_persists_to_db`,
  `endpoint_addr_hoisted_once_per_pass`) ; `iroh_runtime.rs` 1
  (`observed_capture_is_availability_only`) ; `sbfb-manifest` 1
  (`allowlist_mirrors_host_dispatch_schema`).
- **Vitest +10** : Curators 2 + ProjectDetail 2 + Projects 2 + OnboardingEmpty 1 (= 7 page
  smoke) + protocol parity 2 + Nodes B6 1.

Plan test #3 (`observed_capture_bound_to_publisher_pow`) and #9
(`blobserve_mitigation_cell_matches_impl`) were re-cut at preflight: #3 became
`observed_capture_is_availability_only` (decision (b), honest — no false "PoW binding"
test), #9 became doc-only (B8 is a doc correction, no behavior to assert in code). Plan
test #10 `network_page_smoke_renders` dropped (Network already tested, `e980d7e`); scope
5 → 4 pages, preflight-acknowledged. All re-cuts are honest narrowings, not masked debt.

No suite with 0 delta where one was expected. Rust/web deltas are real new behavior,
not stubs.

## Modified-File Branch Coverage

Semantic (read each test in full, not grep-name):
- **B1 duress no-op** (http.rs early-returns) : `set_keep_online_noop_in_duress` +
  `seed_voluntary_noop_in_duress` prove ZERO mutation (`get_keep_online == None`) AND ZERO
  blob tag (`!has_tag`) under `IdentityMode::Duress`, with the app made visible first so a
  NON-duress path WOULD have written. This is the strongest possible assertion of the
  short-circuit (proves the negative). Decoy 200 response bytes match the real success
  bytes (1548==1593, 2102==2284) → indistinguishable, benign. Both sides of the duress
  branch covered (the existing non-duress paths have prior coverage).
- **B2 ingress downgrade** : `aggregator_downgrades_open_source_without_provenance` drives
  the REAL `handle_project_announcement` with a byzantine `is_open_source:true`/null-prov
  announcement (downgraded to false at `/browse` ingress) AND an honest full-provenance
  announcement (preserved). Both branches of `trustworthy_open_source` exercised at the
  real chokepoint.
- **B3 cross-tier chain** : `pull_falls_back_across_tiers_when_ticket_dead` asserts chain
  ORDER (ticket first, directory second), single-tier shapes, and empty chain. The handler
  loop (real fetch) is integration-tested by LIVE WAN acceptance, not in-process; the
  pure `build_seed_fetch_chain` is the unit-testable core and is fully covered. Error-code
  preservation (400/404/BAD_GATEWAY) verified by reading the `chain.is_empty()` block —
  pre-B3 disambiguation kept verbatim.
- **B4 outbox** : `gossip_cmd_outbox_persists_to_db` (multi_thread, real gossip task)
  asserts the `GossipCmd::Outbox` handler persists the unwrapped announcement to the DB
  outbox — the deterministic neighbor-independent half. Honestly scoped (the broadcast
  half is WAN-acceptance, documented).
- **B5 hoisting** : `endpoint_addr_hoisted_once_per_pass` is the standout — it hands
  `remint_and_wrap_for_replay` OTHER's address and asserts the re-minted ticket embeds
  OTHER's endpoint id, PROVING the passed address is used (not re-fetched per entry).
  This is a real semantic proof of the refactor's correctness, not a grep. All 4 replay
  call-sites + the NeighborUp single-payload path are gated by `current_replay_addr` →
  equivalent outcome (0 broadcast when address unavailable, one log instead of N).
- **B6 discriminator** : `b6 : distingue un curateur...` exercises BOTH `data-kind`
  branches (curator-in-entries vs anchor-not-in-entries) with realistic byte-array
  fixtures matching `bytesToHex`. The prior assertion was updated (anchor copy), not
  deleted blindly.
- **B10 parity** : both-sided (Rust `allowlist_mirrors_host_dispatch_schema` + TS
  `BridgeMethodSchema parity`), each pinning the same 15-method canonical set and
  validating real manifests/parses.

No untested business logic > ~10 LOC. B7/B8/B11 are doc-only (no code branch).

## Security And Protocol

Deep pass on every touched sensitive surface:
- **B1 duress** : routes through the canonical `noop_identity::gossip_publish_in_duress` /
  `PublishOutcome::Noop` gate — the exact module whose doc-comment (noop_identity.rs:38-41)
  invites all duress-sensitive routes through it. Consistent with the P1 fix `23a08c9` and
  `run_boot_seed_driver`. The single early-return covers BOTH local mutation (pin/tag/M18
  row) AND the `SeedAnnounced` wire-emit (verified: early-return is BEFORE the
  `emit_seed_announced` call at http.rs:2206-equivalent). No SUR-PROMISE: B1 observed
  decision (b) documents availability-only honestly; the test
  `observed_capture_is_availability_only` PINS that the registry is not publisher-auth, and
  THREAT_MODEL §15.1 + the test comment both refuse to claim a PoW binding the layer cannot
  provide. Correct restraint.
- **B2 trust boundary** : the downgrade is now at the REAL shared chokepoint
  (`handle_project_announcement`, BEFORE `add_direct_entry`), so the served `/browse` card —
  not only the FTS5 index — reflects the downgrade. Same predicate (`trustworthy_open_source`)
  shared with `index_browse_entry`. Wire SHAPE unchanged (`is_open_source` bool stays); only
  the computed VALUE changes at ingress. Front verrou-4 (`source=="direct"` + flag) is
  correctly documented as DECLARATIVE trust, not a crypto attestation (THREAT_MODEL new row).
- **B3** : cross-tier failover resolves the directory tier EAGER even on happy-path (RAM
  snapshot + providers cost, documented). Behavior change is a STRICT improvement (dead
  ticket → directory fallback instead of terminal BAD_GATEWAY). No double-pin: a hash-mismatch
  tier calls `delete_tag` before continuing; the winning tier re-applies the tag. Error
  codes preserved.
- **B8** : THREAT_MODEL cell corrected — `/blob-serve` is PUBLIC by construction (an
  `allow-scripts` sandboxed iframe cannot carry the bearer for its assets); amplification
  bounded by subscribed-only + `MAX_FETCH_PROVIDERS`/cap + timeout, never a bearer. Honest
  correction of a false prior claim.
- **B10** : declarative manifest allowlist, NOT the dispatch/sandbox boundary — correctly
  documented on both sides. No sandbox-escape surface.

No new `unsafe`/`unwrap()`/`panic!`/`todo!`/`unimplemented!` in production code (grep of
`^+` non-comment Rust lines = NONE). No canonical/wire struct touched (`canonical.rs`,
`seed.rs`, `task.rs`, `node_directory.rs`, `curator.rs` absent from diff). No new
`serde(default)`. No dep added/bumped.

## Research And G8

G8 preflight present (`sprint76_phase_b_preflight.md`, SCOPE-CUT-CONSISTENT, supervisor
GO-PREFLIGHT). S1a OSS prior art grounds B1 (VeraCrypt decoy / Signal deniability — benign
indistinguishable decoy), B2 (CERT IDS00-J / arc42 ingress sanitize at chokepoint), B3
(iroh-blobs Downloader intra-vector failover confirms the CROSS-tier gap is real). No new
dep, no crypto/spec work without research (B1-B11 are local logic + tests + doc). All 6
preflight corrections (B1 (b) availability-only, B2 ingress 2252-not-2231, B4 outbox 1790,
B9 5→4 pages, B10 parity-not-blind-align, anchor derivations) are APPLIED in the diff.

## Scope Cuts

SYBIL-SEEDER-TAIL **reconduit S77** (named exemption "dependance interne sharding",
kickoff §7) — diff grep for `sybil|sampling|tail|crowding|shard|tensor|model_digest|quorum|
capability_store` in code = NONE. No leak into Phase C compute territory. The 4 external
carries (P2-A-1 rand, P2-AUDIT-2 iroh, T-NN+2 wasm, P3-OS-1) untouched. Day 0 preserved
(iroh 0.98 pin, feed raw-op extensible, `*_VERSION` at 1). Pre-launch protocol: 0 bump
wire, all local/doc/test/refacto, 0 dep.

## Codex verification

PENDING — not yet run. The driver claims full-workspace 1775 Rust nextest + clippy/fmt/doc/
release green + web coverage gate + Docker canonique 675/675 subset. This review
independently confirmed the touched-crate Rust subset (672/672), the web unit suite
(396/396), and scan-en-strings (clean). The auditor must run the full three-block gate and
Codex, then reconcile.

Security delta: B1 closes the LOCAL-ONLY duress sibling gap (THREAT_MODEL row L→Nil); B2
closes the spoofable-badge ingress gap (new THREAT_MODEL row, residual L); B8 corrects a
false bearer claim. No new attack surface introduced. Net security posture improves.

## Commit Body Draft

```
fix(daemon+shell): Sprint 76 Phase B — duress siblings + 2-report carries + test/doc debt

## Contexte
Phase dette reservee NON convertible (sprint PAIR, Regle 1 G7). Le P1
DURESS-BOOT-LEAK etant deja ferme (23a08c9), B ferme le lot duress freres
LOCAL-ONLY (miroir local-mutation du wire-emit), casse 3 carries 2-reports
anti-escalade G7 (CARRY-3 B2, LOOPBACK-TIERS B7, PULL-3 B3) et comble les trous
de couverture test + corrections doc. SYBIL-SEEDER-TAIL reconduit S77 (exemption
nommee sharding).

## Fichiers
- crates/nexus-shell-daemon/src/http.rs — B1 duress short-circuit
  set_keep_online + seed_voluntary (early-return AVANT toute mutation/emit,
  reponse leurre benigne indistinguable) ; B2 helper partage
  trustworthy_open_source ; B3 chaine cross-tier build_seed_fetch_chain
  ticket->directory.
- crates/nexus-shell-daemon/src/runtime.rs — B2 downgrade is_open_source a
  l'ingress handle_project_announcement ; B4 test GossipCmd::Outbox ; B5 hoist
  my_endpoint_addr() once-per-pass (mint_ticket_for_hash_with_addr +
  current_replay_addr, addr threade dans remint_and_wrap_for_replay + 4 boucles).
- crates/nexus-shell-daemon-core/src/iroh_runtime.rs — B1 observed test (b).
- crates/sbfb-manifest/src/lib.rs — B10 allowlist 10->15 + test parite.
- web/src/pages/Nodes.tsx + Nodes.test.tsx — B6 discriminateur curateur/ancre.
- web/src/pages/__tests__/{Curators,OnboardingEmpty,ProjectDetail,Projects}.test.tsx
  — B9 smoke 4 pages 0-test.
- web/src/bridge/__tests__/protocol.test.ts — B10 verrou parite TS.
- web/package.json + .github/workflows/ci.yml + web/vitest.config.ts — B9 retrait
  etape CI Playwright vacuous (devDep gardee, documentee).
- docs/security/{LOOPBACK_ENDPOINTS_TRUST_TIERS,THREAT_MODEL}.md — B7/B8/B1/B2.
- .planning/active/sprint76_audit_plan.md — B7/B11.

## Delta tests
+8 Rust (http 3, runtime 3, iroh_runtime 1, sbfb-manifest 1) ; +10 Vitest
(4 pages smoke = 7, parite bridge 2, B6 1). Touched-crate nextest 672/672 ;
Vitest 396/396.

## Verification §7.4
[CI manifest a coller : 3 blocs verts + release build.]

## Scope cuts
SYBIL-SEEDER-TAIL reconduit S77 (exemption nommee sharding). 4 carries externes
(P2-A-1 rand, P2-AUDIT-2 iroh, T-NN+2 wasm, P3-OS-1) non touches. Aucune struct
wire/canonical touchee.

## G8 traceability
Preflight sprint76_phase_b_preflight.md verdict SCOPE-CUT-CONSISTENT (HEAD
d6dea45) ; 6 corrections preflight appliquees (B1 (b), B2 2252, B4 1790, B9 5->4,
B10 parite, ancres Phase A). Review sprint76_phase_b_review.md PASS-PENDING.
Codex [SHA + verdict a coller].

## Pre-launch protocol
0 bump wire, grep -c FORMAT_VERSION inchange, tout local/doc/test/refacto, 0 dep.

## Codex verification
[Output brut codex exec -o a coller.]
Security delta : B1 lot duress freres LOCAL-ONLY ferme (THREAT_MODEL row L->Nil) ;
B2 badge is_open_source downgrade a l'ingress /browse (residual L) ; B8 correction
cellule blob-serve bearer. 0 nouvelle surface.

## Carry closure / Unblock
CARRY-3 (B2), LOOPBACK-TIERS (B7), PULL-3 (B3), DURESS-FRERES (B1), T6-OUTBOX
(B4), WS-3/PD-5 (B5), DISCRIMINATEUR (B6), THREAT-BLOBSERVE (B8),
FRONTEND-COVERAGE + CI-PLAYWRIGHT-NOOP (B9), BRIDGE-ALLOWLIST (B10), UX-ARRIVAL
(B11) CLOSED. PULL-3 debloque le dial-set du quorum C/D.
```

## Findings

- **[P2] B10 cross-language parity is two hand-maintained mirrors, not a programmatic
  link.** `allowlist_mirrors_host_dispatch_schema` (Rust) checks against an in-test
  `EXPECTED` const, and `BridgeMethodSchema parity` (TS) against an in-test `CANONICAL`
  const. If a future sprint adds a method to ONE side and its OWN mirror const, neither
  test fails — the two languages can still drift silently because there is no single
  source of truth read by both. The parity lock catches the *list-vs-const* drift within
  a language, not the *Rust-vs-TS* drift it is named for. Mitigation today: both consts are
  the same 15 strings and the doc-comments cross-reference. Acceptable for a debt phase;
  route to a future generated-manifest or a shared JSON fixture both sides read.

- **[P2] B3 eager directory resolution on every happy-path seed.** `directory_snapshot()`
  + `directory_pull_providers` now run even when a live direct ticket would succeed
  (documented as RAM-snapshot cost). For a node with many subscribed directories this is
  per-request work that the pre-B3 happy path avoided. Correctness is fine and the
  trade-off is explicitly chosen (a dead ticket must be able to fall through), but it is a
  genuine discutable trade-off (G4): a lazy "resolve tier-2 only on tier-1 failure" variant
  would avoid the cost at the price of a second snapshot under contention. Acceptable;
  noted for horizon.

- **[P3] Preflight artifact staging.** `sprint76_phase_b_preflight.md` is untracked; ensure
  it is committed with the phase (G8 evidence) and not left dangling. Mechanical.

- **[P3] Decoy/real response divergence on rare error.** The duress `set_keep_online` decoy
  always returns 200, while the real path returns 500 on a DB-write failure. A 500 is a
  transient infra error, not a duress signal, and erring toward benign 200 is the correct
  decoy behavior — but a hyper-rigorous observer model would note the asymmetry. Negligible;
  documented for completeness.

## Residual Risk

Low. The phase is pure debt/refacto/test/doc: 0 wire bump, 0 dep, no canonical struct, no
new unsafe/unwrap/panic in prod, all duress no-ops proven by zero-mutation+zero-tag
assertions, the B2 downgrade hits the real shared chokepoint, and B5's hoist is proven by a
real address-threading test. The 2 P2 findings are forward-looking (parity source-of-truth,
eager-resolution cost), neither blocking. Verdict PASS-PENDING pending the full-workspace
three-block gate + Codex reconciliation; on green there, this is committable as `## Verdict: PASS`.
