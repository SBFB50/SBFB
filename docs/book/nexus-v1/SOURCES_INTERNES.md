# Carte des sources internes

Cette carte relie le livre aux matériaux du dépôt. Elle ne constitue pas une
bibliographie publique et ne signifie pas que chaque note de recherche a été
implémentée. Son rôle est de préserver la filiation des idées et de permettre
une vérification technique ultérieure.

## Vision générale et protocole

- `README.md` — promesse publique prudente, architecture locale, distribution
  pair-à-pair, sandbox, bridge, provenance, recherche, Proof Cards, compute et
  limites de maturité.
- `COMMONS.md` — licence AGPL, absence de CLA, refus des tokens de gouvernance,
  droit au fork et distinction entre protocole libre et licences des apps.
- `SBFB_EXPLAINED.txt` — explication longue du protocole, de ses composants et
  de ses mécanismes sociaux.
- `docs/VISION_USE_CASES.md` — scénarios d’usage combinant distribution
  d’applications, état partagé et calcul local ou distribué.
- `.planning/research/vision_communs_idees_factory_builders.md` — vision des
  communs, rôle des porteurs, builders, pluralité, limites sociologiques et
  communautés humanistes.

## Atelier des communs et Ideas

- `.planning/research/idea_commons_workflow_anti_capture.md` — cadrage
  « Atelier des communs », anti-capture, porteur responsable, curation locale
  et séparation entre proposition pair-à-pair et artefacts de travail.
- `.planning/research/idea_creation_science_rrv_workflow.md` — clarification,
  incubation, divergence, critique sceptique, arbitrage humain et artefacts
  `ResearchPack` / `DecisionPack`.
- `.planning/research/idea_development_workflow_provider_neutral.md` — pipeline
  portable entre modèles et règle « le Hub propose, l’opérateur développe,
  l’humain arbitre, les gates vérifient ».
- `.planning/research/idea_hub_factory_ultracode_redesign.md` — frontières
  d’autorité, modèle de données, gouvernance polycentrique et passage vers
  Factory.

## RRV

- `.planning/research/rrv_app_protocol_best_features.md` — scopes de recherche,
  réponse proof-first, classes « lu / déduit / vérifié / non vérifié »,
  inspection des apps et actions typées.
- `.planning/research/rrv_protocol_boundary_analysis.md` — séparation entre
  primitives protocolaires, index local, formats réseau et application RRV.
- `.planning/research/SYNTHESIS_factory_rrv_protocol.md` — articulation entre
  protocole, RRV, Factory et applications.

## Workflow Hub, Conception et Factory

- `.planning/research/workflow_hub_product_conception_2026-07-15.md` —
  architecture produit du Hub, bibliothèque, auteur, agents, capacités, runs,
  évaluations, cycle de vie, interactions inter-apps et accessibilité.
- `.planning/research/workflow_agents_app_conception_ultradeep_2026-07-15.md`
  — runtime, contrats portables, quatre workflows de référence et frontières
  de sécurité.
- `docs/factory/WORKFLOW_AGENTS_OVERVIEW.md` — séparation Workflow Hub / shell
  de confiance / runtime Rust / protocole, plan compilé immuable et contrats
  versionnés.
- `docs/agent/PROCESS.md` et `docs/agent/AGENT_SYSTEM.md` — processus réel dont
  Factory reprend les rôles, gates, revues, preuves et décisions humaines.
- `docs/factory/FACTORY_GATES.md` — modèle de fabrication et de publication
  vérifiables.

## Équité, contributions et gouvernance

- `docs/FAIRNESS_VISION.md` — critique de la récompense de la seule puissance
  matérielle et principes de contribution irremplaçable, rendements
  décroissants et vieillissement de la réputation.
- `docs/fairness/CONTRIBUTION_FAMILIES_V1.md` — familles compute, stockage et
  relais, ainsi que leurs limites. Plusieurs éléments y sont explicitement
  post-V1 et ne doivent pas être présentés comme acquis.
- `docs/security/THREAT_MODEL.md` — menaces et frontières de sécurité à
  rapprocher de l’état de code au moment de toute publication.

## Discipline d’utilisation

Le livre ne doit jamais servir à transformer une note de recherche en fait
livré. Avant une édition publique :

1. reconstruire l’état du dépôt à une date précise ;
2. distinguer code, tests, preuve multi-nœuds, pilote et hypothèse ;
3. retirer ou reformuler les chiffres externes non revalidés ;
4. faire relire les chapitres sécurité, accessibilité et gouvernance par des
   personnes indépendantes ;
5. conserver dans le livre la différence entre provenance, corroboration et
   vérité.

