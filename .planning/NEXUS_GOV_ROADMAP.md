# NEXUS GOV — Plateforme d'Intelligence Politique Autonome

## Vision
Systeme autonome 24/7 qui collecte, transcrit, analyse et cross-reference
TOUT ce que les politiciens francais disent et font publiquement.
Tweets, videos, interviews TV, votes, patrimoine, affaires judiciaires,
lobbying — tout lie temporellement, tout source, zero opinion.

Combine les capacites de PoliGraph (donnees structurees, 8 sources officielles)
+ NEXUS (LLM local uncensored, transcription, vision, investigation autonome)
+ social media intelligence (tweets, videos, podcasts).

---

## Phase 0 — Fondation (Semaine 1-2)

### 0.1 Restructuration en module independant
- Creer `nexus/gov/` comme module autonome
- Extraire le moteur reutilisable dans `nexus/engine/`:
  - EventBus, ReactiveWorker, VRAMScheduler, LLMRouter
  - OllamaClient, ChromaDB client, Neo4j client
  - SQLite helpers (get_db, _new_id, etc.)
- Le module gov importe depuis engine, ne depend PAS de cold case

### 0.2 Base de donnees dediee
Tables gov dans le meme SQLite (migration PostgreSQL quand necessaire):

```
gov_politicians        — 35K+ politiciens (actifs + historiques)
gov_mandates           — Mandats (depute, senateur, ministre, maire, eurodeput)
gov_parties            — Partis politiques avec couleurs
gov_party_memberships  — Historique des appartenances partisanes
gov_positions          — TOUT ce qu'ils disent/font (votes, tweets, interviews...)
gov_contradictions     — Contradictions detectees cross-source
gov_affairs            — Affaires judiciaires (timeline, statut, categorie)
gov_declarations       — Declarations patrimoine/interets HATVP
gov_laws               — Dossiers legislatifs + stats La Fabrique
gov_press              — Articles de presse mentionnant des politiciens
gov_social_posts       — Posts reseaux sociaux (Twitter, FB, Insta, TikTok)
gov_transcriptions     — Transcriptions video/audio (timestampees)
gov_factchecks         — Fact-checks agreg (AFP, Decodeurs, etc.)
gov_alerts             — Alertes sur contradictions/changements
gov_scan_log           — Historique de toutes les syncs
gov_external_ids       — IDs cross-source (AN, Senat, Wikidata, HATVP)
```

### 0.3 Identity Resolution Engine
- Reconciliation de politiciens a travers 9+ bases:
  AN (acteurRef), Senat (matricule), HATVP, Wikidata (QID),
  data.gouv.fr, nosdeputes.fr (slug), PoliGraph (slug)
