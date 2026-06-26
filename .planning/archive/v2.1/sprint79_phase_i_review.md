# Sprint 79 — Phase I — Review consolidée

**Phase** : I — docs-contract closure (Diátaxis FR + llms.txt + WIRING_SPEC + check-factory-docs.sh) + wrap-up final S79
**Méthode** : Workflow review 7 dimensions + passe adversariale indépendante (rejeu par commande des 3 risques majeurs : vacuité gate, source_ref mort, honnêteté/bump-caché/faux-vert CI)
**Périmètre staged** : 19 fichiers — CI yaml (3), planning (3), `CLAUDE.md`, `SPRINT_LOG.md`, `docs/factory/*` (7), root `llms.txt`, `scripts/*` (3), test `crates/nexus-core-rs/tests/factory_csp_contract.rs`
**Gate testabilité §4** : T1 `app-authoring.spec.ts` BLOQUANT + T2 JSON PASS hérités/maintenus ; livrable Phase I = exemple runnable `include!` (+3 Rust 1991→1994)
**Verdict global** : **PASS-PENDING** (review propre, Codex pas encore joué — PASS-PENDING n'est pas un verdict final committable)

---

## Dimension 1 — Exemple runnable + test `include!` (example-test)

**Synthèse : PASS.** L'exemple `docs/factory/examples/csp_contract.rs` prouve réellement le contrat CSP, sémantiquement et empiriquement :
- 6 directives `'none'` assertées verbatim contre `csp.rs:33` ;
- `CSS_URL_ALLOW` (allowlist 3 URLs svg/xlink/tailwindcss) assertée contre `csp.rs:48-52` ;
- garde anti-drift `AUTHORING_RULES` ↔ `none_directives(BLOB_SERVE_CSP)` (lockstep ordre-sensible).

Option B correcte : re-export top-level `nexus_core_rs::{BLOB_SERVE_CSP, CSS_URL_ALLOW, none_directives}` (`lib.rs:104`), **zéro** dépendance au gate bin-privé `run_gate_csp_authoring`. L'exemple prouve la **source-of-truth** que le gate importe, pas le gate lui-même — honnêtement disclaimé in-file. Math du chemin `include!` correct (`CARGO_MANIFEST_DIR=crates/nexus-core-rs` + `../../` → racine → `docs/factory/examples/csp_contract.rs`), commentaires `//` (pas `//!`) corrects pour un fichier inclus, miroir du précédent sharding `shard_sign_verify.rs`. Drift-guard prouvé empiriquement : mutation `object-src 'none'`→`'self'` fait rougir 2 tests, revert propre.

**Findings retenus** : P3 (« breaks the build » vs test-runtime) ; P3 (vec ordre-sensible plus strict que la prose « set », cohérent avec l'unit test existant — aucun changement requis). Aucun P0/P1/P2.

---

## Dimension 2 — Gate `scripts/check-factory-docs.sh` (gate-script)

**Synthèse : PASS.** Gate sain, **non-vacuux** (11 mutations adversariales → chaque branche FAIL avec le bon diagnostic, restore EXIT=0), BusyBox/bash-safe, injection-free (zéro `eval`, zéro `source`/dot-include ; un `.md` n'est jamais que nourri à `grep`/`wc`/expansion). Volet line-semantic R2 présent et correct (résolution `PRIMITIVES.md:N`/`README.md:N` contre `docs/factory/knowledge/animejs/`, parse slash-lists + ranges, bornes correctes ; les 19 refs de fiche sont in-bounds). Allowlist required-anchor des 9 symboles load-bearing complète et correcte. Run repo : EXIT=0 « clean ».

Caveat acceptable : process substitution `< <(grep …)` (bashism) à 4 sites, mais shebang `#!/usr/bin/env bash` et invocation `bash …` sur image `bash:5` GNU dans les 3 surfaces — cohérent avec le frère `check-sharding-docs.sh`.

**Findings retenus** : P3-1 (pack-scope `animejs` codé en dur, latent daisyui) ; P3-2 (parse symbole premier-vs-dernier `:`) ; P3-3 (`wc -l` off-by-one, direction conservatrice). Tous latents, non déclenchés aujourd'hui. Aucun P0/P1/P2.

---

## Dimension 3 — Câblage CI 3 surfaces + non-régression gates voisins (ci-wiring)

**Synthèse : PASS.** Les 3 surfaces toutes BLOQUANTES et syntaxiquement correctes :
- `.github/workflows/ci.yml:125-126` `[16] factory docs check` (BLOQUANT par défaut GHA, pas de `continue-on-error`) ;
- `.woodpecker/ci-linux.yml:85-87` step `factory-docs-check`, digest `bash:5@sha256:2003051c…` **identique** aux 4 steps bash frères ;
- `scripts/verify.sh:114-115` step 21 sous `set -euo pipefail` → BLOQUANT.

Exactement 3 surfaces, 0 orphelin. Gates voisins NON-régressés : `check-sharding-docs.sh` EXIT=0 (marqueur migré `sharding subsystem only`→`whole-repo agent index is`, sentinelle plus durable ancrée `llms.txt:11`), `check-frontier-contracts.sh` EXIT=0 (incl. non-régression `BLOB_SERVE_CSP`). Root `llms.txt` reword cohérent (bannière sharding+factory, section Factory ajoutée, marqueurs des 2 gates présents, pas de double-couverture orpheline — les 3 liens factory sont link-checkés par `check-sharding-docs.sh`). `bash -n` OK sur les 3 scripts.

**Findings retenus** : 1 P3 informatif (BusyBox-safety vérifiée localement sous GNU grep seulement ; parité établie). Aucun P0/P1/P2.

---

## Dimension 4 — Honnêteté (honesty)

**Synthèse : PASS.** Aucune doc Phase I ne prétend « shipped/LIVE-in-production/déployé » de ce qui est statique/local. Truth-Stack respecté :
- PROVISIONAL + `Not evidenced` (parcours in-vivo + efficacité générative) couverts sur les 6 docs (REFERENCE en EN exempté french-body par design ; EXPLANATION délègue le détail à README — gate-compliant) ;
- caveat cardinal 2-clauses + `0 verdict PASS` cohérents avec `FACTORY_GATES.md:132-183` et le code (`operator_server.rs:437,696` `chat_history_authoritative=false` ; `csp.rs:33` 6 directives `'none'`, 3 tests verts) ;
- wrap-up n'invente aucune claim : delta Rust 1991→1994 vérifié (3 tests réels), « LIVE » = testé hermétiquement (distingué de « LIVE in production »), 3 doc-lints verts, sharding S77 PROVISIONAL correctement reporté.

**Findings retenus** : P3-a (honesty-gate sans BAN négatif explicite, protection indirecte) ; P3-b (clause-2 du caveat non grep-ée directement) ; P3-c (« confirmé » au gate dual-platform devance le run Docker, encadré comme gate pré-push). Aucun P0/P1/P2.

**Note environnement scellée** : un premier passage a renvoyé une vue périmée de `llms.txt` + `csp.rs:33` (affichant `object-src 'self'`) → faux échec apparent. Re-run propre = tout vert / `object-src 'none'`. Artefact de snapshot, pas un finding.

---

## Dimension 5 — Exactitude des source_refs (source-refs, adversarial)

**Synthèse : PASS avec 2 P2.** Chaque `path:Symbol` de `WIRING_SPEC.md` et `llms.txt` a été re-grepé et **EXISTE** ; zéro ancre morte (aucun `blob_serve.rs:286`, aucun `run_gate_authoring_csp`, aucune ancre par numéro de ligne — discipline R3 tenue) ; les 18 liens markdown résolvent. Claims secondaires corroborées byte-for-byte (re-export `lib.rs:104`, miroir `csp-contract.json`, 6 directives, math `include!`, `app-authoring` = 9ᵉ entrée `process.rs:22`).

Mais deux symboles sont **surcharacterisés** (existence valide, contenu/câblage décrit surévalué) :
- **P2-1** — `WIRING_SPEC.md:104-111` laisse entendre que les 2 packs sont surfacés par le context-pack ; `operator_server.rs:363` ne contient qu'`animejs`. daisyui MANIFEST.json est un fichier repo hashé par son test, **pas** émis dans le tableau `authoring_knowledge`.
- **P2-2** — `WIRING_SPEC.md:80` + `llms.txt:30` disent « imports `BLOB_SERVE_CSP` + `none_directives` / imports the policy » ; en prod le gate importe **uniquement** `CSS_URL_ALLOW` (`gates.rs:7`) + table manuelle `CSP_RULES` (`gates.rs:194`). La liaison à `BLOB_SERVE_CSP`/`none_directives` est **test-time** (`gates.rs:1176-1183`), pas import-time.

**Findings retenus** : P2-1, P2-2 (précision de wording, non-bloquant). Aucun P0/P1.

---

## Dimension 6 — Scope cuts + Day-0 + invariants (scope-daydir)

**Synthèse : PASS.** Les six invariants tiennent :
1. **0 bump wire** — aucun fichier canonical touché, aucun `*_FORMAT_VERSION` dans le diff.
2. **0 dep nouvelle** — aucun `Cargo.toml`/`package.json` ; l'unique nouveau Rust n'utilise que des symboles déjà re-exportés (`lib.rs:104`, hors-diff).
3. **0 nouvelle primitive de code** — uniquement test/exemple ; les primitives du gate (`run_gate_csp_authoring`, `none_directives`, `BLOB_SERVE_CSP`) intactes.
4. **Scellage 100% Factory** — aucun `http.rs`/route daemon/autorité-verdict ; propriété non-délégable re-documentée, jamais relâchée ; le nouveau gate est strictement grep/compare.
5. **Connaissance consommée, jamais autoritaire** — `0 verdict PASS` mécaniquement enforced ; `chat_history_authoritative` inchangé.
6. **Day-0 figées (kickoff 177-234)** — #4 source CSP unique respectée (ancre corrigée vs stale `blob_serve.rs:286`), #6 pas de wrapper skills (différé), #8 lint additif, #9 vendoring UMD doctrine inchangée, #11 cœur livré A→H — Phase I = closure docs-contract, pas un defer du cœur.

**Findings retenus** : P3-1 (span A→I > estimation A→G, COMPLIANT) ; P3-2 (Tutoriel Diátaxis différé, documenté/justifié). Aucun P0/P1/P2.

---

## Dimension 7 — Wrap-up + langue + structure docs (wrapup-lang)

**Synthèse : PASS.** Artefacts cohérents et recoupés :
- `sprint79_verification.md` ↔ `sprint80_audit_plan.md` : 8 phases feature A–H + Phase I, 8 hash feat + 4 chores absorbés tous réels en git, tests 1991→1994 (+3) concordants partout, carries P1 (sharding PROVISIONAL + app-authoring in-vivo Not evidenced) + P2 (~21-familles-wire) + Track Testabilité présents ;
- `SPRINT_LOG.md` row 79 sous le bon en-tête `## v2.1`, 5 colonnes, pas de row 78 (S78 déféré) ;
- `CLAUDE.md` Etat actuel : 0 occurrence « S79 OUVERT »/« Phase A LIVREE »/« S78 a ouvrir », ouverture « Sprints 0-77 CLOSED + S79 DONE » ;
- Diátaxis 1:1 propre (EXPLANATION/HOW_TO_WIRE/REFERENCE + Tutoriel honnêtement différé ; README=hub, WIRING_SPEC/llms.txt=couche agent) ;
- french-body OK (corps FR accentués ; REFERENCE EN assumé et exempté par le gate) ;
- 16/16 liens internes résolvent, toutes ancres §P70/§P71/§6.12/AGENT_SYSTEM §7 présentes, tous symboles grep-résolus.

**Findings retenus** : P3-1 (anime.js « (v4.5) » vs « 4.5.0 ») ; P3-2 (verification §5 omet Track Testabilité — pas une incohérence, audit_plan = registre canonique sur-ensemble). Aucun P0/P1/P2.

---

## Passe adversariale — synthèse

**Verdict adversarial : PASS-PENDING.** Rejeu par commande des 3 risques majeurs, toutes sondes négatives :
- **Vacuité gate RÉFUTÉE** — mutation `run_gate_csp_authoring`→`run_gate_csp_BOGUS` dans WIRING_SPEC fait FAIL le gate (double diagnostic « symbol not found » + « load-bearing clause unanchored »), restore EXIT=0.
- **source_ref mort : AUCUN** — 0 ligne `+` `FORMAT_VERSION|ANNOUNCEMENT_VERSION|schema_version` ; aucun `blob_serve.rs:286`/`run_gate_authoring_csp` résiduel.
- **Bump wire caché : AUCUN** — 19 fichiers staged = 0 fichier wire/canonical ; `PROMPT_KINDS` = exactement 9 entrées, `app-authoring` 9ᵉ.
- **Faux-vert CI RÉFUTÉ** — 3 surfaces BLOQUANTES confirmées, gates voisins non-régressés (EXIT 0).

**Faux positif scellé pour Codex** : l'artefact « object-src 'self' » signalé par 3 dimensions (honesty, wrapup-lang, scope-daydir) est un **glitch de snapshot d'outil, PAS un bug**. Vérifié direct : `csp.rs:33` contient `object-src 'none'` (6 directives `'none'` réelles), 3 tests `factory_csp_contract` passent 3/3 0-skip. Si un agent rouvre ce « bug object-src », c'est un faux réveil — il est réglé.

**P2-1 + P2-2 CONFIRMÉS** par l'adversarial comme réels : ils touchent la clause de câblage la plus load-bearing (« la source CSP est unique / le gate ne ré-implémente pas la politique ») dont la formulation actuelle induit en erreur sur le mécanisme réel (lockstep par test, pas import runtime). Non-bloquant pour le DONE, mais à ne pas droper silencieusement.

---

## Findings consolidés

| Sévérité | Titre | Evidence |
|---|---|---|
| **P2-1** | WIRING_SPEC §CONTEXT-PACK surcharacterise `AUTHORING_KNOWLEDGE_MANIFESTS` (animejs seul) | `docs/factory/WIRING_SPEC.md:104-111` laisse entendre les 2 packs surfacés ; `operator_server.rs:363` ne contient qu'`animejs` (commentaire :362 « animejs pack only at this revision »). daisyui MANIFEST.json hashé par test mais absent du tableau `authoring_knowledge`. §KNOWLEDGE :122-130 correct. → resserrer la phrase CONTEXT-PACK. |
| **P2-2** | « le gate imports BLOB_SERVE_CSP + none_directives / imports the policy » — liaison test-time, pas import runtime | `WIRING_SPEC.md:80` + `llms.txt:30`. Gate runtime importe UNIQUEMENT `CSS_URL_ALLOW` (`gates.rs:7`) + `CSP_RULES` manuelle (`gates.rs:194`) ; `BLOB_SERVE_CSP`/`none_directives` liés seulement au test anti-drift (`gates.rs:1176-1183`). « no re-hardcode » OK, « imports the policy » inexact. Ne touche PAS Day-0 #4. → resserrer (« binds via anti-drift coverage test ; imports CSS_URL_ALLOW ; never re-hardcodes the policy string »). |
| P3 | Exemple « breaks the build » vs test-runtime | `csp_contract.rs:18-21` + `factory_csp_contract.rs:12-14` ; drift de valeur panique au runtime nextest, seul un drift d'API casse le compile. Resserrer en « la build/le test échoue ». |
| P3 | Exemple : vec ordre-sensible plus strict que prose « set » | `csp_contract.rs:63-78/97-108` ; garde STRICTE acceptable, cohérente avec unit test `csp.rs:82-95`. Aucun changement. |
| P3 | Gate : pack-scope `animejs` codé en dur (latent daisyui) | `check-factory-docs.sh` check (5) ; alternation `(PRIMITIVES\|README)` seule, README daisyui non distingué. Non déclenché. Future-proofing. |
| P3 | Gate : parse symbole premier-vs-dernier `:` | `check-factory-docs.sh` source-ref (`${ref#*:}`) vs required-anchor (`sed s/.*://`) ; divergent pour `path:Mod::Sym`. 0 `::` rank-1 aujourd'hui. Cosmétique. |
| P3 | Gate : `wc -l` off-by-one (fichier sans newline final) | `check-factory-docs.sh` check (5) ; direction conservatrice (sur-stricte, jamais faux PASS). Note. |
| P3 | CI : BusyBox-safety vérifiée sous GNU grep seulement | `check-factory-docs.sh` ; constructs tous BusyBox-supportés, parité avec les frères éprouvés sur `bash:5`. Risque résiduel faible. |
| P3 | Honnêteté : honesty-gate sans BAN négatif explicite | `check-factory-docs.sh:222-227` ; protection indirecte via PROVISIONAL+caveat. Docs propres aujourd'hui. Cohérent gate sharding. |
| P3 | Honnêteté : clause-2 du caveat non grep-ée directement | `check-factory-docs.sh` ; « connaissance consommée, jamais autoritaire » non asserted en tant que telle (sens couvert par `0 verdict PASS`). |
| P3 | Honnêteté : « confirmé au gate dual-platform » devance le run Docker | `sprint79_verification.md:289-293` ; Windows + doc-lints OK, Docker = gate pré-push (`sprint80_audit_plan.md:415`). Claim forward à ne pas figer. |
| P3 | Scope : span A→I > estimation kickoff A→G | `sprint79_kickoff.md:197` ; COMPLIANT (README §4 ne plafonne pas ; closure canonisée Phase B `b27079c`). Traçabilité. |
| P3 | Scope : Tutoriel Diátaxis différé, documenté | `README.md` diff 760-766 ; rationale (in-vivo Not evidenced), pointe vers exemple runnable. Décision honnête. |
| P3 | Wrap-up : anime.js « (v4.5) » vs « 4.5.0 » | `HOW_TO_WIRE.md:32` vs README L18/REFERENCE L73/SPRINT_LOG/fiche. Aligner sur `4.5.0`. |
| P3 | Wrap-up : verification §5 omet Track Testabilité/TEST-ISOLATION-SBFB-HOME | `sprint79_verification.md` §5 vs `sprint80_audit_plan.md` §3 ; pas une incohérence (audit_plan = registre canonique sur-ensemble). Optionnel : renvoi §5. |

**Total** : 0 P0, 0 P1, **2 P2**, 13 P3. Faux positif retiré : « bug object-src 'self' » (glitch de snapshot, scellé).

---

## Verdict

**PASS-PENDING.**

Les 7 dimensions rendent PASS ; la passe adversariale confirme PASS-PENDING (aucun nouveau trou, vacuité gate réfutée, faux-vert CI réfuté, bump wire caché absent). Le sprint satisfait son gate de testabilité §4 (T1 BLOQUANT + T2 JSON, +3 Rust réels 1991→1994) et tous les invariants (0 bump wire, 0 dep, scellage Factory, source CSP unique, connaissance non-autoritaire).

Justification du verdict :
- **Pas de FAIL** : 0 P0, 0 P1.
- **PASS-PENDING plutôt que CONCERN** : les 2 P2 sont des imprécisions de **wording** doc (aucun impact code/comportement, symboles grep-résolvent, gate passe légitimement) que l'adversarial qualifie explicitement de non-bloquantes pour le DONE. La review est propre ; il manque uniquement le gate Codex.
- **PASS-PENDING n'est PAS un verdict final committable.**

**Action recommandée avant le gate Codex / clôture** : resserrer **P2-1** (`WIRING_SPEC.md:104-111` — scoper la phrase CONTEXT-PACK à « surface the animejs manifest ; daisyui pack is a repo file hashed by its manifest test but not yet in the context-pack array at this revision ») et **P2-2** (`WIRING_SPEC.md:80` + `llms.txt:30` — « binds to `BLOB_SERVE_CSP`/`none_directives` via an anti-drift coverage test ; imports `CSS_URL_ALLOW` ; never re-hardcodes the policy string »), OU les acter explicitement en dette-de-précision PO. Ne pas les droper silencieusement : ils touchent la clause de câblage la plus load-bearing du sprint.

Pour Codex : l'artefact « object-src 'self' » est un faux réveil scellé — `csp.rs:33` = `object-src 'none'`, 3/3 tests `factory_csp_contract` verts. Ne pas rouvrir.

---

## Corrections post-review appliquées (avant Codex)

Les 2 P2 et plusieurs P3 ont été corrigés in-phase (precision/honnêteté = cœur d'une couche docs-contrat) ; gate + test re-verts après corrections :

- **P2-1 (FIXED)** — `WIRING_SPEC.md` §CONTEXT-PACK : reformulé pour dire que `AUTHORING_KNOWLEDGE_MANIFESTS` surface le **MANIFEST animejs seul à cette révision** ; le pack daisyUI vit dans le tree + hashé par son test mais **pas encore émis** dans `authoring_knowledge`.
- **P2-2 (FIXED)** — `WIRING_SPEC.md` §GATE + `llms.txt:30` : « imports the policy » → précis : le scanner runtime importe `CSS_URL_ALLOW` + table `CSP_RULES` écrite à la main ; la couverture policy est garantie par un **test anti-drift `#[cfg(test)]`** (pas un import runtime) ; « not re-hardcoded » conservé (string non dupliquée).
- **P3 (FIXED)** — `HOW_TO_WIRE.md` « (v4.5) » → « (4.5.0) » ; exemple + test « breaks the build » → « value drift fails the test / API drift fails the build » ; `WIRING_SPEC.md` « → red build » → « value drift fails the test, API drift fails the build » ; `sprint79_verification.md` « confirmé au gate dual-platform » → « Windows confirmé ; Docker = re-run pré-push ».
- **P3 (HARDENING)** — `check-factory-docs.sh` : ajout grep **clause-2 du caveat cardinal** (`jamais autoritaire`) sur FR docs + WIRING_SPEC + llms.txt (la clause-2 ne pouvait dériver sans alerte) + note explicite sur l'hypothèse `PACK_DIR=animejs` du volet (5).
- **P3 (DOCUMENTÉS, non corrigés)** — négatif BAN `shipped/production` : NON implémenté délibérément (un grep BAN déterministe est fragile — les docs portent des méta-phrases « ne doit pas prétendre … en production » ; protection indirecte via PROVISIONAL + caveat conservée) ; parse first-vs-last-colon (latent, 0 `::` dans les tokens rank-1, miroir exact de `check-sharding-docs.sh`) ; `wc -l` sous-compte sans newline final (direction conservatrice, jamais de faux PASS) ; BusyBox-safety établie par parité avec les 2 gates frères éprouvés.

Re-vérif après corrections : `check-factory-docs.sh` clean ; `factory_csp_contract` 3/3 ; `cargo fmt --all --check` 0. Verdict reste **PASS-PENDING** jusqu'au gate Codex.

---

## Verdict: PASS

Codex GPT 5.5 (cross-check externe, output brut `sprint79_phase_i_codex_review.md`) : **7/7 livrables CONFIRMÉ, 0 GAP, 0 PARTIEL**. Review Workflow PASS-PENDING + corrections P2/P3 in-phase + Codex CLEAN → **PASS**.

## Codex reconciliation

- Codex a relancé INDÉPENDAMMENT `check-factory-docs.sh`, `check-sharding-docs.sh`, `cargo test -p nexus-core-rs --test factory_csp_contract --locked` (3/3) + le test de non-délégation CSP — tous OK.
- 7/7 CONFIRMÉ : (1) Diataxis FR + REFERENCE EN ; (2) llms.txt factory + section racine ; (3) WIRING_SPEC source-anchored (refs obligatoires grep-résolvent : `none_directives` csp.rs:63, `CSS_URL_ALLOW` csp.rs:48, `PROMPT_KINDS` process.rs:7, `app-authoring` process.rs:22, `authoring_knowledge` operator_server.rs:368, `handle_context_pack` operator_server.rs:375, `TemplateConfig` template_engine.rs:218 ; **0 ancre morte `blob_serve.rs:286`**) ; (4) exemple include! drift-guard (asserts utiles, 3/3) ; (5) gate non-vacu (échoue sur fichier absent / source-ref morte / symbole manquant / ancre obligatoire absente / marqueur honnêteté absent / ligne hors-bornes) + câblé 3 surfaces ; (6) check-sharding non-régression après reword racine ; (7) wrap-up honnête (PROVISIONAL/Not evidenced pour in-vivo + efficacité générative, 1991→1994 plausible).
- **0 GAP P0/P1/P2/P3** → aucune correction post-Codex. Les 2 P2 + P3 de la review Workflow étaient déjà corrigés AVANT Codex (cf. section « Corrections post-review »).
- Suites finales : Win nextest **1994/1994 0-skip** + fmt/clippy/doctest/release ; Docker `sbfb-ci` rust:1.94 **fmt 0** + `factory_csp_contract` 3/3 ; Docker `bash:5` BusyBox 3 doc-lints clean (BusyBox-safety du gate confirmée). **Gate dual-platform satisfait** (fmt 0 sous les 2 toolchains). Full networked nextest --workspace en Docker-on-Windows = env-bloqué (standing memory), couvert par Win 1994 + CI Linux.
