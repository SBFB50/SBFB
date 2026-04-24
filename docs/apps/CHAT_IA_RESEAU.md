# SBFB — Chat IA avec acces a tous les projets du reseau

**Date** : 2026-04-13
**Statut** : design, pre-implementation
**Priorite** : strategique — flywheel d'adoption

---

## Le concept

Un chat IA integre au shell SBFB qui repond a tes questions en
ayant acces au **code source de chaque projet publie sur le reseau
P2P**. Tourne sur les GPU distribues de la communaute. Zero cloud,
zero abonnement, zero compte.

```
Utilisateur : "Comment l'app D&D gere les jets de des ?"

IA SBFB : "Dans le projet dnd-p2p (publie par Alice, 847 stars),
les des sont generes cote client via crypto.getRandomValues()
dans dice.js:42. Le resultat est ecrit dans le CRDT game state
avec le timestamp et le node_id pour transparence. Voici le code :

  const roll = (sides) => {
    const buf = new Uint32Array(1);
    crypto.getRandomValues(buf);
    return (buf[0] % sides) + 1;
  };

Deux autres projets sur le reseau utilisent une approche
differente : jeu-cartes (par Bob) utilise un seed partage..."
```

---

## Pourquoi c'est l'app la plus strategique

C'est le **flywheel d'adoption** :
1. Plus de projets publies → meilleur index IA
2. Meilleur index IA → le chat est plus utile
3. Chat plus utile → plus de devs publient
4. Plus de devs → plus de GPU → meilleur LLM → boucle

Aucune autre app ne cree cette boucle de renforcement.

---

## Comparaison avec l'existant

| Plateforme | Chat IA | Acces au code des apps | P2P | Gratuit |
|-----------|---------|----------------------|-----|---------|
| ChatGPT | Oui | Non (copier-coller) | Non | $20/mois |
| GitHub Copilot | Oui | Ton repo seulement | Non | $10/mois |
| Claude Code | Oui | Ton workspace seulement | Non | $20/mois |
| Cursor | Oui | Ton projet seulement | Non | $20/mois |
| **SBFB Chat** | **Oui** | **Tous les projets du reseau** | **Oui** | **Gratuit** |

**Personne ne fait ca.** Un assistant IA qui connait tout
l'ecosysteme d'apps d'un reseau entier. Comme si Copilot avait
acces a tous les repos GitHub en meme temps — mais sans cloud,
sans abonnement, et avec les GPU de la communaute.

---

## Architecture — RAG distribue sans serveur central

```
┌─────────────────────────────────────────────────┐
│  Chat IA (dans le shell SBFB)                   │
│                                                 │
│  Utilisateur tape une question                  │
│           │                                     │
│           ▼                                     │
│  1. Embedding de la question (nomic-embed-text) │
│           │                                     │
│           ▼                                     │
│  2. Recherche dans l'index vectoriel local      │
│     (cosine similarity, top 10 chunks)          │
│           │                                     │
│           ▼                                     │
│  3. Recupere les chunks de code pertinents      │
│     (depuis les zips deja en cache blob store)  │
│           │                                     │
│           ▼                                     │
│  4. Construit le prompt RAG                     │
│     question + chunks + metadata projet         │
│           │                                     │
│           ▼                                     │
│  5. Submit task au pipeline GPU distribue       │
│     → un worker genere la reponse               │
│     → streaming temps reel mot par mot          │
│           │                                     │
│           ▼                                     │
│  6. Affiche la reponse avec liens vers le code  │
└─────────────────────────────────────────────────┘
```

---

## Index local — comment ca marche sans serveur

Chaque noeud indexe les projets qu'il connait. Pas de serveur
central d'index. Plus tu browse, plus ton index est riche.

