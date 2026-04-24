# SBFB — Dashboard capteurs citoyens distribue

**Date** : 2026-04-13
**Statut** : design, pre-implementation
**Effort** : ~600 LOC, ~2.5h

---

## Le concept

Des Raspberry Pi mesurent la qualite de l'air dans un quartier.
Chaque Pi est un noeud SBFB qui ecrit ses mesures dans un
iroh-doc. N'importe qui deploie une app dashboard (zip) qui
s'abonne aux donnees en temps reel. Un noeud avec GPU fait de la
prediction/alerte via LLM. Le reseau appartient au quartier.

---

## Le pitch r/homeautomation

"J'ai fait un reseau de capteurs de quartier P2P. Chaque
Raspberry Pi publie ses mesures sur le reseau distribue. Le
dashboard se met a jour en temps reel. Pas d'AWS IoT, pas
d'abonnement, pas de single point of failure."

Cible : r/homeautomation, r/raspberry_pi, Sensor.Community.

---

## Architecture

```
Dashboard (zip dans iframe)
  │
  bridge postMessage
  │
  └── iroh-docs CRDT : subscribe aux mesures
      - capteur_alice/temperature → 22.3
      - capteur_alice/humidite → 67
      - capteur_alice/pm25 → 12.4
      - capteur_bob/temperature → 21.8
      - capteur_bob/pm25 → 8.7
      - Timestamps sur chaque mesure
      - Subscribe en temps reel → graphiques live
```

---

## Donnees de marche

- PurpleAir : 25K+ capteurs actifs, $250 par capteur
- Sensor.Community : 15K+ noeuds Europe
- AWS IoT : $0.08/message — un capteur qui envoie toutes les
  minutes coute $3.50/mois
- SBFB : $0
