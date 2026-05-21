# Analyse factuelle : ordonnancement des scopes RRV

**Date :** 2026-05-19
**Statut :** analyse produit factuelle, non engagee en sprint
**Addendum 2026-05-21 :** roadmap v4 conserve `@protocole` d'abord,
mais recadre `@dev` en S70+ par defaut. `@dev` ne bloque pas Gate 1 ;
le pilote ferme teste search/provenance/Proof Cards/publish/Babel
dogfood.
**Question PO :** Faut-il commencer par RRV @dev avant @protocole et @web ?
**Reponse courte :** Non. Commencer par @protocole, pas par @dev.

---

## 1. Tableau comparatif des 3 scopes

| Critere | @protocole (reseau local) | @dev (code workspace) | @web (externe) |
|---------|--------------------------|----------------------|----------------|
| **Donnees existantes** | 7 familles SQLite + DashMap deja presentes dans le daemon (browse entries, feed entries, provenance records, curator lists, project announcements, tasks/kudos, archives zip) | Zero. Pas d'index, pas de tree-sitter, pas d'AST, pas de FTS5 code, pas de search_index.sqlite | Zero. Pas de sidecar SearXNG, pas de connecteur API |
| **Complexite implementation** | BASSE. FTS5 sur des tables SQLite existantes. ~200-300 LOC. Zero crate ajoute (rusqlite bundled FTS5 inclus). 1 migration + 1 handler + 1 indexeur incrementiel | HAUTE. Indexation code multi-langage, chunking, tree-sitter AST, symbols, capabilities, risk scanning, proof cards locales, search_index.sqlite separe. ~600-800 LOC minimum dans un crate separe | MOYENNE. SearXNG Docker sidecar OU API search externe. Questions privacy non resolues. Labels de confiance separes. ~300-400 LOC + config ops |
| **Valeur immediate (pilote ferme)** | HAUTE. Les 2-3 testeurs S69 ont besoin de trouver une app, voir sa provenance, son etat dans le feed. Le browse actuel est un dump sans recherche | BASSE. Le seul developpeur est FlowUP, qui utilise deja Claude Code comme outil de recherche de code. Les testeurs pilote ne sont pas des developpeurs | MOYENNE. Utile pour trouver des briques OSS, mais pas necessaire pour le pilote. Aucun utilisateur du pilote n'a besoin de chercher sur le web via le daemon |
| **Valeur long terme (reseau)** | HAUTE. Fondation pour SearchManifest, Proof Cards, decouverte P2P. C'est la chaine browse -> feed -> provenance -> preuve qui est le coeur de SBFB | MOYENNE. Utile quand Factory fonctionne et que plusieurs apps existent. Aujourd'hui, zero app Factory | HAUTE a terme. La vision "RRV cherche les meilleures briques OSS" depend de @web. Mais la valeur arrive apres que Factory puisse consommer les resultats |
| **Prior art** | AUCUN outil existant ne fait ca. Recherche dans un catalogue P2P verifie avec provenance Ed25519 + feed signe + hash chain. C'est le differenciateur SBFB | Outils existants surabondants : Claude Code, Cursor, Cody, Copilot Workspace, Sourcegraph. SBFB entre dans un marche domine par des acteurs a des centaines de millions | Beaucoup d'outils (SearXNG, Kagi, Perplexity). Mais aucun ne croise web + preuves SBFB. Le croisement est la valeur |
| **Dependance sur autre code** | Aucune. Utilise les tables et structures existantes du daemon | Necessite sbfb-factory (n'existe pas), tree-sitter, schema d'indexation, proof cards locales | Necessite sidecar externe, config reseau, politique privacy |
| **Risque technique** | Faible. FTS5 est compris, les donnees sont la, le path est direct | Eleve. AST parsing multi-langage, chunking, deduplication, ranking complexe | Moyen. SearXNG est stable mais c'est une dep Docker externe |
| **Fait avancer le pilote S69** | Oui directement. Les testeurs cherchent des apps dans le browse | Non. Le pilote teste l'installation, la connexion P2P, le deploy, pas le code | Non. Le pilote est ferme, pas de besoin de recherche web |

---

## 2. Ce que le daemon possede DEJA

Inventaire du code lu directement dans les sources :

### 2.1 Coordinator DB (`db.rs`) — 13 migrations SQLite WAL

Tables existantes avec donnees cherchables :

| Table | Champs exploitables en FTS5 | Migration |
|-------|---------------------------|-----------|
| `public_feed` | op_type, payload (JSON avec project_id, repo_url, commit_sha, artifact_hash), author | M9 |
| `provenance_records` | project_id, repo_url, commit_sha, artifact_hash, node_id, app_version | M12-M13 |
| `tasks` | task_id, project_id, model, task_type | M1+M5 |
| `kudos` | worker_node_id, project_id | M1 |
| `app_storage` | app_name, key, value | M7 |
| `contributor_attestations` | project_id, contributor_node_id, repo_url, commit_sha | M3 |

### 2.2 DashMap in-memory (`browse.rs`, `iroh_runtime.rs`)

| Structure | Champs | Source |
|-----------|--------|--------|
| `BrowseEntry` | project_id, project_name, category, description, curator_pubkey, curator_name, source, status, archive_hash, repo_url, provenance_hash, is_open_source | browse.rs L170-224 |
| `CuratorListEntry` | project_id, name, category, description, curator_pubkey | curator.rs |

### 2.3 Feed entries (`public_feed.rs`)

| Struct | Champs cles |
|--------|-------------|
| `FeedEntry` | version, seq, op (Value), author_pubkey, timestamp, entry_hash, prev_hash, signature |
| `ReleasePublishedPayload` | project_id, repo_url, commit_sha, artifact_hash, provenance_hash, is_open_source |
| `SourceBecameStalePayload` | project_id, reason |

### 2.4 Bridge methods (`protocol.ts`)

Methodes existantes : `task_submit`, `storage_get`, `storage_set`,
`pii_redact`, `storage_list`, `storage_delete`, `identity_pubkey`,
`node_status`, `browse_list`, `storage_version`, `provenance_get`,
`provenance_verify`, `feed_cursor_get`.

### 2.5 Routes HTTP existantes (`http.rs`)

`GET /api/daemon/browse`, `GET /api/daemon/info`,
`GET /api/daemon/curators`, `POST /api/daemon/publish`,
`POST /api/daemon/publish-blob`, `GET /health`,
`GET /blob-serve/{hash}/{*path}`, etc.

**Verdict technique :** Le daemon a assez de donnees structurees
pour qu'une recherche FTS5 @protocole soit utile MAINTENANT.
L'effort est une migration SQLite + un handler HTTP + un indexeur
incrementiel. Zero nouvelle dependance.

---

## 3. Analyse argument par argument

### 3.1 "RRV @dev aide Factory a assembler"

C'est l'argument central des documents de recherche
(`sbfb_project_factory_rrv_oss_research.md` section 3.1).

**Probleme factuel :** Factory n'existe pas. Zero LOC. Le crate
`sbfb-factory` n'est pas cree. Le template engine n'existe pas.
Le broker n'existe pas. Construire RRV @dev pour un outil qui
n'existe pas est une boucle circulaire :

```
On construit RRV @dev
  pour aider Factory a assembler
    mais Factory n'existe pas
      donc RRV @dev n'a rien a chercher d'utile
        donc on construit Factory
          qui n'a pas besoin de RRV @dev pour exister
```

**Fait :** Les documents de recherche le reconnaissent eux-memes.
`sbfb_project_factory_rrv_oss_research.md` section 3.1 :
"Attendre le RRV complet cree un probleme circulaire." Mais la
solution proposee ("ne pas attendre, commencer par RRV @dev
LocalOnly") remplace une circularite par une autre.

**Contre-argument :** Factory peut etre construite sans RRV @dev.
La preuve : SBFB lui-meme a ete construit pendant 65 sprints
sans RRV @dev, avec Claude Code comme outil de recherche de code.
Le PO le sait — c'est exactement ce qui se passe depuis 10 mois.

### 3.2 "RRV @dev = recherche dans le code local, citations fichier:ligne"

**Question factuelle :** Qui a besoin de ca ?

- **FlowUP (le seul developpeur actuel) :** Utilise Claude Code
  depuis 65 sprints. Claude Code fait deja @dev avec une qualite
  superieure a tout ce que RRV @dev produirait au MVP : il lit
  le code, comprend l'AST, suit les imports, genere des citations,
  et il a un modele de langage complet derriere. Construire un
  deuxieme outil de recherche de code est du travail redondant.

- **Les 2-3 testeurs du pilote S69 :** Ce ne sont pas des
  developpeurs. Ils testent l'installation, la connexion P2P,
  le deploy, la stabilite 24h. Ils n'ont pas besoin de chercher
  dans du code.

- **Les futurs developpeurs d'apps SBFB :** Ils n'existent pas
  encore. Et quand ils existeront, ils utiliseront leurs propres
  outils de developpement (IDE, Claude Code, Copilot, etc.).

**Verdict :** RRV @dev n'a pas d'utilisateur immediat qui ne soit
pas deja mieux servi par les outils existants.

### 3.3 "RRV @protocole avec seulement 2-3 apps = listing glorifie"

**C'est un argument valide, mais incomplet.**

Il est vrai que chercher dans 2-3 apps ne justifie pas un moteur
de recherche. Mais RRV @protocole ne cherche pas seulement dans
les apps. Il cherche dans :

- **Le feed** : toutes les operations signees (releases, stale,
  et bientot CuratorVouched). Meme avec 2 apps, le feed peut
  avoir des dizaines d'entries significatives.
- **Les provenance records** : chaque deploy genere un record
  verifiable.
- **Les curator lists** : qui endorse quoi, quand.
- **Les tasks et kudos** : quel travail a ete fait, par qui.
- **Les contributor attestations** : qui a contribue quoi.

Meme avec 2-3 apps, la question "montre moi toutes les releases
verifiees de cette semaine" ou "quels projets ont une provenance
valide" est utile et non triviale. C'est de la gouvernance
cherchable, pas juste un catalogue.

Et surtout : **RRV @protocole est la fondation pour les Proof
Cards (S71) et SearchManifest (S72)**. Sans l'index FTS5 sur les
donnees du daemon, les Proof Cards n'ont pas de source de donnees
structuree pour calculer le score de completude.

### 3.4 "Les briques OSS sont sur le web, donc @web devrait venir en premier"

**L'argument semble logique mais ignore le contexte temporel.**

La vision pitch (`sbfb_rrv_code_factory_vision_pitch.md`) dit :
"RRV cherche les meilleures briques OSS, Factory les assemble."

Mais aujourd'hui :
- Factory n'existe pas (donc personne n'assemble).
- Il n'y a pas de template engine (donc rien a assembler avec).
- Il n'y a pas de broker (donc pas de sandbox pour experimenter).

@web n'a de valeur que si Factory peut consommer les resultats.
Or @web avant Factory = un moteur de recherche web standalone
sans action possible. C'est SearXNG rebadge, pas un produit SBFB.

De plus, @web pose des questions non resolues :
- Privacy : qui voit les requetes ? SearXNG local ou distant ?
- Confiance : comment separer les labels "Web external" des
  preuves SBFB sans confusion utilisateur ?
- Ops : SearXNG = conteneur Docker supplementaire a maintenir.

**Verdict :** @web est le scope le plus utile A TERME pour la
vision "trouver les meilleures briques", mais il n'a pas de
valeur immediate sans Factory fonctionnelle. Et il est le plus
complexe operationnellement.

### 3.5 "Claude Code est deja un RRV @dev"

**C'est le point le plus difficile a contourner.**

| Critere | Claude Code | RRV @dev (MVP projete) |
|---------|------------|----------------------|
| Recherche de code | Oui, multi-fichier, suivi des imports | FTS5 lexical, pas d'import resolution |
| Comprehension semantique | Oui, modele de langage | Non, BM25 + path matching |
| Citations | fichier:ligne | fichier:ligne:hash |
| AST/symbols | Via lecture directe du code | tree-sitter (a implementer) |
| Disponibilite | Maintenant, depuis 10 mois | A construire, ~600-800 LOC |
| Cout d'usage | Abonnement Claude | Gratuit/local |
| Proof labels | Non | Oui (Local indexed, Local tested) |
| Offline | Non | Oui |
| Integre au daemon | Non | Oui (via sbfb-factory) |

Les seuls avantages de RRV @dev sur Claude Code sont : proof
labels, offline, et integration daemon. Les proof labels locaux
sont utiles quand Factory fonctionne. Offline est pertinent
pour la vision anti-censure mais pas pour le pilote (les testeurs
ont Internet). L'integration daemon n'apporte rien tant que
Factory n'existe pas.

**Verdict factuel :** Construire un outil de recherche de code
inferieur a Claude Code pour un seul developpeur qui a deja
Claude Code n'est pas la meilleure allocation de temps de
developpement.

### 3.6 "Le differenciateur SBFB est @protocole, pas @dev"

**C'est le point decisive.**

Aucun outil existant ne fait de la recherche dans un catalogue
P2P verifie avec :
- Provenance Ed25519 signee
- Feed append-only hash-chain BLAKE3
- Curator endorsements signes
- Proof of Work anti-spam
- Labels de preuve separes par source
- Score de completude de preuve deterministe

Ce differenciateur est @protocole, pas @dev. @dev est un terrain
ou SBFB se mesure a Sourcegraph, Cursor, Claude Code, Copilot.
@protocole est un terrain ou SBFB est seul.

---

## 4. Analyse cout/valeur par utilisateur

### 4.1 FlowUP (developpeur principal)

| Scope | Valeur immediate | Cout | Ratio |
|-------|-----------------|------|-------|
| @protocole | Voir ses propres deploys, feed, provenance de maniere structuree. Fonde les Proof Cards | ~300 LOC, 1 phase | HAUT |
| @dev | Deja couvert par Claude Code. Gain marginal des proof labels locaux | ~800 LOC, 2 phases | BAS |
| @web | Trouver des briques OSS. Utile mais faisable via browser normal | ~400 LOC + Docker, 1 phase | MOYEN |

### 4.2 Testeurs pilote (2-3 personnes, S69)

| Scope | Valeur immediate | Cout | Ratio |
|-------|-----------------|------|-------|
| @protocole | Chercher une app, voir sa preuve, comprendre le feed | 0 (deja construit) | TRES HAUT |
| @dev | Aucune. Ce ne sont pas des developpeurs | N/A | ZERO |
| @web | Aucune. Le pilote est ferme | N/A | ZERO |

### 4.3 Futurs developpeurs d'apps SBFB (post-pilote)

| Scope | Valeur immediate | Cout | Ratio |
|-------|-----------------|------|-------|
| @protocole | Decouvrir les apps existantes et leur etat de preuve | 0 (deja construit) | HAUT |
| @dev | Chercher dans le code des apps existantes. Utile quand il y a 10+ apps | Proportionnel au nombre d'apps | MOYEN a terme |
| @web | Trouver des briques OSS a reutiliser. Utile quand Factory fonctionne | Proportionnel a la maturite Factory | HAUT a terme |

---

## 5. Recommandation factuelle

### Ordre recommande : @protocole -> @dev post-pilote -> @web

**Etape 1 (S67-S68) : @protocole**

Construire l'index FTS5 sur les donnees daemon existantes (browse
entries, feed entries, provenance records, curator lists).
Exposer `GET /api/daemon/search`. Construire les Proof Cards.