- RapidFuzz (Jaro-Winkler) pour fuzzy matching noms
- Score de confiance: >= 0.95 auto-link, 0.70-0.95 review
- Table gov_external_ids pour stocker les mappings
- Normalisation noms: accents, particules (de, le, d'), tirets

---

## Phase 1 — Collecte de donnees officielles (Semaine 2-4)

### 1.1 Sources institutionnelles (deja code, a brancher)
| Source | Endpoint | Frequence | Worker |
|--------|----------|-----------|--------|
| AN Scrutins | Scrutins.json.zip (19MB, 6033 votes) | Quotidien | GovVoteSyncWorker |
| AN Dossiers | Dossiers_Legislatifs.json.zip (8.7MB) | Quotidien | GovLawSyncWorker |
| Senat API | /api-senat/senateurs.json (348) | Hebdo | GovSenatSyncWorker |
| data.gouv.fr | CSV deputes Datan | Hebdo | GovDeputeSyncWorker |
| HATVP | liste.csv (3.7MB, 18K+) | Mensuel | GovHATVPSyncWorker |
| La Fabrique | metrics.csv (1117 lois, 77 stats) | Hebdo | GovFabriqueSyncWorker |
| Wikidata | SPARQL (bios, photos, partis) | Hebdo | GovWikidataSyncWorker |
| PoliGraph | /api/affaires (260 affaires) | Quotidien | GovAffairsSyncWorker |

### 1.2 Presse (RSS + SearXNG)
Sources RSS:
- Le Monde politique: https://www.lemonde.fr/politique/rss_full.xml
- Le Figaro politique: https://www.lefigaro.fr/rss/figaro_politique.xml
- Franceinfo politique: https://www.francetvinfo.fr/politique.rss
- Liberation politique: https://www.liberation.fr/arc/outboundfeeds/rss/category/politique/
- Mediapart: RSS (si disponible)
- Public Senat: https://www.publicsenat.fr/rss
- LCP: https://lcp.fr/rss

Worker: GovPressSyncWorker (horaire)
- Fetch RSS → extraire articles → GLiNER NER → match politiciens
- SearXNG pour recherche complementaire (noms de politiciens)
- Dedup par URL
- Store dans gov_press

### 1.3 Fact-checks
- Google Fact Check Tools API (cle API gratuite)
  https://factchecktools.googleapis.com/v1alpha1/claims:search
- AFP Factuel RSS
- Les Decodeurs (Le Monde)
Worker: GovFactcheckSyncWorker (quotidien)

---

## Phase 2 — Social Media Intelligence (Semaine 4-6)

### 2.1 Twitter/X
- Scraper: snscrape ou ntscraper (sans API payante)
- Collecter les tweets de chaque politicien (925 comptes)
- Store dans gov_social_posts
- Worker: GovTwitterSyncWorker (toutes les 2h)
- Analyse: GLiNER NER + LLM classification + embedding

### 2.2 Facebook
- Scraper: facebook-scraper (public posts only)
- Posts publics des pages officielles
- Worker: GovFacebookSyncWorker (toutes les 6h)

### 2.3 Instagram
- Scraper: instaloader (public profiles)
- Posts + captions + stories publiques
- Worker: GovInstagramSyncWorker (quotidien)

### 2.4 TikTok
- Scraper: TikTok-Api ou yt-dlp
- Videos courtes + descriptions
- Worker: GovTikTokSyncWorker (quotidien)

### 2.5 YouTube
- yt-dlp pour download
- Chaines officielles de politiciens
- Chaines parlementaires (LCP, Public Senat)
- Worker: GovYouTubeSyncWorker (quotidien)
- Pipeline: download → faster-whisper → transcription timestampee

---

## Phase 3 — Transcription & Vision (Semaine 6-8)

### 3.1 Transcription audio/video
- faster-whisper (deja dans NEXUS) — modele large-v3
- Transcription timestampee (mot par mot avec timestamp)
- Store dans gov_transcriptions avec timestamps
- Lie a la source (YouTube URL, TV replay URL)
- Worker: GovTranscriptionWorker
  - Subscribe: GOV_VIDEO_DOWNLOADED
  - Output: GOV_TRANSCRIPTION_READY

### 3.2 Analyse visuelle
- CLIP embeddings: identifier les personnes dans les videos/photos
- Extraire les bandeaux TV (texte affiche en bas des JT)
- OCR sur les captures d'ecran de tweets (images de texte)
- Worker: GovVisionWorker
  - Subscribe: GOV_VIDEO_DOWNLOADED, GOV_IMAGE_ADDED
  - Utilise: vision module NEXUS existant

### 3.3 Pipeline video complet
```
Video URL detectee
  → yt-dlp download (audio + video)
    → faster-whisper transcription timestampee
      → GLiNER NER sur transcription (personnes, lois, lieux)
        → LLM resume + extraction positions
          → Embedding vectoriel (nomic-embed)
            → Cross-reference avec votes/declarations
              → Si contradiction → ALERTE
```

---

## Phase 4 — Analyse profonde (Semaine 8-12)

### 4.1 Detection de contradictions cross-source
Le coeur du systeme. LLM analyse:
- Tweet dit X ↔ Vote Y sur le meme sujet
- Interview TV dit X ↔ Declaration patrimoine montre Y
- Position 2020 ↔ Position 2026 (evolution/retournement)
- Promesse electorale ↔ Vote effectif

Worker: GovContradictionAnalyzer
- Subscribe: GOV_POSITION_ADDED
- Pour chaque nouvelle position:
  1. Recuperer toutes les positions du meme politicien sur le meme sujet
  2. Grouper par sujet (fuzzy match)
  3. LLM compare les paires les plus pertinentes
  4. Si contradiction: store + alerte

### 4.2 Analyse des patterns de vote
- Loyaute parti: % de votes conformes au groupe
- Coalitions cachees: qui vote ensemble contre son propre groupe
- Abstentions suspectes: quand et sur quoi
- Evolution temporelle: radicalisation/moderation

Worker: GovVotingPatternAnalyzer (hebdo)
- Calculs statistiques (pas LLM) sur les 6033+ scrutins
- Store: metriques par politicien + par parti

### 4.3 Graph d'influence
Neo4j construit le reseau:
```
(Politicien)-[:A_VOTE_POUR]->(Loi)
(Politicien)-[:MEMBRE_DE]->(Parti)
(Politicien)-[:A_DECLARE]->(Patrimoine)
(Entreprise)-[:A_LOBBY_POUR]->(Loi)
(Politicien)-[:MENTIONNE_AVEC]->(Politicien) [dans articles/tweets]
(Politicien)-[:A_DIT {date, source}]->(Position)
(Position)-[:CONTREDIT]->(Position)
(Politicien)-[:IMPLIQUE_DANS]->(Affaire)
```

Worker: GovNeo4jSyncWorker
- Subscribe: GOV_POSITION_ADDED, GOV_AFFAIR_ADDED, GOV_PRESS_ADDED
- Maintient le graph a jour en continu

### 4.4 Analyse de sentiment presse
- LLM analyse le ton des articles sur chaque politicien
- Tracking temporel: comment la couverture mediatique evolue
- Detection de campagnes mediatiques coordonnees
- Worker: GovSentimentAnalyzer (quotidien)

### 4.5 Classification thematique
13 domaines (comme PoliGraph):
Securite/Justice, Sante, Education, Economie, Environnement,
Social/Travail, Culture, Defense, Affaires etrangeres,
Agriculture, Transport, Numerique, Institutions

LLM classifie chaque position dans 1+ domaines
→ Permet de voir: "Sur l'environnement, ce politicien a dit X mais vote Y"

### 4.6 Score de coherence (pas de scoring de personnes)
PAS un score de confiance/suspicion. Un score de COHERENCE factuel:
- Combien de positions sont coherentes entre elles
- Nombre de contradictions detectees / nombre de positions
- Purement mathematique, pas de jugement
- Affiche comme: "42 positions, 3 contradictions detectees"

---

## Phase 5 — Frontend complet (Semaine 10-14)

### 5.1 Pages (inspire PoliGraph + surpasse)

| Page | Contenu | PoliGraph a? | NEXUS ajoute |
|------|---------|-------------|--------------|
| Dashboard | Stats globales, alertes, tendances | Oui (basique) | Alertes temps reel, tendances IA |
| Politicien | Fiche complete cross-source | Oui | Timeline cross-source (tweets+votes+TV) |
| Reseau | Graph interactif relations | Non (D3 basique) | Reagraph WebGL, clustering, pathfinding |
| Votes | Tous les scrutins + analyse | Oui | Patterns coalitions, loyaute calcule |
| Contradictions | Detectees auto, sourcees | Non | LLM cross-source, timeline |
| Affaires | Judiciaire, timeline, statut | Oui | Enrichi Wikidata + presse auto |
| Presse | Revue de presse agregee | Oui (RSS) | + SearXNG, sentiment tracking |
| Social | Tweets, posts, videos | Non | Timeline reseaux sociaux integree |
| Videos | Transcriptions + recherche | Non | faster-whisper, recherche dans videos |
| Declarations | Patrimoine HATVP | Oui | + evolution temporelle |
| Legislation | Dossiers en cours | Oui | + stats La Fabrique, impact |
| Comparateur | 2 politiciens cote a cote | Oui | + cross-source, pas juste votes |
| Timeline | Chronologie globale filtrable | Non | Tous canaux, tous politiciens |
| Recherche | Semantic search | Oui (RAG pgvector) | RAG ChromaDB local, uncensored |
| Alertes | Notifications live | Non | WebSocket temps reel |
| Carte | Carte interactive elus | Oui (Leaflet) | + Leaflet (deja dans NEXUS) |
| Recap | Resume hebdomadaire | Oui (newsletter) | + LLM local resume |

### 5.2 Composants shadcn
- Toutes les pages en shadcn/ui (deja installe)
- Reagraph pour le graph (deja installe)
- Recharts pour les stats (deja installe)
- Leaflet pour la carte (deja installe)
- DataTable pour les listes (deja code)

### 5.3 Temps reel
- WebSocket pour alertes live (nouvelle contradiction, nouveau vote)
- SSE bridge (deja dans NEXUS) pour les events

---

## Phase 6 — IA avancee (Semaine 12-16)

### 6.1 RAG politique
- Embeddings de TOUT le corpus (positions, articles, transcriptions)
- ChromaDB collections dediees politique
- Question en langage naturel:
  "Quand Marine Le Pen a-t-elle change de position sur l'Europe?"
  → RAG cherche dans toutes les sources → LLM synthetise avec sources

### 6.2 Biographies generees
- LLM genere une biographie factuelle par politicien
- Basee sur: mandats, votes, declarations, presse, affaires
- Mise a jour automatique quand nouvelles donnees

### 6.3 Resume de scrutin avec impact citoyen
- Pour chaque vote: LLM explique l'impact concret sur les citoyens
- "Ce vote signifie que [impact concret]"
- Source vers le texte de loi

### 6.4 Detection d'affaires dans la presse
- LLM analyse les articles de presse
- Detecte les nouvelles affaires judiciaires mentionnees
- Cross-reference avec Judilibre (API Cour de Cassation)
- Moderation automatique (presomption d'innocence)

### 6.5 Newsletter automatique "Alerte Politique"
- LLM compile le recap de la semaine
- Votes importants + contradictions + affaires
- Envoye par email (Mailjet ou SMTP local)

### 6.6 Publication automatique reseaux sociaux
- LLM genere des posts factuels sur les contradictions detectees
- Publication automatique sur Twitter/Bluesky
- 3x/jour comme PoliGraph

---

## Phase 7 — Scale & Production (Semaine 16+)

### 7.1 Migration PostgreSQL (quand SQLite atteint ses limites)
- pgvector remplace ChromaDB
- Concurrent writes illimites
- Full-text search FR natif (tsvector + stemming)
- Backup automatise

### 7.2 Extension EU
- Memes sources pour l'UE:
  European Parliament API (data.europarl.europa.eu)
  EUR-Lex (legislation EU)
- Extension aux 27 pays membres

### 7.3 API publique
- REST/JSON avec pagination (comme PoliGraph)
- Rate limiting, documentation OpenAPI
- Permet a d'autres projets de reutiliser les donnees

### 7.4 Resilience
- Fallback entre sources (PoliGraph → sources officielles → SearXNG)
- Circuit breaker sur chaque source
- Retry avec backoff exponentiel
- Health monitoring de chaque scraper

---

## Workers gouvernement (resume)

| Worker | Subscribe | Frequence | Source |
|--------|-----------|-----------|--------|
| GovVoteSyncWorker | TICK_DAILY | Quotidien | AN ZIP + Senat JSON |
| GovLawSyncWorker | TICK_DAILY | Quotidien | AN Dossiers ZIP |
| GovDeputeSyncWorker | TICK_WEEKLY | Hebdo | data.gouv.fr CSV |
| GovSenatSyncWorker | TICK_WEEKLY | Hebdo | Senat API |
| GovHATVPSyncWorker | TICK_MONTHLY | Mensuel | HATVP CSV |
| GovWikidataSyncWorker | TICK_WEEKLY | Hebdo | Wikidata SPARQL |
| GovFabriqueSyncWorker | TICK_WEEKLY | Hebdo | La Fabrique CSV |
| GovAffairsSyncWorker | TICK_DAILY | Quotidien | PoliGraph API + Wikidata |
| GovPressSyncWorker | TICK_HOURLY | Horaire | RSS + SearXNG |
| GovFactcheckSyncWorker | TICK_DAILY | Quotidien | Google Fact Check API |
| GovTwitterSyncWorker | TICK_2H | 2h | snscrape/ntscraper |
| GovFacebookSyncWorker | TICK_6H | 6h | facebook-scraper |
| GovYouTubeSyncWorker | TICK_DAILY | Quotidien | yt-dlp |
| GovTranscriptionWorker | GOV_VIDEO_DOWNLOADED | Continu | faster-whisper |
| GovVisionWorker | GOV_IMAGE_ADDED | Continu | CLIP/DINOv2 |
| GovContradictionAnalyzer | GOV_POSITION_ADDED | Continu | LLM local |
| GovVotingPatternAnalyzer | TICK_WEEKLY | Hebdo | Stats Python |
| GovNeo4jSyncWorker | GOV_POSITION_ADDED | Continu | Neo4j |
| GovSentimentAnalyzer | GOV_PRESS_ADDED | Quotidien | LLM local |
| GovAlertWorker | GOV_CONTRADICTION_FOUND | Continu | Push |

---

## Ce que NEXUS GOV a que PoliGraph n'a PAS

| Capacite | PoliGraph | NEXUS GOV |
|----------|-----------|-----------|
| LLM local uncensored | Claude Haiku (cloud, censure) | Gemma heretic (local, 0 censure) |
| Transcription video | Non | faster-whisper (local) |
| Analyse d'image/video | Non | CLIP + DINOv2 (local) |
| Social media monitoring | Non | Twitter + FB + Insta + TikTok + YouTube |
| Detection contradictions cross-source | Non | LLM cross-reference tweet↔vote↔interview |
| Investigation autonome 24/7 | Cron GitHub Actions | EventBus reactif local |
| Dark web / Tor | Non | Robin |
| 100% offline | Non (Vercel, Supabase, cloud APIs) | Tout local |
| Graph WebGL interactif | D3 basique SVG | Reagraph WebGL (clustering, pathfinding) |
| Monitoring web continu | RSS horaire (8 sources) | SearXNG (tout le web) |
| Hypotheses automatiques | Non | LLM multi-pass |
| Recherche dans les videos | Non | Semantic search sur transcriptions |
| Timeline cross-source | Non | Tout canal, tout politicien |
| Comparateur cross-source | Votes seulement | Votes + tweets + interviews + patrimoine |

---

## Stack technique finale

| Composant | Technologie | Deja dans NEXUS ? |
|-----------|-------------|-------------------|
| Backend | FastAPI + Python 3.13 | Oui |
| Frontend | React 19 + Vite + Tailwind 4 | Oui |
| UI | shadcn/ui | Oui |
| Graph | Reagraph WebGL | Oui |
| Charts | Recharts | Oui |
| Carte | Leaflet | Oui |
| LLM | Ollama + Gemma heretic | Oui |
| Transcription | faster-whisper | Oui |
| Vision | CLIP + DINOv2 | Oui |
| NER | GLiNER | Oui |
| Entity resolution | RapidFuzz (Jaro-Winkler) | Oui |
| Embeddings | nomic-embed-text | Oui |
| Vectors | ChromaDB (→ pgvector later) | Oui |
| Graph DB | Neo4j | Oui |
| DB | SQLite WAL (→ PostgreSQL later) | Oui |
| Search | FTS5 + SearXNG | Oui |
| Video download | yt-dlp | A ajouter (pip install) |
| Social scraping | snscrape / gallery-dl | A ajouter |
| RSS | feedparser | A ajouter (pip install) |
| Fact-check API | Google Fact Check | A ajouter (cle gratuite) |
| Dark web | Robin / Tor | Oui |
