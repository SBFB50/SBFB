# SBFB — Render farm Blender distribuee

**Date** : 2026-04-13
**Statut** : design, pre-implementation
**Effort** : ~1000 LOC, ~4h

---

## Le concept

Drop ton projet Blender, les GPU du reseau rendent tes frames.
Gratuit. P2P. Pas de file d'attente. Si un noeud crash, un autre
reprend sans recalculer.

---

## Le pitch r/blender

"J'ai fait une render farm P2P gratuite pour Blender. Chaque
artiste contribue son GPU et utilise celui des autres. L'assignation
des frames est en CRDT — si un noeud tombe, un autre reprend
la frame. Zero serveur, zero cout."

Cible : r/blender (2M+), BlenderArtists forum, SheepIt community.

---

## Architecture

```
App Render Farm (zip dans iframe)
  │
  bridge postMessage
  │
  ├── iroh-docs CRDT : assignation des frames
  │   - frame_047 → noeud_alice → status: rendering
  │   - frame_048 → noeud_bob → status: complete
  │   - frame_049 → non assigne → status: pending
  │
  ├── iroh-blobs : stockage du projet + frames rendues
  │   - Upload du .blend → blob
  │   - Chaque frame rendue → blob individuel
  │
  └── Task pipeline : rendu GPU
      - Submit : render frame N du projet X
      - Worker execute via Blender CLI headless
      - Result : hash du blob de la frame rendue
```

---

## Difference avec SheepIt

| Feature | SheepIt | SBFB Render Farm |
|---------|---------|-----------------|
| Persistance de job | Non — crash = recalcul | CRDT — un autre reprend |
| Cout | Gratuit (point system) | Gratuit (reputation) |
| Serveur central | Oui (sheepit-renderfarm.com) | Non |
| Machines connectees | ~600 en moyenne | Reseau SBFB entier |
| Tracking contributions | Points opaques | Reputation publique verifiable |

---

## Limitation honnete

Le rendu Blender necessite **Blender installe** sur le worker, pas
juste Ollama. Le task pipeline actuel est concu pour l'inference
LLM. L'extension au rendu Blender necessite :
- Un nouveau task_type "render" (a cote de "llm")
- Le worker doit detecter si Blender est installe
- Le fichier .blend doit etre transfere via iroh-blobs (potentiellement gros)
