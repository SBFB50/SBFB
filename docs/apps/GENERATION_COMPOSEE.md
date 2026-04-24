# SBFB — Generation composee : chaque app est construite sur les meilleures du reseau

**Date** : 2026-04-13
**Statut** : design, pre-implementation
**Prerequis** : Chat IA reseau (CHAT_IA_RESEAU.md) + bridge postMessage

---

## Le concept

Quand un utilisateur demande de creer un projet, le systeme :
1. Cherche dans tout le reseau les meilleurs morceaux de code
2. Les combine et les ameliore via le LLM distribue
3. Genere une app complete, testee, deployee
4. Cette app rejoint l'index et enrichit les futures creations

Chaque app est construite sur les epaules de toutes les
precedentes. Les bonnes pratiques se propagent automatiquement.
Les mauvais patterns meurent par selection naturelle.

---

## Exemple concret

```
Utilisateur : "Cree-moi un jeu d'echecs multijoueur"

Le systeme cherche dans l'index (847 projets, 12K chunks) :

  CRDT game state :
    → dnd-p2p/crdt_state.js (score qualite: 94/100)
      "Meilleur pattern CRDT pour du game state tour par tour.
       Utilise par 12 autres projets. Zero conflit reporte."

  Systeme de tour :
    → dnd-p2p/turns.js (score: 91/100)
      "Gestion de tour avec timeout et skip automatique."

  UI temps reel :
    → llm-chat/streaming.js (score: 88/100)
      "Pattern streaming avec indicateur de saisie."

  File d'attente :
    → render-farm/task_queue.js (score: 92/100)
      "Queue distribuee avec reprise sur crash."

Le LLM 70B genere en combinant :
  → chess/index.html     : UI echiquier SVG
  → chess/game_state.js  : CRDT adapte de dnd-p2p pour les echecs
  → chess/multiplayer.js : sync inspire de llm-chat
  → chess/timer.js       : horloge avec timeout de dnd-p2p/turns.js
  → chess/ai_opponent.js : adversaire IA via task pipeline

Validation automatique :
  → Scan IA : SAFE (3/3 noeuds)
  → Code source : lie au prompt de generation

Publication :
  → L'app est deployee sur le reseau
  → Indexee : chess/game_state.js rejoint l'index
  → Metadata : "Genere a partir de : dnd-p2p, llm-chat.
    Modele : llama3.1:70b. Score scan : 94/100."

Prochain projet :
  → Un dev veut un jeu de dames
  → Le systeme trouve chess/game_state.js (meilleur que
    dnd-p2p pour les jeux de plateau car deja adapte)
  → Le pattern s'ameliore a chaque reutilisation
```

---

## La courbe d'amelioration — honnete

Ce n'est **pas exponentiel**. C'est une amelioration composee a
rendements decroissants — une courbe en S.

```
Qualite
  │
  │                          ╭──────────── plateau
  │                     ╭────╯
  │                ╭────╯
  │           ╭────╯
  │      ╭────╯
  │  ╭───╯  ← croissance rapide (app 5-50)
  │──╯
  │ ← cold start (app 1-5)
  └──────────────────────────── Nombre d'apps
```

| Phase | Apps | Reutilisation | Amelioration |
|-------|------|---------------|-------------|
| Cold start | 1-5 | 0% | Code from scratch |
| Croissance | 6-20 | ~40% | Bonne — evite de reinventer |
| Acceleration | 21-50 | ~70% | Les bons patterns dominent |
| Maturite | 50-200 | ~85% | Assemblage intelligent |
| Plateau | 200+ | ~90% | Rendements decroissants |

**Pourquoi pas exponentiel** : les patterns convergent. Apres 50
apps, la majorite des problemes courants (CRDT sync, UI chat,
game state, file d'attente) sont resolus dans l'index. Les
nouveaux projets ont des besoins specifiques que l'index ne
couvre pas. C'est la meme dynamique que npm (2M packages mais
95% du code reel utilise les memes 500 packages).

