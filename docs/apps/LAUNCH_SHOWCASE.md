# SBFB — Triplette d'apps de demonstration launch

**Date** : 2026-04-24
**Statut** : design document, pre-implementation

## Objectif

Trois apps qui demonstrent les trois capacites distinctes de SBFB
lors du lancement. Chaque app est inutile sans le reseau P2P —
aucune ne tourne de maniere credible sur un laptop seul. Ensemble
elles prouvent que SBFB n'est pas un framework theorique mais une
plateforme fonctionnelle.

| App | Capacite demontree | Ressource cle |
|---|---|---|
| Alexandria | Stockage distribue (blobs P2P) | SSD + bande passante |
| Surveillance foret | Compute GPU partage (vision) | GPU workers |
| Donjon & Dragon | Compute LLM distribue (texte) | GPU workers + latence basse |

---

## 1. Alexandria — bibliotheque de connaissance multilingue

### Concept

Copie locale integrale de Wikipedia (324 editions linguistiques) +
StackOverflow + Project Gutenberg, accessible a toute IA locale via
un MCP server. Le contenu se distribue sur le reseau SBFB via
iroh-blobs — chaque noeud qui consulte un article le cache et le
re-sert aux suivants.

### Ce que ca demontre

- **Distribution de donnees massives** : 1.1 TB de savoir humain
  distribue en P2P sans serveur central
- **Resilience** : si le noeud source s'eteint, les peers qui ont
  cache le contenu continuent a servir
- **Valeur immediate** : n'importe quel LLM local (Ollama) accede
  a Wikipedia offline via MCP tools

### Corpus et stockage

| Source | Taille | Format |
|---|---|---|
| Wikipedia 324 langues (text + images) | ~800 GB | Dumps ZIM (Kiwix) |
| Wikidata complet (entity linking) | ~80 GB | JSON dump |
| StackOverflow data dump | ~60 GB | XML compresse |
| Project Gutenberg (70k+ livres) | ~15 GB | Texte brut |
| Index full-text (tantivy) | ~100 GB | Binaire index |
| **Total** | **~1.1 TB** | |

Tient sur un SSD 4 To avec ~2.9 TB de marge. Avec 2 ans
d'historique Wikipedia (pour le diff temporel) : ~2.1 TB total,
reste ~1.9 TB.

### Analyse cross-lingue — ce qui marche vs ce qui est survalorise

**Faisable et solide :**

- Acces local offline a Wikipedia 324 langues — trivial avec
  libzim (Python bindings, MIT)
- Cross-referencing via Wikidata IDs — le mapping inter-articles
  existe deja, ~2 semaines de dev
- Affichage comparatif N langues sur un meme sujet — outil d'analyse
  reel pour journalistes, chercheurs, analystes OSINT
- Detection d'asymetries structurelles (article 50 paragraphes en
  bengali vs 3 en anglais) — signal reel, pas besoin de NLP

**Survalorise / dangereux :**

- Score de "confiance cross-lingue" qui pretend mesurer la verite
  d'un fait : convergence = popularite d'une affirmation, pas
  veracite. 280 editions qui reprennent une erreur anglo-saxonne
  produisent un faux "confiance ELEVEE"
- Detection de manipulation automatique sans curation humaine :
  marche sur les cas flagrants (Crimee, Holodomor, Tiananmen) mais
  produit du bruit sur 99% des sujets — les divergences inter-langues
  sont a 95% culturelles, pas malveillantes
- Analyse semantique fine dans les langues low-resource (inuktitut,
  bambara, kashmiri) : les modeles multilingues (mBERT, XLM-R)
  couvrent ~100 langues correctement, qualite en chute libre pour
  les 224 restantes

**La valeur reelle est dans l'acces et la juxtaposition, pas dans
l'analyse automatisee.**

### MCP tools (4 tools via mcp SDK)

```
knowledge_search(query, lang?, limit)
  → Recherche dans les index par langue ou toutes langues

knowledge_compare(topic, langs?)
  → Juxtapose les articles Wikidata-linked dans N langues
  → Detecte les asymetries structurelles (sections presentes/absentes)
  → NE pretend PAS mesurer la "verite" — montre les perspectives

knowledge_coverage(topic)
  → Carte de couverture : quelles langues traitent ce sujet,
    profondeur relative (nombre de sections, taille)

knowledge_drift(topic, lang, period)
  → Diff entre dumps ZIM historiques (si stockes)
  → Montre les ajouts/suppressions de contenu dans le temps
```

### Capacite du noeud de bootstrap

Calcul pour le noeud initial (seul avant replication P2P) :

```
Upload mesure : 952 Mbps = 119 MB/s
Article moyen text-only : ~100 KB
Articles servis/seconde : ~1 190 req/s
Lecture humaine : ~1 req / 3 secondes

Utilisateurs simultanes (text-only) : ~2 500
Utilisateurs simultanes (avec images) : ~500
```

Overhead reseau (TCP, headers, TLS) ~30% inclus. Le bootstrap P2P
n'est pas un probleme — un seul noeud sert une petite ville.

### Stack technique

