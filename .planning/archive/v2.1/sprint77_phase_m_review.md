# Sprint 77 Phase M — Review

## Verdict: PASS

> Review Workflow OK **et** Codex GPT 5.5 réconcilié (3 rounds → CLEAN).
> Le CONCERN initial (1 P1 + 2 P2 + P3 anchors) est **entièrement résolu** ;
> les findings Codex (1 GAP + 1 PARTIEL sur 3 rounds) sont **corrigés** ;
> round-3 = **6 CONFIRME / 0 GAP / 0 PARTIEL**. Voir `## Codex reconciliation`.

---

## Résolution du CONCERN initial

Le premier passage (verdict **CONCERN**) avait relevé exactement **1 P1 + 2 P2 + des P3**. Statut après fixes appliqués en working-tree (non commité, lu sur disque) et vérifiés contre la vérité-terrain :

### P1 — EXPLANATION.md : attribution contiguïté vs couverture — **CORRIGÉ**

**Avant (faux)** : le texte affirmait que `is_pipeline_contiguous` vérifie la couverture de `[0, L)`.

**Après (correct)** — `docs/sharding/EXPLANATION.md:34-43` :
- « `ShardPlan::is_pipeline_contiguous` vérifie seulement la **contiguïté structurelle** : chaque bloc est non vide et commence exactement là où le précédent finit, sans trou ni recouvrement. »
- « Elle **ne vérifie pas** que le plan couvre tout le modèle — un plan peut légitimement décrire un sous-intervalle. »
- « La **couverture exacte de `[0, L)`** … est validée séparément par le scheduler au moment du placement (`covers_full_model`, placement.rs). »

**Vérité-terrain confirmée** :
- `crates/nexus-core-rs/src/shard_plan.rs:212-226` — le corps retourne `false` sur bloc vide/inversé (`layer_start >= layer_end`, l.215) et sur trou/recouvrement (`a.layer_start != end`, l.219) ; **aucune** assertion `layer 0` / `total_layers`. Le doc-comment l.206-211 dit explicitement « This does NOT require the first block to start at layer 0 … The stateful "covers exactly `[0..L)`" check lives in the scheduler ».
- `crates/nexus-coordinator-rs/src/placement.rs:299-307` — `covers_full_model` appelle `is_pipeline_contiguous()` (l.300) **PUIS** asserte `first.layer_start == 0 && last.layer_end == total_layers` (l.304). C'est exactement le mécanisme décrit.
- **Lien ajouté** `../../crates/nexus-coordinator-rs/src/placement.rs` — résout depuis `docs/sharding/` (test `[ -e ]` OK). Le link-check du gate mord si cassé.

La fausse affirmation précédente est **intégralement retirée**. P1 fermé.

### P2 — REFERENCE.md : pas de `TRUST_TIER_SUSPECT` — **CORRIGÉ**

`docs/sharding/REFERENCE.md:120-121` : « There is no `TRUST_TIER_SUSPECT` constant — SUSPECT is the `else` branch below `TRUST_TIER_STANDARD`. »

**Vérité-terrain** `crates/nexus-core-rs/src/verification.rs:280-305` : seuls `TRUST_TIER_TRUSTED = 80` (l.280) et `TRUST_TIER_STANDARD = 50` (l.282) existent comme constantes de tier ; le taux SUSPECT est atteint via le `else` nu (l.303-304). La distinction est nette : `SPOT_CHECK_RATE_SUSPECT_BP` (constante de **taux**, l.291) existe bien, mais `TRUST_TIER_SUSPECT` (constante de **tier**) non. Table 100/500/2000 bp et tiers 80/50 cohérents avec REFERENCE.md:109-112. P2 fermé.

### P2 — script EN_WORDS : « Please » ajouté — **CORRIGÉ**

`scripts/check-sharding-docs.sh:104` contient désormais `Please` dans `EN_WORDS`, miroir de l'intention source `web/scripts/scan-en-strings.sh:26` (`Please\b`). Vérification empirique : grep de la liste complète sur les 3 docs FR (README/EXPLANATION/HOW_TO_WIRE) → **zéro match**. Gate clean. P2 fermé.

### P3 — script anchors : §P64–§P69 + §P39 — **CORRIGÉ**

