# Analyse du Repository danieldurnea/FBI-tools

**Date**: 2026-04-05
**Repository**: https://github.com/danieldurnea/FBI-tools
**Objectif**: Evaluer la pertinence de chaque outil pour NEXUS (Cold Case Investigation System)

---

## 1. METADONNEES DU REPOSITORY

| Attribut | Valeur |
|----------|--------|
| Auteur | Daniel Durnea (unique contributeur) |
| Cree le | 2021-12-26 |
| Dernier push | 2025-03-12 |
| Stars | ~2 451 |
| Forks | ~357 |
| Licence | **AUCUNE** |
| Langage | Aucun (README uniquement) |
| Taille | 151 KB |
| Fichiers | **1 seul fichier** : `Readme.md` (22 328 octets) |
| Branche | master |
| Issues | 7 ouvertes (dont spam), 2 fermees |
| Topics | osint, forensics, security, reconnaissance, penetration-testing |

## 2. NATURE REELLE DU REPOSITORY

**CONSTAT CRITIQUE** : FBI-tools n'est PAS un outil ni un framework. C'est une **liste curatee de liens** (awesome-list) vers des outils OSINT et forensics tiers. Le repository contient un unique fichier Readme.md avec ~100+ liens vers des projets GitHub externes.

- Pas de code source
- Pas de scripts
- Pas de configuration
- Pas de documentation technique
- Pas de tests
- Un seul contributeur qui fait des "Update Readme.md" periodiques
- Messages de commit non informatifs (tous "Update Readme.md")
- Issues polluees par du spam

**Qualite de curation** : Moyenne. Certains doublons (OsintNum liste 2 fois, MEAT liste 2 fois, Octosuite liste 2 fois). Pas de categorisation claire. Melange de projets actifs et abandonnes sans distinction.

---

## 3. INVENTAIRE COMPLET DES OUTILS REFERENCES

### 3.1 PLATEFORMES OSINT & FRAMEWORKS

| Outil | Stars | Lang | Licence | Dernier push | Actif | Pertinence NEXUS |
|-------|-------|------|---------|-------------|-------|-----------------|
| **SpiderFoot** | 17 278 | Python | MIT | 2024-12 | Semi | **HAUTE** |
| **BBOT** | 9 576 | Python | AGPL-3.0 | 2026-04 | Oui | **HAUTE** |
| **reNgine** | 8 539 | HTML/Py | GPL-3.0 | 2026-03 | Oui | MOYENNE |
| **GHunt** | 18 639 | Python | Autre | 2026-03 | Oui | **HAUTE** |
| **OSINT Toolkit** | 846 | JS | MIT | 2025-08 | Oui | MOYENNE |
| **web-check** (Lissy93) | ~10 000+ | JS | MIT | Actif | Oui | MOYENNE |
| **iKy** | 942 | Python | GPL-3.0 | 2026-03 | Oui | **HAUTE** |
| **OS-Surveillance** | - | - | - | - | ? | FAIBLE (SaaS) |
| **gOSINT** | ~500 | Go | - | Ancien | Non | FAIBLE |
| **Karma v2** | ~1 500 | - | - | - | ? | FAIBLE |
| **Sub3suite** | ~500 | C++ | - | - | ? | FAIBLE |
| **Collector** | ~100 | Python | - | - | ? | FAIBLE |
| **Mr.Holmes** | ~800 | Python | - | - | ? | FAIBLE |
| **Ghost Recon** | ~200 | Python | - | - | ? | FAIBLE |
| **Gasmask** | ~1 000 | Python | - | - | ? | FAIBLE |
| **Infoooze** | ~300 | NodeJS | - | - | ? | FAIBLE |

### 3.2 RECHERCHE PAR IDENTITE (Username/Email/Telephone)

