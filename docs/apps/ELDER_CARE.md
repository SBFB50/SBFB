# SBFB — Elder care dashboard familial sans cloud

**Date** : 2026-04-13
**Statut** : design, pre-implementation
**Effort** : ~500 LOC, ~2h

---

## Le concept

La fille a Lyon et le fils a Toronto voient le meme dashboard
temps reel de l'activite de leur parent age. Capteurs locaux
(Raspberry Pi + optionnel camera). Un LLM local detecte les
anomalies (pas de mouvement depuis 4h). Aucune image ne quitte
jamais la maison. Zero abonnement, zero cloud, zero GAFAM.

---

## Le pitch r/homeassistant + r/privacy

"J'ai fait un systeme de suivi pour mes parents ages. Dashboard
partage en temps reel avec ma soeur a 500km. Tout reste local —
pas de Ring, pas d'Alexa, pas de cloud. Le LLM local detecte
les anomalies. Gratuit."

Cible : r/homeassistant (500K+), r/privacy, forums caregivers.

---

## Architecture

```
Dashboard familial (zip dans iframe)
  │
  bridge postMessage
  │
  ├── iroh-docs CRDT : etat du parent
  │   - dernier_mouvement → 2026-04-13T14:32:00
  │   - temperature_maison → 21.5
  │   - porte_entree → fermee
  │   - medicament_matin → pris (08:15)
  │   - medicament_soir → en attente
  │   - alerte_active → aucune
  │
  └── Task pipeline (optionnel) : detection anomalies
      - Prompt : "Le dernier mouvement date de 4h. La
        temperature est stable. Est-ce normal ?"
      - LLM local repond avec un score de risque
      - Notification push si score > seuil
```

---

## Differenciateur

| Feature | Life Alert | Ring | Home Assistant | SBFB Elder |
|---------|-----------|------|---------------|------------|
| Cout/mois | $30-50 | $10 | Gratuit (local) | **Gratuit** |
| Cloud | Oui | Oui (Amazon) | Optionnel | **Non** |
| Donnees chez | Life Alert | Amazon | Local | **Local** |
| Partage familial P2P | Non | Non | Nabu Casa $65/an | **Natif, gratuit** |
| Detection IA | Non | Non | Non | **LLM local** |
| Fonctionne offline | Non | Non | Oui (local) | **Oui + sync P2P** |

---

## Note honnete

La "detection de chute par camera" mentionnee dans le doc vision
necessite un modele de vision (YOLO, MediaPipe), pas un LLM text.
Ce n'est pas dans le scope initial. Le MVP se concentre sur :
- Capteurs de mouvement (PIR) → dernier mouvement
- Capteurs de porte → ouverture/fermeture
- Rappels de medicaments → bouton "pris"
- LLM text pour l'analyse des patterns temporels (pas de la vision)

La detection de chute par camera serait un Sprint ulterieur avec
integration d'un modele vision (non Ollama).