Cout : ~300-400 LOC dans le daemon, zero nouvelle dependance.
Valeur : fondation pour toute la chaine RRV, utilisable
immediatement par le browse, les apps iframe (bridge `search`),
et le futur sbfb-search.

**Etape 2 (S70+ par defaut, stretch S68-S69 si zero-impact) : @dev**

Construire l'index local dans sbfb-factory une fois que le crate
existe et que les templates fonctionnent. A ce stade, RRV @dev a
un cas d'usage concret : aider Factory a trouver des patterns
dans le workspace courant. Ce travail ne doit pas retarder Proof
Cards, publish gate, ni Babel dogfood.

Cout : ~400-600 LOC dans sbfb-factory. tree-sitter en stretch.
Valeur : citations locales, proof labels, aide a l'assemblage.

**Etape 3 (S72+, apres pilote) : @web**

Integrer SearXNG comme sidecar optionnel apres que le pilote
ait valide la chaine @protocole + Factory + Babel. A ce stade,
les labels "Web external" / "Web claim" ont un referentiel de
comparaison (les preuves @protocole).

Cout : ~300 LOC + Docker ops + politique privacy.
Valeur : la vision "trouver les meilleures briques OSS".

### Ce que ca change par rapport aux plans existants

**Roadmap v3 (archive) :** L'ordre etait Factory (S67-S69)
puis RRV (S70-S72). @protocole est reporte en S70. C'est TROP
TARD — les Proof Cards et la recherche locale devraient etre
fondees des S67, pas attendues 3 sprints.

