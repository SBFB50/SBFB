# Sprint 22 Phase C — nexus-phase-auditor review

HEAD pre-commit: `9676bd9` (tip master avant commit Phase C)
Draft commit body: "feat(sprint22): Phase C — Sybil-resistance composition 3 couches (age witness + contributor attestation + Couche 3 RFC)"
Timebox iteration 1 : 55m / iteration 2 (post-fix P1) : 15m

## Verdict : PASS

0 P0, 0 P1. 1 P2 carry-over (delta tests annonce vs reel), 1 P3 cosmétique (DOMAIN re-export).

---

## Retrospective iteration 2 — P1 resolution

Le P1 initial identifiait l'absence du proxy `crates/nexus-shell-daemon/src/http.rs` exige par le plan §6.2 lignes 448-450. Le fix est verifie comme suit (Read + Grep sur les fichiers cites) :

**`crates/nexus-shell-daemon/Cargo.toml` ligne 94** : `reqwest = { workspace = true }` present — workspace reqwest 0.12, features json + rustls-tls (lignes 88-94, commentaire phase C explicite).

**`crates/nexus-shell-daemon/src/http.rs`** :
- `DaemonHttpState` lignes 131-146 : champs `coord_http_client: reqwest::Client` + `coord_base_url: String` presents, documentes.
- Route `GET /api/contributor/verify/{project_id}/{node_id_hex}` ligne 233-236 : montee dans `authed_routes` (derriere `auth_required` middleware — authed conforme).
- Handler `proxy_contributor_verify` lignes 751-813 : guard hex daemon-side `is_64_lowercase_hex` lignes 759 avant forward (400 si invalide), token `X-SBFB-Token` propage ligne 778, 502 Bad Gateway si coord unreachable lignes 802-810, forward body JSON lignes 783-798.
- Constantes publiques `COORD_BASE_URL_ENV = "SBFB_COORD_URL"` ligne 724 + `DEFAULT_COORD_BASE_URL = "http://127.0.0.1:8787"` ligne 731.
- Helper `resolve_coord_base_url()` lignes 738-741 : lit env + trim trailing slash + fallback default.

**`crates/nexus-shell-daemon/src/runtime.rs` lignes 475-479** : construction `DaemonHttpState` wire le client reqwest avec timeout 2s (`Duration::from_secs(2)` ligne 476) + `coord_base_url` via `resolve_coord_base_url()` ligne 479.

**3 tests ajoutes** (lignes 1107-1165) :
- `proxy_contributor_verify_rejects_non_hex_path_params` (l.1108) : short-circuit daemon-side 400, port 9 unreachable = preuve que la guard precede le reseau.
- `proxy_contributor_verify_bad_gateway_when_coord_unreachable` (l.1132) : port 9 → 502, prouve la branche connection-error ne panic pas.
- `resolve_coord_base_url_respects_env_var` (l.1155) : env wins + trailing slash trim + fallback.

**Conclusion** : P1 leve. Route presente, authed, guard hex, token forward, 502 wired, 3 tests couvrant les branches critiques.

---

## Dimensions

### Security

