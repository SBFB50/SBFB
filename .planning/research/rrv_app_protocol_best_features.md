# RRV app protocole - meilleures fonctionnalites

**Date:** 2026-05-22
**Statut:** recherche produit, repo-grounded
**Auteur:** Codex + team d'analyse parallelisee
**Scope:** RRV comme app du protocole, acces `@protocole`, `@dev`, `@web`, code front/back/db des apps connectees, relation avec Factory et Babel
**Non-goal:** transformer Factory en protocole, coder Babel comme livrable final, exposer des donnees privees sans contrat explicite

**Update 2026-05-22:** S70 doit d'abord livrer `Process Portable Complete`.
Les fonctions ci-dessous restent le cap RRV, mais `@dev`, process assistant,
OSS seed, `sbfb-search` et provider router deviennent consommateurs du process
portable apres S70, pas des autorites paralleles ni des livrables S70 par
defaut.

## 1. These

RRV doit etre une app protocolaire first-class: l'interface qui permet de
naviguer le reseau, poser des questions, inspecter les preuves et lancer des
actions bornees sur `@protocole`, `@dev`, `@web`, `@current` et plus tard
`@private:<group>`.

La promesse n'est pas "un Google + chat". La promesse est:

1. trouver un artefact du protocole;
2. expliquer d'ou il vient;
3. montrer ce qui est verifie, deduit ou seulement lu;
4. proposer une action executable avec capability, cout, portee et rollback;
5. conserver les labels de confiance au lieu de les fusionner.

RRV est une app du protocole. SBFB reste le protocole. Factory reste une app
cliente/developpeur qui fabrique ou valide des apps. Babel reste un projet
annexe cree par Factory et utilise comme dogfood.

## 2. Cadrage des roles

| Element | Role correct | Ne doit pas devenir |
| --- | --- | --- |
| SBFB / Nexus | Protocole, daemon, stockage, feed, provenance, browse, bridge, primitives generiques | Produit metier Babel ou Factory |
| RRV | App protocolaire de navigation, question, preuve, action | Simple page search, crawler implicite, source unique de verite |
| Factory | App/outil client externe pour creer, valider, publier et auditer des apps | Runtime obligatoire du protocole, moteur cache du process |
| Babel | Premiere app dogfood creee avec Factory | Objectif final du protocole, sprint decouple du dogfood |
| Process agent | Pack repo-visible reutilisable dans les apps generees | Memoire cachee dans Claude/Codex |

Source locale principale: `.planning/research/SYNTHESIS_factory_rrv_protocol.md`
dit deja que SBFB est le protocole, RRV le moteur, Factory l'atelier, Babel le
premier dogfood. La roadmap v4 fige aussi l'ordre `@protocole` puis `@dev` puis
`@web`.

## 3. Etat repo actuel

### Ce qui existe deja

Le repo a deja assez de primitives pour un MVP RRV `@protocole`:

- FTS5 local dans `crates/nexus-coordinator-rs/src/search.rs`.
- Table `search_index` dans `crates/nexus-coordinator-rs/src/db.rs`.
- Feed public append-only dans `public_feed`.
- Provenance dans `provenance_records`.
- Stockage app namespace dans `app_storage`.
- Routes daemon:
  - `/api/daemon/search`;
  - `/api/daemon/proof-card/{project_id}`;
  - `/api/daemon/feed/entries`;
  - `/api/v1/project/{project_id}/provenance`;
  - `/api/daemon/browse`.
- Bridge app connectee:
  - `browse_list`;
  - `provenance_get`;
  - `provenance_verify`;
  - `feed_cursor_get`;
  - `search`;
  - `proof_card_get`;
  - storage;
  - task submit.
- ProofCard locale deterministe dans
  `crates/nexus-coordinator-rs/src/proof_card.rs`.
- UI ProofCard dans `web/src/components/ProofCard.tsx`.

Cela signifie que RRV peut deja etre dogfoode comme app iframe ou page shell
qui utilise les routes et le bridge existants.

### Ce qui n'existe pas encore

RRV complet n'existe pas encore:

- pas de vraie app RRV dans `web/src/pages` avec scopes, question, resultats,
  actions et preuves;
- pas d'index `@dev` avec fichier, ligne, symbole, hash, commit, licence,
  schema DB ou flux front/back;
- pas d'acces backend/db multi-app via contrat explicite;
- pas de `SearchManifest` reseau operationnel;
- pas de sidecar `@web` avec consentement et labels separes;
- pas de citations exactes dans les resultats search actuels;
- pas de matrice canon `scope -> sources -> APIs -> labels -> actions`.

