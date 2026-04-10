# Politique de securite et de donnees

## Donnees collectees

NEXUS GOV ne collecte QUE des donnees publiques :
- Votes parlementaires (data.assemblee-nationale.fr, data.senat.fr)
- Declarations de patrimoine (hatvp.fr, open data)
- Posts reseaux sociaux publics (Twitter, Facebook, Instagram, TikTok, YouTube)
- Articles de presse publics (RSS, SearXNG)
- Dossiers legislatifs (data.gouv.fr)
- Donnees Wikidata (open data)

## Ce que NEXUS GOV ne fait PAS

- Aucune collecte de donnees privees
- Aucune surveillance de citoyens
- Aucun acces a des comptes prives
- Aucun stockage de donnees biometriques
- Aucune utilisation de cloud non-europeen pour le traitement

## Presomption d'innocence

Les contradictions detectees par NEXUS sont factuelles :
"Le politicien X a dit Y le [date] mais a vote Z le [date]."

Ce ne sont PAS des accusations. La presomption d'innocence s'applique
a toutes les informations presentees par le systeme.

## Traitement local

Toutes les analyses sont effectuees localement :
- LLM : Ollama (modele local, zero envoi de donnees)
- Transcription : faster-whisper (local)
- OCR : PaddleOCR (local)
- Embeddings : nomic-embed-text (local)
- Base de donnees : SQLite/Neo4j/ChromaDB (local)

Aucune donnee n'est envoyee a un service cloud tiers.

## Signaler une vulnerabilite

Si vous decouvrez une vulnerabilite de securite :
1. NE PAS ouvrir une issue publique
2. Envoyez un email a : security@nexusgov.fr (ou ouvrez un rapport prive sur GitHub)
3. Decrivez la vulnerabilite, les etapes de reproduction, et l'impact potentiel
4. Nous repondrons sous 48h

## Droit applicable

NEXUS GOV est un service d'information en ligne au sens de la loi du 29 juillet 1881
sur la liberte de la presse et de la LCEN du 21 juin 2004.

Les donnees sont traitees conformement au RGPD (Reglement UE 2016/679).
Droit d'acces, de rectification et d'effacement : contact@nexusgov.fr