| Outil | Stars | Lang | Licence | Dernier push | Actif | Pertinence NEXUS |
|-------|-------|------|---------|-------------|-------|-----------------|
| **Social Analyzer** | 22 496 | JS | AGPL-3.0 | 2026-01 | Oui | **CRITIQUE** |
| **Holehe** | 10 579 | Python | GPL-3.0 | 2024-09 | Semi | **HAUTE** |
| **Blackbird** | 5 914 | Python | Aucune | 2025-07 | Oui | **HAUTE** |
| **Profil3r** | ~1 500 | Python | - | - | ? | MOYENNE |
| **Alfred** | ~300 | Python | - | - | ? | FAIBLE |
| **CrossLinked** (LinkedIn) | ~1 500 | Python | - | - | ? | MOYENNE |
| **DaProfiler** | ~500 | Python | - | France only | ? | MOYENNE |
| **Toutatis** (Instagram) | ~2 000 | Python | - | - | ? | MOYENNE |
| **Quidam** (Forgot pwd) | ~500 | Python | - | - | ? | MOYENNE |
| **UsernameSearchOSINT** | ~100 | Python | - | - | ? | FAIBLE |
| **OsintNum** (tel) | ~100 | Python | - | - | ? | FAIBLE |
| **WhatsOSINT** | ~200 | Python | - | - | ? | FAIBLE |
| **Terra** (Twitter/IG) | ~300 | Python | - | - | ? | FAIBLE |
| **check-if-email-exists** | ~1 500 | Rust | - | - | ? | MOYENNE |

### 3.3 RECHERCHE EMAIL AVANCEE

| Outil | Stars | Lang | Licence | Dernier push | Actif | Pertinence NEXUS |
|-------|-------|------|---------|-------------|-------|-----------------|
| **iKy** | 942 | Python | GPL-3.0 | 2026-03 | Oui | **HAUTE** |
| **Holehe** | 10 579 | Python | GPL-3.0 | 2024-09 | Semi | **HAUTE** |
| **Infoga** | ~2 000 | Python | - | Ancien | Non | FAIBLE (abandonne) |
| **Protintelligence** (Proton) | ~500 | Python | - | - | ? | MOYENNE |
| **ProtOSINT** (Proton) | ~300 | Python | - | - | ? | FAIBLE |

### 3.4 DARK WEB & TOR

| Outil | Stars | Lang | Licence | Dernier push | Actif | Pertinence NEXUS |
|-------|-------|------|---------|-------------|-------|-----------------|
| **OnionSearch** | 1 653 | Python | GPL-3.0 | 2024-08 | Semi | **HAUTE** |
| **Prying Deep** | 577 | Go | GPL-3.0 | 2024-09 | **ARCHIVE** | MOYENNE |
| DarkWebLink.io | - | Web | - | - | ? | FAIBLE (lien) |
| HiddenWiki.se | - | Web | - | - | ? | FAIBLE (lien) |
| OnionHub.com | - | Web | - | - | ? | FAIBLE (lien) |

### 3.5 MESSAGERIES & RESEAUX SOCIAUX

| Outil | Stars | Lang | Licence | Dernier push | Actif | Pertinence NEXUS |
|-------|-------|------|---------|-------------|-------|-----------------|
| **Telepathy** (Telegram) | 1 202 | Python | MIT | 2026-02 | Oui | **HAUTE** |
| **Telegram Trilateration** | ~500 | Python | - | - | ? | MOYENNE |
| **Telegram Nearby Map** | ~1 000 | JS | - | - | ? | MOYENNE |
| **Twayback** (tweets archives) | ~500 | Python | - | - | ? | MOYENNE |
| **fb_friend_list_scraper** | ~1 000 | Python | - | - | ? | MOYENNE |
| **Social Path** | ~500 | Python | - | - | ? | FAIBLE |
| **Darvester** (Discord) | ~200 | Python | - | - | ? | FAIBLE |
| **WhatsOSINT** | ~200 | Python | - | - | ? | FAIBLE |
| **Kupa3** (trackers web) | ~300 | Python | - | - | ? | FAIBLE |

### 3.6 FORENSICS NUMERIQUE

| Outil | Stars | Lang | Licence | Dernier push | Actif | Pertinence NEXUS |
|-------|-------|------|---------|-------------|-------|-----------------|
| **IPED** | 2 490 | Java | Autre | 2026-04 | Oui | **HAUTE** |
| **Turbinia** (Google) | 788 | Python | Apache-2.0 | 2026-03 | Oui | **HAUTE** |
| **Hayabusa** (Windows logs) | 3 096 | Rust | AGPL-3.0 | 2026-03 | Oui | MOYENNE |
| **MVT** (mobile) | 12 258 | Python | Autre | 2026-04 | Oui | MOYENNE |
| **IRIS Web** (IR platform) | 1 458 | Python | LGPL-3.0 | 2026-02 | Oui | **HAUTE** |
| **PowerForensics** | ~1 000 | PS | - | Ancien | Non | FAIBLE |
| **Live Forensicator** | ~500 | PS | - | - | ? | FAIBLE |
| **Forensix** (Chrome) | ~300 | JS | - | - | ? | FAIBLE |
| **Sabonis** (DFIR pivot) | ~100 | Python | - | - | ? | FAIBLE |
| **Linux Explorer** | ~300 | JS | - | - | ? | FAIBLE |
| **Judge Jury Executable** | ~500 | C# | - | - | ? | FAIBLE |
| **Avilla Forensics** (WhatsApp) | ~1 000 | - | - | - | ? | MOYENNE |
| **Whapa** (WhatsApp) | ~500 | Python | - | - | ? | MOYENNE |
| **RdpCacheStitcher** | ~200 | C++ | - | - | ? | FAIBLE |
| **SimpleImager** | ~100 | - | - | - | ? | FAIBLE |

