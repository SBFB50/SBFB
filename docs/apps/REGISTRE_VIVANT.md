# Registre vivant — concept de projet P2P

**Statut** : concept exploratoire, nom provisoire, non-engagé pour un sprint.
**Date** : 2026-04-20
**Nature** : réflexion design produit, pas plan d'implémentation.

---

## 1. Concept en une phrase

Une infrastructure P2P où des personnes non-alignées (journalistes,
chercheurs, citoyens-lurkers) cross-référencent, annotent et
archivent les données publiques sur les pouvoirs (individus
influents, entreprises, États, zones de conflit) — par
micro-contributions atomiques, sans admin central, sans exposition
de l'identité du contributeur, avec LLM local qui interroge le
corpus en restant sur la machine du chercheur.

**Cadre théorique** : outil de contre-démocratie digitalisée au
sens de Pierre Rosanvallon (*La Contre-Démocratie*, 2006) —
exerce le pouvoir de surveillance et le pouvoir d'imputation sans
prétendre exercer le pouvoir d'obstruction (qui reste aux
institutions). Cohérent avec la sociologie numérique 2026 qui
caractérise le lurking comme filtration défensive active, pas
comme désengagement.

---

## 2. Le gap où ce projet vit

Projets existants qui adressent partiellement le problème :

- ICIJ (Pandora / Panama / Paradise Papers) — enquêtes
  journalistiques sur offshore finance
- OCCRP (Organized Crime and Corruption Reporting Project) —
  investigations transfrontalières
- Bellingcat — OSINT visuelle, workflows communautaires
- Forensic Architecture (Goldsmiths) — reconstruction 3D zones
  de conflit, analyse de crimes de guerre
- ACLED — Armed Conflict Location & Event Data (structuré,
  académique)
- LittleSis — power mapping US (knowledge graph centralisé)
- LobbyFacts.eu — registre lobbies UE
- OpenCorporates — corporate ownership worldwide
- Regards Citoyens — NosDéputés.fr / NosSénateurs.fr FR
- Citizen Lab (Munk School Toronto) — research sur surveillance
  d'État
- Internet Archive — préservation (centralisée, subpoenable)

**Gaps non-couverts** par l'ensemble de ces projets :

1. Tous centralisés — un serveur saisi ou pressions juridiques =
   données perdues ou modérées
2. Tous cloisonnés — pas de cross-référence native entre les
   données des différents projets
3. L'interrogation LLM des corpus passe par OpenAI / Anthropic /
   Gemini — les queries des chercheurs sont visibles par ces
   providers
4. Le lurker (journaliste curieux, chercheur, citoyen) ne peut
   pas contribuer anonymement sans créer de compte ni s'exposer

C'est précisément ce quadruple gap que l'architecture nexus-grid
adresse par construction.

---

## 3. Quatre modules composables

### Module 1 — Archive vivante de documents d'intérêt public

