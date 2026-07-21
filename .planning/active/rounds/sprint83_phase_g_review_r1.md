# Sprint 83 Phase G — Review round 1 (synthèse Workflow + adversarial)

**Sprint** : 83. **Phase** : G — Langage workflow, agents et rôles (`SBFBWorkflowGraph.v1`
+ `SBFBAgentDefinition.v1`, 4 goldens). **Date** : 2026-07-21. **HEAD** : `23fdc1dc`
(arbre sale, commit de phase à suivre). **Autorité de phase** :
`sprint83_phase_g_preflight.md` (verdict PLAN-ADAPT).
**Méthode** : review Workflow à 6 dimensions (conformité diff/préflight, sémantique
des tests anti-tautologie, qualité des 4 goldens, sécurité profonde Plan-Then-Execute,
research grounding/honnêteté, process/livrables/patterns) + passe adversariale de
vérification indépendante sur les findings susceptibles de bloquer (5 P2 re-mesurés
fichier:ligne, tous CONFIRMED). Re-vérification propre du writer sur les deux
sur-affirmations du freeze (D4-SEC-01, D5-01) et sur le statut untracked des livrables
planning (G6-01).

## Verdict: CONCERN

Aucun P0/P1. Le contrat MACHINE de la phase (schéma snapshot pinné, drift-gate, enum
interne-taggée 18 kinds fail-closed, parse strict, 4 goldens qui valident, gel SET+FORME
daté) est solide et non bloquant. Le CONCERN est porté par **5 P2 confirmés**, dont deux
familles demandent une décision AVANT le commit de phase :

