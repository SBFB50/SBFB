# Sprint 40 — Audit findings (Phase 0 gate S41)

**Auditeur** : session fraiche (pas la session qui a code S40).
**Tip d'entree** : `0c557f1` (S40 Phase D wrap-up).
**Verdict** : **PASS** (0 P0, 0 P1, 0 P2 nouveau, 4 P2/P3 carries
confirmes documentes).

## Evidence d'exploration par dimension

### Track A — Securite / canary_input (3/3 PASS)

**A-1 — signable_json() canonical JSON** :
Read `canary_input.rs:49-63`. Les cles du `serde_json::json!({})` sont
ecrites manuellement en ordre alphabetique : `created_at_unix`, `prompts`,
`version` (outer) et `expected_answer`, `prompt`, `prompt_id`, `tolerance`
(inner). serde_json 1.0 sans feature `preserve_order` utilise BTreeMap
(tri alphabetique). Grep `serde_json.*preserve_order` sur tout le
workspace : 0 match — pas de feature activee. Le resultat est donc
alphabetique quel que soit l'ordre d'insertion. Parite avec Python
`json.dumps(sort_keys=True)` confirmee.
Tests roundtrip sign/verify/tamper presents (`canary_input.rs:677-699`).
**PASS.**

**A-2 — Levenshtein strsim vs rapidfuzz** :
Read `canary_input.rs:335`. Utilise `strsim::normalized_levenshtein()`
qui retourne une similarite [0,1] (1 - normalized_distance). Meme
formule que rapidfuzz Python : `edit_distance / max(len1, len2)`.
Observer compare `similarity >= tolerance` (0.85 defaut) — coherent.
Tests `observer_divergence_below_tolerance` et
`observer_no_divergence_above_tolerance` valident les deux branches.
**PASS.**

**A-3 — CanaryInputGuardrail Tripwire** :
Read `canary_input.rs:589-607`. Impl `Guardrail` trait, direction
`Input`, retourne `GuardrailOutcome::Tripwire` quand injection
decidee. Coherent avec PiiInputGuardrail S39 (meme pattern Tripwire).
Test `guardrail_tripwire_on_inject` (`canary_input.rs:816-833`)
present. Carry P2-REVIEW-A-1-S39 (Tripwire vs Mutation trait
extension) 1/3 documente.
**PASS.**

### Track B — Architecture / Tier 3 (4/4 PASS)

**B-1 — redundancy SHA-256 parite** :
Read `redundancy.rs:23-26`. `hash_result_bytes()` utilise
`Sha256::digest(data)` puis `hex::encode()`. Equivalent Python :
`hashlib.sha256(data).hexdigest()`. Parite wire confirmee. Les deux
operent sur bytes (`&[u8]` Rust / `.encode()` Python).
**PASS.**

**B-2 — watermark PRF crosscheck** :
Read `watermark_detector.rs:83-93` vs `crates/nexus-worker-core/
src/llm/watermark.rs:25-34`. Les deux `prf_score()` sont
fonctionnellement identiques : HMAC-SHA256, `to_le_bytes()` pour
context+token_id, `from_be_bytes(result[..8])` pour top8, division
par `u64::MAX`. Parite confirmee.
**PASS.**

**B-3 — rerun anti-loop** :
Read `rerun.rs:39-42`. `should_rerun()` commence par
`if self.is_rerun(task_id) { return false; }`. Un rerun de rerun
est impossible. Test `sampler_anti_loop_rerun_of_rerun`
(`rerun.rs:90-93`) couvre ce cas.
**PASS.**