**Roadmap v4 (canon apres recadrage 2026-05-21) :** l'ordre est
Factory + `@protocole` pour Arc 2, puis `@dev` S70+ par defaut.
C'est le compromis correct : `@protocole` est trivial a construire
(les donnees existent) et necessaire au pilote ; `@dev` depend de
sbfb-factory, d'un corpus source utile, et d'un contrat source-only
si des repos OSS externes sont indexes.

**Recommandation recadree pour S67-S70 :**

```
Phase A : Primitives daemon neutres (sbfb-manifest, CuratorVouched,
          feed/entries) + FTS5 daemon search
          (migration M15 + search.rs + API)
Phase B : FTS5/search @protocole consolide
Phase C : sbfb-factory crate + create/validate/template lock
Phase D/E : provenance/debt/wrap-up, sans @dev

S68 : Proof Cards @protocole + publish gate + UX confiance
S69 : Babel dogfood via Factory + pilote ferme
S70+ : @dev source index / source-only OSS seed si le pilote est propre
```

Cet ordre met @protocole en premier, cree Factory ensuite, puis
reserve @dev au moment ou il existe assez d'apps/sources pour que le
cout d'indexation ait une valeur produit reelle.

---

## 6. Risques de chaque ordonnancement

### 6.1 Si on commence par @dev (plan des docs de recherche)

