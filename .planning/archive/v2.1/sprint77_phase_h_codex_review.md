Audit basé sur l’état courant du dépôt, sans historique de session. Branch: `master`, avec modifications locales Phase H. Tests ciblés exécutés et verts.

### Livrable 1 : N1 VRF spot-check primitive
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/verifiable_draw.rs:27`, `:103`, `:115`, `:131`, `:143`, `:154`
- Evidence :
```rust
103: pub fn vrf_draw(signer: &KeyPair, seed: &[u8]) -> VrfDraw {
104:     let proof = signer.sign(&draw_message(seed));
105:     let output = draw_output(&proof);
106:     VrfDraw { proof, output }
```
`vrf_verify` revalide la signature et recalcule l’output (`:115-121`). `vrf_is_selected` est all-integer `u128` (`:131-135`), avec tests 0%, 100%, >100%. Température et seed sont dérivées de l’output (`:143-156`). Les caveats verbatim sont présents : “NOT an ECVRF (RFC 9381)”, “Ed25519 is malleable”, “mitigation, not a guarantee” (`:27-35`). `Cargo.toml`/`Cargo.lock` ne sont pas modifiés.

### Livrable 2 : Selection VRF côté coordinator
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/rerun.rs:55`, `:68`
- Evidence :
```rust
55: pub fn draw_spotcheck(verifier: &KeyPair, seed: &[u8], rate_bp: u32) -> Option<VrfDraw> {
56:     let draw = vrf_draw(verifier, seed);
57:     if vrf_is_selected(&draw.output, rate_bp) {
68: pub fn verify_spotcheck_selection(
```
`verify_spotcheck_selection` appelle `vrf_verify` puis `vrf_is_selected` (`:74-76`). Scan anti-zombie : aucune définition/usage réel de `RerunSampler`, `DivergenceScorer`, `simple_hash`; seules mentions en commentaires d’historique.

### Livrable 3 : Recompute tolerant + Token-DiFR
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/rerun.rs:110`, `:119`, `:132`, `:142`
- Evidence :
```rust
110: pub fn spotcheck_activation_ok(prover: &ToplocFingerprint, replay: &ToplocFingerprint) -> bool {
111:     prover.compare(replay).accepted
148:     spotcheck_activation_ok(prover_fp, replay_fp)
149:         && tokens_agree(prover_tokens, replay_tokens, TOKEN_AGREEMENT_PCT)
```
La comparaison est bien `ToplocFingerprint::compare`, pas égalité octet. `TOKEN_AGREEMENT_PCT=95` est défini (`:44`), `token_agreement` compte matching/total (`:119-127`), et `tokens_agree` est all-integer (`:132-134`).

### Livrable 4 : Incentive reputationnel gate
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/rerun.rs:93`, `crates/nexus-coordinator-rs/src/kudos_ledger.rs:49`, `:76`
- Evidence :
```rust
93: pub fn spotcheck_creditable(
100:     verify_spotcheck_selection(verifier_pubkey, seed, proof, rate_bp)
101:         && verifier_run_proof.proof.worker_pubkey == *verifier_pubkey
102:         && verifier_run_proof.verify_signature().is_ok()
```
Le crédit réutilise bien `kudos_ledger::credit` existant (`kudos_ledger.rs:76-83`). `HashableKudosEntry` reste inchangé dans le diff local et garde la pré-image `DOMAIN_KUDOS_V1` (`:49-70`). Aucun `crates/nexus-coordinator-rs/src/curator.rs`. Aucun mécanisme monétaire Phase H détecté; les mots stake/bond/slash/burn/deposit n’apparaissent que dans la prose qui les interdit.

### Livrable 5 : Mapping criticité -> niveau
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/verification.rs:343`, `:372`, `crates/nexus-core-rs/src/task.rs:39`, `:232`
- Evidence :
```rust
372: pub fn criticality_maps_to_verification_level(task: &Task) -> VerificationLevel {
373:     if task.verifiable && task.redundancy_factor > 1 {
374:         VerificationLevel::N2
375:     } else if task.verifiable {
376:         VerificationLevel::N1
```
`N3` existe mais n’est jamais retourné (`verification.rs:343-352`, tests `:588-597`). Provenance honnête confirmée : `task_canonical_bytes` retire `redundancy_factor` (`task.rs:39-43`), tandis que `verifiable` est documenté signé (`task.rs:232-239`) et testé comme changeant la signature (`task.rs:962-991`). Table unique de taux en constantes `SPOT_CHECK_RATE_*_BP` + `spot_check_rate_bp` (`verification.rs:284-318`).

### Livrable 6 : Tests CI hermétiques
- Statut : CONFIRME
- Fichier(s) : `verifiable_draw.rs:173`, `rerun.rs:222`, `verification.rs:574`
- Evidence :
```rust
173: fn n1_vrf_selects_deterministic_verifier() {
252: fn n1_spot_check_randomizes_temp_and_seed() {
223: fn incentive_credits_reputation_on_honest_spotcheck() {
575: fn criticality_maps_to_verification_level() {
```
Tests adversariaux présents : tampering VRF (`verifiable_draw.rs:195-210`), bornes de taux (`:214-232`), impostor/non-sélectionné (`rerun.rs:267-294`), tolerant-vs-égalité (`:296-319`), Token-DiFR forgé (`:321-345`). Exécution : `cargo test -p nexus-core-rs verifiable_draw --locked` 6/6, `cargo test -p nexus-coordinator-rs rerun --locked` 8/8, `cargo test -p nexus-core-rs criticality --locked` 2/2, `cargo test -p nexus-core-rs spot_check_rate --locked` 2/2.

### Livrable 7 : Docs + canonical + exports
- Statut : CONFIRME
- Fichier(s) : `docs/security/THREAT_MODEL.md:1086`, `:1140`, `:1253`, `docs/rust/PATTERNS.md:3605`, `canonical.rs:292`, `lib.rs:68`
- Evidence :
```md
1088: Phase H cable la **primitive du 2e etage, N1**
1091: via la comparaison **tolerante** (`ToplocFingerprint::compare`, jamais l'egalite
1094: Le verdict combine l'activation-fingerprint ET les **tokens sous seed
1098: **Construction et honnetete cardinale** : le tirage est une **signature Ed25519
```
THREAT_MODEL est en v12 (`:1253-1265`), inclut N1, incentive “CABLE Phase H” (`:1140-1154`), trois surfaces Sev M (`:1113-1135`) et MAJ §15.2 row I (`:918`). PATTERNS §P65 existe (`PATTERNS.md:3605-3667`). `DOMAIN_VRF_DRAW_V1` est additif (`canonical.rs:292-310`) et `lib.rs` exporte module + constantes/fonctions (`lib.rs:68`, `:191-199`).

## Résumé final
- Total livrables : 7
- Confirmés : 7
- Gaps : 0
- Partiels : 0