**Quoi** : blobs iroh content-addressed BLAKE3 des documents
publics menacés (rapports d'audit Cour des Comptes, contrats
publics archivés, documents gouvernementaux scrubbed from
websites, leaks déjà publiés et vérifiés, comptes rendus
d'audiences publiques, décisions judiciaires).

**Apport nexus-grid** : si le site officiel disparaît ou modifie
le document, la version archivée persiste répliquée. Chaque
contributeur choisit quels blobs il réplique selon ses capacités
stockage. Aucun admin central, aucune fondation à attaquer,
résilience par multiplication volontaire.

**Précédent à dépasser** : Internet Archive (centralisé,
subpoenable), Data Refuge 2017 (sauvetage climate data Trump
admin 1ère mandature, effort ponctuel).

### Module 2 — Graphe de pouvoirs croisés

**Quoi** : iroh-doc CRDT collaboratif où chaque contributeur
ajoute des entités (personnes, entreprises, États, ONG) et des
relations (mandat politique, actionnariat, conjoint de, membre
de, financement de, sanctionné par). Chaque relation cite sa
source (blob Module 1 ou URL externe).

**Queries possibles** : identifier les personnes qui ont à la
fois un mandat électif et une participation significative dans
une entreprise bénéficiaire de marché public dans leur
juridiction. Identifier les réseaux de shell companies reliés à
une personne sanctionnée.

**Apport nexus-grid** : graphe vivant multi-curateur (chaque
curator list a son point de vue), annotations CRDT sans admin,
LLM local qui traverse le graphe pour répondre à des queries
complexes sans les exposer à un provider tiers.

**Précédents à dépasser** : LittleSis (US seul, centralisé,
scrapé), OpenCorporates (corporate only), LobbyFacts (lobbies EU
only), Regards Citoyens (parlementaires FR only). Aucun ne fait
la synthèse cross-silo.

### Module 3 — Lecture collaborative de documents

**Quoi** : annotation CRDT de PDFs volumineux (rapports
parlementaires, décisions judiciaires, contrats publics publiés,
ouvrages de référence OSINT). Chaque annotation = highlight +
commentaire + lien vers entité Module 2 + lien vers autre
document Module 1. Chaque annotation est signée Ed25519 par son
auteur (pseudonyme par défaut, curator-identifiable optionnel).
OCR + LLM extraction entities automatique sur chaque blob ingéré
pour pré-peupler Module 2.

**Apport nexus-grid** : annotations vivent sur le nœud du
contributeur puis répliquées CRDT aux peers intéressés. Pas de
serveur central qui peut être subpoenaed pour révéler qui a
annoté quoi.

**Précédents à dépasser** : DocumentCloud (ICIJ tool centralisé),
Hypothesis (annotation web centralisée).

### Module 4 — Corroboration zones de conflit

**Quoi** : un témoin ou observateur OSINT ajoute un événement
(timestamp + géocoordonnées + médias joints + source) dans un
iroh-doc spécifique à une zone (ex : Liban-2026, Gaza-2026,
Ukraine-Est-2025, Myanmar-2024). Corroboration : d'autres
contributeurs ajoutent des observations indépendantes confirmant
ou contredisant.

**Algorithme de convergence** : 3+ sources indépendantes sur
même événement + fenêtre temporelle (15 min) = haute confiance.
Chaîne de custody cryptographique (événement signé, horodaté,
BLAKE3-hashed, impossible à modifier rétroactivement).

**Queries possibles** : patterns cross-événements ("hôpitaux
frappés dans secteur X pendant période Y"), déplacement de
civils corrélé à activité aérienne, chronologie d'une offensive.

**Apport nexus-grid** : structuration native, corroboration
protocolaire (pas ad hoc Discord/Twitter), LLM local pour
analyse sensible sans exposer les queries.

**Précédents à dépasser** : ACLED (centralisé, pas en temps
réel, pas de corroboration crowdsourced), Forensic Architecture
(cas par cas, centralisé Goldsmiths), Bellingcat workflows
(Discord/Twitter, non-structuré).

---

## 4. Partage de puissance LLM — architecture

### Règle fondamentale à poser d'abord

Distinction critique entre deux types de workloads :

| Workload | Compute | Rationale |
|---|---|---|
| Batch public à l'ingestion | Distribué (task pipeline nexus-grid) | Résultat public, réutilisé par tous, une fois suffit. Pool GPU optimal. |
| Query interactive d'un chercheur | Local (nœud personnel) | La query révèle ce que le chercheur cherche. Threat model T4/T5 exige que la query ne transite par aucun tiers. |

Cette asymétrie suit le pattern Tor : *expensive indexing/crawling
public, interactive browsing private*. Sans elle, le projet
devient un outil de surveillance sur les chercheurs eux-mêmes.

### Mapping workload par module

**Distribué (task pipeline, pool GPU)** :

- Module 1 : OCR + extraction entities + summary + embeddings à
  l'ingestion de chaque blob
- Module 2 : enrichissement auto d'une nouvelle entité,
  cooccurrences matching dans blobs existants, génération de
  description des relations
- Module 3 : OCR + suggestion d'annotations + traductions
  multi-langues
- Module 4 : transcription audio/vidéo (Whisper large-v3),
  traductions (arabe, anglais, français, hébreu, ukrainien selon
  zone), OCR captures, extraction géolocalisation depuis images,
  reconnaissance d'objets (bâtiment, convoi, drone), clustering
  de rapports similaires

**Local (LLM personnel)** :

- Module 1 : recherche textuelle simple (grep, full-text index
  SQLite local)
- Module 2 : TOUTES les queries d'analyse du graphe
- Module 3 : question/answer sur un document individuel
- Module 4 : analyse de patterns cross-événements par un
  chercheur

### Intégration avec le task pipeline existant

Le pipeline nexus-grid actuel (depuis Sprint 4) : `submit →
claim → execute → result`. Aucun nouveau protocole nécessaire.
Extensions concrètes :

1. Nouveaux `Task.task_type` : `doc_ocr`, `doc_summarize`,
   `entity_extract`, `translate`, `transcribe_audio`,
   `geo_locate_image`, `vision_detect`
2. `Task.redundancy_factor` (prévu pour compute theft mitigation)
   — pour Module 4 particulièrement, exiger 3 workers
   indépendants qui produisent le même résultat (majority voting
   anti-manipulation)
3. Rate-limit `governor` Sprint 21 Phase A protège contre un
   adversaire qui floode le pipeline avec fausses ingestions
   pour épuiser le GPU bénévole

### Consent 4 niveaux — mapping concret

Le worker qui consent au GPU-share configure dans `consent.json` :

- **L1 mes projets uniquement** : contribue uniquement aux
  archives utilisées par soi-même. Défaut.
- **L2 open source vérifiés** : contribue aux projets AGPL
  vérifiés Keyoxide (cf. Sprint 14 deploy verified). Curator
  lists OCCRP / ICIJ / Citizen Lab probablement dans cette
  catégorie.
- **L3 whitelist manuelle** : l'opérateur choisit explicitement
  (ex : Bellingcat oui, Forensic Architecture non).
- **L4 tous** : contribue à toute curator list.

Le kudos ledger accumule pour le worker qui a fait le OCR de
500 PDFs. Reste non-monnaie (principe Day 0), mais rend la
contribution visible et cross-référençable.

---

## 5. Avertissements critiques intégrés au design

### A1 — Cible T5 state-targeted automatique

Un tel outil, s'il fonctionne, cible explicitement individus
influents (Module 2) et opérations étatiques (Module 4).
Conséquences :

- **Risque personnel contre le mainteneur** : Pegasus-class
  surveillance, pressions juridiques, harcèlement, défamation
- **Risque réseau** : infiltration Sybil par services de
  renseignement qui ajoutent désinformation pour décrédibiliser,
  ou attaquent l'infrastructure (DDoS, DNS poisoning pkarr, etc.)

**Mitigations protocolaires déjà en place dans nexus-grid** :
signature obligatoire de chaque contribution (Ed25519), curator
lists qui écartent les contributions non-vérifiées, proof-of-work
par contribution (Hashcash S19) pour ralentir Sybil farming,
warrant canary mensuel (S18), Sybil composition 3 couches (S22
en cours), encryption at rest + duress PIN + panic wipe (S20).
Aucune action ne devrait être prise sur Modules 2 et 4 sans que
ces briques soient opérationnelles et testées.

### A2 — Failure mode Bellingcat / OCCRP / ICIJ — faux positifs, harcèlement, diffamation

Les projets OSINT ont tous connu des incidents où des
contributeurs ont doxxé la mauvaise personne, alimenté des
théories complotistes, ou outé des personnes vulnérables par
erreur.

**Mitigations à câbler dans le schéma** :

- **Scope dur** : pas de module doxing individu privé. Seulement
  personnes ayant un rôle public (mandat élu, dirigeant
  entreprise au-delà d'un seuil, officier militaire identifié,
  agent d'État identifié dans exercice de fonction). Règle
  codée dans la Constitution du projet.
- **Process de contestation** : si une personne conteste son
  inscription, protocole de review obligatoire par curator list
  de référence avant que la contestation soit ignorée.

### A3 — Capture par extrêmes (pattern Nextdoor CHI 2024)

Sans design intentionnel, tout outil de transparence dérive
vers un outil de surveillance communautaire contre outsiders.
La recherche [Under the Neighborhood: Hyperlocal Surveillance on
Nextdoor](https://dl.acm.org/doi/10.1145/3613904.3641967) (CHI
2024) documente empiriquement que Nextdoor produit des
"digitally gated communities" avec "race and class exclusion
enforced through private policing" particulièrement dans les
quartiers en gentrification.

**Mitigation** : scope dur limité aux pouvoirs (mandats publics,
fonctions dirigeantes, rôles étatiques), pas aux citoyens
ordinaires. Le schéma Module 2 refuse structurellement les
entités sans mandat/fonction vérifiable. Un utilisateur qui
soupçonne un voisin = refusé par le schéma, pas seulement par
la modération.

### A4 — Éviter overclaim hacktivist

Ne pas positionner comme "révéler la vérité cachée du pouvoir".
Positionner comme "documenter ce qui est déjà public de manière
cross-référencée résistante". Ton calme, factuel, ennuyeux.
Immunise contre problèmes CVP, ciblage T5 accéléré,
décrédibilisation par adversaires.

### A5 — Pas de partnerships formels requis

Cohérent avec vision_model.md (2026-04-20 : no funding / no
fondation / no startup patterns). Si Amnesty / HRW / CPJ /
OCCRP / Bellingcat veulent utiliser le système, ils créent
leurs propres curator lists et gèrent leur propre workflow. Le
projet fournit l'infra, pas le service. Pattern OpenBSD — le
code existe, les utilisateurs viennent ou pas.

### A6 — Workflow Module 4 nécessite adaptation spécifique

Le submit de Module 4 par un utilisateur sur le terrain doit
**obscurcir l'origine IP** avant d'atteindre le task pipeline.
Deux options :

- **Arti embed** (Tor client Rust 2.2 déjà dans
  VALIDATED_BLUEPRINT Couche 2) : l'upload passe par Tor, l'IP
  du témoin n'est jamais liée au blob
- **Relai P2P** : le blob traverse un worker relai (autre nœud
  nexus-grid) avant d'atteindre le task pipeline — pattern
  Tor-like au niveau P2P, non-trivial architecturalement

Module 4 ne devrait pas être déployé avant que l'une des deux
options soit implémentée et testée.

---

## 6. Scope solo réaliste

Ordre d'implémentation compatible avec discipline sprint :

1. **Module 1 (archive blobs)** — trivial sur iroh-blobs
   existant. UI de publication + curation. 1 sprint.
2. **Module 3 (annotation PDF collaborative)** — iroh-doc CRDT
   + shell React avec PDF.js + annotations layer signées. 1-2
   sprints. Sans LLM d'abord, LLM en enrichissement
   second-pass.
3. **Module 2 (graphe pouvoirs)** — complexe, CRDT avec schéma
   validé + cross-link entités. 2-3 sprints.
4. **Module 4 (zones de conflit)** — le plus sensible
   politiquement et techniquement (anonymisation submission).
   À laisser pour après v1.0 et après validation communauté.

Chaque module est shippable indépendamment. Arrêt possible
après Module 1 avec contribution utile déjà faite (archive
résistante).

---

## 7. Ce que ce projet n'est pas

- **Pas WikiLeaks** : pas de hosting de leaks brutes non-vérifiées
- **Pas Anonymous** : pas de narrative hacktivist (cf.
  discussion naming AAA / signaling politique)
- **Pas un outil d'enquête journalistique propriétaire** :
  OCCRP / ICIJ font ça, avec qualité éditoriale et due
  diligence — le projet fournit l'infrastructure, pas le
  produit éditorial
- **Pas une plateforme de deliberation** : la littérature
  (Cardon 2010, 15 ans de confirmation) montre que ces
  plateformes échouent toutes — agrégation de micro-gestes ≠
  délibération collective
- **Pas un outil contre un gouvernement spécifique** : éviter
  ciblage géopolitique directionnel qui attire adversaires
- **Pas un tool pour doxer des citoyens ordinaires** : scope
  strict pouvoirs publics

---

## 8. Précédents honorés, différenciation claire

L'architecture honore :

- **Bellingcat workflow** — OSINT crowdsourcé corroboré
- **LittleSis power mapping** — knowledge graph relations
- **ICIJ document annotation** — DocumentCloud pattern
- **ACLED** — structured conflict data
- **Internet Archive mission** — préservation long-terme
- **Citizen Lab research rigor** — signature + provenance
- **Forensic Architecture** — chaîne de custody cryptographique

Différence nexus-grid sur toutes les propriétés où c'est vital :
**décentralisé, non-subpoenable, queries privées, contributeur
pseudonyme, pas de fondation à attaquer, pas de funder à
influencer**.

---

## 9. Nom

Provisoire : *Registre vivant*. Autres pistes neutres :
*Registre*, *Trace*, *Luminos*, *Prisme*, *Commonrecord*,
*The Index*, *Archive*.

Éviter les noms chargés politiquement (*Vigil*, *Glasnost*,
*Spotlight*, *Watchtower*). Nom descriptif ou technique
préférable au nom slogan — cohérent avec discipline
anti-overclaim du projet.

Choix final à arbitrer au moment où le projet sortirait du
statut exploratoire.

---

## 10. Conditions de viabilité

Ce projet est **possible uniquement parce que nexus-grid est ce
qu'il est** :

- Solo maintainer assumé (cf. vision_model.md)
- AGPL-3.0 copyleft
- Pas de fondation à pressurer pour modérer
- Architecture vraiment P2P (pas federated pseudo-décentralisé)
- Compute décentralisé qui permet LLM local pour queries
  sensibles

Un projet équivalent lancé par une startup ou une fondation
serait immédiatement pressuré pour modération / compliance.
Le projet nexus-grid livre du code AGPL et n'a aucune surface
juridique où l'on peut lui demander de modifier l'outil. C'est
l'indépendance architecturale qui fait la proposition de valeur
géopolitique, pas un discours sur l'indépendance.

---

**Document exploratoire** — aucun engagement sprint par sprint.
À revisiter lorsque les briques sécurité S20-S22 seront matures
et que le user souhaitera sortir du statut exploratoire. La
discipline scope-cut s'applique : ne commencer Module 1 que si
décision explicite kickoff sprint.
