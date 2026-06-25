# Review Phase E — Sprint 79 (Factory app-authoring) — ROUND 2

> Methode : Workflow ultracode `wf_018e684f-08e` (round 1, FAIL) puis
> `wf_af30e01a-b5f` (round 2, PASS-PENDING) — 5 dimensions Opus 4.8 1M +
> verification adversariale + synthese. Etat mecanique re-verifie apres
> correctifs : `cargo fmt --all --check` clean ; `cargo clippy -p nexus-core-rs
> -p nexus-shell-daemon-core -p nexus-shell-daemon -p sbfb-factory --all-targets
> -- -D warnings` exit 0 ; `cargo nextest -p nexus-core-rs -p sbfb-factory`
> **640/640 0-skip** ; `node check-csp.mjs` showcase PASS ;
> `bash scripts/check-frontier-contracts.sh` exit 0.

## Contexte

Phase E livre le **gate CSP DETERMINISTE Rust BLOQUANT** au publish Factory et
**factorise la source CSP** en une verite unique. Le round 1 avait renvoye FAIL
(1 P1 confirme + P2/P3). Cette re-review verifie de visu sur le code reel que :
(a) le P1 est reellement ferme, (b) les fixes n'ont pas casse la detection,
(c) aucun nouveau P0/P1. Cas B, AVANT commit atomique et AVANT Codex.

Verifications live executees par le reviewer :
- `bash scripts/check-frontier-contracts.sh` -> **EXIT 0**
- `node examples/daisyui-animejs-showcase/scripts/check-csp.mjs` -> **EXIT 0**
- `git show HEAD:.../blob_serve.rs | grep BLOB_SERVE_CSP` -> **byte-identique** a `csp.rs:33`
- `grep nexus-shell-daemon crates/sbfb-factory/Cargo.toml` -> **absent** (seule `nexus-core-rs`)
- regex testees empiriquement sur les shapes adverses

## Dimension 1 — regex-security : CONCERN -> resolu (voir Correctifs)

- **[INFO] P1 round 1 reellement ferme.** `scripts/check-frontier-contracts.sh`
  `CSP_FILE="crates/nexus-core-rs/src/csp.rs"` ; assertion des 6 directives
  `'none'` sur la ligne unique `csp.rs:33`. Run live = EXIT 0. Gate CI restaure.
- **[P2 -> FIXED]** Faux-negatif : tags a separateur slash `<script/src=//evil>`
  echappaient aux patterns d'attribut (exigeaient `\s` apres le nom de tag) ET
  au catch-all (seulement `https?://`). Durcissement P3 round 2 incoherent
  (presence en `[\s/>]`, attributs en `\s`). **Corrige post-review** (voir §Correctifs).
- **[P3]** `//host` protocol-relatif isole dans une string JS hors contexte
  attribut non detecte. Documente (CSP runtime + Phase H) dans FACTORY_GATES.md.
- **[P3]** `setAttribute('action',…)` et `<iframe>` litteral flaguent quelle que
  soit la valeur (faux-positif conservateur, formes mortes sous sandbox sans
  `allow-forms`). Acceptable per preflight adaptation 7.

## Dimension 2 — scope-planadapt : PASS

Les 6 adaptations PLAN-ADAPT realisees fidelement, aucune Day-0 violee.
- ADAPT 1 : `BLOB_SERVE_CSP` en EXACTEMENT un endroit (`csp.rs:33`),
  `blob_serve.rs` `pub use nexus_core_rs::csp::{...}` (consts supprimees, pas
  dupliquees), `lib.rs` re-exporte, `http.rs:556` inchange. `sbfb-factory/Cargo.toml`
  : AUCUN edge daemon-core -> 0 crate transitif neuf, Factory-hors-daemon (v4 D2) tenu.
- ADAPT 4 : `run_gate_csp_authoring` a `pipeline.rs:52` HORS du bloc
  `if !skip_gates`, BLOQUANT (return Err) ; `test_pipeline_csp_gate_blocks_even_with_skip_gates`
  (skip_gates=true) prouve le non-bypass.
- Gate DETERMINISTE : pur regex/WalkDir, 0 ML/scoring/random/reseau. 0 bump wire.
  Daemon neutre (aucune route).