Conclusion: le bon livrable court terme n'est pas "inventer tout RRV". C'est
construire la surface RRV sur les primitives `@protocole`, puis ajouter `@dev`
et `@web` par couches separees.

## 4. Modele d'acces aux apps connectees

La phrase "RRV a acces au code frontend/backend/base de donnees des apps
connectees" doit etre vraie par contrat, pas par intrusion.

### 4.1 Frontend

RRV peut inspecter le frontend quand l'app publie:

- une archive statique servie par le protocole;
- un `SBFB.json` ou manifest de source;
- un repo URL + commit + provenance hash;
- un source pack verifiable;
- des assets publics.

RRV peut indexer:

- `index.html`;
- JS/TS/CSS bundles ou sources;
- manifest;
- README/docs;
- permissions bridge;
- composants, routes, storage keys declarees;
- chaines UI si elles sont dans l'archive ou le source pack.

### 4.2 Backend

RRV ne doit pas supposer qu'un backend existe ou qu'il est lisible. Il peut
indexer le backend seulement si l'app fournit un contrat:

- repo public ou source pack;
- service manifest;
- OpenAPI/JSON schema;
- routes declarees;
- logs publics/expurges;
- proof de build;
- commit et hash d'artefact.

Sans ce contrat, RRV peut seulement dire: "backend non publie" ou "backend non
verifiable".

### 4.3 Base de donnees et storage

RRV ne doit jamais lire les donnees privees multi-app par defaut. Il peut
indexer:

- schemas publies;
- migrations publiees;
- clefs storage declarees;
- types de donnees;
- statistiques anonymisees;
- donnees publiques de feed/provenance;
- exemples fixtures;
- exports consentis par l'app ou par l'utilisateur.

Les lignes de donnees, secrets, tokens, messages prives, PII et donnees
chiffrees restent hors index par defaut. Le mode `@private:<group>` devra etre
un contrat explicite, chiffrable, auditable et revocable.

### 4.4 Process et projet

RRV doit aussi pouvoir lire le process des apps creees par Factory:

- `.planning/active/*`;
- `.planning/research/*`;
- `docs/agent/PROCESS.md`;
- `prompts/agent/*`;
- `scripts/agent/agentctl.py`;
- `.githooks/*`;
- rapports preflight/review/verification;
- commit body et evidence de tests.

C'est la difference entre "une app generee" et "un projet verifiable": le code
et le process arrivent ensemble.

## 5. Scopes RRV

| Scope | Sources | Usage principal | Maturite |
| --- | --- | --- | --- |
| `@current` | App ouverte, storage autorise, etat UI, erreurs locales | Diagnostiquer ce que l'utilisateur regarde | A specifier |
| `@protocole` / `@network` | browse, feed, provenance, curations, archives, tasks/kudos | Trouver et prouver les apps du reseau | Primitives deja la |
| `@dev` | code, manifests, schemas, tests, process, symbols, capabilities | Aider devs, Factory, audits, forks | Post-Gate 1 |
| `@web` | sources web externes via sidecar | Comparer au monde externe | Post-pilote, opt-in |
| `@private:<group>` | corpus et compute de groupe | Recherche privee, labo, clients | Futur |
| `@Babel`, `@OCR-App` | alias projet/app nommee | Focus sur une app precise | UI/spec |

Regles UX:

- le scope visible est obligatoire dans chaque resultat;
- `@current` est le defaut si une app est ouverte;
- `@protocole` est le defaut sinon;
- `@web` reste off par defaut;
- RRV peut proposer d'elargir le scope mais ne doit pas le faire
  silencieusement;
- une synthese peut croiser plusieurs scopes, mais chaque phrase conserve sa
  provenance.

## 6. Format de reponse RRV

RRV doit repondre avec une structure stable:

1. **Reponse courte:** ce que le systeme pense etre la bonne reponse.
2. **Preuves:** artefacts cites, hashes, fichiers, lignes, feed entries,
   provenance, signatures, freshness.
3. **Limites:** ce qui manque, ce qui est deduit, ce qui n'est pas verifie.
4. **Actions:** ouvrir, verifier, auditer, fork, publier, demander au reseau,
   generer via Factory.

Chaque claim doit etre classee:

