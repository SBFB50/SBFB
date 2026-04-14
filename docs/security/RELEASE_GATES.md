# Release Gates — stub (scope-cut Phase E Sprint 17)

**Statut** : document stub. Le contenu formel (RELEASE_GATES.md
complet ~400 LOC) etait prevu en Sprint 17 Phase E mais le
scope-cut acte au wrap-up (`60b539a`) a officialise le report a
un **sprint OpSec dedie futur** (quand fondation multi-juridiction
en place, cf [`VALIDATED_BLUEPRINT.md` Couche 10](VALIDATED_BLUEPRINT.md#couche-10--operational-security)).

Ce stub existe uniquement pour resoudre les cross-references
laissees dans les docs livres Sprint 17 Phase A-D.

---

## Source canonique actuelle — Gates 1-4 mapping

Le mapping Gate → Sprint → Tier adresse est entierement documente
dans [`HARDENING_ROADMAP.md §7`](HARDENING_ROADMAP.md#7-gates-debloquage-sequencing) :

| Gate | Tier max | Sprint debloquant | Exemple app |
|---|---|---|---|
| **Gate 1** | T0-T1 | S18 (quick-wins + audit S16 leve) | DnD Forge, hello-world |
| **Gate 2** | T0-T2 | S22 (supply chain + encryption + rate-limit + Sybil base) | TransLingua, FamilyScan |
| **Gate 3** | T0-T3 | **S29** (tech S27 + audit externe Cure53/ToB S29) | PolitiScan, NEXUS cold-case |
| **Gate 4** | T0-T5 | **~S35-38** (tech S30 + partnership + beta 18 mois + ethics board) | LibanLive, war-crime doc |

**Ship-blocker ethique** : aucune app classee pour population
cible T5 (LibanLive-class) ne peut sortir en beta ouverte avant
Gate 4 effectif complet. Clause structurelle — le code sera
techniquement capable de ship, le release **n'est pas autorise**
par policy. Cf [`ADVERSARIES.md §3.1`](ADVERSARIES.md#31-pourquoi-t5-non-atteignable-avant-gate-4-complet).

---

## Items differes a sprint OpSec dedie futur

Le RELEASE_GATES.md complet (~400 LOC prevus) devait livrer :

1. **Enforcement mechanism formel app-by-app** :
   `ProjectAnnouncement.gate_tier` (v? TBD Sprint 18+) + checkbox
   pre-requis par gate verifiable automatiquement par coordinator.
   Actuellement TBD, tracee dans
   [`HARDENING_ROADMAP.md §7 line 491-492`](HARDENING_ROADMAP.md#7-gates-debloquage-sequencing).

2. **Checklist concrete par gate** : liste auditable d'items
   (crypto, supply chain, audit, partnership, beta duration) que
   l'app doit cocher avant release. Actuellement informelle dans
   `HARDENING_ROADMAP.md §7` prerequis column.

3. **Path d'escalade** : app qui monte de gate (DnD Forge Gate 1
   → Gate 2 si devient hub social). Governance TBD.

4. **Revocation policy** : app depublie apres incident security
   majeur. Governance TBD.

---

## Mapping tier → gate

Source : [`ADVERSARIES.md §3`](ADVERSARIES.md#3-mapping-tier--app-risk-gate).

---

**Quand ce stub sera-t-il remplace ?** Lors du sprint OpSec dedie
qui etablira :
- Fondation legale (association loi 1901 / Stichting / 501c3)
- Board multi-jurisdictionnel signe
- Budget partnership Amnesty/HRW/CPJ/EFF initial

Ce stub restera en place jusque-la pour preserver les liens.
