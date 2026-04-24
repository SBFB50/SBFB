# SBFB — Plateforme de crise : catastrophes humanitaires et environnementales

**Date** : 2026-04-13
**Statut** : design, pre-implementation

---

## Le probleme — les chiffres

| Fait | Chiffre | Source |
|------|---------|--------|
| Personnes deplacees dans le monde | **139.3 millions** (2025) | UNHCR |
| Erreurs medicales evitables apres catastrophe liees a la communication | **80%** des evenements graves | Joint Commission |
| Pertes economiques des interruptions de service | **7.4x** les dommages directs | GIR Report 2025 |
| Tours cellulaires detruites par un ouragan typique | **Des centaines** — batteries de backup durent **12h max** | TowerPoint / FCC |
| Hopitaux prepares a la reunification familiale | **65.9%** seulement | HHS ASPR TRACIE |
| Refugies sans papiers d'identite | **Des millions** — systemes biometriques centralises dependent d'internet | UNHCR |
| Panne de Berlin 2026 (sabotage) | **100K personnes**, plusieurs jours, plus longue panne depuis 1945 | Sources presse |

**Le constat** : quand l'infrastructure tombe (electricite, tours
cellulaires, internet, serveurs), tous les outils modernes de
coordination tombent avec. WhatsApp, Google, les systemes de
l'ONU — tout depend du cloud. Les gens se retrouvent isoles avec
leur telephone en mode avion.

---

## Pourquoi SBFB est fait pour ca

Le stack SBFB a ete concu pour fonctionner **sans infrastructure
centrale** :

| Capacite | En crise |
|----------|---------|
| P2P via iroh (holepunch + relay + mDNS LAN) | Fonctionne en WiFi direct, Bluetooth, mDNS local — pas besoin de tour cellulaire |
| CRDT offline-first | Les donnees se synchronisent quand les noeuds se retrouvent — pas besoin de connexion permanente |
| Apps en zip distribue | L'app de crise se propage de telephone en telephone par mesh — pas besoin de store |
| LLM local (Ollama) | Traduction, triage, aide a la decision — sans internet |
| Pas de serveur central | Si un noeud tombe, le reseau continue |

**SBFB est le seul stack ou l'app, les donnees, et l'IA
continuent de fonctionner quand tout le reste est mort.**

---

## 10 apps de crise — chaque phase de la catastrophe

### Phase 1 — AVANT (preparation + alerte precoce)

#### App 1 — Reseau de capteurs d'alerte precoce

Capteurs distribues (inondation, qualite de l'air, sismique,
incendie) sur le modele PurpleAir/Sensor.Community mais P2P :

```
┌──────────────────────────────────────────────────┐
│  Alerte reseau citoyen — Quartier Saint-Martin   │
│──────────────────────────────────────────────────│
│                                                  │
│  🔴 ALERTE INONDATION                           │
│  Capteur Pont-Neuf : niveau eau +180cm en 2h    │
│  Capteur Rue Basse : niveau eau +120cm en 2h    │
│  Tendance : ████████████████ montee rapide       │
│                                                  │
│  IA locale : "Depassement cote d'alerte prevu   │
│  dans ~3h. Pattern similaire a la crue du        │
│  15/03/2024. Recommandation : evacuation zone    │
│  basse dans les 2h."                             │
│                                                  │
│  [Voir carte zones a risque]                     │
│  [Alerter mes voisins]                           │
│  [Plan d'evacuation]                             │
└──────────────────────────────────────────────────┘
```

**Pourquoi SBFB** : les capteurs ecrivent dans iroh-docs CRDT.
Les alertes se propagent en P2P meme si le reseau 4G est sature.
Le LLM local analyse les tendances sans envoyer les donnees a
un serveur. Les voisins abonnes recoivent l'alerte directement
sur leur telephone.

#### App 2 — Kit de preparation personnel

Chaque foyer a une app avec son plan d'evacuation, ses contacts
d'urgence, son inventaire de kit. Synchronise en CRDT entre les
membres de la famille.

```
┌──────────────────────────────────────────────────┐
│  Kit d'urgence famille Dupont                    │
│──────────────────────────────────────────────────│
│                                                  │
│  Inventaire :                                    │
│  ✓ Eau 6L (verifie 01/04)                       │
│  ✓ Lampe torche + piles                          │
│  ⚠ Medicaments Pierre — EXPIRE 15/03            │
│  ✓ Radio a manivelle                             │
│  ✓ Copies papiers (identite, assurance)          │
│  ✗ Nourriture 72h — MANQUANT                    │
│                                                  │
│  Point de ralliement : Ecole Jean Moulin         │
│  Contact urgence : Marie 06.XX.XX.XX.XX         │
│                                                  │
│  [Mettre a jour] [Partager avec la famille]      │
└──────────────────────────────────────────────────┘
```

---

### Phase 2 — PENDANT (les premieres 72h)

#### App 3 — Communication mesh quand tout est coupe

**Le probleme central** : les tours cellulaires sont detruites.
Pas de 4G, pas de WiFi. Les gens sont isoles.

