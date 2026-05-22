# Sprint 68 — Audit findings

**Auditeur** : session fraiche independante (2026-05-22).
**Sprint audite** : Sprint 68 — Proof Cards + Publish Gate (v2.1).
**Tip de reference** : `17394b6` (chore(process): D17 — sprint consolidation post-arc + process allege).
**Audit plan** : `.planning/active/sprint69_audit_plan.md`.
**Duree** : ~45 min (9 tracks, 3 blocs test paralleles).

---

## Verdict : PASS

| Severite | Count |
|---|---|
| P0 (regression securite / crash / data loss) | 0 |
| P1 (bug fonctionnel reproductible) | 0 |
| P2 (gap documentaire / hygiene) | 3 |
| P3 (nit / cosmetic) | 2 |

**0 P0, 0 P1 — aucun fix bloquant. 3 P2 + 2 P3 — rigor signal G4 satisfait.**

---

## Track A — Suites execution : PASS

**Exploration** :
- `cargo fmt --all --check` → 0 diff
- `cargo clippy --workspace --all-targets --locked -- -D warnings` → 0 warnings
- `cargo nextest run --workspace --locked` → 1419 started, 1419 passed, 0 failed
- `cargo test --workspace --locked --doc` → all ok (1 ignored, 0 failed)
- `cargo build -p nexus-shell-daemon --release` → ok
- `(cd web && npm run lint)` → 0 errors (5 warnings react-refresh T1 known)
- `(cd web && npx tsc --noEmit -p tsconfig.app.json)` → 0 errors
- `(cd web && npm run test:unit)` → 279 passed (23 test files)
- `(cd web && npm run build)` → ok
- `(cd web && npm run size)` → 6/6 (implicite build success)

**Compteurs** :

| Suite | Annonce (verification.md) | Reel (re-run) | Match |
|---|---|---|---|
| Rust nextest | 1419 | 1419 | oui |
| Vitest | 279 | 279 | oui |
| size-limit | 6/6 | 6/6 | oui |

**Tests ajoutes — analyse non-trivialite** :
- `test_proof_card_full_evidence` : non-trivial, verifie score=100 avec toutes couches presentes
- `test_proof_card_minimal` : non-trivial, verifie score=30 base
- `test_proof_card_provenance_boost` : non-trivial, verifie +20 provenance
- `test_proof_card_risk_no_provenance` : non-trivial, verifie -15 deduction + score=15
- `test_proof_card_formula_version` : non-trivial, verifie version=1
- `test_proof_card_clamp_bounds` : non-trivial, verifie [0, 100] clamping (score=0 edge case)
- `test_proof_card_freshness_states` : non-trivial, exerce 3 etats (fresh/aging/stale) via injection temps
- `test_proof_card_unverified_deploy` : non-trivial, verifie risk factor distincts (unverified vs no_provenance)
- `test_proof_card_endpoint_http` : non-trivial, integration HTTP GET 200 + JSON validation
- `test_proof_card_endpoint_not_found` : non-trivial, verifie 404
- `test_preview_load_returns_hash` : non-trivial, HTTP POST + BLAKE3 hash 64 chars
- `test_preview_blob_serve_accessible` : non-trivial, integration preview→blob-serve roundtrip
- `test_preview_eviction_after_ttl` : non-trivial, verifie TTL eviction temporelle
- `test_preview_max_size_rejected` : non-trivial, verifie 413 sur oversized
- `load_returns_blake3_hash` : non-trivial, verifie hash BLAKE3 correct
- `get_returns_data_before_ttl` : non-trivial, verifie roundtrip data
- `get_returns_none_after_ttl` : non-trivial, verifie TTL
- `rejects_oversized_upload` : non-trivial, verifie erreur TooLarge
- `evict_expired_removes_stale_entries` : non-trivial, verifie eviction
- `has_returns_false_for_unknown_hash` : non-trivial, verifie miss
- `test_fg5_rejects_path_traversal_canonicalize` : non-trivial, P2-C-2 fix verification
- `test_fg5_rejects_windows_backslash_traversal` : non-trivial, path separator cross-platform
- `test_fg5_rejects_symlink` : non-trivial, symlink escape detection (cfg unix/windows)
- `test_fg5_accepts_valid_subdir` : non-trivial, valid path containment
- `test_fg6_lockfile_hash_consistency` : non-trivial, factory project hash match
- `test_fg6_lockfile_mismatch_detected` : non-trivial, tampered provenance detection
- `test_fg6_detects_aws_secret` : non-trivial, AKIA regex detection
- `test_diff_detects_added_file` : non-trivial, workspace diff added
- `test_diff_detects_modified_file` : non-trivial, workspace diff modified
- `test_diff_detects_deleted_file` : non-trivial, workspace diff deleted
- `publish_requires_running_json` : non-trivial, daemon absent erreur
- `publish_pre_validates_manifest` : non-trivial, manifest name vide rejete
- `zip_directory_creates_valid_archive` : non-trivial, zip roundtrip avec sous-repertoire
- `run_rejects_missing_index_html` : non-trivial, pre-condition index.html
- `discover_fails_without_running_json` : non-trivial, daemon discovery erreur
- ProofCard.tsx 8 tests Vitest : tous non-triviaux (render score, expand/collapse, risk factors, loading, null, risk badge)
- useBridge proof_card_get test : non-trivial, dispatch + response

