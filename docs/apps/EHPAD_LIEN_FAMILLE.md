# SBFB — Lien EHPAD-Famille : garder le contact humain sans cloud

**Date** : 2026-04-13
**Statut** : design, pre-implementation

---

## Le probleme

La solitude des residents en EHPAD est une crise de sante
publique. Les personnes de 50+ ans en situation de solitude vont
atteindre **2 millions en 2025-2026** (+49% en 10 ans). La
solitude augmente le risque de deces de **26 a 45%** (PMC 2025).

Pendant ce temps :
- Les familles vivent loin (mobilite professionnelle)
- Les appels a l'accueil EHPAD sont frustrants et rares
- Les tablettes "seniors" actuelles (GrandPad, Familizz)
  sont des **abonnements cloud** ($30-50/mois) ou les donnees
  du resident (photos, messages, habitudes) sont chez un tiers
- Post-scandale Orpea, les familles veulent de la **transparence**
  mais les EHPAD n'ont pas les outils
- 66% des EHPAD manquent de personnel — pas le temps d'organiser
  des visios sur le PC de l'accueil

---

## Le concept SBFB

Une app unique qui remplace tablette senior + WhatsApp famille +
logiciel EHPAD de communication, le tout :
- **Sans cloud** — les photos de mamie ne sont pas chez Google
- **Sans abonnement** — pas de $30/mois pour parler a sa mere
- **Ultra simple** — interface adaptee aux personnes agees
- **Avec IA locale** — le LLM aide, stimule, et connecte

L'app tourne sur une tablette dans la chambre du resident,
connectee au reseau SBFB de l'EHPAD. La famille installe SBFB
chez elle et voit le resident dans Browse.

---

## Les fonctionnalites — du lien humain augmente par l'IA

### 1. Visio simplifiee — un seul bouton

Le resident a une tablette avec UNE interface :

```
┌──────────────────────────────────────┐
│                                      │
│         Photo de Marie               │
│         (fille)                      │
│                                      │
│     ╔══════════════════════╗         │
│     ║   APPELER MARIE      ║         │
│     ╚══════════════════════╝         │
│                                      │
│         Photo de Pierre              │
│         (fils)                       │
│                                      │
│     ╔══════════════════════╗         │
│     ║   APPELER PIERRE     ║         │
│     ╚══════════════════════╝         │
│                                      │
│         Photo des petits-enfants     │
│                                      │
│     ╔══════════════════════╗         │
│     ║   VOIR LES MESSAGES  ║         │
│     ╚══════════════════════╝         │
│                                      │
└──────────────────────────────────────┘
```

Pas de mot de passe. Pas de menu. Pas de parametres. Juste
des photos et des gros boutons. Un resident avec debut de
demence peut utiliser ca.

**SBFB** : la visio passe par WebRTC dans l'iframe. Le signal
transite par iroh relay (pas de serveur Zoom/Google Meet).
**L'enregistrement de la visio n'existe nulle part** — pas de
cloud qui stocke la conversation.

---

### 2. Mur de photos familial — partage P2P

La famille envoie des photos depuis son telephone. Le resident
les voit defiler en diaporama sur sa tablette.

```
┌──────────────────────────────────────┐
│                                      │
│   ┌──────────────────────────────┐   │
│   │                              │   │
│   │   [Photo du petit-fils       │   │
│   │    au parc, hier]            │   │
│   │                              │   │
│   └──────────────────────────────┘   │
│                                      │
│   "Lucas au parc avec son velo !"    │
│   Envoyee par Marie, hier 16h30     │
│                                      │
│   ◄  ●●●○○  ►                       │
│                                      │
│   [REPONDRE avec un coeur ❤]        │
│   [APPELER MARIE]                    │
│                                      │
└──────────────────────────────────────┘
```

**SBFB** : les photos sont des blobs iroh. Elles se synchronisent
en P2P entre le telephone de Marie et la tablette de maman.
**Pas d'album Google Photos, pas d'iCloud.** Les photos de
famille restent dans la famille.

Le resident peut reagir avec un gros bouton coeur — Marie
recoit la notification via iroh-docs CRDT. Elle sait que maman
a vu la photo.

---

### 3. Messages vocaux — plus simple que le texte

Le resident ne peut pas taper sur un clavier. Il appuie sur
un bouton et parle. Le message vocal est envoye a la famille.

