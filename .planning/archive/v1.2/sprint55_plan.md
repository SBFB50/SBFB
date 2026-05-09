# Sprint 55 — Plan d'execution detaille

**Tip d'entree verifie** : `ee0e54c` (post-migration S54 archive).
**Date** : 2026-05-07.

---

## §1 Etat verifie a l'entree

| Check | Commande | Resultat |
|---|---|---|
| Tip | `git rev-parse --short HEAD` | `ee0e54c` |
| cargo fmt | `cargo fmt --all --check` | 0 diff |
| cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings |
| Rust nextest | `cargo nextest run --workspace --locked` | 1207 (1 flaky browse) |
| Vitest | `npm run test:unit` (web/) | 250 |
| size-limit | `npm run size` (web/) | 6/6 |

---

## §2 Decisions Day 0 (gelees, rappel)

- **D1** : Woodpecker serveur Docker Compose + Caddy auto-TLS
- **D2** : GHA validation via push + run documentation
- **D3** : LT-7 build task routing dispatcher + executor tmpdir MVP +
  quorum SHA256 validator
- **D4** : P2 batch 4 items quick (jitter, project_name, SAFETY, naming)

---

## §3 Research consulte

- context7 `woodpecker-ci/woodpecker` : Docker Compose setup,
  GitHub OAuth, Caddy reverse proxy, v3 breaking changes
- `docs/architecture/SELF_HOSTED_BUILD.md` : design doc LT-7 3 tiers
- Code lu : `dispatcher.rs` (205 LOC, submit_task + TaskDispatcher),
  `validator.rs` (226 LOC, validate_result + ResultValidator),
  `types.rs` (TaskStatus enum + TaskSubmission struct),
  `task.rs` nexus-core-rs (Task struct + metadata BTreeMap)
- `configs/systemd/` : 3 services existants (daemon, worker, coordinator)

---

## §4 Phase A — Woodpecker serveur + GHA validation (3/3 MANDATORY)

### §4.1 Scope

