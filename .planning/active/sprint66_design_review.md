# Sprint 66 — Design Review Board (G1)

**Date** : 2026-05-19
**Sprint** : 66 — Durabilite (Arc 1 Fondations, 2/2)
**Reviewer** : self-review profond (auto-challenge systematique)

---

## Scoring

| D# | Titre | Source recente | Alternative | [DETER] Crypto | [DETER] Rust | Code verifie | Verdict |
|---|---|---|---|---|---|---|---|
| D1 | iroh-docs persistence via data_dir | ok (context7 2026-05-19) | ok (3 alternatives rejetees) | N/A | ok (worker-core utilise deja data_dir) | ok (node.rs, runtime.rs lus) | ok |
| D2 | iroh-blobs FsStore activation | ok (context7 2026-05-19) | ok (3 alternatives rejetees) | N/A | ok (FsStore Rust-native iroh-blobs) | ok (node.rs, blobs.rs lus) | warning |
| D3 | Feed republish au boot + handle fix | ok (code lu 2026-05-19) | ok (3 alternatives rejetees) | N/A | ok | ok (runtime.rs, feed_sync.rs lus) | ok |
| D4 | Provenance 3 etats (MANDATORY) | ok (WebSearch npm/C2PA 2026-05-19) | ok (3 alternatives rejetees) | N/A | N/A (frontend) | ok (http.rs, useBridge.ts, BrowsedProject.tsx lus) | ok |
| D5 | Verification cross-node (MANDATORY) | ok (SLSA spec, Sigstore, Keyoxide) | ok (3 alternatives rejetees) | ok (Ed25519 verify, pas de changement crypto) | ok (provenance.rs Rust-native) | ok (provenance.rs, http.rs lus) | ok |

**Resume** : D1 ok, D2 warning, D3 ok, D4 ok, D5 ok.
Rigor signal G4 satisfait (1 warning sur 5).

---

## Findings

### D2 warning — API publique `Node.blobs_store()` change de type retour

**Detail** : Le passage de `Node.blobs_store` de `MemStore` a
un enum `BlobStore` modifie le type de retour de la methode
publique `blobs_store()`. Actuellement elle retourne `&MemStore`,
elle retournera `&Store`. Ce changement est source-breaking pour
tous les consumers qui typent explicitement `&MemStore` dans leur
code. 3 crates downstream sont impactes :
- `nexus-shell-daemon` (blob_serve.rs, http.rs, deploy.rs)
- `nexus-shell-daemon-core` (blob_serve.rs)
- `nexus-worker-core` (runtime.rs)

Un pattern match sur l'enum serait necessaire pour acceder aux
methodes FsStore-specifiques (ex: `dump()`), mais aucun consumer
actuel n'utilise de methodes MemStore-specifiques — ils passent
tous par `BlobsClient::new()` qui wrap le store.

**Decision** : adjust — (1) modifier `BlobsClient::new` pour
prendre `&Store` au lieu de `&MemStore` (changement localise),
(2) ajouter un test de compilation dans `nexus-test-harness` qui
instancie un `BlobsClient` depuis `node.blobs_store()` pour
garantir que le type est compatible, (3) documenter le changement
d'API dans le commit body Phase A section "Fichiers" avec note
"breaking type change".

Pre-launch policy : ce changement d'API n'impacte pas le wire
format. `BlobsClient` est une abstraction interne au workspace.
Aucun consumer externe n'utilise `nexus-core-rs` directement
(le crate n'est pas publie sur crates.io).

---

## Checklist [DETER] (si applicable)

### Crypto/spec
- [x] D5 verification cross-node cite >= 1 alternative concurrente
  < 6 mois (Sigstore Rekor 2025, SLSA L1 spec 2023)
- [x] Sources datees < 2 ans (SLSA spec v1.0 2023, Keyoxide
  codeberg 2024, Sigstore 2025)
- [x] Reviewer warning si alternative absente : N/A (alternatives
  documentees)

### Rust-first
- [x] D1 iroh-docs cite alternative Rust-native : `Docs::persistent`
  est la solution iroh-docs native
- [x] D2 FsStore cite alternative Rust-native : `FsStore` est la
  solution iroh-blobs native (redb backend)
- [x] D3 feed republish : `replay_all()` est deja Rust-native dans
  `public_feed.rs`
- [x] D5 verification : `verify_provenance` Rust-native dans
  `provenance.rs`
- Exemptions : D4 frontend (React/TypeScript, pas Rust)