1. **Deux sur-affirmations dans le freeze `sprint83_node_profile_freeze.md`** (D5-01 « v1
   SANS CYCLE par construction / sans analyse de cycles » ; D4-SEC-01 « deux graphes
   différents ne peuvent pas s'effondrer sur le même digest `graph_semantic` ») — ce sont
   des garanties FAUSSES dans un artefact SIGNÉ et daté qui est un livrable dur G.2,
   committé DANS la phase G. Le projet interdit d'expédier une garantie fausse ; à corriger
   avant le commit du freeze.
2. **Deux défauts sémantiques dans les goldons canoniques** (G3-01 branche `feature`
   d'ideas morte de bout en bout ; G3-02 seul exemplaire `map` nourri par le conteneur
   entier au lieu de l'item) — les goldens sont les exemplaires de référence ; fixer
   maintenant vs consigner explicitement à Phase H = arbitrage à trancher.
3. **Livrables planning untracked** (G6-01) — le freeze daté (livrable G.2 + critère
   d'acceptation G.4) et le préflight sont `??` : à `git add` explicitement au commit.

Ces items ne cassent pas l'acceptance machine de G (les 5 P2 sont sans impact sur les
suites de G) mais exigent une décision explicite avant Codex/commit — d'où CONCERN et non
PASS-PENDING.

## État de la vérification (mesuré ce jour, arbre sale)

- **Rust Windows** nextest workspace **2209 passed + 1 skip** connu (delta **+17** :
  6 `agent_definition` + 11 `graph`) ; crate seul 64/64 sous `-p` ET couvert sous
  `--workspace`.
- clippy `--all-targets` -D warnings 0 ; fmt 0 ; doctests (10 suites) 0 ; release build 0.
- 0 diff `Cargo.toml`/`Cargo.lock` (mesuré) ; SPDX 390.
- `check-frontier-contracts` exit 0, **census 29** [2 tagged : `pack.rs`, `shard_plan.rs`]
  — frontière G en PROSE, aucun `// FRONTIER:` machine, aucun `DOMAIN_*`/`*_FORMAT_VERSION`
  neuf.
- **web** complet vert (lint/tsc/unit/coverage/build/size/scan) ; **operator** vert
  (lint/unit/build/size/gates).
- **Mutations 7/7 tuées** (détail en fin de doc).
- **Docker `sbfb-ci` (fmt 1.94 + nextest Linux) = GATE PRE-COMMIT** : doit être vert avant
  le commit de phase (EN COURS au fan-out ; non encore observé vert par ce round).

## Dimension 1 — Conformité diff vs préflight

**Résumé** : conformité PLEINE. Les 5 blocants B1-B5, les 13 adaptations A1-A13 et les 9
interdits §8.3 du préflight sont tous respectés, chacun vérifié fichier:ligne. B2 enum
interne-taggée (`graph.rs:473` `tag="kind"`), 18 renames dottés canoniques, `deny_unknown_fields`
par corps de variante, rejet du champ inconnu prouvé kind-par-kind (`unknown_field_rejected_for_every_kind`),
zéro `untagged`/`alias` hors prose. B3 handoff : `payload_schema_ref` + `context_policy`
REQUIS (jamais `Option`/`default`), discriminé d'`agent.as_tool`. B4 frontière en PROSE
identique au précédent Phase D, gate non déclenché (census 29 confirmé). B5 vacuité fermée
(contrôle négatif discriminant + plancher de richesse + note d'équivalence honnête, sans
dép `jsonschema`). A6/A8/A10 : policies D2 déclarées, zéro secret + id structurellement
unique (`nodes: BTreeMap`), artefact de gel daté et complet.
**Findings retenus** : aucun P0/P1/P2/P3. Une seule NUANCE INFO (dans le scope du préflight,
pas une déviation) : la politique de join `quorum` existe et est testée unitairement
(`join_policy_variants_are_strict`) mais n'est exercée par aucun golden (les 4 utilisent
all/any) — A4 n'exigeait qu'« un golden non-all », satisfait par ideas « any ». Facultatif
hors-exigence : un golden exerçant `quorum(n)` au vertical qui en aura l'usage.

## Dimension 2 — Sémantique des tests (anti-tautologie)

**Résumé** : suite de haute qualité, discipline anti-tautologie forte. Le compte **+17** est
CONFIRMÉ (11 `graph.rs` + 6 `agent_definition.rs`). `node_profile_frozen` mesure le schéma
ÉMIS (`schemars::schema_for!(WorkflowNode)`) contre le miroir `FROZEN_NODE_KINDS` — pas une
2e liste manuelle, SET-check discriminant (jamais `count==18`). `golden_pins_resolve` est
STRICT (panic fail-on-missing + assert fail-on-stale) hors `UPDATE_SNAPSHOTS`.
`reachable_without` est une accessibilité-évitant-un-nœud CORRECTE (`continue` sur l'exclu
AVANT expansion) et le dominator-check factory est réel (topologie linéaire vérifiée dans le
golden). TOUTES les comparaisons golden/snapshot sont en `serde_json::Value` ou `PartialEq`
typé, JAMAIS en string (robuste au piège `preserve_order` : `IndexMap::PartialEq` ordre-indépendant ;
`to_string_pretty` seulement en écriture mode update). Les négatifs `graph.rs` prouvent
d'abord que la base non-mutée parse avant chaque mutation unique.
**Findings retenus** :
- **D2-01 (P3)** — Trois négatifs d'`agent_definition.rs` (`missing_budget_dimension`,
  `missing_context_policy`, `unknown_field_is_rejected`, `:300-327`) mutent `sample_json()`
  puis `expect_err` SANS prouver dans le même test que la base non-mutée parse ; le garde est
  indirect (`sample_agent_parses_and_round_trips`). Si la fixture régressait vers un état
  invalide, les trois passeraient pour la MAUVAISE raison. `graph.rs` est plus rigoureux
  (`golden_negative_controls_reject:958` asserte la base). *Fix* : ajouter un
  `from_value::<SbfbAgentDefinition>(sample_json()).expect("control base parses")` en tête de
  chaque négatif.

## Dimension 3 — Qualité sémantique des 4 goldens