| Classe | Sens | Exemple |
| --- | --- | --- |
| Lu | Directement present dans un fichier, feed, schema, manifest, archive | `SBFB.json` declare `capabilities.storage` |
| Deduit | Resultat d'analyse locale ou rapprochement de sources | "ce flow semble ecrire dans app storage" |
| Verifie | Signature, hash, test, build, provenance ou quorum valide | "provenance hash correspond a l'archive" |
| Non verifie | Source externe ou absence de preuve | "claim web non confirmee par SBFB" |

## 7. Meilleures fonctionnalites

### F1. Barre question-action scoped

Une entree unique, mais pas magique:

```text
@protocole trouve les apps de lecture avec provenance complete
@dev @Babel ou est geree la progression de lecture ?
@web compare Babel avec les readers open source existants
@current pourquoi cette app n'arrive pas a verifier sa provenance ?
```

L'UI doit afficher chips de scope, filtres, type de source, et etat de
confiance.

### F2. Resultats proof-first

Chaque resultat montre:

- nom/projet;
- scope;
- source type;
- score de pertinence;
- score ProofCard;
- etat `verified`, `unverified`, `stale`, `source-only`, `web external`;
- raison courte;
- dernier hash connu;
- actions immediates.

Le score de recherche et le score de preuve ne doivent pas etre melanges.

### F3. Fiche app/projet

Une fiche RRV par app:

- identite projet;
- archive et hash;
- feed timeline;
- provenance;
- curations et desaveux;
- repo/source pack;
- capabilities bridge;
- storage keys declarees;
- schemas declares;
- risques;
- actions possibles;
- historique Factory si l'app a ete creee par Factory.

Cette fiche devient le "dossier verifiable" d'une app.

### F4. Timeline feed lisible

RRV doit transformer le feed en histoire comprehensible:

- release publiee;
- source devenue stale;
- curator vouched;
- curator disendorsed;
- provenance ajoutee;
- SearchManifest publie plus tard;
- audit worker plus tard.

Ce n'est pas seulement un log: c'est la memoire verifiable de l'app.

### F5. Inspecteur de provenance

Action centrale: "Verifier".

L'inspecteur doit expliquer:

- quelle signature est verifiee;
- quel hash est compare;
- quelle archive est concernee;
- quel repo/commit est lie;
- pourquoi la preuve est complete ou incomplete;
- quels risques restent.

Il doit aussi rendre visibles les cas faibles: pas de provenance, source stale,
hash absent, repo non public, release non verifiee.

### F6. Explorateur `@dev LocalOnly`

Post-Gate 1, RRV doit indexer le code des apps connectees quand le contrat le
permet:

- fichiers et lignes;
- symboles;
- composants front;
- endpoints back;
- schemas DB;
- migrations;
- tests;
- manifests;
- permissions bridge;
- appels storage/task/feed/provenance;
- TODO/risques;
- licences;
- secret scan status.

Questions cibles:

```text
@dev @Babel ou est stockee la progression utilisateur ?
@dev @network quelles apps utilisent task_submit ?
@dev @Babel explique le flux front -> storage -> provenance
@dev trouve les apps qui declarent open source mais sans provenance hash
```

### F7. Explorateur DB/storage avec consentement

RRV doit distinguer trois niveaux:

| Niveau | Autorise par defaut | Contenu |
| --- | --- | --- |
| Schema | Oui si publie | tables, clefs, types, migrations |
| Metadata | Oui si publie ou local | compteurs, freshness, namespaces, taille |
| Donnees | Non | lignes, valeurs, messages, PII, secrets |

L'action "inspecter donnees" doit etre explicite, scopee, journalisee et
revocable.

### F8. Actions typees

RRV doit etre un navigateur d'actions, pas seulement de liens.

Actions de lecture:

- ouvrir app;
- ouvrir fiche preuve;
- ouvrir source;
- copier citation;
- voir feed entry.

Actions de verification:

- verifier provenance;
- recalculer hash;
- relancer index local;
- verifier source stale;
- demander audit worker.

Actions de creation:

- fork via Factory;
- creer app derivee;
- extraire template;
- generer diff;
- ajouter process pack.

Actions reseau:

- publier;
- demander quorum;
- partager SearchManifest;
- rejoindre groupe prive;
- contribuer compute batch.

Chaque action declare:

- capability requise;
- effet local/reseau;
- cout compute ou reseau;
- donnees envoyees;
- rollback possible;
- preuve produite.

### F9. Cross-source composer sans fusion du trust

RRV peut comparer `@protocole`, `@dev` et `@web`, mais jamais ecrire une
synthese qui upgrade un fait externe en preuve protocolaire.

Format recommande:

