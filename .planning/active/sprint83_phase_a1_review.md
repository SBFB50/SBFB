# Sprint 83 — Phase A1 — Review

## Verdict: PASS

Review Workflow ultracode = fan-out avant-plan synchrone de 3 agents opus-4-8[1m]
(dimensions + vérification adversariale), README §4.5. Promu PASS après
reconciliation Codex (GAPS: 0 ; PARTIEL documentaire carry 7 adressé, cf.
§Codex reconciliation).

Lot de 9 carries CI/hygiène hors-thème (condition 2 audit S82). Diff = 32 fichiers.

## Suites §7.4 (dual-platform)

- **Windows** : `cargo fmt --all --check` VERT ; `cargo clippy --workspace
  --all-targets --locked -D warnings` VERT ; `cargo nextest run --workspace
  --locked` **2121/2121 PASS 0-skip** (2110 → 2121, +11) ; `cargo test
  --workspace --locked --doc` VERT ; `cargo build -p nexus-shell-daemon
  --release` VERT.
- **Docker canonique Linux (`sbfb-ci`)** : `cargo fmt --all --check` VERT ;
  `cargo clippy` VERT ; `cargo nextest run --workspace --locked` **2125/2125
  PASS 0-skip** (+4 `#[cfg(unix)]`).
- `cargo deny check` : advisories + bans + licenses + sources **OK**.
- `bash .claude/hooks/tests/test_lightcheck_g1_case_insensitive.sh` : **3/3 PASS**.
- 10 watcher tests re-run Windows : **10/10 PASS** (no-regression dunce).
- Node local v22.22.0 (engines >=22 satisfait ; advisory, pas d'engine-strict).

## Delta tests

Rust nextest Win 2110 → 2121 (+11) : +8 `golden_http_*` familles, +1
`golden_families_stable`, +1 `dial_set_sampling_distribution`, +1
`test_engine_confined_root`. Docker 2121 → 2125 (miroir +4 unix). Shell : +1
test hook. Vitest web/operator inchangés (0 code source web/operator touché).

## Findings (3 agents adversariaux, tous VERDICT PASS)

**Agent 1 (prod-sensible : SEEDER + TEST-ISOLATION)** — PASS. Vérifié : contrat
best-effort SEEDER tient (`sort_by_key` reorder in-place, aucun id droppé,
consommateur `directory_pull_providers` = head-take du vecteur réordonné) ;
déterminisme cross-plateforme (blake3 lave l'ordre HashMap non-déterministe, 0
collision) ; séparateurs 0x1f corrects (context-scoping) ; test
`dial_set_sampling_distribution` sémantiquement valide + NON flaky (blake3
déterministe sur inputs fixes) ; TEST-ISOLATION field wiring correct
(destructure→assign, pas de None hard-codé) ; chaque site RUN garde son TempDir
guard vivant (`_state_tmp`/`_home` nommés) ; boot-only=None ne flushe jamais
(Engine::new ne flushe pas) ; `test_engine_confined_root` drive `flush_state_snapshot`
+ assert dest sous tempdir ET != worker_state_file() ; main.rs=None correct prod.

**Agent 2 (macos watcher + goldens)** — PASS. Vérifié : `dunce::canonicalize`
PARTOUT (grep : aucun `std::fs::canonicalize` dans les fixtures ; le seul
std::fs::canonicalize repo = store_migration workspace_root, hors watcher) ;
jamais sur chemin inexistant ; keystore_integration base-cohérent (with_params +
blob_path + watch tous sur `base`) ; test-only 0 dep runtime (dunce dev-only 4
crates) ; complétude prouvée (les 10 positive-reload tests couverts ; les
watcher tests non-touchés sont boot-only/négatifs/non-reload, insensibles au
drop d'event) ; 8 contrats goldens matchent les handlers ligne-par-ligne sur
nœud-vide ; **preview_load hash `6e46dd10...` vérifié par `b3sum` du littéral
`{}`** (node-indépendant, prouvé par golden_families_stable 2 nœuds frais) ;
took_ms seul champ volatil nouveau ; pas de test de découverte résiduel.

**Agent 3 (CI + hook + docs + scope)** — PASS. Vérifié : suppression pip-audit
structurellement propre (cargo-deny + npm-audit intacts, YAML valide) ; allowlist
Unlicense/MIT-0 justifiée (plus permissifs que MIT/CC0 déjà autorisés, pas de
régression sécurité) ; multiple-versions="warn" non touché ; bumps Node 22
cohérents + engines advisory + JSON valide ; hook :256 SEULE ligne changée
(autres sites $PHASE lowercase intacts) ; test hook isolé + grep message BLOCK
spécifique ; A1 ne s'auto-bloque pas (PHASE_UPPER="A1") ; §P68 "MIROIR pas
single-source" exact ; http.rs cite bien 21 modules (`ls | wc -l`=21) ; THREAT
§15 distinction ancre/dial-tail correcte ; pas de STALE-PHASE ; scope chantier
INTACT (sbfb-workflow-core/gate_matrix/t2_spec/docs-factory 0 modif) ; 0 wire
bump (aucun *_VERSION/DOMAIN/FEED_FORMAT).

## P3 acceptés (non-bloquants, documentés)

1. **`seed_registry.rs` `sort_by_key`** recompute blake3 O(n log n) fois ;
   `sort_by_cached_key` serait idiomatique. Négligeable (cap TTL + downstream
   PULL_PROVIDER_CAP) ET **volontairement conservé pour cohérence avec le MIROIR
   `placement.rs::sampling_key`** qui utilise aussi `sort_by_key`.
2. **`set_state_flush_path` (runtime.rs)** = 0 caller (superseded par le champ
   boot) — **pré-existant** (0 caller avant aussi), `pub` donc pas de warning
   dead_code ; conservé pour compat, référencé honnêtement dans la doc du champ.
3. **Commentaire golden oracle "clock"** — corrigé inline (précision : l'oracle
   attrape les valeurs qui DIFFÈRENT entre 2 nœuds ; une horloge grossière est
   attrapée par l'allowlist explicite `took_ms`, pas par l'oracle seul).

## G8 traceability

Preflight PLAN-ADAPT respecté : carry 7 non exécuté tel que proposé (cible
réfutée par les faits du preflight) ; 8 autres EXECUTE. Refutations preflight
honorées (dunce jamais std::fs ; EngineBoot per-Engine jamais env global ;
deny.toml manifestes MONO confirmés). 0 décision D1..D5/Day-0 rouverte.

Carry 7 (flake nextest) — résolution honnête : discipline no-band-aid respectée
(`.config/nextest.toml` INCHANGÉ ; la cible proposée était factuellement fausse,
cf. preflight). L'identification des flakes RÉELS nécessite des logs CI (evidence
gap au HEAD, non produisible localement). C'est une **carry STANDING** documentée
ici + dans le preflight (verdict PLAN-ADAPT) ; elle rejoint le ledger de carries
du wrap-up S83 (verification.md §5) et sera portée au prochain `audit_plan` à
l'ouverture de S84 — je NE crée PAS `sprint84_audit_plan.md` prématurément (il
est produit au kickoff S84, comme pour toutes les carries standing).

