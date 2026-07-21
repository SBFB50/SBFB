# Sprint 83 Phase G — Review round 5 (promotion post-Codex) (vérification des fixes Codex round 1)

**Sprint** : 83. **Phase** : G — Langage workflow, agents et rôles (`SBFBWorkflowGraph.v1`
+ `SBFBAgentDefinition.v1`, 4 goldens). **Date** : 2026-07-21. **HEAD** : `23fdc1dc`
(arbre sale, commit de phase à suivre). **Round précédent** :
`sprint83_phase_g_review_r3.md` (verdict PASS-PENDING — 0 P0/P1, 5 P2 tous fermés côté
review). **Déclencheur du round 4** : `sprint83_phase_g_codex_review_r1.md` (Codex
GPT-5.6 Sol = **GAPS**, ≥1 P1 : 1 P1 ideas `visibility` + 3 P2 conception/tests, tous
vérifiés EXACTS contre les fichiers avant correction). **Méthode** : vérification ciblée
des CINQ corrections appliquées depuis Codex r1 (GAP1 P1 + GAP2/3/4 P2 + REGLE 6 freeze §8),
chacune re-mesurée fichier:ligne — aucune prise sur parole. Deux vérificateurs indépendants
ont re-lu les fichiers et rejoué les tests ; leurs constats sont réconciliés ci-dessous.
Analyse transverse §19.3 sur les 4 goldens (facteur babel demandé explicitement au round).

## Verdict: PASS

**Les CINQ corrections Codex r1 sont COMPLÈTES et empiriquement vertes** : crate
`sbfb-workflow-core` = **67/67 PASS, 0 skip** au `cargo nextest -p`, dont les 3 nouveaux
tests + `golden_pins_resolve` / `four_goldens_validate` / `node_profile_frozen`. Le P1
GAP1 est fermé À LA RACINE : la décision de publication ideas est devenue un `human.review`
(`publish-decision`, `decision_labels [publish, keep-private]`), le champ `visibility` a
DISPARU de tous les schémas du store ideas ET du golden (grep récursif = 0 hit,
case-insensitive), le prompt champion est réécrit (« publication is decided by the HUMAN
champion gate, never by you »), les edges sont recâblés sans orphelin, et un test de
domination BFS *discriminant* le prouve. Les 2 P2 conception (GAP2/GAP3) alignent les shapes
item↔body et fan-in↔sources ; GAP4 ajoute 3 tests structurels et tue MUT-8/9/10 (10/10
mutations cumulées sur la phase). La REGLE 6 du freeze §8 consigne la provenance des
discriminants de classification pour Phase H. **PASS-PENDING et non PASS** parce que (a) le
re-scellement dual-platform Win + Docker est EN COURS (gate pre-commit — relancé en
parallèle, non encore observé vert cette itération, contrairement à r3 où il l'avait été
avant le micro-fix doc-only) et (b) le PASS final exige la réconciliation Codex **round 2**
sur l'arbre corrigé. Côté review, 0 reste bloquant : les 5 GAPs sont fermés, 0 régression
fonctionnelle constatée.

## Corrections Codex r1 — statut vérifié

