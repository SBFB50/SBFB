# RRV scoped search, source crossing, and compute groups

**Date:** 2026-05-16
**Status:** recherche produit post-S66, non engagee en sprint
**Scope:** UX RRV, recherche par perimetre, croisement sources, groupes prives de compute, compute public app-driven
**Related docs:**
- `.planning/research/chat_ia_reseau_recherche_reseau_rnd.md`
- `.planning/research/public_verifiable_feed_roadmap.md`
- `.planning/research/p2panda_public_protocol_briques.md`
- `.planning/research/babel_translation_protocol.md`
- `.planning/research/DISTRIBUTED_GPU_RESEARCH.md`
- `.planning/research/gpu_pooling_distributed_inference.md`

---

## 1. These produit

Apres S66, RRV ne doit pas etre pense comme une simple barre de recherche
generale. La bonne experience est un **chat de recherche-action avec perimetre
explicite**.

L'utilisateur doit pouvoir demander:

```text
@Babel quelles langues manquent de validateurs ?
@network trouve une app verifiee pour traduire un PDF scanne
@network @web compare les apps OCR SBFB avec les solutions externes
@Babel @OCR-App cree un workflow de traduction OCR
@dev dans Babel, ou est geree la provenance des chunks ?
```

La valeur produit n'est pas "l'IA cherche partout". La valeur est:

```text
L'utilisateur choisit le perimetre, SBFB garde les niveaux de preuve separes,
et chaque resultat propose une action verifiable.
```

---

## 2. Cas de depart: seulement Babel

Si Babel est la seule application publique au debut, RRV reste utile. Il devient
d'abord un **moteur de preuve et d'exploration de Babel**, pas encore un moteur
de recherche general.

Questions utiles cote utilisateur:

- Quelles langues sont couvertes ?
- Quelles traductions sont verifiees ?
- Quelle source a servi pour ce passage ?
- Quel chunk a ete traduit par quel worker ou validateur ?
- Ou manque-t-il des validateurs humains ?
- Quels corpus sont redistribuables ?
- Quels resultats sont stale ou incomplets ?
- Quelle tache peut tourner sur ma machine ?

Produit:

```text
Babel devient une vitrine inspectable: corpus, provenance, gaps,
contributions, et actions sont visibles dans le chat.
```

Le risque a eviter: vendre RRV comme "moteur du reseau" avant qu'il existe
plusieurs apps. Le bon wording est:

```text
RRV commence comme explorateur verifiable de Babel, puis s'etend au catalogue.
```

---

## 3. Quand plusieurs apps existent

Avec plusieurs applications, RRV devient le **moteur de decouverte intelligent
du reseau SBFB**.

L'utilisateur ne cherche plus seulement un nom d'app. Il cherche une capacite:

- "trouve une app qui traduit un PDF scanne"
- "quelle app peut analyser un corpus juridique hors ligne ?"
- "montre les outils open source verifies pour resumer une video"
- "quelle app utilise un modele compatible avec ma VRAM ?"
- "quel workflow combine OCR + traduction + validation humaine ?"

RRV peut comparer les apps selon:

- provenance source;
- artifact hash;
- build quorum;
- compatibilite machine;
- niveau de preuve;
- curations;
- risques connus;
- fraicheur;
- disponibilite reseau;
- actions possibles.

Actions utilisateur attendues:

- ouvrir;
- installer;
- verifier une release;
- lancer un audit reseau;
- fork;
- composer un workflow;
- creer une nouvelle app a partir de briques existantes.

---

## 4. Modele de perimetre dans le chat

Le chat doit eviter les reponses implicites. Le perimetre doit etre visible et
controllable.

### 4.1 Perimetres

| Tag / mode | Sens | Exemple |
|---|---|---|
| `@current` | App ouverte, defaut | `@current resume les erreurs recentes` |
| `@Babel` | App/projet precis | `@Babel quelles langues sont faibles ?` |
| `@Babel @OCR-App` | Croisement d'apps nommees | `cree un workflow OCR + traduction` |
| `@network` | Catalogue SBFB verifie ou verifiable | `trouve une app compatible avec ma machine` |
| `@web` | Web externe | `compare avec les solutions existantes` |
| `@dev` | Recherche developpement | `cherche API, manifests, erreurs, patterns` |
| `@private:<group>` | Groupe prive | `cherche dans les resultats du groupe labo-x` |

### 4.2 Defaut UX recommande

