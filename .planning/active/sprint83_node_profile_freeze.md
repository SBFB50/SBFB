# Sprint 83 Phase G — Gel du profil de nodes (`SBFBWorkflowGraph.v1`)

**Date de la décision** : 2026-07-21.
**Décideur** : Phase G (main thread), sous préflight `sprint83_phase_g_preflight.md`
(verdict PLAN-ADAPT, blocant B1) ; source design : Research Pack
`workflow_agents_app_conception_ultradeep_2026-07-15.md` §10.3 (liste CANDIDATE,
« cette liste n'est pas gelée ») + kickoff D4 (gelée).
**Statut** : GELÉ après validation des 4 goldens (factory, ideas, conception,
babel — chacun committé, strictement parsé, round-trip prouvé, et l'ENSEMBLE
exerce les 18 kinds : test `graph::tests::golden_richness_floor`).

## 1. Le roster gelé — 18 node kinds

Miroir machine : `graph::FROZEN_NODE_KINDS` (`crates/sbfb-workflow-core/src/graph.rs`),
mesuré contre le schéma ÉMIS par le test SET-check discriminant
`graph::tests::node_profile_frozen` (ajout/retrait d'UNE variante → rouge ;
jamais un `count == 18`).

| # | kind | Rôle | Agentique |
|---|---|---|---|
| 1 | `input` | valider et introduire l'entrée typée | non |
| 2 | `transform` | mapping déclaratif pur (field_map, zéro code) | non |
| 3 | `model.invoke` | un appel LLM structuré, sans boucle ni outil | limité |
| 4 | `agent.invoke` | agent borné ; seul override = budget (atténuation) | oui |
| 5 | `agent.as_tool` | spécialiste appelé, RETOUR au parent (synthèse+guardrails parent) | oui |
| 6 | `handoff` | transfert de contrôle, payload schématisé + allowlist REQUIS, sans retour | oui |
| 7 | `capability.call` | activité autorisée par référence, args typés, zéro endpoint inline | non |
| 8 | `knowledge.query` | recherche dans un corpus PINNÉ | limité |
| 9 | `switch` | routage déterministe, labels statiques + port `default` réservé | non |
| 10 | `classifier` | le modèle PROPOSE un label d'un set clos ; hors-set → `default` | limité |
| 11 | `parallel` | branches statiques concurrentes (ports = labels) | non |
| 12 | `map` | fan-out borné (`max_cardinality` ≥ 1 constant) sur subworkflow pinné | non |
| 13 | `join` | fan-in à politique EXPLICITE {all, any, quorum(n≥1)} + sources déclarées | non |
| 14 | `repeat` | boucle bornée (`max_iterations` ≥ 1 constant) sur subworkflow pinné, arrêt par flag booléen de sortie | variable |
| 15 | `human.review` | interruption durable, décisions = labels statiques, chrome hôte | humain |
| 16 | `subworkflow` | appel d'un plan enfant PINNÉ | variable |
| 17 | `wait.signal` | attente durable d'un signal TYPÉ délivré par l'hôte | non |
| 18 | `output` | valider et publier un artefact durable | non |

Aucune fusion : les goldens montrent que chaque kind porte une sémantique ou un
risque distinct (`switch` déterministe vs `classifier` modèle-proposé ;
`parallel` statique vs `map` dynamique borné ; `handoff` transfert vs
`agent.as_tool` retour). Fusionner l'un de ces couples rendrait le risque
ambigu — exactement ce que §10.3 demandait de vérifier par les goldens.

## 2. Réconciliation `sequence` (préflight B1/M6 — divergence D4 vs §10.3)

`kickoff.md:449` (D4 GELÉE) énumère « sequence/switch/classifier/parallel/map/
join/repeat borné… » ; la table §10.3 du Research Pack (18 rangs) n'a PAS de
node `sequence`, et aucun node `sequence` n'existe dans le crate.

**Décision** : `sequence` est un MODE DE COMPOSITION, pas un node kind. Dans un
graphe data-flow, la séquence EST le câblage des edges (`A.out → B.in`) — un
node `sequence` n'aurait ni sémantique propre, ni risque propre, ni politique
propre. L'énumération du kickoff mélangeait familles de composition et node
kinds ; D4 reste honorée : la composition séquentielle existe structurellement
dans chaque golden (aucun n'a moins de 9 edges séquentiels). Le roster gelé est
la table §10.3, 18 kinds exactement.

## 3. Fail-closed sur node-kind inconnu (divergence ASSUMÉE, tension AM-12)

Le graphe signé REFUSE tout node kind inconnu et tout champ inconnu (enum
interne-taggée `kind` + `deny_unknown_fields` par variante, prouvé kind par
kind par `unknown_field_rejected_for_every_kind` sur les exemplaires réels des
goldens). C'est l'INVERSE de la politique feed raw-op (`CLAUDE.md
§Pre-launch protocol policy`, tolérance additive) : un plan qui recevra des
capacités doit échouer fermé sur ce qu'il ne comprend pas — un « node inconnu
stocké et propagé sans l'interpréter » serait un trou d'autorité, pas de la
robustesse. Pré-launch, étendre le roster = éditer la v1 courante (redéfinition
libre, goldens mis à jour dans le MÊME commit) ; post-launch, ce sera un bump
d'identité de contrat.

## 4. Le gel pinne le SET **et** la FORME (préflight S1a-F10)

- SET : `FROZEN_NODE_KINDS` ↔ schéma émis (`node_profile_frozen`).
- FORME : chaque corps de variante est pinné par le snapshot
  `contracts.schema.json` (`schema_snapshot_stub`, STRICT fail-on-missing) —
  un champ ajouté/retiré/renommé sous un kind stable rougit le drift-gate
  (l'anti-patron n8n « typeVersion drift sous nom stable »).
- Une seule orthographe canonique par kind (dottée : `agent.invoke`,
  `wait.signal`, …), zéro alias (l'anti-patron Dify `if-else`/`if_else` —
  fatal sous JCS).

## 5. v1 n'a AUCUNE primitive de back-edge ; le rejet de cycle est un refus H

Ce qui est vrai **par construction** : le format n'offre AUCUNE primitive de
boucle par arête — l'itération n'existe qu'à travers `repeat`/`map`, dont le
corps est un subworkflow PINNÉ content-addressé (`body_subworkflow_ref`) ; la
récursion directe est inexprimable. Ce qui n'est PAS garanti par construction :
`edges` est une liste libre — un auteur peut écrire `A→B, B→A`, et Phase G le
PARSE. Le **rejet de tout cycle sur edges quelconques** est le refus §10.5 du
compilateur (Phase H), consigné en §8 — la cible v1 reste « tout plan compilé
est un DAG », strictement plus fort que « cycle non porté par un node de boucle
borné », mais c'est le COMPILATEUR qui le rend vrai, pas le parse. Assoupli
seulement si un vertical (Z-AC) le prouve nécessaire — redéfinition consciente
de la v1.

## 6. Modèle de document F↔G (préflight M5/A9 — décision)

La présentation (canvas x/y/layout/zoom…) vit dans une **ENVELOPPE** autour du
graphe (document canvas du Hub, Phase T) — JAMAIS dans le graphe sémantique :
`deny_unknown_fields` la refuse au parse, et les 10 noms de
`pack::PRESENTATION_KEYS` sont disjoints des champs des deux contrats
(test `schema_property_names_disjoint_from_presentation_keys`) ET absents des
goldens (test `goldens_pass_pack_semantic_view_unchanged` :
`graph_semantic_view == identité`, `presentation_view == {}`). PORTÉE HONNÊTE
de cette garantie : elle couvre les CHAMPS DE STRUCT (structurel, schéma émis)
et les 4 goldens committés (mesuré) — PAS un graphe arbitraire : les clés
DATA-DRIVEN (`nodes` id de node, clés de `field_map`) ne sont pas couvertes
par `deny_unknown_fields`, et un node id nommé `layout` parserait puis serait
strippé par `graph_semantic_view` (deux plans différents partageraient alors
un même digest `graph_semantic`). Aucun bypass d'autorité VIVANT en G : le
digest faisant autorité est `content` (recalculé depuis `PACK.lock`), pas
`graph_semantic`. La fermeture générale est un refus compilateur consigné
en §8.

## 7. Sémantique des pins de référence

Chaque `*Ref.blake3` = pin `PACK.lock` (BLAKE3 hex des bytes du fichier
référencé) — la règle Phase F, AUCUNE règle de digest neuve. Dans les fixtures
committées, les bytes sont canonicalisés CRLF→LF (règle source-digest de
`factory_profile`) parce que les fixtures git subissent la translation de fin
de ligne au checkout, contrairement aux archives d'un vrai pack. La règle de
RÉSOLUTION (localiser le fichier depuis la ref) et la compatibilité de ports
(nominale par digest, v1) appartiennent au compilateur Phase H.

## 8. Consigné pour Phase H (refus §10.5 hors-scope G, préflight S1a-F7/A10)

Cinq règles statiques ABSENTES de la liste §10.5 du Research Pack (ou
sur-couvertes par elle), à ajouter au compilateur :
1. **Intégrité référentielle des edges** : chaque `edge.from`/`edge.to` nomme un
   node existant (« node inaccessible » ne couvre pas un edge vers un id
   inexistant). L'unicité des node ids est déjà structurelle (map keyed by id).
2. **Unicité des ports par label** : un `switch`/`classifier` dont chaque label
   (+ `default`) a exactement une arête sortante ; un `join` dont les
   `source_node_ids` existent et alimentent réellement le join.
3. **Rejet de tout cycle sur edges quelconques** (cible « tout plan compilé est
   un DAG », cf. §5 — le parse G ne le garantit pas).
4. **Clés data-driven disjointes de `pack::PRESENTATION_KEYS`** : tout node id
   ET toute clé de `field_map` (et toute clé data-driven future) hors des 10
   noms réservés (cf. §6 — `deny_unknown_fields` ne couvre pas les clés de
   map).
5. **Labels de routage sains** : `"default"` ∉ `branch_labels` (collision avec
   le port réservé) et `branch_labels`/`decision_labels` non vides.
6. **Provenance des discriminants de classification** (§19.3, Codex r1 GAP1) :
   un `switch`/`classifier` dont dépend un `capability.call` d'egress ou de
   publication ne peut PAS discriminer sur un champ de CLASSIFICATION
   (public/privé, niveau de confidentialité) produit par une sortie de
   modèle — cette classe de champ vient d'une source déterministe ou d'un
   `human.review`, jamais d'un LLM. Portée : les champs de classification
   SEULEMENT — un gate de QUALITÉ alimenté par un modèle (ex. babel
   `quality-gate`, dont la publication passe de toute façon par le signal
   communautaire `wait.signal`) est une classe différente, hors de ce refus. Le golden ideas
   l'exemplifie : la publication est un `human.review` et AUCUN schéma du
   store ideas ne porte de champ de visibilité qu'un modèle pourrait
   produire (test `ideas_publication_dominated_by_human_review`).

S'y ajoutent, déjà listés §10.5 et déclarés par la forme G : satisfiabilité du
`join` (all vs branches exclusives), budget enfant ≤ parent par dimension,
`capability_ref ∈ capability_requests` (forme prête, test golden), handoff
autorisé par le donneur (exercé par les goldens), refs pinnées résolubles.
**Carry P2 THREAT_MODEL (préflight A13/S3-08)** : v21 ne porte aucun asset
« graphe signé = plan » — l'asset/surface Plan-Then-Execute est routé à la
phase d'enforcement (H, sinon AH), tracé au body du commit G.

## 9. Preuves (exit machine G.4)

- `cargo nextest run -p sbfb-workflow-core --locked` : les 4 goldens parsent
  strictement + round-trip (`four_goldens_validate`), pins résolus
  (`golden_pins_resolve`), contrôles négatifs rouges
  (`golden_negative_controls_reject`), richesse plancher + roster 18/18 exercé
  (`golden_richness_floor`), gel SET-check (`node_profile_frozen`),
  `handoff_not_agent_as_tool`, disjonction présentation, cohérence
  Factory↔`FactoryProcessProfile.v1` (`factory_golden_matches_process_profile`).
- `bash scripts/check-frontier-contracts.sh` : exit 0, census 29 intact
  (aucun `DOMAIN_*` ni `*_FORMAT_VERSION` neuf — frontière en PROSE,
  précédent `factory_profile.rs`).