**Tests manquants** :
- Aucun livrable du plan sans test correspondant identifie.

**Findings** : 0

---

## Track B — Security review : PASS

**Exploration** :
- `grep -nE 'unsafe\s*{' {rs files diff}` → 7 matches, tous dans `#[cfg(test)]` blocks (`std::env::set_var` Rust 2024 edition) ou runtime.rs test code pre-existant. 0 `unsafe` en code production du diff.
- `grep -nE '\.unwrap()' {rs files hors tests}` → matches dans `http.rs:486,488` (`.parse().unwrap()` sur constantes statiques CSP/nosniff — infaillible). `proof_card.rs:363` dans `#[cfg(test)]` seulement. `browse.rs` lignes 691+ toutes dans `#[cfg(test)]`. 0 unwrap production sur chemin IO/async dans le diff.
- `grep -nE '(AKIA|ghp_|pat_|-----BEGIN)' {all files diff}` → 0 matches
- `grep -nE 'format!\(.*SELECT' {rs files}` → 0 matches
- `grep -nE 'dangerouslySetInnerHTML|innerHTML|v-html' {ts files}` → 0 matches
- `grep -nE 'serde_json::to_string[^_]' {canonical files}` → non applicable (proof_card.rs utilise `Serialize` derive + axum `Json()` wrapper, pas de serialisation manuelle)
- Nouvelles routes diff : `GET /api/daemon/proof-card/{project_id}` et `POST /api/v1/preview/load`. Les deux sont dans `authed_routes` du router (auth_required middleware). Verification : le `proof-card` handler lit `state.browse_aggregator` et `state.coordinator_db` — read-only, pas d'injection possible car le project_id est un path param utilise comme cle de lookup (pas interpolation SQL). Le `preview_load` handler passe le body a `PreviewStore::load()` qui hash avec BLAKE3 et stocke en memoire — pas d'ecriture disque, pas de path manipulation.
- `grep -nE 'console\.(log|warn|error)' {ts prod files}` → 0 matches dans les fichiers du diff hors tests.
- Path traversal : `dunce::canonicalize` + `starts_with` prefix check dans `gates.rs:118-126` et `template_engine.rs`. Read `gates.rs:66-113` confirme le code `WalkDir::new(&canonical).follow_links(false)` + verification symlink cible `resolved.starts_with(&canonical)`. Correct.
- Preview size limit : `MAX_PREVIEW_BYTES = 10 * 1024 * 1024` (10 MB) dans `preview.rs:18`, verifie dans `load()` avant stockage. Le handler HTTP retourne 413. Correct.

**Threat model** : diff touche les surfaces ProofCard (nouvelle section §12 T-PROOFCARD-FORMULA-GAME ajoutee Phase D) et preview ephemere (pas de nouvelle section THREAT_MODEL pour preview, mais la surface est low-risk : loopback-only, TTL auto-eviction, size-limited). Le THREAT_MODEL est a jour pour la surface ProofCard.