**Résumé** : les 4 goldens sont structurellement solides — chaque `edge.from/to` nomme un
node existant (4 tops + 7 subworkflows) ; tous les `from_port` de switch/classifier/parallel/human.review
correspondent aux labels déclarés + port `default` câblé ; chaque label a ≥1 arête sortante ;
les 3 joins nomment des sources existantes ; chaque agent invoqué/handoff est dans `agent_refs`
et son fichier existe ; couverture 18/18 kinds confirmée ; invoke-budgets ≤ agent-def, inner-nodes
≤ budget subworkflow ; factory réaliste (driver/reviewer/codex-verifier ↔ `FactoryProcessProfile`,
codex=gpt-5.6-sol). Les défauts trouvés sont tous de nature data-flow/typage (que Phase H
refuserait) et ne cassent pas le parse/round-trip — mais dégradent la valeur d'exemplar.
**Findings retenus** :
- **G3-01 (P2, CONFIRMED adversarial)** — ideas : la branche `feature` est incohérente de bout
  en bout. `publish-gate` (switch, discriminant `visibility`) est alimenté par `collect`, qui
  peut émettre `refine-report` (sortie de `refine-rounds`→subworkflow `ideas-refine-round`), or
  `refine-report` {draft,ready} NE PORTE PAS `visibility` et ne conforme pas non plus au schéma
  `idea-note` {summary,visibility} du terminal `private-note`. Une idée `feature` route donc
  TOUJOURS `default`→`private-note` et ne conforme JAMAIS — elle ne peut jamais être publiée,
  contrairement à bug/vision qui fonctionnent. *Fix* : insérer un `transform` normalisant
  `refine-report`→`idea-note` (mapper draft→summary + propager visibility) entre `refine-rounds`
  et `collect`, OU faire produire `idea-note` par `ideas-refine-round`.
- **G3-02 (P2, CONFIRMED adversarial)** — conception : le body du `map` reçoit la COLLECTION
  entière au lieu d'un ITEM. `variant-fanout` (map, `collection_field angles`, items = strings)
  a un body `conception-variant` dont l'`input_schema_ref` = `angle-list` (conteneur
  {angles:[string]}), idem son node `propose` — chaque invocation reçoit la liste complète,
  contredisant la sémantique per-item du MapNode ET la desc « for one angle ». `map` n'apparaît
  QUE dans conception ⇒ c'est le SEUL exemplaire canonique, il enseigne le mauvais pattern.
  *Fix* : créer un schéma item `design-angle` et le référencer comme `input_schema_ref` du
  subworkflow et du node `propose`.
- **G3-03 (P3)** — conception : `conception-judge.input_schema_ref` = `angle-list` alors que le
  node `judge` consomme la sortie agrégée du map (`variants`) + `checks-join` ; l'input réel
  du judge n'est pas modélisé. *Fix* : déclarer un schéma {variants,checks} au lieu de réutiliser
  `angle-list`.
- **G3-04 (P3)** — deux nodes déclarent un budget wall-clock > timeout node effectif (factory
  `drive-implementation` 5400 > 3600 ; ideas `deep-research` 2700 > 1800) : budget inatteignable,
  incohérence numérique. *Fix* : `max_wall_clock_seconds ≤ default_node_timeout_seconds`, ou
  relever le timeout, ou (Phase H) override per-node.
- **G3-05 (INFO)** — factory : l'enveloppe `max_total_tokens=4M` est < somme des maxima déclarés
  (~5,9M avec 5 rounds Codex au plafond) ; plafonds ≠ réservations donc non-invalide, mais le
  golden ne démontre pas une enveloppe auto-cohérente. *Fix optionnel* : relever le plafond ou
  documenter le bornage volontaire.

## Dimension 4 — Sécurité profonde (invariant Plan-Then-Execute)

