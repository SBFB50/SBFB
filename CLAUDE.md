# NEXUS — Cold Case Investigation System

## Projet
Systeme d'investigation persistant pour cold cases. Pas un chatbot one-shot — un outil qui accumule de l'intelligence sur des semaines/mois.

## Architecture
- **Backend**: FastAPI (port 8000)
- **Dashboard**: Streamlit (port 8501)
- **LLMs**: Ollama (port 11434)
  - `nexus` (Gemma 4 26B Heretic uncensored) — analyse profonde, hypotheses, rapports
  - `huihui_ai/deepseek-r1-abliterated:14b` — raisonnement chain-of-thought, verification logique
  - `gemma4:e4b` — extraction rapide d'entites, filtrage, reformulation
  - `nomic-embed-text` — embeddings vectoriels
  - `voxtral-mini:4b` — transcription audio/video
- **Search**: SearXNG clearweb (port 8888) + Robin dark web/Tor (port 9090)
- **Storage**: SQLite (donnees) + Neo4j (graphe, port 7474) + ChromaDB (vecteurs, port 8100)
- **Vision**: CompreFace (reconnaissance faciale, port 8000)

## Stack
- Windows 11, RTX 5080 16GB VRAM
- Python 3.13 + conda
- Docker Desktop (SearXNG, Neo4j, ChromaDB, CompreFace, Robin)
- Ollama

## Fichiers importants
- `docs/DEVELOPMENT-PLAN.md` — Plan de dev 6 phases (~207h)
- `docs/OSINT-with-LLM-research.md` — Recherche integration OSINT-with-LLM
- `docs/LOCAL-DEEP-RESEARCH-ANALYSIS.md` — Recherche integration Local Deep Research
- `prompts/` — 5 templates de prompts pour l'analyse
- `Modelfile.gemma4-heretic` — Definition du modele nexus actif

## Strategie multi-modele
Ne PAS utiliser un seul LLM pour tout. Router les taches :
- Taches mecaniques (extraction, filtrage, JSON) → gemma4:e4b (rapide, 80 t/s)
- Raisonnement logique (contradictions, verification) → deepseek-r1:14b (CoT)
- Analyse profonde (hypotheses, scoring, rapports) → nexus:26b (le cerveau)
- Audio/video → voxtral:4b
- Embeddings → nomic-embed-text
- Serialiser les appels GPU (16GB VRAM partagee) — pas de modeles en parallele

## Principes
- Systeme PERSISTANT — chaque donnee est stockee, rien n'est perdu
- INCREMENTAL — nouvelles donnees re-evaluent les hypotheses existantes
- MONITORING CONTINU — recherches automatiques toutes les 6h (clearweb) et 24h (dark web)
- IMPARTIAL — pas de biais, pensee adversariale sur chaque hypothese
- UNCENSORED — les modeles heretic/abliterated ne refusent pas d'analyser du contenu sensible
- Multi-sources — clearweb + dark web + recon (WHOIS/Shodan/VT) + audio + visages

## Langue
L'utilisateur est francophone. Repondre en francais. Les prompts systeme des LLMs sont en francais.
Le code et les commentaires peuvent etre en anglais.

## Ce qui est deja operationnel
- Ollama + nexus (Gemma 4 26B Heretic) ✅
- Open WebUI (port 3000) ✅
- SearXNG (port 8888) ✅
- Docker Desktop ✅
- 5 prompts d'analyse ✅