`scripts/check-sharding-docs.sh:76-82` asserte chaque ancre individuellement via `anchor_present`. Présence vérifiée dans `docs/rust/PATTERNS.md` (counts grep) : §P64→12, §P65→6, §P66→1, §P67→3, §P68→3, §P69→3, §P39→1 — **toutes ≥1**. Le gate ne vire pas faussement au rouge. P3 fermé.

**Gate ré-exécuté** : `bash scripts/check-sharding-docs.sh` → `clean (links + anchors + honesty + french-body)`, **exit 0**.

---

## Re-review — 3 dimensions

### RV1 — Delta grounding (PASS)
Les 4 fixes vérifiés CORRECTS contre la vérité-terrain ; aucune erreur nouvelle introduite ; gate vert (exit 0). Le P1 reflète fidèlement shard_plan.rs:201-226 + placement.rs:299-307 ; le lien placement.rs résout et pointe le bon fichier ; `covers_full_model` est bien live (appelé dans le chemin de placement). Le P2 SUSPECT correspond verbatim à verification.rs:280-305. « Please » présent (l.104), zéro faux-positif sur les 3 docs FR. Les 7 ancres PATTERNS présentes.
- **P3 (gardé)** : asymétrie de nommage — EXPLANATION dit « scheduler au moment du placement », le doc-comment Rust dit « scheduler (Phase D) ». Les deux désignent le même composant (placement.rs EST le scheduler). Aucune inexactitude. Aucune action.

