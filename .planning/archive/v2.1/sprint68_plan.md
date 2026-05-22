# Sprint 68 — Plan (Proof Cards + Publish Gate)

**Ecrit** : 2026-05-21.
**Tip master** : `b937b03`.
**Roadmap** : Sprint 2/3, v2.1 Arc 2 Factory + RRV @protocole + Canari.

---

## §1 Etat verifie a l'entree

| Suite | Count | Commande | Observed |
|---|---|---|---|
| Rust nextest | 1384 | `cargo nextest run --workspace --locked` | |
| Rust doctests | ok | `cargo test --workspace --locked --doc` | |
| cargo fmt | 0 diff | `cargo fmt --all --check` | |
| cargo clippy | 0 warnings | `cargo clippy --workspace --all-targets --locked -- -D warnings` | |
| Vitest | 270 | `(cd web && npm run test:unit)` | |
| size-limit | 6/6 | `(cd web && npm run size)` | |
| release build daemon | ok | `cargo build -p nexus-shell-daemon --release` | |
| release build factory | ok | `cargo build -p sbfb-factory --release` | |
| **Total** | **~1660** | | |

---

## §2 Decisions Day 0 (gelees)

| D# | Decision | Implication code |
|---|---|---|
| D1 | ProofCard struct Rust + formule score deterministe | `proof_card.rs` (NEW coordinator) + `http.rs` (endpoint) + `protocol.ts` + `useBridge.ts` + `sbfb-bridge.js` |
| D2 | Preview ephemere via blob-serve existant | `http.rs` (POST preview/load) + `preview.rs` (NEW daemon-core, store + TTL) |
| D3 | Publish path Factory→daemon via deploy-from-repo | `publish.rs` (NEW factory) + `preview_cmd.rs` (NEW factory) + `main.rs` (subcommands) |
| D4 | Factory gates FG4-FG7 + P2-C-2 fix | `gates.rs` (NEW factory) + `diff.rs` (NEW factory) + `Cargo.toml` (dunce) + `template_engine.rs` (canonicalize) |
| D5 | Proof Card UI composant shell Browse | `ProofCard.tsx` (NEW) + `BrowsedProject.tsx` + `protocol.ts` + `sbfb-bridge.js` + `useBridge.ts` |

---

## §3 Graphe de dependances inter-phases

```
Phase A (ProofCard compute + endpoint)
  |
  v
Phase B (Preview ephemere + publish path)
  |
  v
Phase C (Factory gates FG4-FG7 + path traversal fix)
  |
  v
Phase D (ProofCard UI + Browse integration)
  |
  v
Phase E (Verification + wrap-up)
```

Phase B depend de A parce que le publish path cree des projets
que la ProofCard compute. Phase C depend de B parce que les gates
FG4-FG7 utilisent le preview (FG7) et le diff (FG4). Phase D depend
de A parce que le composant UI consomme l'endpoint proof-card livre
en Phase A.

---

## §4 Phase A — ProofCard computation + daemon endpoint

### §4.1 Scope

Creer la struct ProofCard dans nexus-coordinator-rs avec la formule de
score deterministe SYNTHESIS §4.6. Exposer un endpoint GET daemon.
Ajouter la methode bridge proof_card_get. Mettre a jour sbfb-manifest
allowlist.

### §4.2 Livrables

| Fichier | Changement |
|---|---|
| `crates/nexus-coordinator-rs/src/proof_card.rs` | NEW — struct ProofCard, formule compute_proof_card(), 7 risk factors, formula_version 1 |
| `crates/nexus-coordinator-rs/src/lib.rs` | pub mod proof_card |
| `crates/nexus-shell-daemon/src/http.rs` | GET /api/daemon/proof-card/{project_id} handler |
| `crates/nexus-shell-daemon-core/src/runtime.rs` | wiring proof_card dans DaemonRuntime (acces browse cache + feed + provenance + curators) |
| `crates/sbfb-manifest/src/lib.rs` | ajout "proof_card_get" dans BRIDGE_METHOD_ALLOWLIST |
| `web/src/api/protocol.ts` | schema Zod ProofCard |
| `web/src/hooks/useBridge.ts` | dispatch case proof_card_get |
| `web/public/sbfb-bridge.js` | methode proof_card_get |
| `examples/sbfb-explorer/sbfb-bridge.js` | sync SDK |
| `examples/sbfb-ideas/sbfb-bridge.js` | sync SDK |