### 3.7 FORENSICS MOBILE

| Outil | Stars | Lang | Licence | Dernier push | Actif | Pertinence NEXUS |
|-------|-------|------|---------|-------------|-------|-----------------|
| **MVT** | 12 258 | Python | Autre | 2026-04 | Oui | MOYENNE |
| **Andriller** | ~1 000 | Python | - | - | ? | MOYENNE |
| **MEAT** (iOS) | ~500 | Python | - | - | ? | FAIBLE |
| **Androidqf** | ~300 | Go | - | - | ? | FAIBLE |
| **ANDROPHSY** | ~200 | - | - | - | ? | FAIBLE |
| **iOS Freq Locations** | ~300 | Python | - | - | ? | MOYENNE |

### 3.8 IMAGE, METADATA & VISION

| Outil | Stars | Lang | Licence | Dernier push | Actif | Pertinence NEXUS |
|-------|-------|------|---------|-------------|-------|-----------------|
| **face_recognition** | 56 263 | Python | MIT | 2024-08 | Semi | **CRITIQUE** |
| **imago-forensics** | 268 | Python | MIT | 2021-12 | Non | FAIBLE (abandonne) |
| **ExifLooter** | ~500 | Go | - | - | ? | MOYENNE |
| **autoexif** | ~100 | Python | - | - | ? | FAIBLE |
| **PDFMtEd** | ~200 | - | - | - | ? | FAIBLE |
| **Audio Metadata** | ~100 | JS | - | - | ? | FAIBLE |
| **App Metadata** | ~500 | - | - | - | ? | FAIBLE |
| **roop** (face swap) | ~30 000+ | Python | - | - | ? | FAIBLE (ethique) |

### 3.9 GEOLOCALISATION & IP

| Outil | Stars | Lang | Licence | Dernier push | Actif | Pertinence NEXUS |
|-------|-------|------|---------|-------------|-------|-----------------|
| **Geospatial Intel Library** | ~500 | - | - | - | ? | MOYENNE |
| **IP Geolocation** | ~500 | Python | - | - | ? | FAIBLE |
| **Cameradar** (RTSP) | ~4 000 | Go | - | - | ? | MOYENNE |
| **Kamerka GUI** (IoT) | ~2 000 | Python | - | - | ? | MOYENNE |

### 3.10 DOMAINES, DNS & RECONNAISSANCE WEB

| Outil | Stars | Lang | Licence | Dernier push | Actif | Pertinence NEXUS |
|-------|-------|------|---------|-------------|-------|-----------------|
| **SquatSquasher** | ~200 | Python | - | - | ? | FAIBLE |
| **Chiasmodon** | ~500 | Python | - | - | ? | FAIBLE |
| **opensquat** | ~1 000 | Python | - | - | ? | FAIBLE |
| **GooFuzz** | ~1 000 | Shell | - | - | ? | FAIBLE |
| **commit-stream** | ~500 | Go | - | - | ? | FAIBLE |

### 3.11 THREAT INTELLIGENCE & MONITORING

| Outil | Stars | Lang | Licence | Dernier push | Actif | Pertinence NEXUS |
|-------|-------|------|---------|-------------|-------|-----------------|
| **Mihari** | 934 | Ruby | MIT | 2026-03 | Oui | **HAUTE** |
| **Oblivion** (data leaks) | ~300 | Python | - | - | ? | MOYENNE |
| **ransomposts** | ~300 | - | - | - | ? | MOYENNE |
| **teler** (HTTP IDS) | ~2 000 | Go | - | - | ? | FAIBLE |
| **TRACEE** (eBPF) | ~3 000 | Go | Apache | Actif | Oui | FAIBLE |
| **Bevigil CLI** | ~200 | Python | - | - | ? | FAIBLE |