**L'app** : SBFB + Meshtastic LoRa comme couche de transport.
Les telephones communiquent via des noeuds LoRa ($20 chacun,
10+ jours d'autonomie, recharge solaire) qui forment un mesh.

```
┌──────────────────────────────────────────────────┐
│  SBFB Mesh — Mode crise                         │
│  Reseau : 47 noeuds connectes                    │
│  Couverture : ~3km rayon                         │
│──────────────────────────────────────────────────│
│                                                  │
│  Messages recents :                              │
│                                                  │
│  14:32 Mairie : "Centre d'hebergement ouvert     │
│  gymnase Pasteur. Capacite 200 personnes."       │
│                                                  │
│  14:28 Pompiers : "Rue du Port impraticable.     │
│  Detour par avenue de la Gare."                  │
│                                                  │
│  14:15 Citoyen : "Famille 4 pers bloquee        │
│  2e etage 12 rue des Lilas. Besoin evacuation."  │
│                                                  │
│  13:50 Hopital : "Urgences saturees. Blessures   │
│  legeres → pharmacie Place du Marche."           │
│                                                  │
│  [Envoyer message] [SOS + position GPS]          │
│  [Je suis en securite ✓]                         │
│  [Carte des points d'eau/nourriture]             │
└──────────────────────────────────────────────────┘
```

**Pourquoi SBFB** : les messages se propagent de noeud en noeud
via LoRa. iroh-docs CRDT merge les messages quand les noeuds
se retrouvent. **Pas besoin d'internet pour que 47 personnes
dans un rayon de 3km communiquent.** Un seul noeud qui retrouve
de la connectivite propage tout au monde exterieur.

Meshtastic gere le transport LoRa. SBFB gere les apps, le
CRDT, et l'IA locale. C'est complementaire, pas concurrent.

#### App 4 — "Je suis vivant" — reunification familiale

**Le probleme** : 80% des appels post-catastrophe sont "est-ce
que ma famille est vivante ?". Les systemes (Google Person
Finder, Croix-Rouge Restoring Family Links) dependent d'internet.

**L'app** : chaque personne peut marquer son statut "en securite"
avec sa position GPS. Le statut se propage en P2P mesh. Les
familles se retrouvent sans internet.

```
┌──────────────────────────────────────────────────┐
│  Famille Dupont — Statut                        │
│──────────────────────────────────────────────────│
│                                                  │
│  Marie (moi)   ✅ En securite — Gymnase Pasteur  │
│                   Mis a jour il y a 12 min       │
│                                                  │
│  Pierre        ✅ En securite — Bureau, centre    │
│                   Mis a jour il y a 45 min       │
│                                                  │
│  Lucas (fils)  ❓ Statut inconnu                  │
│                   Dernier signal : ecole Moulin  │
│                   Il y a 2h                      │
│                                                  │
│  Maman (EHPAD) ✅ En securite — EHPAD Les Tilleuls│
│                   Confirme par personnel 13h30   │
│                                                  │
│  [Mettre a jour mon statut]                      │
│  [Chercher : Lucas]                              │
│  [Envoyer signal SOS]                            │
└──────────────────────────────────────────────────┘
```

**Pourquoi SBFB** : le statut est un iroh-doc CRDT avec la cle
publique de chaque membre de la famille. Meme sans internet,
si le telephone de Marie et celui de Pierre se retrouvent dans
le meme mesh LoRa, les statuts se synchronisent. Le systeme
fonctionne aussi longtemps que les telephones ont de la batterie.

#### App 5 — Triage medical assiste par IA

**Le probleme** : les premiers secours sont submerges. Pas assez
de medecins. Les volontaires non formes ne savent pas prioriser.

**L'app** : un LLM local aide au triage (classification START) :

```
┌──────────────────────────────────────────────────┐
│  Triage assistant — Patient #47                  │
│──────────────────────────────────────────────────│
│                                                  │
│  Questions rapides :                             │
│                                                  │
│  Le patient marche ?           [OUI] [NON]       │
│  Le patient respire ?          [OUI] [NON]       │
│  Frequence respiratoire ?      [<30] [>30]       │
│  Pouls radial palpable ?       [OUI] [NON]       │
│  Repond aux ordres simples ?   [OUI] [NON]       │
│                                                  │
│  → Classification : 🟡 URGENCE DIFFEREE (Jaune)  │
│                                                  │
│  "Fracture ouverte avant-bras. Hemorragie        │
│  controlee par compression. Conscient. Stable.   │
│  Prioriser les rouges avant ce patient."         │
│                                                  │
│  [Enregistrer] [Patient suivant] [Appeler SAMU]  │
│                                                  │
│  Stats zone : 12 Vert, 8 Jaune, 3 Rouge, 1 Noir │
└──────────────────────────────────────────────────┘
```

Le LLM ne diagnostique pas — il guide un volontaire non forme
a travers le protocole START standard. Les donnees de triage se
propagent en CRDT au poste medical avance qui voit arriver les
patients priorises.

