# Sprint 32 — Plan d'execution (dette pair iroh 0.98 + carries)

**Ecrit** : 2026-04-27 (post-kickoff, pre-Phase A)
**Tip entree** : `1cc4734`

---

## §1 Etat verifie a l'entree

```
HEAD: 1cc4734 (chore(planning): sprint 31 audit findings — verdict PASS)
```

| Suite | Count | Status |
|---|---|---|
| Rust nextest | 878/878 | ✅ |
| Rust clippy | 0 warnings | ✅ |
| Rust fmt | clean | ✅ |
| Python SDK | 195 passed | ✅ |
| Python coord | 406 passed + 36 failed (PyO3 stale) + 6 skipped | ✅ (stale root cause) |
| Python gov | 46 passed | ✅ |
| Vitest | 267/267 (23 files) | ✅ |
| Playwright | 41+2f (env) | ✅ (env root cause) |
| size-limit | 7/7 | ✅ |

---

## §2 Decisions Day 0 (gelees — rappel)

- **D1** : iroh stack upgrade simultane 4 crates (0.97→0.98 / blobs
  0.99→0.100). Day 0 #3 leve.
- **D2** : rusqlite 0.32→0.36 + arti-client 0.41 dep activation
  feature `tor`.
- **D3** : Wire max_tokens dans executor task_runner.rs.
- **D4** : P2 batch audit S31 (HARDENING compteurs, Tor log, FROST
  error tests, Playwright COEP tentative).