### 3.12 RESSOURCES EDUCATIVES & COLLECTIONS

| Ressource | Description | Pertinence |
|-----------|-------------|------------|
| MetaOSINT | Collection 4000+ ressources OSINT | Reference |
| Linux for OSINT 21-day | Cours cipher387 | Educatif |
| Python for OSINT 21-day | Cours cipher387 | Educatif |
| OSINT Cheat Sheet | PirateMoo gitbook | Reference |
| osint_stuff_tool_collection | cipher387, centaines d'outils | Reference |
| OhShINT | Blog + ressources OSINT | Reference |
| awesome-forensics | Liste curatee forensics | Reference |
| ForensicsTools | Liste outils forensics | Reference |
| DistroForensics | ISOs specialisees | Reference |
| OSINTko | Kali + OSINT packages | Reference |
| CyberPunkOS | VM anti-fake news | Faible |
| OSINT JUMP | VM recon | Faible |
| FBI Crime Data Explorer | Donnees FBI | MOYENNE |
| Offensive OSINT Blog | Blog | Reference |

---

## 4. OUTILS A INTEGRER EN PRIORITE DANS NEXUS

### TIER 1 — Integration directe prioritaire (semaines 1-4)

#### 1. Social Analyzer (22 496 stars, AGPL-3.0, actif)
- **Fonction**: Recherche de profils sur 1000+ reseaux sociaux via API/CLI/Web
- **Pourquoi**: Piece maitresse pour tracer une identite a travers les reseaux
- **Integration**: API REST, output JSON, s'interface nativement avec un pipeline
- **Synergie NEXUS**: Resultats → gemma4:e4b (extraction entites) → Neo4j (graphe relations)
- **Risque**: AGPL-3.0 (copyleft fort) — utiliser comme service externe, pas embarquer le code

#### 2. Holehe (10 579 stars, GPL-3.0, Python)
- **Fonction**: Verifie si un email est enregistre sur 120+ sites via forgot-password
- **Pourquoi**: Methode passive, aucun contact avec la cible, crucial pour cold cases
- **Integration**: Bibliotheque Python, import direct dans le backend FastAPI
- **Synergie NEXUS**: Email input → Holehe → sites trouves → SearXNG deep search par site
- **Risque**: Certains modules cassent quand les sites changent leur flow. Maintenance communautaire.

#### 3. Blackbird (5 914 stars, Python, actif)
- **Fonction**: Recherche de comptes par username ET email sur les reseaux sociaux
- **Pourquoi**: Complementaire a Social Analyzer, plus leger, output JSON
- **Integration**: CLI wrapper ou import Python
- **Synergie NEXUS**: Username → Blackbird → profils trouves → Neo4j + scoring nexus:26b

#### 4. OnionSearch (1 653 stars, GPL-3.0, Python)
- **Fonction**: Scrape des URLs sur differents moteurs de recherche .onion
- **Pourquoi**: Complete Robin (dark web search deja dans NEXUS) avec d'autres moteurs
- **Integration**: Script Python, facile a wrapper
- **Synergie NEXUS**: Queries → OnionSearch via Tor → resultats → ChromaDB + analyse nexus:26b
- **Risque**: Necessite Tor. Certains moteurs .onion disparaissent regulierement.

#### 5. GHunt (18 639 stars, Python, actif)
- **Fonction**: Framework offensif Google — extraction d'informations depuis comptes Google
- **Pourquoi**: Google est omnipresent, extraction de reviews, photos, lieux, calendriers
- **Integration**: CLI Python, output JSON structuré
- **Synergie NEXUS**: Email/ID Google → GHunt → donnees → Neo4j (lieux, relations, timeline)
- **Risque**: Necessite des cookies/tokens Google. Maintenance reactive aux changements Google.

### TIER 2 — Integration secondaire (semaines 5-8)

#### 6. BBOT (9 576 stars, AGPL-3.0, Python, tres actif)
- **Fonction**: Scanner internet recursif — OSINT, subdomains, emails, cloud, secrets
- **Pourquoi**: Framework modulaire avec 100+ modules, pipeline automatise
- **Integration**: API Python, output JSON/Neo4j natif
- **Synergie NEXUS**: Domaine cible → BBOT scan → donnees structurees → tous les stores NEXUS
- **Risque**: AGPL-3.0. Tres large scope — configurer finement pour eviter le bruit.