```
┌──────────────────────────────────────┐
│                                      │
│  Message de Maman (aujourd'hui 14h): │
│                                      │
│     ╔═══════════════════════╗        │
│     ║  🔊 Ecouter (0:23)    ║        │
│     ╚═══════════════════════╝        │
│                                      │
│  "Marie, j'ai vu la photo de Lucas, │
│   il a grandi ! Tu viens dimanche ?" │
│   (transcription IA)                 │
│                                      │
│  Reponse de Marie (15h02) :          │
│     ╔═══════════════════════╗        │
│     ║  🔊 Ecouter (0:15)    ║        │
│     ╚═══════════════════════╝        │
│                                      │
│  "Oui maman, on vient dimanche       │
│   avec les enfants !"                │
│   (transcription IA)                 │
│                                      │
│     ╔═══════════════════════╗        │
│     ║  ENREGISTRER UN MESSAGE║        │
│     ╚═══════════════════════╝        │
│                                      │
└──────────────────────────────────────┘
```

**SBFB** : le message vocal est un blob iroh. Whisper (Ollama)
transcrit localement pour que la famille puisse lire si elle
ne peut pas ecouter. **L'audio ne quitte jamais le reseau SBFB**
— pas de serveur vocal cloud.

---

### 4. Jeux inter-generationnels — jouer ensemble a distance

**Le differenciateur SBFB.** Aucune solution existante ne propose
des jeux partages en temps reel entre le resident et sa famille.

```
┌──────────────────────────────────────┐
│  Jeu de memoire — Maman vs Lucas    │
│──────────────────────────────────────│
│                                      │
│  ┌──┐ ┌──┐ ┌──┐ ┌──┐               │
│  │🌸│ │??│ │??│ │🌺│               │
│  └──┘ └──┘ └──┘ └──┘               │
│  ┌──┐ ┌──┐ ┌──┐ ┌──┐               │
│  │??│ │🌸│ │??│ │??│               │
│  └──┘ └──┘ └──┘ └──┘               │
│                                      │
│  Maman : 3 paires                    │
│  Lucas : 2 paires                    │
│                                      │
│  C'est ton tour, Maman !            │
│  (Lucas attend a Lyon)               │
│                                      │
│  [Touche une carte]                  │
└──────────────────────────────────────┘
```

Jeux disponibles :
- **Memory** — stimulation cognitive + fun familial
- **Loto/Bingo** — le petit-fils tire les numeros, mamie marque
- **Quiz photos** — "Qui est sur cette photo ?" (reminiscence)
- **Mots croises collaboratifs** — la famille aide a distance
- **Dessin partage** — le petit-fils dessine, mamie voit en direct

**SBFB** : l'etat du jeu est dans iroh-docs CRDT. Le resident
joue sur sa tablette, le petit-fils sur son telephone. Le tour
se synchronise en temps reel via fibre EU (~25ms). C'est du
**gaming P2P pour personnes agees** — le meme stack que le D&D
mais avec des cartes memory au lieu de gobelins.

---

### 5. Compagnon IA — contre la solitude entre les visites

**Le probleme** : les visites sont rares. Entre deux visites, le
resident est seul. Les robots compagnons (ElliQ, ~$250 + $30/mois)
sont chers et cloud-dependants.

**L'app** : un compagnon conversationnel simple qui :
- Rappelle des souvenirs ("Tu te souviens quand Marie a eu son
  diplome ? C'etait en quelle annee ?")
- Pose des questions de stimulation cognitive
- Lit les actualites a voix haute (text-to-speech local)
- Raconte des histoires / blagues
- Rappelle les prochains evenements ("Marie vient dimanche !")

```
┌──────────────────────────────────────┐
│  Bonjour Maman ! Comment ca va      │
│  aujourd'hui ?                       │
│                                      │
│  ╔══════════════════════╗            │
│  ║  "Ca va bien merci"   ║  ← vocal │
│  ╚══════════════════════╝            │
│                                      │
│  C'est super ! Tu sais quoi ? Marie │
│  a envoye une nouvelle photo de      │
│  Lucas hier. Tu veux la voir ?       │
│                                      │
│  Et dimanche, toute la famille vient │
│  te voir ! Lucas a hate de te montrer│
│  son nouveau velo.                   │
│                                      │
│  Tu veux qu'on fasse un petit jeu    │
│  de memoire en attendant ?           │
│                                      │
│  [VOIR LA PHOTO]                     │
│  [JOUER]                             │
│  [APPELER MARIE]                     │
│  [RACONTER UNE HISTOIRE]            │
└──────────────────────────────────────┘
```

**SBFB** : le LLM tourne sur le GPU de l'EHPAD (ou celui de
Marie a Lyon — la puissance partagee du reseau). Le compagnon
connait le contexte familial (photos recues, visites planifiees,
evenements) parce qu'il a acces au meme iroh-doc CRDT.

**Important** : le compagnon ne remplace pas la famille. Il fait
le **pont entre les visites**. Il rappelle que Marie a envoye une
photo, que Pierre appelle ce soir, que les petits-enfants viennent
dimanche. Il maintient le lien dans la tete du resident.

---