#### App 6 — Carte des ressources collaborative

```
┌──────────────────────────────────────────────────┐
│  Carte de crise — Zone inondation centre-ville   │
│──────────────────────────────────────────────────│
│                                                  │
│  ┌─────────────────────────────────────┐         │
│  │  🏥 Hopital (sature)                │         │
│  │  💧 Point d'eau potable (Mairie)    │         │
│  │  🍞 Distribution nourriture (Gym)   │         │
│  │  ⛺ Hebergement (Gymnase Pasteur)   │         │
│  │  ⚠ Route coupee (Rue du Port)      │         │
│  │  🔋 Point de recharge (Caserne)     │         │
│  │  📡 Noeud mesh actif (47 connectes) │         │
│  │  🔴 Zone dangereuse (niveau eau)    │         │
│  └─────────────────────────────────────┘         │
│                                                  │
│  Derniere MAJ : il y a 8 min (via mesh local)    │
│  Sources : 23 citoyens + 4 autorites             │
│                                                  │
│  [Signaler une ressource]                        │
│  [Signaler un danger]                            │
│  [Demander de l'aide]                            │
└──────────────────────────────────────────────────┘
```

Chaque citoyen peut ajouter un point sur la carte. Les donnees
mergent en CRDT. La carte est a jour meme sans internet central.

---

### Phase 3 — APRES (reconstruction + coordination)

#### App 7 — Recensement des degats distribue

Les habitants photographient les degats chez eux. Les photos
sont stockees en blobs iroh avec GPS et timestamp. Le LLM local
aide a categoriser (structure, toiture, inondation, electrique).

```
Rapport degats — 12 rue des Lilas
  Photos : 4 (stockees localement, GPS tague)
  Classification IA :
    - Inondation RDC (1.2m estimee)
    - Structure : murs porteurs intacts
    - Toiture : tuiles deplacees secteur nord
    - Electrique : tableau coupe (securite OK)
  
  Priorite IA : Moyenne (habitable apres pompage)
  
  Soumis par : Pierre Dupont, 14/04/2026 09:30
  Verifie par : 0 (en attente verification voisinage)
```

Quand la connectivite revient, toutes les donnees se
synchronisent vers les assurances et les autorites — avec
photos, GPS, timestamps, et classification. Des semaines
de bureaucratie compressees en heures.

#### App 8 — Coordination des benevoles

```
┌──────────────────────────────────────────────────┐
│  Benevoles actifs : 127                          │
│──────────────────────────────────────────────────│
│                                                  │
│  Besoins en cours :                              │
│  🔴 5 personnes pour pompage Rue Basse (urgent)  │
│     → 2 inscrits, besoin 3 de plus              │
│  🟡 Transport materiel vers gymnase              │
│     → Vehicule necessaire, 1 offre              │
│  🟢 Distribution repas 18h                       │
│     → 12 inscrits, suffisant                     │
│                                                  │
│  IA matching :                                   │
│  "Jean (a 800m, vehicule) correspond au besoin   │
│  transport materiel. Notification envoyee."       │
│                                                  │
│  [Je suis disponible] [J'ai un vehicule]         │
│  [J'ai de la nourriture] [J'ai de l'eau]         │
└──────────────────────────────────────────────────┘
```

#### App 9 — Journal de crise pour les autorites

Historique complet de la crise avec timeline, decisions prises,
ressources deployees. Tout est dans le CRDT — immutable,
timestamp, signe par chaque contributeur. Utile pour le retour
d'experience (RETEX) et les rapports officiels.

#### App 10 — Suivi psychologique post-crise

Comme le journal anonyme de l'app hopital burnout — un espace
ou les sinistres et les intervenants peuvent exprimer leur
vecu. Le LLM detecte les signaux de stress post-traumatique
(pas de diagnostic — juste un signal d'alerte pour les
professionnels).

---

## L'integration Meshtastic + SBFB

```
Couche transport (quand internet est mort) :
  Meshtastic LoRa nodes ($20 chacun)
    → Portee : 1-10km selon terrain
    → Autonomie : 10+ jours
    → Recharge : panneau solaire 5W
    → Mesh automatique : chaque noeud relaye
  
Couche application (SBFB) :
  iroh-docs CRDT → les messages/donnees mergent
  Apps en zip → se propagent de noeud en noeud
  LLM local → triage, traduction, matching

Passerelle vers le monde :
  Un seul noeud avec connectivite satellite ou 4G
  → Synchronise tout le mesh vers le reseau global
  → Les familles a l'exterieur voient les statuts
```

**Le noeud LoRa a $20 + un vieux telephone = un kit de survie
numerique complet.** L'app de crise est deja dans le zip — pas
besoin de telecharger quoi que ce soit pendant la catastrophe.

---

## Pourquoi c'est different de tout ce qui existe

