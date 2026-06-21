Audit fait sur l’état de travail de `master` (`placement.rs` est présent mais non suivi Git). Test ciblé lancé : `cargo test -p nexus-coordinator-rs placement --locked` -> 11 tests passés.

### Livrable 1 : module créé et enregistré
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/placement.rs:1`, `crates/nexus-coordinator-rs/src/lib.rs:27`
- Evidence :
```rust
// placement.rs existe
//! Sprint 77 Phase D — Parallax placement scheduler (phase 1).

// lib.rs
pub mod pii_redactor;
pub mod placement;
pub mod pow_counter;
```
- Note : `placement.rs` est `??` dans `git status`, donc non versionné dans l’état Git courant.

### Livrable 2 : seuil sharding
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/placement.rs:209`
- Evidence :
```rust
let max_free = candidates.iter().map(|w| w.vram_free_bytes).max()
    .expect("candidates non-empty");
if model.quantized_vram_bytes <= max_free {
    return Ok(PlacementOutcome::EndpointFederation);
}
```
- Le shard dégénéré est aussi refusé après planification : `plan.assignments.len() < MIN_SHARD_WORKERS` retourne une erreur lignes 285-288.

### Livrable 3 : water-filling exact et borné
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/placement.rs:384`, `:403`, `:428`
- Evidence :
```rust
let caps: Vec<u32> = ordered.iter()
    .map(|w| layer_capacity(w.vram_free_bytes, per_layer_bytes).min(u32::MAX as u64) as u32)
    .collect();

let mut alloc: Vec<u32> = ordered.iter().enumerate()
    .map(|(i, w)| ((total * w.vram_free_bytes as u128 / sum_w) as u32).min(caps[i]))
    .collect();
```
```rust
while assigned < total_layers {
    let mut progress = false;
    for &i in &prio {
        if alloc[i] < caps[i] {
```
- La boucle augmente `assigned` uniquement si `alloc[i] < caps[i]`, s’arrête à `total_layers`, et retourne une erreur si aucun progrès n’est possible. Les multiplications de répartition utilisent `u128`.

### Livrable 4 : k-medoids RTT déterministe
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/placement.rs:460`, `:492`, `:540`, `:577`
- Evidence :
```rust
pub(crate) fn cluster_order_by_rtt(...) -> Vec<usize> {
    let n = workers.len();
    if n <= 1 {
        return (0..n).collect();
    }
```
```rust
let medoids = pam_swap(&d, &pubkeys, pam_build(&d, &pubkeys, k));
let mut clusters: BTreeMap<[u8; 32], (usize, Vec<usize>)> = BTreeMap::new();
```
- `rg` ne trouve aucun appel `rand` / `thread_rng` dans `placement.rs` ; seulement des commentaires “randomness”. Pas de GeoIP/ASN/région, et `BTreeMap` + tie-break `pubkey` rendent l’ordre déterministe. La permutation complète est construite en poussant chaque index `p` exactement une fois lignes 493-518.

### Livrable 5 : ShardPlan contigu et couverture complète
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/placement.rs:251`, `:280`, `:299`; `crates/nexus-core-rs/src/shard_plan.rs:211`
- Evidence :
```rust
let start = cursor;
let end = cursor + layers;
cursor = end;
assignments.push(ShardAssignment { layer_start: start, layer_end: end, ... });
```
```rust
if !covers_full_model(&plan, model.total_layers) {
    return Err(CoordinatorError::Validation(
        "placement: produced plan does not cover [0..total_layers) contiguously".into(),
```
- `covers_full_model` appelle `is_pipeline_contiguous()` puis vérifie `first.layer_start == 0` et `last.layer_end == total_layers`.

### Livrable 6 : absorption SYBIL-SEEDER-TAIL
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/placement.rs:313`, `:345`, `:769`
- Evidence :
```rust
fn sampling_key(session_id: &str, pubkey: &[u8; 32]) -> [u8; 32] {
    let mut input = Vec::with_capacity(session_id.len() + 32);
    input.extend_from_slice(session_id.as_bytes());
    input.extend_from_slice(pubkey);
    *blake3::hash(&input).as_bytes()
}
```
```rust
usable.sort_by(|a, b| {
    b.vram_free_bytes.cmp(&a.vram_free_bytes).then_with(|| {
        sampling_key(session_id, &a.worker_pubkey)
            .cmp(&sampling_key(session_id, &b.worker_pubkey))
```
- Le test `sybil_seeder_tail_sampling_is_deterministic_non_lexicographic` vérifie sélection par clé BLAKE3, ordre non lexicographique et reproductibilité lignes 781-803.

### Livrable 7 : conformité 0-bump, no-float-leak, scope cut, constantes
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/placement.rs:60`, `:66`, `:71`, `:78`, `:84`
- Evidence :
```rust
pub const MIN_SHARD_WORKERS: usize = 2;
pub const KMEDOIDS_DEFAULT_K: usize = 2;
pub const KMEDOIDS_MAX_ITER: usize = 64;
pub const MISSING_RTT_PENALTY_MICROS: u64 = 60_000_000;
```
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerPlacementProfile {
    pub worker_pubkey: [u8; 32],
    pub vram_free_bytes: u64,
```
- Aucun `Serialize/Deserialize`, `DOMAIN_*`, `*_FORMAT_VERSION`, `f32` ou `f64` en code dans `placement.rs`. `canonical.rs`, `shard_plan.rs` et `consent.rs` n’ont pas de diff Git. Le seul `estimated_vram_mb` trouvé est dans `consent.rs`, non modifié.

### Livrable 8 : tests utiles
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/placement.rs:642`, `:665`, `:691`, `:725`, `:769`, `:807`
- Evidence :
```rust
assert_eq!(l1 + l2 + l3, 80, "every layer is placed exactly once");
assert_eq!(l2, l1 + l3, "double VRAM => double the layer share");
assert_eq!(outcome, PlacementOutcome::EndpointFederation);
```
```rust
assert_eq!(order.len(), 4);
assert_eq!(pa.abs_diff(pb), 1, "low-RTT pair A,B must be consecutive");
assert_eq!(placed, 80);
```
- Les tests défensifs couvrent VRAM agrégée insuffisante, entrées dégénérées, RTT manquant, stabilité de sampling et plans partiels/gappés. Aucun test sans assertion utile trouvé.

## Résumé final
- Total livrables : 8
- Confirmés : 8
- Gaps : 0
- Partiels : 0

Défauts spécifiques recherchés : pas de cas trouvé où la somme des couches placées diffère de `total_layers`; pas de non-déterminisme dans le module; pas de float dans le `ShardPlan` produit; pas de tri lexicographique caché dans la sélection candidate; les `expect` production sont protégés par des gardes; les tests ciblés passent.
