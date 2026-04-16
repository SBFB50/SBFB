# Sprint 19 Phase B — nexus-phase-auditor review

**HEAD pre-commit** : `ab6985c`
**Draft commit title** : `feat(sprint19): Phase B — PoW Hashcash gossip subscribe (difficulty 2^18 per-relai)`
**Timebox** : 18 min
**Auditor** : nexus-phase-auditor (session 2026-04-16)

## Verdict : PASS

**Promu CONCERN → PASS** après intégration des 2 mitigations
code/commit recommandées par l'auditeur avant commit :

- **P2-1 FIXED** : commit body enrichi d'une section "Déviation vs
  plan §5.2" explicitant le report `iroh_runtime.rs` wiring →
  Sprint 20 Phase 1. Auditeur S20 Phase 0 aura la trace.
- **P2-3 FIXED** : `.planning/research/S19_phase_B_pow_hashcash_design.md`
  (42 KB, threat model T0-T3 + alternatives + limitations)
  désormais stagé dans ce commit.

**P2-2 + P3-1 + P3-2 reportés Phase F wrap-up** (cohérent avec
Phase A pattern : P3 non-code / errata planning / doc-only reportés
au wrap-up).

## Verdict initial (pré-intégration mitigations) : CONCERN

0 finding P0, 0 finding P1. Commit autorisé avec 3 findings P2 et
2 findings P3 à logguer. Verdict initial conservé ci-dessous pour
trace audit.

---

## Dimensions

### Security

- `#![forbid(unsafe_code)]` présent dans `crates/nexus-core-rs/src/lib.rs:31`. Aucun bloc `unsafe` dans les 3 nouveaux modules. La mention "zero unsafe, zero async" dans `pow_gossip.rs:66` est factuellement correcte. **PASS**.
- `unwrap()` en prod : les 3 occurrences prod sont dans `PowSolveCache` sous la forme `.expect("PowSolveCache mutex poisoned")`. Pattern mutex-poison standard et légitime — un mutex ne peut être poisonné que si un thread panique en le tenant. Les `unwrap()` `#[cfg(test)]` sont tous dans des contextes test. **PASS**.
- Secrets / path traversal : aucun token, clé, ou motif `AKIA/ghp_/pat_`. `relay_pow_policy.rs` ne fait que `fs::read_to_string` avec un path fourni par env var ou `~/.sbfb/`. **PASS**.
- Loopback / wire / zip : aucun fichier `loopback/`, `blob_serve/`, `zip` touché. `canonical.rs` reçoit uniquement l'ajout de la constante `DOMAIN_POW_V1`. **PASS**.
- Canonicalization : le pre-image SHA256 passe par `canonical_bytes(self, DOMAIN_POW_V1)` (JCS + domain tag) dans `pow.rs:257`. Le transport dans `pow_gossip.rs` utilise `serde_json::to_vec` pour l'enveloppe wire — intentionnel et documenté (`pow_gossip.rs:20-26`). **PASS**.
- Domain separation : `DOMAIN_POW_V1 = b"nexus-pow-v1"` avec null-byte separator. Disjoint de tous les domaines existants (task/result/claim/invite/kudos/curator-list/provenance/warrant-canary). **PASS**.
- `PowError::ZeroDifficulty` : correctement surfacé si la policy retourne 0 — la défense ne peut pas être silencieusement désactivée. **PASS**.

### Patterns

- SPDX headers : `// SPDX-License-Identifier: AGPL-3.0-or-later` présent en ligne 1 des 4 nouveaux fichiers (`pow.rs`, `pow_gossip.rs`, `relay_pow_policy.rs`, `benches/pow.rs`). **PASS**.
- Pre-launch protocol policy : `POW_FORMAT_VERSION: u16 = 1` — ne bumpe pas. Le module doc et le champ `format_version` documentent explicitement la politique pre-v1.0. **PASS**.
- Version hardness : `solve()` et `verify_stateless()` rejettent `format_version != POW_FORMAT_VERSION` avec `UnknownVersion` — pas de tolerant decoder multi-version. **PASS**.
- Workspace deps : `sha2`, `criterion`, `dashmap`, `hex`, `toml` utilisent `{ workspace = true }` dans `crates/nexus-core-rs/Cargo.toml`. **PASS**.
- PATTERNS.md mis à jour : Sprint 19.1 (primitive/wire/enforcement separation) et Sprint 19.2 (forward-compat publisher_pubkey) ajoutés. **PASS**.
- 0 tests ignorés/skippés : `cargo test -p nexus-core-rs` confirme `0 ignored` dans tous les `test result:` outputs. **PASS**.
- Difficulty clamp : `HashcashChallenge::new` clamp silencieux à `MAX_DIFFICULTY_BITS=30` documenté + test `new_clamps_difficulty_at_maximum`. `RelayPowPolicy::from_file` refuse loud si `> MAX_DIFFICULTY_BITS`. Double protection conforme. **PASS**.

### Scope-cuts

Grep sur les mots-clés kickoff §6 dans le diff :