| Solution | Fonctionne sans internet | Apps distribuees | IA locale | CRDT offline | P2P mesh |
|----------|------------------------|-----------------|-----------|-------------|----------|
| WhatsApp | Non | Non | Non | Non | Non |
| Google Person Finder | Non | Non | Non | Non | Non |
| Croix-Rouge RFL | Non | Non | Non | Non | Non |
| Zello (talkie-walkie) | Partiellement (WiFi) | Non | Non | Non | Non |
| Meshtastic seul | **Oui** | Non (texte brut) | Non | Non | **Oui** |
| **SBFB + Meshtastic** | **Oui** | **Oui** | **Oui** | **Oui** | **Oui** |

**Meshtastic transporte des messages texte courts.** SBFB
transporte des **apps completes avec IA, donnees structurees,
et synchronisation offline.** Ensemble, c'est le seul systeme
de crise qui fonctionne quand absolument tout est mort.

---

## Marche et financement potentiel

| Source | Montant | Pertinence |
|--------|---------|-----------|
| FEMA Building Resilient Infrastructure (US) | Grants disponibles pour IoT inondation | Capteurs d'alerte precoce |
| EU Horizon Europe — Disaster Resilience | Multi-millions EUR | Recherche + deploiement |
| Fonds urgence EHPAD France | 300M EUR 2025 | Lien EHPAD + crise |
| UN OCHA / UNHCR technology grants | Variable | Camps refugies |
| Fondation Croix-Rouge | Variable | Innovation humanitaire |
| NLNet (a deja finance l'audit Tauri) | 50K-200K EUR | Open source resilience |

---

## Effort d'implementation

| Composant | LOC | Temps |
|-----------|-----|-------|
| App 1 — Capteurs alerte precoce | ~600 | 3h |
| App 2 — Kit preparation | ~300 | 2h |
| App 3 — Communication mesh (integration Meshtastic) | ~800 | 5h |
| App 4 — "Je suis vivant" reunification | ~500 | 3h |
| App 5 — Triage medical IA | ~400 | 3h |
| App 6 — Carte ressources collaborative | ~600 | 4h |
| App 7 — Recensement degats | ~400 | 3h |
| App 8 — Coordination benevoles | ~500 | 3h |
| App 9 — Journal de crise | ~300 | 2h |
| App 10 — Suivi psy post-crise | ~250 | 2h |
| **Total** | **~4650 LOC** | **~30h (~5 sessions)** |

---

## Le pitch

"Quand l'ouragan coupe l'electricite et les tours cellulaires,
WhatsApp meurt. Google meurt. Tout meurt. Sauf le mesh.

SBFB + Meshtastic = un reseau de crise a $20 par noeud qui
fait circuler des apps completes, synchronise les donnees
sans internet, et fait tourner une IA locale pour le triage
et la coordination.

Un seul noeud qui retrouve la connectivite synchronise tout
le quartier avec le reste du monde.

Open source. Gratuit. Pas d'abonnement. Pas de cloud.
Fonctionne quand tout le reste est mort."

**Ou poster** :
- r/preppers (2.5M membres)
- r/EmergencyManagement
- Meshtastic community (tres actif, 50K+ utilisateurs)
- Conferences : CCC (Chaos Communication Congress), FOSDEM
- ONG : Croix-Rouge Innovation, MSF, OCHA Innovation
- EU calls Horizon Europe Disaster Resilience

---

## Systeme de cagnotte + operations de deploiement

### Le probleme

Les gens en zone sinistree (Ukraine, Syrie, zones inondees) n'ont
pas $100 pour 5 noeuds LoRa. Et les apps specifiques a leur crise
n'existent pas au moment ou la catastrophe frappe.

Il faut deux choses en parallele :
1. **Financer et deployer le hardware** (noeuds LoRa)
2. **Creer les apps de crise en temps reel** pendant que la
   catastrophe se deroule

### Modele : Operation de deploiement SBFB

```
┌──────────────────────────────────────────────────┐
│  SBFB Operations — Ukraine Kherson Inondation    │
│──────────────────────────────────────────────────│
│                                                  │
│  CAGNOTTE                                        │
│  Objectif : 50 noeuds LoRa + 20 tablettes       │
│  ████████████████░░░░ 78% ($3,900 / $5,000)     │
│                                                  │
│  207 donateurs depuis 12 pays                    │
│  Derniers dons : Alice (FR) $20, Bob (DE) $50   │
│                                                  │
│  DEPLOIEMENT                                     │
│  Statut : En cours — 32/50 noeuds deployes      │
│  Equipe terrain : 4 volontaires + 1 ONG locale   │
│  Couverture : 12km2 (quartiers Korabel, Dnipro)  │
│                                                  │
│  APPS CREEES POUR CETTE CRISE                    │
│  ✓ "Je suis vivant" (generee J+0, 500 users)    │
│  ✓ Carte des zones inondees (generee J+1)        │
│  ✓ Distribution eau potable (generee J+2)        │
│  → Triage medical (en cours de generation)       │
│                                                  │
│  [Donner] [Devenir volontaire terrain]           │
│  [Proposer du GPU pour generer des apps]         │
│  [Suivre l'operation en direct]                  │
└──────────────────────────────────────────────────┘
```

### Le flow complet

```
JOUR 0 : La catastrophe frappe (inondation, seisme, conflit)
  │
  ▼
HEURE 1 : Un utilisateur SBFB (n'importe ou dans le monde)
  cree une "Operation" sur le reseau
  → Nom : "Inondation Kherson 2026"
  → Zone : coordonnees GPS
  → Besoins estimes : 50 noeuds LoRa, 20 tablettes
  → Budget : $5000
  │
  ▼
HEURE 1-6 : La cagnotte se remplit
  → Les noeuds SBFB du monde entier voient l'operation
    dans Browse (section "Operations de crise")
  → Dons en crypto (iroh wallet simple, pas de banque
    necessaire) et/ou lien vers page cagnotte externe
    (GoFundMe, Donorbox)
  → Les donateurs voient en temps reel le financement
    via iroh-docs CRDT
  │
  ▼
HEURE 2-12 : Des apps sont generees en temps reel
  → Le systeme de generation composee (GENERATION_COMPOSEE.md)
    cree des apps specifiques a la crise
  → Un dev volontaire tape : "Genere une app de carte
    des zones inondees pour Kherson avec signalement
    citoyen des routes coupees"
  → Le LLM distribue (GPU de la communaute mondiale)
    genere l'app en ~30 minutes
  → L'app est scannee, validee, deployee sur le reseau
  → Les premiers noeuds sur zone la recuperent
  │
  ▼
JOUR 1-3 : Le hardware est achete et expedie
  → Les volontaires locaux (ou ONG partenaire) achetent
    les noeuds LoRa avec les fonds de la cagnotte
  → Alternative : des stocks pre-positionnes dans des
    hubs regionaux (voir ci-dessous)
  → Les noeuds sont deployes sur les points hauts
    (toits, pylones, arbres)
  │
  ▼
JOUR 2+ : Le reseau mesh est operationnel
  → 32 noeuds couvrent 12 km2
  → Les apps de crise se propagent via le mesh
  → Les statuts "je suis vivant" remontent vers
    le monde exterieur via la premiere passerelle
    satellite/4G
  │
  ▼
APRES LA CRISE : L'operation se ferme
  → Bilan transparent : chaque euro depense est trace
    dans le CRDT (immuable, signe par chaque acteur)
  → Les noeuds LoRa restent en place comme infrastructure
    permanente pour le quartier
  → Le RETEX (retour d'experience) est automatique :
    timeline, messages, decisions — tout est dans le CRDT
```

### Financement — 3 sources combinees

| Source | Pour qui | Comment |
|--------|---------|---------|
| **Micro-dons communaute SBFB** | Les 52M d'utilisateurs Ollama, les gamers D&D, la communaute r/preppers | Bouton "Donner $5" dans l'app Browse quand une operation est active |
| **Grants institutionnels** | EU Horizon Europe, FEMA, UNHCR Innovation, NLNet | Propositions de subventions basees sur les RETEX des operations precedentes |
| **Stocks pre-positionnes par ONG** | Croix-Rouge, MSF, OCHA | Partenariat : l'ONG stocke des kits LoRa+SBFB dans ses entrepots regionaux, SBFB fournit le logiciel |

### Le kit de deploiement rapide

```
Kit "SBFB Emergency" — $100 par unite
┌─────────────────────────────────────┐
│  5x noeuds Meshtastic LoRa         │
│     (Heltec V3, $20 chacun)        │
│  5x panneau solaire 5W             │
│     ($5 chacun)                     │
│  1x carte SD avec SBFB pre-installe│
│     + apps de crise generiques      │
│  1x guide deploiement (3 pages,    │
│     pictogrammes, multi-langue)     │
│  Total : $100 pour couvrir ~10km2  │
└─────────────────────────────────────┘

Le kit couvre une ville de 10 000 habitants.
$0.01 par personne couverte.
```

**Pre-positionne** dans les hubs UNHCR, les entrepots Croix-Rouge,
les casernes de pompiers. Quand la catastrophe frappe, le kit est
deja sur place. Pas besoin d'attendre les fonds.

### Creation d'apps en direct — la generation composee de crise

Le systeme de generation composee (GENERATION_COMPOSEE.md)
prend une dimension critique en situation de catastrophe :

```
Volontaire dev (a Paris, en securite) :
  "Genere une app de distribution d'eau potable
   pour Kherson. Points de distribution sur une carte.
   Les habitants signalent les points vides. L'IA
   optimise les itineraires des camions-citernes."

Le LLM distribue (GPU de la communaute mondiale) :
  → Trouve dans l'index : carte collaborative (app capteurs),
    matching besoins/offres (app entraide), signalement
    (app catastrophe)
  → Combine et adapte en ~30 min
  → Traduit l'interface en ukrainien (LLM traduction locale)
  → Scanne pour la securite
  → Deploie sur le reseau

30 minutes plus tard :
  → L'app est sur le mesh de Kherson
  → Les habitants l'utilisent pour trouver de l'eau
```

**C'est ca la puissance de la generation composee en crise** :
chaque catastrophe est differente, mais les briques (carte,
signalement, matching, communication) sont les memes. Le LLM
assemble les briques existantes pour le contexte specifique.

### LoraType — l'inspiration ukrainienne

Le projet LoraType (github.com/AutomationArt/LoraType),
cree par un labo R&D ukrainien, est un communicateur de
dernier recours : ESP32 + LoRa + clavier + ecran e-ink.
Citation : "Created not by a Corporation for population,
but by people for people."

SBFB peut integrer LoraType comme noeud compatible.
Le firmware Meshtastic ukrainien (github.com/meshtastic-ua)
est deja actif avec des patches specifiques au contexte.

### Tracabilite des fonds — zero opacite

Chaque euro de la cagnotte est trace dans un iroh-doc CRDT :

```json
{
  "operation": "inondation-kherson-2026",
  "transactions": [
    {
      "type": "donation",
      "from": "alice_node_id",
      "amount_eur": 20,
      "timestamp": "2026-06-15T14:32:00Z",
      "signature": "ed25519:..."
    },
    {
      "type": "purchase",
      "item": "5x Heltec V3 LoRa nodes",
      "amount_eur": 95,
      "vendor": "Aliexpress order #XXX",
      "receipt_blob": "iroh:abc123...",
      "purchased_by": "volontaire_terrain_node_id",
      "timestamp": "2026-06-16T09:15:00Z",
      "signature": "ed25519:..."
    },
    {
      "type": "deployment",
      "node_serial": "HEL-V3-0042",
      "location_gps": [46.6354, 32.6169],
      "location_name": "Toit ecole #4, Korabelny",
      "deployed_by": "volontaire_terrain_node_id",
      "photo_blob": "iroh:def456...",
      "timestamp": "2026-06-17T11:30:00Z",
      "signature": "ed25519:..."
    }
  ],
  "balance_eur": 3805,
  "nodes_deployed": 32,
  "nodes_target": 50,
  "coverage_km2": 12
}
```

**Chaque don, chaque achat, chaque noeud deploye est signe
cryptographiquement et visible par tous.** Pas de caisse noire.
Pas de doute sur ou va l'argent. C'est le standard que les
ONG traditionnelles ne peuvent pas atteindre.

### Volontariat GPU a distance

Les gens qui ne peuvent pas donner d'argent ni aller sur le
terrain peuvent contribuer **leur GPU** :

```
┌──────────────────────────────────────────────────┐
│  Operation Kherson — Contribution GPU            │
│──────────────────────────────────────────────────│
│                                                  │
│  Votre GPU est utilise pour :                    │
│                                                  │
│  ✓ Generer l'app "carte des zones inondees"      │
│    (terminee en 28 min grace a 12 GPU)           │
│                                                  │
│  → Traduire l'interface en ukrainien             │
│    (en cours, votre GPU contribue)               │
│    ████████████░░░░ 73%                          │
│                                                  │
│  En attente :                                    │
│  ○ App "recherche personnes disparues"            │
│  ○ Traduction en roumain (refugies Moldavie)     │
│                                                  │
│  Votre contribution : 4h32 de GPU ce mois        │
│  Operations aidees : 2 (Kherson, Valence)        │
│                                                  │
│  [x] Contribuer mon GPU aux operations de crise  │
│  [ ] Limiter a 50% GPU                           │
│  [ ] Uniquement la nuit                          │
└──────────────────────────────────────────────────┘
```

**3 facons de contribuer, selon ce que tu as :**
1. **De l'argent** → cagnotte pour le hardware
2. **Du GPU** → analyse permanente + generation d'apps + traduction
3. **Du temps terrain** → deployer les noeuds physiquement

---

## IA de situation permanente — salle de crise distribuee

### Le concept

Le GPU partage ne sert pas juste a generer des apps
ponctuellement. Il fait tourner un **LLM permanent qui monitore
toute l'operation en temps reel**, 24h/24, alimente par chaque
message, chaque capteur, chaque signalement du mesh.

C'est une **salle de crise distribuee mondiale** : pas un
batiment a Geneve avec 50 analystes, mais 47 GPU repartis dans
12 pays qui font le meme travail en continu.

### Ce que le LLM permanent analyse

Toutes les donnees de l'operation convergent dans le CRDT :
- Messages du mesh (~200/heure)
- Statuts "je suis vivant" (GPS + timestamp)
- Signalements citoyens (routes coupees, besoins)
- Donnees de triage medical
- Points de distribution (eau, nourriture)
- Transactions de la cagnotte
- Noeuds LoRa deployes / etat du reseau

Le LLM produit en continu :

```
┌──────────────────────────────────────────────────┐
│  IA Situation — Operation Kherson                │
│  LLM 70B distribue sur 47 GPU mondiaux          │
│  Actualise toutes les 30 secondes                │
│──────────────────────────────────────────────────│
│                                                  │
│  SYNTHESE 16h32 :                                │
│  "Le niveau d'eau monte dans le secteur sud      │
│  (+40cm en 2h, 3 capteurs concordants).          │
│  12 familles signalees dans la zone — aucune     │
│  n'a marque 'en securite' depuis 3h.             │
│  Le centre d'hebergement gymnase Pasteur est     │
│  a 78% de capacite (156/200). Le point de        │
│  distribution eau Mairie est signale vide         │
│  depuis 45 min par 4 citoyens.                   │
│  Recommandation : evacuation secteur sud +        │
│  reapprovisionnement eau prioritaire."           │
│                                                  │
│  ALERTES ACTIVES :                               │
│  - Secteur sud : montee des eaux (3 capteurs)    │
│  - 12 familles sans signal depuis 3h             │
│  - Point d'eau Mairie vide depuis 45min          │
│  - Gymnase a 78% — prevoir 2e hebergement        │
│  - Hopital : flux stabilise (8 entrees/h)        │
│                                                  │
│  PATTERN DETECTE :                               │
│  "Les messages du secteur est mentionnent         │
│  'odeur de gaz' (3 messages en 20 min depuis     │
│  3 personnes differentes). Possible fuite de     │
│  gaz suite a la rupture de canalisation.          │
│  Alerte envoyee aux pompiers sur le mesh."       │
│                                                  │
│  STATS OPERATION :                               │
│  Personnes en securite confirmee : 847/~2000     │
│  Personnes recherchees : 23                       │
│  Personnes retrouvees aujourd'hui : 7             │
│  Messages mesh traites : 4,271                    │
│  Benevoles actifs : 127                           │
│  GPU contribues mondialement : 47                 │
└──────────────────────────────────────────────────┘
```

### Ce qu'un humain ne peut pas faire mais le LLM oui

| Tache | Analyste humain | LLM permanent SBFB |
|-------|----------------|-------------------|
| Lire 200 messages/heure en continu | Impossible (fatigue apres 2h) | 24h/24 sans interruption |
| Detecter "3 personnes mentionnent odeur de gaz dans le meme secteur en 20 min" | Rate dans le flux | **Detecte en temps reel** |
| Croiser capteurs (eau monte) + statuts (12 familles silencieuses) | Possible mais lent (~30min) | **Instantane** — alerte composite en 30s |
| Traduire ukrainien → anglais → roumain en continu | 1 traducteur par langue, couteux | **Automatique** sur chaque message |
| Rapport structuré pour les autorites | 2-4h par rapport, 1x/jour | **Toutes les 30 secondes**, format ONG standard |
| Matcher 200 besoins avec 127 benevoles | Tableur Excel, 1h | **Temps reel**, a chaque nouveau besoin ou benevole |

### Detection de patterns — la feature critique

Le LLM lit chaque message individuellement ET detecte les
correlations que personne ne voit :

```
Pattern 1 — Fuite de gaz :
  Message 14h12 : "ca sent bizarre rue de la Gare" (Ahmed)
  Message 14h25 : "odeur de gaz pres du pont" (Olena)
  Message 14h31 : "mon voisin dit que ca sent le gaz" (Petro)
  → 3 personnes, 3 endroits proches, 20 minutes
  → LLM : "ALERTE : possible fuite de gaz secteur est.
    3 signalements independants en 20min. Recommandation :
    evacuation perimeter 500m + alerte pompiers."

Pattern 2 — Zone silencieuse :
  Secteur sud : 12 familles signalees hier
  Aujourd'hui : 0 message depuis 3h
  Capteurs : niveau d'eau +40cm
  → LLM : "ALERTE : 12 familles sans signal dans zone
    ou l'eau monte. Derniers contacts : entre 12h et 13h.
    Priorite evacuation maximale."

Pattern 3 — Capacite depassee :
  Gymnase : +8 arrivees/heure, capacite 200
  Projection : plein dans ~5h
  Mairie : signale "eau potable epuisee"
  → LLM : "ALERTE LOGISTIQUE : le gymnase sera sature
    a 22h au rythme actuel. Identifier un 2e site.
    Le point d'eau Mairie est a sec — reapprovisionner
    ou rediriger vers l'ecole Moulin (capteur OK)."
```

Aucun coordinateur humain ne peut faire ces correlations
en temps reel sur des centaines de messages en 3 langues.

### Architecture GPU permanente

```
Operation active : Kherson (critique)

GPU mondiaux contribues : 47
  France    : 12 GPU (dont 3x RTX 4090)
  Allemagne : 8 GPU
  USA       : 15 GPU
  Canada    : 5 GPU
  Japon     : 4 GPU
  Autres    : 3 GPU

Repartition des taches permanentes :

  ┌─ Analyse situationnelle (70B, 3 GPU, continu)
  │  Lit TOUS les messages du CRDT
  │  Synthese toutes les 30s
  │  Detection de patterns d'urgence
  │
  ├─ Traduction live (8B, 1 GPU, continu)
  │  Chaque message en 3 langues
  │  Ukrainien ↔ Anglais ↔ Roumain
  │
  ├─ Matching besoins/ressources (8B, 1 GPU, continu)
  │  "Famille de 4 cherche hebergement"
  │  → "Gymnase Pasteur a 44 places, 1.2km"
  │
  ├─ Detection de personnes (8B, 1 GPU, continu)
  │  Croise "personne recherchee" avec "je suis vivant"
  │  → "Lucas Dupont recherche → signal detecte pres
  │     de l'ecole Moulin il y a 45min"
  │
  ├─ Generation d'apps a la demande (70B, 3 GPU, ponctuel)
  │  Apps specifiques quand un nouveau besoin emerge
  │
  └─ Rapports automatiques (8B, 1 GPU, toutes les 6h)
      Format ONG standard (OCHA sitrep)
      Envoye quand un noeud a internet
```

### Multi-operations simultanees

```
┌──────────────────────────────────────────────────┐
│  Operations actives sur le reseau SBFB           │
│──────────────────────────────────────────────────│
│                                                  │
│  CRITIQUE — Inondation Kherson (Ukraine) J+3     │
│  847 en securite / ~2000                         │
│  47 GPU │ $3,900 collectes │ 32 noeuds LoRa      │
│                                                  │
│  CRITIQUE — Seisme Antalya (Turquie) J+1         │
│  210 en securite / ~5000                         │
│  23 GPU │ $1,200 collectes │ 8 noeuds LoRa       │
│                                                  │
│  ACTIF — Feux de foret Valence (Espagne) J+5     │
│  Phase reconstruction                            │
│  12 GPU │ $8,500 collectes                        │
│                                                  │
│  TERMINE — Crue Garonne (France)                 │
│  RETEX disponible │ 15 noeuds permanents          │
│                                                  │
│  [Toutes les operations] [Creer] [Contribuer GPU]│
└──────────────────────────────────────────────────┘
```

Les GPU de la communaute se repartissent entre les operations
selon l'urgence — J+1 a la priorite sur J+5.

### Comparaison avec OCHA (ONU)

OCHA (Bureau de coordination humanitaire de l'ONU) a un budget
annuel de $300M. Un "situation report" OCHA prend des jours a
produire, passe par des dizaines de validations, et arrive
souvent quand les decisions ont deja ete prises sur le terrain.

| Critere | OCHA / ONU | SBFB IA permanente |
|---------|-----------|-------------------|
| Budget | $300M/an | Micro-dons + GPU benevole |
| Temps pour un sitrep | Jours | **30 secondes** |
| Frequence | 1x/jour au mieux | **Continu** |
| Langues | Anglais + francais | **Toutes** (traduction LLM) |
| Detection patterns | Analystes humains (fatigue) | **24h/24** sans interruption |
| Donnees terrain | Rapports manuels des equipes | **Temps reel** via mesh |
| Couverture | Zones avec presence ONU | **Partout ou il y a un noeud** |
| Transparence | Rapports publies apres coup | **Visible en direct** par tous |

SBFB ne remplace pas l'ONU — les ONG ont les ressources
logistiques (camions, helicopteres, medecins). Mais SBFB peut
fournir a l'ONU une **vision situationnelle en temps reel**
qu'ils n'ont jamais eue, alimentee par les gens sur le terrain,
analysee par les GPU du monde entier.

### Apres la crise — les noeuds restent

Les noeuds LoRa deployes pendant la crise ne sont pas retires.
Ils deviennent une **infrastructure permanente** pour le
quartier :
- Reseau mesh citoyen (communication locale sans telecom)
- Capteurs environnementaux (qualite de l'air, niveau d'eau)
- Alerte precoce pour la prochaine crise
- Point d'acces aux apps SBFB du reseau

L'operation humanitaire laisse derriere elle une infrastructure
resiliente. Chaque crise renforce le reseau pour la suivante

Sources:
- [Communication failures 80% sentinel events (Joint Commission)](https://www.finchmccranie.com/blog/healthcare-communication-failures-and-medical-malpractice-what-patients-need-to-know-in-2026/)
- [Cell towers 12h backup only (TowerPoint)](https://towerpoint.com/hurricanes-cell-towers-how-did-they-fare/)
- [Information isolation pockets (Scientific American)](https://www.scientificamerican.com/article/how-fires-floods-and-hurricanes-create-deadly-pockets-of-information-isolation/)
- [Meshtastic 10+ jours autonomie, deploiement Berlin 2026](https://ai-tec.eu/blackout-meshtastic-off-grid-communication/)
- [139.3M personnes deplacees (UNHCR 2025)](https://www.unhcr.org/refugee-statistics)
- [FEMA IoT flood sensors approved (Green Tech Book WIPO)](https://www.wipo.int/web-publications/green-technology-book-solutions-for-confronting-climate-disasters/en/communications-and-digital-coordination.html)
- [99% message delivery hybrid mesh (arxiv 2024)](https://arxiv.org/html/2410.13977v1)
- [Economic losses 7.4x direct damage (GIR 2025)](https://cdri.world/)
- [65.9% hospitals prepared reunification (HHS ASPR)](https://asprtracie.hhs.gov/technical-resources/64/family-reunification-and-support/0)
