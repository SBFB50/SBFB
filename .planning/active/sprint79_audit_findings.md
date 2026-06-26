<!-- Audit gate S79 = Phase 0 de S80. Joué en Workflow ultracode (run wf_f1cfb078-709, 13 agents Opus 4.8 1M : 9 tracks structurels + 3 lentilles adversariales + synthèse). Le P1 load-bearing a été re-vérifié indépendamment par le main thread (voir §Réconciliation main-thread). -->

# Sprint 79 — Audit Gate Findings

## Périmètre

- **Diff audité** : `9297f08~1..f4b4600` (20 commits, tip `f4b4600`).
- **Objet** : capacité Factory « app-authoring » (anime.js 4.5.0 + daisyUI 5.5.23) — knowledge packs versionnés blake3, prompt-kind `app-authoring`, gate CSP déterministe Rust non-délégable `run_gate_csp_authoring` (source CSP unique `BLOB_SERVE_CSP` → `csp.rs:33`), self-check runtime, couche docs-contrat (`docs/factory/` Diataxis FR + `llms.txt` + `WIRING_SPEC.md` + exemple `include!` runnable + `check-factory-docs.sh`).
- **Structure** : 8 phases feature A-H + Phase I (GUIDE closure + wrap-up).
- **Méthode** : 9 tracks structurels + 3 lentilles adversariales, chaque claim re-vérifiée contre le code/fichiers réels (jamais la prose du wrap-up). Findings ré-ancrés `fichier:ligne` / git ref par le synthétiseur sur les gaps load-bearing.

## Verdict : CONDITIONAL PASS

0 P0 · **1 P1 nouveau** (traçable, correctif ciblé) · 8 P2 · 11 P3.

Le sprint livre du solide et LIVE-testé là où ça compte (gate CSP déterministe, packs blake3 hermétiques, doc-lints BLOQUANTS prouvés non-vacuous par mutation, source CSP unique). Le CONDITIONAL tient à **un gap de non-délégabilité du gate CSP** à l'échelle des verbes de publish (claim Day-0 #1 « scellage 100% Factory » matériellement fausse pour la boucle fork→edit→redeploy), gap réel mais **atténué** par la CSP runtime inchangée qui reste la vraie frontière d'isolation. Les deux carries P1 connus (sharding in-vivo, app-authoring in-vivo) sont escaladés honnêtement, PAS des régressions.

## Table des 9 tracks

| Track | Domaine | Statut | Findings nets (post-dédup) |
|---|---|---|---|
| T1 | Suites / delta tests | CLEAN | +3 `factory_csp_contract` CONFIRMÉ (3/3) ; 1 P2 (test ancre le contrat, pas le gate) + 1 P3 (baseline absolu non rejoué) |
| T2 | Sécurité CSP | CLEAN→**ÉLARGI** | 5 invariants tiennent DANS la pipeline `publish` ; **mais** non-délégabilité fausse hors `publish` (→ P1 consolidé) + 2 P2 (Vendored rename, scanner évadable) |
| T3 | Patterns §P70/§P71 | CLEAN | Patterns fidèles au code ; 1 P2 (footgun 16-hex) + 2 P3 (prose stale, drift 22/25) |
| T4 | Scope / Day-0 | CLEAN | 0 bump wire, 0 primitive Phase I, scellage additif ; carry P1 in-vivo + 2 P3 (élargissement docs-contrat, drift libellé A→G vs A-I) |
| T5 | Non-vacuité gates | CLEAN | Mutation-testé : `check-factory-docs.sh` + exemple `include!` DÉTECTENT le drift ; 1 P2 (substring) + 2 P3 |
| T6 | Review files | CLEAN | 9 triplets présents, Codex bruts ; 2 P2 (P1 Codex B shippé+mal-routé) + 1 P3 (frontier_volet4 non routé) |
| T7 | Carries | CLEAN | 6 carries reportés sans sur-vente ; 2 P3 (omissions §5) |
| T8 | Hardening | CLEAN | 0 dispense CSP/COEP/COOP, byte-identique ; 1 P2 (faux-négatifs runtime) + 1 P3 (HARDENING_ROADMAP) |
| T9 | Méta-process / G8 | CLEAN | T1/T2 testabilité livrés ; 1 P2 (run 6 PLAN-ADAPT) + 1 P3 (artefact JSON gitignored) |

