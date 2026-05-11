# Sprint 59 — Plan d'execution detaille

**Ecrit** : 2026-05-11 (post-kickoff S59).
**Tip master** : `80ec664`.

---

## §1 Etat verifie a l'entree

| Metrique | Valeur |
|---|---|
| Tip master | `80ec664` |
| Rust nextest | 1240 pass, 0 fail |
| Rust doctests | 6 pass, 1 ignored |
| cargo fmt | 0 diff |
| cargo clippy | 0 warnings |
| release build | ok |
| Vitest | 256 pass |
| npm lint | 0 error |
| tsc | 0 error |
| npm build | ok |
| size-limit | 6/6 |
| scan-en-strings | clean |

---

## §2 Decisions Day 0 (gelees, rappel)

- **D1** : LT-1 Kudos-v2 — log-utility `floor(1000 * log2(1 + tokens))`
  + EMA decay alpha=0.97 sur effective_score()
- **D2** : Verified deploy E2E — SBFB.json exemples + E2E test gate
  + frontend Deploy page
- **D3** : Launcher error UX — raw FFI MessageBoxW cfg(windows) +
  eprintln fallback
- **D4** : Storage carries — validation is_replicated_app() +
  governor GCRA rate-limit 10 writes/min/author/app

---

## §3 Research consulte

- `.planning/research/S21_research_fair_allocation_mechanisms.md`
  — combo 3 couches (log-utility + DRF + EMA). S59 implementa A+C.
- context7 `/microsoft/windows-rs` — MessageBoxW FFI patterns.
- `crates/nexus-shell-daemon/src/deploy.rs` — deploy E2E backend
  complet (679 LOC, clone → verify → zip → provenance → blob →
  gossip).
- `crates/nexus-coordinator-rs/src/kudos_ledger.rs` — credit()
  flat, 237 LOC, 9 tests.
- `crates/nexus-coordinator-rs/src/fairness.rs` — compute_gini()
  / compute_top_k_share() / compute_churn_rate(), 103 LOC.
- `crates/nexus-shell-daemon/src/browse_limiter.rs` — Governor
  GCRA pattern de reference pour D4.

---

## §4 Dependencies inter-phases

```
Phase A (Kudos-v2)  ─── independante
Phase B (Deploy)    ─── independante
Phase C (Launcher + storage) ─── independante
Phase D (Wrap-up)   ─── depend A + B + C
```

