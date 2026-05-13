# Chat IA Reseau - R&D recherche non-Google pour SBFB

**Date:** 2026-05-12  
**Statut:** recherche produit post-v1.0, non engagee en sprint  
**Scope:** SBFB en tant que reseau/protocole, pas les apps personnelles publiees sur un noeud  
**Question:** peut-on creer un type de recherche different de la recherche Google classique pour le Chat IA Reseau ?  
**Verdict court:** oui. La bonne direction n'est pas de refaire Google. C'est de creer une **Recherche Reseau Verifiable**: une recherche qui trouve des capacites, du code, des preuves, des artefacts executables, des peers, des historiques de taches et des lignages de projets dans le reseau SBFB.

---

## 0. These produit

Google cherche des pages.

SBFB peut chercher des **objets reseau executables et verifiables**:

- apps publiees,
- morceaux de code reutilisables,
- preuves de provenance,
- artefacts iroh-blobs,
- commits sources,
- capabilities exposees par une app,
- etats CRDT publics,
- taches IA et resultats signes,
- workers capables d'executer une action,
- forks et lignages,
- curations communautaires,
- resultats reproduits par quorum.

La promesse produit n'est donc pas:

> "Un moteur de recherche web dans SBFB."

La promesse devrait etre:

> "Je demande au reseau ce qui existe, ce qui est fiable, ce qui tourne chez moi, ce qui peut etre fork, audite, compose ou execute maintenant."

Nom de travail recommande: **Recherche Reseau Verifiable** (`RRV`).

Autres noms possibles:

- **Network Proof Search**
- **Executable Search**
- **Capability Search**
- **Proof-of-Use Search**
- **Search-to-Run**

Le nom interne le plus exact est `RRV`, parce qu'il met au centre le differenciateur: la recherche retourne une preuve exploitable, pas seulement une URL.

---

## 1. Point de depart dans le repo

### 1.1 Chat IA Reseau existe deja comme vision de RAG distribue

Source repo: [`docs/apps/CHAT_IA_RESEAU.md`](../../docs/apps/CHAT_IA_RESEAU.md)

Le design actuel de Chat IA Reseau decrit deja:

- un chat IA integre au shell SBFB;
- un acces au code source des projets publies sur le reseau P2P;
- un index local par noeud;
- un embedding de la question;
- une recherche top chunks dans un index vectoriel local;
- un prompt RAG avec chunks + metadata projet;
- une execution via pipeline GPU distribue;
- un mode P2P optionnel pour partager l'index.

Ce design est bon comme **MVP RAG**.

Limite produit: il reste proche du schema "recherche semantique de chunks + reponse IA". C'est deja puissant, mais ce n'est pas encore un nouveau type de recherche.

### 1.2 Generation Composee pousse deja vers une recherche de patterns

Source repo: [`docs/apps/GENERATION_COMPOSEE.md`](../../docs/apps/GENERATION_COMPOSEE.md)

La Generation Composee ajoute une idee majeure:

- le systeme ne cherche plus seulement des informations;
- il cherche les meilleurs morceaux de code du reseau;
- il combine les patterns;
- il garde la lignee des sources;
- il reinjecte les nouvelles apps dans l'index.

Donc la recherche utile pour SBFB doit deja depasser "trouver un document". Elle doit trouver:

- un pattern;
- une capacite;
- une preuve de qualite;
- une compatibilite de licence;
- un contexte d'execution;
- une filiation reutilisable.

### 1.3 Le modele de publication donne une base de confiance rare

Source repo: [`docs/architecture/PUBLISH_MODEL.md`](../../docs/architecture/PUBLISH_MODEL.md)

Le modele SBFB distingue:

- `Local Draft`,
- `Unverified Build`,
- `Verified Release`,
- `Stale Source`.

Une `Verified Release` lie:

```text
repo_url + commit_sha + artifact_hash + provenance_hash
```

C'est essentiel pour la recherche. Un moteur classique retourne "voici un lien". SBFB peut retourner:

- "voici un artefact immutable";
- "voici le commit source exact";
- "voici la provenance";
- "voici si les workers L2 peuvent l'accepter";
- "voici si la source est encore reverifiable";
- "voici les chunks de code qui supportent l'affirmation".

Cette brique est un avantage structurel.

### 1.4 Le self-hosted build ouvre la recherche par preuve d'execution

Source repo: [`docs/architecture/SELF_HOSTED_BUILD.md`](../../docs/architecture/SELF_HOSTED_BUILD.md)

Le self-hosted build decrit le reseau qui compile lui-meme des releases verifiees avec quorum de hash.

Pour la recherche, cela permet une future requete du type:

> "Trouve-moi une app de traduction offline, open source verifiee, qui build sur Windows et Linux, et prouve-le."

Le moteur ne doit pas seulement chercher dans un index. Il peut declencher:

- verification de build;
- execution de tests;
- scan securite;
- quorum multi-worker;
- generation d'un paquet de preuves.

### 1.5 Browse, publish et bridge exposent deja des signaux utiles

Sources repo:

- [`crates/nexus-shell-daemon-core/src/browse.rs`](../../crates/nexus-shell-daemon-core/src/browse.rs)
- [`crates/nexus-shell-daemon-core/src/publish.rs`](../../crates/nexus-shell-daemon-core/src/publish.rs)
- [`web/public/sbfb-bridge.js`](../../web/public/sbfb-bridge.js)
- [`crates/nexus-core-rs/src/task.rs`](../../crates/nexus-core-rs/src/task.rs)

Signaux deja presents ou proches:

| Brique | Signal exploitable par la recherche |
|---|---|
| `BrowseEntry` | nom projet, categorie, description, curator, source, status, ticket archive, repo, provenance, open source |
| `ProjectAnnouncement` | publication gossip, apps, archive, repo, provenance, `is_open_source` |
| `Task` | prompt, type, modele, priorite, metadata string-only, cout estime, redondance |
| `ResultEntry` | futur support de preuves de resultat et quorum |
| `bridge.submitTask` | lancer une recherche active via workers |
| `bridge.getBrowseList` | recuperer les projets visibles |
| `bridge.piiRedact` | nettoyer les requetes avant de les envoyer au reseau |
| `bridge.getNodeStatus` | contextualiser par peers, uptime, version |
| `bridge.getStorageVersion` | suivre l'etat replique public d'une app |

Conclusion: SBFB a deja les primitives pour faire une recherche qui mele **contenu, confiance, disponibilite et action**.

---

## 2. Recherche open source: ce qui existe deja

Cette section compare les projets open source ou ouverts pertinents. Le but n'est pas de copier, mais d'identifier les briques qui ont fait leurs preuves.

### 2.1 Meta-search: SearXNG