#### 7. SpiderFoot (17 278 stars, MIT, Python)
- **Fonction**: Automatisation OSINT, 200+ modules, threat intelligence, cartographie surface d'attaque
- **Pourquoi**: Le plus mature des outils OSINT open source, interface web incluse
- **Integration**: API REST, Docker, output en multiples formats
- **Synergie NEXUS**: Peut servir de moteur OSINT principal, resultats → Neo4j + ChromaDB
- **Risque**: Derniere activite dec 2024. Projet potentiellement en ralentissement. MIT = OK juridiquement.

#### 8. iKy (942 stars, GPL-3.0, Python, actif)
- **Fonction**: Collecte info depuis un email — profils, timeline, visualisation
- **Pourquoi**: Approche centree email avec visualisation de graphe integree
- **Integration**: Docker, API REST
- **Synergie NEXUS**: Email → iKy → profil complet → Neo4j

#### 9. Telepathy (1 202 stars, MIT, Python, actif)
- **Fonction**: Investigation de chats Telegram — membres, messages, media, metadata
- **Pourquoi**: Telegram est tres utilise dans les affaires criminelles
- **Integration**: CLI Python, output JSON
- **Synergie NEXUS**: Channel/groupe Telegram → Telepathy → messages → ChromaDB + analyse nexus:26b
- **Risque**: Necessite un compte Telegram + API keys.

#### 10. IRIS Web (1 458 stars, LGPL-3.0, Python, actif)
- **Fonction**: Plateforme collaborative de reponse aux incidents (DFIR)
- **Pourquoi**: Modele d'architecture similaire a NEXUS — timeline, artefacts, collaboration
- **Integration**: Etudier l'architecture plutot qu'integrer directement
- **Synergie NEXUS**: Source d'inspiration pour le dashboard et la gestion de cas

### TIER 3 — A evaluer / usage ponctuel

#### 11. Mihari (934 stars, MIT, Ruby, actif)
- **Fonction**: Aggregateur de requetes pour threat hunting OSINT
- **Pourquoi**: Monitoring automatise, alertes, integration multiples sources
- **Integration**: Ruby (pas Python) — utiliser comme service Docker separe
- **Synergie NEXUS**: Monitoring continu → alertes → re-evaluation hypotheses

#### 12. face_recognition (56 263 stars, MIT, Python)
- **Fonction**: API de reconnaissance faciale la plus simple du monde
- **Pourquoi**: NEXUS a deja CompreFace, mais cette lib est plus legere pour du batch
- **Integration**: Import Python direct
- **Synergie NEXUS**: **Deja couvert par CompreFace** — garder en backup/alternative
- **Note**: Pas maintenu activement (dernier push aout 2024, 832 issues ouvertes)

#### 13. IPED (2 490 stars, Java, actif)
- **Fonction**: Traitement et analyse de preuves numeriques (law enforcement)
- **Pourquoi**: Utilise par les forces de l'ordre bresiliennes, tres complet
- **Integration**: Java (pas Python) — outil standalone, pas d'API facile
- **Synergie NEXUS**: Usage ponctuel pour analyse de copies forensiques

#### 14. Turbinia (788 stars, Apache-2.0, Python, Google, actif)
- **Fonction**: Automatisation et mise a l'echelle des outils forensiques
- **Pourquoi**: Orchestre d'autres outils forensiques, backed by Google
- **Integration**: Docker, Cloud-oriented
- **Synergie NEXUS**: Pertinent si NEXUS traite des images disque/dumps memoire

---

## 5. EVALUATION DES RISQUES

### 5.1 Risques juridiques et licences

| Licence | Outils | Implication pour NEXUS |
|---------|--------|----------------------|
| **AGPL-3.0** | Social Analyzer, BBOT | **Attention** : si le code est integre, tout NEXUS doit etre AGPL. SOLUTION : utiliser comme services externes (Docker/API), pas d'import direct. |
| **GPL-3.0** | Holehe, OnionSearch, iKy, reNgine | Meme contrainte copyleft. Utiliser via subprocess/API. |
| **MIT** | SpiderFoot, OSINT Toolkit, IRIS Web, Mihari, face_recognition, imago | **OK** : integration libre, aucune contrainte. |
| **Apache-2.0** | Turbinia | **OK** : compatible usage commercial et prive. |
| **LGPL-3.0** | IRIS Web | **OK** : peut etre lie sans contaminer. |
| **Aucune** | FBI-tools lui-meme, Blackbird | **Risque** : pas de licence = tous droits reserves par defaut. Usage a verifier. |
| **Autre/Custom** | GHunt | Verifier les termes specifiques avant integration. |