### §4.3 Tests plan

1. `test_proof_card_full_evidence` — verifie score = 100 quand toutes les couches sont presentes
2. `test_proof_card_minimal` — verifie score = 30 (base) quand seul le resultat existe
3. `test_proof_card_provenance_boost` — verifie +20 quand provenance verified
4. `test_proof_card_risk_no_provenance` — verifie -15 quand no_provenance factor
5. `test_proof_card_formula_version` — verifie formula_version = 1
6. `test_proof_card_clamp_bounds` — verifie que le score reste dans [0, 100]
7. `test_proof_card_endpoint_http` — integration HTTP GET /api/daemon/proof-card/{id}
8. `test_proof_card_bridge_method` (Vitest) — schema Zod validation + dispatch

### §4.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-coordinator-rs -E 'test(proof_card)' --locked
# >= 6 tests pass
cargo nextest run -p nexus-shell-daemon -E 'test(proof_card)' --locked
# >= 1 test pass
(cd web && npm run test:unit)
# proof_card bridge test pass
grep -q "proof_card_get" crates/sbfb-manifest/src/lib.rs
# present dans allowlist
```

### §4.5 Commit cible

`feat(proof-card): Sprint 68 Phase A — ProofCard computation + daemon endpoint`

Body : 9 sections (Contexte, Fichiers, Delta tests, Verification §7.4,
Scope cuts, G8 traceability, Pre-launch protocol, Codex verification,
Carry closure).

---

## §5 Phase B — Preview ephemere + Factory publish path

### §5.1 Scope

Implanter le preview ephemere dans le daemon (POST /api/v1/preview/load
+ TTL eviction). Ajouter les subcommands `sbfb-factory preview` et
`sbfb-factory publish` qui communiquent avec le daemon via HTTP.

### §5.2 Livrables

| Fichier | Changement |
|---|---|
| `crates/nexus-shell-daemon-core/src/preview.rs` | NEW — PreviewStore HashMap + TTL 30 min + eviction task |
| `crates/nexus-shell-daemon-core/src/lib.rs` | pub mod preview |
| `crates/nexus-shell-daemon/src/http.rs` | POST /api/v1/preview/load handler (multipart ou raw bytes → hash) |
| `crates/nexus-shell-daemon/src/runtime.rs` | PreviewStore integration + spawn eviction task |
| `crates/sbfb-factory/src/publish.rs` | NEW — lit running.json, pre-validation, POST deploy-from-repo |
| `crates/sbfb-factory/src/preview_cmd.rs` | NEW — zip le repertoire, POST preview/load, affiche URL blob-serve |
| `crates/sbfb-factory/src/main.rs` | subcommands Preview + Publish ajoutes |
| `crates/sbfb-factory/Cargo.toml` | dep reqwest (HTTP client), zip (creation archive) |

### §5.3 Tests plan

1. `test_preview_load_returns_hash` — POST /api/v1/preview/load avec bytes → 200 + hash
2. `test_preview_blob_serve_accessible` — apres load, GET /blob-serve/{hash}/index.html → 200
3. `test_preview_eviction_after_ttl` — apres TTL, le hash n'est plus accessible (404)
4. `test_preview_max_size_rejected` — zip > 10 MB rejete (413)
5. `test_publish_requires_running_json` — publish sans running.json → erreur claire
6. `test_publish_pre_validates_manifest` — manifest invalide → erreur avant appel daemon

### §5.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-shell-daemon -E 'test(preview)' --locked
# >= 4 tests pass
cargo nextest run -p sbfb-factory -E 'test(publish)' --locked
# >= 2 tests pass
cargo build -p sbfb-factory --release
# compile OK
sbfb-factory --help | grep -E "preview|publish"
# subcommands presentes
```