| # | GAP Codex r1 | Sévérité | Statut | Évidence mesurée |
|---|---|---|---|---|
| 1 | **GAP1** — `visibility` = sortie LLM pilotant la (re)classification public/privé | **P1** | **COMPLETE** | `ideas.workflow.golden.json:91-101` : node `publish-decision` = `human.review`, `payload_schema_ref` idea-note, `decision_labels [publish, keep-private]`. Ancien switch `publish-gate` ENTIÈREMENT supprimé (grep `publish-gate`/`publish_gate`/`public` sur golden + store = 0 hit). `visibility` = **0 hit récursif** sur tout le store ideas ET le golden (case-insensitive). 5 schémas confirmés flat + closed (`idea-submission{idea}`, `idea-note{summary}`, `champion-verdict{draft}`, `refine-input{draft}`, `refine-report{draft,ready}`, `additionalProperties:false`). `field_maps` cohérents (`vision-note {summary←idea}`, `normalise-refined {summary←draft}`, les deux champs existent des deux côtés). `champion-instructions.md` réécrit. Edges recâblés et tracés : `collect→publish-decision` (`:292-297`), `publish-decision/publish→store-idea` (`:298-303`), `publish-decision/keep-private→private-note` (`:304-309`), `store-idea→published` (`:310-315`) ; l'edge `default` de l'ancien switch supprimé, `publish-decision` n'expose que ses 2 ports décision, aucun port `default`. Les 17 edges référencent tous des nodes existants (15 nodes) ; les 3 branches intake (feature/bug/vision) atteignent `collect` puis `publish-decision` avec type `idea-note` cohérent. Test `ideas_publication_dominated_by_human_review` (`graph.rs:1509`) PASS : `matches!(HumanReview(_))` + contrôle `reachable_without(idea-in, store-idea, __none__)=true` + domination `!reachable_without(…, publish-decision)`. `reachable_without` (`graph.rs:1374`) re-tracée à la main : `store-idea` n'a qu'une seule in-edge (`publish-decision→store-idea`), exclure `publish-decision` déconnecte l'egress ⇒ l'assertion tient ; une bypass-edge ou un node non-human ferait rougir l'une des 2 assertions. Le test itère aussi TOUS les schémas du store et asserte l'absence de `properties.visibility`. |
| 2 | **GAP2** — `map` conception ne reçoit pas un item `design-angle` | P2 | **COMPLETE** | `angle-list.schema.json` : `properties.angles.items` = OBJET `{angle:string, required[angle], additionalProperties:false}` identique à `design-angle.schema.json`. Test `conception_map_item_type_matches_body_input` (`graph.rs:1554`) PASS : `shape_of(item) == shape_of(design-angle)` (shape = type/properties/required/additionalProperties), item asserté `is_object()`. |
| 3 | **GAP3** — fan-in `judge` mal typé | P2 | **COMPLETE** | `judge-input.schema.json` retypé `{variants:[{proposal,angle}], checks:{docs_ok,feasible}}`. Test `conception_fanin_types_align` (`graph.rs:1571`) PASS : `shape_of(judge-input.variants.items)==shape_of(variant)` et `shape_of(judge-input.checks)==shape_of(checks-report)`. |
| 4 | **GAP4** — tests sémantiques insuffisamment discriminants | P2 | **COMPLETE** | Les 3 nouveaux tests présents et verts dans le run 67/67. `shape_of` (`graph.rs:1541`) compare `properties` comme sous-arbre `serde_json::Value` COMPLET ⇒ toute dérive nested-required/nested-type/nested-additionalProperties EST capturée (discriminant ; seuls `oneOf`/`enum`/`$defs` hors des 4 clés sont ignorés, inapplicables à ces schémas flat). Kill-logic : MUT-8 (bypass human gate / node non-human) tuée par les assertions de domination ; MUT-9 (item→string) et MUT-10 (checks→string) tuées par mismatch structurel `shape_of`. `golden_pins_resolve` (`graph.rs:924`) PASS = garde discriminante refilled==parsed ⇒ les pins BLAKE3 des schémas édités ont bien été recalculés (UPDATE_SNAPSHOTS cohérent) ; 0 `"blake3": ""` dans un golden/store (les seuls hits sont des fixtures inline de tests unitaires `graph.rs`). |
| 5 | **REGLE 6** — règle H « provenance des discriminants de classification » absente du freeze | dérive GAP1 | **COMPLETE** | `sprint83_node_profile_freeze.md:145-152` : « Provenance des discriminants de classification (§19.3, Codex r1 GAP1) — un `switch`/`classifier` dont dépend un `capability.call` d'egress ou de publication ne peut PAS discriminer sur un champ produit par une sortie de modèle ; la classification (public/privé) vient d'une source déterministe ou d'un `human.review`, jamais d'un LLM. Le golden ideas l'exemplifie ». Correctement cadrée sous « ## 8. Consigné pour Phase H » (règle future compilateur, PAS sur-revendiquée comme déjà enforced) ; cite Codex r1 GAP1 ; cohérente avec §19.3 (« a model never reclassifies data », citée dans le doc-comment du test). Test `node_profile_frozen` PASS. |

