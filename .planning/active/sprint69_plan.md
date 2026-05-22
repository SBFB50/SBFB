# Sprint 69 — Plan (Babel dogfood via Factory + pilote ferme + Gate 1)

**Ecrit** : 2026-05-22.
**Tip master** : `0c2c2a8`.
**Roadmap** : Sprint 3/3, v2.1 Arc 2 Factory + RRV @protocole + Canari.

---

## §1 Etat verifie a l'entree

| Suite | Count | Commande | Observed |
|---|---|---|---|
| Rust nextest | 1419 | `cargo nextest run --workspace --locked` | 1419/1419 PASS |
| Rust doctests | ok | `cargo test --workspace --locked --doc` | ok |
| cargo fmt | 0 diff | `cargo fmt --all --check` | |
| cargo clippy | 0 warnings | `cargo clippy --workspace --all-targets --locked -- -D warnings` | |
| Vitest | 279 | `(cd web && npm run test:unit)` | |
| size-limit | 6/6 | `(cd web && npm run size)` | |
| release build daemon | ok | `cargo build -p nexus-shell-daemon --release` | |
| release build factory | ok | `cargo build -p sbfb-factory --release` | |
| **Total** | **~1704** | | |

---

## §2 Decisions Day 0 (gelees)

| D# | Decision | Implication code |
|---|---|---|
| D1 | FG8 Provenance Ed25519 verification post-publish | `gates.rs`, `publish.rs`, `Cargo.toml` |
| D2 | Babel Reader template static-reader via Factory | `templates/static-reader/`, `template_engine.rs` |
| D3 | FG9 Publish pipeline FG4→FG5→FG6→publish→FG8 | `pipeline.rs` (NEW), `publish.rs`, `gates.rs` |
| D4 | Factory audit log JSONL + P2-I-2 + P2-B-1 | `audit_log.rs`, `preview.rs`, `count-tests.sh`, `THREAT_MODEL.md` |
| D5 | Gate 1 test protocol + pilote ferme prep | `GATE1_TEST_PROTOCOL.md` |

---

## §3 Graphe de dependances inter-phases

```
Phase A (carries + audit log + preview cap)
    |
    v
Phase B (FG8 + FG9 pipeline)  -- depend de A car pipeline utilise audit_log
    |
    v
Phase C (Babel template + dogfood E2E)  -- depend de B car dogfood publie via pipeline
    |
    v
Phase D (Gate 1 docs + CLI subcommands FG5/FG7)  -- depend de C car test protocol reference le dogfood
    |
    v
Phase E (verification + wrap-up)
```

Phase A est prerequis de Phase B parce que le publish pipeline (Phase
B) ecrit dans l'audit log (Phase A). Phase B est prerequis de Phase C
parce que le dogfood Babel (Phase C) utilise `sbfb-factory publish`
avec le pipeline FG4-FG8 (Phase B). Phase C est prerequis de Phase D
parce que le test protocol Gate 1 (Phase D) reference l'app Babel
deployee comme scenario de test.

---

## §4 Phase A — P2-I-2 template body + P2-B-1 preview cap + audit log

### §4.1 Scope

Resoudre P2-I-2 3/3 MANDATORY (template body standardise), P2-B-1
(MAX_PREVIEW_ENTRIES cap dans preview store), et creer le module audit
log JSONL pour Factory. Mettre a jour THREAT_MODEL.md pour la surface
preview abuse.

### §4.2 Livrables

| Fichier | Description |
|---|---|
| `crates/sbfb-factory/src/audit_log.rs` | NEW — module audit log JSONL. Struct `AuditEntry` (timestamp, command, args, result, gates_results). Fonctions `log_entry()` (append JSONL a `~/.sbfb/factory-audit.log`) et `audit_log_path()`. |
| `crates/sbfb-factory/src/main.rs` | Ajouter `mod audit_log;` + appel `audit_log::log_entry()` apres chaque subcommand. |
| `crates/nexus-shell-daemon-core/src/preview.rs` | Ajouter `const MAX_PREVIEW_ENTRIES: usize = 10;` + check dans `load()` → `PreviewError::TooManyEntries`. |
| `docs/security/THREAT_MODEL.md` | Ajouter §13 Preview ephemere : vecteur DoS local, mitigations (MAX_PREVIEW_BYTES + MAX_PREVIEW_ENTRIES + TTL + loopback-only + auth). |
| `scripts/count-tests.sh` | NEW — parse sortie nextest + Vitest, affiche compteurs structures. Utilise dans le process commit body. |