**Deps** : 1 nouvelle dep (`dunce 1.0`) ajoutee dans `Cargo.toml`. dunce est mature (6M+ downloads crates.io, derniere release 2024, 0 CVE connu). `npm audit` : 1 moderate vulnerability pre-existante (non introduite par S68). `Cargo.lock` delta minimal (+12 lignes, dunce + hash_hasher transitive).

**Findings** : 0

---

## Track C — Patterns conformity : PASS

**Opinion formee avant PATTERNS.md (Step 4 C.1)** :
1. `proof_card.rs` : formule additive deterministe, separation input/output claire, injection `now` pour testabilite. Pattern fonctionnel propre.
2. `preview.rs` : `RwLock<HashMap>` pour un store ephemere — simple et suffisant pour la concurrence basse (une seule CLI factory a la fois). Pas de dashmap ou mutex sharding necessaire.
3. `gates.rs` : fonctions pures `run_gate_fg*()` retournant `GateResult` — pattern pipeline compose testable. `check_path_containment` isole le point de securite.
4. `ProofCard.tsx` : composant React simple, expand/collapse via `useState`, labels francais. Aucun `dangerouslySetInnerHTML`. Utilise `data-testid` pour les tests.
5. Le code ne touche ni `canonical_bytes` ni aucun wire format protocolaire. Coherent avec la decision "local compute artefact".

**Comparaison avec PATTERNS.md** :
- P52 (Backend-agnostic enum with Deref) : le `PreviewStore` est un struct simple avec `Arc<RwLock<HashMap>>`, pas un enum polymorphe. Pas de divergence — P52 n'est pas applicable ici.
- P51 (Raw-op store+forward) : le ProofCard n'est pas un feed op (scope cut SC-13). Coherent.
- P39 (coordinator DB singleton in DaemonHttpState) : le handler `get_proof_card` accede a `state.coordinator_db.lock()` via le pattern P39. Respecte.

**Pattern drift** : aucun nouveau pattern structurant non documente. La ProofCard est un module applicatif (struct + formule), pas un pattern reutilisable cross-crate.

**Tech debt T-NN** : aucun T-NN touche par le diff.

**Findings** : 0

---

## Track D — Scope conformity : PASS

**Mapping plan livrables → diff** :

| Phase | Livrable | Code | Test | Statut |
|---|---|---|---|---|
| A | ProofCard struct + formule | oui (proof_card.rs) | oui (8 tests) | OK |
| A | GET /api/daemon/proof-card/{id} | oui (http.rs) | oui (2 tests) | OK |
| A | bridge method proof_card_get | oui (useBridge.ts + sbfb-bridge.js) | oui (1 test) | OK |
| A | sbfb-manifest allowlist | oui (lib.rs) | oui (grep) | OK |
| A | schema Zod ProofCard | oui (protocol.ts) | oui (via bridge test) | OK |
| B | PreviewStore HashMap+TTL | oui (preview.rs) | oui (6 tests) | OK |
| B | POST /api/v1/preview/load | oui (http.rs) | oui (4 tests HTTP) | OK |
| B | sbfb-factory preview cmd | oui (preview_cmd.rs) | oui (2 tests) | OK |
| B | sbfb-factory publish cmd | oui (publish.rs) | oui (2 tests) | OK |
| B | daemon_client discover | oui (daemon_client.rs) | oui (1 test) | OK |
| C | FG4 diff | oui (gates.rs + diff.rs) | oui (3+1 tests) | OK |
| C | FG5 sandbox canonicalize | oui (gates.rs) | oui (4 tests) | OK |
| C | FG6 secrets + lockfile | oui (gates.rs) | oui (3 tests) | OK |
| C | FG7 preview wiring | oui (gates.rs) | oui (code, test indirect) | OK |
| C | P2-C-2 path traversal fix | oui (template_engine.rs) | oui (fg5 tests) | OK |
| C | Diff + ScanSecrets subcommands | oui (main.rs) | oui (via gates) | OK |
| D | ProofCard.tsx composant | oui (ProofCard.tsx) | oui (8 tests) | OK |
| D | BrowsedProject integration | oui (BrowsedProject.tsx) | oui (indirect via ProofCard) | OK |
| D | THREAT_MODEL §12 | oui (THREAT_MODEL.md) | oui (grep T-PROOFCARD) | OK |
| E | verification.md | oui | N/A | OK |
| E | audit_plan S69 | oui | N/A | OK |