- `libzim` (Python, MIT) — lecture fichiers ZIM
- `tantivy` (Rust, MIT) ou `meilisearch` — indexation full-text
- `qwikidata` ou parsing JSONL direct — entity linking Wikidata
- `mcp` SDK v1.27+ — exposition MCP tools (cf. Sprint 26 Phase B)
- iroh-blobs — distribution P2P des articles caches

### Telechargement initial

~1.1 TB one-shot. Kiwix fournit les dumps en torrent.
- Fibre 500 Mbps : ~5h
- Fibre 2.3 Gbps (mesure) : ~1h30
- ADSL 50 Mbps : ~2 jours

---

## 2. Surveillance de foret — compute vision distribue

### Concept

App de detection de feux de foret et de deforestation a partir
d'images satellite (Sentinel-2, Landsat, FIRMS). L'inference vision
(segmentation, classification) est distribuee sur les GPU des noeuds
SBFB. Les resultats s'aggregent en CRDT.

### Ce que ca demontre

- **Compute GPU partage** : un modele de vision (YOLO, SAM, ou
  U-Net) tourne sur les GPU de N workers repartis dans le monde
- **Use case a impact reel** : la surveillance des forets est un
  probleme concret qui justifie le compute distribue
- **Temps reel accessible** : les images Sentinel-2 sont publiques
  (ESA Copernicus), le compute est la seule barriere — SBFB la leve

### Pipeline

```
Images satellite (Sentinel-2 / FIRMS)
    ↓
Noeud coordinator decoupe en tiles
    ↓
Tasks distribuees aux GPU workers (SBFB task pipeline)
    ↓
Workers executent inference vision (segmentation foret / feu)
    ↓
Resultats agreges en CRDT (carte de chaleur, alertes)
    ↓
App React dans iframe affiche la carte + alertes
```

### Modeles candidats

| Modele | Taille | GPU min | Tache |
|---|---|---|---|
| YOLOv8-seg | ~25 MB | RTX 3060 | Detection objets + segmentation |
| U-Net Sentinel | ~100 MB | RTX 3060 | Segmentation foret/non-foret |
| SAM (Segment Anything) | ~2.5 GB | RTX 3070+ | Segmentation zero-shot |
| FIRMS hotspot classifier | ~50 MB | RTX 3060 | Classification feu/non-feu |

### Donnees source (publiques, gratuites)

- **Sentinel-2** (ESA Copernicus) : images 10m resolution, revisit 5j
- **FIRMS** (NASA) : points chauds actifs, temps reel
- **Landsat** (USGS) : images 30m resolution, historique 50 ans
- **Global Forest Watch** : donnees de reference deforestation

---

## 3. Donjon & Dragon — LLM distribue interactif

### Concept

Un D&D ou le Maitre du Donjon est une IA distribuee sur les GPU de
tous les joueurs. Zero serveur. Zero abonnement. Le game state
persiste en CRDT entre les sessions. N'importe qui publie une
campagne comme un zip sur le reseau.

Design detaille : `docs/DND_P2P_DESIGN.md`

### Ce que ca demontre

- **Compute LLM distribue** : l'inference texte repartie sur N GPU
  produit un DM IA dont la qualite augmente avec le nombre de
  joueurs
- **Etat partage CRDT** : le game state (fiches persos, inventaire,
  carte, historique narratif) synchronise en temps reel sans serveur
- **Latence acceptable** : le tour par tour tolere 2-5 secondes de
  generation — naturel pour le jeu de role

### Pourquoi c'est le showcase parfait pour le GPU partage

1. **Demonstrable en live** — un spectateur voit le jeu tourner,
   comprend immediatement la valeur
2. **Viralite** — "D&D avec DM IA gratuit" est un post Hacker News
   front page
3. **GPU scaling visible** — ajouter un joueur (= un GPU) ameliore
   la qualite du DM en temps reel, l'audience le constate
4. **Zero barriere** — RTX 3060 suffit pour un modele 8B a 40 tok/s

---

## Synergie des trois apps

```
Alexandria          → prouve : "le reseau distribue des DONNEES"
Surveillance foret  → prouve : "le reseau distribue du COMPUTE VISION"
Donjon & Dragon     → prouve : "le reseau distribue du COMPUTE LLM"
```

Un visiteur qui voit les trois comprend en 30 secondes que SBFB
n'est pas un framework — c'est une plateforme ou n'importe quelle
app peut exploiter le stockage et le compute de tous les noeuds,
sans serveur central, sans abonnement, sans permission.

### Ordre de priorite d'implementation

1. **Alexandria** — le plus simple techniquement (pas de GPU requis
   pour le noyau, stockage + index + MCP), demontre la distribution
   de donnees, utile des jour 1 meme avec 1 seul noeud
2. **D&D** — design detaille deja ecrit (`DND_P2P_DESIGN.md`),
   le pipeline task SBFB est deja fonctionnel, forte viralite
3. **Foret** — requiert integration donnees satellite + modele
   vision, le plus complexe mais le plus impactant pour la
   credibilite ("P2P qui sauve des forets" > "P2P qui joue a D&D")