- **D5** : Levee formelle pin iroh 0.97 (Day 0 #3 reformulee).

---

## §3 Research consulte

- context7 iroh `/websites/rs_iroh` : 1743 snippets, Endpoint builder
  API, SecretKey::generate signature.
- crates.io API : matrice deps iroh-blobs 0.100 (^0.98), iroh-docs
  0.98 (^0.98 + ^0.100 blobs), iroh-gossip 0.98 (^0.98).
- WebSearch GitHub : n0-computer/iroh releases v0.98.0, CHANGELOG.md.
- Agent Explore codebase : cartographie usage iroh dans 6 fichiers
  nexus-core-rs/src/ (node.rs, relay_config.rs, discovery.rs, blobs.rs,
  docs.rs, gossip.rs). Pas de ConnectionType, pas de SecretKey::generate
  direct.

---

## §4 Dependencies inter-phases

```
Phase A (iroh upgrade) → Phase B (rusqlite + arti)
  B depend de A car l'upgrade iroh peut modifier les versions
  transitives resolvables. Mieux vaut stabiliser iroh d'abord.
Phase B (arti activation) → Phase C (batch carries)
  C wire max_tokens dans executor. Independant de A/B, mais
  sequencer apres pour eviter merge conflicts sur Cargo.toml.
Phase C → Phase D (wrap-up)
  D documente tout.
```

---

## §5 Phase A — iroh stack upgrade 0.97→0.98

### §5.1 Scope

Upgrade simultane des 4 crates iroh workspace. Resolution de tous
les breaking changes pour que les 878 tests Rust restent verts.
Le release build `nexus-shell-daemon --release` doit aussi compiler.

### §5.2 Fichiers touches

| Fichier | Role |
|---|---|
| `Cargo.toml` (workspace) | Bump iroh 0.97→0.98, iroh-docs 0.97→0.98, iroh-gossip 0.97→0.98, iroh-blobs 0.99→0.100 |
| `crates/nexus-core-rs/src/node.rs` | Endpoint::builder + SecretKey usage — adapter aux breaking changes #4075 #4107 |
| `crates/nexus-core-rs/src/relay_config.rs` | RelayMode/RelayMap — adapter si API change (#4130) |
| `crates/nexus-core-rs/src/discovery.rs` | Endpoint, EndpointAddr, TransportAddr — adapter ConnectionType removal |
| `crates/nexus-core-rs/src/blobs.rs` | BlobsProtocol, BlobTicket, Downloader, MemStore — iroh-blobs 0.100 API |
| `crates/nexus-core-rs/src/docs.rs` | iroh-docs Doc/Author/LiveEvent — adapter si API change |
| `crates/nexus-core-rs/src/gossip.rs` | Gossip/GossipTopic/Event — iroh-gossip 0.98 API |
| `crates/nexus-core-rs/src/pkarr_resolver.rs` | PkarrRelayClient — pkarr vendored (#4026) impact potentiel |
| `crates/nexus-core-rs/Cargo.toml` | Deps crate-level si features changent |
| `crates/nexus-shell-daemon-core/src/*.rs` | Consomme via abstractions core-rs — adapter si types changent |
| `crates/nexus-worker-core/src/*.rs` | Idem — impact minimal si core-rs absorbe |

### §5.3 Tests plan

Tests existants (878) doivent rester verts — pas de nouveaux tests
prevus. L'upgrade est une migration, pas une feature. Si un test
echoue a cause d'un changement d'API, le fix est dans l'adaptation
du code, pas dans le test.

Test supplementaire optionnel :
1. `test_endpoint_online_semantics` — si le comportement
   Endpoint::online() change significativement (attend relay #4115),
   ajouter un test unitaire pour documenter la nouvelle semantique.

### §5.4 Critere d'acceptation

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked  # 878 pass
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release
```

### §5.5 Commit cible

```
feat(sprint32): Sprint 32 Phase A — iroh stack upgrade 0.97→0.98 workspace-wide

## Contexte
Sprint pair dette (§6.2.1 Regle 1). Day 0 #3 (pin iroh 0.97) formellement
leve. LT-6 trigger met (iroh 0.98.0 publie 2026-04-17, 10j).

## Fichiers
- Cargo.toml workspace: iroh 0.97→0.98, iroh-docs 0.97→0.98,
  iroh-gossip 0.97→0.98, iroh-blobs 0.99→0.100
- crates/nexus-core-rs/src/*.rs: adaptation 8 breaking changes

## Delta tests
878 Rust pass → 878 pass (migration, pas feature).

## Verification §7.4
cargo fmt ✅ / clippy ✅ / nextest 878 ✅ / doctests ✅ / release build ✅
Python (SDK 195 / coord 406+36f+6s / gov 46) ✅
Frontend (lint / tsc / vitest 267 / build / size 7/7) ✅

## Scope cuts respectes (kickoff §7)
12 items inchanges.

## G8 traceability
sprint32_phase_A_preflight.md verdict: [A REMPLIR]

## Pre-launch protocol
*_VERSION = 1 partout. iroh relay v2 = interne iroh, transparent SBFB.
0 wire format SBFB modifie.
```

---

## §6 Phase B — rusqlite 0.36 + arti-client dep activation

### §6.1 Scope

Upgrade rusqlite 0.32→0.36 workspace-wide (bundled, libsqlite3-sys
0.34). Decommenter arti-client 0.41 + tor-rtcompat 2.0 dans
nexus-core-rs. Remplir la feature `tor` avec les vrais deps.
Resout P2-REVIEW-C-1 (1/3→closed) et P3-AUDIT-1 (compile trap).

### §6.2 Fichiers touches

| Fichier | Role |
|---|---|
| `Cargo.toml` (workspace) | Bump rusqlite 0.32→0.36 |
| `crates/nexus-core-rs/Cargo.toml` | Decommenter `arti-client = "0.41"`, `tor-rtcompat`. Feature `tor = ["dep:arti-client", "dep:tor-rtcompat"]` |
| `crates/nexus-core-py/Cargo.toml` | Verifier passthrough feature `tor = ["nexus-core-rs/tor"]` |
| `crates/nexus-core-rs/src/tor_transport.rs` | Verifier compilation `#[cfg(feature = "tor")]` bloc avec arti_client reel |
| `crates/nexus-worker-core/src/*.rs` | Verifier compat rusqlite 0.36 (allowlist SQLite) |
| `crates/nexus-shell-daemon-core/src/*.rs` | Verifier compat rusqlite 0.36 (quarantine, trust cache) |

### §6.3 Tests plan

1. `cargo build -p nexus-core-rs --features tor` — compilation OK
   avec arti-client reel (pas juste feature vide).
2. Tests SQLite existants (nexus-worker-core allowlist, shell-daemon-core
   quarantine, trust cache) doivent rester verts avec rusqlite 0.36.
3. Test optionnel : `test_tor_transport_arti_bootstrap_timeout` si
   l'API arti-client est testable sans reseau Tor reel (mock ou timeout).

### §6.4 Critere d'acceptation

```bash
cargo build -p nexus-core-rs --features tor  # compile
cargo nextest run --workspace --locked        # 878+ pass
cargo build -p nexus-shell-daemon --release   # release OK
```

### §6.5 Commit cible

```
feat(sprint32): Sprint 32 Phase B — rusqlite 0.36 + arti-client dep activation tor feature

## Contexte
P2-REVIEW-C-1 (1/3→closed) : rusqlite upgrade resout le conflit
libsqlite3-sys qui bloquait arti-client 0.41 dep activation depuis S31.
P3-AUDIT-1 (closed) : feature gate `tor = []` n'est plus un compile trap.

## Fichiers
- Cargo.toml workspace: rusqlite 0.32→0.36
- crates/nexus-core-rs/Cargo.toml: arti-client 0.41 + tor-rtcompat 2.0
  deps activees, feature tor remplie
- crates/nexus-core-py/Cargo.toml: passthrough feature verifie

## Delta tests
878 Rust pass → [N] pass (possiblement +1 bootstrap timeout test).

## Verification §7.4
cargo fmt ✅ / clippy ✅ / nextest [N] ✅ / doctests ✅ / release build ✅
cargo build --features tor ✅
Python ✅ / Frontend ✅

## Scope cuts respectes (kickoff §7)
12 items inchanges.

## Carry closure
P2-REVIEW-C-1 rusqlite + arti dep activation: CLOSED (1/3→resolved)
P3-AUDIT-1 tor feature gate compile trap: CLOSED

## Pre-launch protocol
*_VERSION = 1 partout. arti-client = runtime dep, pas wire format.
```

---

## §7 Phase C — P2 batch carries audit S31

### §7.1 Scope

Batch des items P2/P3 identifes par l'audit S31. Wire max_tokens
executor, fix HARDENING compteurs, fix Tor log, FROST error tests,
et tentative Playwright COEP.

### §7.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-executor/src/task_runner.rs` | Wire max_tokens via GenerationOptions (P2-AUDIT-1) |
| `crates/nexus-executor/src/ipc.rs` | Verifier que max_tokens est bien dans TaskExecuteParams |
| `docs/security/HARDENING_ROADMAP.md` | Fix compteurs frontmatter + S32 entry (P2-AUDIT-2) |
| `packages/nexus-coordinator/src/nexus_coordinator/coordinator.py` | Fix Tor boot log misleading (P3-AUDIT-3) |
| `crates/nexus-shell-daemon/src/http.rs` | FROST HTTP error path tests (P3-AUDIT-2) |
| `web/e2e/*.spec.ts` | Playwright COEP iframe test tentative (P2-REVIEW-B-1-S30) |

### §7.3 Tests plan

1. `test_execute_task_ollama_mock_respects_max_tokens` — nouveau test
   verifiant que max_tokens est passe a Ollama comme num_predict.
2. `frost_http_invalid_threshold_k_gt_n` — FROST error path k>n.
3. `frost_http_malformed_json_body` — FROST malformed request.
4. `frost_http_wrong_participant_round2` — wrong participant ID.
5. `frost_http_invalid_nonces_aggregate` — invalid nonces → error.
6. Playwright COEP iframe test (si env stable) — 1 test E2E.

### §7.4 Critere d'acceptation

```bash
cargo nextest -p nexus-executor   # max_tokens test pass
cargo nextest -p nexus-shell-daemon -E 'test(frost_http)'  # 4+N tests
uv run pytest packages/nexus-coordinator/tests/ -q  # Tor log fix
npx playwright test  # COEP test (si env stable)
```

### §7.5 Commit cible

```
feat(sprint32): Sprint 32 Phase C — P2 batch carries audit S31 + Playwright COEP attempt

## Contexte
P2-AUDIT-1 (closed): max_tokens wire dans GenerationOptions.
P2-AUDIT-2 (closed): HARDENING compteurs corriges.
P3-AUDIT-3 (closed): Tor boot log differencie disabled vs unavailable.
P3-AUDIT-2 (closed): 4 FROST HTTP error path tests.
P2-REVIEW-B-1-S30: [TENTATIVE/EXEMPTION — a determiner Phase C].

## Fichiers
- crates/nexus-executor/src/task_runner.rs: wire max_tokens
- docs/security/HARDENING_ROADMAP.md: compteurs + S32 entry
- packages/nexus-coordinator/coordinator.py: Tor log fix
- crates/nexus-shell-daemon/src/http.rs: 4 FROST error tests
- web/e2e/*.spec.ts: Playwright COEP test (si applicable)

## Delta tests
[N] Rust → [N+5] (1 max_tokens + 4 FROST error)
[M] Playwright → [M+1] (si COEP test passe)

## Verification §7.4
cargo fmt ✅ / clippy ✅ / nextest ✅ / doctests ✅ / release build ✅
Python ✅ / Frontend ✅ / Playwright ✅

## Scope cuts respectes (kickoff §7)
12 items inchanges.

## Carry closure
P2-AUDIT-1 executor param drops: CLOSED
P2-AUDIT-2 HARDENING compteurs: CLOSED
P3-AUDIT-2 FROST error paths: CLOSED
P3-AUDIT-3 Tor boot log: CLOSED
P2-REVIEW-B-1-S30 Playwright COEP: [CLOSED/EXEMPTION_MANDATORY_S33]

## Pre-launch protocol
*_VERSION = 1 partout. 0 wire format modifie.
```

---

## §8 Phase D — Wrap-up + verification + audit plan S33

### §8.1 Scope

Verification fail-fast 28+ rows. Production des 3 livrables planning
(verification, audit_plan S33, carry_summary). Mise a jour des docs
de reference (SPRINT_LOG, CLAUDE.md, HARDENING_ROADMAP, ROADMAP_COMMITMENTS,
memory). Migration active/ → archive/v1.2/.

### §8.2 Fichiers touches

| Fichier | Role |
|---|---|
| `.planning/active/sprint32_verification.md` | Self-report fail-fast |
| `.planning/active/sprint33_audit_plan.md` | Plan audit pour S33 Phase 0 |
| `.planning/active/sprint32_carry_summary.md` | Carries restants |
| `docs/claude/SPRINT_LOG.md` | Row S32 |
| `CLAUDE.md` | §Etat actuel (iroh 0.98 reference) |
| `docs/security/HARDENING_ROADMAP.md` | S32 entry, trigger LT-6 resolved |
| `docs/release/ROADMAP_COMMITMENTS.md` | LT-6 status → resolved S32 |
| Memory files | nexus_grid_pivot.md + MEMORY.md updates |

### §8.3 Commit cible

```
chore(sprint32): Phase D — wrap-up + verification + audit plan S33 + migration
```

---

## §9 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | Rust compile workspace | `cargo build --workspace --locked` | 0 errors | |
| 2 | Rust nextest pass | `cargo nextest run --workspace --locked` | 878+ pass, 0 fail | |
| 3 | Rust doctests pass | `cargo test --workspace --locked --doc` | 0 fail | |
| 4 | Rust clippy clean | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 5 | Rust fmt clean | `cargo fmt --all --check` | no output | |
| 6 | Release build daemon | `cargo build -p nexus-shell-daemon --release` | Finished | |
| 7 | Python ruff format | `uv run ruff format --check packages/` | clean | |
| 8 | Python ruff check | `uv run ruff check packages/` | pass | |
| 9 | SDK 195 pass | `uv run pytest packages/nexus-sdk/tests/ -q` | 195 pass | |
| 10 | Coord 406+ pass | `uv run pytest packages/nexus-coordinator/tests/ -q` | 406+ pass + 36f stale | |
| 11 | Gov 46 pass | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 pass | |
| 12 | Frontend lint | `cd web && npm run lint` | 0 errors | |
| 13 | Frontend tsc | `npx tsc --noEmit -p tsconfig.app.json` | clean | |
| 14 | Vitest 267+ pass | `npm run test:unit` | 267+ pass | |
| 15 | Frontend build | `npm run build` | success | |
| 16 | size-limit 7/7 | `npm run size` | 7/7 pass | |
| 17 | Playwright | `npx playwright test` | 41+ pass | |
| 18 | en-strings | `bash scripts/scan-en-strings.sh` | clean | |
| 19 | iroh version 0.98 | `grep 'iroh = "0.98"' Cargo.toml` | found | |
| 20 | iroh-blobs 0.100 | `grep 'iroh-blobs = "0.100"' Cargo.toml` | found | |
| 21 | iroh-docs 0.98 | `grep 'iroh-docs = "0.98"' Cargo.toml` | found | |
| 22 | iroh-gossip 0.98 | `grep 'iroh-gossip = "0.98"' Cargo.toml` | found | |
| 23 | rusqlite 0.36 | `grep 'rusqlite.*0.36' Cargo.toml` | found | |
| 24 | tor feature compile | `cargo build -p nexus-core-rs --features tor` | success | |
| 25 | max_tokens test | `cargo nextest -p nexus-executor -E 'test(max_tokens)'` | 1 pass | |
| 26 | FROST error tests | `cargo nextest -p nexus-shell-daemon -E 'test(frost_http)'` | 8+ pass | |
| 27 | FORMAT_VERSION v1 | `grep _VERSION crates/nexus-core-rs/src/ \| grep -v "= 1"` | 0 matches | |
| 28 | HARDENING compteurs | `grep '~878 Rust' docs/security/HARDENING_ROADMAP.md` | found | |
| 29 | Planning docs | kickoff + plan + design_review + preflights + reviews | complets | |

---

## §10 Git plan

```
1. chore(planning): sprint 32 kickoff + plan + design review
2. [G8 preflight Phase A] chore(planning): sprint 32 Phase A — G8 preflight
3. feat(sprint32): Sprint 32 Phase A — iroh stack upgrade 0.97→0.98
4. [review Phase A] chore(planning): sprint 32 Phase A — review
5. [G8 preflight Phase B] chore(planning): sprint 32 Phase B — G8 preflight
6. feat(sprint32): Sprint 32 Phase B — rusqlite 0.36 + arti-client dep activation
7. [review Phase B] chore(planning): sprint 32 Phase B — review
8. [G8 preflight Phase C] chore(planning): sprint 32 Phase C — G8 preflight
9. feat(sprint32): Sprint 32 Phase C — P2 batch carries audit S31
10. [review Phase C] chore(planning): sprint 32 Phase C — review
11. chore(sprint32): Phase D — wrap-up + verification + audit plan S33 + migration
```

---

## §11 Scope cuts (copie kickoff §7)

1. iroh relay over Tor → S33+
2. Nym mixnet phase 1 → S33+
3. TEE H100 attestation → post-v1.0
4. DKG distribue FROST → post-v1.0
5. Recrutement mainteneurs → post-v1.0
6. Onion service hosting → post phase 1
7. Full process isolation blob-serve → LT
8. openai-agents-python → pas de dep
9. llama.cpp executor → S33+
10. Output filter client-side → S34
11. iroh 1.0 wait → 0.98 est le cible
12. rusqlite 0.39 → 0.36 suffisant

---

## §12 Risks (rappel kickoff §9)

R1-R6 cf. kickoff §9. Principal risque : R1 (iroh breaking changes
cascade) + R2 (iroh-blobs 0.100 ticket wire).

---

## §13 Checkpoint de cloture

1. 29/29 fail-fast rows vertes
2. 11 commits (1 planning + 3 feat + 3 chore review/preflight per
   phase + 1 wrap-up)
3. 3 fichiers planning ecrits (verification, audit_plan, carry_summary)
4. iroh 0.98 dans Cargo.toml
5. `cargo build --features tor` compile
6. HARDENING_ROADMAP.md a jour
7. ROADMAP_COMMITMENTS.md LT-6 resolved
8. Memory mise a jour
9. Migration active/ → archive/v1.2/