**B-4 — honeypot eclipse seuils** :
Read `honeypot.rs:10-11`. `ECLIPSE_CO_LOCATION_THRESHOLD = 0.80`,
`ECLIPSE_CONSECUTIVE_ROTATIONS = 3`. `evaluate()` verifie
`pct >= self.threshold` (L95) et `*streak >= self.required_rotations`
(L100). Tests `eclipse_alert_threshold` (4 rotations, alerte) et
`eclipse_no_alert_below_threshold` (30%, pas d'alerte) couvrent les
deux branches.
**PASS.**

### Track C — Tests / coverage (3/3 PASS)

**C-1 — delta cumule 991→1023 (+32)** :
Verifie par `cargo nextest run --workspace --locked --no-fail-fast` :
1023 passed, 0 skipped. (Un test flaky `browse::probe_and_cache_*`
dans shell-daemon-core a echoue au 1er run puis passe au 2e — non
touche par S40, pre-existant).

Decomposition : Phase A +3 (http.rs), Phase B +13 (canary_input.rs),
Phase C +16 (redundancy 3 + watermark 4 + rerun 5 + honeypot 4).
Total = 32, coherent avec verification.md.
**PASS.**

**C-2 — canary_input 13 tests** :
13 tests comptees dans `canary_input.rs#[cfg(test)]` L639-835 :
serde_roundtrip, sign_verify, tampered_fails, wrong_pubkey_fails,
save_load, injector_rate_always, injector_round_robin,
observer_divergence, observer_no_divergence, ring_buffer_bounded,
policy_from_toml, default_seed_prompts_count, guardrail_tripwire.
Couvre : set crypto, injector, observer, policy, guardrail, seeds.
**PASS.**

**C-3 — Tier 3 16 tests** :
redundancy.rs (3) : majority, mismatch, pending.
watermark_detector.rs (4) : not_watermarked, too_few, prf_deterministic,
prf_different.
rerun.rs (5) : anti_loop, rate_zero, get_original, scorer_match,
scorer_mismatch.
honeypot.rs (4) : unique_keys, alert_threshold, no_alert_below,
rotation_new_peers.
Total 16, couvre les 4 modules.
**PASS.**

### Track D — Process / meta (3/3 PASS)

**D-1 — G8 preflights** :
3/3 artefacts presents dans `.planning/active/` :
`sprint40_phase_A_preflight.md`, `sprint40_phase_B_preflight.md`,
`sprint40_phase_C_preflight.md`. Commits chore(planning) anterieurs
aux commits feat confirmes par git log (`4dec922` < `2b6e3dd`,
`e6fd5fc` < `f5b6731`, `4bbd37c` < `0b9df49`).
**PASS.**

**D-2 — scope cuts 12/12** :
Grep du diff S40 (`f8fae0c..0b9df49`) pour les 12 scope cuts du
kickoff §D5 :
- Pas de routes HTTP canary_input (SC-1) — grep `/api/canary_input` : 0
- Pas de wire dispatcher (SC-2) — grep `DispatchHook` : 0
- Pas de gossip sync (SC-3) — grep `gossip.*canary` : 0
- Pas de quarantine/upload queue Rust (SC-4,5) — grep `quarantine_queue\|upload_queue` : 0
- Pas de migration routes API (SC-6) — pas de changement `api/` Python
- Python non supprime (SC-7) — `packages/` non touche
- Pas de CI/VPS/tag/kudos/mutation (SC-8..12) — aucun match
**PASS.**

**D-3 — dette pair 5 items** :
(a) result_event_tx : diff `http.rs:149-155` supprime `#[allow(dead_code)]`
    + L1400-1402 wire `send(ResultEvent::NewResult(entry))`. RESOLU.
(b) substring : diff `output_filter.rs:62-77` supprime variable morte
    `prompt_chars`. Le `return true` (early exit premier match) etait
    deja en place — le fix est cleanup dead code. RESOLU.
(c) chain singleton : diff `guardrails.rs:132-143` remplace fonctions
    `default_{output,input}_chain()` retournant valeur par OnceLock
    retournant `&'static`. RESOLU.
(d) HTTP integration tests : 3 tests ajoutes (`submit_task_pii_rejected`,
    `canary_observed_post_ok`, `canary_network_health_get_ok`). RESOLU.
(e) lowercase doc : §P40 ajoute dans PATTERNS.md. RESOLU.
**PASS (5/5).**

### Track E — Dependencies (2/2 PASS)

**E-1 — versions + advisories** :
Deps ajoutees dans `crates/nexus-coordinator-rs/Cargo.toml` :
- `toml = { workspace = true }` → `0.8` (standard, parser TOML)
- `sha2 = { workspace = true }` → `0.10` (RustCrypto, deja transitive)
- `hmac = { workspace = true }` → `0.12` (RustCrypto, deja transitive)
`cargo audit` non installe, mais les 3 crates sont des deps workspace
existantes (sha2/hmac transitives via iroh stack, toml via arti-client).
Pas d'advisory connue sur ces versions.
**PASS.**

**E-2 — transitives inattendues** :
Cargo.lock diff montre +2 entries (toml 0.8 ajoute `toml_edit` +
`winnow` si pas deja present). sha2/hmac etaient deja en transitive —
pas de nouvelle dep. Aucune dep inattendue.
**PASS.**

### Track F — Doc coherence (4/4 PASS)

**F-1 — HARDENING_ROADMAP compteurs** :
`docs/security/HARDENING_ROADMAP.md:3` — `last_validated: 2026-04-29`
mention `1023 Rust / ~2026 total`. Coherent avec nextest.
**PASS.**

**F-2 — CLAUDE.md etat actuel** :
`CLAUDE.md:124` — `Sprints 0-40 CLOSED`. Compteurs `1023 Rust / ~2026
total` a L126. Carries S41 documentes L128-140.
**PASS.**

**F-3 — Phase review files** : 3/3 presents
(`sprint40_phase_A_review.md`, `_B_`, `_C_`).
**PASS.**

**F-4 — Phase preflight files** : 3/3 presents
(`sprint40_phase_A_preflight.md`, `_B_`, `_C_`).
**PASS.**

## Carries S41 confirmes

Les 4 nouveaux carries issus des phase reviews S40 sont correctement
documentes dans `sprint41_audit_plan.md` §Carries et dans CLAUDE.md :

| Item | Compteur | Source |
|---|---|---|
| P2-REVIEW-B-1-S40 rand_range non-random | 1/3 | Phase B review |
| P2-REVIEW-C-1-S40 SHA-256 vs BLAKE3 | 1/3 | Phase C review |
| P3-REVIEW-B-1-S40 Manager multiple Mutex | 1/3 | Phase B review |
| P3-REVIEW-C-1-S40 rerun deterministic hash | 1/3 | Phase C review |

Les carries pre-existants (P2-A-1 rand blocker, P2-AUDIT-2 iroh,
P2-REVIEW-A-1-S39, P2-REVIEW-B-1-S39, P3-*-S39) sont aussi
correctement reportes.

## Observation (non-finding, informatif)

- **browse flaky** : `nexus-shell-daemon-core browse::tests::
  probe_and_cache_with_quorum_majority_continues_to_dial` echoue
  au 1er run (timing), passe au 2e. Non touche par S40 (dernier
  modifie S24). Pre-existant. Pas un finding S40.

## Recommendation

Commit autorise. 0 P0, 0 P1, aucun fix requis avant ouverture S41.