```
1. Le daemon voit un nouveau projet dans Browse
   → Telecharge le zip (deja fait pour l'iframe)
   → Decompresse les fichiers source

2. Pour chaque fichier .js/.ts/.py/.html/.rs/.md/.css :
   → Decoupe en chunks de ~500 tokens
   → Genere un embedding via Ollama (nomic-embed-text, 137M params)
     nomic-embed-text tourne sur n'importe quel GPU, meme GTX 1650
   → Stocke embedding + chunk + metadata dans SQLite local

3. Quand l'utilisateur pose une question :
   → Embedding de la question (meme modele)
   → Recherche des 10 chunks les plus proches (cosine similarity)
   → Construit le prompt : question + chunks + metadata
   → Submit au pipeline GPU → reponse streaming
```

### Pourquoi SQLite et pas un vector DB

- Zero dep supplementaire (SQLite est deja dans le stack)
- sqlite-vec extension (~50 LOC d'integration) fait de la recherche
  cosine sur des embeddings stockes en BLOB
- Pour ~10K chunks (50 projets de taille moyenne), la recherche
  brute-force prend <10ms — pas besoin d'index HNSW
- Quand le reseau grandit a 10K+ projets, migration vers un index
  plus efficace (usearch, faiss) si necessaire

---

## Le prompt RAG

```
Tu es un assistant expert en code pour la plateforme SBFB.
Tu as acces au code source des projets publies sur le reseau P2P.

Projets connus sur le reseau : {nombre_projets}
Chunks indexes : {nombre_chunks}

Voici les extraits de code les plus pertinents pour la question :

--- Projet: {nom_projet_1} (par {auteur}, {stars} stars) ---
Fichier: {chemin_fichier}
```{langage}
{chunk_code}
```

--- Projet: {nom_projet_2} (par {auteur}) ---
Fichier: {chemin_fichier}
```{langage}
{chunk_code}
```

[... top 10 chunks ...]

Question de l'utilisateur : {question}

Reponds en :
1. Citant les projets et fichiers specifiques
2. Montrant le code pertinent
3. Comparant les approches si plusieurs projets sont pertinents
4. Suggerant des ameliorations si demande
```

---

## Exemples d'utilisation

| Question | Ce que l'IA fait |
|----------|------------------|
| "Comment marche le combat dans le D&D P2P ?" | Cherche dans dnd-p2p, trouve les fonctions combat, explique avec le code |
| "Quel projet utilise le mieux les CRDT ?" | Compare les patterns CRDT de tous les projets, analyse les approches |
| "Aide-moi a ajouter un graphique a l'app capteurs" | Lit le code, genere le diff, l'utilisateur peut l'appliquer |
| "Montre-moi toutes les apps avec du streaming audio" | Cherche les patterns audio dans l'index, liste avec extraits |
| "Trouve les bugs de secu dans ce projet" | Analyse le code avec le prompt de scan IA |
| "Cree une app todo basee sur le style du D&D" | Utilise le code D&D comme reference, genere une nouvelle app |
| "Compare les systemes d'auth de 3 projets" | Extrait les modules d'auth, compare cote a cote |
| "Ce projet a-t-il des deps vulnerables ?" | Parse le package.json/Cargo.toml, verifie les versions |

---

## Mode P2P optionnel — index partage

En option, les noeuds peuvent **partager leur index** pour
enrichir celui des autres :

```
Mode local (par defaut) :
  → Chaque noeud indexe seulement les projets qu'il a browse
  → Fonctionne offline
  → Zero dependance sur les autres

Mode P2P (opt-in) :
  → Les embeddings sont publies dans un iroh-doc partage
  → Un nouveau noeud recupere l'index du reseau instantanement
  → Pas besoin de re-telecharger + re-embedd tous les projets
  → Le noeud verifie les embeddings en recalculant un sample
```

Le mode P2P transforme le chat IA d'un outil individuel en une
**intelligence collective** : chaque noeud qui indexe un nouveau
projet enrichit le savoir de tout le reseau.

---

## Ce qui existe deja dans le stack SBFB

| Brique | Statut | Detail |
|--------|--------|--------|
| Zips de projets en cache | **Existe** | blob store iroh |
| Metadata projets (nom, auteur, desc) | **Existe** | BrowseEntry |
| Task pipeline GPU | **Existe** | dispatcher + worker + Ollama |
| Ollama embedding (nomic-embed-text) | **Natif Ollama** | `ollama pull nomic-embed-text` |
| SQLite dans le coordinator | **Existe** | aiosqlite deja utilise |
| Bridge postMessage | **Sprint 13** | Prerequis |
| Streaming reponse | **A ajouter** | Ollama supporte, pipeline pas encore |

---

## Effort d'implementation

| Composant | LOC | Temps |
|-----------|-----|-------|
| Index vectoriel SQLite + sqlite-vec | ~300 | 2.5h |
| Chunking des sources (decoupe + dedup) | ~200 | 1.5h |
| Embedding via Ollama nomic-embed-text | ~150 | 1h |
| RAG prompt builder (question + chunks + metadata) | ~150 | 1h |
| Chat UI dans le shell (composant React) | ~400 | 3h |
| Integration bridge postMessage | ~100 | 0.5h |
| Auto-index a chaque nouveau projet browse | ~200 | 1.5h |
| **Total** | **~1500 LOC** | **~11h (2 sessions)** |

---

## Interface

```
┌──────────────────────────────────────────────────┐
│  SBFB Chat IA                    [Index: 847 ▼] │
│──────────────────────────────────────────────────│
│                                                  │
│  ┌─ Utilisateur ─────────────────────────────┐   │
│  │ Comment l'app D&D gere le systeme de      │   │
│  │ combat ?                                  │   │
│  └───────────────────────────────────────────┘   │
│                                                  │
│  ┌─ SBFB IA (70B, 3 GPU, streaming) ────────┐   │
│  │                                           │   │
│  │ Dans **dnd-p2p** (par Alice), le combat   │   │
│  │ est gere dans `combat.js` :               │   │
│  │                                           │   │
│  │ ```js                                     │   │
│  │ function resolveCombat(attacker, target) { │   │
│  │   const roll = dice(20);                  │   │
│  │   const hit = roll + attacker.stats.for   │   │
│  │     >= target.ac;                         │   │
│  │   if (hit) {                              │   │
│  │     const dmg = dice(weapon.damage) ▊     │   │
│  │ ```                                       │   │
│  │                                           │   │
│  │ Sources: dnd-p2p/combat.js:42-67          │   │
│  │          dnd-p2p/dice.js:12-18            │   │
│  └───────────────────────────────────────────┘   │
│                                                  │
│  ┌───────────────────────────────────────────┐   │
│  │ > Pose ta question...                     │   │
│  │                                    [Envoyer]│  │
│  └───────────────────────────────────────────┘   │
│                                                  │
│  Projets indexes: 47 │ Chunks: 12,847           │
│  GPU actif: Eve (4060 Ti) │ Modele: 70B streaming│
└──────────────────────────────────────────────────┘
```

---

## Risques et mitigations

| Risque | Impact | Mitigation |
|--------|--------|-----------|
| Index trop gros en memoire | Lenteur | SQLite + pagination, pas tout en RAM |
| Embeddings divergent entre noeuds (mode P2P) | Resultats incoherents | Meme modele (nomic-embed-text) + hash des poids dans l'attestation |
| Code malveillant indexe | L'IA cite du code dangereux | Le systeme de scan IA filtre avant indexation — seuls les projets SAFE sont indexes |
| Hallucination (l'IA invente du code) | Reponse fausse | Les chunks cites sont lies au code reel — l'utilisateur peut cliquer pour verifier |
| Context window depasse (trop de chunks) | Reponse tronquee | Limite a 10 chunks (~5000 tokens) + le prompt, reste dans 8K pour un 8B ou 32K pour un 70B |
| nomic-embed-text pas assez bon pour du code | Mauvais recall | Alternative : Ollama supporte aussi mxbai-embed-large (334M params, meilleur sur du code) |
