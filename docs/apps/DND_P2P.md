# SBFB — Donjons & Dragons P2P avec DM IA distribue

**Date** : 2026-04-13
**Statut** : design document, pre-implementation

## Le concept

Un D&D ou le Maitre du Donjon est une IA distribuee sur les GPU de
tous les joueurs. Zero serveur. Zero abonnement. Zero censure.
Le game state persiste en CRDT entre les sessions. N'importe qui
publie une campagne comme un zip sur le reseau SBFB.

---

## Pourquoi D&D est le use case parfait pour SBFB

| Critere | D&D P2P |
|---------|---------|
| Type d'IA necessaire | **Text-in/text-out** — Ollama fait exactement ca |
| Latence acceptable | **Tour par tour** — 2-5 secondes sont naturelles |
| Conflits CRDT | **Minimaux** — chaque joueur ecrit sa fiche, le DM ecrit le monde |
| GPU requis | **RTX 3060** suffit pour un 8B a 40 tok/s |
| Public cible | **50M+ joueurs de D&D**, 13.7M comptes Roll20 |
| Support mobile | **Non necessaire** — on joue a D&D sur PC/laptop |
| Viralite | **Forte** — "D&D avec DM IA gratuit" est un post HN front page |

---

## Comparaison avec l'existant

| Plateforme | DM | Cout | Serveur | Donnees | Moddable | Multi-GPU |
|-----------|-----|------|---------|---------|----------|-----------|
| **Roll20** | Humain requis | $5-10/mois | Cloud | Chez Roll20 | API limitee | Non |
| **D&D Beyond** | Humain requis | $3-6/mois | Hasbro | Chez Hasbro | Non | Non |
| **AI Dungeon** | IA cloud | $10-30/mois | Latitude | Censure active | Non | Non |
| **KoboldAI** | IA locale | Gratuit | Aucun | Locale | Oui | **Non** (1 GPU) |
| **ChatGPT** | IA cloud | $20/mois | Microsoft | Chez OpenAI | Non | Non |
| **SBFB D&D** | **IA distribuee** | **Gratuit** | **Aucun** | **Locale** | **Total** | **Oui** |

Le differenciateur unique : la qualite du DM IA **augmente avec le
nombre de joueurs** (plus de GPU disponibles). Personne d'autre ne
fait ca.

---

## Hardware reel des joueurs — Steam Survey mars 2026

