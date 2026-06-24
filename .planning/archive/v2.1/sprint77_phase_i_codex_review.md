Audit effectué sur le working tree courant : `master`, commit `fdc65a2`, avec fichiers Phase I non committés.

### Livrable 1 : N2 redondance tolérante
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/redundancy.rs:57`, `:64`, `:74`, `:86`, `:139`
- Evidence :
```rust
pub fn fingerprints_agree(a: &ToplocFingerprint, b: &ToplocFingerprint) -> bool {
    a.compare(b).accepted && b.compare(a).accepted
}
...
pub fn tolerant_quorum_accepts(fingerprints: &[ToplocFingerprint], min_agree: usize) -> bool {
    largest_agreeing_cluster(fingerprints) >= min_agree
}
```
Implémentation en clique via `largest_agreeing_cluster`, pas pivot-count, pas égalité hash.

### Livrable 2 : N2 coordinator additif
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/validator.rs:118`, `:220`, `:343`, `:377`
- Evidence :
```rust
.filter(|(entry, sketch)| {
    entry.verify_signature().is_ok()
        && sketch.commitment() == entry.proof.activation_fingerprint
})
...
if tolerant_quorum_accepts(&verified, min_agree) {
```
`validate_tolerant_quorum_shard` filtre bien signature + carrier N0 signé avant vote. `git diff -U0 -- validator.rs` montre uniquement import + nouvelle fonction/tests, aucune hunk dans `validate_quorum_pre_guardrail` ni dans le dispatch `redundancy_factor > 1`.

### Livrable 3 : N3 commit-reveal
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/activation_commit.rs:105`, `:181`, `:206`, `:244`, `:298`
- Evidence :
```rust
if !reveal.opens(committed) {
    return RevealVerdict::CommitmentMismatch;
}
if verifier_recompute.compare(&reveal.sketch).accepted {
    RevealVerdict::Accepted
}
```
Payload signé contient `version`, `worker_pubkey`, `session_id`, `frontier_index`, `commitment`. Vérification : version -> caps -> attribution -> crypto. Le verdict de reveal est binding puis `compare`, jamais égalité de commitment.

### Livrable 4 : N3 SENTINEL
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/sentinel.rs:48`, `:68`, `:127`, `:155`
- Evidence :
```rust
let deviates = signal.abs_diff(ema).saturating_mul(SENTINEL_BP_DENOMINATOR)
    >= self.thresh_bp.saturating_mul(ema);
if !deviates {
    self.ema = Some(ema_step(ema, signal, self.alpha_bp));
}
```
EMA entière `u128`, forward-only, pas de `f32/f64`, localisation directe par frontière, et outlier non injecté dans la baseline.

### Livrable 5 : Wire / versions / dépendances
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/canonical.rs:312`, `:332`, `crates/nexus-core-rs/src/lib.rs:87`
- Evidence :
```rust
pub const DOMAIN_ACTIVATION_COMMIT_V1: &[u8] = b"nexus-activation-commit-v1";
...
pub use canonical::{
    DOMAIN_ACTIVATION_COMMIT_V1, DOMAIN_AGE_WITNESS_V1, ...
```
`git diff` ne montre aucun bump de `*_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION` existant. Aucun `Cargo.toml`, `Cargo.lock` ou `package*.json` modifié.

### Livrable 6 : Tests obligatoires
- Statut : CONFIRME
- Fichier(s) : `redundancy.rs:159`, `:175`; `validator.rs:999`; `activation_commit.rs:335`; `sentinel.rs:183`
- Evidence :
```rust
assert_ne!(fps[0].commitment(), fps[1].commitment(), ...);
assert!(!tolerant_quorum_accepts(&fps, 3));
...
assert_eq!(verify_reveal(&entry.payload, &reveal, &recompute), RevealVerdict::Accepted);
```
Les tests existent et assertent utilement. Le roundtrip N3 est nommé `n3_activation_commit_reveal_roundtrip`. Tests ciblés exécutés et PASS : `cargo test -p nexus-core-rs --locked n2_`, `cargo test -p nexus-core-rs --locked n3_`, `cargo test -p nexus-coordinator-rs --locked validator_exact_quorum_unchanged`, `cargo test -p nexus-coordinator-rs --locked n2_shard_quorum`.

### Livrable 7 : Docs
- Statut : CONFIRME
- Fichier(s) : `docs/security/THREAT_MODEL.md:1156`, `:1200`, `:1362`; `docs/rust/PATTERNS.md:3672`
- Evidence :
```text
le verdict est en deux temps : binding ... PUIS correction TOLERANTE (`compare`), jamais
l'egalite du commitment BLAKE3 ...
Ce n'est donc PAS la soundness fraud-proof d'opML
```
Docs alignées avec le code : §16 N2/N3, surfaces SI-6..SI-11, row §15.2 mise à jour, changelog v13, §P66. Elles évitent la sur-promesse opML, évitent “bissection O(1)”, et gardent PO-12 non monétaire.

## Résumé final
- Total livrables : 7
- Confirmés : 7
- Gaps : 0
- Partiels : 0