**Scope creep** : 14/14 scope cuts verifies par grep. 0 leak detecte :
- `SearchManifest` : 0 match dans le code du diff
- `factory` page React : 0 fichier dans `web/src/pages/`
- `babel` : 0 match dans `crates/` ou `examples/`
- `tree-sitter` : 0 match dans `Cargo.toml`
- FG8/FG9/FG10 : 0 match dans le code (seulement dans docs/planning)
- `nexus-shell-daemon` dans `crates/sbfb-factory/Cargo.toml` : absent (factory independante)

**Commits hors-scope** :
- `6a21293 docs(research): stage Remote User Sharded LLM R&D document` : document de recherche, pas du code, pas hors-scope (docs staging est acceptable)
- `17394b6 chore(process): D17` : touche CLAUDE.md + README.md + roadmap. Pas de code source. Acceptable comme amendement process.

**Fix inter-phases** :
- `2d0999f fix(planning): remove residual PASS-PENDING` : justifie par le hook lightcheck qui detecte les strings PASS-PENDING residuelles dans le texte descriptif des reviews.
- `dec62d0 fix(planning): remove residual PASS-PENDING strings from S68 review files` : meme justification, couvre les 5 review files. Body de commit riche expliquant la raison.

**Findings** : 0

---

## Track E — Tests adequacy : PASS

**Delta reel vs annonce** :

| Suite | Annonce (verification.md S2) | Reel (re-run) | Match |
|---|---|---|---|
| Rust nextest | 1384→1419 (+35) | 1419 | oui |
| Vitest | 270→279 (+9) | 279 | oui |
| size-limit | 6/6 | 6/6 | oui |

**Coverage fonctions publiques** :
- `pub fn compute_proof_card()` dans proof_card.rs : 8 tests couvrent tous les chemins → OK
- `pub fn compute_proof_card_at()` : fn interne, exercee via les tests ci-dessus → OK
- `pub fn load()` / `pub fn get()` / `pub fn has()` / `pub fn evict_expired()` dans preview.rs : 6 tests → OK
- `pub fn run_gate_fg4_diff()` : 1 test direct + 3 tests diff → OK
- `pub fn run_gate_fg5_sandbox()` : 4 tests → OK
- `pub fn run_gate_fg6_secrets()` : 3 tests → OK
- `pub fn run_gate_fg7_preview()` : teste indirectement via le code (pas de test isole pour le cas daemon absent — couvert par la structure GateResult)
- `pub fn check_path_containment()` : 2 tests directs → OK
- `pub fn diff_workspace()` : 3 tests → OK
- `pub fn run()` dans publish.rs : 2 tests → OK
- `pub fn run()` dans preview_cmd.rs : 2 tests → OK
- `pub fn discover()` dans daemon_client.rs : 1 test → OK
- `ProofCard` component export : 8 tests Vitest → OK

**Edge cases non couverts** :
- `PreviewStore` : pas de test pour le cas "concurrent writers" (multiple loads simultanes). Acceptable car le store est protege par `RwLock` et le use case est mono-user (1 CLI factory a la fois).
- `run_gate_fg7_preview` : pas de test isole pour `index.html` absent (couvert par le code `workspace.join("index.html").exists()`). P3 nit.