### §5.5 Commit cible

`feat(factory): Sprint 68 Phase B — preview ephemere + publish path`

Body : 9 sections obligatoires.

---

## §6 Phase C — Factory gates FG4-FG7 + path traversal fix

### §6.1 Scope

Implanter les gates FG4-FG7 dans sbfb-factory. Corriger le path
traversal Windows (P2-C-2) avec `dunce::canonicalize`. Exposer
`sbfb-factory diff` et `sbfb-factory scan-secrets` comme subcommands.

### §6.2 Livrables

| Fichier | Changement |
|---|---|
| `crates/sbfb-factory/src/gates.rs` | NEW — run_gate_fg4_diff(), run_gate_fg5_sandbox(), run_gate_fg6_secrets(), run_gate_fg7_preview() |
| `crates/sbfb-factory/src/diff.rs` | NEW — compare workspace vs template, affiche ajoutes/modifies/supprimes |
| `crates/sbfb-factory/src/template_engine.rs` | refactor validate() : dunce::canonicalize + prefix check remplace contains("..") |
| `crates/sbfb-factory/src/main.rs` | subcommands Diff + ScanSecrets ajoutes |
| `crates/sbfb-factory/Cargo.toml` | dep dunce |

### §6.3 Tests plan

1. `test_fg5_rejects_path_traversal_canonicalize` — path `foo/../../../etc/passwd` rejete apres canonicalize
2. `test_fg5_rejects_windows_backslash_traversal` — path `foo\..\..\etc` rejete
3. `test_fg5_rejects_symlink` — symlink sortant du workspace rejete
4. `test_fg5_accepts_valid_subdir` — path `src/components/App.tsx` accepte
5. `test_fg6_lockfile_hash_consistency` — factory.template.lock hash == factory.provenance.json template_hash
6. `test_fg6_lockfile_mismatch_detected` — hash altere → erreur
7. `test_diff_detects_added_file` — fichier ajoute au workspace visible dans diff
8. `test_diff_detects_modified_file` — fichier modifie visible dans diff
9. `test_scan_secrets_cli_subcommand` — `sbfb-factory scan-secrets` detects AKIA dans un fichier test

### §6.4 Critere d'acceptation

```bash
cargo nextest run -p sbfb-factory -E 'test(fg5) | test(fg6) | test(diff) | test(scan_secrets)' --locked
# >= 9 tests pass
cargo build -p sbfb-factory --release
sbfb-factory --help | grep -E "diff|scan-secrets"
# subcommands presentes
# P2-C-2 resolved : canonicalize dans validate()
```

### §6.5 Commit cible

`feat(factory): Sprint 68 Phase C — Factory gates FG4-FG7 + path traversal fix`

Body : 9 sections obligatoires. Carry closure : P2-C-2 RESOLVED.

---

## §7 Phase D — Proof Card UI composant + Browse integration

### §7.1 Scope

Creer le composant React ProofCard.tsx, l'integrer dans BrowsedProject,
wirer le bridge proof_card_get, tester le composant.

### §7.2 Livrables

| Fichier | Changement |
|---|---|
| `web/src/components/ProofCard.tsx` | NEW — composant carte expandable, score 0-100, couches de preuve, risk factors |
| `web/src/pages/BrowsedProject.tsx` | integration ProofCard, appel bridge proof_card_get au chargement |
| `web/src/api/protocol.ts` | (deja livre Phase A — verification coherence) |
| `web/src/hooks/useBridge.ts` | (deja livre Phase A — verification coherence) |
| `docs/security/THREAT_MODEL.md` | §12 ProofCard surface : T-PROOFCARD-FORMULA-GAME (attaquant qui optimise pour le score sans substance) |

### §7.3 Tests plan

1. `test_proof_card_renders_score` (Vitest) — composant affiche le score numerique
2. `test_proof_card_renders_layers` (Vitest) — composant affiche les couches (provenance, license, etc.)
3. `test_proof_card_expandable` (Vitest) — clic expand montre les details
4. `test_proof_card_risk_factors_visible` (Vitest) — risk factors affiches si presents
5. `test_proof_card_formula_game_threat` — grep T-PROOFCARD dans THREAT_MODEL.md