**La recherche confirme** : les lois d'echelle des LLM (Kaplan
2020) montrent que la performance suit une loi de puissance
(power law), pas une exponentielle. Plus de donnees = mieux,
mais avec des rendements decroissants.

---

## Ce qui EST reellement puissant

### 1. Zero cold start pour les nouveaux devs

```
Aujourd'hui (npm, GitHub) :
  Dev veut une app CRDT → Google → 15 tutos → copie-colle
  → adapte → debug 3h → ca marche

SBFB avec le systeme :
  Dev dit "cree une app avec sync CRDT" → le systeme trouve
  le meilleur pattern CRDT du reseau, deja teste, deja
  optimise → genere le code adapte → deploy en 30 secondes
```

### 2. Propagation automatique des bonnes pratiques

Si le D&D a un excellent pattern CRDT et que le jeu d'echecs le
reutilise, et qu'ensuite le jeu de cartes reutilise la version
amelioree du jeu d'echecs — les bonnes pratiques se propagent
sans documentation, sans tuto, sans conference.

Le code qui marche survit et se reproduit. Le code fragile n'est
pas selectionne par l'index (score de qualite bas).

### 3. Le plancher de qualite monte

La pire app du reseau est quand meme construite a partir des
meilleurs morceaux existants. Sur npm, le plancher c'est
`left-pad`. Sur SBFB, le plancher c'est le meilleur CRDT
pattern du reseau.

### 4. Tracabilite totale

Chaque app generee inclut dans ses metadata :
- Les projets sources utilises
- Les chunks specifiques reutilises
- Le modele LLM et sa version
- Le score de scan IA

N'importe qui peut remonter la chaine : "cette fonction vient
du D&D, qui l'avait adaptee du chat, qui l'avait creee from
scratch". C'est un arbre genealogique du code.

---

## Architecture

```
┌────────────────────────────────────────────────────┐
│  "Cree-moi un jeu d'echecs multijoueur"           │
│                                                    │
│  1. RECHERCHE (index RAG local)                    │
│     → Top 20 chunks de 8 projets                   │
│     → Filtre : qualite (scan score), pertinence,   │
│       licence compatible, diversite de source       │
│                                                    │
│  2. GENERATION (LLM 70B distribue)                 │
│     → Prompt avec les chunks + consignes            │
│     → Output : 4-6 fichiers JS/HTML/CSS             │
│     → Streaming temps reel : l'utilisateur voit     │
│       le code se generer                            │
│                                                    │
│  3. PREVIEW (iframe sandbox)                       │
│     → L'app generee est rendue dans une iframe      │
│     → L'utilisateur teste avant de publier          │
│     → "Ca te plait ? [Publier] [Modifier] [Refaire]"│
│                                                    │
│  4. VALIDATION (scan IA distribue)                 │
│     → 3 noeuds scannent le code genere              │
│     → SAFE → pret a publier                         │
│     → SUSPICIOUS → l'utilisateur review             │
│                                                    │
│  5. PUBLICATION                                    │
│     → Zip + deploy sur le reseau                    │
│     → Metadata : sources, modele, scan score        │
│     → Indexation : les chunks rejoignent l'index    │
│                                                    │
│  6. EVOLUTION COMMUNAUTAIRE                        │
│     → Un dev fork l'app et l'ameliore               │
│     → La version amelioree a un meilleur score      │
│     → L'index prefere la version amelioree          │
│     → Le prochain projet herite de l'amelioration   │
└────────────────────────────────────────────────────┘
```

---

## Le modele d'evolution biologique

Le systeme fonctionne comme la selection naturelle :

```
MUTATION  : le LLM genere des variations en combinant des patterns
SELECTION : les apps utilisees et bien notees montent dans l'index
HERITAGE  : chaque nouvelle app herite des meilleurs genes (chunks)
EXTINCTION: les apps non utilisees descendent dans l'index
SPECIATION: des branches de patterns emergent pour des niches
            (jeux, IoT, chat, etc.)
```