Deployer le serveur Woodpecker CI sur le VPS sbfb-eu. Caddy reverse
proxy avec TLS automatique (Let's Encrypt). Valider le pipeline
`.woodpecker/ci-linux.yml` end-to-end. Pusher master vers GitHub,
declencher le workflow GHA, documenter le run ID. CLOSE 2 items
3/3 MANDATORY.

### §4.2 Fichiers touches

| Fichier | Role |
|---|---|
| `configs/woodpecker/docker-compose.yml` | NEW — server + agent Docker Compose |
| `configs/woodpecker/Caddyfile` | NEW — reverse proxy TLS Let's Encrypt |
| `configs/woodpecker/.env.example` | NEW — template variables (secrets exclus) |
| `configs/systemd/woodpecker.service` | NEW — systemd service Docker Compose |
| `docs/architecture/SELF_HOSTED_BUILD.md` | Update — VPS status + Woodpecker operational |

### §4.3 Tests plan

1. `woodpecker-server` accessible HTTPS (curl health endpoint)
2. Pipeline `.woodpecker/ci-linux.yml` declenche sur push → 12 steps green
3. GHA workflow declenche sur push → fmt + clippy + tests green
4. Pas de tests unitaires Rust/Vitest pour cette phase (infra only)

### §4.4 Critere d'acceptation

- `curl -s https://<woodpecker-host>/healthz` → 200
- Pipeline Woodpecker run visible dans l'UI
- GHA run ID documente dans le commit body
- 2 items 3/3 MANDATORY documentes CLOSED

### §4.5 Commit cible

```
feat(sprint55): Sprint 55 Phase A — Woodpecker server deploy + GHA validation

## Contexte
Deploiement Woodpecker CI server sur VPS sbfb-eu (135.181.42.188).
Docker Compose (server + agent) derriere Caddy reverse proxy
(TLS Let's Encrypt automatique). GitHub OAuth app pour access
repo + webhooks. CLOSE P2-REVIEW-B-1-S52 (Woodpecker serveur
3/3 MANDATORY) et P2-REVIEW-B-2-S52 (GHA validation 3/3 MANDATORY).

## Fichiers
| Fichier | Role |
|---------|------|
| configs/woodpecker/docker-compose.yml | Docker Compose server + agent |
| configs/woodpecker/Caddyfile | Caddy TLS reverse proxy |
| configs/woodpecker/.env.example | Template variables |
| configs/systemd/woodpecker.service | systemd service |
| docs/architecture/SELF_HOSTED_BUILD.md | VPS status update |

## Tests delta cumule
Entree: 1207 Rust / 250 Vitest
Phase A: +0 Rust / +0 Vitest (infra only)
Cumule: 1207 / 250

## Scope cuts respectes
12/15 non touches. VPS TLS → fait (Caddy). systemd service → fait.

## GHA validation
Run ID: <a documenter post-push>
```

---

## §4.1 Phase A.1 — CI test fiabilite (timing-dependent cleanup)

### §4.1.1 Scope

Phase A a deploye Woodpecker CI et revele 4 bugs Linux-only +
1 test flaky timing-dependent. La Phase A.1 stabilise les 7 tests
restants qui utilisent des sleep fixes au lieu de poll+deadline,
pour que le pipeline CI soit un vrai gate de qualite (0 flaky).

### §4.1.2 Fichiers touches

| Fichier | Tests | Pattern actuel | Fix |
|---|---|---|---|
| `crates/nexus-worker-core/src/engine/runtime.rs` | `rate_limit_gate_rejects_saturated_tuple` + `rate_limit_gate_defer_preserves_task` | sleep 1.5s preuve negative | Compteur atomique tick_count ou poll+deadline |
| `crates/nexus-shell-daemon/src/validator_loop.rs` | 3 tests validator_loop | sleep 50ms puis verif DB | poll+deadline sur DB state |
| `crates/nexus-shell-daemon/src/dispatch_loop.rs` | `dispatch_loop_writes_to_doc` | sleep 100ms puis verif doc | poll+deadline sur doc entries |
| `crates/nexus-shell-daemon/src/named_pipe_server.rs` | `end_to_end_named_pipe_serves_handler_response` | sleep 50ms pour bind | poll+deadline sur server ready |

### §4.1.3 Tests plan

Pas de nouveaux tests — transformation des 7 tests existants
de sleep-based vers poll+deadline ou compteur atomique. Les
assertions restent les memes, seul le mecanisme de synchronisation
change.

### §4.1.4 Critere d'acceptation

- `cargo nextest run --workspace --locked` : 0 flaky sur WSL Linux
- `cargo nextest run --workspace --locked` : 0 flaky sur Windows
- 0 `thread::sleep` comme mecanisme de synchronisation dans les
  7 tests cibles (grep verification)

### §4.1.5 Commit cible

```
feat(sprint55): Sprint 55 Phase A.1 — CI test fiabilite (7 timing-dependent tests stabilises)

## Contexte
Phase A a deploye Woodpecker CI et revele que les tests timing-
dependent echouent sur du hardware rapide (VPS Docker) mais
passent en local WSL (lent I/O Hyper-V). 7 tests utilisaient
des sleep fixes comme mecanisme de synchronisation — remplaces
par poll+deadline ou compteurs atomiques.

## Fichiers
[table]

## Tests delta cumule
Entree: 1207 Rust / 250 Vitest
Phase A.1: +0 Rust / +0 Vitest (refactoring tests existants)
Cumule: 1207 / 250

## Scope cuts respectes
15/15 non touches.
```

---

## §5 Phase B — LT-7 build executor + dispatcher routing

### §5.1 Scope

Ajouter le support `task_type: "build"` dans le protocole.
Dispatcher routing pour differencier build tasks des inference
tasks. Build executor MVP dans un tmpdir (clone + cargo build +
SHA256). Pas de quorum encore (Phase C).

### §5.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/dispatcher.rs` | Routing task_type "build" : relaxer validation prompt/model pour build tasks |
| `crates/nexus-coordinator-rs/src/types.rs` | BuildSubmission struct OU validation conditionnelle dans TaskSubmission |
| `crates/nexus-worker-core/src/build_executor.rs` | NEW — clone repo, cargo build, SHA256, ResultEntry |
| `crates/nexus-worker-core/src/lib.rs` | pub mod build_executor |
| `crates/nexus-worker-core/Cargo.toml` | dep sha2 (SHA256 calcul) |

### §5.3 Tests plan

1. `test_submit_build_task_accepts_empty_prompt` — build task sans
   prompt est valide
2. `test_submit_build_task_requires_metadata_keys` — build.repo,
   build.commit, build.binary requis dans metadata
3. `test_build_executor_sha256_matches_known_binary` — SHA256 d'un
   binaire connu (fixture)
4. `test_build_executor_rejects_missing_repo` — erreur si build.repo
   manquant

### §5.4 Critere d'acceptation

- `cargo nextest run -p nexus-coordinator-rs --locked` → +2 tests
- `cargo nextest run -p nexus-worker-core --locked` → +2 tests
- Build executor compile et produit un SHA256 sur un repo local

### §5.5 Commit cible

```
feat(sprint55): Sprint 55 Phase B — LT-7 build executor + dispatcher task_type routing

## Contexte
Foundation LT-7 Tier 2 : support task_type "build" dans le
protocole SBFB. Le dispatcher route les build tasks differemment
(relaxation validation prompt/model, validation metadata build.*).
Le build executor (nexus-worker-core/build_executor.rs) clone le
repo dans un tmpdir, execute cargo build --release --locked, et
calcule le SHA256 du binaire produit.

## Fichiers
[table]

## Tests delta cumule
Entree: 1207 Rust / 250 Vitest
Phase B: +4 Rust / +0 Vitest
Cumule: 1211 / 250

## Scope cuts respectes
15/15 non touches.
```

---

## §6 Phase C — LT-7 quorum SHA256 + integration test

### §6.1 Scope

Implementer la verification quorum pour les build tasks. Le
validator accumule N resultats (redundancy_factor) avant de
comparer les SHA256. Majorite identique → accepte. Divergence →
rejet. Integration test build task E2E.

### §6.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/validator.rs` | Quorum accumulation : AwaitingQuorum status, collect N results, SHA256 comparison |
| `crates/nexus-coordinator-rs/src/types.rs` | TaskStatus::AwaitingQuorum variant |
| `crates/nexus-coordinator-rs/src/db.rs` | Migration : table task_results (task_id, worker_id, result_hash, sha256) + queries accumulation |
| `crates/nexus-coordinator-rs/src/dispatcher.rs` | Build task dispatch avec redundancy_factor default 3 |

### §6.3 Tests plan

1. `test_build_result_transitions_to_awaiting_quorum` — premier
   resultat build → AwaitingQuorum (pas Completed)
2. `test_quorum_majority_sha256_accepts` — 2/3 SHA256 identiques →
   Completed
3. `test_quorum_divergence_rejects` — 3 SHA256 differents → Rejected
4. `test_quorum_single_outlier_detected` — 2 match + 1 divergent →
   Completed + outlier logged
5. `test_inference_task_bypasses_quorum` — task_type "inference"
   garde le comportement single-result existant

### §6.4 Critere d'acceptation

- `cargo nextest run -p nexus-coordinator-rs --locked` → +5 tests
- Quorum 2/3 SHA256 match dans tests
- Inference tasks non affectes (backward compatible)

### §6.5 Commit cible

```
feat(sprint55): Sprint 55 Phase C — LT-7 quorum SHA256 validation + build E2E test

## Contexte
Verification quorum pour les build tasks : le validator accumule
N resultats (redundancy_factor) et compare les SHA256. 2/3 match
→ accepted. 0 match → rejected + alerte. Le statut AwaitingQuorum
est un etat intermediaire entre Dispatched et Completed. Les
inference tasks gardent le comportement single-result existant.

## Fichiers
[table]

## Tests delta cumule
Entree: 1207 Rust / 250 Vitest
Phase C: +5 Rust / +0 Vitest
Cumule: 1216 / 250

## Scope cuts respectes
15/15 non touches.
```

---

## §7 Phase D — P2 batch quick carries

### §7.1 Scope

Resoudre 4 items P2 quick pour prevenir l'accumulation.

### §7.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon/src/runtime.rs` | jitter ±15s sur republish timer (ligne ~1015) |
| `crates/nexus-shell-daemon/src/invite_api.rs` | Extraire "sbfb" hardcode vers constante |
| `crates/nexus-worker-core/src/invite.rs` | INVITE_VERSION → INVITE_FORMAT_VERSION, u8 → u16 |
| `crates/nexus-launcher/src/main.rs` | // SAFETY: sur libc::kill SIGTERM (ligne 515) |
| `crates/nexus-test-harness/src/lib.rs` | // SAFETY: sur libc::kill SIGINT (ligne 147) |
| `crates/nexus-shell-daemon/src/named_pipe_server.rs` | // SAFETY: sur blocs unsafe Win32 FFI |

### §7.3 Tests plan

1. Pas de nouveaux tests (changements mecaniques : commentaires,
   renommage, constante). Les tests existants valident la non-
   regression (invite round-trip, gossip timer).

### §7.4 Critere d'acceptation

- `cargo nextest run --workspace --locked` → 1216+ (0 regression)
- `grep "INVITE_FORMAT_VERSION" crates/nexus-worker-core/src/invite.rs` → present
- `grep "// SAFETY:" crates/nexus-launcher/src/main.rs` → present sur libc::kill
- `grep "jitter\|thread_rng\|gen_range" crates/nexus-shell-daemon/src/runtime.rs` → present

### §7.5 Commit cible

```
feat(sprint55): Sprint 55 Phase D — P2 batch quick carries (jitter + SAFETY + naming)

## Contexte
4 items P2 resolus : (1) jitter ±15s sur republish timer 45s
(prevention thundering-herd), (2) project_name "sbfb" hardcode →
constante, (3) // SAFETY: comments sur unsafe FFI pre-existants,
(4) INVITE_VERSION → INVITE_FORMAT_VERSION rename + u8→u16.

## Fichiers
[table]

## Tests delta cumule
Entree: 1207 Rust / 250 Vitest
Phase D: +0 Rust / +0 Vitest (mecaniques)
Cumule: 1216 / 250

## Scope cuts respectes
15/15 non touches.
```

---

## §8 Phase E — Wrap-up + verification + audit plan S56

### §8.1 Scope

Cloturer le sprint. Mettre a jour CLAUDE.md, HARDENING_ROADMAP,
ecrire verification.md et sprint56_audit_plan.md.

---

## §9 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1216, 0 fail | |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | |
| 6 | npm lint | `npm run lint` (web/) | 0 error | |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | |
| 8 | Vitest | `npm run test:unit` (web/) | >= 250 | |
| 9 | npm build | `npm run build` (web/) | ok | |
| 10 | size-limit | `npm run size` (web/) | 6/6 | |
| 11 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | |
| 12 | Playwright | `npx playwright test` (web/) | >= 42 | |
| 13 | Woodpecker health | `curl -s https://<host>/healthz` | 200 | |
| 14 | Pipeline E2E | Woodpecker run visible | green | |
| 15 | GHA run | `gh run list` | run ID documente | |
| 16 | Phase A preflight G8 | verdict | EXECUTE | |
| 17 | Phase A review | verdict | PASS | |
| 18 | Phase B preflight G8 | verdict | EXECUTE | |
| 19 | Phase B review | verdict | PASS | |
| 20 | Phase C preflight G8 | verdict | EXECUTE | |
| 21 | Phase C review | verdict | PASS | |
| 22 | Phase D preflight G8 | verdict | EXECUTE | |
| 23 | Phase D review | verdict | PASS | |
| 24 | build task dispatch | test_submit_build_task | pass | |
| 25 | quorum SHA256 | test_quorum_majority_sha256 | pass | |
| 26 | INVITE_FORMAT_VERSION | grep | present | |
| 27 | Scope cuts | 15/15 respectes | all checked | |
| 28 | Delta tests | cumule documente | documented | |

---

## §10 Git plan

| # | Commit | Scope |
|---|---|---|
| 0 | `chore(planning): Sprint 55 kickoff + plan + design review + migration` | Planning docs |
| 1 | `feat(sprint55): Sprint 55 Phase A — Woodpecker server deploy + GHA validation` | Infra CI |
| 1.1 | `feat(sprint55): Sprint 55 Phase A.1 — CI test fiabilite` | CI fiabilite |
| 2 | `feat(sprint55): Sprint 55 Phase B — LT-7 build executor + dispatcher routing` | Build protocol |
| 3 | `feat(sprint55): Sprint 55 Phase C — LT-7 quorum SHA256 validation + build E2E test` | Quorum |
| 4 | `feat(sprint55): Sprint 55 Phase D — P2 batch quick carries` | Dette |
| 5 | `chore(sprint55): Phase E — wrap-up + verification + audit plan S56` | Cloture |

---

## §11 Scope cuts (copie kickoff §7)

1. LT-7 cross-platform builds — Tier 3
2. LT-7 toolchain bundle iroh-blobs — Tier 3
3. LT-7 auto-update reseau — Tier 3
4. LT-7 build log streaming — Tier 3
5. LT-7 podman rootless sandbox — S56+
6. Outbox persistant fichier — S56 (3/3 MANDATORY)
7. Browse_request rate-limit — S56 (3/3 MANDATORY)
8. Test E2E multi-noeuds — S56
9. Windows test cfg(unix) — S56
10. forbid-deny-doc PATTERNS — S56
11. Lightcheck edition faux-positif — S56
12. rustfmt drift sessions — S56
13. Flaky browse test — S56
14. Pre-v1.0 apps Protocol Explorer + Ideas Hub — S56-S57
15. LT-1 Kudos-v2 fairness reform — S58+

---

## §12 Risks (copie kickoff §9)

| # | Risque | Mitigation |
|---|---|---|
| R1 | VPS RAM 8GB insuffisante | Scope cut Phase B/C → S56 |
| R2 | GitHub OAuth intervention utilisateur | Documentation pas-a-pas |
| R3 | SHA256 non-determinisme residuel | SOURCE_DATE_EPOCH, documenter |
| R4 | tmpdir pas isole | MVP limitation documentee |
| R5 | GHA flaky | Re-run, documenter |
| R6 | DB migration quorum | ALTER TABLE simple |

---

## §13 Checkpoint de cloture

Sprint 55 est clos quand :
1. Woodpecker serveur accessible HTTPS (Phase A)
2. GHA run documente (Phase A)
3. `task_type: "build"` route dans dispatcher (Phase B)
4. Build executor produit SHA256 valide (Phase B)
5. Quorum 2/3 SHA256 match dans tests (Phase C)
6. 4 P2 quick resolus (Phase D)
7. 28+ rows fail-fast verts (Phase E)
8. sprint56_audit_plan.md ecrit (Phase E)

---

## §14 Dependencies inter-phases

```
Phase A (infra) ← aucune dependance code
Phase A.1 (CI fiabilite) ← depend Phase A (pipeline deploye)
Phase B (build executor) ← aucune dependance Phase A/A.1
Phase C (quorum) ← depend Phase B (build task routing)
Phase D (P2 batch) ← aucune dependance
Phase E (wrap-up) ← depend toutes phases
```

Phase A.1 stabilise les tests pour que le pipeline CI de Phase A
soit un vrai gate. Phases B et D sont independantes de A.1.
Phase C depend de B.