1. Si l'utilisateur est dans une app, le defaut est `@current`.
2. Si l'utilisateur ajoute `@network`, le chat etend au reseau SBFB.
3. Si l'utilisateur ajoute `@web`, les resultats web restent etiquetes comme
   externes.
4. Si l'utilisateur ajoute `@dev`, les resultats privilegient code, schemas,
   manifests, capabilities, logs, tests, erreurs et patterns reutilisables.

Le chat peut proposer un changement de perimetre, mais ne doit pas l'inventer
silencieusement.

---

## 5. Croiser `@network` et `@web`

Oui, croiser les sources est une feature majeure.

Exemple:

```text
@network @web trouve les meilleures solutions de traduction OCR et compare-les
a ce qui existe deja.
```

Reponse attendue:

- couche SBFB: apps verifiees, provenance, build, compatibilite, curations,
  taches executables;
- couche web: solutions externes, docs, repos publics, articles, benchmarks;
- synthese: comparaison explicite des niveaux de preuve et actions possibles.

Regle non negociable:

```text
Fusionner le ranking est acceptable. Fusionner la confiance ne l'est pas.
```

Chaque resultat doit garder un label de preuve:

| Label | Sens |
|---|---|
| `SBFB verified` | provenance + artifact + verification locale OK |
| `SBFB unverified` | vu sur le reseau mais preuve incomplete |
| `SBFB stale` | source ou release obsoletes |
| `Web external` | source externe, non verifiee par SBFB |
| `Web claim` | affirmation web non prouvee |
| `Verified by workers` | verification active executee par workers |

Exemple de synthese produit:

```text
Tesseract est plus mature sur le web, mais l'app SBFB X est verifiee,
installable localement, compatible avec ta machine, et auditable par 3 workers.
```

---

## 6. Recherche developpement

Le mode `@dev` doit servir aux constructeurs, maintainers, curateurs et groupes
de recherche.

Il cherche dans:

- code source;
- manifests `SBFB.json`;
- capabilities;
- provenance;
- schemas;
- logs publics;
- tests;
- erreurs;
- snippets reutilisables;
- docs techniques;
- lineage/forks.

Questions utiles:

```text
@dev @Babel ou est geree la validation des chunks ?
@dev @network trouve un pattern de sync offline reutilisable
@dev @Babel @web compare notre pipeline OCR avec les pratiques open source
```

Le resultat doit citer:

- fichier;
- lignes ou byte range;
- commit;
- artifact hash;
- provenance hash;
- niveau de preuve;
- risque connu;
- action possible.

---

## 7. Compute: deux produits a separer

Il faut separer deux lignes produit.

### 7.1 Groupes prives de compute

Produit:

```text
Un groupe ferme invite ses machines et partage CPU/GPU pour recherche,
coding, traduction, audit, benchmarks, indexation ou generation, sans publier
les resultats sur le reseau public tant qu'il ne le decide pas.
```

Cas d'usage:

- recherche code confidentielle;
- audit interne avant publication;
- generation d'app en groupe;
- benchmark de modeles;
- traduction privee;
- indexation de corpus interne;
- verification collective avant release.

Primitives necessaires:

- groupe prive avec invite;
- allowlist membres/machines;
- chiffrement tasks/artifacts;
- consentement GPU/CPU;
- quotas par membre;
- logs/audit internes;
- stockage prive;
- publication volontaire vers le reseau public;
- separation stricte entre resultats prives et public feed.

MVP raisonnable:

```text
Private compute group = batch tasks + artifacts chiffres + allowlist + quotas.
```

### 7.2 Compute public app-driven

Produit:

```text
Une app publique peut demander de la puissance au reseau. Les workers acceptent
selon leurs regles de consentement.
```

Cas Babel:

- traduire des chunks en continu;
- valider automatiquement;
- proposer aux humains les chunks incertains;
- re-traduire quand un modele s'ameliore;
- produire des attestations de contribution;
- scorer qualite et provenance.

Regles worker:

- modele autorise;
- VRAM max;
- watts max;
- horaires;
- quotas;
- langues acceptees;
- type de tache;
- publication ou non du resultat;
- niveau de kudos/recompense.

MVP raisonnable:

```text
App-driven batch compute = une app publie des taches paralleles, les workers
opt-in les executent, les resultats sont signes et verifiables.
```

---

## 8. Gros modeles locaux et partage de puissance

Le besoin utilisateur est valide: un groupe peut vouloir s'allier pour utiliser
des gros modeles locaux ou faire de la recherche coding en clos.

Mais il faut distinguer:

### 8.1 Batch distribue verifiable