| Colonne | Contenu |
| --- | --- |
| Protocole | feed, browse, provenance, proof-card |
| Code | fichiers, lignes, commits, schemas |
| Web | liens externes, claims, docs, comparateurs |
| Synthese | conclusion avec conflits visibles |

Labels minimaux:

- `Local indexed`;
- `Local generated`;
- `Local tested`;
- `SBFB verified`;
- `SBFB unverified`;
- `SBFB stale`;
- `External OSS source index`;
- `Web external`;
- `Web claim`;
- `Verified by workers`.

### F10. SearchManifest opt-in

SearchManifest doit permettre au reseau d'annoncer ce qu'un noeud sait
rechercher, sans envoyer les requetes utilisateur.

Regles:

- opt-in strict;
- signature obligatoire;
- taille limitee;
- rate-limit;
- expiration;
- privacy notice;
- pas de private data;
- resultat distant marque comme annonce, pas comme preuve;
- verification locale ensuite via feed/provenance/hash.

### F11. Worker verification et compute batch

Le compute reseau doit commencer par du batch verifiable:

- scan licence;
- scan secret;
- build/test;
- audit source;
- indexation code;
- traduction de chunks Babel;
- verification provenance;
- extraction de schemas.

RRV affiche:

- qui a verifie;
- quel worker/quorum;
- quels inputs;
- quels outputs;
- quels hashes;
- quel cout;
- quel niveau de confiance.

Pas de promesse d'inference distribuee temps reel tant que le batch verifiable
n'est pas robuste.

### F12. Process assistant pour Factory et apps generees

Factory doit pouvoir reutiliser le process actuel comme pack:

- `docs/agent/PROCESS.md`;
- `docs/agent/TOOLING.md`;
- `prompts/agent/*.md`;
- `scripts/agent/agentctl.py`;
- `.githooks/pre-commit`;
- `.githooks/commit-msg`;
- templates preflight/review/commit body;
- `.planning/active/sprint0_*`;
- `.planning/research/*`.

S70 doit d'abord rendre ce pack portable et verifiable via
`process_portable_complete_s70.md`: `AGENT_SYSTEM`, `handoff`, `agentctl
status-sprint`, `lint-planning`, `audit-commit`, gates/hooks/CI. Factory ne
doit pas packager un process encore implicite dans Claude/Codex.

RRV doit ensuite pouvoir lire ce process dans une app creee par Factory:

- sprint courant;
- phases;
- preflight G8;
- review PASS/PASS-PENDING;
- verification;
- audit plan;
- risques;
- decisions fossil/live/canon.

Cela donne une fonctionnalite forte: "montre-moi pourquoi cette app a ete
creee proprement", pas seulement "montre-moi son code".

### F13. Generation composee proof-aware

Quand Factory creera une app a partir de briques existantes, RRV doit choisir
les briques selon:

- preuve;
- licence;
- usage;
- freshness;
- diversite;
- test coverage;
- compatibilite capabilities;
- risques;
- provenance;
- historique de bugs.

RRV ne doit pas seulement trouver "le meilleur code". Il doit trouver "la
meilleure brique reutilisable avec preuve et cout de risque explicite".

### F14. Dossier Babel comme canari

Tant que Babel est la premiere app, RRV doit avoir un dossier Babel:

- app creee via Factory;
- template utilise;
- provenance Factory;
- archive publiee;
- Browse visible;
- search `babel` retourne l'app;
- ProofCard affichee;
- source pack si disponible;
- storage progression si declaree;
- feed timeline;
- gaps et actions.

Babel n'est pas le produit final. C'est la preuve que la boucle
Factory -> app -> protocole -> RRV -> preuve fonctionne.

## 8. Architecture cible

```text
Utilisateur
  |
  v
RRV App
  - question/action UI
  - scopes
  - resultats proof-first
  - dossiers apps
  - actions bornees
  |
  +-- Bridge app connectee
  |     - search
  |     - proof_card_get
  |     - browse_list
  |     - provenance_get/verify
  |     - storage consented
  |     - task_submit
  |
  +-- RRV Index Service
  |     - @protocole FTS5
  |     - @dev LocalOnly source index
  |     - schemas/capabilities/process index
  |     - SearchManifest opt-in
  |
  +-- Daemon/protocole SBFB
  |     - blobs
  |     - feed
  |     - provenance
  |     - browse
  |     - storage
  |     - task/worker
  |
  +-- Factory app
  |     - create
  |     - validate
  |     - preview
  |     - publish
  |     - process pack
  |
  +-- Apps connectees
        - Babel
        - futures apps
        - source packs
        - manifests
        - schemas publics
```