### 5.2 Risques de maintenance

| Niveau | Outils | Situation |
|--------|--------|-----------|
| **Actif (2026)** | BBOT, GHunt, IRIS Web, IPED, MVT, Mihari, iKy, Telepathy, Turbinia, Hayabusa, Social Analyzer | Maintenance reguliere, communaute active |
| **Semi-actif (2024-2025)** | SpiderFoot, Holehe, Blackbird, OnionSearch | Derniers commits > 6 mois, fonctionnel mais ralentissement |
| **Abandonnes** | imago-forensics (2021), Infoga, PowerForensics | Ne pas integrer. Code obsolete. |
| **Archive** | Prying Deep | Officiellement archive. Ne pas integrer. |

### 5.3 Risques de securite

- **Chrome Extractor** / **Firefox Decrypt** : Outils d'extraction de mots de passe. Potentiel malveillant. **NE PAS INTEGRER** sauf cadre forensique strictement controle.
- **roop** (face swap) : Outil de deepfake. Aucune pertinence investigative. **EXCLURE**.
- **Telegram Trilateration** : Tracking de localisation. Legalement tres sensible. **EXCLURE** sauf mandat.
- **Cameradar** : Hack de cameras RTSP. **EXCLURE** — illegale sans autorisation.
- **DaProfiler** : Web scraping + Google dorking — peut declencher des bans IP. Utiliser avec prudence.

### 5.4 Risques techniques

- **Dependances API tierces** : OsintNum (APILayer), Bevigil, Chiasmodon — dependent de services payants/changeants
- **Rate limiting** : Social Analyzer, Holehe, Blackbird peuvent etre bloques par les plateformes
- **Tor** : OnionSearch necessite un proxy Tor fonctionnel (deja disponible via Robin dans NEXUS)
- **VRAM** : Aucun de ces outils n'utilise de GPU — pas de conflit avec la strategie multi-modele NEXUS

---

## 6. PLAN D'INTEGRATION CONCRET

### Phase 1 : Pipeline d'identification (Semaines 1-2, ~20h)

```
Objectif : Tracer une identite a travers le web a partir d'un nom/email/username

1. Installer Holehe dans l'environnement conda NEXUS
   pip install holehe
   → Wrapper Python dans backend/services/holehe_service.py
   → Endpoint FastAPI : POST /api/osint/email-check

2. Installer Blackbird
   pip install blackbird
   → Wrapper Python dans backend/services/blackbird_service.py
   → Endpoint FastAPI : POST /api/osint/username-search

3. Social Analyzer via Docker (AGPL, ne pas embarquer)
   docker run -p 9500:9500 qeeqbox/social-analyzer
   → Client HTTP dans backend/services/social_analyzer_client.py
   → Endpoint FastAPI : POST /api/osint/social-search

4. Pipeline orchestrateur :
   Input (email/username/phone)
   → gemma4:e4b : normalisation, extraction variantes
   → Holehe (email) + Blackbird (username) + Social Analyzer (all)
   → Resultats → SQLite (raw) + Neo4j (relations) + ChromaDB (embeddings)
   → nexus:26b : scoring de pertinence, rapport
```

### Phase 2 : Recherche approfondie (Semaines 3-4, ~20h)

```
Objectif : Enrichir les profils trouves avec des donnees detaillees

1. GHunt (comptes Google)
   pip install ghunt
   → backend/services/ghunt_service.py
   → Endpoint : POST /api/osint/google-profile

2. OnionSearch (dark web, complete Robin)
   pip install onionsearch
   → backend/services/onion_search_service.py
   → Utiliser le proxy Tor de Robin (port 9090)
   → Endpoint : POST /api/osint/darkweb-search

3. Pipeline d'enrichissement :
   Profils trouves Phase 1
   → Pour chaque profil Google : GHunt → lieux, reviews, photos
   → Pour chaque nom/alias : OnionSearch → mentions dark web
   → Tout → Neo4j (enrichissement du graphe)
   → deepseek-r1:14b : verification croisee, contradictions
```