### §4.3 Tests plan

1. `test_preview_rejects_too_many_entries` — charge 11 previews de 1KB, verifie que le 11e retourne `PreviewError::TooManyEntries`
2. `test_preview_accepts_after_eviction` — charge 10 previews, evicte 1, charge 1 nouveau → ok
3. `test_audit_log_writes_jsonl` — appelle `log_entry()`, lit le fichier, verifie que la ligne est du JSON valide
4. `test_audit_log_appends` — 2 appels, verifie 2 lignes dans le fichier

### §4.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-shell-daemon-core -E 'test(preview)' --locked  # >= 6 (4 existants + 2 nouveaux)
cargo nextest run -p sbfb-factory -E 'test(audit)' --locked               # >= 2
grep -c "MAX_PREVIEW_ENTRIES" crates/nexus-shell-daemon-core/src/preview.rs  # 1+
grep -q "§13\|Preview ephemere\|preview abuse" docs/security/THREAT_MODEL.md  # present
test -f scripts/count-tests.sh                                              # exists
```

### §4.5 Commit cible

`feat(factory): Sprint 69 Phase A — preview cap + audit log + P2-I-2 template`

Body 9 sections obligatoires :
Contexte, Fichiers, Delta tests, Verification §7.4, Scope cuts
respectes, G8 traceability, Pre-launch protocol, Codex verification,
Carry closure.

Procedure delta tests (P2-I-2 resolution) : lancer `cargo nextest run
--workspace --locked` → lire "N tests run: N passed" → copier le
compteur reel dans §Delta tests. Script `count-tests.sh` disponible
comme aide-memoire.

---

## §5 Phase B — FG8 Provenance Ed25519 + FG9 Publish pipeline

### §5.1 Scope

Implanter FG8 (verification provenance Ed25519 post-publish dans
sbfb-factory) et FG9 (pipeline integre orchestrant FG4→FG5→FG6→
publish→FG8). Retirer les `#[allow(dead_code)]` de gates.rs via le
wiring naturel dans le pipeline.

### §5.2 Livrables

| Fichier | Description |
|---|---|
| `crates/sbfb-factory/src/gates.rs` | Ajouter `run_gate_fg8_provenance(provenance_json: &str, node_public_key: &[u8; 32]) -> GateResult`. Utilise `nexus_coordinator_rs::provenance::verify_provenance()`. Retirer `#[allow(dead_code)]` de fg5, fg7, check_path_containment. |
| `crates/sbfb-factory/src/pipeline.rs` | NEW — `run_publish_pipeline(workspace, repo_url, skip_gates: bool) -> Result<PipelineResult>`. Sequence : FG4 diff (informatif) → FG5 sandbox (bloquant) → FG6 secrets (bloquant) → publish POST → FG8 provenance (bloquant). `PipelineResult` contient le hash, provenance_hash, et gate_results. |
| `crates/sbfb-factory/src/publish.rs` | Refactor : extraire la logique POST dans une fonction `post_deploy_from_repo()`. `run()` appelle `pipeline::run_publish_pipeline()` au lieu de faire le POST directement. |
| `crates/sbfb-factory/src/daemon_client.rs` | Ajouter `get_provenance(project_id: &str) -> Result<String>` — GET /api/v1/project/{id}/provenance. Ajouter `get_node_id() -> Result<[u8; 32]>` — GET /api/daemon/status, extraire node_id hex. |
| `crates/sbfb-factory/src/main.rs` | Ajouter `mod pipeline;` + flag `--skip-gates` sur Publish subcommand. Chaque subcommand appelle `audit_log::log_entry()`. |
| `crates/sbfb-factory/Cargo.toml` | Dep `nexus-coordinator-rs` (features: default, pour `verify_provenance`). |

### §5.3 Tests plan