## 9. Phasage recommande

### R0 - Spec produit et matrice

Livrable:

- cette doc;
- matrice `scope -> sources -> APIs -> labels -> actions`;
- definition du format de reponse;
- threat model RRV;
- liste de tests acceptance.

### R1 - MVP `@protocole` local

Livrable:

- app/page RRV avec barre question-action;
- chips de scope;
- appel `/api/daemon/search`;
- ProofCard sur resultat;
- actions ouvrir/verifier/voir feed;
- empty states et erreurs;
- tests UI + endpoint.

Ce niveau exploite les primitives deja presentes.

### R2 - Dossier app et citations protocole

Livrable:

- fiche app/projet;
- timeline feed;
- provenance explainer;
- curations;
- citations hashes/entries;
- ajout de `entry_hash`, `provenance_hash`, `archive_hash` ou references
  equivalentes dans les resultats search quand disponible.

### R3 - `@dev LocalOnly`

Livrable:

- index source local;
- file/line/hash;
- symboles et capabilities;
- schemas;
- manifest/bridge analyzer;
- process pack analyzer;
- secret/licence risk flags.

Ce niveau est post-Gate 1 par defaut.

### R4 - Acces DB/storage sous contrat

Livrable:

- schema explorer;
- storage namespace explorer;
- consent UI;
- redaction;
- audit log;
- no-private-data-by-default tests.

### R5 - `@web` sidecar

Livrable:

- consentement;
- web labels separes;
- comparaison externe;
- aucune promotion en `SBFB verified`;
- cache local optionnel;
- logs de sources.

### R6 - SearchManifest reseau

Livrable:

- format signe;
- publication opt-in;
- decouverte;
- expiration;
- rate-limit;
- verification tamper/oversize;
- UI "annonce distante" vs "preuve locale".

### R7 - Worker verification

Livrable:

- audit worker;
- build/test worker;
- scan worker;
- quorum;
- proof outputs;
- cout/quota;
- batch first.

### R8 - Factory + generation composee

Livrable:

- Factory consomme RRV `@dev`;
- selection de briques proof-aware;
- process pack injecte;
- Babel ou app derivee creee avec dossier RRV complet.

## 10. Implication Sprint 69

S69 ne doit pas devenir "construire RRV complet".

Le bon recadrage S69:

- Factory livre le flow dogfood Babel;
- Babel est creee par Factory, pas codee comme produit final;
- les outputs Babel/Factory doivent etre indexables plus tard par RRV;
- metadata `SBFB.json` et provenance doivent etre propres;
- ProofCard et search `@protocole` prouvent Babel;
- process pack doit etre assez clair pour etre reutilise;
- `@dev`, `@web`, SearchManifest et app RRV complete restent S71+ ou
  post-pilote selon la roadmap, apres S70 Process Portable Complete.

S69 peut donc preparer RRV sans absorber RRV.

## 11. Tests d'acceptance RRV

### Local `@protocole`

- search `babel` retourne Babel apres publication;
- chaque resultat affiche scope et source type;
- chaque resultat peut ouvrir une ProofCard;
- ProofCard preserve les labels de preuve;
- feed timeline affiche au moins release/provenance/curation si presents;
- resultat absent donne un empty state actionnable;
- endpoint search limite pagination/offset/limit.

### Citations et preuves

- un resultat cite `entry_hash` ou artefact equivalent quand disponible;
- provenance verify affiche success/failure et raison;
- source stale visible;
- no provenance visible;
- score de recherche et score de preuve separes.

### `@dev`

- fichier/ligne/hash presents pour resultats code;
- symboles indexables;
- manifest capabilities indexees;
- schema DB public indexe;
- secret scan bloque ou marque les secrets;
- code prive non indexe sans opt-in;
- licence affichee.

### DB/storage

- schema public lisible;
- donnees privees non lisibles par defaut;
- consent requis pour valeurs;
- audit log ecrit;
- revocation retire l'acces;
- redaction testee.

### `@web`

- off par defaut;
- demande consentement;
- resultats labels `Web external` ou `Web claim`;
- aucune claim web ne devient `SBFB verified`;
- sources externes citees.

### SearchManifest

- manifest signe accepte;
- manifest tampered rejete;
- manifest oversized rejete;
- manifest expire ignore;
- publication rate-limitee;
- requete utilisateur jamais publiee;
- resultat distant marque comme annonce.

### Actions