## Codex reconciliation

Codex GPT-5.6 Sol (effort max, read-only) — `sprint83_phase_a1_codex_review.md`
brut. **Statut : GAPS: 0, PARTIELS: 1.** 0 bug de correction/sécurité, 0
violation de scope, 0 wire bump (tout vérifié indépendamment : hash preview
recalculé blake3, 21 modules `*_api.rs` comptés, hex-validation des champs
prouvant que le séparateur 0x1f ne peut apparaître dans les entrées légitimes,
diff hook 1-add/1-del, `.config/nextest.toml` inchangé, YAML/JSON parsés).

**PARTIEL (carry 7) ADRESSÉ** : Codex a relevé que le review/preflight annonçait
un « re-route vers `sprint84_audit_plan.md` » alors que ce fichier n'existe pas
encore — sur-affirmation documentaire (pas un défaut runtime). Corrigé : la
formulation ci-dessus (§G8) et le commit body ne référencent plus un fichier
inexistant ; carry 7 est une carry STANDING documentée dans le preflight + ce
commit, portée au ledger du wrap-up S83. Aucun code touché ⟹ pas de re-run des
suites nécessaire (Codex a validé les 9 carries code CLEAN). Boucle Codex arrêtée
au critère « CLEAN ou P2/P3 documentés » (PARTIEL documentaire résolu).
