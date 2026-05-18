# Research: Factory + Babel Canari dans la roadmap S65-S75

Date: 2026-05-18

Statut: CANON CANDIDATE / research d'arbitrage PO.

Objet: integrer la decision PO suivante dans la roadmap: Babel ne doit pas
etre seulement un signal test externe ou un bloc repousse a S75. Babel doit
etre la premiere app canari generee par la Factory, et la Factory doit elle
aussi produire des artefacts de sprint comme SBFB.

Ce document ne remplace pas encore
`.planning/roadmap_v2_public_trust_rrv_factory.md`. Il fournit la justification
produit, architecture, gates et sequence sprint pour corriger cette roadmap.

## 0. Verdict court

La correction PO est valide.

L'ancienne sequence "RRV puis Factory puis Babel" est trop abstraite. Elle
repousse le premier vrai dogfood applicatif alors que le risque central de
Factory est justement de savoir si elle peut produire, verifier, publier et
faire vivre une app reelle.

La sequence corrigee est:

1. S65-S66 restent obligatoires pour la confiance et la durabilite.
2. Factory devient un fil rouge des S65/S66, puis un livrable concret en S67.
3. Babel Reader devient la premiere app canari generee par Factory en S69.
4. RRV observe, indexe et explique les preuves Factory/Babel a partir de S70.
5. S73-S75 ne sont plus "premiere creation de Factory"; ils deviennent
   industrialisation, durcissement, second souffle Babel et release proof.

Le point de produit cle: ne pas construire "Babel complet" maintenant. Construire
un canari Babel Reader qui force Factory a prouver:

- generation d'un repo app;
- artefacts de sprint;
- manifest app portable;
- bridge method allowlist;
- preview sandboxee;
- publish gate;
- provenance;
- feed;
- Browse;
- verification par l'utilisateur.

## 1. Decision PO

Decision:

> Factory est l'atelier generique. Babel est le premier dogfood app. RRV est
> le moteur de decouverte et preuve qui inspecte ce que Factory produit.

Implications:

- Babel ne doit pas etre code a la main hors Factory.
- Factory ne doit pas etre une app iframe avec pouvoirs speciaux.
- Le protocole SBFB ne doit pas connaitre de logique metier `babel_*`.
- RRV ne doit pas etre le pre-requis qui bloque Factory.
- Le go-live public reste repousse; le canari est d'abord local/pilote ferme.

## 2. Pourquoi changer la roadmap

La roadmap V2 actuelle contient une bonne intuition mais une mauvaise
sequence. Elle dit deja que Factory est un broker/module daemon, pas une app
iframe, et que Babel peut etre un MVP leger avec fixtures. Mais elle place
Factory/Babel en S73-S75 apres RRV complet, ce qui cree trois problemes:

1. La Factory reste theorique trop longtemps.
2. RRV manque d'un objet reel riche a indexer.
3. Babel est traite comme "grande app future" au lieu de "preuve produit".

La correction rend le risque visible plus tot. Si Factory ne sait pas generer
Babel Reader proprement, il vaut mieux le savoir avant de promettre une
plateforme d'apps verifiables.

## 3. Frontieres de responsabilite

### SBFB / Nexus protocole

Responsabilites:

- identite locale;
- storage applicatif;
- bridge sandboxe;
- provenance generique;
- publish path;
- public feed append-only;
- Browse/catalogue;
- verification crypto.

Non-responsabilites:

- workflow metier Babel;
- prompts Factory;
- execution shell depuis iframe;
- methode protocole `babel_translate`;
- methode protocole `factory_shell_exec`.

### Factory

Responsabilites:

- UI shell `/factory`;
- broker local privilegie cote daemon;
- generation de repos apps;
- templates versionnes;
- domain packs;
- preview sandboxee;
- diff review;
- publish check;
- artefacts de sprint;
- provenance Factory locale;
- evidence pack app.

Factory peut etre dans le workspace `nexus` pour le MVP, mais elle doit rester
conceptuellement separee du protocole metier. Extraction en repo sibling
possible plus tard si la surface grandit.

### Babel

Responsabilites:

- app metier de lecture/traduction;
- source manifests;
- fixtures multilingues;
- progression/bookmarks;
- reviews minimales;
- provenance visible;
- dogfood de `task_submit` si backend pret.

Babel est une app du protocole, pas le protocole.

### RRV

Responsabilites:

- indexer localement les apps, manifests, feed, provenance, proof packs;
- produire des Proof Cards;
- separer `Local generated`, `Local tested`, `SBFB verified`, `SBFB stale`;
- publier des SearchManifests uniquement opt-in.

RRV ne "fait pas confiance a Factory". Il indexe et explique les artefacts que
Factory produit.

## 4. Invariants de confiance

Aucune preuve publique ne doit depasser ce qu'un tiers peut verifier.

Racine de confiance acceptable:

- source publique;
- commit pin;
- hash artefact;
- provenance signee;
- feed signe et chaine;
- wording exact dans l'UI.

Tout le reste reste local, auto-atteste ou canari ferme.

Formulations autorisees:

- "source verifiable";
- "provenance verifiable";
- "commun logiciel sous AGPL-3.0-or-later";
- "Provenance";
- "Signature verifiee" seulement apres verification effective.

Formulations a bannir tant que les gates ne sont pas fermes:

- "badge verifie" pour un simple `provenance_hash`;
- "open source verifie" pour une app;
- "code reseau = code du repo";
- "build reproductible" sans preuve de rebuild tiers;
- "decentralisation totale" tant que seed/bootstrap/allowlist existent;
- "compute correct" pour un worker Babel signe mais non verifie par quorum.

## 5. Etat technique repo utile

### Ce qui existe deja

- `deploy-from-repo` clone un repo public, lit `SBFB.json`, exige
  `index.html`, zippe, calcule un hash, genere une provenance signee et annonce
  dans Browse.
- Le modele publish canonique est "Verified Release =
  `repo_url + commit_sha + artifact_hash + provenance_hash`".
- Le bridge expose deja les primitives utiles au MVP Babel:
  `task_submit`, `storage_get`, `storage_set`, `storage_list`,
  `storage_delete`, `identity_pubkey`, `node_status`, `browse_list`,
  `storage_version`, `provenance_get`, `provenance_verify`,
  `feed_cursor_get`.
- Le public feed sait representer une operation `ReleasePublished`.
- Les recherches existantes cadrent Babel comme app vitrine et Factory comme
  broker local/sandbox, pas comme iframe privilegiee.

### Ce qui manque ou bloque

- `deploy-from-repo` ne cree pas encore automatiquement l'entree public feed
  `ReleasePublished`.
- `SBFB.json.node_id` est encore une contrainte de deploy; elle bloque les
  templates portables et les exemples avec `PLACEHOLDER`.
- `project_id`/Browse semble encore lie au `node_id` du daemon dans certains
  chemins; plusieurs apps publiees par le meme daemon doivent etre clarifiees.
- Le publish path accepte `http` ou `https` alors que le public feed exige
  `https`.
- La durabilite blobs n'est pas encore acceptable pour un pilote externe si le
  store reste volatil.
- La replication storage generique pour une nouvelle app Babel n'est pas encore
  prouvee comme mecanisme P2P complet.
- La roadmap/doc canon n'est pas synchronisee: `CLAUDE.md`, classification docs,
  `SPRINT_LOG.md` et `.planning/active/` sont encore stale.

## 6. Contrat Factory minimal

Factory MVP doit prendre:

- template id + version;
- variables utilisateur;
- domain pack Babel;
- repo cible;
- config publish;
- niveau de risque app.

Factory MVP doit produire:

- `index.html`;
- `SBFB.json`;
- `sbfb-bridge.js` ou SDK bridge copie;
- `README.md`;
- tests de base;
- `.planning/active/sprint01_plan.md`;
- `.planning/active/sprint01_phase_A_review.md`;
- `.planning/active/sprint01_verification.md`;
- `factory.template.lock`;
- `factory.provenance.json`;
- `factory.audit.jsonl`;
- `proof-pack/` ou `evidence/`.

Factory MVP doit garantir:

- generation deterministe;
- aucune ecriture hors workspace autorise;
- diff review avant apply;
- audit JSONL de chaque operation;
- preview sandboxee avant publish;
- refus des methodes bridge inconnues;
- refus symlinks/path traversal/secrets;
- aucun acces shell/FS depuis l'iframe app.

## 7. Manifest app v2

Direction recommandee:

```json
{
  "schema_version": 2,
  "name": "babel-reader",
  "version": "0.1.0",
  "display_name": "Babel Reader",
  "description": "Reader offline multilingue avec provenance source",
  "category": "language",
  "license": "AGPL-3.0-or-later",
  "bridge": {
    "methods": [
      "storage_get",
      "storage_set",
      "storage_list",
      "storage_delete",
      "identity_pubkey",
      "node_status",
      "browse_list",
      "task_submit",
      "provenance_get",
      "provenance_verify",
      "feed_cursor_get"
    ],
    "events": ["storage_update", "task_result_ready"]
  },
  "tech": {
    "type": "static",
    "entry_point": "index.html",
    "build": null
  },
  "requirements": {
    "min_bridge_version": "1",
    "offline": true
  }
}
```

Decision: `node_id` ne doit pas etre une propriete du manifest app.

Raison:

- `node_id` appartient au deployeur;
- l'attribution est deja dans la provenance signee;
- un manifest portable produit un artifact hash reproductible;
- les templates publics ne peuvent pas connaitre le futur deployeur.

Migration:

- rendre `node_id` optionnel/deprecie si present;
- ne plus rejeter si `node_id != state.node_id`;
- logger un warning;
- retirer `node_id` des exemples;
- garder le `node_id` dans `ProvenanceRecord`.

## 8. Babel Reader canari

### Scope MVP

Livrer:

- app `babel-reader` generee par Factory;
- 3 textes domaine public ou fixtures sures;
- environ 5 langues fixtures;
- liste de textes;
- vue lecture;
- toggle langue;
- progression/bookmarks;
- reviews minimales;
- source manifest visible;
- provenance app visible;
- feed cursor visible;
- verification provenance depuis le bridge;
- task traduction mock si backend reel absent.

Ne pas livrer dans le MVP:

- corpus Gutenberg massif;
- ingestion multi-source;
- consensus humain robuste;
- liseuse dediee;
- NLLB live obligatoire;
- protocole metier `babel_translate`;
- publication publique large.

### Storage propose

Namespaces:

- `texts/{text_id}`;
- `translations/{lang}/{text_id}`;
- `bookmarks/{pubkey}/{text_id}`;
- `reviews/{translation_id}/{pubkey}`;
- `manifests/sources/{source_id}`;
- `app/state/{pubkey}`.

### Source manifest minimal

Chaque texte doit avoir:

- `source_id`;
- `source_url` ou origine locale fixture;
- `source_hash`;
- base de droits;
- juridictions;
- redistribution autorisee;
- traduction autorisee;
- licence ou politique de sortie;
- attribution;
- takedown policy;
- date d'import;
- signataire ou auteur du manifest.

Point important: la provenance ne prouve pas les droits. Le rights gate precede
la traduction.

## 9. Publish path canari

Chemin cible:

1. Factory genere Babel en Local Draft.
2. Broker produit un diff.
3. Utilisateur approuve.
4. Preview locale sandboxee.
5. Tests Babel passent.
6. Commit + push repo public.
7. `POST /api/v1/deploy-from-repo`.
8. Daemon clone, valide manifest/index, zippe, hash, signe provenance.
9. Blob archive publie/persiste.
10. Browse entry creee.
11. Public feed `ReleasePublished` cree automatiquement.
12. Evidence pack capture provenance, feed entry, Browse, tests, screenshot.

Gate: si l'etape 11 manque, Babel peut etre un canari Browse/provenance local,
mais pas encore une preuve public feed complete.

## 10. Roadmap S65-S75 revisee

### S65 - Contrat public + contrat Factory/Babel

Objectif produit:

Rendre le langage public exact et definir ce que Factory aura le droit de
generer, afficher et publier.

Livrables:

- fermer `P2-FEED-INSERT-NO-AUTH-TIER`;
- trancher version guard ou raw op policy;
- corriger wording "verifie/open source";
- definir manifest app v2;
- definir artefacts sprint app;
- definir gates Factory G0-G10;
- ajouter ce research dans la canonisation Gate 0.

Gate:

- aucune UI ne sur-promet;
- feed_insert a un auth tier explicite;
- Factory n'a pas encore de pouvoir non cadre;
- `CLAUDE.md` et roadmap pointent vers la meme decision.

### S66 - Durabilite avant app canari

Objectif produit:

Une app publiee ne doit pas disparaitre au restart.

Livrables:

- blob store persistant;
- restart E2E;
- republish DB vers iroh-docs;
- leak feed join corrige;
- evidence restart multi-daemon.

Gate:

- app archive + provenance + feed survivent aux restarts;
- pas de pilote externe tant que ce gate echoue.

### S67 - Factory Foundation / Sprint OS

Objectif produit:

Factory sait creer un repo app minimal avec un process de sprint comme SBFB.

Livrables:

- module/broker Factory minimal;
- template app statique;
- `SBFB.json v2`;
- retrait/deprecation `node_id` manifest;
- `factory.template.lock`;
- `factory.provenance.json`;
- `factory.audit.jsonl`;
- sprint skeleton genere;
- validation manifest/bridge methods.

Gate:

- `sbfb create` ou commande equivalente genere une app statique publiable;
- Explorer/Ideas restent compatibles;
- aucune ecriture hors workspace.

### S68 - Broker, preview, publish gate

Objectif produit:

Factory ne se contente plus de scaffold; elle sait prevenir une publication
fragile.

Livrables:

- UI `/factory`;
- diff preview;
- apply approuve;
- preview sandboxee;
- scan secrets;
- path traversal/symlink deny;
- publish-check;
- proof pack Factory;
- integration deploy-from-repo;
- insertion `ReleasePublished` si possible.

Gate:

- publish refuse manifest invalide, methode bridge inconnue, secret, path
  traversal, symlink, repo non HTTPS;
- deploy roundtrip produit archive + provenance + Browse + feed si le feed est
  cable.

### S69 - Babel Reader canari ferme

Objectif produit:

Babel est la premiere app reelle produite par Factory, testee par 2-3 personnes,
mais sans promesse publique large.

Livrables:

- domain pack Babel;
- app `babel-reader` generee par Factory;
- fixtures multilingues;
- source manifests;
- storage local;
- reviews minimales;
- provenance visible;
- Browse visible;
- pilote ferme.

Gate:

- Babel n'est pas code a la main hors Factory;
- utilisable 24h;
- provenance verifiee;
- no P0/P1;
- feedback testeurs documente.

### S70 - RRV LocalOnly sur corpus reel

Objectif produit:

RRV commence a expliquer ce que Factory et Babel ont produit.

Decision moteur:

FTS5 d'abord. Tantivy devient gate post-S75 si volume, p95 ou features le
justifient.

Livrables:

- index local FTS5;
- index apps Factory;
- index Babel;
- index manifest/provenance/feed;
- recherche locale avec citations;
- labels separes `Local generated`, `Local tested`, `SBFB verified`, `SBFB stale`.

Gate:

- Babel trouvable localement;
- resultats avec path/hash/citation;
- aucun melange silently avec sources web.

### S71 - Proof Cards

Objectif produit:

Chaque resultat app/provenance devient explicable.

Livrables:

- Proof Card Babel;
- Proof Card Factory artifact;
- score de completude deterministe;
- warning si source non reverifiable;
- warning si provenance absente;
- UI proof sans "trust score" social.

Gate:

- un projet sans provenance ne peut pas etre presente comme verifie;
- une Factory generation locale ne devient pas confiance reseau.

### S72 - SearchManifest opt-in

Objectif produit:

Rendre Babel et les apps Factory decouvrables sur le reseau sans publier les
requetes utilisateur.

Livrables:

- `SearchManifest` signe;
- publication opt-in;
- limite de taille;
- rate limits;
- validation feed;
- sync 3 noeuds.

Gate:

- SearchManifest ne contient pas de requetes privees;
- les manifests sont verifiables et expirables;
- si le feed raw-op n'est pas pret, pas de nouvelle op publique fragile.

### S73 - Factory hardening / templates

Objectif produit:

Transformer le chemin Babel canari en outil reutilisable.

Livrables:

- templates additionnels;
- template lock stable;
- deuxieme app simple si utile;
- migration app v1/v2;
- erreurs UX propres;
- docs create/publish.

Gate:

- deux apps generees sans regression;
- hash templates stable;
- compat v1 maintenue.

### S74 - Babel translation beta

Objectif produit:

Tester la traduction comme fonctionnalite, sans la rendre bloquante pour la
preuve Factory.

Livrables:

- `task_submit` traduction mock ou worker local feature-gated;
- resultat stocke;
- review utilisateur;
- provenance du draft;
- fallback fixtures officiel.

Gate:

- si NLLB/worker non pret, le sprint reste valide via fixtures;
- aucune affirmation "compute correct" sans verification.

### S75 - Pack produit defendable

Objectif produit:

Assembler SBFB + Factory + Babel + RRV en demonstration defendable.

Livrables:

- evidence pack final;
- Babel proof card;
- Factory proof card;
- RRV local + SearchManifest si gates OK;
- release narrative;
- decision go/no-go public.

Gate:

- pas de promesse "public" si durabilite/provenance/feed/pilote restent
  incomplets;
- le produit montre une app creee par Factory, pas seulement une spec.

## 11. Gates Factory G0-G10

G0 - Classification app:

- domaine;
- risque donnees;
- bridge methods;
- network needs;
- compute needs.

G1 - Scope:

- MVP borne;
- non-goals explicites;
- no Babel complet.

G2 - Template:

- template id/version;
- hash;
- lockfile.

G3 - Manifest:

- schema v2 valide;
- no `node_id`;
- bridge allowlist.

G4 - Diff:

- preview obligatoire;
- approbation utilisateur.

G5 - Sandbox:

- canonicalize;
- prefix check;
- symlink deny;
- no shell depuis iframe.

G6 - Secrets/deps:

- scan secrets;
- lockfile;
- SBOM si publish.

G7 - Preview:

- iframe sandbox;
- CSP;
- no external fetch par defaut.

G8 - Provenance:

- `factory.provenance.json`;
- generator version;
- template hash;
- variables hash;
- source commit.

G9 - Publish:

- repo HTTPS;
- commit 40 hex;
- artifact hash;
- provenance;
- Browse;
- feed.

G10 - Review:

- sprint review;
- `## Verdict: PASS`;
- evidence pack.

## 12. Tests d'acceptance

Tests bloquants avant Babel canari:

- `deploy-from-repo` accepte app sans `node_id`;
- `deploy-from-repo` refuse manifest invalide;
- `deploy-from-repo` refuse methode bridge inconnue;
- `deploy-from-repo` ou publish-check refuse repo non HTTPS pour feed;
- app generee contient `index.html`, `SBFB.json`, SDK bridge;
- app generee contient planning sprint;
- app generee contient `factory.template.lock`;
- app generee contient `factory.provenance.json`;
- path traversal refuse;
- symlink refuse;
- secret fixture refuse;
- preview iframe smoke test;
- Babel affiche textes fixtures;
- Babel lit/ecrit progression storage;
- Babel appelle `identity_pubkey`;
- Babel affiche provenance;
- Babel lit feed cursor;
- Babel deploy -> Browse -> open -> provenance verify;
- feed `ReleasePublished` cree si chemin feed cable;
- test negatif: aucune methode `babel_*`, `factory_*`, `shell_*`.

Tests apres S70/S71:

- RRV trouve Babel localement;
- Proof Card affiche source, artifact, provenance, feed;
- Proof Card indique stale si source cassable;
- score de completude deterministe;
- FTS5 p95 acceptable sur corpus test.

## 13. Synchronisation canonique requise

Fichiers a corriger ensuite:

- `.planning/roadmap_v2_public_trust_rrv_factory.md`
  - remplacer "Arc 2 avant Arc 3" par "Factory/Babel canari avance en parallele
    borne apres S65/S66";
  - remplacer S73/S75 premiere Factory par S67-S69 canari;
  - remplacer Tantivy-first par FTS5-first;
  - remplacer feed v2/bump par raw-op strategy si PO valide;
  - ajouter ce research en evidence Gate 0.

- `CLAUDE.md`
  - remplacer l'ancienne roadmap;
  - corriger vocabulaire "open source";
  - ajouter Factory+Babel canari;
  - corriger bridge methods;
  - retirer "code reseau = code repo".

- `.planning/codebase/doc_classification.md`
  - classer ce document;
  - corriger counts;
  - corriger "roadmap_v2 n'existe pas".

- `.planning/codebase/claudemd_update_plan.md`
  - remplacer le plan stale "pas de nouveau fichier roadmap";
  - integrer ce pivot;
  - retirer les instructions memory hors repo comme gate canon.

- `.planning/active/`
  - archiver S64;
  - creer `sprint65_kickoff.md`;
  - creer `sprint65_plan.md`.

- `docs/architecture/PUBLISH_MODEL.md`
  - remplacer "open source verifie" cote app par "source verifiable" ou
    clarifier que AGPL/open source qualifie le code SBFB, pas toutes les apps.

## 14. Decisions ouvertes

PO doit trancher:

1. Ouvrir `v2.1` pour S65+ ou garder `v2.0`.
2. Factory dans workspace `nexus` pour MVP ou repo sibling immediat.
3. Nom CLI: `sbfb create`, `nexus factory create`, ou route UI seulement.
4. Niveau exact du domain pack Babel S69.
5. FTS5-first confirme comme decision S70.
6. Feed raw op confirme comme decision S67/S72.
7. `project_id` pour plusieurs apps d'un meme daemon.
8. Exigence de repo public pour le premier Babel canari ou mode local-only
   accepte avant push.

Recommandations:

- `v2.1 OPEN` pour S65+.
- Factory dans workspace pour MVP, extraction plus tard.
- FTS5-first.
- Feed raw op / `serde_json::Value` opaque avant nouvelles ops publiques.
- Babel S69 en canari ferme local/pilote.
- Babel translation beta S74, non bloquante.

## 15. Sources repo utilisees

Sources principales:

- `.planning/roadmap_v2_public_trust_rrv_factory.md`
- `.planning/research/s73_s75_factory_babel_research.md`
- `.planning/research/sbfb_project_factory_rrv_oss_research.md`
- `.planning/research/rrv_scoped_search_compute_groups.md`
- `.planning/research/babel_translation_protocol.md`
- `.planning/research/factory_deploy_constraint_research.md`
- `.planning/research/tantivy_vs_fts5_decision.md`
- `.planning/research/feed_version_bump_strategy.md`
- `.planning/research/licence_anti_capture_research.md`
- `.planning/research/s65_contrat_public_research.md`
- `.planning/research/s66_durabilite_research.md`
- `.planning/research/s68_s69_preuves_pilote_research.md`
- `.planning/active/sprint65_audit_plan.md`
- `.planning/codebase/APPS_BRIDGE_DOCS.md`
- `docs/architecture/PUBLISH_MODEL.md`
- `docs/protocol/PUBLIC_FEED_SPEC.md`
- `docs/affine-sbfb/04_BABEL_SUR_SBFB.md`
- `web/src/bridge/protocol.ts`
- `web/src/bridge/useBridge.ts`
- `crates/nexus-shell-daemon/src/deploy.rs`
- `crates/nexus-coordinator-rs/src/public_feed.rs`
- `crates/nexus-coordinator-rs/src/provenance.rs`

## 16. Bottom line

Le meilleur produit n'est pas "go-live public S65", ni "RRV complet avant
Factory". Le meilleur produit est:

1. confiance exacte;
2. durabilite;
3. Factory qui genere une vraie app;
4. Babel Reader comme canari ferme;
5. RRV qui prouve et explique ce canari;
6. seulement ensuite public plus large.

Ce pivot rend Factory mesurable. Il transforme Babel de concept futur en test
direct du protocole, sans faire porter a Babel complet tout le poids de la
roadmap.