- [x] **unsafe/unwrap** : `#[forbid(unsafe_code)]` en ligne 31 de `crates/nexus-core-rs/src/lib.rs`. Aucun `unwrap()` non-justifie dans le diff — tous les sites utilisent `expect("sign succeeds")` dans les tests ou retournent `Result`. `crates/nexus-core-py/src/lib.rs` reste `#[forbid(unsafe_code)]` (ligne 28). 0 finding.
- [x] **JCS canonical** : `canonical_bytes(&payload, DOMAIN_AGE_WITNESS_V1)` dans `age_witness.rs:191,212`. `canonical_bytes(&unsigned, DOMAIN_CONTRIBUTOR_ATTESTATION_V1)` dans `contributor.rs:261,343`. Aucune utilisation de `serde_json::to_string` pour les payloads signes. Conforme P31 PATTERNS.md.
- [x] **Domain separation nouveaux tags** : `DOMAIN_AGE_WITNESS_V1 = b"nexus-age-witness-v1"` (`canonical.rs:150`) et `DOMAIN_CONTRIBUTOR_ATTESTATION_V1 = b"nexus-contributor-attestation-v1"` (`canonical.rs:166`) — distincts des 9 domaines existants. Test explicite `canonical_bytes_are_domain_separated_from_other_payloads` dans `age_witness.rs:363-375`.
- [x] **Loopback security** : `api/contributor.py` derriere `LoopbackAuthMiddleware` (sprint 16). Input validation `_validate_hex()` sur `project_id` et `node_id_hex` avant query SQLite. 0 injection.
- [x] **Proxy guard hex daemon-side** : `is_64_lowercase_hex` (`http.rs:815-819`) — 64 chars, ASCII hexdigit, no uppercase. Short-circuit 400 avant toute I/O reseau.
- [x] **Token forward proxy** : `X-SBFB-Token` propage (`http.rs:777-779`), tous les autres headers stripes (pas de forwarding Cookie/Host/Accept drift).
- [x] **SQLite** : `contributor_registry.py` utilise `sqlite3.connect` avec parametres lies (`?` placeholders). WAL mode active. 0 injection.
- [x] **Pas de secrets hardcodes** : grep sur le diff — 0 pattern `AKIA|ghp_|pat_|sbfb_` trouve.
- [x] **Future timestamp attack** : `age_witness.rs:228-232` rejette `first_seen_ts > now_ts` avec `AgeWitnessError::FutureTimestamp`.
- [x] **Sybil chain** : `MIN_WITNESS_AGE_DAYS = 30` (ligne 60 `age_witness.rs`) — empeche bootstrap chain temoin->temoigne par un nouveau noeud.

**Verdict Security** : PASS. 0 finding.

---

### Patterns

- [x] **P14 — JCS canonical bytes pour tout payload signe** : respecte.
- [x] **P20 — Domain separation tag par famille** : 2 nouveaux tags additifs, correctement documentes dans `canonical.rs`.
- [x] **P23 — Error type distinct par module** : `AgeWitnessError` + `ContributorAttestationError`, chacun impl `Display`, `Error`, `From<..> for NexusError`.
- [x] **P30 — PyO3 binding sans panic** : `build_contributor_attestation`, `verify_contributor_attestation`, `verify_age_witness` retournent `PyResult<_>`. 0 `.unwrap()`.
- [x] **P31 — Hot-reload TOML watcher** : `BootstrapAllowlistWatcher::spawn()` miroir exact de `PowPolicyWatcher` S20 Phase C — `notify::RecommendedWatcher`, debounce 50ms, fail-closed.
- [x] **P3 — Result propagation** : fonctions publiques Couche 1 + Couche 2 retournent `Result<_>`. 0 `.unwrap()` expose.

**Verdict Patterns** : PASS. 0 finding.

---

### Working tree audit (G5)

Fichiers du diff Phase C categorises :

| Fichier | Categorie | Verdict |
|---|---|---|
| `crates/nexus-core-rs/src/attestations/` (mod.rs, age_witness.rs, contributor.rs) | PHASE | attendu plan §6.2 |
| `crates/nexus-core-rs/src/canonical.rs` | PHASE | attendu plan §6.2 |
| `crates/nexus-core-rs/src/curator.rs` | PHASE | attendu plan §6.2 |
| `crates/nexus-core-rs/src/gossip.rs` | PHASE | attendu plan §6.2 |
| `crates/nexus-core-rs/src/lib.rs` | PHASE | re-exports nouveaux modules |
| `crates/nexus-shell-daemon-core/src/bootstrap_allowlist.rs` | PHASE | attendu plan §6.2 |
| `crates/nexus-shell-daemon-core/src/lib.rs` | PHASE | expose bootstrap_allowlist |
| `crates/nexus-core-py/src/lib.rs` | PHASE | PyO3 bindings attendus |
| `crates/nexus-shell-daemon/src/http.rs` | PHASE | proxy Couche 2 (fix P1 iteration 2) |
| `crates/nexus-shell-daemon/src/runtime.rs` | PHASE | wire coord_http_client (fix P1 iteration 2) |
| `crates/nexus-shell-daemon/Cargo.toml` | PHASE | reqwest dep (fix P1 iteration 2) |
| `packages/nexus-coordinator/src/nexus_coordinator/api/contributor.py` | PHASE | attendu plan §6.2 |
| `packages/nexus-coordinator/src/nexus_coordinator/contributor_registry.py` | PHASE | attendu plan §6.2 |
| `packages/nexus-coordinator/src/nexus_coordinator/coordinator.py` | PHASE | hook contributor_registry |
| `packages/nexus-coordinator/src/nexus_coordinator/paths.py` | PHASE | `contributor_registry_path()` |
| `packages/nexus-coordinator/src/nexus_coordinator/api/app.py` | PHASE | mont contributor_router |
| `packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py` | PHASE | hook attestation post-provenance |
| `packages/nexus-coordinator/tests/test_deploy.py` | PHASE | assertion contributor_registry ajoutee |
| `packages/nexus-coordinator/tests/test_contributor_registry.py` | PHASE | attendu plan §6.3 |
| `docs/security/CONTRIBUTOR_ATTESTATION_PREDICATE.md` | PHASE | attendu plan §6.2 (P0-G1-2 ack) |
| `docs/security/CONTRIBUTOR_ATTESTATION_RFC.md` | PHASE | attendu plan §6.2 Couche 3 |

