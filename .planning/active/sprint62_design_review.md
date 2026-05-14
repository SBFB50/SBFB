# Sprint 62 — Design Review Board (G1)

**Date** : 2026-05-14
**Reviewer** : agent Explore independant (session fraiche)
**Sprint** : 62 — sync P2P durable + anti-spam minimal
**Methode** : scoring tiers par D-decision (✅/⚠️/❌)

---

## Scoring global

| Decision | Tier | Risque majeur |
|---|---|---|
| D1 iroh-docs namespace | ⚠️ | Pas de changelog 0.98 valide formellement |
| D2 Hash-chain per-auteur | ✅ | Causal ordering (R1) — documente et mitige |
| D3 Anti-spam 4-gates | ⚠️ | Hashcash 2026 relevance + taux 5 ops/min non calibre |
| D4 Phase dette A | ✅ | Perf materialize_incremental O(N²) worst-case acceptable pilot |
| D5 Gate de scission | ✅ | Critere "anti-spam hot path" vague |

**Rigor signal G4** : 2 ⚠️ sur 5, 0 ❌. Satisfait.

---

## D1 ⚠️ — iroh-docs 0.98 changelog

iroh-docs 0.98 n'a pas de changelog publie accessible trouve par
WebSearch. Les docs.rs generales confirment l'architecture (namespace,
CRDT, range-based reconciliation, LiveEvent), mais la validation
specifique 0.98 vs 0.97 manque.

Le codebase AppStorage S58 utilise deja l'exact meme API
(`import_ticket()`, `subscribe()`, `LiveEvent::InsertRemote`) contre
iroh-docs 0.98 en production. Risque reduit par le precedent code.

**Recommandation** : valider API shape exacte avant Phase B kickoff
via le code existant (`storage_api.rs`).

---

## D2 ✅ — Hash-chain per-auteur

Sources completes. Pattern kudos_ledger + public_feed S61 valides.
Alternatives correctement rejetees (chaine globale impossible
concurrent, DAG surconception, relais central incompatible P2P).
Causal ordering documente en R1 avec mitigation timestamp.

---

## D3 ⚠️ — Anti-spam 4-gates

### Hashcash 2026

Hashcash (1997 Back) est une primitive CPU-bound simple. Equihash
(memoire-hard, verification rapide, utilise par Zcash/Komodo) est
une alternative non evaluee dans D3. Hashcash fonctionne, mais
l'absence d'evaluation comparative est un gap documental.

### Taux 5 ops/min

Non calibre sur un modele d'attaque ou un benchmark feed P2P. Pas
de comparaison avec SSB (local-first, pas de rate-limit reseau),
Nostr (client-to-relay), AT Protocol (serveurs tiers). Aucun de
ces projets ne fait du feed P2P peer-to-peer pur.

### Pollution namespace iroh-docs

Un attaquant pourrait PoW-spammer le namespace iroh-docs avant
que les rate-limit/quarantine gates admissions locales ne le
bloquent. Les entrees invalides existent dans iroh-docs mais ne
sont jamais materialisees localement. Cout de reconciliation =
risque residuel.

---

## D4 ✅ — Phase dette A

4 P2 S61 correctement identifies comme prerequis sync.
P2-NSIS-UNINSTALL justifie en sprint pair. F2 materialize_incremental
pourrait etre O(N²) worst-case mais acceptable pour pilot.

---

## D5 ✅ — Gate de scission

3 criteres sync mesurables et binaires (offline catch-up, replay
idempotent, 2+ noeuds). Gate Phase C appropriee. Anti-spam retire
du gate (implemente Phase D, evalue separement).

---

## Verdict: PASS

2 ⚠️ sur 5, 0 ❌. Les D-decisions sont solides.

CONCERN initiale sur 3 incoherences — toutes resolues dans
`61b41fc` (fix(planning)) :

1. Gate Phase C : anti-spam retire du gate (Phase D acceptance).
   Gate = 3 criteres sync. **Corrige.**
2. Contrat PoW : fige optionnel wire format + enforce remote sync.
   **Corrige.**
3. G2 iroh-docs 0.99.0 : documente dans kickoff, meme conclusion
   (defere, depend iroh 1.0.0-rc.0). **Corrige.**