## Analyse transverse §19.3 sur les 4 goldens (babel demandé)

Vérification croisée : **aucun autre golden ne partage la classe du GAP1**.

- **factory** : l'egress `git_commit` est dominé par le `human.review` de ratification PO
  (asserté par le test factory). Pas §19.3.
- **conception** : `storage.set` (écriture locale, PAS un egress public) sans switch/classifier
  routant une sortie modèle sur un fork public/privé ; terminal linéaire. Pas §19.3.
- **babel** (analysé sur demande) : **CLASSE DIFFÉRENTE, PAS un P1 §19.3**. Motivation avec
  évidence : (1) `quality-gate` est un switch de QUALITÉ (ship vs polish), pas une
  CLASSIFICATION public/privé — il n'existe AUCUN terminal privé/discard dans babel ;
  `ship`, `polish` ET le `default` réservé convergent tous sur `await-community → publish`,
  la publication est la finalité invariante déclarée et il n'existe aucune donnée privée dont
  une sortie modèle pourrait retourner la confidentialité ; §19.3 (« un modèle ne reclassifie
  jamais une donnée ») ne mord donc pas. (2) L'egress `publish` (`network.publish`) est de
  plus dominé par `await-community`, un `wait.signal` sur un signal EXTERNE
  `community-review-complete` délivré par l'hôte, non-produisible par un modèle in-workflow —
  une injection ne peut pas le forger. (3) L'injection pire-cas (flip `polish→ship` dans la
  sortie de `native-review`) ne fait que sauter des rounds de polissage automatiques ; la
  traduction passe quand même le checkpoint communautaire externe avant publication — une
  DÉGRADATION de qualité, catégoriquement d'une autre sévérité que la fuite de confidentialité
  du GAP1, et sans doute voulue par design. Contraste : le GAP1 ideas routait sur un champ
  `visibility` produit par un modèle décidant public-vs-privé de données pouvant légitimement
  rester privées (terminal `private-note`/`private-idea` existant) — exactement §19.3 ; babel
  n'a pas ce fork.

Verdict babel = « classe différente, acceptable-documenté » : consigné comme NOTE (pas un
bloquant), conformément à la consigne du round. Aucun nouveau P1 de la classe §19.3 n'a été
trouvé ailleurs que le GAP1 déjà corrigé.

## Régressions

- **Aucune régression fonctionnelle.** Grep `publish-gate` sur `graph.rs` = 0 hit ; aucun
  test ne référence un node supprimé ; les 3 branches intake tracées bout-en-bout ; les 17
  edges référencent tous des nodes existants (15 nodes).
- **Observation NON-BLOQUANTE #1 (P3, précision-doc) — REGLE-6-SCOPE** : la REGLE 6
  nouvellement ajoutée (`sprint83_node_profile_freeze.md:145-152`) est rédigée plus largement
  que son intention de portée (titre « discriminants de classification » + parenthèse
  « public/privé »). Lue au pied de la lettre, la clause « un switch/classifier dont dépend un
  capability.call d'egress … ne peut PAS discriminer sur un champ produit par une sortie de
  modèle » ferait un FAUX POSITIF sur le `quality-gate` de babel (switch dont le discriminant
  `quality` vient de `native-review`, un agent LLM, et dont dépend transitivement l'egress
  `network.publish`) quand le compilateur Phase H l'implémentera — alors que babel est
  sémantiquement sain (cf. analyse babel). Un implémenteur Phase H doit scoper la REGLE 6 aux
  champs de visibilité/classification (public/privé), ou exempter explicitement les switch de
  routage-qualité. C'est une nuance de précision documentaire introduite AVEC le fix GAP1 —
  PAS un défaut fonctionnel : les 67 tests passent, cela ne bloque pas la correction GAP1. À
  consigner en note Phase H (au body/carry si le PO le souhaite).