- ADAPT 2/3/5/6 realises (mirror + test anti-drift ; test cross-crate inline
  binary-only ; TempDir inline ; check-csp.mjs consomme le manifeste).
- **[P3]** Classification Vendored par nom de fichier (`*.min.js`/`*.umd.js`) :
  micro-surface assumee, primitives reseau verifiees dans tous les tiers.
  **Documentee** dans FACTORY_GATES.md (post-review).

## Dimension 3 — test-coverage : PASS

Fondations anti-drift SOLIDES. `test_csp_gate_covers_every_none_directive` est un
vrai garde (itere `none_directives(BLOB_SERVE_CSP)` x `CSP_RULES`) ;
`csp_contract_json_mirrors_the_rust_consts` asserte csp/none_directives/css_url_allow
par recompute ; `test_pipeline_csp_gate_blocks_even_with_skip_gates` prouve le
non-bypass ; `test_csp_gate_allowlist_matches_on_origin_boundary` couvre
l'anti-bypass starts_with ; `test_csp_gate_case_sensitivity` couvre
Fetch/prefetcher negatif + `<FORM ACTION>` positif. Chaque LABEL de CSP_RULES a
un declencheur teste.
- **[P2 -> FIXED]** Aucun test ne gardait le groupe `[^>]*` (attributs avant
  l'attribut reseau). **Corrige post-review** (cas `attrs before form action` +
  `attrs before link href`).

## Dimension 4 — cross-crate-drift : PASS

- `BLOB_SERVE_CSP` (`csp.rs:33`) BYTE-IDENTIQUE a HEAD `070c7a9`, COOP/COEP idem.
  Aucun changement silencieux de la CSP runtime.
- Re-export preserve le SEUL consommateur runtime `http.rs:556/561/565` sans
  toucher les call-sites ; grep exhaustif confirme aucun autre consommateur casse.
