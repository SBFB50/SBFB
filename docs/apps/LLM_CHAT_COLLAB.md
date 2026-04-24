# SBFB — LLM Chat collaboratif P2P

**Date** : 2026-04-13
**Statut** : design, pre-implementation
**Effort** : ~800 LOC, ~3h

---

## Le concept

Un ChatGPT P2P. Tes GPU, tes donnees, tes potes. Zero cloud.

- Tu ouvres le chat dans Browse, tu tapes un prompt
- Le LLM repond via le GPU de n'importe quel noeud du reseau
- La conversation synchronise en CRDT — tes potes voient tes
  echanges en temps reel
- Vous pouvez tous parler au meme LLM, annoter les reponses,
  forker des conversations
- Zero compte OpenAI, zero API key, zero cout par token

---

## Le pitch r/LocalLLaMA

"J'ai fait un ChatGPT P2P. Tes GPU, tes donnees, tes potes.
Zero cloud. Zero abonnement. Zero censure. Le LLM tourne sur
les GPU distribues du reseau."

Cible : r/LocalLLaMA (1.5M membres), Hacker News.

---

## Architecture

```
Chat UI (zip dans iframe)
  │
  bridge postMessage
  │
  ├── iroh-docs CRDT : historique des conversations
  │   - Chaque message = une entree dans le doc
  │   - Chaque user ecrit ses messages (pas de conflit)
  │   - Le result IA est ecrit par le worker qui genere
  │
  └── Task pipeline : generation LLM
      - Submit prompt + conversation context
      - Round-robin : le GPU libre le plus rapide repond
      - Streaming temps reel
```

---

## Features

- **Multi-user** : plusieurs personnes dans la meme conversation
- **Fork** : dupliquer une conversation a partir d'un point
- **Annotations** : reagir / commenter une reponse IA
- **Modele au choix** : l'utilisateur choisit le modele (8B rapide
  ou 70B distribue narratif)
- **Historique persistant** : CRDT survit aux deconnexions
- **Export** : telecharger la conversation en markdown