- action lecture ne demande pas capability inutile;
- action publication confirme portee reseau;
- action worker affiche cout;
- action destructive demande confirmation;
- rollback indique si impossible.

## 12. Risques majeurs

| Risque | Impact | Mitigation |
| --- | --- | --- |
| RRV vendu trop tot comme moteur reseau global | Produit mensonger | LocalOnly + labels de maturite |
| Confusion `@web` / SBFB verified | Trust casse | Labels separes, jamais de fusion |
| Fuite `@dev` ou DB | Donnees privees exposees | Opt-in, redaction, secret scan, schema-only default |
| SearchManifest privacy leak | Revele corpus/capacites | Opt-in, expiration, taille limitee |
| Score gaming | Faux sentiment de securite | Afficher formule, facteurs, risques |
| Actions dangereuses | Publication/compute non voulu | Capability, confirmation, cout, rollback |
| Docs fossiles indexees comme verite | Mauvais phasage | Research registry `live/canon/candidate/fossil` |
| Factory devient protocole | Couplage long terme | Factory app externe, primitives daemon generiques |

## 13. Docs a creer ou mettre a jour

Recommande:

- `docs/product/RRV_PRODUCT.md`: principes, scopes, UX, format de reponse,
  actions, non-goals.
- `docs/protocol/RRV_SEARCH.md`: schema search, citations, index lifecycle,
  SearchManifest.
- `docs/security/THREAT_MODEL.md`: section RRV, `@dev`, `@web`,
  SearchManifest, actions executables.
- `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md`: inclure search et
  proof-card.
- `docs/protocol/SBFB_JSON_V2.md`: clarifier source-only/source-index futur.
- `docs/apps/CHAT_IA_RESEAU.md`: reclasser en vision post-Gate ou reecrire en
  RRV proof-first.
- `docs/apps/GENERATION_COMPOSEE.md`: reclasser apres RRV `@dev` et Factory.

## 14. Sources repo

Sources structurantes:

- `.planning/roadmap_v4_neutral_protocol_factory_rrv.md`: Factory externe,
  Babel dogfood, ordre `@protocole -> @dev -> @web`, Gate 1 sans `@dev`.
- `.planning/research/SYNTHESIS_factory_rrv_protocol.md`: synthese canonique
  Factory/RRV/protocole, scopes, SearchManifest, tests.
- `.planning/research/rrv_scoped_search_compute_groups.md`: scopes UX,
  croisement sources, groupes prives, compute batch.
- `.planning/research/s70_s72_rrv_research.md`: recherche RRV LocalOnly,
  Proof Cards, SearchManifest.
- `docs/apps/CHAT_IA_RESEAU.md`: vision historique chat IA reseau et acces aux
  sources.
- `docs/apps/GENERATION_COMPOSEE.md`: vision composition d'apps par briques.
- `docs/protocol/SBFB_JSON_V2.md`: limite actuelle sur source-only/source-index.
- `docs/protocol/PUBLIC_FEED_SPEC.md`: contraintes `is_open_source` et
  `provenance_hash`.
- `crates/nexus-coordinator-rs/src/search.rs`: search FTS5 actuel.
- `crates/nexus-coordinator-rs/src/proof_card.rs`: ProofCard.
- `crates/nexus-coordinator-rs/src/db.rs`: storage, feed, provenance,
  search_index.
- `crates/nexus-shell-daemon/src/http.rs`: endpoints daemon.
- `web/src/bridge/protocol.ts`: methodes bridge declarees.
- `web/src/bridge/useBridge.ts`: mapping bridge vers endpoints.
- `web/public/sbfb-bridge.js`: API JS exposee aux apps.
- `web/src/components/ProofCard.tsx`: UI ProofCard.

Sources externes non normatives, seulement pour comparer les attentes produit
autour du codebase-aware assistant:

- Sourcegraph Cody docs: https://sourcegraph.com/docs/cody
- VS Code Copilot workspace context: https://code.visualstudio.com/docs/copilot/workspace-context

## 15. Verdict

RRV doit devenir le navigateur decisionnel du protocole:

```text
Qu'est-ce que le protocole sait ?
D'ou vient l'information ?
Est-ce verifie, deduit, lu ou externe ?
Quel risque reste ?
Quelle action sure puis-je lancer maintenant ?
```

Le meilleur produit n'est donc pas une page de recherche. C'est une app qui
relie recherche, preuve, code, process, storage, reseau et actions, avec une
frontiere claire entre `@protocole`, `@dev`, `@web`, Factory et Babel.
