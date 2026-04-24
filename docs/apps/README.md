# SBFB — Apps planifiees

Design docs pour les applications a construire sur la plateforme
SBFB. Chaque app utilise les 3 briques du stack (P2P distribution +
CRDT sync + GPU compute distribue).

## Prerequis commun

Toutes les apps necessitent le **bridge postMessage** (Sprint 13)
pour communiquer entre l'iframe et le shell (iroh-docs + task
pipeline).

## Apps

| # | App | Fichier | LOC | Temps | Priorite |
|---|-----|---------|-----|-------|----------|
| 1 | Chat IA avec acces a tous les projets du reseau | [CHAT_IA_RESEAU.md](CHAT_IA_RESEAU.md) | ~1500 | ~11h | Strategique (flywheel) |
| 2 | D&D P2P avec DM IA 70B distribue streaming | [DND_P2P.md](DND_P2P.md) | ~2200 | ~8h | Virale (50M joueurs D&D) |
| 3 | LLM Chat collaboratif P2P | [LLM_CHAT_COLLAB.md](LLM_CHAT_COLLAB.md) | ~800 | ~3h | Validation bridge |
| 4 | Render farm Blender distribuee | [RENDER_FARM.md](RENDER_FARM.md) | ~1000 | ~4h | Communaute 2M+ |
| 5 | Dashboard capteurs citoyens | [CAPTEURS_IOT.md](CAPTEURS_IOT.md) | ~600 | ~2.5h | Niche IoT |
| 6 | Elder care dashboard familial | [ELDER_CARE.md](ELDER_CARE.md) | ~500 | ~2h | Emotionnel / r/privacy |
| 7 | Generation composee (apps construites sur les meilleures du reseau) | [GENERATION_COMPOSEE.md](GENERATION_COMPOSEE.md) | ~1000 | ~6.5h | Flywheel evolution |
| 8 | Lien EHPAD-Famille (visio, jeux, compagnon IA, livre de vie) | [EHPAD_LIEN_FAMILLE.md](EHPAD_LIEN_FAMILLE.md) | ~2650 | ~18.5h | Impact humain / presse |
| 9 | Plateforme de crise catastrophes (10 apps, mesh LoRa, triage IA) | [CATASTROPHE_HUMANITAIRE.md](CATASTROPHE_HUMANITAIRE.md) | ~4650 | ~30h | Sauver des vies / ONG / EU grants |

**Total : ~14900 LOC, ~85.5h, ~15 jours**

## Ordre d'implementation recommande

```
Jour 1 : Sprint 13 — bridge postMessage + launcher (12h)
Jour 2 : LLM Chat collab (3h) + Chat IA reseau (6h)
         → Valide le bridge, premiere app postable
Jour 3 : D&D P2P (8h)
         → La plus virale, la plus complexe
Jour 4 : Render farm (4h) + Capteurs IoT (2.5h)
Jour 5 : Elder care (2h) + polish + deploy
Jour 6 : Posts Reddit/HN + monitoring reactions
```

## Vision doc

Le document de vision global avec les 15 use cases et les donnees
de marche est dans [../VISION_USE_CASES.md](../VISION_USE_CASES.md).