- [x] PHASE : 21 fichiers — tous attendus par le plan (18 iteration 1 + 3 fix P1 iteration 2).
- [x] CRAFT : 0 fichiers planning/docs Claude melanges.
- [x] DEBT : 0 fichiers scope-cut hors-phase.
- [x] NOISE : 0 (pas de .pdb, .env, node_modules, cache).

**Verdict Working tree audit** : PASS.

---

### G8 traceability

- [x] Artefact G8 present : `.planning/active/sprint22_phase_C_preflight.md` (commit `fb16a50`).
- [x] Verdict preflight : **EXECUTE plan-as-is** — scans S1/S2/S3/S4 tous clean.
- [x] Pas de DESIGN-CONFLICT, pas de SCOPE-CUT-CONSISTENT → aucun carry-over S+1 requis.
- [x] Pas de Cas D hotfix (G8 applicable et applique).

**Verdict G8 traceability** : PASS.

---

### Scope-cuts

- [x] **Redundancy voting (S22→S23+S24)** : 0 fichier diff touche `redundancy_factor`, `redundancy_voting`, `majority`.
- [x] **Sandbox tool-calling (post-S25)** : 0 fichier diff lie.
- [x] **Couche 3 DelegationCert implem (S23-S27)** : `attestations/mod.rs:24` confirme design-only. `DOMAIN_DELEGATION_CERT_V1` non-ajoute a `canonical.rs`.

**Verdict Scope-cuts** : PASS. 0 scope leak detecte.

---

### Tests-delta

Delta reel mesure (post-fix iteration 2, `cargo nextest run --workspace --locked` = 705 passes) :

| Suite | Baseline S22-B | Reel post-C | Delta |
|---|---|---|---|
| Rust (nextest) | 702 | 705 | +3 (tests http.rs proxy) |
| Rust total Phase C | 702 | 705 | +3 proxy fix |
| Rust Phase C core | 666 baseline Phase A → 699 attendu | 705 | +39 depuis Phase A |
| Python coord | 255+3 inchange | 255+3 | 0 (fix proxy = Rust only) |

**Comptage tests Rust du diff complet Phase C** (iteration 1 + iteration 2) :

| Fichier | Tests |
|---|---|
| `age_witness.rs` | 8 |
| `contributor.rs` | 10 |
| `gossip.rs` | 7 |
| `bootstrap_allowlist.rs` | 8 |
| `http.rs` (proxy, iteration 2) | 3 |
| **Total Rust Phase C** | **36** |

**Python coord** : +6 (`test_contributor_registry.py` nouveau fichier).

**Total delta Phase C** : **+36 Rust + 6 Python = +42**.

Le body commit annoncait "+49". Ecart de +7 — le commit body doit etre recalibre a "+42" (ou le chiffre exact verifie par `cargo nextest run --workspace --locked` qui retourne 705 depuis la baseline 666 Phase A = +39 Rust total depuis debut sprint, mais iteration entre phases A et C inclut aussi Phase B +7 Rust = +7+36 = correctement wrappes).

**P2 residuel** : le body commit doit afficher le delta correct. Non-bloquant mais a corriger avant staging.