### 6. Livre de vie numerique — reminiscence assistee par IA

Pour les residents avec demence debutante, le LLM aide a
construire et parcourir un livre de vie :

```
┌──────────────────────────────────────┐
│  Mon Livre de Vie                    │
│──────────────────────────────────────│
│                                      │
│  ┌──────────────────────────────┐    │
│  │  [Photo de mariage, 1978]    │    │
│  └──────────────────────────────┘    │
│                                      │
│  "C'etait le 15 juin 1978. Tu as    │
│   epouse Jean a l'eglise de         │
│   Montmartre. Marie raconte que tu  │
│   portais une robe en dentelle que  │
│   ta mere avait faite."             │
│                                      │
│  Enregistre par : Marie (2026-03-15)│
│                                      │
│  [Photo suivante ►]                  │
│  [Ecouter l'histoire 🔊]            │
│  [Ajouter un souvenir]              │
│                                      │
└──────────────────────────────────────┘
```

La famille contribue des photos et des textes. Le LLM genere
des narrations a partir de ces fragments. Le resident parcourt
son histoire. C'est de la **reminiscence assistee par IA** —
une technique documentee pour ralentir le declin cognitif.

**SBFB** : le livre de vie est un iroh-doc CRDT. La famille
ajoute des souvenirs depuis n'importe ou. Le LLM local genere
les narrations. Les photos sont des blobs iroh. Tout reste
dans le reseau familial.

---

### 7. Agenda visuel — structure et reperes

Pour les residents desorientes, un agenda simple et visuel :

```
┌──────────────────────────────────────┐
│  Aujourd'hui — Mercredi 13 Avril    │
│──────────────────────────────────────│
│                                      │
│  ☀ MATIN                            │
│  ✓ 08:00 Petit dejeuner             │
│  ✓ 09:30 Gym douce (salle commune)  │
│  → 10:30 Atelier peinture           │
│                                      │
│  🌤 APRES-MIDI                       │
│    14:00 Repos                       │
│    15:30 Gouter                      │
│    16:00 Jeu de memory avec Lucas !  │
│                                      │
│  🌙 SOIR                            │
│    18:30 Diner                       │
│    19:30 Appel visio Marie ♥         │
│    20:30 Coucher                     │
│                                      │
│  DIMANCHE : Visite de toute la      │
│  famille ! (dans 4 jours)            │
│                                      │
└──────────────────────────────────────┘
```

L'equipe soignante et la famille alimentent le meme agenda
via CRDT. Le resident a une vue claire de sa journee.

---

### 8. Mur d'expression EHPAD — lien social entre residents

Les residents peuvent poster des messages / photos / dessins
sur un mur commun de l'EHPAD :

```
┌──────────────────────────────────────┐
│  Mur de l'EHPAD Les Tilleuls        │
│──────────────────────────────────────│
│                                      │
│  Mme Martin (Ch.101) :              │
│  "J'ai peint un coucher de soleil   │
│   ce matin !" [photo]               │
│   ❤ 7 coeurs                        │
│                                      │
│  M. Dupont (Ch.108) :               │
│  "Qui veut jouer aux cartes cet     │
│   apres-midi ?"                     │
│   ✋ 3 partants (Mme Petit, M.Leroy,│
│      Mme Garcia)                     │
│                                      │
│  Activites du jour :                │
│  🎵 Musique a 15h (5 inscrits)      │
│  🎨 Peinture a 10h30 (3 inscrits)   │
│                                      │
│  [Poster un message]                │
│  [S'inscrire a une activite]        │
└──────────────────────────────────────┘
```

---

## Pourquoi SBFB bat toutes les solutions existantes

| Feature | Familizz | GrandPad | WhatsApp | SBFB |
|---------|----------|----------|----------|------|
| Cout/mois | ~€20 | ~$50 | Gratuit | **Gratuit** |
| Donnees chez | Familizz (cloud) | GrandPad (cloud) | Meta | **Personne (P2P)** |
| Visio | Basique | Oui | Oui | **Oui (WebRTC P2P)** |
| Photos partagees | Oui | Oui | Oui | **Oui (blobs iroh)** |
| Messages vocaux | Non | Oui | Oui | **Oui + transcription IA locale** |
| Jeux famille a distance | **Non** | **Non** | **Non** | **Oui (CRDT temps reel)** |
| Compagnon IA | **Non** | **Non** | **Non** | **Oui (LLM local)** |
| Livre de vie IA | **Non** | **Non** | **Non** | **Oui (reminiscence IA)** |
| Lien EHPAD-famille | Oui (leur produit) | Non | Non | **Oui (meme CRDT)** |
| Mur social residents | Non | Non | Non | **Oui** |
| Fonctionne offline | Non | Non | Non | **Oui (CRDT sync)** |
| RGPD par construction | Non (cloud) | Non (cloud US) | Non (Meta) | **Oui (zero cloud)** |