### §7.4 Critere d'acceptation

```bash
(cd web && npm run test:unit)
# ProofCard tests pass (4+ nouveaux)
(cd web && npm run lint)
# 0 errors
(cd web && npx tsc --noEmit -p tsconfig.app.json)
# 0 errors
(cd web && npm run build)
# ok
(cd web && npm run size)
# 6/6
(cd web && bash scripts/scan-en-strings.sh)
# clean (composant en francais)
grep -q "T-PROOFCARD" docs/security/THREAT_MODEL.md
# present
```

### §7.5 Commit cible

`feat(shell): Sprint 68 Phase D — Proof Card UI + Browse integration`

Body : 9 sections obligatoires.

---

## §8 Phase E — Verification + wrap-up

### §8.1 Scope

Fail-fast checklist complete, audit_plan S69, CLAUDE.md, SPRINT_LOG.md,
memory update.

### §8.2 Livrables

| Fichier | Changement |
|---|---|
| `.planning/active/sprint68_verification.md` | NEW — fail-fast 30+ rows |
| `.planning/active/sprint69_audit_plan.md` | NEW — 9 tracks |
| `CLAUDE.md` | update compteurs, etat S68, carries |
| `docs/claude/SPRINT_LOG.md` | row S68 |

### §8.3 Tests plan

Pas de tests code — documentation seulement.

### §8.4 Critere d'acceptation

```bash
test -f .planning/active/sprint68_verification.md
test -f .planning/active/sprint69_audit_plan.md
```

### §8.5 Commit cible

`docs(sprint68): Sprint 68 Phase E — verification + wrap-up`

Body : 9/9 sections (incluant G8 et Pre-launch protocol, correction
P2-I-1).

---

## §9 Delta tests estime

| Phase | Rust | Vitest | Detail |
|---|---|---|---|
| A | +8 | +1 | proof_card 6 + endpoint 1 + bridge 1 Vitest |
| B | +6 | +0 | preview 4 + publish 2 |
| C | +9 | +0 | fg5 4 + fg6 2 + diff 2 + scan_secrets 1 |
| D | +0 | +4 | ProofCard UI 4 Vitest |
| E | +0 | +0 | documentation |
| **Total** | **+23** | **+5** | |
| **Sortie estimee** | **1407** | **275** | **~1688** |

---

## §10 Fail-fast checklist

| # | Check | Commande | Critere |
|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1407 |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok |
| 5 | release build daemon | `cargo build -p nexus-shell-daemon --release` | ok |
| 6 | release build factory | `cargo build -p sbfb-factory --release` | ok |
| 7 | npm lint | `(cd web && npm run lint)` | 0 errors |
| 8 | tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 errors |
| 9 | Vitest | `(cd web && npm run test:unit)` | >= 275 |
| 10 | npm build | `(cd web && npm run build)` | ok |
| 11 | size-limit | `(cd web && npm run size)` | 6/6 |
| 12 | scan-en-strings | `(cd web && bash scripts/scan-en-strings.sh)` | clean |
| 13 | scan-trust-wording | `bash scripts/scan-trust-wording.sh` | clean |
| 14 | sync-bridge-sdk | diff sbfb-bridge.js copies | identical |
| 15 | proof_card tests | `cargo nextest run -p nexus-coordinator-rs -E 'test(proof_card)' --locked` | >= 6 |
| 16 | proof_card endpoint | `cargo nextest run -p nexus-shell-daemon -E 'test(proof_card)' --locked` | >= 1 |
| 17 | preview tests | `cargo nextest run -p nexus-shell-daemon -E 'test(preview)' --locked` | >= 4 |
| 18 | publish tests | `cargo nextest run -p sbfb-factory -E 'test(publish)' --locked` | >= 2 |
| 19 | fg5 sandbox tests | `cargo nextest run -p sbfb-factory -E 'test(fg5)' --locked` | >= 4 |
| 20 | fg6 lockfile tests | `cargo nextest run -p sbfb-factory -E 'test(fg6)' --locked` | >= 2 |
| 21 | diff tests | `cargo nextest run -p sbfb-factory -E 'test(diff)' --locked` | >= 2 |
| 22 | ProofCard UI Vitest | `(cd web && npm run test:unit)` includes ProofCard | >= 4 |
| 23 | proof_card_get allowlist | `grep -q "proof_card_get" crates/sbfb-manifest/src/lib.rs` | present |
| 24 | THREAT_MODEL ProofCard | `grep -q "T-PROOFCARD" docs/security/THREAT_MODEL.md` | present |
| 25 | factory subcommands | `cargo run -p sbfb-factory -- --help` includes preview, publish, diff, scan-secrets | present |
| 26 | factory no daemon dep | `! grep -q "nexus-shell-daemon" crates/sbfb-factory/Cargo.toml` | absent |
| 27 | dunce in factory deps | `grep -q "dunce" crates/sbfb-factory/Cargo.toml` | present |
| 28 | P2-C-2 resolved | `cargo nextest run -p sbfb-factory -E 'test(fg5_rejects_windows)' --locked` | pass |
| 29 | verification.md | `test -f .planning/active/sprint68_verification.md` | exists |
| 30 | audit_plan S69 | `test -f .planning/active/sprint69_audit_plan.md` | exists |