**Verdict Tests-delta** : PASS avec P2 recalibration annonce.

---

### Research-grounding

- [x] **reqwest ajout Cargo.toml** : `reqwest = { workspace = true }` — reqwest est une dep workspace existante (utilisee par le worker depuis S13). Pas de bump de version, pas de nouvelle lib. Tracage research non-requis pour une dep workspace pre-existante.
- [x] **Aucune autre dep Rust ajoutee/bumpee** dans le diff.
- [x] **pyproject.toml** : non modifie. `sqlite3` est stdlib Python.
- [x] **in-toto v1.0 Statement** : trace dans preflight S1 ligne 26-27 + kickoff §4 D1 + PREDICATE.md §5.
- [x] **ed25519-dalek 2.1** : inchange en version, trace preflight S1 ligne 27.

**Verdict Research-grounding** : PASS. 0 dep non-tracee.

---

### Horizon long-terme + documentation amont

- [x] **Design doc present** : `docs/security/CONTRIBUTOR_ATTESTATION_PREDICATE.md` cree AVANT le code (livrable P0-G1-2 ack obligatoire plan §6.2.2).
- [x] **Alternatives rejetees citees** : kickoff §4 D1 Couche 1 lignes 343-354 + Couche 2 lignes 389-398 — toutes avec rationale factuel.
- [x] **Matthew-effect caveat LT-1 TODO** : commente dans `curator.rs:292-298`, `attestations/contributor.rs:28-36`, `contributor_registry.py:27-36`, `attestations/mod.rs:30-43`.
- [x] **Solution la plus poussee** : JCS + Ed25519 + in-toto v1.0 + SQLite WAL + hot-reload TOML watcher.
- [x] **Aucune estimation LOC prescriptive** : estimations plan §6 sont descriptives (orientations scope, non-prescriptives).

**Verdict Horizon long-terme** : PASS.

---

## Findings

### P2 — Delta tests commit body a recalibrer

**Severite** : P2 non-bloquant.

**Description** : Le commit body annonce "+49 tests" alors que le delta reel Phase C est +36 Rust + 6 Python = +42 (ou +39 Rust depuis Phase A base 666 selon facon de compter). Le chiffre exact doit etre factuellement correct avant staging.

**Action** : relancer `cargo nextest run --workspace --locked` + `pytest packages/nexus-coordinator/tests/ -q` et corriger la ligne delta dans le commit body.

---

### P3 — `DOMAIN_PROVENANCE_V1` et `DOMAIN_WARRANT_CANARY_V1` non re-exportes en `lib.rs`

**Severite** : P3 cosmetique, pre-existant hors-scope Phase C.

**Description** : `crates/nexus-core-rs/src/lib.rs:63-66` re-exporte les 2 nouveaux domaines Phase C mais pas `DOMAIN_PROVENANCE_V1` ni `DOMAIN_WARRANT_CANARY_V1` (anciens, definis dans `canonical.rs`). Potentiellement manquant via `nexus_core.DOMAIN_*` Python. A tracker carry S23 si impact confirme.

---

## Recommandation

**Commit autorise.** P1 leve. 0 P0, 0 P1 actif.

Avant staging du commit body : recalibrer le chiffre "+49" au delta reel (+42 ou valeur issue de `cargo nextest --workspace --locked`). P2 non-bloquant mais la convention SBFB exige un delta factuel.

P3 DOMAIN re-export : carry S23, aucune action requise avant commit Phase C.

---

## Resume dimensions

| Dimension | Verdict iteration 1 | Verdict iteration 2 |
|---|---|---|
| Security | PASS | PASS (proxy guard hex + token forward vérifiés) |
| Patterns | PASS | PASS |
| Working tree audit (G5) | PASS* (http.rs absent) | PASS (21 fichiers PHASE) |
| G8 traceability | PASS | PASS |
| Scope-cuts | PASS | PASS |
| Tests-delta | CONCERN (+49 body vs +39 reel) | PASS avec P2 recalibration |
| Research-grounding | PASS | PASS (reqwest dep workspace pre-existante) |
| Horizon long-terme | PASS | PASS |
| **P1 proxy http.rs** | **FAIL** | **LEVE — route presente, authed, guard hex, token forward, 502 wired, 3 tests** |