**Les 3 features que personne n'a** :
1. Jeux inter-generationnels P2P en temps reel
2. Compagnon IA local avec contexte familial
3. Livre de vie / reminiscence assistee par IA

---

## Impact mesurable

| Metrique | Baseline | Avec l'app (estime) | Source |
|----------|----------|-------------------|--------|
| Sentiment de solitude | 60%+ des residents | -30 a -50% (meta-analyse techno visio) | JMIR Aging 2022 |
| Frequence contact famille | 1-2x/semaine | 1-2x/jour (messages + photos + jeux) | — |
| Declin cognitif | Normal pour l'age | Ralenti (reminiscence + jeux cognitifs) | PMC 2025 |
| Burnout aidants familiaux | 78% | Reduit (coordination + transparence) | Stacker 2026 |
| Cout pour la famille | €20-50/mois | **€0** | — |
| Confiance famille envers EHPAD | Basse (post-Orpea) | Augmentee (transparence) | — |

---

## Architecture technique

```
EHPAD Les Tilleuls
├── Routeur WiFi interne
├── Raspberry Pi (noeud SBFB, coordinateur EHPAD)
│   └── GPU optionnel (ou utilise le GPU de la famille)
├── Tablette chambre 101 (Mme Martin)
│   └── App SBFB en browser (interface simplifiee)
├── Tablette chambre 108 (M. Dupont)
│   └── App SBFB en browser
└── PC bureau equipe soignante
    └── Dashboard equipe (planning, transmissions)

        ↕ iroh P2P (fibre EU, ~15-25ms) ↕

Famille Marie (Lyon)
├── Smartphone → app SBFB (photos, messages, jeux)
└── PC avec GPU → fait tourner le compagnon IA

Famille Pierre (Toronto)
└── Laptop → app SBFB (visio, messages)
```

**Le GPU de la famille fait tourner le LLM pour le resident.**
Marie a un PC gaming a Lyon avec une RTX 3060. Son GPU fait
tourner le compagnon IA pour maman a l'EHPAD. C'est le meme
principe que le D&D P2P — la puissance GPU partagee.

---

## Effort d'implementation

| Composant | LOC | Temps |
|-----------|-----|-------|
| Interface senior simplifiee (gros boutons, photos) | ~500 | 3h |
| Visio WebRTC dans iframe | ~300 | 2h |
| Mur photos (blobs iroh + galerie) | ~250 | 2h |
| Messages vocaux (record + blob + Whisper transcription) | ~300 | 2h |
| Jeux memory/loto (CRDT game state) | ~400 | 3h |
| Compagnon IA (prompt + contexte familial) | ~300 | 2h |
| Livre de vie (photos + narration IA) | ~250 | 2h |
| Agenda visuel (CRDT partage equipe/famille) | ~150 | 1h |
| Mur social residents | ~200 | 1.5h |
| **Total** | **~2650 LOC** | **~18.5h (~3 sessions)** |

---

## Le pitch

"On a fait une app gratuite pour que les familles restent
connectees avec leurs parents en EHPAD. Visio en un clic,
photos partagees, jeux a distance avec les petits-enfants,
et un compagnon IA qui fait le pont entre les visites. Pas
d'abonnement, pas de cloud — les photos de mamie restent
dans la famille. Open source."

**Ou poster** :
- Associations d'aidants (France Alzheimer, Petits Freres des
  Pauvres, France Assos Sante)
- r/eldercare, r/dementia, r/caregiversupport
- Forums EHPAD / directeurs d'etablissements
- Presse locale (angle humain fort)

Sources:
- [2M personnes 50+ en solitude 2025-2026 (+49%)](https://pmc.ncbi.nlm.nih.gov/articles/PMC9940659/)
- [Solitude augmente mortalite 26-45% (PMC)](https://pubmed.ncbi.nlm.nih.gov/40653342/)
- [Video-conferences reduisent solitude en EHPAD (JMIR Aging)](https://aging.jmir.org/2022/4/e40125)
- [78% aidants en burnout (Stacker 2026)](https://keyt.com/stacker-parenting-family/2026/02/19/2026-caregiver-burnout-statistics-how-stress-shows-up-in-family-caregiving/)
- [66% EHPAD manquent de personnel (DREES)](https://drees.solidarites-sante.gouv.fr/publications/etudes-et-resultats/le-personnel-et-les-difficultes-de-recrutement-dans-les-ehpad)
- [Familizz — communication EHPAD famille](https://www.familizz.com/en/communication-ehpad-famille-application/)
- [IA compagnon vs solitude (PMC 2025)](https://pmc.ncbi.nlm.nih.gov/articles/PMC11898439/)