Source: [SearXNG docs](https://docs.searxng.org/)

SearXNG est un metasearch engine libre qui agrege des resultats de nombreux services, avec une orientation forte privacy/no profiling et self-hosting. Il a des moteurs configurables, des result types, une Search API, des plugins et des answerers.

Lecon pour SBFB:

- interessant comme **connecteur web optionnel**;
- utile pour avoir une passerelle "web actuel" sans crawler central;
- ne doit pas etre le coeur du Chat IA Reseau.

Limite pour SBFB:

- SearXNG agrege des moteurs externes;
- il ne verifie pas que le resultat est un artefact executable;
- il ne connait pas les preuves SBFB, les workers, les capabilities, ni les taches.

Position recommandee:

```text
SearXNG = mode "Web sidecar"
RRV SBFB = mode "Network-native proof search"
```

### 2.2 P2P search: YaCy

Source: [YaCy](https://www.yacy.net/)

YaCy propose un mode P2P avec index partage, pas de serveur central et une logique "web search by the people". C'est le precedent open source le plus proche de l'idee d'un index distribue.

Lecon pour SBFB:

- le P2P search est faisable;
- un index partage peut vivre sans serveur central;
- les noeuds peuvent contribuer au crawling/indexation.

Limite pour SBFB:

- YaCy reste centre sur le web et les pages;
- il ne manipule pas les notions SBFB de release verifiee, task quorum, consent worker, iroh-blobs, source commit, capability d'app.

Position recommandee:

```text
YaCy prouve le P2P search.
SBFB doit faire le P2P proof/action search.
```

### 2.3 Code search: Zoekt

Source: [Zoekt GitHub](https://github.com/sourcegraph/zoekt)

Zoekt est un moteur de recherche texte pour code source. Il est optimise pour le substring/regexp matching, les index trigram, la recherche multi-repo et le ranking par signaux de code comme les symboles.

Lecon pour SBFB:

- un moteur de recherche code doit etre excellent en lexical exact, pas seulement vectoriel;
- les symboles comptent enormement;
- regexp, path filters, language filters et symbol matches sont indispensables.

Limite pour SBFB:

- Zoekt ne fait pas de provenance reseau;
- il ne sait pas si un repo est publie sous forme d'artefact immutable;
- il ne declenche pas de verification ou d'execution.

Position recommandee:

```text
MVP local: SQLite FTS5 + sqlite-vec.
Evolution code search: s'inspirer de Zoekt pour trigram/symbol ranking.
```

### 2.4 Full-text Rust: Tantivy

Source: [Tantivy GitHub](https://github.com/quickwit-oss/tantivy)

Tantivy est une librairie full-text en Rust inspiree de Lucene, avec BM25, tokenizers configurables, phrase queries, incremental indexing et multithreaded indexing.

Lecon pour SBFB:

- tres bon candidat Rust pour un index local embarque;
- coherent avec le virage Rust du projet;
- plus controle qu'un serveur externe type Elasticsearch.

Limite pour SBFB:

- Tantivy est une librairie d'indexation, pas une architecture reseau;
- il faut ajouter provenance, embeddings, AST, ranking SBFB et protocole P2P.

Position recommandee:

```text
Court terme: SQLite FTS5 pour aller vite.
Moyen terme: Tantivy pour index lexical robuste dans le daemon Rust.
```

### 2.5 Vector search local: sqlite-vec et pgvector

Sources:

- [sqlite-vec GitHub](https://github.com/asg017/sqlite-vec)
- [pgvector GitHub](https://github.com/pgvector/pgvector)

`sqlite-vec` est une extension SQLite de recherche vectorielle, petite, portable, sans dependances, compatible Windows/Linux/macOS/WASM/Raspberry Pi. `pgvector` ajoute la recherche vectorielle a Postgres avec exact/approx nearest neighbors et plusieurs distances.

Lecon pour SBFB:

- `sqlite-vec` correspond tres bien a un noeud local;
- `pgvector` est plus pertinent pour une infra serveur, moins pour le desktop/P2P local-first;
- l'index local doit rester transportable, sauvegardable et facile a reconstruire.

Position recommandee:

```text
MVP Chat IA: SQLite + FTS5 + sqlite-vec.
Pas de Postgres obligatoire dans le chemin end-user.
```

### 2.6 Hybrid search: Qdrant, LanceDB, OpenSearch, Vespa

Sources:

- [Qdrant hybrid dense/sparse](https://qdrant.tech/articles/sparse-vectors/)
- [LanceDB hybrid search](https://docs.lancedb.com/search/hybrid-search)
- [OpenSearch neural/hybrid search](https://docs.opensearch.org/2.14/search-plugins/neural-search/)
- [Vespa hybrid search tutorial](https://docs.vespa.ai/en/learn/tutorials/hybrid-search.html)

Le standard moderne n'est plus "keyword ou vectoriel". C'est hybride:

- lexical/BM25 pour exactitude, identifiants, symboles, termes rares;
- dense vectors pour sens et paraphrases;
- sparse vectors pour requetes lexicales enrichies;
- reranking pour la precision finale;
- ranking multi-signal.

Lecon pour SBFB:

- un Chat IA Reseau qui ne fait que du vectoriel sera mediocre sur le code;
- un Chat IA Reseau qui ne fait que du lexical sera mediocre sur les intentions utilisateur;
- le bon moteur est un **hybride local + reseau + preuve**.

Position recommandee:

```text
RRV = lexical + semantic + AST + proof + availability + usage + safety.
```

### 2.7 RAG orchestration: LlamaIndex et Haystack

Sources:

- [LlamaIndex RAG](https://developers.llamaindex.ai/python/framework/understanding/rag/)
- [Haystack introduction](https://docs.haystack.deepset.ai/docs/intro)

LlamaIndex documente les etapes RAG classiques: loading, indexing, storing, querying, evaluation. Haystack insiste sur les pipelines, components, document stores, agents et tools.

Lecon pour SBFB:

- les primitives RAG sont connues;
- l'evaluation doit etre prevue tot;
- les tools/agents peuvent router une question vers plusieurs retrievers.

Limite pour SBFB:

- ces frameworks ne sont pas P2P-native;
- ils n'integrent pas naturellement iroh-blobs, provenance SBFB, consent worker, quorum, bridge iframe;
- ils peuvent inspirer la structure mais ne doivent pas devenir une dependance centrale lourde.

Position recommandee:

```text
S'inspirer du pipeline RAG.
Ne pas externaliser l'architecture coeur dans un framework Python lourd.
```

### 2.8 Personal AI/search: Khoj

Source: [Khoj docs](https://docs.khoj.dev/)

Khoj est un assistant personnel open source qui peut repondre avec les fichiers fournis, trouver notes/documents en langage naturel et fonctionner en self-host.

Lecon pour SBFB:

- le "personal AI over your data" est valide;
- l'UX de recherche naturelle sur notes/docs a une demande claire;
- local-first et privacy sont des angles forts.

Limite pour SBFB:

- SBFB n'est pas seulement "mes documents";
- SBFB a un reseau, des apps, des artefacts, des workers, des preuves et du compute.

Position recommandee:

```text
RRV doit fusionner "personal memory" + "network memory".
```

### 2.9 AI search web: Perplexica

Source: [Perplexica GitHub](https://github.com/comput3ai/c3-perplexica)

Perplexica est un moteur de recherche IA open source inspire de Perplexity. Il utilise SearXNG pour recuperer des resultats web actuels, puis embeddings/similarity/reranking pour produire une reponse avec sources.

Lecon pour SBFB:

- bon modele pour un mode "web actuel avec citations";
- evite de devoir maintenir un crawler quotidien;
- montre une UX attendue: reponse directe + sources.

Limite pour SBFB:

- reste web-first;
- ne retourne pas des actions SBFB;
- pas de preuve de build, pas de capability, pas de provenance d'artefact.

Position recommandee:

```text
Perplexica-like = option pour web search.
RRV = reseau SBFB first.
```

### 2.10 Code intelligence: tree-sitter, Semgrep, CodeQL, SCIP

Sources:

- [tree-sitter GitHub](https://github.com/tree-sitter/tree-sitter)
- [Semgrep docs](https://semgrep.dev/docs/introduction)
- [CodeQL docs](https://codeql.github.com/docs/codeql-overview/about-codeql/)
- [Sourcegraph precise code intelligence / SCIP](https://sourcegraph.com/docs/code_intelligence/explanations/precise_code_intelligence)

Lecon pour SBFB:

- pour chercher du code utile, il faut parser le code;
- les symboles, imports, call graphs et patterns statiques sont des signaux plus fiables que des chunks texte;
- la recherche securite doit etre deterministe quand possible;
- l'IA doit expliquer et prioriser, pas remplacer les scanners.

Position recommandee:

```text
Phase 1: chunk texte + metadata.
Phase 2: tree-sitter symbols.
Phase 3: Semgrep-like rules pour risques.
Phase 4: CodeQL/SCIP-like precise intelligence pour langages majeurs.
```

### 2.11 Content routing: IPFS/IPNI et iroh

Sources:

- [IPFS content addressing](https://docs.ipfs.tech/concepts/content-addressing/)
- [IPNI](https://docs.ipfs.tech/concepts/ipni/)
- [iroh blobs](https://docs.iroh.computer/protocols/blobs)
- [iroh overview](https://docs.iroh.computer/what-is-iroh)

IPFS/IPNI montrent le modele "contenu adresse par hash + index de providers". iroh-blobs donne a SBFB une brique directe: blobs content-addressed par BLAKE3, streaming verifie, range requests, collections, et composition avec gossip/docs sur un endpoint.

Lecon pour SBFB:

- un index reseau ne devrait pas stocker tout le contenu;
- il doit publier des manifests et pointer vers des blobs;
- la verification par hash doit etre native;
- les index shards peuvent etre eux-memes des blobs.

Position recommandee:

```text
SearchManifest via gossip/docs.
IndexShard via iroh-blobs.
AnswerProof via result signed + blob/hash refs.
```

---

## 3. Ce que SBFB peut faire d'unique

### 3.1 Difference fondamentale avec Google

| Google-like | SBFB RRV |
|---|---|
| Cherche des pages web | Cherche des objets reseau executables |
| Rank par popularite, liens, SEO, fraicheur | Rank par pertinence + preuve + disponibilite + securite + usage |
| L'utilisateur clique et juge lui-meme | Le systeme peut verifier, build, run, auditer |
| Source = URL | Source = commit + hash artefact + provenance + chunk hash |
| Index centralise | Index local-first, shards P2P opt-in |
| Resultat passif | Resultat actionnable: ouvrir, run, fork, auditer, composer |
| Pas de consent GPU local | Respect du consent worker et des caps |
| Pas de notion de "peut tourner chez moi" | Recherche par machine, OS, VRAM, watt, consent |

La nouveaute n'est pas "recherche decentralisee" seule. YaCy existe deja.

La nouveaute est:

```text
recherche decentralisee + artefacts executables + preuves de provenance
+ compute distribue + generation composee + consent + quorum.
```

### 3.2 Definition produit de RRV

La Recherche Reseau Verifiable est un moteur qui transforme une question en:

1. candidats locaux;
2. candidats reseau;
3. preuves associees;
4. actions possibles;
5. reponse IA citee;
6. option de verification active.

Exemple:

```text
Question:
  "Trouve-moi le meilleur pattern de sync CRDT pour une app de chat offline."

RRV retourne:
  - 6 chunks de code issus de 4 projets verifies;
  - les commits exacts;
  - les artefacts iroh-blobs;
  - les scores de reutilisation;
  - les risques detectes;
  - les alternatives;
  - un bouton "generer une app avec ce pattern";
  - un bouton "lancer un audit reseau 3 workers".
```

### 3.3 Les objets indexes

RRV ne doit pas indexer seulement des documents. Il doit indexer des types.

| Objet | Exemple | Pourquoi c'est different |
|---|---|---|
| `Project` | app publiee | unite visible par Browse |
| `Release` | commit + artefact + provenance | unite de confiance |
| `SourceFile` | `src/sync.ts` | recherche code |
| `CodeChunk` | fonction ou bloc | RAG et generation |
| `Symbol` | fonction, classe, route, event | code search precis |
| `Capability` | "offline transcription", "task_submit", "storage sync" | recherche par capacite |
| `PermissionSurface` | bridge methods, task types, storage | audit et consent |
| `BuildProof` | hash quorum | verifier que ca build |
| `TaskTrace` | task input metadata + result + worker sig | preuve d'execution |
| `WorkerCapability` | modele, VRAM, OS, caps | "qui peut executer ?" |
| `CuratorAttestation` | recommandation signee | confiance sociale |
| `LineageEdge` | app B reutilise chunk de app A | generation composee |
| `RiskFinding` | Semgrep/code audit | recherche securite |
| `PublicState` | CRDT public indexable | recherche d'etat reseau |

### 3.4 Les modes de recherche

Le Chat IA Reseau devrait proposer plusieurs modes, pas une seule barre.

| Mode | Requete type | Reponse attendue |
|---|---|---|
| `Ask` | "Comment marche cette app ?" | reponse citee avec chunks |
| `Find app` | "Une app de traduction offline" | apps + preuves + actions |
| `Find pattern` | "Meilleur pattern CRDT chat" | code chunks + lineage + score |
| `Compare` | "Compare 3 approches auth" | tableau + citations + risques |
| `Can I run?` | "Est-ce que ca tourne chez moi ?" | OS/VRAM/CPU/deps + verdict |
| `Can I trust?` | "Est-ce verifie ?" | provenance + source + stale status |
| `Audit` | "Cherche XSS dans ces apps" | findings statiques + IA + preuves |
| `Compose` | "Cree une app inspiree de..." | sources candidates + generation |
| `Watch` | "Alerte si une app ajoute task_submit" | abonnement sur manifests |
| `Ask network` | "Demande a 3 workers de verifier" | resultats signes + quorum |

### 3.5 Resultat = carte actionnable, pas lien

Un resultat RRV doit ressembler a:

```yaml
result_id: sbfbrrv_01
kind: Project | CodeChunk | Capability | Proof | WorkerPool
title: "babel-offline-translate"
summary: "App de traduction offline avec index local et tasks GPU."
why_matched:
  - "capability: translation"
  - "requires no external cloud"
  - "verified release"
proof:
  project_id: "..."
  repo_url: "https://..."
  commit_sha: "..."
  artifact_hash: "blake3:..."
  provenance_hash: "blake3:..."
  source_state: "verified_release"
availability:
  reachable_peers: 3
  cached_locally: true
  last_probe: "2026-05-12T20:41:00Z"
security:
  is_open_source: true
  risk_level: "medium"
  findings: 2
actions:
  - open
  - run
  - inspect_source
  - fork
  - audit_network
  - use_as_generation_source
```

Chaque carte doit repondre a trois questions:

1. Pourquoi ce resultat ?
2. Quelle preuve ?
3. Quelle action maintenant ?

---

## 4. Les features uniques a viser

### Feature 1 - Recherche par capacite

**Question utilisateur:**

```text
"Trouve une app qui peut traduire des documents offline, sans envoyer mes donnees au cloud."
```

Le moteur cherche:

- capabilities declarees dans `SBFB.json`;
- code qui appelle ou non des endpoints externes;
- usage de `bridge.submitTask`;
- usage de `piiRedact`;
- dependances reseau;
- metadata de provenance.

Valeur produit:

- l'utilisateur ne cherche pas une page;
- il cherche une capacite actionnable;
- SBFB peut dire "oui/non, et pourquoi".

MVP:

- inferer capabilities via manifest + scan heuristique;
- stocker dans `search_capabilities`;
- exposer des filtres UI.

### Feature 2 - Recherche "peut tourner chez moi ?"

**Question utilisateur:**

```text
"Montre seulement les apps qui tournent sur Windows sans Docker et qui n'ont pas besoin de plus de 8GB VRAM."
```

Signaux:

- OS support dans metadata package;
- scripts build;
- resultats de tests precedents;
- tasks avec `estimated_vram_mb`;
- worker capabilities;
- artefacts installables.

Valeur produit:

- reduit la friction adoption;
- transforme la recherche en assistant d'installation;
- exploite directement les builds Windows/macOS/Linux post-v1.

MVP:

- champs `platforms`, `requires_gpu`, `estimated_vram_mb`, `installer_available`;
- scan des manifests et releases;
- badge "compatible avec ce noeud".

### Feature 3 - Recherche par preuve de confiance

**Question utilisateur:**

```text
"Je veux seulement des projets open source verifies, pas des zips uploades."
```

Signaux:

- `is_open_source`;
- `repo_url`;
- `provenance_hash`;
- `artifact_hash`;
- etat `Verified Release` ou `Stale Source`;
- quorum build futur.

Valeur produit:

- la recherche devient une couche de trust;
- les workers L2 et les utilisateurs voient le meme etat;
- evite la confusion "open source claim" vs "open source verifie".

MVP:

- filtres `verified_only`, `source_available`, `stale_allowed`;
- affichage du proof bundle dans chaque resultat.

### Feature 4 - Recherche de patterns de code

**Question utilisateur:**

```text
"Quel est le meilleur pattern de file d'attente de taches dans le reseau ?"
```

Signaux:

- lexical exact: `queue`, `task`, `claim`, `timeout`;
- embeddings semantiques;
- symboles tree-sitter;
- reuse count;
- scan score;
- bugs connus;
- resultats de tests;
- lignage Generation Composee.

Valeur produit:

- SBFB devient un "npm vivant" de patterns;
- les devs cherchent des solutions, pas des repos;
- la generation composee obtient une selection plus robuste.

MVP:

- chunker par fonction;
- extraction symboles JS/TS/Rust/Python;
- ranking `pattern_score`.

### Feature 5 - Recherche active par workers

**Question utilisateur:**

```text
"Verifie si cette app peut etre packagee en .deb et donne-moi une preuve."
```

Le moteur peut:

1. chercher le projet;
2. soumettre une tache `network_search_verify`;
3. demander a N workers de build/test;
4. comparer les hashes/logs;
5. retourner un verdict signe.

Valeur produit:

- la recherche devient computationnelle;
- un resultat peut etre prouve apres la requete;
- c'est impossible pour un moteur de recherche classique sans infrastructure d'execution.

MVP:

- task type `search_verify`;
- metadata string-only:

```text
search.query_hash
search.target_project_id
search.target_commit_sha
search.action = "build" | "test" | "audit" | "summarize"
search.privacy_mode
```

### Feature 6 - Reponse IA avec citations verifiables

Un Chat IA classique cite parfois des pages.

RRV doit citer:

- fichier;
- byte range ou line range;
- chunk hash;
- commit;
- artifact hash;
- provenance hash;
- worker result id si verification active.

Format:

```text
Selon `src/sync.ts` dans commit `abc123`, chunk `blake3:...`,
la sync utilise un LWW register. L'artefact publie est
`blake3:...` et la provenance est `blake3:...`.
```

Valeur produit:

- plus dur d'halluciner;
- l'utilisateur peut inspecter;
- la reponse a une chaine de preuve.

### Feature 7 - Recherche de lineage

**Question utilisateur:**

```text
"D'ou vient ce pattern de stockage replique ?"
```

Le moteur montre:

```text
sbfb-chat/storage.js
  <- protocol-explorer/storage.js
  <- ideas-hub/storage.js
  <- base bridge storage sample
```

Valeur produit:

- rend la generation composee transparente;
- detecte monoculture et propagation de bugs;
- permet de remonter une faille a tous les descendants.

MVP:

- `LineageEdge { from_chunk, to_chunk, reason, generation_task_id }`;
- hash de chunk;
- metadata de generation.

### Feature 8 - Recherche securite reseau

**Question utilisateur:**

```text
"Quelles apps publiees appellent task_submit sans piiRedact ?"
```

Signaux:

- AST tree-sitter;
- regles Semgrep-like;
- bridge method usage;
- policy `COMPUTE_THREATS`;
- consent worker.

Valeur produit:

- SBFB surveille son ecosysteme;
- les curateurs peuvent auditer avant de recommander;
- les workers peuvent refuser des classes de tasks.

MVP:

- rules simples JS:
  - `submitTask` sans `piiRedact` dans le meme flow;
  - fetch externe;
  - eval/new Function;
  - stockage de tokens;
  - iframe sandbox escape attempts.

### Feature 9 - Recherche par disponibilite reseau

**Question utilisateur:**

```text
"Montre les apps disponibles maintenant, pas juste referencees."
```

Signaux:

- `BrowseStatus`;
- reachability probe;
- nombre de peers providers;
- cache local;
- age du dernier probe;
- tickets blobs.

Valeur produit:

- evite la frustration "resultat mort";
- favorise les projets bien seedes;
- rend visible la sante P2P.

MVP:

- ranking boost `reachable`;
- filtres `cached_locally`, `reachable_now`, `providers >= N`.

### Feature 10 - Recherche consensus pour sujets sensibles

**Question utilisateur:**

```text
"Est-ce que ce projet viole la politique de consent worker L2 ?"
```

Le moteur peut demander a plusieurs workers:

- analyse statique;
- analyse IA;
- comparaison;
- vote final.

Valeur produit:

- plus robuste qu'une reponse LLM unique;
- utile pour securite, trust, gouvernance;
- reutilise la redondance de taches.

MVP:

- `redundancy_factor = 3` pour `search_audit`;
- result format normalise;
- majority/consensus simple.

### Feature 11 - Recherche "watch"

**Question utilisateur:**

```text
"Alerte-moi quand une app de traduction verified release ajoute support OCR."
```

RRV devient aussi un systeme d'abonnement:

- surveille manifests;
- surveille capabilities;
- surveille diffs de code;
- surveille nouvelles releases.

Valeur produit:

- remplace partiellement "watch GitHub" mais en reseau;
- oriente contribution et adoption;
- cree un fil d'activite intelligent.

MVP:

- saved query locale;
- re-run a chaque nouveau `SearchManifest`;
- notification shell.

### Feature 12 - Recherche MCP locale

Le moteur doit etre exposable comme MCP local-only.

Tools possibles:

```text
sbfb_search(query, filters)
sbfb_search_code(pattern, language, verified_only)
sbfb_explain_project(project_id)
sbfb_can_run(project_id, local_node_profile)
sbfb_audit_project(project_id, policy)
sbfb_find_capability(capability, constraints)
sbfb_lineage(chunk_id)
```

Valeur produit:

- Claude/Codex/GPT/local LLM peuvent utiliser la recherche SBFB;
- le repo et le reseau deviennent une base de connaissance active;
- la logique reste locale et verifiable.

---

## 5. Architecture cible

### 5.1 Vue globale

```text
Utilisateur
  |
  v
Chat IA Reseau UI
  |
  v
Query Planner
  |
  +--> Privacy Gate / PII Redaction
  |
  +--> Local Search
  |      - metadata
  |      - FTS
  |      - vector
  |      - symbols
  |      - risks
  |
  +--> Network Expansion (opt-in)
  |      - Browse entries
  |      - SearchManifest gossip/docs
  |      - IndexShard iroh-blobs
  |
  +--> Active Verification (opt-in)
  |      - task_submit
  |      - worker quorum
  |      - build/test/audit
  |
  v
Ranker + Proof Builder
  |
  v
Answer Synthesizer
  |
  v
Result cards + cited response + actions
```

### 5.2 Local index store

Nom de travail:

```text
search_index.sqlite
```

Tables recommandees:

```sql
projects(
  project_id text primary key,
  project_name text not null,
  category text not null,
  description text not null,
  source text not null,
  status text not null,
  curator_pubkey text,
  repo_url text,
  provenance_hash text,
  artifact_hash text,
  archive_ticket text,
  is_open_source integer not null,
  source_state text not null,
  last_seen_at integer not null,
  last_probed_at integer
);

releases(
  release_id text primary key,
  project_id text not null,
  repo_url text,
  commit_sha text,
  artifact_hash text,
  provenance_hash text,
  source_state text not null,
  build_quorum_hash text,
  indexed_at integer not null
);

source_files(
  file_id text primary key,
  release_id text not null,
  path text not null,
  language text,
  content_hash text not null,
  size_bytes integer not null
);

chunks(
  chunk_id text primary key,
  file_id text not null,
  release_id text not null,
  path text not null,
  language text,
  start_byte integer not null,
  end_byte integer not null,
  start_line integer,
  end_line integer,
  content_hash text not null,
  text text not null,
  chunk_kind text not null,
  symbol_name text
);

symbols(
  symbol_id text primary key,
  file_id text not null,
  release_id text not null,
  name text not null,
  kind text not null,
  language text,
  line integer,
  chunk_id text
);

capabilities(
  capability_id text primary key,
  project_id text not null,
  release_id text,
  name text not null,
  confidence real not null,
  source text not null,
  evidence_chunk_id text
);

risk_findings(
  finding_id text primary key,
  project_id text not null,
  release_id text,
  severity text not null,
  rule_id text not null,
  message text not null,
  chunk_id text,
  status text not null
);

lineage_edges(
  edge_id text primary key,
  from_chunk_id text not null,
  to_chunk_id text not null,
  relation text not null,
  generation_task_id text,
  confidence real not null
);

search_manifests(
  manifest_id text primary key,
  project_id text not null,
  release_id text,
  manifest_hash text not null,
  index_blob_hash text,
  signer_pubkey text,
  received_at integer not null,
  verified integer not null
);

worker_answers(
  answer_id text primary key,
  query_hash text not null,
  task_id text not null,
  worker_pubkey text,
  result_hash text,
  verdict text,
  created_at integer not null
);
```

Notes:

- `chunks.text` alimente FTS5.
- `chunks.embedding` peut vivre dans une table `vec_chunks` si `sqlite-vec`.
- `risk_findings` permet le mode audit.
- `lineage_edges` sert Generation Composee.
- `search_manifests` sert l'expansion P2P.

### 5.3 Identite des chunks

Un chunk ne doit pas etre identifie par "path + line", car les lignes bougent.

Proposition:

```text
chunk_id = blake3(
  "sbfb.chunk.v1" ||
  artifact_hash ||
  file_content_hash ||
  normalized_path ||
  start_byte ||
  end_byte ||
  chunk_content_hash ||
  chunker_version
)
```

Avantage:

- stable pour citation;
- dedup possible entre projets;
- lineage possible;
- verification locale possible.

### 5.4 Pipeline d'ingestion

```text
1. Discover
   - BrowseEntry
   - ProjectAnnouncement
   - curator list
   - direct project

2. Verify metadata
   - repo_url
   - provenance_hash
   - is_open_source
   - source state

3. Fetch artifact
   - archive_ticket
   - iroh-blobs
   - local cache

4. Unpack safely
   - size limits
   - path traversal guard
   - binary skip

5. Extract files
   - JS/TS/Rust/Python/HTML/CSS/MD first
   - manifests: package.json, Cargo.toml, pyproject.toml, SBFB.json

6. Chunk
   - markdown by headings
   - code by functions/classes when possible
   - fallback token/line chunks

7. Lexical index
   - FTS5 MVP
   - Tantivy later

8. Embeddings
   - local embedding model
   - model digest stored
   - re-embedding versioned

9. Code intelligence
   - tree-sitter symbols
   - imports/routes/events
   - bridge method usage

10. Risk pass
   - static rules
   - dependency scan light
   - permission surface

11. Capability inference
   - manifest declarations
   - code evidence
   - LLM optional but evidence-bound

12. Publish optional manifest
   - SearchManifest v1
   - index shard blob
   - signed summary
```

### 5.5 SearchManifest v1

`SearchManifest` est le resume public, signe et leger, qu'un noeud peut publier pour aider les autres a decouvrir son index sans tout recuperer.

Schema conceptuel:

```json
{
  "v": 1,
  "type": "sbfb.search_manifest",
  "project_id": "...",
  "release_id": "...",
  "project_name": "...",
  "category": "...",
  "repo_url": "...",
  "commit_sha": "...",
  "artifact_hash": "blake3:...",
  "provenance_hash": "blake3:...",
  "is_open_source": true,
  "source_state": "verified_release",
  "index_blob_hash": "blake3:...",
  "index_kind": ["fts", "vector", "symbols", "risk", "capabilities"],
  "chunk_count": 1240,
  "symbol_count": 312,
  "capabilities": ["translation", "offline", "task_submit"],
  "risk_summary": {
    "critical": 0,
    "high": 0,
    "medium": 2,
    "low": 4
  },
  "embedding_model": {
    "name": "nomic-embed-text",
    "digest": "..."
  },
  "chunker_version": "sbfb-chunker-v1",
  "created_at": "2026-05-12T20:00:00Z",
  "signer_pubkey": "...",
  "signature": "..."
}
```

Transport possible:

- gossip topic pour annoncer un nouveau manifest;
- iroh-docs pour etat multiwriter;
- iroh-blobs pour l'index shard compresse.

### 5.6 IndexShard

Un `IndexShard` est un blob telechargeable contenant:

- metadata projet;
- chunks hashes;
- FTS compact;
- embeddings optionnels;
- symboles;
- capabilities;
- risk findings.

Formats possibles:

| Format | Avantage | Limite | Verdict |
|---|---|---|---|
| SQLite `.db.zstd` | simple, inspectable, compatible local | attention compat extension vec | bon MVP |
| Tantivy segment | performant lexical | Rust-specific, schema a gerer | bon v2 |
| JSONL zstd | simple a verifier | plus lent a charger | bon fallback |

Recommandation:

```text
MVP: JSONL zstd ou SQLite sans extension exotique.
Local runtime: SQLite + sqlite-vec.
V2: Tantivy pour lexical high quality.
```

### 5.7 Privacy modes

La recherche reseau ne doit pas envoyer toutes les questions au reseau par defaut.

Modes:

| Mode | Description | Defaut |
|---|---|---|
| `LocalOnly` | requete et index restent locaux | oui |
| `FederatedRedacted` | PII redaction puis broadcast limite | non |
| `FederatedPublic` | requete publique assumee | non |
| `AuditQuorum` | envoie une tache a N workers | non |
| `WebSidecar` | interroge SearXNG/web | non |

UX:

- la barre de recherche doit afficher le mode actif;
- toute sortie du noeud doit etre explicite;
- `piiRedact` doit etre appele avant les modes reseau;
- les requetes sensibles doivent rester locales par defaut.

### 5.8 Ranking multi-signal

Formule de depart:

```text
score =
  0.18 * lexical_score
+ 0.18 * semantic_score
+ 0.12 * symbol_score
+ 0.12 * proof_score
+ 0.10 * availability_score
+ 0.10 * safety_score
+ 0.08 * usage_score
+ 0.06 * freshness_score
+ 0.06 * diversity_score
- risk_penalties
- stale_penalties
- license_penalties
```

Definitions:

| Signal | Source |
|---|---|
| `lexical_score` | FTS/BM25/path/symbol exact |
| `semantic_score` | embeddings |
| `symbol_score` | tree-sitter, ctags-like |
| `proof_score` | verified release, provenance, build quorum |
| `availability_score` | browse reachability, providers, cache local |
| `safety_score` | risk findings, consent compatibility |
| `usage_score` | reuse count, forks, task success, kudos futur |
| `freshness_score` | release recente, source reachable |
| `diversity_score` | evite monoculture de chunks |

Les poids doivent etre config et mesurables. Le premier MVP peut utiliser une formule simple et logger les scores pour calibration.

### 5.9 Niveaux de preuve

| Niveau | Nom | Condition |
|---|---|---|
| P0 | Indexed claim | resultat vient d'un index local, pas encore verifie |
| P1 | Content proof | chunk hash + artifact hash verifiables |
| P2 | Source proof | repo + commit + provenance presents |
| P3 | Verified release | etat SBFB open source verifie |
| P4 | Rebuilt proof | build/test reexecute par worker |
| P5 | Quorum proof | N workers independants convergent |
| P6 | Live run proof | app ou capability executee maintenant |

La UI devrait afficher un badge:

```text
P3 Verified Release
P5 Quorum Verified
P1 Content Only
```

---

## 6. Experience utilisateur

### 6.1 Barre unique, modes visibles

La recherche peut rester simple:

```text
[ LocalOnly v ]  Que veux-tu chercher dans le reseau ?
```

Mais les filtres doivent etre visibles:

- Verified only;
- Reachable now;
- Can run here;
- Code only;
- Apps only;
- Ask network;
- Include web sidecar;
- Safe for worker L2.

### 6.2 Result card

Chaque resultat doit avoir:

- titre;
- type;
- resume;
- raison du match;
- preuve;
- risques;
- actions.

Actions minimales:

| Action | Effet |
|---|---|
| `Open` | ouvrir la page projet/app |
| `Inspect` | voir fichiers/chunks/preuves |
| `Run` | lancer si compatible |
| `Fork` | demarrer nouveau projet |
| `Use as source` | envoyer a Generation Composee |
| `Audit` | lancer analyse locale ou reseau |
| `Cache` | garder les blobs/index localement |

### 6.3 Reponse Chat IA

Format recommande:

```text
Verdict:
  Reponse courte.

Preuves:
  - Projet A, fichier X, chunk hash Y, commit Z.
  - Projet B, fichier X, chunk hash Y, commit Z.

Comparaison:
  Approche A vs B.

Actions:
  [Ouvrir] [Auditer] [Fork] [Generer avec ces sources]
```

La reponse doit separer:

- ce qui est lu dans l'index;
- ce qui est infere par l'IA;
- ce qui a ete verifie activement.

### 6.4 Exemples de requetes qui montrent la difference

```text
"Trouve les apps open source verifiees qui peuvent faire OCR offline."
"Quels projets utilisent bridge.submitTask et ont appele piiRedact avant ?"
"Quel pattern CRDT est le plus reutilise dans le reseau ?"
"Est-ce que cette release a encore une source reverifiable ?"
"Demande a 3 workers de tester le build Linux de ce projet."
"Trouve toutes les apps qui ont ajoute fetch() externe depuis leur derniere release."
"Montre les descendants de ce chunk vulnerable."
"Cree une app en reutilisant seulement des chunks P3+ et licence MIT/AGPL compatible."
"Quels projets sont disponibles maintenant sur au moins 2 peers ?"
"Quels workers peuvent executer un resume 70B avec moins de 12GB VRAM ?"
```

---

## 7. Integration avec les briques SBFB

### 7.1 iroh-blobs

Role:

- artefacts d'apps;
- blobs d'index;
- proof bundles;
- logs de build;
- snapshots de resultats.

Design:

```text
artifact_hash -> source archive
index_blob_hash -> IndexShard
proof_blob_hash -> AnswerProof bundle
```

### 7.2 iroh-gossip / iroh-docs

Role:

- annoncer un `SearchManifest`;
- synchroniser les manifests publics;
- propager les updates de capabilities;
- alimenter Watch mode.

Design:

```text
topic: nexus-grid/search/v1
message types:
  - search_manifest
  - search_manifest_revoke
  - search_watch_hint
```

### 7.3 Browse

Role:

- point d'entree discovery;
- status reachability;
- curator/public direct source;
- metadata affichable.

RRV ne remplace pas Browse. Il l'enrichit.

```text
Browse = catalogue lisible.
RRV = moteur d'intelligence sur le catalogue.
```

### 7.4 Publish model

Role:

- gate de confiance;
- ranking proof;
- filtre worker L2;
- badges.

RRV doit reprendre exactement les etats existants:

- Local Draft;
- Unverified Build;
- Verified Release;
- Stale Source.

Pas de nouvelle terminologie floue.

### 7.5 Task pipeline

Role:

- active verification;
- summarization distribuee;
- audit quorum;
- build/test proof;
- generation composee.

Attention:

- les prompts de recherche peuvent contenir du prive;
- le mode `LocalOnly` doit etre defaut;
- `piiRedact` avant toute task reseau;
- worker consent et caps doivent etre respectes.

### 7.6 Worker quorum

Role:

- valider une reponse sensible;
- verifier un build;
- detecter resultats divergents;
- donner un niveau P5.

Cas ou quorum vaut le cout:

- securite;
- publication;
- build;
- claims de compatibilite;
- resultats qui orientent une generation composee massive.

Cas ou quorum est inutile:

- recherche locale rapide;
- explication simple;
- navigation Browse;
- questions non critiques.

### 7.7 Generation Composee

Role:

- consommer les resultats `Find pattern`;
- choisir les chunks sources;
- garder lineage;
- reinjecter le projet genere dans l'index.

RRV devient le moteur de selection de Generation Composee.

```text
Sans RRV: Generation Composee choisit des chunks surtout par similarite.
Avec RRV: elle choisit par similarite + preuve + qualite + licence + lineage + diversity.
```

### 7.8 Kudos / reputation futur

Role:

- usage score;
- author reputation;
- curator confidence;
- anti-spam;
- selection de chunks.

Attention:

- ne pas transformer la recherche en concours de popularite;
- garder `diversity_score`;
- penaliser sybil/spam;
- separer "auteur connu" de "preuve technique".

---

## 8. Roadmap proposee

Cette roadmap ne remplace pas les carries post-v1.0. Elle doit venir apres ou en parallele d'un sprint dedie, une fois les gates S61 critiques stabilisees.

### Phase R0 - Spec et dataset de tests

Objectif:

- figer le vocabulaire RRV;
- definir 30 requetes benchmark;
- definir 10 projets fixtures;
- definir scores attendus.

Deliverables:

- `docs/search/RRV_SPEC.md`;
- `tests/fixtures/search_projects/*`;
- benchmark JSON de requetes.

Critere PASS:

- une requete a toujours un expected class/type;
- les fixtures couvrent verified/unverified/stale/reachable/unreachable.

### Phase R1 - Index local lexical

Objectif:

- indexer projets caches;
- FTS5 sur chunks;
- result cards basiques.

Deliverables:

- `crates/nexus-search-core` ou module daemon;
- table `chunks`;
- endpoint local `GET /search?q=...`;
- UI search minimale.

Critere PASS:

- recherche exacte par nom de fonction;
- filtre verified only;
- citation fichier/path/chunk.

### Phase R2 - Embeddings et hybrid ranking

Objectif:

- ajouter embeddings locaux;
- combiner lexical + vectoriel;
- logging des scores.

Deliverables:

- integration Ollama embedding ou embedder local;
- `sqlite-vec`;
- ranker multi-signal v0.

Critere PASS:

- requetes semantiques trouvent des chunks sans mots exacts;
- requetes code exactes ne regressent pas.

### Phase R3 - Symbols et capability inference

Objectif:

- tree-sitter JS/TS/Rust/Python;
- extraction symboles;
- detection bridge methods;
- capabilities evidence-bound.

Deliverables:

- table `symbols`;
- table `capabilities`;
- filtre UI capability;
- scanner `bridge.submitTask`, `piiRedact`, `storage_*`.

Critere PASS:

- "apps qui utilisent task_submit" retourne les bons projets;
- chaque capability a une evidence chunk.

### Phase R4 - Proof bundle

Objectif:

- rendre la recherche verifiable;
- afficher provenance;
- niveaux P0-P3.

Deliverables:

- `SearchResultProof`;
- badges P0/P1/P2/P3;
- lien artifact/provenance/source.

Critere PASS:

- impossible d'afficher "open source verifie" sans provenance;
- stale source visible comme degraded trust.

### Phase R5 - SearchManifest et index shards P2P

Objectif:

- passer du local-only a reseau opt-in;
- publier manifests;
- fetch index shards.

Deliverables:

- `SearchManifest v1`;
- topic gossip/docs;
- `IndexShard` blob;
- verification signature/hash.

Critere PASS:

- noeud A indexe projet;
- noeud B recupere manifest;
- noeud B peut fetch shard;
- noeud B verifie hashes avant usage.

### Phase R6 - Recherche active

Objectif:

- `Ask network`;
- build/test/audit via workers;
- resultats signes.

Deliverables:

- task type `search_verify`;
- result schema;
- quorum optionnel;
- UI "verification active en cours".

Critere PASS:

- une requete peut declencher 3 workers;
- le resultat affiche accord/divergence;
- le proof level passe P4/P5.

### Phase R7 - Generation Composee integration

Objectif:

- utiliser RRV pour choisir les sources de generation;
- stocker lineage;
- eviter monoculture.

Deliverables:

- selector `generation_sources`;
- lineage table;
- compatibility license filter;
- diversity scoring.

Critere PASS:

- chaque app generee garde ses sources chunk;
- recherche lineage fonctionne;
- un chunk vulnerable peut retrouver ses descendants.

---

## 9. Anti-patterns a eviter

### 9.1 Refaire Google

Mauvais objectif:

```text
"On va crawler le web et faire un moteur generaliste."
```

Pourquoi c'est mauvais:

- cout infra enorme;
- SEO/spam;
- pas de lien direct avec SBFB;
- concurrence frontale contre acteurs impossibles a battre;
- faible differenciation.

Bon objectif:

```text
"On cherche ce que le reseau SBFB peut prouver et executer."
```

### 9.2 Tout envoyer aux workers

Mauvais objectif:

```text
"Chaque recherche devient une task IA reseau."
```

Pourquoi c'est mauvais:

- fuite privacy;
- latence;
- cout GPU;
- consent complexe;
- spam de tasks.

Bon objectif:

```text
"Local first. Reseau seulement si l'utilisateur choisit Ask network / Audit quorum."
```

### 9.3 Vector-only

Mauvais objectif:

```text
"On met tout en embeddings et c'est fini."
```

Pourquoi c'est mauvais:

- mauvais pour noms de fonctions;
- mauvais pour symboles;
- mauvais pour versions;
- hallucination de proximite;
- difficulte a expliquer le ranking.

Bon objectif:

```text
"Hybrid: FTS + vector + AST + proof + availability."
```

### 9.4 IA comme preuve

Mauvais objectif:

```text
"Le LLM dit que c'est safe donc c'est safe."
```

Pourquoi c'est mauvais:

- hallucination;
- non determinisme;
- prompt injection;
- responsabilite securite.

Bon objectif:

```text
"Scanners deterministes d'abord, LLM pour expliquer, quorum pour verifier."
```

### 9.5 Index P2P non verifie

Mauvais objectif:

```text
"Un peer m'envoie des embeddings, je les utilise directement."
```

Pourquoi c'est mauvais:

- poisoning;
- ranking manipulation;
- embeddings malveillants;
- sybil.

Bon objectif:

```text
"Chaque shard est signe, hash, sample-verifie, et degradable en confiance."
```

---

## 10. Risques reels et mitigations

| Risque | Description | Mitigation |
|---|---|---|
| Index poisoning | Peers publient faux manifests ou faux scores | signatures, provenance, sample re-index, curator trust |
| Sybil ranking | Beaucoup de faux peers boostent un projet | proof score > popularity, rate limits, reputation prudente |
| Privacy leak | requete utilisateur envoyee au reseau | LocalOnly default, PII redaction, modes explicites |
| Hallucination | Chat affirme sans preuve | citations obligatoires, separation lu/infere/verifie |
| Monoculture | Generation reutilise toujours le meme pattern | diversity score, lineage warnings |
| Copyright/licence | chunks incompatibles reutilises | license metadata, filtre generation |
| Storage bloat | index shards trop gros | manifests legers, shards opt-in, eviction |
| Latence | recherche active lente | progressive results, local first, verification async |
| Compute abuse | search_verify spam | quotas, consent, caps watts/VRAM/heures |
| Stale source | repo disparu apres publication | badge Stale Source, degraded proof, mirrors |
| Model drift | embeddings changent selon model/version | model digest, re-index versioning |
| False confidence | preuve P1 affichee comme P5 | niveaux de preuve stricts |

---

## 11. Strategie produit

### 11.1 Positionnement externe

Phrase simple:

```text
Google trouve des pages. SBFB trouve des capacites verifiables que le reseau peut executer, auditer, fork et composer.
```

Phrase contributeur:

```text
RRV est une couche de recherche pour un reseau P2P d'apps et de compute:
elle indexe code, artefacts, provenance, capabilities et preuves d'execution.
```

Phrase end-user:

```text
Demande au reseau ce qui existe, ce qui est fiable, et ce que tu peux lancer maintenant.
```

### 11.2 Pourquoi ca peut devenir une feature signature SBFB

Parce que toutes les briques SBFB convergent dessus:

- P2P discovery donne le graphe de projets;
- iroh-blobs donne les artefacts verifiables;
- publish model donne le trust;
- task pipeline donne l'execution;
- workers donnent le compute;
- bridge donne l'API app-shell;
- Generation Composee donne la boucle d'amelioration;
- Kudos/curation donnent le signal social;
- Radicle/Codeberg/GitHub donnent la resilience source.

Ce n'est pas une app parmi d'autres. C'est une couche transversale du protocole.

### 11.3 Ce que ca sert concretement

Pour un utilisateur:

- trouver une app fiable;
- savoir si elle tourne localement;
- comprendre comment elle marche;
- eviter les zips non verifies;
- demander un audit avant d'executer.

Pour un dev:

- trouver le meilleur pattern du reseau;
- comparer des implementations;
- generer avec des sources prouvees;
- retrouver les descendants d'un bug;
- publier des capabilities mieux decouvrables.

Pour un worker operator:

- savoir quelles taches sont safe;
- filtrer par open source verifie;
- inspecter le cout estime;
- refuser des patterns dangereux.

Pour un curateur:

- trouver des projets recommandables;
- detecter stale source;
- suivre les releases;
- faire des listes de qualite.

Pour SBFB comme reseau:

- reduire le cold start;
- augmenter la qualite moyenne;
- rendre visible la confiance;
- transformer chaque publication en matiere premiere pour les suivantes.

---

## 12. Decision technique recommandee

### 12.1 MVP a construire

Le MVP RRV doit etre petit mais propre:

```text
LocalOnly Search:
  - index Browse projects already cached
  - SQLite FTS5 chunks
  - sqlite-vec embeddings
  - provenance fields in results
  - verified/reachable filters
  - Chat IA answer with citations
```

Ne pas commencer par:

- global search network;
- active worker verification;
- web crawling;
- full AST all languages;
- ranking reputation complexe.

### 12.2 Premier schema de ranking

Pour MVP:

```text
score =
  0.35 * bm25
+ 0.30 * vector
+ 0.15 * proof
+ 0.10 * availability
+ 0.10 * safety
```

Puis ajouter symboles et lineage.

### 12.3 Premier set de filtres UI

Obligatoires:

- `Verified only`;
- `Reachable now`;
- `Code`;
- `Apps`;
- `Can run here`;
- `Include unverified` off by default;
- `Ask network` separate button.

### 12.4 Premier set de requetes benchmark

Exemples:

```text
1. "apps qui utilisent task_submit"
2. "projets open source verifies"
3. "pattern CRDT storage"
4. "code qui appelle piiRedact"
5. "apps disponibles maintenant"
6. "source stale"
7. "generation composee lineage"
8. "risk eval"
9. "installer windows"
10. "offline translation"
```

---

## 13. Conclusion

Oui, SBFB peut creer un type de recherche autre que Google.

Mais le point cle est de ne pas viser "moteur web". Le bon axe est:

```text
Recherche Reseau Verifiable = search + provenance + execution + P2P + generation.
```

Le Chat IA Reseau peut devenir l'interface naturelle de cette recherche:

- il comprend la question;
- il interroge l'index local;
- il etend au reseau si l'utilisateur accepte;
- il cite des preuves;
- il propose des actions;
- il peut declencher des workers pour verifier.

La feature unique n'est pas "IA qui cherche".

La feature unique est:

```text
IA qui demande au reseau SBFB ce qui existe, ce qui est prouve,
ce qui est disponible, ce qui peut tourner, et ce qui peut etre
reutilise pour construire la suite.
```

Si SBFB reussit ca, le Chat IA Reseau devient plus qu'une app. Il devient:

- l'explorateur du reseau;
- le moteur de selection de Generation Composee;
- le tableau de confiance des releases;
- le moteur de decouverte des capabilities;
- le point d'entree naturel pour contributeurs, curateurs, devs et users.

La prochaine etape raisonnable est un sprint R&D/spec court:

1. figer `RRV_SPEC.md`;
2. creer fixtures de recherche;
3. implementer index local FTS + proof fields;
4. brancher une UI minimale dans le Chat IA Reseau;
5. mesurer sur 30 requetes benchmark avant d'ajouter le P2P.

---

## 14. Sources externes consultees

- SearXNG docs: <https://docs.searxng.org/>
- YaCy: <https://www.yacy.net/>
- Zoekt: <https://github.com/sourcegraph/zoekt>
- Tantivy: <https://github.com/quickwit-oss/tantivy>
- sqlite-vec: <https://github.com/asg017/sqlite-vec>
- pgvector: <https://github.com/pgvector/pgvector>
- Qdrant hybrid search: <https://qdrant.tech/articles/sparse-vectors/>
- LanceDB hybrid search: <https://docs.lancedb.com/search/hybrid-search>
- OpenSearch neural search: <https://docs.opensearch.org/2.14/search-plugins/neural-search/>
- Vespa hybrid search: <https://docs.vespa.ai/en/learn/tutorials/hybrid-search.html>
- LlamaIndex RAG: <https://developers.llamaindex.ai/python/framework/understanding/rag/>
- Haystack intro: <https://docs.haystack.deepset.ai/docs/intro>
- Khoj docs: <https://docs.khoj.dev/>
- Perplexica: <https://github.com/comput3ai/c3-perplexica>
- tree-sitter: <https://github.com/tree-sitter/tree-sitter>
- Semgrep docs: <https://semgrep.dev/docs/introduction>
- CodeQL docs: <https://codeql.github.com/docs/codeql-overview/about-codeql/>
- Sourcegraph SCIP/code intelligence: <https://sourcegraph.com/docs/code_intelligence/explanations/precise_code_intelligence>
- IPFS content addressing: <https://docs.ipfs.tech/concepts/content-addressing/>
- IPNI: <https://docs.ipfs.tech/concepts/ipni/>
- iroh blobs: <https://docs.iroh.computer/protocols/blobs>
- iroh overview: <https://docs.iroh.computer/what-is-iroh>

## 15. Sources repo utilisees

- [`docs/apps/CHAT_IA_RESEAU.md`](../../docs/apps/CHAT_IA_RESEAU.md)
- [`docs/apps/GENERATION_COMPOSEE.md`](../../docs/apps/GENERATION_COMPOSEE.md)
- [`docs/architecture/PUBLISH_MODEL.md`](../../docs/architecture/PUBLISH_MODEL.md)
- [`docs/architecture/SELF_HOSTED_BUILD.md`](../../docs/architecture/SELF_HOSTED_BUILD.md)
- [`web/public/sbfb-bridge.js`](../../web/public/sbfb-bridge.js)
- [`crates/nexus-shell-daemon-core/src/browse.rs`](../../crates/nexus-shell-daemon-core/src/browse.rs)
- [`crates/nexus-shell-daemon-core/src/publish.rs`](../../crates/nexus-shell-daemon-core/src/publish.rs)
- [`crates/nexus-core-rs/src/task.rs`](../../crates/nexus-core-rs/src/task.rs)
- [`.planning/roadmap_v1.0_alexandria.md`](../roadmap_v1.0_alexandria.md)