**Résumé** : la FORME des deux contrats porte l'invariant « le graphe signé EST le plan » de
façon solide. Aucun champ n'est sémantiquement endpoint/commande/code (`capability.call` = pure
référence ; `transform` = field_map déclaratif) ; l'override d'`agent.invoke` est budget-only
par construction (atténuation-seule sans champ d'élargissement) ; les labels switch/classifier
sont un `Vec` statique + port `default` réservé (le modèle SÉLECTIONNE, ne minte jamais une
destination) ; le fail-closed (deny_unknown + enum taggée) est justifié partout vs feed raw-op.
Le carry P2 THREAT_MODEL « graphe signé = plan » est tracé vers l'enforcement Phase H (freeze §8)
et le census 29 reste intact.
**Findings retenus** :
- **D4-SEC-01 (P2, CONFIRMED adversarial + re-vérifié par le writer)** — la disjonction
  `PRESENTATION_KEYS` n'est garantie ni structurellement ni pour un graphe arbitraire.
  `graph_semantic_view` (`pack.rs:1236-1246`) filtre par KEY récursivement à toute profondeur ;
  `nodes: BTreeMap<String,WorkflowNode>` (`graph.rs:628`, id = clé JSON) et `field_map: BTreeMap<String,String>`
  (`:170`, clé = nom de champ) sont des clés DATA-DRIVEN que `deny_unknown_fields` NE couvre PAS
  (il garde les champs de struct, pas les clés de map). Un node id ou une clé field_map valant
  ex. `"layout"`/`"x"` PARSE puis DISPARAÎT du digest sémantique — deux plans différents peuvent
  donc partager le même digest `graph_semantic`. Le test `schema_property_names_disjoint...`
  ne mesure QUE les property-names de schéma ; `goldens_pass_pack_semantic_view_unchanged` ne
  couvre QUE les 4 goldens committés (0 collision aujourd'hui = pas d'exploit vivant). Le freeze
  §6:102 SUR-AFFIRME l'impossibilité générale. *Nuance de scope re-vérifiée* : le byte-binding
  faisant autorité est le digest `content` (recomputé depuis `PACK.lock`), PAS `graph_semantic`
  — donc aucun bypass d'autorité VIVANT en G ; c'est une sur-affirmation du freeze + une défense-en-profondeur
  manquante pour Phase H. *Fix* : consigner en freeze §8 un 3e refus Phase H « tout node id ET
  toute clé de field_map (et toute clé data-driven) DISJOINTS de `pack::PRESENTATION_KEYS` » ;
  adoucir §6 en « les champs de struct + les 4 goldens sont vérifiés ; l'arbitraire est un refus
  Phase H ».
- **D4-SEC-02 (P3)** — bornes NonZero sans plafond haut (`max_iterations`/`max_cardinality`/`quorum`
  = `u32::MAX` admissible ; budgets/timeouts sans cap). Backstop `MAX_FILE_SIZE_BYTES=256MiB`
  plafonne la TAILLE mais pas le multiplicateur `repeat×map`. La borne reste une CONSTANTE signée
  (pas une sortie LLM) ⇒ l'invariant tient, le risque est DoS-dans-le-budget. *Fix* : noter le
  H-point au body du commit / module doc ; cap dur optionnel si un vertical le justifie.
- **D4-SEC-03 (P3)** — les champs texte libres (description, model_id, field_map values,
  allowed_context_fields, discriminant_field, signal_name…) sont HORS de `looks_like_materialised_secret`
  (qui ne s'applique qu'à `secret_refs`), et les fichiers graph/agent sont pinnés par digest, JAMAIS
  embarqués dans le root que `scan_for_disclosures` parcourt ⇒ jamais scannés. « Zéro secret par
  construction » ne prouve que l'absence d'un champ secret, pas qu'un paste accidentel ne peut rider
  dans le texte libre (gap latent, model_id des goldens propres aujourd'hui). *Fix* : note honnête
  dans le module doc + scan best-effort des bytes pinnés routé Phase H/T.
- **D4-SEC-04 (P3)** — le port réservé `default` peut collisionner avec un `branch_labels` valant
  littéralement `"default"`, et un `branch_labels` vide route tout vers default : aucune garde
  structurelle en G (ambiguïté de routage, pas escalade d'autorité — le set clos garantit qu'aucune
  destination neuve n'est mintée). *Fix* : consigner pour Phase H « `"default"` ∉ branch_labels »
  (+ non-vide) en freeze §8.

## Dimension 5 — Research grounding + honnêteté des claims

**Résumé** : quasi-totalité des claims fidèlement ancrés. La réconciliation `sequence` (freeze §2)
cite `kickoff.md:449` verbatim et correspond à la table research §10.3 (18 rangs, aucun node
`sequence`) ; table freeze §1 == `FROZEN_NODE_KINDS` == enum `WorkflowNode` (18/18), et
`golden_richness_floor` prouve `exercised == frozen`. §7 « aucune règle de digest neuve » honnête
et disclosé (réutilise `factory_profile::canonicalise` Phase D + PACK.lock blake3 Phase F, deux
préexistantes). Citation §19.3 du ClassifierNode correcte. Toutes les références « Phase H/L1/T »
et « AM-3 » sont des délégations de scope LÉGITIMES ancrées dans plan/kickoff — PAS des promesses
futures interdites (STALE-PHASE-K).
**Findings retenus** :
- **D5-01 (P2, CONFIRMED adversarial + re-vérifié par le writer)** — le module doc `graph.rs:19-21`
  affirme « the v1 graph itself is cycle-free by construction, which is STRICTER than research §10.5 »
  et le freeze §5 (`:84-91`) « le graphe top-level reste un DAG par construction… sans analyse de
  cycles ». C'est une SUR-AFFIRMATION : `edges: Vec<WorkflowEdge>` (`graph.rs:630`) porte des
  from/to arbitraires SANS contrainte d'acyclicité ; l'intégrité référentielle des edges est
  elle-même renvoyée à Phase H (`:49-52`) ; research §10.5 liste explicitement le rejet de cycle
  comme devoir DU COMPILATEUR ; le freeze §8 consigne l'intégrité edges + l'unicité de port mais
  PAS la détection de cycle. Un golden hand-authored avec edges A→B, B→A parse en Phase G. La
  prémisse « by construction » est vraie pour l'ABSENCE de PRIMITIVE de back-edge (itération
  uniquement via repeat/map délégués à un subworkflow PINNÉ, content-addressé, récursion
  impossible) mais FAUSSE pour un jeu d'edges arbitraire. *Fix* : reformuler en distinguant
  ce qui est vrai « par construction » (aucune primitive de back-edge) de ce qui reste un devoir
  compilateur (rejet de tout cycle sur edges quelconques, §10.5) ; supprimer/qualifier « sans
  analyse de cycles » en freeze §5 et AJOUTER « rejet de cycle sur edges quelconques » à la liste
  §8 Phase H.
- **D5-02 (P3)** — le module doc `graph.rs:38-40` « an override can only ATTENUATE » englobe
  implicitement le budget, alors que sa MAGNITUDE n'est pas structurellement clampée au parse
  (`AgentInvokeNode.budget` peut déclarer une valeur supérieure ; la comparaison enfant≤parent
  est numérique et Phase-H-seule). Le doc de type est, lui, précis. *Fix* : aligner la phrase-résumé
  sur le doc de type (« atténuation structurelle des tools/handoffs/context ; magnitude bornée
  enfant≤parent par le compilateur Phase H »).

## Dimension 6 — Process + livrables + patterns

**Résumé** : Phase G propre. TOUS les livrables G.2 présents : `SBFBWorkflowGraph.v1`,
`SBFBAgentDefinition.v1`, enum interne-taggée 18 nodes à ports typés, `handoff`≠`agent.as_tool`,
4 policies (error/timeout/budget/cancellation), 4 goldens (Factory PREMIER, 3 agents distincts),
schémas exportés + drift-gate, artefact de gel DATÉ portant M6/§2/§3/§4/§8. Les 2 écarts au plan
G.3 sont couverts PLAN-ADAPT (`node_profile_18_frozen`→`node_profile_frozen` SET-check ;
`schema_drift::graph_schema_matches_snapshot` fondu dans `schema_snapshot_stub` étendu). FRONTIER
en PROSE, census 29 [2 tagged]. Compteurs cohérents (64 `#[test]` crate = 47+17 ; 2192→2209 Win).
Patterns conformes (SPDX, titres `.v1`, enums sans titre = convention crate, 0 `Option`/`default`,
`NonZeroU32`). `gate_matrix` : 0 ligne §23 propriétaire=G. Dimension 6bis : lecteurs = compilateur
Phase H (même crate), Hub TS Phase T (runtime distinct = vraie frontière), packs F ; étiquette
docs-contrat correcte (contrat MACHINE fermé maintenant, GUIDE/llms.txt différés au wrap-up AM-3).
**Findings retenus** :
- **G6-01 (P2, CONFIRMED adversarial + re-vérifié par le writer)** — les 2 artefacts de planning
  attendus par la phase sont UNTRACKED (`?? sprint83_node_profile_freeze.md`,
  `?? sprint83_phase_g_preflight.md`). Le freeze est un livrable G.2 DUR (plan.md:649-650) ET un
  critère d'acceptation G.4 machine-vérifiable — son omission raterait SILENCIEUSEMENT un critère
  (aucun signal build pour un `.md`, contrairement à un module source référencé par `lib.rs`). Les
  sources untracked (`graph.rs`, 4 goldens, `workflow_golden_artifacts/**`) seraient rattrapées par
  le build ; les `.md`, non. *Fix* : `git add` explicite des 2 `.md` au commit ; vérifier
  `git diff --cached --stat` que le freeze daté ET le préflight sont dans l'index.
- **G6-02 (P3)** — deux tests nommés au plan G.3 n'existent pas sous leur nom brut (adaptations
  PLAN-ADAPT sans perte de couverture : `node_profile_18_frozen`→`node_profile_frozen` ;
  `schema_drift::…graph_schema_matches_snapshot`→`schema_snapshot_stub` étendu). *Fix* : nommer
  ces renommages/fusions comme PLAN-ADAPT dans la section traçabilité du body G.
- **G6-03 (P3)** — la clôture docs-contrat GUIDE/llms.txt des 2 frontières neuves est différée au
  wrap-up S83 (AM-3) ; seul le contrat machine + la prose frontier sont posés (split correct §6.12).
  Légitime mais doit rester tracé sous peine de carry orphelin. *Fix* : nommer explicitement la dette
  « GUIDE/llms.txt `SBFBWorkflowGraph.v1` + `SBFBAgentDefinition.v1` → wrap-up S83 (AM-3) » dans la
  section Carry-closure du body G, pour que le Track K docs-contrat de l'audit gate la retrouve.

## Findings réfutés en adversarial

**Aucun.** La passe de vérification indépendante a couvert les 5 findings susceptibles de bloquer
(G3-01, G3-02, D4-SEC-01, D5-01, G6-01) et les a tous jugés **CONFIRMED** avec re-mesure
fichier:ligne ; aucun finding n'a été réfuté. Les P3/INFO des dimensions n'ont pas été contestés.
Le writer a re-vérifié à part D4-SEC-01, D5-01 (sur-affirmations dans le code/freeze) et G6-01
(statut untracked) — les trois tiennent.

## Mutations — 7/7 tuées

1. enum rename → `node_profile_frozen` rouge (SET-check schéma émis vs `FROZEN_NODE_KINDS`).
2. drop `deny_unknown_fields` sur `InputNode` → `unknown_field_rejected_for_every_kind` rouge.
3. champ nommé `layout` → disjonction présentation rouge.
4. edge bypass → dominator-check factory rouge.
5. handoff illicite → `golden_richness_floor` / allowlist rouge.
6. JSON tronqué → `four_goldens_validate` rouge.
7. pin corrompu → `golden_pins_resolve` rouge.

## Items bloquants avant Codex/commit

1. **D5-01 — corriger la sur-affirmation « cycle-free by construction / sans analyse de cycles »**
   dans `graph.rs:19-21` (module doc) + freeze §5 (`:84-91`) : distinguer construction-facts (aucune
   primitive de back-edge ; repeat/map délèguent à un enfant pinné content-addressé) du devoir
   compilateur (rejeter tout cycle sur edges quelconques, §10.5), et AJOUTER « rejet de cycle sur
   edges quelconques » à la liste freeze §8 Phase H. Garantie fausse dans un artefact signé committé.
2. **D4-SEC-01 — adoucir la sur-affirmation §6:102 du freeze** (« deux graphes différents ne peuvent
   pas s'effondrer sur le même digest `graph_semantic` ») : la scoper aux champs de struct (structurel)
   + aux 4 goldens (runtime), et AJOUTER un 3e refus Phase H en §8 (« node id + clés field_map /
   data-driven DISJOINTS de `pack::PRESENTATION_KEYS` »). Noter que le digest `content` faisant
   autorité écarte tout bypass VIVANT en G, mais la claim d'impossibilité générale est fausse.
3. **G6-01 — stager les 2 livrables planning untracked** : `git add` explicite de
   `.planning/active/sprint83_node_profile_freeze.md` (livrable G.2 + critère G.4) et de
   `.planning/active/sprint83_phase_g_preflight.md` (autorité de phase) ; vérifier via
   `git diff --cached --stat` avant le commit.
4. **G3-01 + G3-02 — arbitrer les 2 défauts sémantiques des goldens canoniques** : décider
   fix-now vs consignation explicite à Phase H (validation cross-edge / item-type). Comme ce sont
   les exemplaires de référence (le `map` de conception est le SEUL exemplaire du kind), les
   expédier faux enseigne un mauvais pattern.

**Rappel gate pré-commit** : le run Docker `sbfb-ci` (fmt 1.94 + nextest Linux) doit être vert
avant le commit de phase (non encore observé vert par ce round). **Après ces items, la phase passe
en Codex** (le verdict PASS-PENDING/PASS suit les rounds Codex, pas ce round de review).