- `rate_limit` : absent du code. **PASS**.
- `kudos` : présent **uniquement dans des commentaires doc** décrivant les forward-compat S22. Aucun code kudos fonctionnel. **PASS**.
- `encryption_at_rest`, `duress`, `domain_fronting`, `tor_bridge` : absents. **PASS**.
- `ML-DSA` / `ML-KEM` : présent uniquement dans les commentaires doc forward-compat (`pow.rs:49`). Aucun code PQC fonctionnel. **PASS**.
- `2_node_iroh` : absent. L'integration test `end_to_end_*` utilise un channel `mpsc` en-process, documenté "mock transport" (pow_gossip.rs:545-562). **PASS**.
- **Enforcement runtime gossip** : le wiring `iroh_runtime.rs` (prévu plan §5.2) n'est PAS dans le diff. PATTERNS.md §19.1 le documente comme "Enforcement → Sprint 20+ wiring". Voir finding **P2-1**.

### Tests-delta

- Annoncé : +33 (+32 unit + 1 doc-test)
- Mesuré :
  - `pow.rs` : 14 `#[test]` ✓
  - `relay_pow_policy.rs` : 6 `#[test]` ✓
  - `pow_gossip.rs` : 12 `#[test]` ✓
  - doc-test `pow.rs` ligne 60 (`no_run`) : 6 doc-tests post-B vs 5 pre-B = +1 ✓
  - Total avant Phase B (commit `ab6985c`) : **487**
  - Total après Phase B (staged) : **520**
  - Delta mesuré : **+33** ✓
- 0 fail, 0 ignored sur `cargo test --workspace --locked`. **PASS**.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` : 0 warning. **PASS**.
- `cargo fmt --all --check` : clean. **PASS**.

### Research-grounding

**Deps nouvelles ou bumpées dans ce diff :**

| Dep | Statut | Trace research |
|---|---|---|
| `sha2 = "0.10"` (nouveau workspace) | Nouvelle | Plan §3.1 mentionne "Hashcash SHA256 Rust (WebSearch)" + kickoff §4 D2 |
| `criterion = "0.5"` (nouveau workspace) | Nouvelle | Cargo.toml inline justification "0.5 is the stable line as of 2026-04" |
| `dashmap`, `hex`, `toml` | Workspace pre-existant | Pre-existing, **PASS** |

APIs crypto / specs standardisées : SHA256 Hashcash, JCS (RFC 8785). Références kickoff §4 D2 "Tor PoW rendez-vous point 2023, Lightning Network invoice PoW, RFC 6110 Hashcash anti-spam". **PASS**.

### Horizon long-terme §6.7

- SHA256 vs BLAKE3 : choisi pour audit clarity S29 Cure53/ToB (Bitcoin / Tor rend-point PoW 2023 / Lightning all SHA256) — **rationale documenté** dans `pow.rs:32-41` module doc. **PASS**.
- `publisher_pubkey` field présent → **forward-compat PQC** documenté (PATTERNS.md §19.2). **PASS**.
- Design doc `.planning/research/S19_phase_B_pow_hashcash_design.md` (42 KB) présent — staging requis pour P2-3.

---

## Findings

- **P2-1** — Scope réduction non annoncée dans la PR/commit : le plan §5.2 liste `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` comme fichier Phase B. Le diff ne contient pas ce fichier. PATTERNS.md §19.1 documente "Enforcement → Sprint 20+ wiring" mais le commit body devrait mentionner explicitement la déviation. **Mitigation appliquée** : commit body enrichi section "Déviation vs plan §5.2".

- **P2-2** — Inexactitude documentation research-grounding : plan §3.1 et kickoff §4 D2 affirment "sha2 crate (deja dep via nexus-core-rs)" — factuellement faux. **Reporté Phase F wrap-up** (errata dans §Research consulté). Pas de code change.

- **P2-3** — Design doc non stagé : `.planning/research/S19_phase_B_pow_hashcash_design.md` existe (untracked, 42 KB, contenu substantiel) mais pas stagé. **Mitigation appliquée** : design doc staged dans ce commit.

- **P3-1** — `criterion 0.5` : commentaire Cargo.toml justifie le non-upgrade à 0.6 ("new proc-macro API") mais sans trace context7 dédiée. Risque faible (dev-dep only). **Reporté Phase F** : ajouter note context7 ou accepter la justification inline.

- **P3-2** — Biais entropie nonce : `solve()` itère `for nonce in 0u64..` depuis 0 déterministement (intentionnel, reproductibilité bench, documenté `pow.rs:352-356`). Pas un problème sécurité (canonical bytes diffèrent par topic/pubkey donc pre-images distincts) mais corrélation observable. **Reporté Phase F** : documenter comme limitation connue dans le module doc.

---

## Recommendation

**Commit autorisé.** 0 P0, 0 P1.

Mitigations intégrées dans ce commit :
1. Commit body enrichi avec déviation `iroh_runtime.rs` → S20+ (P2-1).
2. `S19_phase_B_pow_hashcash_design.md` staged (P2-3).

Reportés Phase F wrap-up :
- P2-2 : errata plan §3.1 / kickoff §4 D2 (remplacer "sha2 crate deja dep" par "sha2 crate introduite Phase B").
- P3-1 : trace context7 pour criterion 0.5 ou accepter justification inline.
- P3-2 : documenter limitation biais nonce dans module doc `pow.rs`.

## Commit metadata

- Range : 1 commit `feat(sprint19): Phase B — PoW Hashcash gossip subscribe (difficulty 2^18 per-relai)`.
- Delta tests attendu / mesuré : +33 (+32 unit + 1 doc-test). Total 487 → 520.
- Suites vertes : `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo check -p nexus-core-rs --benches --locked`.
- Pas de modification Python / Vitest / Playwright — suites non-exécutées car hors périmètre ; Phase F les ré-exécutera.