1. `test_fg8_provenance_valid_signature` — genere une provenance signee, verifie FG8 PASS
2. `test_fg8_provenance_wrong_key` — genere avec key A, verifie avec key B → FG8 FAIL
3. `test_fg8_provenance_tampered_json` — modifie artifact_hash, verifie FG8 FAIL
4. `test_pipeline_aborts_on_secrets` — workspace avec fichier contenant `AKIA...` → pipeline FAIL avant publish
5. `test_pipeline_aborts_on_path_traversal` — workspace avec symlink externe → pipeline FAIL avant publish
6. `test_pipeline_runs_diff_informational` — diff montre fichiers, pipeline continue

### §5.4 Critere d'acceptation

```bash
cargo nextest run -p sbfb-factory -E 'test(fg8)' --locked        # >= 3
cargo nextest run -p sbfb-factory -E 'test(pipeline)' --locked    # >= 3
grep -c "allow(dead_code)" crates/sbfb-factory/src/gates.rs      # 0
grep -q "run_gate_fg8" crates/sbfb-factory/src/gates.rs           # present
grep -q "run_publish_pipeline" crates/sbfb-factory/src/pipeline.rs  # present
```

### §5.5 Commit cible

`feat(factory): Sprint 69 Phase B — FG8 provenance Ed25519 + FG9 publish pipeline`

Body 9 sections obligatoires. Procedure delta tests P2-I-2 : lancer
nextest → compteur reel → copier dans body.

---

## §6 Phase C — Babel Reader template + Factory dogfood E2E

### §6.1 Scope

Creer le template `static-reader` pour Babel, tester le flow complet
create→validate→preview→publish→browse→search→proof-card. Le test est
un E2E documentaire (execution manuelle du flow complet avec daemon
local, documentee dans le commit body).

### §6.2 Livrables

| Fichier | Description |
|---|---|
| `crates/sbfb-factory/src/templates/static-reader/index.html` | NEW — squelette reader HTML/CSS/JS. Navigation prev/next entre "sections" de texte. Dark theme. Responsive. Placeholders `{{name}}` substitues par template_engine. |
| `crates/sbfb-factory/src/templates/static-reader/SBFB.json` | NEW — manifest v2 template. `schema_version: 2`, `name: "{{name}}"`, `category: "content"`, `bridge.methods: ["storage_get", "storage_set", "identity_pubkey"]`. |
| `crates/sbfb-factory/src/templates/static-reader/sbfb-bridge.js` | NEW — copie SDK bridge depuis `web/public/sbfb-bridge.js`. |
| `crates/sbfb-factory/src/templates/static-reader/.gitignore` | NEW — node_modules, .DS_Store. |
| `crates/sbfb-factory/src/templates/static-reader/README.md` | NEW — instructions utilisateur (comment ajouter du contenu, comment publier). |
| `crates/sbfb-factory/src/template_engine.rs` | Ajouter support template name "static-reader" dans `TEMPLATES` map. |

### §6.3 Tests plan

1. `test_create_static_reader_template` — `sbfb-factory create --template static-reader --name test-app` produit un repertoire avec index.html, SBFB.json, sbfb-bridge.js
2. `test_validate_static_reader_passes` — le projet genere passe `sbfb-factory validate`
3. `test_static_reader_template_substitution` — les placeholders `{{name}}` sont remplaces dans SBFB.json et index.html

### §6.4 Critere d'acceptation

```bash
cargo nextest run -p sbfb-factory -E 'test(static_reader)' --locked  # >= 3
cargo run -p sbfb-factory -- create --template static-reader --name babel-test --output /tmp/babel-test  # success
ls crates/sbfb-factory/src/templates/static-reader/index.html  # exists
```

### §6.5 Commit cible

`feat(factory): Sprint 69 Phase C — Babel Reader template + dogfood E2E`

Body 9 sections obligatoires. Le § Contexte documente le flow E2E
complet execute manuellement : create → validate → preview → publish →
browse → search → proof-card. Avec les commandes exactes et les
resultats observes.

---

## §7 Phase D — Gate 1 test protocol + CLI subcommands FG5/FG7

### §7.1 Scope

Produire le document `GATE1_TEST_PROTOCOL.md` avec les 9 procedures
pas-a-pas pour le pilote ferme. Exposer les subcommands CLI `sandbox`
et `preview-check` pour retirer les `#[allow(dead_code)]` restants
(si Phase B ne les a pas tous retires). Documenter les instructions
d'installation pour les testeurs.

