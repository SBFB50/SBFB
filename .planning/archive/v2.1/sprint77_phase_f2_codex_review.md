Audit basé sur `git status`, `git diff HEAD`, inspection du fichier untracked `shard_claim.rs`, et tests ciblés.

### Livrable 1 : SEAM `ShardForwarder`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/shard.rs:223`, `:261`, `:315`, `:340`; `crates/nexus-core-rs/src/lib.rs:165`
- Evidence :
```rust
315:         while let Some(frame) = read_frame(&mut recv).await.map_err(AcceptError::from_err)? {
316:             let out = self
317:                 .forwarder
318:                 .forward(&frame)
319:                 .map_err(AcceptError::from_err)?;
```
Trait, `EchoForwarder`, stockage `Arc<dyn ShardForwarder>`, factory avec forwarder, re-export lib.rs présents. L’erreur forwarder est mappée en `AcceptError`, pas de panic.

### Livrable 2 : Worker shard claim
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/src/engine/shard_claim.rs:271`, `:316`, `:333`, `:405`; `crates/nexus-worker-core/src/engine/runtime.rs:674`
- Evidence :
```rust
276:     // 1. Signature FIRST — a forged manifest never reaches membership / I/O.
277:     manifest_entry
278:         .verify_signature()
279:         .map_err(|e| ClaimRejection::BadManifestSignature(e.to_string()))?;
```
```rust
674:         let self_pubkey = self.keypair.public_bytes();
676:         let assignment =
677:             crate::engine::shard_claim::authorize_claim(manifest_entry, group, &self_pubkey)?;
680:         let facts = crate::engine::shard_claim::read_gguf_model_facts(model_path)?;
681:         let snapshot = self.gpu_snapshot(gpu_index).map_err(|e| {
```
`authorize_claim` fait signature -> membership -> assignment. `assess_capacity` valide `ShardWindow::new(...)` avant estimation (`shard_claim.rs:316-333`). `runtime.rs` fait authorize -> read_gguf -> snapshot ponctuel -> assess.

Round 2 R1-PARTIEL-A : RESOLU. `read_gguf_model_facts` rejette maintenant `is_degenerate_geometry(...)` en `ModelUnreadable` (`shard_claim.rs:405-409`) et le test hermétique existe (`shard_claim.rs:516-534`).

### Livrable 3 : Accessors GGUF vendores
- Statut : CONFIRME
- Fichier(s) : `vendor/llama-cpp-2/src/gguf/mod.rs:106`, `:124`, `:143`, `:158`; `crates/nexus-worker-core/src/engine/shard_claim.rs:375`, `:418`
- Evidence :
```rust
143:     pub fn meta_str(&self, key: &str) -> Option<String> {
148:         if self.kv_type(idx) != llama_cpp_sys_2::GGUF_TYPE_STRING {
149:             return None;
150:         }
151:         self.val_str(idx).map(str::to_string)
```
```rust
418:         let name = ctx.tensor_name(i).ok_or_else(|| {
419:             ClaimRejection::ModelUnreadable(format!("tensor {i} has an unreadable name"))
420:         })?;
421:         tensor_sizes.push((name.to_string(), ctx.tensor_size(i)));
```
`tensor_name`, `tensor_size`, `tensor_type`, `meta_str`, `meta_u32` sont présents et type-checkés avant `val_*`. `read_gguf_model_facts` lit arch, block_count, embedding_length, head_count, head_count_kv et table des tailles.

Round 2 R1-PARTIEL-B : RESOLU. Un nom de tenseur illisible rejette tout le header au lieu d’être sauté.

### Livrable 4 : `ShardBackendForwarder`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/src/llm/shard.rs:176`, `:471`, `:494`
- Evidence :
```rust
494:     impl nexus_core_rs::ShardForwarder for ShardBackendForwarder {
495:         fn forward(&self, upstream_frame: &[u8]) -> nexus_core_rs::Result<Vec<u8>> {
496:             if upstream_frame.len() % 4 != 0 {
497:                 return Err(nexus_core_rs::NexusError::Other(format!(
```
Le forwarder est feature-gated via `llm_llama_cpp`, convertit little-endian bytes -> `f32`, appelle `forward_hidden`, puis réencode en little-endian bytes (`llm/shard.rs:502-513`).

### Livrable 5 : Cap frame
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/shard.rs:70`, `:85`, `:97`, `:119`
- Evidence :
```rust
 85: pub const MAX_SHARD_FRAME_BYTES: usize = 256 * 1024 * 1024;
 97: pub const MAX_SHARD_N_CTX: u32 = 8192;
119: fn header_to_frame_len(header: [u8; 4]) -> Result<usize> {
121:     if len > MAX_SHARD_FRAME_BYTES {
```
Doc corrigée fp32 et calcul 8192 x 8192 x 4 = 256 MiB présent (`shard.rs:72-84`). Rejet avant allocation confirmé dans `read_frame` (`shard.rs:151-164`).

### Livrable 6 : THREAT_MODEL §16
- Statut : CONFIRME
- Fichier(s) : `docs/security/THREAT_MODEL.md:980`, `:998`, `:1004`, `:1036`, `:1053`, `:1123`
- Evidence :
```md
998: | **SI-1 Activation reconstruction** ... | **High** | ...
999: | **SI-2 Layer gradient leakage** ... | N/A | ...
1000: | **SI-3 Activation fingerprinting** ... | Medium | ...
1001: | **SI-4 Collusion inter-workers** ... | **High** | ...
1002: | **SI-5 Latence side-channel** ... | Low | ...
```
Caveat activations en clair / pas de TEE GPU consumer / aucun secret dans prompts présent (`:1006-1014`). Incentive réputationnel non-monétaire présent (`:1036-1043`). Ancien §16 renommé §17 (`:1053`) et changelog v10 présent (`:1123-1130`).

### Livrable 7 : Tests
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/src/engine/shard_claim.rs:505`, `:516`, `:596`, `:645`, `:706`; `crates/nexus-core-rs/src/shard.rs:590`, `:615`
- Evidence :
```rust
590:     #[tokio::test]
591:     async fn shard_forward_invokes_forwarder() {
605:         assert_eq!(
606:             out, b"abab",
```
```rust
706:     #[test]
707:     #[ignore = "requires SBFB_SHARD_TEST_GGUF (llama-arch GGUF on disk)"]
708:     fn read_gguf_model_facts_extracts_llama_geometry() {
```
Tests exécutés :
`cargo test -p nexus-core-rs --locked shard_forward -- --nocapture` : 2 passed.
`cargo test -p nexus-worker-core --locked shard_claim -- --nocapture` : 9 passed. Il y a donc 11 hermétiques en pratique avec le test R2 `degenerate_geometry_is_rejected_fail_closed`, plus 2 tests GGUF `#[ignore]`.

### Livrable 8 : Invariants 0-bump-wire + patch note
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/shard.rs:95`, `:215`; `patches/llama-cpp-shard.patch:7`
- Evidence :
```rust
 95: /// frame cap. This is a placement/claim policy constant, never serialised on
 96: /// the wire (0-bump).
 97: pub const MAX_SHARD_N_CTX: u32 = 8192;
```
```text
 7: # SBFB Sprint 77 Phase F2 — Rust-side addition ...
 9: # GgufContext gains tensor_name / tensor_size / tensor_type accessors ...
15: # type-size table). No C++ change.
```
Scan `git diff HEAD` : pas de nouveau `DOMAIN_*`, pas de bump `*_FORMAT_VERSION`; seule mention ajoutée de `DOMAIN_SHARD_PLAN_V1` est documentaire dans le threat model. Frame reste opaque (`shard.rs:215-217`).

## Résumé final
- Total livrables : 8
- Confirmés : 8
- Gaps : 0
- Partiels : 0

Aucun chemin fail-open VRAM trouvé dans le chemin runtime F2, aucun manifeste non vérifié n’atteint le claim autorisé, aucun bump wire caché détecté, et aucun panic ajouté sur entrée réseau dans le forwarding inspecté.

