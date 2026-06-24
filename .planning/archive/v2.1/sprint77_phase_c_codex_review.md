Audit fait sur le code `staged` via `git diff --cached`. Test ciblé exécuté : `cargo test -p nexus-core-rs shard_plan --locked` => 24 tests passés.

### Livrable 1 : `shard_plan.rs` primitives wire
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/shard_plan.rs:143-178`, `188-225`, `234-319`, `373-489`
- Evidence :
```rust
143:pub struct ShardAssignment {
147:    pub worker_pubkey: [u8; PUBLIC_KEY_LENGTH],
150:    pub layer_start: u32,
156:    pub layer_end: u32,
164:    pub shard_hashes: Vec<[u8; 32]>,
173:    #[serde(default)]
```
`ShardPlan::is_pipeline_contiguous()` détecte bloc vide/inversé, gap et overlap aux lignes `211-225`. `RunMetrics` est tout entier (`u64`/`u32`) aux lignes `379-398`. `RunProof::new()` force `activation_fingerprint: [0u8; 32]` aux lignes `460-467`.

### Livrable 2 : domaines canonicals additifs
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/canonical.rs:257-286`, `307-314`
- Evidence :
```rust
273:pub const DOMAIN_SHARD_PLAN_V1: &[u8] = b"nexus-shard-plan-v1";
286:pub const DOMAIN_RUN_PROOF_V1: &[u8] = b"nexus-run-proof-v1";
311:    out.extend_from_slice(domain);
312:    out.push(0);
313:    out.extend_from_slice(&body);
```
Le diff staged montre uniquement une insertion après `DOMAIN_COMPUTE_GROUP_V1`; aucun domaine existant n’est réécrit.

### Livrable 3 : `lib.rs` module + re-exports
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/lib.rs:62`, `77-83`, `169-173`
- Evidence :
```rust
62:pub mod shard_plan;
82:    DOMAIN_RESULT_V1, DOMAIN_RUN_PROOF_V1, DOMAIN_SEED_REQUEST_V1, DOMAIN_SEED_RESPONSE_V1,
83:    DOMAIN_SHARD_PLAN_V1, DOMAIN_TASK_V1, DOMAIN_WARRANT_CANARY_V1, canonical_bytes,
169:pub use shard_plan::{
170:    KvCachePolicy, RUN_PROOF_FORMAT_VERSION, RUN_PROOF_MAX_PARTICIPANTS, RunMetrics, RunProof,
```

### Livrable 4 : miroir crypto de `compute_group`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/compute_group.rs:189-197`, `220-234`; `crates/nexus-core-rs/src/shard_plan.rs:330-369`, `495-527`
- Evidence :
```rust
331:        if manifest.initiator != keypair.public_bytes() {
336:        check_manifest_caps(&manifest)?;
337:        let bytes = canonical_bytes(&manifest, DOMAIN_SHARD_PLAN_V1)?;
338:        let signature = keypair.sign(&bytes);
```
`verify_signature()` suit aussi l’ordre demandé : version puis caps avant hash puis attribution puis `crypto::verify` (`shard_plan.rs:355-369`, `513-527`). Les appels `canonical_bytes` ne ciblent que `manifest`, `self.manifest`, `proof`, `self.proof` (`337`, `368`, `502`, `526`), jamais les enveloppes `*Entry`.

### Livrable 5 : caps DoS nommés au sign et verify
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/shard_plan.rs:87-107`, `336-337`, `362-368`, `501-502`, `520-526`, `534-585`
- Evidence :
```rust
534:fn check_manifest_caps(manifest: &ShardedSessionManifest) -> Result<()> {
535:    if manifest.plan.assignments.len() > SHARD_PLAN_MAX_ASSIGNMENTS {
542:    if manifest.session_id.len() > SESSION_ID_MAX {
549:    if manifest.group_id.len() > SHARD_GROUP_ID_MAX {
557:        if a.shard_hashes.len() > SHARD_HASHES_MAX {
```
Les caps sont appelés avant `canonical_bytes` au sign et au verify pour manifest et run proof.

### Livrable 6 : versions wire net-new sans bump existant
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/shard_plan.rs:70-80`
- Evidence :
```rust
76:pub const SHARD_PLAN_FORMAT_VERSION: u16 = 1;
80:pub const RUN_PROOF_FORMAT_VERSION: u16 = 1;
```
Le diff staged ne modifie aucun `*_FORMAT_VERSION` existant.

### Livrable 7 : 24 tests utiles dans `shard_plan.rs`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/shard_plan.rs:641-1086`
- Evidence :
```rust
641:    #[test]
642:    fn shard_plan_signature_roundtrip() {
647:        let entry = ShardedSessionManifestEntry::sign(manifest, &initiator).unwrap();
649:            .verify_signature()
651:        assert_eq!(entry.initiator, initiator.public_bytes());
```
Couverture vérifiée : roundtrips `642-653`, `677-689`; tamper payload/signature `693-714`, `1049-1065`; attribution/wrong signer `718-736`, `829-834`, `1039-1045`; caps sign+verify `741-825`, `953-1035`; version mismatch `926-949`; domain separation/cross-domain deux directions `838-869`, `1069-1085`; contiguity et serde `657-673`, `874-917`. `Select-String '#[test]'` retourne 24, et le test ciblé Rust confirme `24 passed`.

### Points critiques
- A : CONFIRME. `canonical_bytes` préfixe `<domain> + 0x00 + body` (`canonical.rs:307-314`) et les tests cross-domain couvrent les deux directions (`shard_plan.rs:853-869`, `1069-1085`).
- B : CONFIRME. Caps avant hash aux deux verify (`shard_plan.rs:356-368`, `514-526`).
- C : CONFIRME. Signature sur payload seul, jamais l’enveloppe (`337`, `368`, `502`, `526`).
- D : CONFIRME. `rg f32|f64` ne trouve que le doc-comment `shard_plan.rs:49`; champs signés entiers `379-398`.
- E : CONFIRME. `#[serde(default)]` seulement sur `fallback_node` et `activation_fingerprint` (`173`, `438`), pas sur version/identité (`235-243`, `409-415`).
- F : CONFIRME. N0 est documenté comme auto-attestation, non preuve de correction (`34-44`, `431-438`) et forcé à zéro par `RunProof::new()` (`460-467`).

## Resume final
- Total livrables : 7
- Confirmes : 7
- Gaps : 0
- Partiels : 0