| Risque | Probabilite | Impact |
|--------|------------|--------|
| Construit un outil de recherche inferieur a Claude Code | Haute | Temps perdu, pas d'utilisateur |
| Factory n'existe pas, @dev n'a rien d'utile a indexer | Certaine au jour 0 | Boucle circulaire |
| Les testeurs pilote n'en ont pas besoin | Certaine | Pas de feedback utilisable |
| Retarde @protocole qui est le differenciateur | Haute | Manque la fenetre de credibilite |

### 6.2 Si on commence par @protocole (recommandation)

| Risque | Probabilite | Impact |
|--------|------------|--------|
| 2-3 apps seulement dans le catalogue | Certaine | Recherche utile mais pas impressionnante. Mitige par le feed et la provenance qui ont plus d'entries que les apps |
| @dev arrive plus tard | Faible impact | Claude Code couvre le gap |
| @web arrive plus tard | Faible impact court terme | La vision OSS est repoussee mais pas compromise |

### 6.3 Si on commence par @web

| Risque | Probabilite | Impact |
|--------|------------|--------|
| SearXNG a maintenir sans cas d'usage Factory | Haute | Cout ops sans valeur |
| Privacy non resolue | Haute | Contradictoire avec la posture SBFB |
| Pas de referentiel de comparaison pour les labels | Certaine | Confusion utilisateur |