### §7.2 Livrables

| Fichier | Description |
|---|---|
| `docs/release/GATE1_TEST_PROTOCOL.md` | NEW — 9 procedures pas-a-pas correspondant aux 9 criteres Gate 1 (roadmap v4). Formulaire feedback (table critere / resultat / notes / bloqueur). Instructions installation Windows/macOS/Linux. |
| `crates/sbfb-factory/src/main.rs` | Si necessaire : ajouter subcommands `Sandbox` et `PreviewCheck` qui wrappent les fonctions de gates.rs. |

### §7.3 Tests plan

Pas de test code Phase D — les livrables sont documentaires. Si des
subcommands CLI sont ajoutes, tests indirects via les fonctions de
gates.rs deja couvertes.

### §7.4 Critere d'acceptation

```bash
test -f docs/release/GATE1_TEST_PROTOCOL.md                          # exists
grep -c "Installation\|Connexion P2P\|Deploy app\|Babel\|Feed sync\|Restart\|Stabilite\|Search\|Proof Card" docs/release/GATE1_TEST_PROTOCOL.md  # 9+
```

### §7.5 Commit cible

`docs(release): Sprint 69 Phase D — Gate 1 test protocol + pilote ferme prep`

Body 9 sections obligatoires.

---

## §8 Phase E — Verification + wrap-up + audit_plan S70

### §8.1 Scope

Verification fail-fast, audit_plan S70, CLAUDE.md, SPRINT_LOG.md,
memory update, Gate 1 self-checklist.

### §8.2 Livrables

| Fichier | Description |
|---|---|
| `.planning/active/sprint69_verification.md` | Self-report fail-fast (25-30 rows). |
| `.planning/active/sprint70_audit_plan.md` | Plan audit Gate 1 + S69 review. |
| `CLAUDE.md` | Mise a jour etat S69, compteurs, carries. |
| `docs/claude/SPRINT_LOG.md` | Row S69. |

### §8.3 Tests plan

Pas de test code Phase E — artefacts documentaires.

### §8.4 Critere d'acceptation

```bash
test -f .planning/active/sprint69_verification.md  # exists
test -f .planning/active/sprint70_audit_plan.md    # exists
```

### §8.5 Commit cible

`docs(sprint69): Sprint 69 Phase E — verification + wrap-up`

Body 9 sections obligatoires.

---

## §9 Delta tests estime

| Phase | Rust | Vitest | Detail |
|---|---|---|---|
| A | +4 | +0 | preview 2 + audit_log 2 |
| B | +6 | +0 | fg8 3 + pipeline 3 |
| C | +3 | +0 | static_reader template 3 |
| D | +0 | +0 | documentation seulement |
| E | +0 | +0 | documentation seulement |
| **Total** | **+13** | **+0** | |
| **Sortie estimee** | **1432** | **279** | **~1717** |

---

## §10 Fail-fast checklist

| # | Check | Commande | Critere |
|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1430 |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok |
| 5 | release build daemon | `cargo build -p nexus-shell-daemon --release` | ok |
| 6 | release build factory | `cargo build -p sbfb-factory --release` | ok |
| 7 | npm lint | `(cd web && npm run lint)` | 0 errors |
| 8 | tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 errors |
| 9 | Vitest | `(cd web && npm run test:unit)` | >= 279 |
| 10 | npm build | `(cd web && npm run build)` | ok |
| 11 | size-limit | `(cd web && npm run size)` | 6/6 |
| 12 | scan-en-strings | `(cd web && bash scripts/scan-en-strings.sh)` | clean |
| 13 | scan-trust-wording | `bash scripts/scan-trust-wording.sh` | clean |
| 14 | sync-bridge-sdk | diff sbfb-bridge.js copies | identical |
| 15 | preview cap test | `cargo nextest run -p nexus-shell-daemon-core -E 'test(preview)' --locked` | >= 6 |
| 16 | audit_log tests | `cargo nextest run -p sbfb-factory -E 'test(audit)' --locked` | >= 2 |
| 17 | fg8 tests | `cargo nextest run -p sbfb-factory -E 'test(fg8)' --locked` | >= 3 |
| 18 | pipeline tests | `cargo nextest run -p sbfb-factory -E 'test(pipeline)' --locked` | >= 3 |
| 19 | static-reader template | `cargo nextest run -p sbfb-factory -E 'test(static_reader)' --locked` | >= 3 |
| 20 | dead_code cleanup | `grep -c "allow(dead_code)" crates/sbfb-factory/src/gates.rs` | 0 |
| 21 | THREAT_MODEL preview | `grep -q "Preview" docs/security/THREAT_MODEL.md` | present |
| 22 | GATE1 test protocol | `test -f docs/release/GATE1_TEST_PROTOCOL.md` | exists |
| 23 | count-tests script | `test -f scripts/count-tests.sh` | exists |
| 24 | factory subcommands | `cargo run -p sbfb-factory -- --help` | preview, publish, diff, scan-secrets, create, validate |
| 25 | factory no daemon dep direct | `! grep -q "nexus-shell-daemon" crates/sbfb-factory/Cargo.toml` | absent (dep via coordinator-rs) |
| 26 | verification.md | `test -f .planning/active/sprint69_verification.md` | exists |
| 27 | audit_plan S70 | `test -f .planning/active/sprint70_audit_plan.md` | exists |