### Phase 3 : Monitoring continu (Semaines 5-6, ~15h)

```
Objectif : Surveillance automatique de nouvelles mentions

1. Configurer BBOT en mode Docker (AGPL)
   docker-compose pour BBOT
   → Client API dans backend/services/bbot_client.py
   → Scans periodiques configures dans le scheduler NEXUS

2. Evaluer Mihari pour alertes automatisees
   Docker container Ruby
   → Webhooks vers FastAPI

3. Integration avec le monitoring existant :
   Clearweb (SearXNG toutes les 6h) + BBOT enrichissement
   Dark web (Robin toutes les 24h) + OnionSearch complement
   → Nouvelles donnees → re-evaluation hypotheses (nexus:26b)
```

### Phase 4 : Forensics & messageries (Semaines 7-8, ~15h)

```
Objectif : Analyse de donnees Telegram et artefacts numeriques

1. Telepathy (Telegram)
   pip install telepathy-osint
   → backend/services/telepathy_service.py
   → Endpoint : POST /api/osint/telegram-investigate

2. Etudier IRIS Web pour l'architecture de gestion de cas
   → Adapter les patterns pour le dashboard Streamlit NEXUS

3. Optionnel : SpiderFoot comme moteur OSINT de backup
   docker run -p 5001:5001 spiderfoot
```

### Architecture d'integration finale

```
                     ┌─────────────┐
                     │   Streamlit  │
                     │  Dashboard   │
                     └──────┬───────┘
                            │
                     ┌──────┴───────┐
                     │   FastAPI    │
                     │   Backend    │
                     └──────┬───────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
        ┌─────┴─────┐ ┌────┴────┐ ┌──────┴──────┐
        │ OSINT      │ │ Search  │ │ Forensics   │
        │ Pipeline   │ │ Engine  │ │ Pipeline    │
        ├────────────┤ ├─────────┤ ├─────────────┤
        │ Holehe     │ │ SearXNG │ │ Telepathy   │
        │ Blackbird  │ │ Robin   │ │ ExifLooter  │
        │ Social     │ │ Onion   │ │ IPED (ext)  │
        │  Analyzer* │ │  Search │ │ Turbinia    │
        │ GHunt      │ │ BBOT*   │ │  (ext)      │
        │ iKy        │ │         │ │             │
        └─────┬──────┘ └────┬────┘ └──────┬──────┘
              │             │             │
              └─────────────┼─────────────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
        ┌─────┴────┐ ┌─────┴────┐ ┌──────┴─────┐
        │ SQLite   │ │ Neo4j    │ │ ChromaDB   │
        │ (donnees)│ │ (graphe) │ │ (vecteurs) │
        └──────────┘ └──────────┘ └────────────┘

        * = Containers Docker separes (licence AGPL)
```

---

## 7. CONCLUSION

### Le repo FBI-tools en resume
- **Nature** : Awesome-list, PAS un outil. Zero code, un seul fichier README.
- **Qualite** : Curation moyenne (doublons, pas de categorisation, melange actif/abandonne)
- **Valeur** : Bonne source de decouverte d'outils, mais chaque outil doit etre evalue individuellement
- **Maintenance** : Mises a jour sporadiques (toutes les 2-4 mois), un seul contributeur

### Ce qu'il faut retenir pour NEXUS
Sur ~100+ outils references, **10 sont reellement pertinents et integrables** :
1. **Social Analyzer** — identification cross-platform (CRITIQUE)
2. **Holehe** — verification email passive (HAUTE)
3. **Blackbird** — recherche username (HAUTE)
4. **GHunt** — intelligence Google (HAUTE)
5. **OnionSearch** — complement dark web (HAUTE)
6. **BBOT** — scanner OSINT automatise (HAUTE)
7. **SpiderFoot** — framework OSINT mature (HAUTE)
8. **iKy** — profiling email (HAUTE)
9. **Telepathy** — investigation Telegram (HAUTE)
10. **Mihari** — monitoring/alertes OSINT (HAUTE)

### Ce que NEXUS a deja et qui rend certains outils redondants
- **CompreFace** couvre deja face_recognition
- **Robin** couvre partiellement OnionSearch et Prying Deep
- **SearXNG** couvre partiellement les recherches web de SpiderFoot

### Budget temps estime pour l'integration complete : ~70h (8-10 semaines)
