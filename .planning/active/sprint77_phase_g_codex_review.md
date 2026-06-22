`git --no-pager diff` était vide : les changements non committés sont staged. J’ai donc audité le diff réel via `git --no-pager diff --cached`, sans modifier l’index.

### Livrable 1 : `toploc.rs` net-new TOPLOC
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/toploc.rs:64`, `:96`, `:165`, `:206`, `:231`, `:267`, `:281`
- Evidence :
```rust
66: pub const TOPLOC_TOP_K: usize = 128;
73: pub const TOPLOC_THRESH_EXP_MISMATCH: u32 = 38;
77: pub const TOPLOC_THRESH_MANT_MEAN: u32 = 10;
81: pub const TOPLOC_THRESH_MANT_MEDIAN: u32 = 8;
96: pub fn bf16_bits(value: f32) -> u16 {
97:     (value.to_bits() >> 16) as u16
}
```
```rust
165: pub fn from_topk(topk: &[(u32, f32)]) -> Self {
168:     .take(TOPLOC_TOP_K)
169:     .map(|&(idx, v)| (idx, bf16_bits(v)))
171: kept.sort_unstable_by_key(|&(idx, _)| idx);
```
```rust
206: pub fn to_bytes(&self) -> Vec<u8> {
209:     out.extend_from_slice(&(n as u32).to_be_bytes());
211:     out.extend_from_slice(&idx.to_be_bytes());
212:     out.extend_from_slice(&vb.to_be_bytes());
```
```rust
230: let n = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
231: if n > TOPLOC_TOP_K {
232:     return Err(NexusError::Other(format!(
243: let mut indices = Vec::with_capacity(n);
```
`from_bytes` rejette donc `n > TOPLOC_TOP_K` avant allocation. `commitment()` est bien `blake3_hash(&self.to_bytes())` lignes 267-268. `compare()` aligne par index via `HashMap` lignes 281-300, utilise sentinelle `u64::MAX` lignes 307-308, et seuils stricts lignes 316-322.

### Livrable 2 : `lib.rs` module + re-exports additifs
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/lib.rs:66`, `:184`
- Evidence :
```rust
66: pub mod toploc;
...
184: pub use toploc::{
185:     TOPLOC_THRESH_EXP_MISMATCH, TOPLOC_THRESH_MANT_MEAN, TOPLOC_THRESH_MANT_MEDIAN, TOPLOC_TOP_K,
186:     ToplocComparison, ToplocFingerprint, bf16_bits,
187: };
```
Aucun nouveau `DOMAIN_*` ni `*_FORMAT_VERSION` ajouté dans le diff indexé.

### Livrable 3 : `verification.rs` Layer-3 re-cablée TOPLOC
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/verification.rs:33`, `:246`, `:395`, `:414`
- Evidence :
```rust
35: //! Sprint 77 Phase G upgrades this layer from an inert logprob hash
36: //! to the real **N0 TOPLOC commitment** ([`crate::toploc`]).
39: //! all-integer encoding in `logprobs_hash`; we compare it for
40: //! equality against a registered reference.
```
```rust
246: Some(expected) if *expected == reported_lp => {
247:     (LayerResult::passed("logprob hash match"), 1)
249: Some(_) => (LayerResult::failed("logprob hash mismatch"), -5),
262: ban: false,
```
```rust
399: let commitment =
400:     crate::toploc::ToplocFingerprint::from_topk(&[(3, 12.0), (7, -8.0), (1, 5.0)])
401:         .commitment();
408: assert!(report.passed);
410: assert_eq!(report.trust_delta, 1);
```
```rust
420: crate::toploc::ToplocFingerprint::from_topk(&[(3, 12.0), (7, -8.0)]).commitment();
422: crate::toploc::ToplocFingerprint::from_topk(&[(9, 99.0), (2, -50.0)]).commitment();
432: assert_eq!(report.logprobs.status, CheckStatus::Failed);
433: assert_eq!(report.trust_delta, -5);
434: assert!(!report.ban);
```

### Livrable 4 : docs code `task.rs` / `shard_plan.rs` / `canonical.rs`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/task.rs:504`, `crates/nexus-core-rs/src/shard_plan.rs:431`, `crates/nexus-core-rs/src/canonical.rs:275`
- Evidence :
```rust
504: /// **N0 TOPLOC commitment** (Sprint 77 Phase G): the 32-byte BLAKE3
508: /// [`crate::verification`]). The wire name is kept (`logprobs_hash`, 0 bump
515: /// **Auto-attestation caveat:** this is a self-claim of the worker, NOT a
520: pub logprobs_hash: [u8; 32],
```
```rust
431: /// **N0 TOPLOC commitment** (Sprint 77 Phase G). The 32-byte BLAKE3
438: /// **Binding only, not a tolerant proof:** a BLAKE3 commitment is compared
443: /// verifier recomputes it this is a self-claim
447: pub activation_fingerprint: [u8; 32],
```
```rust
279: /// auto-attestation ("here is what I EXECUTED") carrying the N0 TOPLOC
280: /// `activation_fingerprint` slot (a BLAKE3 commitment, [`crate::toploc`])
285: /// prefix forces disjoint pre-images. Purely additive, 0-bump
287: pub const DOMAIN_RUN_PROOF_V1: &[u8] = b"nexus-run-proof-v1";
```

### Livrable 5 : worker helper + méthode gated
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/src/llm/shard.rs:142`, `:187`, `:195`, `:362`, `:658`
- Evidence :
```rust
142: pub fn top_k_by_magnitude(values: &[f32], k: usize) -> Vec<(u32, f32)> {
152: (false, false) => vb
155:     .then(a.cmp(&b)),
159:     .take(k.min(values.len()))
```
```rust
187: pub fn toploc_commitment(hidden: &[f32]) -> [u8; 32] {
188:     let topk = top_k_by_magnitude(hidden, nexus_core_rs::TOPLOC_TOP_K);
189:     nexus_core_rs::ToplocFingerprint::from_topk(&topk).commitment()
}
```
```rust
195: #[cfg(feature = "llm_llama_cpp")]
196: mod backend {
362: pub fn toploc_commitment_last_token(&self, boundary: &[f32]) -> LlmBackendResult<[u8; 32]> {
373:     let last = &boundary[(n_tokens - 1) * self.n_embd..];
374:     Ok(super::toploc_commitment(last))
```
Test CI utile présent lignes 658-673 avec `assert_eq!` déterministe et `assert_ne!` swap-sensitive. Exécuté : `cargo test -p nexus-worker-core toploc_commitment_is_deterministic_and_swap_sensitive --locked` passe.

### Livrable 6 : `THREAT_MODEL.md`
- Statut : CONFIRME
- Fichier(s) : `docs/security/THREAT_MODEL.md:918`, `:1036`, `:1064`, `:1181`
- Evidence :
```md
918: ... primitive N0 TOPLOC (`toploc.rs`, commitment BLAKE3 ...) + helper worker ...
918: ... L'écriture/emission signee ... ride le data-plane (Phase H/I/J) ...
918: ... | **M (emission + recompute N1/N2 = Phase H/I)** |
```
```md
1036: ### N0 TOPLOC fingerprint (Sprint 77 Phase G)
1039: `nexus-core-rs/src/toploc.rs` calcule le **commitment BLAKE3 32B**
1048: commitment TOPLOC (egalite) ... 0 bump wire
1049: (slots deja v1), 0 nouveau `DOMAIN_*`.
```
```md
1064: **Caveat auto-attestation (cardinal)** : le commitment ...
1065: SON propre run est un **self-claim**, jamais une preuve
1067: `ToplocFingerprint::compare`) exige le sketch complet des deux cotes
```
Entrée v11 confirmée lignes 1181-1191.

### Livrable 7 : `PATTERNS.md` P64 + amendement P60.3
- Statut : CONFIRME
- Fichier(s) : `docs/rust/PATTERNS.md:3409`, `:3552`, `:3571`, `:3580`, `:3587`
- Evidence :
```md
3412: **Update (S77 Phase G)**: the real commitment primitive is now delivered
3414: `verification.rs` treats `logprobs_hash` as a TOPLOC commitment by equality.
3415: on-wire SIGNED emission ... rides the session data-plane (H/I/J)
```
```md
3559: **The 32-byte wire slot can only hold a COMMITMENT, never the tolerant
3562: forces a BLAKE3 commitment of the canonical integer encoding into the slot.
3566: (`ToplocFingerprint::compare`) needs the full sketch on both sides
```
```md
3571: **Sketch-direct over the GF(65497) polynomial
3580: **The hashed pre-image must be ALL-INTEGER
3587: **Tolerant comparison stays integer even locally.
3592: A `compare()` whose call-sites are all `#[cfg(test)]` is correct
```

### Livrable 8 : tests mandates + boundaries
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/toploc.rs:356`, `:379`, `:411`, `:515`, `:548`, `:571`
- Evidence :
```rust
356: fn toploc_fingerprint_encode_decode_roundtrip() {
361: assert_eq!(f.indices(), &[1, 3, 5, 7, 9]);
364: let back = ToplocFingerprint::from_bytes(&bytes).expect("decode");
365: assert_eq!(back, f, "round-trip is exact");
```
```rust
379: fn toploc_detects_model_swap() {
397: assert_ne!(prover.commitment(), swapped.commitment(), ...)
404: let cmp = prover.compare(&swapped);
407: assert!(!cmp.accepted, "disjoint top-k must be rejected");
```
```rust
411: fn toploc_accepts_same_model_within_threshold() {
430: assert_eq!(cmp.exp_mismatches, 0);
432: assert_eq!(cmp.mant_err_sum, 5, "mantissa error 1 each");
433: assert!(cmp.accepted, ...)
```
Boundaries : even median lignes 515-544, same-index exponent mismatch lignes 548-567, threshold strict `<` lignes 571-628. Exécuté : `cargo test -p nexus-core-rs toploc --locked` passe 17 tests.

### Invariants transverses
- NO-FLOAT pre-image : CONFIRME. `to_bytes()` sérialise seulement `u32` + `u16` lignes 206-214 ; seul boundary float->int est `bf16_bits()` lignes 96-97.
- 0-BUMP WIRE : CONFIRME. Slots `[u8; 32]` réutilisés dans `task.rs:520` et `shard_plan.rs:447`; `DOMAIN_RUN_PROOF_V1` inchangé ligne 287 ; grep du diff indexé : aucun ajout `pub const DOMAIN_*` / `*_FORMAT_VERSION`.
- SCOPE : CONFIRME. `rg` ne trouve `ToplocFingerprint::compare` / `.compare(` que dans docs et tests `toploc.rs`; aucun fichier `rerun`, `redundancy`, `validator` dans `git diff --cached --name-only`; aucune émission prod `RunProof` ajoutée.

## Resume final
- Total livrables : 8
- Confirmes : 8
- Gaps : 0
- Partiels : 0