| Biologie | SBFB |
|----------|------|
| Gene | Chunk de code |
| Organisme | App complete |
| Fitness | Score qualite (scan + usage + reputation auteur) |
| Reproduction | Reutilisation du chunk dans une nouvelle app |
| Mutation | Le LLM adapte le chunk au nouveau contexte |
| Selection naturelle | L'index favorise les chunks les plus reutilises et les mieux notes |
| Environnement | Les besoins des utilisateurs du reseau |

---

## Score de qualite d'un chunk dans l'index

```
score(chunk) =
    0.3 × scan_score          (le code est-il safe ?)
  + 0.3 × reuse_count         (combien d'apps l'utilisent ?)
  + 0.2 × author_reputation   (l'auteur est-il fiable ?)
  + 0.1 × freshness           (le code est-il recent ?)
  + 0.1 × diversity_bonus     (le pattern est-il unique ?)

Un chunk avec score > 0.8 est "gene dominant" — propose en
priorite par le systeme de generation.
Un chunk avec score < 0.3 est "gene recessif" — ignore sauf
si explicitement demande.
```

---

## Risques honnetes

| Risque | Pourquoi c'est reel | Mitigation |
|--------|-------------------|-----------|
| **Monoculture de code** | Si tout le monde reutilise le meme pattern CRDT, un bug affecte tout le reseau | Le score `diversity_bonus` favorise les alternatives. Le scan IA detecte les patterns a risque. |
| **Hallucination LLM** | Le LLM combine 3 bons patterns en quelque chose qui ne marche pas | L'etape 3 (preview) laisse l'utilisateur tester. L'etape 4 (scan) valide automatiquement. |
| **Copyright / licences** | App A en AGPL + App B en MIT = probleme | L'index stocke la licence de chaque chunk. Le generateur filtre par compatibilite. |
| **Pollution de l'index** | 1000 apps mediocres generees automatiquement | Rate-limit sur la generation (5/jour/noeud). Le score de qualite fait descendre les apps non utilisees. |
| **Le LLM copie sans comprendre** | Pattern CRDT pour du D&D applique a un systeme bancaire | Le prompt inclut le contexte du nouveau projet. L'utilisateur valide en preview. |
| **Boucle de degradation** | Si une mauvaise app est reutilisee, la mauvaise qualite se propage | Le scan IA + le score de qualite + l'usage reel filtrent. Les chunks non reutilises meurent naturellement. |

---

## Effort supplementaire vs le Chat IA

Le Chat IA reseau (CHAT_IA_RESEAU.md) construit deja l'index RAG.
La generation composee ajoute :

| Composant | LOC | Temps |
|-----------|-----|-------|
| Prompt de generation (template multi-source) | ~200 | 1.5h |
| Preview iframe avant publication | ~150 | 1h |
| Metadata tracabilite (sources, modele, score) | ~100 | 0.5h |
| Score de qualite des chunks | ~150 | 1h |
| Filtre de licence compatible | ~100 | 0.5h |
| UI "Creer un projet" dans le shell | ~300 | 2h |
| **Total supplementaire** | **~1000 LOC** | **~6.5h** |

Faisable en **1 session** apres le Chat IA reseau.

---

## Ce que personne d'autre ne fait

| Plateforme | Generation de code | Acces au code des autres | Selection naturelle |
|-----------|-------------------|------------------------|-------------------|
| GitHub Copilot | Oui | Code public GitHub (centralise, opaque) | Non |
| ChatGPT | Oui | Training data fige | Non |
| Cursor | Oui | Ton projet seulement | Non |
| npm | Non (c'est un registry) | Packages individuels | Popularite (downloads) |
| **SBFB** | **Oui (LLM distribue)** | **Tous les projets du reseau P2P** | **Oui (score qualite + usage)** |

La combinaison **generation IA + index P2P + selection naturelle**
n'existe nulle part. C'est le differenciateur ultime de la
plateforme.