## Findings consolidés

### P1 (1 — nouveau, traçable, à router en Phase 0 S80)

**P1-1 — Le gate CSP `run_gate_csp_authoring` n'est PAS non-délégable à l'échelle des verbes de publish (claim Day-0 #1 « scellage 100% Factory » matériellement fausse pour la boucle d'authoring).**
La CLI `sbfb-factory` expose plusieurs chemins qui publient des octets d'app, mais le gate n'est câblé que sur `publish`.
- Évidence verbe `redeploy` : `crates/sbfb-factory/src/main.rs:256` `Command::Redeploy → atelier::redeploy` ; `crates/sbfb-factory/src/atelier.rs:70` `pub fn redeploy(...)` zippe le workspace local édité et POST `{base}/api/v1/deploy-workspace` (atelier.rs:114) — `grep 'run_gate|csp' atelier.rs` = vide. C'est précisément la boucle fork→edit→iterate, cœur du « app-authoring », et elle saute le gate intégralement.
- Évidence route daemon : `crates/nexus-shell-daemon/src/deploy.rs:233` `deploy_workspace` et `deploy.rs:65+` `deploy_from_repo` — `grep 'csp|run_gate|authoring|sandbox' deploy.rs` = 0 hit ; routes `http.rs:421` (`/api/v1/deploy-workspace`) et `http.rs:414` (`/api/v1/deploy-from-repo`), atteignables en HTTP loopback direct (auth bearer) hors Factory.
- Le seul chemin gaté : `main.rs:277` `Command::Publish → publish::run → run_publish_pipeline` ; `pipeline.rs:48-52` place `run_gate_csp_authoring` délibérément HORS du bloc `skip_gates` (test `pipeline.rs:195` prouve qu'il bloque même `--skip-gates`) — mais `pipeline.rs:62` `post_deploy_from_repo` POST ensuite le `repo_url` au daemon qui clone HEAD sans rejouer le gate (le gate scanne le workspace LOCAL via `WalkDir`, gates.rs:386).
- **Non routé** : `grep 'redeploy|deploy-workspace|CSP gate|task_response' .planning/active/sprint80_audit_plan.md` = 0 hit ; le §3 ne porte que les carries familles-wire + in-vivo.
- **Atténuation (pourquoi P1 et non P0)** : la CSP runtime injectée par blob-serve (`csp.rs:33`, inchangée) reste inconditionnelle — l'app redéployée tourne quand même sous `connect-src 'none'` / `sandbox allow-scripts` sans `allow-same-origin`. Aucune évasion réseau effective ; l'impact fonctionnel de sécurité est ~P2, mais la claim flagship du sprint est fausse et l'écart Day-0 #1 est non escaladé.
- **Reco** : router en Phase 0 S80 ; trancher fix (câbler `run_gate_csp_authoring` sur `redeploy` + côté daemon `deploy_workspace`/`deploy_from_repo`) vs amendement Day-0 assumé documenté (« runtime-CSP-only est la frontière ; le gate Factory est un lint d'auteur best-effort »).

### P2 (8 — dette documentée)

**P2-1 — Commentaires-promesses non-scrubbés dans un fichier wire-format + gate anti-promise aveugle + P1 Codex Phase B sur-affirmé « routé ».**
`git show f4b4600:crates/nexus-core-rs/src/schemas/task_response.rs` : lignes `:14` (« S22+ sandbox activates »), `:93` (« until Sprint 22 activates »), `:95` (« when S22 lands ») toujours présentes au tip. `PROMISE_RE` (`scripts/check-frontier-contracts.sh:64-66`) ne les matche pas (0/3 ; seul un « When Sprint N » synthétique matche) → le gate passe vert sur 3 instances live de l'anti-pattern qu'il prétend interdire. `sprint79_phase_b_review.md:12-14` affirme « GAP DISCLOSÉ + ROUTÉ (carry task_response) via META-1 » mais `grep 'task_response|until Sprint|activates' sprint80_audit_plan.md` = 0 hit — routage inexistant. Reco : scrubber les 3 commentaires OU élargir `PROMISE_RE` à la classe `until/when Sprint N`, et router réellement en S80.

**P2-2 — Classification `Vendored` par nom de fichier relâche 2 contrôles sur simple renommage.**
`crates/sbfb-factory/src/gates.rs:353-362` : tout fichier `.min.js`/`.umd.js` (ou sous `vendor/`) → `CspTier::Vendored` ; `gates.rs:447-463` n'applique `MODULE_SCRIPT_PATTERN` et `ABSOLUTE_URL_PATTERN` qu'au tier `Scanned`. Un auteur peut renommer `app.js → app.min.js` pour échapper aux contrôles module-script + URL-absolue. Atténuation : CSP runtime tient. Reco : classifier par contenu/heuristique, pas par suffixe seul.

**P2-3 — Le scanner CSP statique est intrinsèquement évadable (accès réseau assemblé au runtime).**
`crates/sbfb-factory/src/gates.rs:189-298` : détection regex (`\bfetch\s*\(`, etc.). Un appel construit dynamiquement (`globalThis['fet'+'ch']`, `fetch` via `atob`, `action`/`href`/`url()` construits en JS) n'est pas vu. Honnêtement disclosé `THREAT_MODEL.md §13.1` (« scanner regex aveugle au code/URL assemblé au runtime »). Le gate est un filet additif ; la garantie d'isolation repose entièrement sur la CSP navigateur inchangée.

**P2-4 — `source-ref` symbol check = présence substring, pas ancrage de définition.**
`scripts/check-factory-docs.sh:180` `grep -qF "$sym" "$path"` : la ref `path:Symbol` est validée par simple présence de la chaîne, pas par une définition. Corrobore la doctrine memory (« BLOB_SERVE_CSP testé par substring »). Limite réelle de profondeur, non de non-vacuité (mutation-testé OK pour drift/bornes).

**P2-5 — Convention fragile gate volet-4 : tout token 16-hex minuscule dans une fiche knowledge-backed traité comme digest de pack (footgun SHA git).**
`scripts/check-frontier-contracts.sh:180-182` + `docs/rust/PATTERNS.md:3900-3902` : « EVERY lowercase 16-hex token is treated as a pack digest (do not embed an unrelated 16-hex identifier, e.g. a git SHA prefix) ». Un préfixe de SHA git de 16 hex dans une fiche déclencherait un faux échec de gate.

**P2-6 — Le test runnable ancre le CONTRAT CSP (primitive `csp.rs`), pas le gate `run_gate_csp_authoring` directement.**
`crates/nexus-core-rs/tests/factory_csp_contract.rs:18-24` + `docs/factory/examples/csp_contract.rs:7-12` : le gate vit dans `sbfb-factory` (crate binaire-pur, pas de lib target) donc non liftable via `use`. L'anti-rot couvre la dérive de valeur du contrat CSP, pas une régression de logique du gate lui-même.

**P2-7 — `sprint79_phase_b_codex_review.md` committé STALE.**
Codex lui-même (round 4, P2) écrivait « update or delete that untracked review file before committing it » ; le fichier a été committé tel quel à `b27079c`, gelant un artefact que Codex déclarait « stale and now false ». Incohérence de discipline d'artefact relevée par aucune track initiale.

**P2-8 — Run de 6 PLAN-ADAPT consécutifs (8/9 phases) : le plan/kickoff portait une imprécision factuelle récurrente.**
Verdicts G8 : C=EXECUTE seul ; A,B,D,E,F,G,H,I=PLAN-ADAPT (run D→I = 6 consécutifs). Corrections factuelles explicites : `sprint79_phase_a_preflight.md:66` (« hashé GRATUITEMENT par compute_output_hash » → FALSE), `sprint79_phase_g_preflight.md` (« 8 thèmes built-in » → daisyUI 5.5.23 = 35 thèmes), `sprint79_phase_d_preflight.md` (`daisyui/MANIFEST.json` inexistant à Phase D). Tous evidence-based, aucun ne touche une Day-0 — mais le signal méta indique un plan initial fragile. Reco : durcir le kickoff factuel S80.

### P3 (11 — nits)

- **P3-1** — Baseline absolu nextest 1991→1994 non re-rejoué dans l'audit (seul le delta +3 `factory_csp_contract` est ancré, full workspace hors-périmètre). Risque résiduel de compte gonflé si une autre suite a régressé.
- **P3-2** — `docs/rust/PATTERNS.md:3854` §P70 layer 2 prose « F to come » stale : le pack daisyUI (Phase F) a livré dans le même sprint (`docs/factory/knowledge/daisyui/MANIFEST.json` existe à `f4b4600`).
- **P3-3** — Drift numérique : `PATTERNS.md:3884-3886` « 22 of the 25 » familles non-schématisées vs `sprint80_audit_plan.md:58-60` « ~21 ».
- **P3-4** — `sprint79_verification.md:92-110` §5 (carry-over memory) omet 2 des 6 carries (Track Testabilité standing T1/T2 + TEST-ISOLATION-SBFB-HOME) présents dans `sprint80_audit_plan.md §3` ; auto-flag existant `sprint79_phase_i_review.md:132`.
- **P3-5** — Nom « volet (5) line-semantic » (`sprint79_verification.md:50-53`) sur-vend un check existence+bornes-de-ligne (`sprint80_audit_plan.md §3` caveat correctement « pas que la ligne supporte encore la claim »).
- **P3-6** — Preuve PASS de T2 au wrap-up = assertion prose ; l'artefact JSON est gitignored (`.gitignore` `545c78e` : `scripts/acceptance/.app_authoring_last_result.json` + `.app_authoring_pw.json`, `git ls-files` → aucun committé). `sprint79_verification.md:76-78` affirme « T2 artefact JSON PASS confirmés verts » en prose.
- **P3-7** — `docs/security/HARDENING_ROADMAP.md` non référencé pour la surface app-authoring alors que le track le nomme ; mais c'est un doc historique scopé « Sprint 18-30 » — `THREAT_MODEL.md §13.1` est la bonne localisation (couverture satisfaite sur le doc vivant).
- **P3-8** — Volet (3) french-body de `check-factory-docs.sh:106` quasi-vacant : `EN_WORDS` = sous-ensemble de 9 termes UI seulement ; de larges pans d'anglais non-traduit passeraient. Limite documentée « narrow subset » (l.104).
- **P3-9** — Honesty-gate = présence-seule (`check-factory-docs.sh:91-104,224-230` `require_marker = grep -qF`), aucun ban négatif d'une bannière contradictoire ; corroboré `sprint79_phase_i_review.md:126`.
- **P3-10** — Volet (5) fiche-line `PACK_DIR` figé à `animejs` (`check-factory-docs.sh:243-244,249`) : une ref bare-name daisyUI (`PRIMITIVES.md:N`) ne serait pas résolue vers le bon pack.
- **P3-11** — `sprint79_frontier_volet4_codex_review.md:40,68` PARTIEL disclosé (« 16-hex non-pack interdit non explicitement nommé ») non routé ; lot codex-only sans `_review.md` dédié.

## Carries P1 escaladés (PAS des régressions — section dédiée)

Ces deux carries sont des gaps CONNUS, honnêtement reportés ; ils ne comptent pas comme P1 bloquants du sprint.

- **CARRY-P1-A — Sharding S77 PROVISIONAL** : orchestrateur de session in-vivo + benchmark live cross-machine 2-machines = RIG-ABSENT. `sprint80_audit_plan.md:50-53`. Le diff `9297f08~1..f4b4600` ne touche aucun orchestrateur/benchmark (factory-only) ; `ShardSessionPanel.tsx` RÉAFFIRME « carry S78 » / « seam non encore branché » au lieu de le clore. Honnête, non sur-vendu.
- **CARRY-P1-B — app-authoring in-vivo `Not evidenced`** : parcours auteur réel → gate → self-check → publish → rendu cross-pair JAMAIS exercé in-vivo ; efficacité générative non mesurée. `sprint80_audit_plan.md:54-57` + `sprint79_verification.md:94-97`. Le harness T2 (`scripts/acceptance/app_authoring_capability.sh`) est HERMÉTIQUE (Playwright spawn daemon local, fixtures CLEAN/DIRTY committées, verdict JSON PASS/BLOCK/RIG-ABSENT) — couvre le statique, pas le flux cross-pair. « Not evidenced » exact. Contraste avec Day-0 #11 « 0 defer du cœur » : tension réelle, mais la capacité STATIQUE (gate+packs+template+self-check) est LIVE-testée ; seul le bout-en-bout in-vivo est différé → carry légitime, pas régression.

**À router aussi en S80** : le nouveau P1-1 (gate CSP non câblé sur `redeploy`/`deploy-workspace`/`deploy-from-repo`) est actuellement ABSENT de `sprint80_audit_plan.md §3` — l'ajouter au registre forward est une condition du CONDITIONAL PASS.

## Méta-process & testabilité (G8)

- **Gate de testabilité §4 LIVRÉ** : T1 `web/e2e/app-authoring.spec.ts` (3 sous-tests untagged → inclus dans `--grep-invert @compute`, BLOQUANT) câblé `ci.yml` step [10c] + `verify.sh` step 15, avec contrôle négatif load-bearing (clean=0 / dirty≥1). T2 `scripts/acceptance/app_authoring_capability.sh` émet artefact JSON `PASS/BLOCK/RIG-ABSENT` avec gardes anti-faux-vert (TESTS_TOTAL≥3, SKIPPED==0, parser python3 obligatoire). Aucune prose `DIFFERE-materiel` non-machine-lisible dans le scope S79.
- **Doc-lints BLOQUANTS prouvés non-vacuous** (T5, mutation transitoire restaurée) : `check-factory-docs.sh` détecte drift source-ref, marqueur d'honnêteté manquant, ligne fiche hors-bornes ; l'exemple `include!` casse 2/3 tests sur drift de valeur CSP. `check-frontier-contracts.sh` a une garde anti-no-op (`grep -rqF '// FRONTIER: ShardPlan '` obligatoire). Câblés 3 surfaces (Woodpecker `bash:5`, GHA, `verify.sh` sous `set -euo pipefail`).
- **G8 verdicts** : 1 EXECUTE + 8 PLAN-ADAPT (evidence-based, 0 Day-0 touchée — le PLAN-ADAPT Phase E a REFUSÉ d'importer `nexus-shell-daemon-core` [+32 crates] pour tenir « 0 dep / Factory hors daemon », factorisant `BLOB_SERVE_CSP` dans `nexus-core-rs/src/csp.rs:33`). Run de 6 consécutifs → P2-8.
- **Review files** : 9 phases A-I × triplet (preflight/review/codex_review) présents ; Codex bruts (Phase B en anglais = signature de non-réécriture) ; Phase I 7/7 CONFIRMÉ 0 GAP vérifié dans l'output brut.

## Note de clôture

Sprint techniquement sain et exceptionnellement bien instrumenté : la couche docs-contrat et les gates sont prouvés non-vacuous, la source CSP est réellement unique, 0 bump wire, 0 dispense d'isolation, scellage additif. Le seul écart structurel — un verbe central de la boucle d'authoring (`redeploy`) qui contourne le gate flagship — n'est pas une faille de sécurité exploitable (la CSP runtime tient) mais une **claim sur-affirmée et non escaladée** ; il doit être routé et tranché en Phase 0 S80. Combiné aux deux carries in-vivo honnêtes, cela classe S79 en **CONDITIONAL PASS** : le cœur statique est livré et éprouvé, l'épreuve in-vivo et la cohérence du périmètre de scellage restent à fermer.

---

## Réconciliation main-thread (vérification indépendante)

Le main thread a re-joué les ancres load-bearing du P1-1 sur le code réel (`f4b4600`), sans se fier à la synthèse des agents :

- `grep -rn run_gate_csp_authoring crates/` (hors tests) → **un seul** appel de production : `crates/sbfb-factory/src/pipeline.rs:52`, délibérément hors du bloc `skip_gates` (commentaire `pipeline.rs:48`, test `pipeline.rs:195`). CONFIRMÉ.
- `crates/sbfb-factory/src/atelier.rs:70` `redeploy()` → POST `/api/v1/deploy-workspace` (`atelier.rs:114`) ; **0 hit** `run_gate|csp|authoring` dans `atelier.rs`. CONFIRMÉ.
- `crates/nexus-shell-daemon/src/deploy.rs` `deploy_from_repo:65` + `deploy_workspace:233` → **0 hit** `csp|run_gate|authoring`. CONFIRMÉ.

Donc le P1-1 est **réel** : le verbe `redeploy` (cœur de la boucle d'authoring fork→edit→iterate) et les routes daemon publient des octets d'app sans rejouer le gate CSP « non-délégable ».

**Tension décision gelée** : câbler le gate *côté daemon* (`deploy.rs`) contredirait la décision figée « Factory = outil client externe, hors daemon » (D2). Le daemon reste neutre ; la frontière qu'il applique inconditionnellement à TOUTE app est la **CSP runtime blob-serve** (`csp.rs:33`, inchangée). La résolution du P1-1 est donc un **arbitrage Day-0** (fix client-side `redeploy` ± amendement de la formulation « scellage 100% Factory ») et NON un fix mécanique — escaladé au PO.

**Preuve de suites indépendante** : `cargo nextest run --workspace --locked` (Win natif) = **1994 tests run, 1994 passed, 0 skipped** (exit 0). Le delta annoncé 1991→1994 (+`factory_csp_contract`) est CONFIRMÉ, baseline absolu inclus (clôt le P3-1).

**Verdict main-thread** : CONDITIONAL PASS confirmé. Le P1-1 est routé dans `sprint80_audit_plan.md §3` (condition du CONDITIONAL) ; sa résolution (fix vs amendement Day-0) attend l'arbitrage PO avant la Phase A de S80.

### P1-1 — RÉSOLU en Phase 0 (mise à jour post-arbitrage)

Arbitrage PO : **Option A** (fix client-side + amendement Day-0). Fermé par le
commit `c0a2ffe` :
- `crates/sbfb-factory/src/atelier.rs` : `redeploy()` appelle désormais
  `run_gate_csp_authoring` AVANT le zip + la découverte daemon (mirror
  `pipeline.rs`, gate BLOQUANT) ; le verbe `redeploy` est scellé à l'identique
  de `publish`. Test `redeploy_blocks_on_csp_violation` (+1 Rust).
- **Daemon INCHANGÉ** : les routes `deploy.rs` restent neutres (câbler le gate
  côté daemon contredirait la décision gelée « Factory hors daemon » / D2). La
  frontière inconditionnelle de toute app reste la CSP runtime (`csp.rs:33`).
- **Amendement Day-0** : « scellage 100% Factory » = le client Factory gate
  chaque verbe de publish (`publish` + `redeploy`) ; daemon neutre + CSP runtime
  = frontière inconditionnelle. Documenté `docs/factory/FACTORY_GATES.md`
  (§Non-delegable + §Portee du scellage + Principe 1) + commentaires code.
- Vérification : fmt 0 / clippy 0 (sbfb-factory) / nextest sbfb-factory 201/201
  0-skip / 3 doc-lints clean.

**Conséquence verdict** : le nouveau P1 (la condition du CONDITIONAL) est levé
en Phase 0 → **PASS effectif** pour le périmètre S79. Demeurent les **2 carries
P1 honnêtes** (sharding S77 in-vivo RIG-ABSENT, app-authoring in-vivo
`Not evidenced`) — escaladés, non bloquants, normaux pour ce sprint.