Bon MVP.

Exemples:

- traduire 10 000 chunks Babel;
- indexer 100 repos;
- lancer 500 tests;
- generer 20 variantes;
- auditer 30 fichiers;
- comparer plusieurs modeles;
- produire embeddings par shard.

Pourquoi c'est adapte:

- parallele naturellement;
- tolerant a la latence;
- verifiable par hash/signature/quorum;
- compatible consentement worker;
- utile pour Babel et coding research.

### 8.2 Inference distribuee temps reel d'un seul gros modele

Non MVP.

Problemes:

- latence reseau;
- memoire distribuee;
- orchestration tensor/pipeline parallel;
- confidentialite;
- drivers/GPU heterogenes;
- cout debug tres eleve;
- faible valeur avant d'avoir batch compute stable.

Decision recommandee:

```text
S'orienter d'abord vers batch distribue verifiable. Garder l'inference
distribuee temps reel comme recherche long terme.
```

---

## 9. Sequence produit apres S66

Cette sequence ne remplace pas un vrai kickoff. Elle traduit la recherche RRV
en sprints probables.

| Sprint cible | Theme produit | Resultat utilisateur |
|---|---|---|
| S67 | RRV Spec + LocalOnly MVP | chercher dans l'app courante ou Babel avec citations et preuves |
| S68 | Ranking hybride + proof cards | resultats classes par pertinence + preuve + disponibilite |
| S69 | Symbols + capabilities + mode dev | chercher des APIs, patterns, capabilities, risques |
| S70 | SearchManifest + shards opt-in | decouvrir des resultats verifies au-dela du local |
| S71 | Recherche active par workers | demander au reseau de verifier/auditer un resultat |
| S72 | Generation Composee | utiliser RRV pour choisir les briques d'une nouvelle app/workflow |
| S73+ | Private compute groups | groupes fermes pour recherche/coding avant publication |
| S74+ | App-driven compute public | apps comme Babel demandent du compute continu au reseau |

Note: si S66 livre deja une partie `SearchManifestPublished`, S70 devient un
sprint de solidification et d'UX plutot qu'un sprint de creation.

---

## 10. Non-goals initiaux

Ne pas commencer par:

- global network search par defaut;
- crawling web automatique;
- active worker verification pour chaque requete;
- inference distribuee temps reel multi-GPU;
- ranking reputation complexe;
- publication automatique de resultats prives;
- melange silencieux entre web externe et preuves SBFB.

---

## 11. Questions ouvertes

1. Le tag de perimetre doit-il etre tape (`@Babel`) ou selectionne via UI ?
2. `@web` doit-il etre disponible par defaut ou derriere un consentement ?
3. Quel label exact pour une source web non verifiee ?
4. Quel niveau de preuve minimal pour apparaitre dans `@network` ?
5. Les groupes prives doivent-ils utiliser le public feed avec chiffrement ou un feed separe ?
6. Comment publier volontairement un resultat prive vers le reseau public ?
7. Quel seuil de consentement GPU pour Babel en continu ?
8. Comment eviter qu'une app abuse du compute public ?
9. Quel modele de kudos/recompense pour les workers Babel ?
10. Quelle limite initiale pour `search_verify` afin d'eviter le spam ?

---

## 12. Verdict court

RRV doit commencer par une recherche locale verifiable et scoped. Les tags
`@project`, `@network`, `@web`, `@dev` et `@private:<group>` donnent a
l'utilisateur le controle du perimetre. Le croisement `@network @web` est utile
a condition de garder les niveaux de preuve separes. Le compute partage doit
commencer par des taches batch verifiables, puis seulement plus tard explorer
l'inference distribuee temps reel.

---

## 13. Lien avec SBFB Project Factory

Complement dedie:

- `.planning/research/sbfb_project_factory_rrv_oss_research.md`

Decision ajoutee apres recherche OSS/architecture:

```text
Ne pas attendre le RRV complet pour demarrer Project Factory.
Demarrer Project Factory avec un noyau RRV @dev LocalOnly.
Garder Project Factory hors du protocole metier: repo/app separee,
connectee aux primitives generiques SBFB.
Garder Babel comme repo applicatif cree/dogfoode par Factory.
```

Rationale:

- Project Factory donne un cas d'usage concret a RRV;
- RRV local donne a Project Factory des citations, preuves et actions;
- Babel donne un dogfood non trivial;
- le protocole reste neutre;
- `@network`, `SearchManifest` et verification workers restent des etapes
  ulterieures, pas des preconditions du noyau local.
