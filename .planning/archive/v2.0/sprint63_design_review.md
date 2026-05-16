# Sprint 63 — Design Review Board (G1)

**Date** : 2026-05-15
**Reviewer** : agent Explore independant (session fraiche)
**Sprint** : Sprint 63 (verification tiers + UX)
**Scoring** : D1 ⚠️, D2 ✅, D3 ✅, D4 ⚠️, D5 ✅
**Rigor signal G4** : satisfait (2 ⚠️ sur 5, 0 ❌)

---

## Scoring par decision

### D1 — Provenance endpoint (SQLite M12 + HTTP GET) — ⚠️

Source presente mais partiellement validee. `deploy.rs` genere bien
le ProvenanceRecord (lignes 159-165) et l'insere dans le zip (lignes
195-207). La migration M12 n'existe pas encore (M11 = unique index
feed entry_hash S62 Phase B). La decision repose sur une migration
future. L'absence actuelle de `provenance_records` en DB confirme
le stockage zip-only.

**Angle mort** : la numerotation M12 n'a pas ete validee contre le
code actuel au moment du draft.

### D2 — Bridge verification (3 methodes postMessage) — ✅

Fondation solide. 10 methodes publiques actuelles confirmees dans
sbfb-bridge.js (submitTask, getStorage, setStorage, piiRedact,
listStorage, deleteStorage, getIdentityPubkey, getNodeStatus,
getBrowseList, getStorageVersion + 2 callback patterns). Pattern
postMessage etabli depuis Sprint 13. Les 3 nouvelles methodes
suivent le meme `bridge._call()` interne avec response
`sbfb-bridge-response`.

### D3 — UI proof-chain (modal VerificationDetail) — ✅

Badge ShieldCheck confirme `BrowsedProject.tsx` ligne 271-279,
conditionne par `entry.provenance_hash`. Infrastructure
provenance_hash dans BrowseEntry existante. Clic → modal = pattern
coherent shadcn Dialog.

### D4 — MANDATORY carries (IMAGE-DEP + PLAYWRIGHT-REFACTOR) — ⚠️

IMAGE-DEP : `image 0.25` avec feature "png" confirme dans
Cargo.toml (ligne 21). Le remplacement par crate `png` minimal
est viable mais le poids exact des transitives n'a pas ete compare
factuellement (pas de `cargo tree -d` execute).

PLAYWRIGHT-REFACTOR : la cause du blocage (spawn Python via `uv run`)
est connue (Python supprime S50-S51). Le fix (spawn daemon Rust)
est logique mais non verifiable sans execution.

**Angle mort** : proof que `png` est effectivement plus leger que
`image` avec `default-features=false, features=["png"]` non
demontree.

### D5 — Scope : CuratorVouched/BuildQuorumReached differes — ✅

CuratorVouched et BuildQuorumReached absents de deploy.rs,
contributor_registry.rs, et provenance.rs. Le decoupage Sprint 63
(visibilite) vs Sprint 64 (enrichissement) est logique. Protocol
Explorer absorbe en Phase D si budget = raisonnable.

---

## Checklist crypto/spec [DETER]

- [x] D1 : ProvenanceRecord Ed25519 — alternative concurrente
  non applicable (SLSA L1 propre au projet, pas de lib externe)
- [x] D2 : postMessage bridge — pas de composant crypto
- [x] D3 : UI only — pas de composant crypto
- [x] D4 : PNG decode — pas de composant crypto
- [x] D5 : scope cut — pas de composant crypto

## Checklist Rust-first [DETER]

- [x] D1 : SQLite via rusqlite — Rust natif
- [x] D4 IMAGE-DEP : `png` crate = Rust natif (vs `image` = aussi
  Rust natif, question de footprint pas de runtime)
- [x] D4 PLAYWRIGHT : TypeScript (exemption frontend UX)