Aucune dependance sequentielle entre A, B, C. L'ordre est choisi
par priorite (LT-1 pre-v1.0 d'abord).

---

## §5 Phase A — LT-1 Kudos-v2 fairness reform

### §5.1 Scope

Implementer les Couches A et C du combo fairness :
1. **Log-utility transform** dans credit() — diminishing returns
2. **EMA effective score** — decay exponentiel des contributions
   anciennes
3. **API integration** — get_project_kudos() retourne scores
   effectifs
4. **Fairness metrics** — Gini/top-k consomment scores effectifs

Le hash chain reste inchange (entries DB gardent le montant
log-transforme). Les entries existantes (pre-S59) sont deja
log-transformees par la nouvelle formule — pas de migration
physique, la transformation est appliquee au moment du credit().

### §5.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/kudos_ledger.rs` | credit() log-utility + effective_score() EMA + get_project_kudos() effective |
| `crates/nexus-coordinator-rs/src/fairness.rs` | compute_gini/top_k/churn consomment effective scores |
| `crates/nexus-coordinator-rs/src/kudos_api.rs` | API handlers passent now_secs a get_project_kudos |
| `crates/nexus-coordinator-rs/src/types.rs` | KudosEntry inchange (amount reste u64) |

### §5.3 Tests plan

1. `test_credit_log_utility` : credit(tokens=1) et credit(tokens=100),
   verifier que ratio < 10x (log compression)
2. `test_credit_log_utility_zero` : credit(tokens=0) → amount >= 1
   (floor + max(1))
3. `test_effective_score_decays` : 2 entries, une vieille (90j),
   une recente (1j). Score effectif recente > vieille.
4. `test_effective_score_recent_only` : entry creee maintenant →
   score ≈ amount (pas de decay).
5. `test_get_project_kudos_effective` : 2 contributors, verifier
   que totaux sont EMA-weighted.
6. `test_chain_integrity_after_log` : credit + verify_chain toujours
   ok avec montants log-transformes.
7. `test_fairness_gini_with_effective` : Gini consomme effective
   scores (integration fairness.rs).

### §5.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-coordinator-rs --locked
cargo clippy -p nexus-coordinator-rs --all-targets --locked -- -D warnings
cargo fmt --all --check
```

7 nouveaux tests passent. credit() utilise log-utility.
get_project_kudos() retourne EMA-weighted scores. verify_chain ok.

### §5.5 Commit cible

```
feat(sprint59): Sprint 59 Phase A — LT-1 Kudos-v2 log-utility + EMA fairness reform

## Contexte
LT-1 pre-v1.0 (9 sprints carry depuis S50). La formule flat
amount = tokens_generated produit un Matthew effect non borne.
Couche A (log-utility) + Couche C (EMA decay) du combo recommande
research S21. DRF (Couche B) reporte post-v1.0 (pas d'infra
multi-ressource).

## Fichiers
| Fichier | Role |
|---------|------|
| kudos_ledger.rs | credit() log2 + effective_score() EMA alpha=0.97 + get_project_kudos() effective |
| fairness.rs | compute_gini/top_k/churn passent par effective_score |
| kudos_api.rs | handler passe now_secs |

## Delta tests
| Suite | Avant | Apres | Delta |
|-------|-------|-------|-------|
| Rust workspace | 1240 | 1247 | +7 |

## Scope cuts respectes
- DRF (Couche B) — post-v1.0
- Kudos-weighted voting — post-v1.0
- quality/trust factors — post-v1.0

## CLOSE
- LT-1 Kudos-v2 fairness reform (ROADMAP_COMMITMENTS pre-v1.0)
```

---

## §6 Phase B — Verified deploy E2E + seed SBFB.json + Deploy page

### §6.1 Scope

1. **SBFB.json** pour les 2 apps exemples
2. **E2E integration test** exercant le flow complet
3. **Frontend Deploy page** dans le shell React

### §6.2 Fichiers touches

| Fichier | Role |
|---|---|
| `examples/sbfb-explorer/SBFB.json` | NEW — metadata deploy (node_id placeholder) |
| `examples/sbfb-ideas/SBFB.json` | NEW — metadata deploy |
| `crates/nexus-shell-daemon/src/deploy.rs` | E2E test module |
| `web/src/pages/Deploy.tsx` | NEW — formulaire deploy React |
| `web/src/App.tsx` | route /deploy |
| `web/src/components/sidebar/` | lien Deploy dans navigation |
| `scripts/sync-bridge-sdk.sh` | ajout SBFB.json dans la copie |

### §6.3 Tests plan

1. `test_deploy_from_repo_e2e` (gate SBFB_INTEGRATION=1) : creer
   repo git local → init + add index.html + SBFB.json → commit →
   POST deploy-from-repo → verifier response deployed=true + hash
   non vide + provenance_hash non vide + commit_sha correct.
2. `test_deploy_sbfb_json_missing` : repo sans SBFB.json →
   400 Bad Request.
3. `test_deploy_node_id_mismatch` : SBFB.json avec mauvais
   node_id → 400 Bad Request.
4. Vitest : `Deploy.test.tsx` — render form, submit mock, display
   result.

### §6.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-shell-daemon --locked
cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && npm run test:unit
```

E2E test vert (SBFB_INTEGRATION=1). Deploy page render ok.
SBFB.json present dans les 2 apps exemples.

### §6.5 Commit cible

```
feat(sprint59): Sprint 59 Phase B — Verified deploy E2E + seed SBFB.json + Deploy page

## Contexte
Le backend deploy.rs (679 LOC, S42) est complet mais jamais teste
de bout en bout. SBFB.json ajoute aux apps exemples pour les
rendre deployables. Frontend Deploy page ferme la boucle UX
deploy → browse → run.

## Fichiers
| Fichier | Role |
|---------|------|
| examples/sbfb-explorer/SBFB.json | NEW seed metadata |
| examples/sbfb-ideas/SBFB.json | NEW seed metadata |
| deploy.rs | +3 tests E2E (gate SBFB_INTEGRATION) |
| Deploy.tsx | NEW React deploy form |
| App.tsx | route /deploy |

## Delta tests
| Suite | Avant | Apres | Delta |
|-------|-------|-------|-------|
| Rust workspace | 1247 | 1250 | +3 |
| Vitest | 256 | 258 | +2 |

## Scope cuts respectes
- Keyoxide verification — S60/post-v1.0
- Auto-deploy webhooks — post-v1.0
- Build from source — hors scope (LT-7 separe)
```

---

## §7 Phase C — Launcher readiness + storage carries

### §7.1 Scope

1. **MessageBoxW** raw FFI pour erreurs fatales launcher
2. **Storage carries** validation + rate-limit

### §7.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-launcher/src/main.rs` | error_msgbox() + appels dans spawn_daemon path |
| `crates/nexus-shell-daemon/src/storage_api.rs` | validation is_replicated_app() dans storage_join |
| `crates/nexus-shell-daemon/src/http.rs` | rate-limit middleware storage endpoints (governor GCRA) |
| `crates/nexus-shell-daemon-core/src/storage_limiter.rs` | NEW — StorageWriteLimiter keyed per-author per-app |

### §7.3 Tests plan

1. `test_storage_join_rejects_non_replicated` : storage_join avec
   app_name non replique → 400 Bad Request.
2. `test_storage_join_accepts_replicated` : storage_join avec
   app_name replique → 200 OK (ou 2xx).
3. `test_storage_rate_limit_under_quota` : 5 writes rapides →
   tous acceptes.
4. `test_storage_rate_limit_over_quota` : 15 writes en 1s →
   certains rejetes (429 Too Many Requests).

### §7.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-shell-daemon -p nexus-shell-daemon-core -p nexus-launcher --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

MessageBoxW compile sur Windows (cfg-gate). Storage validation
et rate-limit testes.

### §7.5 Commit cible

```
feat(sprint59): Sprint 59 Phase C — Launcher MessageBox + storage validation + rate-limit

## Contexte
Le launcher a windows_subsystem="windows" (pas de console) —
sans MessageBox, les erreurs daemon sont invisibles. Les 2 carries
storage (P2-STORAGE-JOIN-VALIDATE 1/3 + P2-STORAGE-ANTISPAM 1/3)
sont resolus avant 3/3 MANDATORY en S60.

## Fichiers
| Fichier | Role |
|---------|------|
| main.rs (launcher) | error_msgbox() raw FFI cfg(windows) |
| storage_api.rs | validation is_replicated_app in storage_join |
| http.rs | governor GCRA storage rate-limit |
| storage_limiter.rs | NEW StorageWriteLimiter |

## Delta tests
| Suite | Avant | Apres | Delta |
|-------|-------|-------|-------|
| Rust workspace | 1250 | 1254 | +4 |

## Scope cuts respectes
- macOS/Linux MessageBox — S60
- Validation schema JSON per-app — post-v1.0

## CLOSE
- P2-STORAGE-JOIN-VALIDATE (2/3 → CLOSED)
- P2-STORAGE-ANTISPAM (2/3 → CLOSED)
```

---

## §8 Phase D — Wrap-up + verification + audit plan S60

### §8.1 Scope

Livrables de cloture sprint :
1. verification.md (28+ fail-fast rows)
2. sprint60_audit_plan.md (7+ tracks)
3. CLAUDE.md update S59 CLOSED
4. HARDENING_ROADMAP update last_validated S59
5. SPRINT_LOG row S59
6. Memory nexus_grid_pivot.md tip update

### §8.2 Commit cible

```
chore(sprint59): Phase D — wrap-up + verification + audit plan S60
```

---

## §9 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1254, 0 fail | |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | |
| 6 | npm lint | `npm run lint` (web/) | 0 error | |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | |
| 8 | Vitest | `npm run test:unit` (web/) | >= 258 | |
| 9 | npm build | `npm run build` (web/) | ok | |
| 10 | size-limit | `npm run size` (web/) | 6/6 | |
| 11 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | |
| 12 | Phase A preflight G8 | verdict | EXECUTE | |
| 13 | Phase A review | verdict | PASS | |
| 14 | Phase B preflight G8 | verdict | EXECUTE | |
| 15 | Phase B review | verdict | PASS | |
| 16 | Phase C preflight G8 | verdict | EXECUTE | |
| 17 | Phase C review | verdict | PASS | |
| 18 | LT-1 Kudos-v2 CLOSE | credit() log-utility | present | |
| 19 | LT-1 effective_score | EMA alpha=0.97 | present | |
| 20 | SBFB.json seed apps | examples/*/SBFB.json | present | |
| 21 | Deploy E2E test | test_deploy_from_repo_e2e | present | |
| 22 | Deploy page | web/src/pages/Deploy.tsx | present | |
| 23 | MessageBoxW | error_msgbox() cfg(windows) | present | |
| 24 | STORAGE-JOIN-VALIDATE | is_replicated_app check | present | |
| 25 | STORAGE-ANTISPAM | governor GCRA storage | present | |
| 26 | Scope cuts | 14/14 respectes | all checked | |
| 27 | Delta tests | cumule documente | documented | |
| 28 | Sync bridge SDK | `bash scripts/sync-bridge-sdk.sh` | exit 0 | |

---

## §10 Git plan

| # | Commit | Scope |
|---|---|---|
| 1 | `feat(sprint59): Sprint 59 Phase A — LT-1 Kudos-v2 log-utility + EMA fairness reform` | kudos_ledger.rs + fairness.rs + kudos_api.rs |
| 2 | `feat(sprint59): Sprint 59 Phase B — Verified deploy E2E + seed SBFB.json + Deploy page` | SBFB.json + deploy.rs tests + Deploy.tsx |
| 3 | `feat(sprint59): Sprint 59 Phase C — Launcher MessageBox + storage validation + rate-limit` | main.rs launcher + storage_api.rs + http.rs + storage_limiter.rs |
| 4 | `chore(sprint59): Phase D — wrap-up + verification + audit plan S60` | docs planning |

---

## §11 Scope cuts (copie kickoff §7)

1. AppStorage Phase 2 (namespace per manifest) — S60+
2. AppStorage Phase 3 (optimisations, purge) — post-v1.0
3. Kudos-v2 DRF (Couche B multi-ressource) — post-v1.0
4. Kudos-weighted voting — post-v1.0
5. Keyoxide identity verification in deploy — S60/post-v1.0
6. NSIS/WiX installer — S60
7. Tray icon — S60
8. Frontend P2P distribution — S60
9. Protocol Explorer F3 (gossip stats avance) — S60+
10. Protocol Explorer F4 (tutoriel interactif) — post-v1.0
11. Ideas Hub F3 (lier repos Git) — S60
12. Ideas Hub F4-F5 (groupes, integration) — post-v1.0
13. Ticket Write rotation dynamique (Option B/C) — post-v1.0
14. LT-7 Tier 3 validation controlee — S60 pre-tag

---

## §12 Risks (copie kickoff §9)

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Log-utility change scores existants | Medium | Low | Pre-launch, 0 user externe |
| R2 | EMA alpha trop agressif/doux | Low | Low | Constante nommee, ajustable |
| R3 | Deploy E2E test depend de git | Medium | Medium | Gate SBFB_INTEGRATION |
| R4 | Frontend Deploy expose API | Low | Low | Deja expose (bearer auth) |
| R5 | MessageBoxW bloque thread | Low | Low | Appele avant exit uniquement |
| R6 | Rate-limit storage restrictif | Low | Medium | 10/min largement suffisant |

---

## §13 Checkpoint de cloture

1. 28/28 fail-fast rows vertes
2. 4 commits (3 feat + 1 chore) landed sur master
3. LT-1 Kudos-v2 CLOSED dans ROADMAP_COMMITMENTS
4. P2-STORAGE-JOIN-VALIDATE CLOSED
5. P2-STORAGE-ANTISPAM CLOSED
6. verification.md + sprint60_audit_plan.md ecrits
7. CLAUDE.md + SPRINT_LOG + HARDENING_ROADMAP mis a jour
8. Memory nexus_grid_pivot.md tip a jour