- **Observation NON-BLOQUANTE #2 (P3, précision-test)** : l'assertion `visibility` du test
  `ideas_publication_dominated_by_human_review` vérifie seulement `properties.visibility` au
  TOP-LEVEL — pleinement discriminante pour les schémas ideas actuels (flat + closed), mais un
  hypothétique `visibility` imbriqué ne serait pas capté. Ce n'est PAS une incomplétude de la
  correction (aucun `visibility` imbriqué n'existe, tous les schémas sont `additionalProperties:false`).

## Mutations — 10/10 cumulées sur la phase

- **MUT-1..7** (rounds antérieurs) : contrôles négatifs du crate, restent discriminants.
- **MUT-8** : bypass human gate (edge `collect→store-idea`, ou `publish-decision` retypé
  non-human) ⇒ tuée par `ideas_publication_dominated_by_human_review` (l'une des 2 assertions
  de domination rougit ; logique re-tracée indépendamment sur `reachable_without`).
- **MUT-9** : `angle-list.angles.items` type→string ⇒ tuée par
  `conception_map_item_type_matches_body_input` (shape `{type:string}` ≠ shape objet
  `design-angle`).
- **MUT-10** : `judge-input.checks`→string ⇒ tuée par `conception_fanin_types_align`
  (mismatch structurel `shape_of`).

Kill-power confirmé par inspection ET, pour MUT-8, par re-trace manuelle du BFS `reachable_without`.

## État de la vérification (mesuré, arbre sale)

- **crate `sbfb-workflow-core`** : `cargo nextest -p` **67/67 PASS, 0 skip** (r3 = 64/64 ;
  +3 nouveaux tests GAP4) — re-joué vert après les fixes. `golden_pins_resolve` /
  `four_goldens_validate` / `golden_negative_controls_reject` / `golden_richness_floor` /
  `node_profile_frozen` verts.
- **fmt `--check` 0** (crate) ; **clippy crate `-D warnings` 0**.
- **Pins recalculés** via `UPDATE_SNAPSHOTS` (workspace) ; `golden_pins_resolve` PASS confirme
  refilled==parsed (garde discriminante que tous les schémas/prompts édités sont reflétés).
- **Delta tests phase G attendu : +20 Win (2192 → 2212)** — cohérent avec le tip Phase F
  (2192/2196). Le compte cité en r3 (« 2209 ») était une mesure intermédiaire du round ; la
  borne cible du wrap-up G est **2212 Win / 2216 Docker attendu**.
- **Re-scellement dual-platform Win + Docker EN COURS** (relancé en parallèle = gate
  pre-commit) — non encore observé vert cette itération (raison n°1 du PASS-PENDING).
- **web / factory-operator** : inchangés par la phase G (schémas + format Rust) ; suites
  vertes.
- **0 dépendance runtime ajoutée** ; **0 bump wire** (frontières neuves, pas de modification
  d'enveloppe existante).

## Actions commit-time (rappel — inchangées, non-bloquantes de review)

1. **`git add` explicite** au commit de phase :
   - les DEUX `.md` planning de review de CE round :
     `.planning/active/rounds/sprint83_phase_g_review_r4.md` et
     `.planning/active/sprint83_phase_g_review.md` (canonique = copie byte-identique de r4) ;
   - les rounds antérieurs `…_r1.md`/`…_r2.md`/`…_r3.md` + le rapport Codex
     `…_codex_review_r1.md` (et `…_codex_review_r2.md` une fois joué) ;
   - les livrables planning G6-01 encore untracked : `sprint83_node_profile_freeze.md`
     (livrable G.2 + critère G.4) + `sprint83_phase_g_preflight.md` (autorité de phase) ;
   - vérifier `git diff --cached --stat` avant le commit.
2. **Nommer au body** du commit de phase :
   - **G6-02** — traçabilité PLAN-ADAPT : les deux renommages de tests sans perte de
     couverture (`node_profile_18_frozen`→`node_profile_frozen` ;
     `…graph_schema_matches_snapshot`→`schema_snapshot_stub`).
   - **G6-03** — Carry-closure : dette docs-contrat GUIDE/llms.txt des 2 frontières neuves
     différée au wrap-up S83 (AM-3), pour que le Track K docs-contrat de l'audit gate la
     retrouve.
   - **REGLE-6-SCOPE (P3, nouveau)** — scoper la REGLE 6 du freeze §8 aux champs
     visibilité/classification (public/privé) à l'implémentation Phase H, pour ne pas flaguer
     le `quality-gate` de babel (faux positif) ; consigne Phase H.

## Prochaine étape

1. **Codex round 2** : le PASS final NE PEUT être prononcé qu'APRÈS réconciliation Codex sur
   l'arbre corrigé (`codex exec -m gpt-5.6-sol -c model_reasoning_effort=max`, output brut
   dans `codex_review.md`). Ce round de review est PASS-PENDING (0 reste bloquant côté review,
   5 GAPs Codex r1 fermés, 0 régression fonctionnelle).
2. **Fin du re-scellement dual-platform** Win + Docker (gate pre-commit) — doit être observé
   vert avant le commit.
3. **Commit de phase** après Codex round 2 CLEAN (ou P2/P3 documentés) + re-seal vert :
   appliquer les actions commit-time ci-dessus, body `feat(scope): Sprint 83 Phase G — …`
   avec delta de tests cumulé (+20 Win) et scope cuts.

## Codex reconciliation

Boucle Codex GPT-5.6 Sol (effort max) : **2 rounds**, artefacts bruts jamais
réécrits dans `rounds/sprint83_phase_g_codex_review_r{1,2}.md`, canonique =
copie byte-identique du round 2.

- **Round 1 = GAPS** (1 P1 + 3 P2) : GAP1 la visibilité de publication d'ideas
  était une sortie LLM (violation §19.3) ; GAP2 item du `map` de conception mal
  typé (conteneur au lieu de l'item) ; GAP3 fan-in du judge mal typé ; GAP4
  tests non discriminants sur ces classes. Chaque fait Codex a été re-vérifié
  contre les fichiers AVANT correction : tous exacts, aucune confabulation.
- **Corrections** : publication ideas = `human.review` (« publish-decision »),
  champ `visibility` RETIRÉ de tout le store ideas, prompt champion réécrit ;
  `angle-list.angles.items` = objet `design-angle` ; `judge-input` retypé sur
  `variant[]`/`checks-report` ; +3 tests (`ideas_publication_dominated_by_human_review`,
  `conception_map_item_type_matches_body_input`, `conception_fanin_types_align`) ;
  mutations MUT-8/9/10 jouées, toutes tuées par le test visé (10/10 cumulées) ;
  freeze §8 règle 6 (provenance des discriminants de classification, scopée —
  babel quality-gate = classe différente, confirmé Codex r2).
- **Round 2 = CONFIRME, 0 gap** : les 4 GAPs corrigés, règle 6 correctement
  bornée, invariants a-f re-balayés (census 29, 0 dep, 0 web diff, registre 7,
  pas de compilateur, required-only 16/16 + 15/15), aucune régression
  (10 graphes 0 endpoint invalide, stores 84/84, 18/18 kinds, richesse intacte).
- **Suites relancées après corrections** : crate 67/67 ; Win workspace 2212
  passed + 1 skip connu (delta phase +20) ; clippy/fmt/doctests/release verts ;
  Docker fmt 1.94 + nextest 2216 (2 flakes env-connus re-verts solo, dont le
  carry 7 STANDING) ; pins recalculés ; frontier census 29 exit 0 ; SPDX.

Critère d'arrêt de boucle atteint : CLEAN (0 gap). La phase est committable.