### RV2 — Fresh red-team (PASS)
Effort adversarial réel : les 3 cibles à plus forte valeur ont survécu au code.
- Paragraphe P1 (contiguïté vs couverture) — **réfuté comme défaut** : correct contre shard_plan.rs:201-226 et placement.rs:299-307.
- Note P2 SUSPECT — **réfutée** : correspond à verification.rs:280-306.
- Chasse aux contradictions cross-doc / threat-model — **réfutée** : les 4 docs disent « carry S78 » (jamais « Phase K ») pour l'orchestrateur, conforme à la directive ; les commentaires source http.rs/THREAT_MODEL gardent le cadrage « Phase K » mais ne font pas partie des 4 docs en revue.
- Claims porteuses re-confirmées contre le code : route stub `{found:false,session:null}` (http.rs), whitelist ShardSessionView 2 champs, sbfb-bridge.js 3 méthodes seules, VRF≠ECVRF, SENTINEL O(1) EMA entière, consts TOPLOC 128/38/10/8, caps wire (256 MiB frame, etc.), 5 familles DOMAIN_*_V1, N2 clique pairwise non-transitive, quorum exact-match inchangé, kudos non-monétaire, admission≠confidentialité dans les 3 docs FR.
- **P3 (gardés)** : (a) README.md:71-72 / REFERENCE.md:16-17 — label « schémas JSON générés » pointant vers le générateur `schemas/shard.rs` plutôt que les artefacts `*.schema.json` (le lien résout, c'est juste un nom imprécis) ; (b) note de phrasing délégation purement cosmétique. Ni l'un ni l'autre n'est faux.

### RV3 — Gate integrity (PASS)
Les 3 checks RUN et MORDENT — 5 injections adversariales ont chacune renvoyé exit 1 (lien placement.rs cassé, ancre §P manquante, PROVISIONAL retiré, « Please » injecté dans doc FR, « S78-pending tuning » retiré), gate exit 0 après restauration. Pas de faux-vert (le gate est clean ET échoue sur chaque classe de violation — pas de tautologie). BusyBox-safe : seuls `-oE/-qF/-qE/-nE` utilisés ; `-P`/`--include`/`\b` n'apparaissent que dans le commentaire l.7 ; `\b` et `\s*` retirés de EN_WORDS. shellcheck `--severity=warning` CLEAN. 3 surfaces CI câblées BLOQUANTES : `.github/workflows/ci.yml` [14], `.woodpecker/ci-linux.yml` `sharding-docs-check`, `scripts/verify.sh` step 19 — chacune un `bash scripts/check-sharding-docs.sh` non-advisory.
- **P3 (gardé)** : le commentaire l.102-103 qualifie EN_WORDS de « mirror » de scan-en-strings.sh alors que c'est un miroir délibérément relâché (BusyBox-safe : `\b` et `\s*` retirés). Reformulation optionnelle « BusyBox-safe approximation of » ; aucun changement de code requis.

---

## P2/P3 documentés (acceptés)

| Sév | Item | Statut |
|---|---|---|
| P3 | EXPLANATION « scheduler/placement » vs doc-comment « Phase D » (deux noms, même composant) | Accepté — aucune inexactitude |
| P3 | README/REFERENCE label « schémas JSON générés » → lien vers générateur `schemas/shard.rs` (résout) | Accepté — imprécision de nom, non bloquant |
| P3 | Phrasing délégation premier/dernier bloc (caller responsibility, fidèle) | Accepté — listé pour complétude |
| P3 | Commentaire script « mirror » vs approximation BusyBox-safe relâchée | Accepté — reformulation optionnelle |

Aucun de ces points n'est faux ni bloquant. Aucun `must_fix`.

**Polish post-synthèse (P3, zéro-risque) appliqué** : 2 des 4 P3 ci-dessus ont
été reformulés après le verdict — (a) label README/REFERENCE « schémas JSON
générés » + lien pointé vers le répertoire `crates/nexus-core-rs/src/schemas/`
(au lieu du générateur `shard.rs`) ; (b) commentaire script reformulé en
« BusyBox-safe subset of scan-en-strings.sh ». Gate ré-exécuté **exit 0**,
shellcheck `--severity=warning` **EXIT=0**, lien répertoire résout. Les 2 P3
restants (nommage « placement » vs « Phase D », phrasing délégation) sont
acceptés sans changement.

---

## Rationale de clôture

Le CONCERN initial reposait sur un unique P1 d'attribution (contiguïté structurelle confondue avec couverture `[0, L)`) plus 2 P2 et des P3. Vérification indépendante contre `shard_plan.rs:201-226`, `placement.rs:294-307`, `verification.rs:278-306`, `scan-en-strings.sh:26` et `docs/rust/PATTERNS.md` : les 4 fixes correspondent verbatim au code, aucune régression introduite, et le doc-lint gate s'exécute vert tout en mordant sur chaque classe de violation (anti-tautologie prouvée par 5 injections). La feature reste correctement marquée PROVISIONAL et le caveat cardinal « admission ≠ confidentialité » est présent dans les 3 docs FR. **Le CONCERN est résolu.**

---

## Codex reconciliation

Vérification croisée Codex GPT 5.5 (`codex exec`, modèle externe, anti-auto-validation), 3 rounds. L'artefact brut est `sprint77_phase_m_codex_review.md` (output `codex exec -o`, non réécrit). Rounds 1-2 préservés dans le scratchpad de session.

- **Round 1** — 4 CONFIRME / **1 GAP** / 1 PARTIEL.
  - **GAP (HOW_TO_WIRE.md)** : « le bridge a exactement trois méthodes (task_submit, storage_get, storage_set) » — **faux** contre `web/public/sbfb-bridge.js` (qui expose aussi task_result, storage_list, storage_delete, pii_redact, browse_list…). Le fait porteur « aucune méthode shard » restait vrai. **Corrigé** : reformulé en « whitelist de méthodes (soumission/résultat de tâche, stockage, rédaction PII) — aucune ne touche le sharding », sans compte fragile. Vérité-terrain re-vérifiée (grep `shard` = 0, méthodes réelles lues `:170-388`).
  - **PARTIEL (script)** : bash-only (arrays, process substitution). **Non-issue** : invoqué `bash` partout, image Woodpecker `bash:5`, miroir exact de `check-spdx.sh` ; la « BusyBox-safety » requise portait sur grep (`-P`/`--include`, respectée). Round-2 l'a re-classé CONFIRME.
- **Round 2** — 5 CONFIRME / 0 GAP / **1 PARTIEL**.
  - **PARTIEL (REFERENCE.md)** : la ligne `ComputeGroup` de la table omettait `version: u16` (obligatoire `compute_group.rs:92`, listé spec). **Corrigé** : champ ajouté (les 3 types signés listent désormais `version` uniformément). Vérité-terrain re-vérifiée (`compute_group.rs:88-92`, premier champ, pas de serde-default).
- **Round 3** (post-fix) — **6 CONFIRME / 0 GAP / 0 PARTIEL**. CLEAN.

Après chaque correction : gate `check-sharding-docs.sh` ré-exécuté **exit 0**, fixtures de test Codex (`.preview-stage/*doclint*`, `.tmp-shard-doc-gate-audit-*`) nettoyées. Aucune suite Rust/Vitest concernée (phase doc-only, 0 code touché). Séquence stricte respectée : review PASS-PENDING → Codex (3 rounds) → réconciliation PASS → commit.