| GPU | VRAM | Part Steam | Llama 8B Q4 tok/s | Prix occasion |
|-----|------|-----------|-------------------|---------------|
| RTX 3060 | 12 GB | 4.47% (#1) | 38-42 tok/s | ~200 EUR |
| RTX 4060 | 8 GB | 4.17% (#3) | 37 tok/s | ~280 EUR |
| RTX 4060 Ti | 16 GB | ~2% | 48 tok/s | ~380 EUR |
| GTX 1650 | 4 GB | ~3.5% | 15 tok/s (3B) | ~100 EUR |
| RTX 4060 Laptop | 8 GB | 4.30% (#2) | ~35 tok/s | laptop |

**Le joueur moyen a une RTX 3060 (12GB) ou RTX 4060 (8GB), 16GB RAM,
CPU 6 cores.** C'est suffisant pour un Llama 8B.

Sources :
- Steam Hardware Survey mars 2026
- singhajit.com (RTX 3060 benchmarks)
- databasemart.com (RTX 4060 benchmarks)
- localscore.ai (Llama 3.1 8B benchmarks)

---

## Scenario realiste — partie de D&D a 5 joueurs

```
Alice  : RTX 3060 12GB  → 8B a 40 tok/s
Bob    : RTX 4060 8GB   → 8B a 37 tok/s
Charlie: GTX 1650 4GB   → 3B a 15 tok/s
Diana  : RTX 3060 12GB  → 8B a 40 tok/s
Eve    : RTX 4060 Ti 16GB → 8B a 48 tok/s

VRAM total : 52 GB
```

### Donnees reseau fibre EU 2026

Mesures reelles sur FTTH europeenne (sources : nPerf 2026, iroh
GitHub #3699, WifiTalents, GPON EU measurements) :

| Trajet | Latence RTT |
|--------|-------------|
| FTTH domestique (meme ville) | 1-7 ms |
| France → France (Paris-Lyon, ~465km) | 8-12 ms |
| France → Allemagne (Paris-Frankfurt) | 15-25 ms |
| iroh holepunch direct (additionnel) | 3-8 ms |
| iroh relay fallback (si holepunch echoue) | 70-190 ms |

91% des foyers francais en FTTH fin 2024. Le holepunch iroh reussit
dans la majorite des cas avec la fibre (NAT simple). Le relay est
le fallback, pas le cas normal.

### Mode A — 70B distribue streaming (par defaut, recommande)

**Le 70B est le mode par defaut pour la narration.** Sur fibre EU
2026, la latence reseau est negligeable (<25ms intra-France). Le
bottleneck est uniquement le compute GPU.

Un DM humain met 15-45 secondes entre l'action d'un joueur et la
fin de sa description. Un 70B distribue en streaming met 10-12
secondes — **plus rapide qu'un humain**, et le texte s'affiche mot
par mot en temps reel.

```
70B Q4 (~40GB VRAM) split sur 3 noeuds fibre EU :
  Eve    (4060 Ti, 16GB) : couches 1-25
  Alice  (RTX 3060, 12GB): couches 26-45
  Diana  (RTX 3060, 12GB): couches 46-65 + offload CPU

Latence reseau par token : ~31ms (transfert activation 2MB + 15ms RTT)
Compute par token : ~25ms
Total par token : ~56ms → ~18 tok/s

Pour 100 tokens : ~5.6 secondes
Pour 200 tokens (description riche) : ~11 secondes
```

**Avec le streaming, le joueur lit des la premiere seconde :**

```
Seconde 0   : "Je lance boule de feu sur les gobelins"
Seconde 0.5 : "Vous..."
Seconde 1.0 : "Vous levez les mains et une sphere..."
Seconde 3.0 : "...de flammes jaillit. Les gobelins hurlent..."
Seconde 6.0 : "...14 points de degats. Deux s'effondrent..."
Seconde 10  : "...Le chef, noirci mais vivant, leve sa hache.
               Que faites-vous ?"
```

L'experience n'est pas "attendre 10s puis lire" — c'est "regarder
l'histoire s'ecrire en direct", comme quand un DM humain parle.

**Qualite narrative 70B vs 8B :**

| Aspect | 8B | 70B |
|--------|-----|-----|
| Descriptions | Fonctionnel | Litteraire, immersif |
| Dialogues PNJ | Repetitif | Personnalites distinctes |
| Regles D&D | Approximatif | Precise, cite les mecaniques |
| Twists | Previsibles | Surprenants, coherents |
| Atmosphere | "Salle sombre" | "L'air s'epaissit d'une humidite grasse..." |

Pour un JDR, la qualite narrative est tout. Le 70B streaming en
10-12s bat le 8B en 2s.

### Mode B — 8B rapide (fallback / actions mecaniques)

Le 8B sert pour :
- **Actions mecaniques** : jets de des, calcul de degats, verifications
  de regles — pas besoin de narration
- **Fallback** si moins de 3 GPU connectes (pas assez de VRAM pour 70B)
- **Rafale d'actions** : 5 joueurs agissent en meme temps, chaque
  worker prend une tache, reponses mecaniques en parallele

```
Joueur tape "jet de sauvegarde"
  → N'importe quel noeud libre (8B local)
  → 30 tokens / 40 tok/s = ~0.8 seconde
  → "Jet de Constitution DD 15. Tu as +3, il te faut 12+.
     [De: 14] Reussi — tu resistes au poison."
```

### Mode C — Generation asynchrone (entre sessions)

Le reseau combine toute la puissance disponible pour generer du
contenu en batch :

- Nouveaux donjons et zones
- Backstory des PNJ
- Quests secondaires
- Lore du monde

Pas de contrainte de latence. Meme le relay fallback suffit. Les
joueurs retrouvent du contenu frais a chaque session.

---

## Latences mesurees — 5 joueurs fibre EU 2026

| Etape | Latence | Detail |
|-------|---------|--------|
| Submit task → iroh-doc write | ~1 ms | Ecriture locale |
| iroh-docs sync vers les workers | ~15-25 ms | Holepunch direct fibre |
| Worker claim (poll interval) | ~100 ms | Configurable |
| Claim propagation | ~20 ms | Holepunch direct |
| **Generation LLM 70B (100 tok)** | **~5600 ms** | 3 GPU distribues, 18 tok/s |
| Result propagation | ~20 ms | Holepunch direct |
| **TOTAL mode 70B** | **~5.8s** | Streaming visible des 0.5s |
| **TOTAL mode 8B** | **~2.3s** | Round-robin, GPU libre |

**Sur fibre EU 2026, le reseau ajoute ~180ms au total. Le reste
c'est du compute pur.** La fibre europeenne est aussi rapide que
du LAN pour ce use case.

### Si tous les joueurs agissent en meme temps

```
5 actions simultanees :
  → 3 GPU font le 70B distribue pour l'action 1 (narrative)
  → 2 GPU restants font du 8B pour les actions 2-3 (mecanique)
  → Actions 4-5 en queue, servies des que GPU 1 libere (~5.8s)
  → Temps total pour 5 actions : ~12 secondes

Avec un DM humain, 5 actions simultanees = 2-3 minutes de chaos.
Le DM IA est plus rapide et plus organise.
```

---

## Le mode DM Smart (recommande)

```
┌─────────────────────────────────────────────────┐
│  Narration (par defaut, 70% du jeu)             │
│  → 3 GPU distribues, 70B streaming, 18 tok/s    │
│  → Texte mot par mot des 0.5 seconde            │
│  → Descriptions, dialogues, revelations         │
├─────────────────────────────────────────────────┤
│  Mecanique (20% du jeu)                         │
│  → 1 noeud, 8B local, 40 tok/s                 │
│  → Reponse en <1 seconde                        │
│  → Jets de des, calcul degats, regles           │
├─────────────────────────────────────────────────┤
│  Generation de monde (entre sessions, 10%)      │
│  → Tous les noeuds, batch asynchrone            │
│  → Donjons, PNJ, quests, lore                   │
│  → Pas de contrainte temps reel                 │
└─────────────────────────────────────────────────┘
```

---

## Architecture technique

### Game state (iroh-docs CRDT)

```json
{
  "campaign": {
    "name": "La Crypte des Ames Perdues",
    "dm_mode": "ai",
    "dm_model": "llama3.1:70b-instruct-q4_K_M",
    "dm_fallback_model": "llama3.1:8b-instruct-q4_K_M",
    "world_seed": 42,
    "rules": "dnd5e"
  },
  "characters": {
    "<alice_node_id>": {
      "name": "Elara",
      "class": "Magicienne",
      "level": 5,
      "hp": 28,
      "hp_max": 32,
      "stats": {
        "for": 8, "dex": 14, "con": 12,
        "int": 18, "sag": 13, "cha": 10
      },
      "inventory": ["baton_arcanique", "potion_soin_x2", "grimoire"],
      "spells_prepared": ["boule_de_feu", "bouclier", "detection_magie"]
    }
  },
  "world": {
    "current_location": "caverne_niveau_2",
    "explored_rooms": ["entree", "salle_garde", "caverne_niveau_1"],
    "active_quests": [
      {"name": "Trouver l'amulette", "status": "en_cours"}
    ],
    "generated_zones": {}
  },
  "combat": null,
  "chat_history": []
}
```

**Pas de conflit CRDT** : chaque joueur ecrit SA fiche (un seul
writer par cle). Le DM IA ecrit le monde, le combat, et le chat.
Les des sont lances cote client et ecrits dans le doc (transparents
pour tous).

### Prompt systeme du DM IA

```
Tu es un Maitre du Donjon expert pour Donjons & Dragons 5e edition.

Regles :
- Narre a la deuxieme personne du pluriel ("Vous entrez...")
- Respecte les regles D&D 5e (jets de sauvegarde, classes d'armure,
  degats par sort)
- Genere des descriptions immersives mais concises (3-5 phrases max)
- Joue les PNJ avec des personnalites distinctes
- Demande des jets de des quand necessaire ("Jet de Perception DD 14")
- Ne tue jamais un personnage sans laisser une chance de sauvetage
- Adapte la difficulte au niveau du groupe

Etat actuel :
{game_state_json}

Derniere action du joueur :
{player_action}

Reponds avec :
1. La narration de ce qui se passe
2. Les consequences mecaniques (degats, jets, etc.)
3. La question "Que faites-vous ?" pour relancer
```

### Interface de jeu (app zip dans iframe)

```
┌──────────────────────────────────────────────────────┐
│  La Crypte des Ames Perdues       [DM: 70B streaming]│
│──────────────────────────────────────────────────────│
│                                                      │
│  ┌──────────────┐  ┌────────────────────────────────┐│
│  │ Carte        │  │ Chat narratif            ▊    ││
│  │              │  │                               ││
│  │  ░░███░░░    │  │ DM: L'air s'epaissit d'une   ││
│  │  ░░█@█░░░    │  │ humidite grasse qui colle a   ││
│  │  ░░███░░░    │  │ votre peau. Quelque part dans ││
│  │  ░░░█░░░░    │  │ l'obscurite, un goutte-a-     ││
│  │  ░░░█████    │  │ goutte regulier rythme le     ││
│  │              │  │ silence comme un pouls        ││
│  │ @ = groupe   │  │ malade...                     ││
│  │ █ = explore  │  │                               ││
│  │ ░ = inconnu  │  │ Elara: Je lance detection     ││
│  └──────────────┘  │ de magie.                     ││
│                    │                               ││
│  ┌──────────────┐  │ DM: Tes doigts tracent les    ││
│  │ Elara  Lv5   │  │ signes arcanes dans l'air et  ││
│  │ Magicienne   │  │ tes pupilles virent au violet.││
│  │ PV: 28/32    │  │ Une aura necromanque pulse    ││
│  │ CA: 13       │  │ derriere le mur est — ▊       ││
│  │              │  │                               ││
│  │ Sorts:       │  │ [De: 14] [De: 7]              ││
│  │ - Boule feu  │  └────────────────────────────────┘│
│  │ - Bouclier   │                                    │
│  │ - Detection  │  ┌────────────────────────────────┐│
│  │              │  │ > Action libre...              ││
│  │ Inventaire:  │  │                                ││
│  │ - Baton arq. │  │ [Attaquer] [Sort] [Parler]    ││
│  │ - Potions x2 │  │ [Explorer] [Repos]            ││
│  └──────────────┘  └────────────────────────────────┘│
│                                                      │
│  DM 70B ████████████░░░░ 18 t/s streaming            │
│  3 GPU distribues (Eve+Alice+Diana) ● fibre EU       │
│  Joueurs: 5/5 ● Latence reseau: 22ms                │
└──────────────────────────────────────────────────────┘
```

Le curseur ▊ avance en temps reel pendant la generation. Le joueur
lit le texte au fur et a mesure — comme un DM humain qui parle.

---

## Ce que les joueurs voient pour le GPU

```
┌──────────────────────────────────────────┐
│  Puissance du reseau                     │
│                                          │
│  Eve    (4060 Ti)    ██████████  48 t/s  │
│  Alice  (RTX 3060)   ████████░░  40 t/s  │  ← DM 70B
│  Diana  (RTX 3060)   ████████░░  40 t/s  │  ← distribue
│  Bob    (RTX 4060)   ███████░░░  37 t/s  │    sur ces 3
│  Charlie(GTX 1650)   ███░░░░░░░  15 t/s  │
│                                          │
│  VRAM combinee : 52 GB (40 GB pour 70B)  │
│  DM 70B distribue : 18 tok/s streaming   │
│  Bob + Charlie : standby (8B mecanique)  │
│  File d'attente : 0 requetes             │
│                                          │
│  [x] Contribuer mon GPU au DM            │
└──────────────────────────────────────────┘
```

---

## Features de gameplay

### Mode DM humain assiste par IA

Un joueur peut etre le DM. L'IA genere des suggestions que le DM
humain peut accepter, modifier, ou rejeter :

```
DM humain tape : "ils entrent dans la salle du boss"

IA suggere : "La salle s'ouvre sur une caverne immense.
Au centre, un golem de pierre de trois metres se dresse,
les yeux brillant d'une lueur rouge. Des chaines brisees
pendent de ses poignets. Le sol est jonche d'ossements.
Jet d'initiative ?"

DM humain : [Accepter] [Modifier] [Regenerer] [Ecrire moi-meme]
```

### Campagnes partageables

Une campagne entiere est un iroh-doc. N'importe qui peut :
- **Publier** sa campagne comme un zip (monde + regles custom + PNJ)
- **Forker** une campagne existante et la modifier
- **Rejoindre** une campagne en cours via le Browse SBFB

### Des transparents

Chaque lancer de de est ecrit dans le CRDT avec le timestamp et
le node_id. Tous les joueurs voient le meme resultat. Pas de
triche possible — le de est genere cote client mais verifie par
consensus (hash du seed + timestamp).

### Persistance entre sessions

Le game state est dans iroh-docs. Quand tous les joueurs se
deconnectent, le monde **survit**. A la prochaine session, tout
est la — positions, inventaires, quests, PNJ rencontres. Pas
de serveur a maintenir entre les sessions.

---

## Effort d'implementation

**Prerequis** : bridge postMessage (Sprint 13).

| Composant | LOC estimees |
|-----------|-------------|
| Game state schema + CRDT mapping | ~200 |
| DM prompt systeme + parsing reponse | ~150 |
| Interface HTML/CSS/JS (zip) | ~800 |
| Carte ASCII/SVG interactive | ~300 |
| Fiche de personnage editable | ~200 |
| Systeme de des + verification | ~100 |
| Chat narratif + historique | ~150 |
| Selection GPU round-robin | ~100 (dans le task pipeline existant) |
| Mode epique (13B 2-GPU) | ~200 (tensor split via task pipeline) |
| **Total** | **~2200 LOC** |

Faisable en **1 sprint** (Sprint 14) apres le bridge.

---

## Potentiel viral

**Le pitch** : "J'ai fait un D&D P2P ou le DM est une IA
distribuee sur les GPU de tes potes. Zero serveur, zero
abonnement, zero censure. Le monde persiste entre les sessions.
Plus vous etes nombreux, meilleur est le DM."

**Ou poster** :
- r/DnD (3.8M membres)
- r/LocalLLaMA (1.5M membres)
- r/rpg (2.3M membres)
- Hacker News
- itch.io (35M visiteurs/mois)

**Pourquoi ca prend** :
1. Le pain point #1 de D&D est "trouver un DM disponible" — l'IA
   le resout
2. La communaute LocalLLaMA est obsedee par les use cases concrets
   pour les LLM locaux — le D&D P2P est le use case parfait
3. Zero cout vs $20/mois ChatGPT ou $10/mois Roll20
4. Moddable a l'infini — n'importe qui fork la campagne ou l'app
5. Le cote "GPU de mes potes" est un hook social naturel

---

## Risques et mitigations

| Risque | Impact | Mitigation |
|--------|--------|-----------|
| 8B pas assez bon pour un DM | Narration mediocre | Mode DM hybride (humain assiste par IA). Les 8B recents (Llama 3.1, Qwen 2.5) sont significativement meilleurs qu'il y a 1 an. |
| Latence WiFi pour le mode 13B | 8-12 secondes | Le mode 13B est opt-in. Le round-robin 8B est le defaut. |
| Conflits CRDT si 2 joueurs agissent en meme temps | Game state incoherent | File d'attente de tour. Le DM IA traite les actions une par une. Le CRDT merge les fiches perso (pas de conflit car 1 writer par perso). |
| Triche (modifier sa fiche) | Joueur se donne 999 PV | Les modifications de fiche sont visibles par tous dans l'historique CRDT. Le DM IA peut valider les stats. Mode "fiche verrouillee" optionnel. |
| Personne n'a Ollama installe | Friction d'installation | Le launcher SBFB (Sprint 13) installe Ollama automatiquement si absent. |
