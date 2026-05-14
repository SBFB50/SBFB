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

4 criteres mesurables et binaires. Gate Phase C appropriee.
Ambiguite mineure : "anti-spam hot path" pas defini en termes
de deliverables exacts (FeedEntry field? test coverage?).

**Recommandation** : clarifier le critere dans verification.md.

---

## Verdict: CONCERN

2 ⚠️ sur 5, 0 ❌. Les D-decisions sont globalement solides mais
3 incoherences factuelles doivent etre resolues avant Phase A code :

1. Gate Phase C : le critere "anti-spam hot path" est implemente
   en Phase D mais evalue en Phase C — contradiction logique.
   Le gate devrait porter sur 3 criteres sync (offline catch-up,
   replay idempotent, 2+ noeuds). Anti-spam = acceptance Phase D.
2. Contrat PoW : kickoff dit "chaque operation porte une preuve PoW"
   (mandatoire) et "pow_proof optionnel" (serde default). Figer :
   optionnel wire format (compat), enforce sur remote sync.
3. G2 iroh-docs : kickoff evalue "iroh > 0.98" mais pas "iroh-docs
   > 0.98". crates.io montre iroh-docs 0.99.0 (2026-05-08). Meme
   conclusion (defere, depend iroh 1.0.0-rc.0) mais a documenter.

CONCERN levee quand kickoff + plan corrigent ces 3 points.