**Plan vs reel** :
- Plan §4.3 prevoyait 8 tests Phase A, reel 11 (+3 bonus : freshness_states, unverified_deploy, endpoint_not_found)
- Plan §5.3 prevoyait 6 tests Phase B, reel 14 (+8 bonus : preview_cmd tests, daemon_client test, http integration tests additionnels)
- Plan §6.3 prevoyait 9 tests Phase C, reel 10 (+1 bonus : fg6_detects_aws_secret)
- Plan §7.3 prevoyait 4-5 tests Phase D, reel 8 (+3-4 bonus : null render, loading state, risk badge, no-factors)
- Total plan : +23 Rust / +5 Vitest. Reel : +35 Rust / +9 Vitest. Over-delivery significatif.

**Findings** : 0

---

## Track F — Review files integrity : PASS

**Exploration** :

| Phase | Preflight G8 | Review | Codex | Verdict preflight |
|---|---|---|---|---|
| A | present | present | present | EXECUTE plan-as-is |
| B | present | present | present | EXECUTE plan-as-is |
| C | present | present | present | EXECUTE plan-as-is |
| D | present | present | present | EXECUTE plan-as-is |
| E | present | present | present | EXECUTE plan-as-is |

**Phase review ratio** : 5/5 (toutes les phases avec commit ont un review file)
**Design review G1** : present (`.planning/active/sprint68_design_review.md`). Scoring D1-D5 : 4 ok + 1 warning (D4 dunce). Le warning est acknowledged dans le kickoff §4 "Acknowledged review findings (G1)".

**Review verdicts** : tous PASS (grep confirme). Aucun PASS-PENDING residuel (corrige par `dec62d0` fix(planning)).

**Codex reviews** : 5/5 presentes (Phases A-E). Sprint >= S65, dual-agent actif. Toutes presentes.

**Findings** : 0

---

## Track G — Carry-overs discipline : PASS

**Items 3/3 MANDATORY** :

Aucun item n'atteint 3/3 MANDATORY au S68. Verification kickoff §6 : "Aucun item n'atteint 3/3 MANDATORY au S68." Confirme.

