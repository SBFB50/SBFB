# NEXUS — Cold Case Analyst

## Installation

```bash
# Créer le modèle custom dans Ollama
ollama create nexus -f Modelfile

# Lancer une session d'analyse
ollama run nexus
```

## Utilisation

### Analyse d'un cold case
```
Colle tes données (rapports de police, témoignages, données OSINT, etc.)
et NEXUS les analysera automatiquement avec sa méthodologie en 5 phases.
```

### Exemple de prompt utilisateur
```
Voici les données du cold case #2019-4472 :

[RAPPORT DE POLICE]
...

[TÉMOIGNAGES]
...

[DONNÉES TÉLÉPHONIQUES]
...

[TRANSACTIONS FINANCIÈRES]
...

Analyse complète. Identifie les connexions cachées et propose des hypothèses classées par plausibilité.
```

### Analyse continue (réinjection de nouvelles données)
```
MISE À JOUR — Nouvelles données pour le case #2019-4472 :

[NOUVELLES DONNÉES]
...

Réévalue toutes les hypothèses précédentes à la lumière de ces nouvelles informations.
```

## Architecture recommandée avec OSINT

```
SpiderFoot (collecte OSINT automatisée)
       ↓
Données structurées (JSON/CSV)
       ↓
NEXUS (analyse via Ollama API localhost:11434)
       ↓
Neo4j (stockage du graphe relationnel)
       ↓
Dashboard (visualisation)
```

## API Ollama — Intégration programmatique

```python
import requests
import json

def analyze_cold_case(case_data: str) -> str:
    response = requests.post(
        "http://localhost:11434/api/generate",
        json={
            "model": "nexus",
            "prompt": case_data,
            "stream": False
        }
    )
    return response.json()["response"]

# Exemple
result = analyze_cold_case("""
Cold Case #2019-4472
Victime: Jean Dupont, 45 ans
Dernière localisation connue: 48.8566°N, 2.3522°E
Date disparition: 2019-03-15
...
""")
print(result)
```