---

## §11 Scope cuts

| # | Item | Sprint cible | Rationale |
|---|---|---|---|
| 1 | SearchManifest wire format + gossip | S71 | Protocole reseau. Hors scope Arc 2. D17 : S70 = consolidation. |
| 2 | Page React /factory | S70+ | CLI suffit pour S69 et le pilote. |
| 3 | @dev index tree-sitter | S70+ | Decision PO 2026-05-21 : @dev non bloquant Gate 1. |
| 4 | Template react-vite | S70+ | 3 templates suffisent. |
| 5 | CuratorVouched UI shell | S70+ | Feed vouch S67. UI post-pilote. |
| 6 | FG10 Review gate | S70+ | Lint/analyse statique post-Gate 1. |
| 7 | Fuzzing cargo-fuzz/proptest | post-Gate 1 | Hors scope fonctionnel. |
| 8 | Feed format version bump | post-launch | Pre-launch policy. |
| 9 | ProofCard comme feed op | S71+ | Candidat SearchManifest. |
| 10 | Diff engine avance | S70+ | Diff basique S68 suffit. |
| 11 | Multi-template switching UI | S70+ | CLI template choice suffit. |
| 12 | Factory update-check | post-launch | Pas de telemetrie. |
| 13 | Babel traduction live | post-launch | Reader statique dogfood. |
| 14 | iroh 1.0 upgrade | Gate 1 decision | Evalue post-S69. |

---

## §12 Risks

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | sbfb-factory dep coordinator-rs couplage | Medium | Medium | Dep read-only (verify_provenance pur). Extraire si gene. |
| R2 | Babel dogfood publish path bug | Medium | High | Test E2E documentaire Phase C. Fix immediat. |
| R3 | Pilote > 5 P0/P1 | Medium | High | Sprint fix S69.5 (roadmap v4). |
| R4 | P2-I-2 erreur humaine | Low | Low | Script count-tests parse nextest. |
| R5 | MAX_PREVIEW_ENTRIES trop bas | Low | Low | Defaut 10, augmentable. |
| R6 | Template static-reader trop minimal | Medium | Low | Squelette editable. |
| R7 | Gate 1 macOS installation | Medium | Medium | .dmg S60 existe. Workaround documente. |

---

## §13 Checkpoint de cloture

- [ ] 27/27 fail-fast verts
- [ ] 3 commits feat (Phase A, B, C) + 1 commit docs (Phase D) + 1 commit docs (Phase E)
- [ ] verification.md + audit_plan S70 ecrits
- [ ] GATE1_TEST_PROTOCOL.md ecrit
- [ ] P2-I-2 3/3 CLOSED (script + procedure)
- [ ] P2-B-1 CLOSED (MAX_PREVIEW_ENTRIES)
- [ ] P3-I-2 CLOSED (dead_code retires)
- [ ] FG8 + FG9 operationnels
- [ ] Template static-reader fonctionnel
- [ ] THREAT_MODEL.md §preview ajoute
- [ ] Memory nexus_grid_pivot.md tip + compteurs mis a jour
- [ ] SPRINT_LOG.md row S69 ajoutee