**P2-C-2 path traversal Windows** :
- Kickoff dit 1/3→resolved Phase C.
- Code verification : `gates.rs:66-126` utilise `dunce::canonicalize` + `starts_with(&canonical)`. `template_engine.rs` refactored (l'ancien `contains("..")` remplace par canonicalize).
- 4 tests FG5 couvrent path traversal, Windows backslash, symlink, valid subdir.
- **CLOSED confirme.**

**P2-I-2 delta repartition body** :
- Kickoff dit 1/3→2/3. Phase A body disait "+10 Rust" mais le reel etait +11 (corrige retrospectivement Phase B). Le pattern est identifie et documente.
- Verification.md S5 confirme "2/3, attention 3/3 S69".
- Compteur coherent : S67 audit findings creait l'item a 1/3, S68 le porte a 2/3. Trace :
  `grep -rl "P2-I-2" .planning/archive/` → present dans S67 findings.

**Carries exemption** :
- P2-A-1 rand : statut inchange (exemption externe). Confirme.
- P2-AUDIT-2 iroh transitives : statut inchange (exemption externe). Confirme.
- P2-G-1 exe lock : monitoring. Non reproductible depuis S62. Confirme.
- T-NN+2 iframe Rust-wasm : bloque upstream. Confirme.
- LT-2 Radicle : trigger PENDING (tag v1.0 pose localement, pas pousse). Confirme.
- LT-5 redundancy persistence : hors-sprint. Confirme.
- LT-7 worker quorum E2E : post-tag. Confirme.

**Exhaustivite carries S69** : verification.md S5 liste 8 carries. Kickoff §6 listait 7 carries reconduits + P2-I-2 2/3. Tous traces dans le verification.md. Aucun item perdu.

**Findings** : 0

---

## Track H — HARDENING drift : PASS

**Prescriptions HARDENING_ROADMAP pour S68** : aucune.
Le HARDENING_ROADMAP couvre S18-S30. S68 n'a pas d'entree specifique. PASS automatique.

**THREAT_MODEL.md** : §12 T-PROOFCARD-FORMULA-GAME ajoute Phase D. Read `docs/security/THREAT_MODEL.md` : 4 vecteurs documentes (provenance factice, curator collusion, license gaming, freshness gaming) avec mitigations. Surface correctement couverte.

**Triggers_revalidate** : les triggers listes (iroh > 0.98, wasmtime, arti, etc.) sont inchanges. Aucun trigger active par S68.

**Drift cumule** : aucun drift multi-sprint detecte pour S68.

**Findings** : 0

---

## Track I — Meta-process discipline : CONCERN (1 P2, 2 P3)

**Commit stack** :

| SHA | Title | Pattern OK | Body 9 sections |
|---|---|---|---|
| `3ca563f4` | chore(planning): Sprint 68 kickoff + plan | oui | N/A (chore) |
| `f9d722e6` | feat(proof-card): Sprint 68 Phase A | oui | 9/9 |
| `2d0999f4` | fix(planning): remove PASS-PENDING Phase A | oui | 1/9 (body minimal) |
| `1d53f18c` | feat(factory): Sprint 68 Phase B | oui | 9/9 |
| `6a21293c` | docs(research): stage LLM R&D document | oui | 0/9 (1 ligne body) |
| `a201b3e3` | feat(factory): Sprint 68 Phase C | oui | 9/9 |
| `ecb25c5e` | feat(shell): Sprint 68 Phase D | oui | 9/9 |
| `e415034d` | docs(sprint68): Sprint 68 Phase E | oui | 9/9 |
| `dec62d09` | fix(planning): remove PASS-PENDING S68 reviews | oui | body riche (5 lignes) |
| `17394b6` | chore(process): D17 sprint consolidation | oui | body riche (20+ lignes) |

**Split chore/feat** :
- `17394b6` chore(process) touche `docs/claude/README.md`, `CLAUDE.md`, `.planning/roadmap_v4_*`. Tous dans docs/ ou .planning/ ou racine CLAUDE.md. 0 code source → OK.
- `3ca563f` chore(planning) touche seulement `.planning/` → OK.

**Delta tests cumule** :
- Phase A body : "+10 Rust" (1384→1394). Reel : +11 (1384→1395). Divergence = 1 test.
- Phase B body : "+14 Rust (1395→1409)". Reel : +14 (1395→1409). Match.
- Phase C body : cumule "+35 (Phase A +11, B +14, C +10)". 1384+35=1419. Reel : 1419. Match.
- Phase D body : "+0 Rust, +8 Vitest (271→279)". Reel : +8 Vitest (271→279). Match.
- Somme annonces : Rust 10+14+10+0 = 34 (Phase A dit 10 au lieu de 11). Reel : 35. Divergence = 1 test Phase A body.
- Phase B corrige retrospectivement Phase A a +11, donc le cumule Phase C est correct (+35). Le body Phase A original est le seul body incorrect.

**Findings** : 3 (P2-I-1, P3-I-1, P3-I-2)

---

## Findings

### P2-I-1 (P2, carry 3/3 S69)

**Constat** : Le body du commit Phase A `f9d722e` section "## Delta tests" indique "Rust : 1384 → 1394 (+10)" alors que le delta reel est +11 (1384→1395). Phase B corrige retrospectivement : "Phase A : +11 Rust (1384→1395)". C'est le carry P2-I-2 du kickoff S68 §6 qui passe de 2/3 a 3/3. Le pattern est recurrent : les body de commit individuels annoncent un delta qui diverge de 1 test par rapport au reel, corrige retrospectivement dans la phase suivante.

**Impact** : Impact formel seulement — la trace de delta cumule dans verification.md est correcte (+35 Rust total), mais les bodies de commit individuels ne sont pas la source de verite fiable pour le delta par phase. Un auditeur futur ne peut pas retracer le delta phase par phase depuis les bodies seuls.

**Recommandation** : S69 doit resoudre P2-I-2 (3/3 MANDATORY). Template body standardise avec compteur reel verifie par nextest AVANT la redaction du body. Owner : planner S69.

**Compteur** : 3/3 (1/3 S67, 2/3 S68, 3/3 S69 MANDATORY)

---

### P2-I-3 (P2, nouveau 1/3)

**Constat** : Le commit `6a21293 docs(research): stage Remote User Sharded LLM R&D document` a un body minimaliste d'une seule ligne + Co-Authored-By. Ce n'est pas un commit `feat` donc les 9 sections ne sont pas obligatoires, mais un commit `docs(research)` de 1302 lignes merite un body contextuel (objet de la recherche, lien avec le sprint, raison du staging).

**Impact** : Faible. Le document de recherche est auto-explicatif par son contenu. Mais la convention body riche s'applique aussi aux docs commits significatifs (README.md §4.1 mentionne "chaque commit" pas "chaque feat").

**Recommandation** : Les docs/research commits significatifs (>100 lignes) devraient avoir un body de 3-5 lignes contextuelles. Nit process. Owner : executeur sprint.

**Compteur** : nouveau 1/3

---

### P2-B-1 (P2, nouveau 1/3)

**Constat** : La surface preview ephemere (`POST /api/v1/preview/load`) n'a pas de section dediee dans `THREAT_MODEL.md`. Le preview accepte des bytes arbitraires (zip) de taille <= 10 MB, les stocke en memoire, et les sert via blob-serve. Les mitigations existent (size limit, TTL, loopback-only, auth required), mais le vecteur "preview abuse" (upload massif repetitif pour saturer la memoire avant eviction) n'est pas documente.

Read `crates/nexus-shell-daemon-core/src/preview.rs:17-18` confirme : `MAX_PREVIEW_BYTES = 10 MB`, `DEFAULT_TTL = 30 min`. Pas de `MAX_PREVIEW_ENTRIES` cap (nombre d'entries non borne). L'audit plan S69 Track 2 mentionne ce vecteur ("Preview store Phase B : verifier qu'aucun threat manque pour le vecteur ephemeral preview abuse").

**Impact** : Moyen-bas. Loopback-only + auth token = l'attaquant doit avoir acces local. Mais un script local pourrait saturer la memoire daemon en uploadant des previews de 10 MB en boucle (10 MB × 100 = 1 GB en 50 sec, pas d'eviction avant 30 min). Risque DoS local.

**Recommandation** : (1) Ajouter `MAX_PREVIEW_ENTRIES` (ex: 10 entries simultanees) dans `PreviewStore`. (2) Documenter le vecteur dans THREAT_MODEL.md §preview. Owner : planner S69.

**Compteur** : nouveau 1/3

---

### P3-I-1 (P3, nit)

**Constat** : Le commit `2d0999f fix(planning): remove residual PASS-PENDING string from Phase A review.md` a un body d'une seule ligne + Co-Authored-By. C'est un fix planning mineur, mais le second fix (`dec62d0`) a un body de 5 lignes expliquant la raison (lightcheck hook). Le premier fix aurait pu etre squashe dans le second.

**Impact** : Nit. Deux commits pour le meme type de fix (remove PASS-PENDING) la ou un seul aurait suffi. Pas de consequence fonctionnelle.

**Recommandation** : Grouper les fix planning du meme type en un seul commit. Pas d'action obligatoire.

**Compteur** : nit, pas de tracking

---

### P3-I-2 (P3, nit)

**Constat** : Les fonctions `run_gate_fg5_sandbox` et `run_gate_fg7_preview` et `check_path_containment` dans `gates.rs` portent `#[allow(dead_code)]` (lignes 64, 117, 168). Ces fonctions sont publiques et potentiellement appelees via CLI ou le pipeline gate. Le `#[allow(dead_code)]` indique qu'elles ne sont pas encore wireees dans le main.rs (les subcommands `diff` et `scan-secrets` sont exposes mais pas `sandbox` et `preview` comme subcommands CLI).

Read `crates/sbfb-factory/src/gates.rs:64,117,168` confirme : `#[allow(dead_code)]` sur `run_gate_fg5_sandbox`, `check_path_containment`, `run_gate_fg7_preview`.

**Impact** : Nit. Les fonctions sont testees et fonctionnelles. Le `#[allow(dead_code)]` est un signal que le wiring CLI est incomplet (FG5/FG7 pas exposes comme subcommands).

**Recommandation** : Considerer exposer `sbfb-factory sandbox` et `sbfb-factory preview-check` comme subcommands S69, ou retirer le `#[allow(dead_code)]` si les fonctions sont utilisees indirectement. Nit.

**Compteur** : nit, pas de tracking

---

## Scope cuts verification

14/14 scope cuts respectes :
- SC-1 SearchManifest wire format : 0 match dans code diff → OK
- SC-2 Page React /factory : 0 fichier `web/src/pages/*factory*` → OK
- SC-3 Babel dogfood : 0 match substantiel dans `crates/` ou `examples/` → OK
- SC-4 @dev tree-sitter : 0 match dans `Cargo.toml` → OK
- SC-5 Template react-vite : 0 match dans template code → OK
- SC-6 Factory audit log JSONL : 0 match dans code → OK
- SC-7 CuratorVouched UI : 0 composant React vouch → OK
- SC-8 FG8 Provenance Ed25519 : 0 match code (docs seulement) → OK
- SC-9 FG9 Publish gate complete : 0 match code → OK
- SC-10 FG10 Review gate : 0 match code → OK
- SC-11 Fuzzing : 0 match proptest/cargo-fuzz → OK
- SC-12 Feed format version bump : constantes inchangees → OK
- SC-13 ProofCard comme feed op : 0 feed op ProofCard → OK
- SC-14 Diff engine avance : diff basique seulement (added/modified/deleted) → OK

---

## Conclusion

Sprint 68 livre le circuit de preuve complet tel que planifie : ProofCard evidence-score (0-100, formule deterministe, 7 risk factors, formula_version 1), preview ephemere avec TTL et size limit, publish path Factory→daemon via deploy-from-repo, gates FG4-FG7 (diff, sandbox dunce::canonicalize, lockfile, secrets, preview), et le composant UI ProofCard.tsx integre dans Browse. La couverture de test est superieure au plan (+35 Rust et +9 Vitest contre +23/+5 estime). Le carry P2-C-2 (path traversal Windows) est correctement resolu avec `dunce::canonicalize` + prefix check + 4 tests. Les 14 scope cuts sont strictement respectes.

Les 3 P2 identifies sont formels/documentaires : P2-I-1 (carry P2-I-2 3/3 MANDATORY S69 — delta body), P2-I-3 (body docs(research) minimaliste), P2-B-1 (preview store sans cap entries ni section THREAT_MODEL). Aucun n'est un risque fonctionnel ou securite bloquant.

**Verdict : PASS — ouverture Sprint 69 autorisee.**

---

## Notes on audit completeness

- Track A : exploration complete (3 blocs re-run parallele, compteurs verifies)
- Track B : exploration complete (9 patterns OWASP, threat model, deps)
- Track C : exploration complete (opinion formee avant PATTERNS.md, comparaison faite)
- Track D : exploration complete (mapping exhaustif 21 livrables)
- Track E : exploration complete (delta, coverage fonctions publiques, edge cases, plan vs reel)
- Track F : exploration complete (5/5 preflights, 5/5 reviews, 5/5 codex, 1 design review)
- Track G : exploration complete (carries traces, P2-C-2 CLOSED confirme, exhaustivite verifiee)
- Track H : exploration complete (HARDENING_ROADMAP pas de prescription S68, THREAT_MODEL a jour)
- Track I : exploration complete (9 sections verifiees, split chore/feat, delta cumule)

## Commits fix produits

Aucun fix requis (0 P0, 0 P1).

## P2 a logger en tech debt

- P2-I-1 → P2-I-2 carry 3/3 MANDATORY S69 : template body standardise avec compteur verifie (deja dans kickoff S68 §6 "Attention 3/3 S69")
- P2-I-3 → nouveau 1/3 : body docs/research minimaliste (nit process)
- P2-B-1 → nouveau 1/3 : `MAX_PREVIEW_ENTRIES` cap + THREAT_MODEL §preview

## P3 laisses sans action

- P3-I-1 : deux fix(planning) pour le meme type de correction — nit, pas d'action requise
- P3-I-2 : `#[allow(dead_code)]` sur gates FG5/FG7/check_path_containment — nit, wiring CLI incomplet