---

## §11 Scope cuts

| # | Item | Sprint cible | Rationale |
|---|---|---|---|
| 1 | SearchManifest wire format + gossip | S70 | Protocole reseau hors scope S68 |
| 2 | Page React /factory | S69+ | CLI suffit pour S68 |
| 3 | Babel dogfood via Factory | S69 | Attend publish path S68 |
| 4 | @dev index tree-sitter | S70+ | Non bloquant Gate 1 |
| 5 | Template react-vite | S69+ | 2 templates suffisent |
| 6 | Factory audit log JSONL | S69+ | Gates tracent par stdout |
| 7 | CuratorVouched UI shell | S70+ | Feed ok, UI post-pilote |
| 8 | FG8 Provenance Ed25519 | S69 | Depend publish complet |
| 9 | FG9 Publish gate complete | S69 | S68 livre publish basic |
| 10 | FG10 Review gate | S69 | Depend FG8+FG9 |
| 11 | Fuzzing cargo-fuzz/proptest | post-audit | Hors scope fonctionnel |
| 12 | Feed format version bump | post-launch | Pre-launch policy |
| 13 | ProofCard comme feed op | S70+ | Compute local S68 |
| 14 | Diff engine avance | S69+ | Basique S68, semantique post-pilote |

---

## §12 Risks

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Preview fuite memoire | Medium | Medium | TTL + eviction + test cycle |
| R2 | dunce edge cases Windows | Low | Medium | Tests UNC + junction + fallback |
| R3 | ProofCard scores confus | Medium | Low | formula_version + couches detaillees |
| R4 | publish sans running.json | Low | Medium | Message erreur clair |
| R5 | proof_card_get sandbox | Low | High | Read-only + allowlist test |
| R6 | P2-I-2 atteint 3/3 S69 | High | Low | Template body S69 |
| R7 | Preview vs BlobStore interaction | Low | Medium | HashMap separe + test isolation |

---

## §13 Checkpoint de cloture

- [ ] 30/30 fail-fast verts
- [ ] 4 commits feat (Phase A, B, C, D) + 1 commit docs (Phase E)
- [ ] verification.md + audit_plan S69 ecrits
- [ ] THREAT_MODEL.md enrichi (T-PROOFCARD)
- [ ] PATTERNS.md mis a jour si nouveaux patterns
- [ ] CLAUDE.md + SPRINT_LOG.md a jour
- [ ] ProofCard affichee dans Browse
- [ ] Factory subcommands preview + publish + diff + scan-secrets
- [ ] sbfb-factory ne depend PAS de nexus-shell-daemon-core
- [ ] P2-C-2 path traversal RESOLVED
- [ ] Memory nexus_grid_pivot.md tip + compteurs mis a jour