- `csp-contract.json` correspond EXACTEMENT aux consts (test recompute + a l'oeil).
  Chemin relatif resout (node EXIT 0).
- Parite rule-par-rule JS NETWORK vs Rust CSP_RULES : ensemble IDENTIQUE, flags de
  casse alignes, 4 nouvelles directives + `//` + type=module des deux cotes.
- **[INFO]** `//host` hors contexte non detecte par aucun gate — limite partagee
  assumee, documentee.

## Dimension 5 — docs-patterns : PASS

- **[INFO]** Aucune promesse de phase future interdite : les refs « Phase H »
  sont des pointeurs de design (copule « is »/« self-check »), aucune ne matche
  `PROMISE_RE`. Conforme §6.12.
- FACTORY_GATES.md (FG-CSP) + THREAT_MODEL §13.1 decrivent EXACTEMENT le code
  (3 tiers, hors skip_gates, source unique, rationale form-action-vs-connect-src
  correct) et sont HONNETES sur les limites. Constantes nommees respectees.
- **[P3 -> FIXED]** Faux-positif intentionnel du tier Scanned (URL affichee
  non-fetchee refusee) maintenant explicite cote utilisateur. **Doc ajoutee** (post-review).
- **[INFO]** FG7 dans le diagramme apres FG-CSP : ecart PRE-EXISTANT (pipeline ne
  lance pas FG7), pas une regression Phase E.

## Verif adversariale des findings P0/P1

Aucun P0/P1 confirme. Le seul candidat « securite » (slash-separator bypass) a ete
teste empiriquement, classe P2 (mitige par la CSP runtime + nature
authoring-discipline), puis **ferme post-review**. Aucun faux-positif bloquant une
app legitime (CSS_URL_ALLOW applique au tier Scanned y compris `.css` ; tier
Vendored garde license/namespace ; `.json`/`SBFB.json`/provenance skip). Templates
react/static verifies PASSENT.

## Correctifs post-review (in-phase, avant Codex)

Les 3 P2/P3 carry-recommandes ont ete fermes dans la phase (pas reportes) :
1. **Slash-separator faux-negatif** : les 4 patterns d'attribut
   `<script>`/`<link>`/`<form>`/`<base>` + `MODULE_SCRIPT_PATTERN` passent de `\s`
   a `[\s/]` apres le nom de tag (Rust `gates.rs` + miroirs `check-csp.mjs`),
   fermant `<script/src=//evil>`/`<base/href=//evil>`. Cas de test ajoutes
   (`slash-separator script src`, `slash-separator base href`).
2. **Garde du groupe `[^>]*`** : cas `attrs before form action` +
   `attrs before link href` ajoutes a `test_csp_gate_rejects_violations`.
3. **Honnetete doc** : FACTORY_GATES.md documente la discipline no-CDN (URL
   affichee non-fetchee refusee) + la classification Vendored par nom de fichier.
   THREAT_MODEL §13.1 + la docstring du gate listent le `//host` isole comme
   faux-negatif assume (autorite = CSP runtime + Phase H).

Suites re-passees apres correctifs : fmt clean, clippy exit 0, nextest 640/640,
check-csp.mjs PASS, check-frontier-contracts exit 0.

## Codex reconciliation (GPT 5.5)

Codex `codex exec` joue 2 rounds (artefact brut `sprint79_phase_e_codex_review.md`,
non reecrit) :
- **Round 1** : 9 livrables, 8 CONFIRME, 0 GAP, 1 PARTIEL + risque FAUX-POSITIF
  reel : la regle `@import url(` bloquait TOUT import, or un `@import url('./local.css')`
  resout same-origin (`default-src 'self'`) et est legitime. **Corrige** : regle
  `@import` rendue remote-only (formes `url()` + string), Rust + miroir JS, + test
  `test_csp_gate_local_css_import_passes`.
- **Round 2** (code final) : 9 livrables, **8 CONFIRME, 0 GAP, 1 PARTIEL**. Le FP
  `@import` n'apparait plus. Points de risque verifies : gate hors `skip_gates`
  CONFIRME ; CSP byte-identique (`old_length=215 new_length=215 byte_identical=True`)
  CONFIRME ; aucun edge `sbfb-factory -> nexus-shell-daemon-core` CONFIRME.

Le PARTIEL residuel + 2 notes de risque sont des **decisions de design
Day-0-conformes / limites acceptees** (documentees, non corrigees) :
1. **PARTIEL (Livrable 4)** : l'impl de production importe `CSS_URL_ALLOW` (meme
   module source-unique `nexus_core_rs::csp`) ; `BLOB_SERVE_CSP` est consomme comme
   AUTORITE ANTI-DRIFT par le test cross-crate (`none_directives(BLOB_SERVE_CSP)`).
   Aucune chaine CSP n'est re-hardcodee. Day-0 #4 (« importer BLOB_SERVE_CSP, jamais
   re-hardcoder, factoriser + test cross-crate ») est satisfait — le gate applique
   les directives par regex et le test lie la couverture a la policy. Choix delibere
   (enforcement-par-regex + binding-par-test), pas une demi-implementation.
2. **`<form action="/local">` non bloque** : la regle `form-action` cible
   l'exfiltration (action distante `https?:`/`//`) + `setAttribute('action')`.
   `form-action 'none'` + le sandbox sans `allow-forms` bloquent DEJA toute
   soumission au runtime ; flaguer tout `<form>` ferait des faux-positifs sur les
   forms a handler JS legitimes. La conformite des forms a action locale (mortes
   sous sandbox) est laissee au runtime + Phase H (preflight adaptation 7, review P3).
3. **`<img src="//evil">` protocol-relatif** hors contexte link/script/@import/url() :
   faux-negatif assume, documente dans `FACTORY_GATES.md` §Limites (couvert par CSP
   runtime + self-check Phase H).

Aucun GAP P0/P1 sur les 2 rounds. Suites re-passees apres le fix `@import` : fmt
clean, nextest CSP 13/13, check-csp.mjs PASS.

## Verdict : PASS

0 P0/P1 confirme (review 2 rounds + Codex 2 rounds). Le P1 du round 1 (gate CI
`check-frontier-contracts.sh`) est ferme ; le FP `@import` trouve par Codex est
ferme ; les 6 PLAN-ADAPT et toutes les Day-0 sont honorees ; les carries
review/Codex sont soit fermes in-phase, soit documentes comme limites assumees
cohérentes avec la nature « gate de surface » (autorite = CSP runtime + Phase H).
Verdict promu PASS apres reconciliation Codex.