---

## 7. Contradiction avec le plan actuel

**Oui, cette analyse contredit partiellement les documents de
recherche existants.**

Les trois documents de recherche
(`rrv_scoped_search_compute_groups.md`,
`sbfb_rrv_code_factory_vision_pitch.md`,
`sbfb_project_factory_rrv_oss_research.md`) sont unanimes sur
"RRV @dev d'abord, reseau ensuite". Le document Factory/RRV OSS
research est le plus categorique (section 3.1 : "Le bon ordre :
Project Factory local-first -> RRV @dev LocalOnly -> Babel
dogfood -> SearchManifest -> RRV network").

Cette analyse dit : **@protocole d'abord, @dev apres le pilote par
defaut, @web plus tard.**

La raison de la divergence : les documents de recherche sont
ecrits dans une logique "Factory-first" ou RRV @dev est
subsidiaire a Factory. C'est logique si Factory est le produit
central. Mais la question du PO est plus fondamentale : est-ce
que RRV @dev a de la valeur INDEPENDAMMENT de Factory ? La
reponse factuelle est non — pas tant que Claude Code existe et
que le seul utilisateur est le developpeur principal.

Le differenciateur de SBFB n'est pas la recherche de code. C'est
la recherche dans un protocole verifie. @protocole est le seul
scope ou SBFB n'a pas de concurrent.

---

## 8. Resume en une phrase

Commencer par @protocole (FTS5 sur les donnees daemon existantes,
~300 LOC, zero dep) parce que c'est la seule chose que personne
d'autre ne fait, que les donnees sont deja la, et que les
utilisateurs du pilote en ont besoin — puis co-developper @dev
avec Factory quand Factory existe et que Gate 1 n'est plus en risque,
et @web apres le pilote.

---

## 9. Sources

### Code lu directement

- `crates/nexus-coordinator-rs/src/db.rs` (13 migrations, tables existantes)
- `crates/nexus-coordinator-rs/src/public_feed.rs` (FeedEntry, operations)
- `crates/nexus-shell-daemon-core/src/browse.rs` (BrowseEntry, aggregator)
- `crates/nexus-shell-daemon/src/http.rs` (routes existantes, DaemonHttpState)
- `web/src/bridge/protocol.ts` (bridge methods)

### Documents de recherche

- `.planning/research/rrv_scoped_search_compute_groups.md`
- `.planning/research/sbfb_rrv_code_factory_vision_pitch.md`
- `.planning/research/sbfb_project_factory_rrv_oss_research.md`
- `.planning/research/s70_s72_rrv_research.md`

### Roadmaps

- `.planning/roadmap_v3_public_trust_factory_babel_rrv.md` (canon)
- `.planning/roadmap_v4_neutral_protocol_factory_rrv.md` (draft)
