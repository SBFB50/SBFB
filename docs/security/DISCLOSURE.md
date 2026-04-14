# Responsible disclosure — stub (scope-cut Phase E Sprint 17)

**Statut** : document stub. Le contenu formel (DISCLOSURE.md
complet ~150 LOC) etait prevu en Sprint 17 Phase E mais le
scope-cut acte au wrap-up (`60b539a`) a officialise le report a
un **sprint OpSec dedie futur**.

Ce stub existe uniquement pour resoudre les cross-references
laissees dans les docs livres Sprint 17 Phase A-D.

---

## Pattern cible (source canonique)

Le pattern disclosure est documente dans
[`VALIDATED_BLUEPRINT.md` Couche 10 Operational security](VALIDATED_BLUEPRINT.md#couche-10--operational-security) :

- `.well-known/security.txt` a la racine du site projet
- PGP key publique pour chiffrer les reports
- **90 days embargo** standard (industry norm Google Project Zero)
- CVE assignment workflow via **GitHub Security Advisories**
- Hall of Fame dans SECURITY.md racine

---

## Items differes a sprint OpSec dedie futur

Le DISCLOSURE.md complet devait livrer :

1. **`.well-known/security.txt` concret** a la racine du repo —
   quick-win Sprint 19+ (no-blocker S18, cf audit finding **F-1**
   dans `sprint17_audit_findings.md`).

2. **SLA disclosure formel** : response sous 72h, fix sous 30
   jours pour P0/P1, publication publique post-embargo.

3. **CVE coordination workflow** documente pas-a-pas (qui assigne,
   qui valide, qui publie).

4. **Hall of Fame** : SECURITY.md racine + credits pour chercheurs
   qui ont signale responsablement.

5. **Bug bounty informel** via HackerOne Community Edition —
   triage-as-a-service gratuit OSS (cf
   [`PARTNERSHIPS.md`](PARTNERSHIPS.md)).

---

## Action immediate — quick-win Sprint 19+

Le finding **F-1** de l'audit Sprint 17 recommande de livrer au
minimum :

- `.well-known/security.txt` stub dans le repo
- `SECURITY.md` racine avec PGP key + contact email + SLA minimal

Estimation : ~50 LOC docs, 1 quick-win Sprint 19 ou plus tot si
un incident externe force la main.

---

**Quand ce stub sera-t-il remplace ?** Lors du sprint OpSec dedie
qui livrera le workflow complet + bug bounty actif. Ce stub
restera en place jusque-la pour preserver les liens.
