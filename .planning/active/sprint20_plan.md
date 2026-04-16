# Sprint 20 — Plan d'execution (encryption at rest + duress + panic wipe + PoW wire + structured output + canary auto-publish + dual-transport)

**Ecrit** : 2026-04-16 (meme commit `chore(planning)` que
`sprint20_kickoff.md` + `sprint20_design_review.md` + `sprint20_
carry_summary.md`).
**Tip master d'entree** : `3a7f0a3` (post-S19 audit gate leve
via `1af90b3..3a7f0a3`).

---

## 1. Etat verifie a l'entree

### 1.1 Commit stack context

```
3a7f0a3 test(sprint19): audit-P2 A-3 — probe_and_cache quorum majority continues to dial  ← TIP COURANT
1af90b3 chore(sprint19): audit-P2 batch — claim fix + PATTERNS callouts + ruff + F-review migrate
bf7dd62 chore(sprint19): resolve Phase F placeholders to real SHA `619059b`
619059b chore(sprint19): Phase F — wrap-up + verification + audit plan S20 + migrate planning
...
```

### 1.2 Compteurs de tests observes

| Suite | Count | Verification |
|---|---|---|
| Rust workspace | 538 | `cargo test --workspace --locked` → `test result: ok` somme |
| Python SDK | 185 | `uv run pytest packages/nexus-sdk/tests/ -q` |
| Python coord | 208 + 3 skipped | `uv run pytest packages/nexus-coordinator/tests/ -q` |
| Python app-gov | 46 | `uv run pytest packages/nexus-app-gov/tests/ -q` |
| Vitest unit | 239 | `cd web && npm run test:unit` |
| Playwright | 38 | `cd web && npx playwright test` |
| size-limit | 7/7 | `cd web && npm run size` |
| SPDX | 246+ | SPDX headers grep cumulatif workspace |
| **Total** | **~1260** | |

### 1.3 Verification lint/format entree

```bash
cargo fmt --all --check          # clean
cargo clippy --workspace --all-targets --locked -- -D warnings  # clean
uv run ruff format --check packages/  # clean (post-D-2 fix)
uv run ruff check packages/      # clean
```

---

## 2. Decisions Day 0 (gelees — cf. kickoff §4)

| D | Decision | Implications code |
|---|---|---|
| **D1** | Double-layer Ed25519 wrap : Argon2id(PIN)+AES-256-GCM + OS keyring wrap KEK defense-en-profondeur | `keystore.rs` trait + `LocalFileKeyStore` impl + bump deps `argon2`/`aws-lc-rs`/`keyring` |
| **D2** | Argon2id m=64 MiB, t=3, p=1 (calibre ~3s/attempt) | constants `ARGON2_MEM_COST/TIME_COST/PARALLELISM` + bench Phase A |
| **D3** | Duress PIN = fake keypair noop responses (pas wipe immediate), panic wipe = gesture 5-tap separe | `KeyStore::unlock` differential Normal/Duress, `<node_id>_duress.enc` fake keypair |
| **D4** | Structured output via `llguidance` (Microsoft, Rust, 50µs/token) | `llama-cpp-2` feature `llguidance` + `-DLLAMA_LLGUIDANCE=ON` + schema JSON task response |
| **D5** | Cap G7 = 2/2 (Meta-1 Radicle + gitignore), PoW wire scope S20 Phase C, TLS wire T20 tech debt, DHT strict post-Gate-2 | voir `sprint20_carry_summary.md` |

**Ne pas rebattre** — figees kickoff S20 §4. Design Review Board G1
acknowledge dans kickoff §4 « Acknowledged review findings » (D3 ⚠️
adjust pre-gel, D1 precision adjust, autres noted no action).

---

## 3. Research consulte

### 3.1 Pre-plan research (2026-04-16)

- **Signal Secure Value Recovery** (blog signal.org/blog/secure-
  value-recovery/) : Argon2id + MAC entanglement, ~3s/attempt
  calibration, Shared Preferences encryption key wrap pattern.
- **Sygnia DPAPI downfall 2024** (sygnia.co blog) : master keys
  LSASS extractibles via Mimikatz `/unprotect`, **user-scope
  DPAPI = protection nulle vs same-user malicious process**.
- **SpecterOps DPAPI abuse guide 2024-2026** : operational
  guidance offensive user DPAPI — same-user process can extract
  DPAPI master keys directly from LSASS memory.
- **RFC 9106 Argon2** : Argon2id recommandation principale
  (side-channel resistant + side-channel timing-attack
  resistant).
- **OWASP Password Storage Cheat Sheet 2024-2025** : minimum
  m=19 MiB, t=2, p=1 pour passphrase. PIN court requires bump
  (>= 64 MiB).
- **`docs.rs/keyring`** (via context7
  `/websites/rs_keyring_keyring`) : API trait + cross-platform
  Linux/Windows/macOS/iOS/FreeBSD/OpenBSD support, keyutils
  persistent Linux hybrid.
- **GrapheneOS features + forum 2024-2026** : duress PIN pattern
  saisi anywhere credential demande → wipe hardware keystore +
  eSIM + forced shutdown irreversible. Hidden volume **detectable
  via forensics** (Passware Kit 2025 VeraCrypt 1.26.15).
- **`docs.rs/llguidance` 0.7+** : crate Microsoft, Rust, Lark-like
  grammar + JSON Schema native, 50 µs/token.
- **llama.cpp/docs/llguidance.md** : `-DLLAMA_LLGUIDANCE=ON` cmake
  flag + ExternalProject_Add cargo fetch/build.
- **arxiv 2501.10868 JSONSchemaBench** (jan 2025) : 6 SOTA
  frameworks comparison, llguidance p99 0.5 ms, GBNF native
  slower, XGrammar pas llama.cpp.
- **`aws-lc-rs` HPKE RFC 9180** : FIPS 140-3 validated, aligne
  VALIDATED_BLUEPRINT S17 preference FIPS track.
- **`cargo` registry local** : `rustsec/advisory-db` cross-check
  keyring-rs + argon2 + aws-lc-rs advisories — 0 hit actif.
- **RustSec Advisory Database** : cross-check 2026-04-16, no
  active advisory on keyring, aws-lc-rs, argon2, zeroize, secrecy.
- **`aes-gcm = "0.10"`** (RustCrypto) — substitut pour `aws-lc-rs`
  sur Windows dev : `aws-lc-sys` requires NASM au build (via le
  crate `jobserver` → `cc` → `aws-lc-sys::build_script`), et
  l'absence de NASM sur une machine Windows standard bloque le
  workspace build. Algorithme AES-256-GCM byte-identique (RFC 5116
  AEAD, RFC 5297), audit surface equivalente. Adopte par `age`,
  `openmls`, `signal-rs` en production. RustSec advisory-db
  cross-check 2026-04-16 : 0 finding actif sur `aes-gcm` ou sa
  dependance `aes` / `ghash`. Migration future vers `aws-lc-rs`
  (pour un build FIPS 140-3 reellement valide) = one-file swap au
  site `build_aead_key` / `seal_in_place` / `open_in_place` dans
  `crates/nexus-core-rs/src/keystore.rs`, trace tech debt T25
  PATTERNS.md §Sprint 20.1.

### 3.2 Code registry local

- `crates/nexus-core-rs/src/canonical.rs` : JCS canonical bytes
  pattern reutilise pour Argon2id input canonical encoding
  (domain tag `DOMAIN_KEYSTORE_V1`).
- `crates/nexus-shell-daemon-core/src/auth.rs:421-607` : pattern
  `TokenRotator` + `notify` file-watcher reutilise Phase A
  `<node_id>.enc` hot-reload (file-watcher pattern S16
  ConsentWatcher + S18 TokenRotator + S19 PinValidator).
- `crates/nexus-core-rs/src/pow.rs` + `pow_gossip.rs` :
  primitive PoW livree S19 `edfc51b` + `08f4e41`, ~660 LOC +
  32 tests, ready-to-wire Phase C.
- `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` : point de
  wire `GossipClient::subscribe_with_pow` — grep `TODO(S20)`
  Phase C.
- `crates/nexus-launcher/src/main.rs` : point d'injection `sbfb
  init --pin` + `sbfb unlock` CLI Phase A.

### 3.3 Research a faire session fraiche (par phase)

- **Phase A** : verifier `keyring = "3.6"` version courante via
  context7 + cargo registry, cross-check advisory-db refresh au
  jour du kickoff Phase A.
- **Phase A** : bench `cargo bench --bench keystore` sur hardware
  target (RTX 5080 dev + Raspberry Pi 4 low-end) confirmer
  `derive_kek` wall-clock < 5 s. Ajuster m=32 MiB si Pi 4 trop
  lent (D2 acknowledge).
- **Phase C** : confirmer signature `GossipClient::subscribe` iroh
  0.97 au wire point (risque breaking change mineur vs S19
  primitive).
- **Phase D** : verifier `llama-cpp-2` version courante + feature
  `llguidance` disponible (crate 0.1.143+ au moment research,
  possibles bumps).
- **Phase E warrant canary** : verifier ce que S18 E2 `04c9621` a
  deja livre (publish OR scheduler), eviter duplicate. Grep
  `canary-monthly.yml` + `gossip_topic "nexus-grid/warrant-canary
  /v1"`.

---

## 4. Phase A — Encryption at rest keypair (big rock) (+25 tests)

### 4.1 Scope

Livre le **double layer encryption at rest** pour le keypair
Ed25519 identity du daemon SBFB :

1. trait `KeyStore` abstrait (prep pour impls hardware S22+)
2. `LocalFileKeyStore` impl double layer :
   - AES-256-GCM (`aws-lc-rs`) wrap Ed25519 privee
   - KEK = Argon2id(PIN + 16-byte random salt, m=64 MiB, t=3, p=1)
   - OS keyring (`keyring-rs`) stocke un wrap supplementaire de
     la KEK (defense-en-profondeur vs same-user process via
     Argon2id(PIN) fallback)
3. Format blob v1 `~/.sbfb/keyring/<node_id>.enc` (cf. kickoff
   §D1)
4. Memory protection : `secrecy::SecretBox<[u8; 32]>` + `zeroize`
   impls
5. Bench `cargo bench --bench keystore` confirmer KEK derive < 5 s

### 4.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-core-rs/src/keystore.rs` (nouveau) | trait `KeyStore` + `LocalFileKeyStore` + `derive_kek` + blob v1 encode/decode + canonical bytes `DOMAIN_KEYSTORE_V1` |
| `crates/nexus-core-rs/src/lib.rs` | module + re-exports publics `KeyStore`, `LocalFileKeyStore`, `KeyStoreError` |
| `crates/nexus-core-rs/Cargo.toml` | deps `argon2 = "0.5"`, `aws-lc-rs = "1.18"`, `secrecy = "0.10"`, `keyring = "3.6"`, bump existing `zeroize = "1.8"` (confirmer via context7 Phase A) |
| `crates/nexus-core-rs/benches/keystore.rs` (nouveau) | criterion bench `derive_kek` + `encrypt_identity` + `decrypt_identity` |
| `crates/nexus-launcher/src/main.rs` | CLI `sbfb init --pin <pin>` + `sbfb unlock --pin <pin>` flows interactifs |
| `crates/nexus-launcher/src/unlock.rs` (nouveau) | flow interactive PIN prompt (ne jamais log PIN, zeroize buffer apres use) |
| `crates/nexus-shell-daemon/src/runtime.rs` | bootstrap daemon consume `Identity` from `KeyStore::unlock(pin)` plutot que `load_or_generate_keypair()` direct |
| `docs/rust/PATTERNS.md` | section `§Sprint 20.1 Encryption at rest double layer` + `§T-keystore-bench-reference` (numeros wall-clock reproductible) |
| Tests : `keystore.rs` module + integration `encryption_roundtrip` + `encryption_wrong_pin_rejected` + `encryption_replay_different_salt` + `encryption_version_bump_fallback_reject` (rest-at-rest `*_VERSION` v1 pre-launch + failure case quand blob v2 presente) | 25 tests |

### 4.3 Structure `keystore.rs` (esquisse)

```rust
// SPDX-License-Identifier: AGPL-3.0-or-later
//! Encryption at rest pour le keypair Ed25519 d'identite daemon.
//!
//! Double layer :
//! 1. AES-256-GCM wrap la privee avec KEK = Argon2id(PIN + salt)
//! 2. OS keyring (`keyring-rs`) stocke un wrap additionnel de la
//!    KEK (defense-en-profondeur vs DPAPI/Keychain/Secret Service
//!    user-scope gap documente Sygnia 2024 + SpecterOps 2026).

use secrecy::{ExposeSecret, SecretBox};
use zeroize::Zeroize;

pub trait KeyStore: Send + Sync {
    fn init(&self, pin: &str, duress_pin: Option<&str>) -> Result<(), KeyStoreError>;
    fn unlock(&self, pin: &str) -> Result<Identity, UnlockError>;
    fn rotate_pin(&self, old: &str, new: &str) -> Result<(), KeyStoreError>;
    fn wipe(&self) -> Result<(), KeyStoreError>;
}

pub struct Identity {
    pub keypair: SecretBox<ed25519_dalek::SigningKey>,
    pub mode: IdentityMode,
}

pub enum IdentityMode {
    Normal,
    Duress,  // cf. Phase B
}

pub struct LocalFileKeyStore {
    data_dir: PathBuf,
    keyring_service: String,
}

impl KeyStore for LocalFileKeyStore {
    // init(): generate Ed25519, derive KEK, encrypt, write blob,
    // optionally write duress blob (Phase B), store KEK wrap in
    // OS keyring via `keyring::Entry`
    // unlock(): read blob, derive KEK from PIN, decrypt, return
    // Identity{mode: match blob selected}
    // rotate_pin(): unlock → generate new salt → re-derive KEK →
    // re-encrypt → atomic rename
    // wipe(): zeroize RAM keypair + secure-unlink blob files +
    // delete OS keyring entry
}

const DOMAIN_KEYSTORE_V1: &[u8] = b"sbfb-keystore-v1";
const ARGON2_MEM_COST: u32 = 64 * 1024; // 64 MiB in KiB
const ARGON2_TIME_COST: u32 = 3;
const ARGON2_PARALLELISM: u32 = 1;
```

### 4.4 Tests plan (25 tests)

Primitive (14 tests dans `keystore.rs` module) :

1. `derive_kek_deterministic_same_pin_same_salt` : Argon2id
   deterministic
2. `derive_kek_different_salt_different_kek` : salt matters
3. `derive_kek_different_pin_different_kek` : PIN matters
4. `blob_v1_encode_decode_roundtrip` : format stable
5. `blob_v1_magic_bytes_rejected_if_wrong` : "SBFBK1" check
6. `blob_v1_version_mismatch_rejected` : reject v2 at v1 parser
7. `encrypt_identity_wrong_kek_aead_fails` : wrong KEK → AEAD
   reject
8. `unlock_wrong_pin_returns_unlock_error` : PIN incorrect
9. `unlock_with_correct_pin_returns_identity` : happy path
10. `zeroize_drops_plaintext_key_in_memory` : zeroize active
11. `canonical_bytes_include_domain_tag` : `DOMAIN_KEYSTORE_V1`
12. `rotate_pin_invalidates_old_pin` : old PIN fails post-rotate
13. `rotate_pin_preserves_same_keypair` : seulement wrap change
14. `wipe_removes_blob_and_keyring_entry` : fichier + OS keyring
    absent post-wipe

Integration (8 tests dans `crates/nexus-core-rs/tests/keystore_
integration.rs` nouveau) :

15. `init_creates_blob_and_keyring_entry` : post-init les 2 existent
16. `init_idempotent_rejects_reinit` : prevent ecrasement
17. `unlock_after_restart_works` : persistence disque
18. `hot_reload_blob_rotation_watcher` : file-watcher detecte
    rotation
19. `concurrent_unlock_same_pin_safe` : thread-safety
20. `concurrent_unlock_different_pins_safe` : race pas de leak
21. `blob_corruption_fails_loud` : ciphertext bitflip → AEAD err
22. `keyring_entry_missing_falls_back_to_blob_only` : degraded
    mode (single layer Argon2id si keyring indispo, warn log)

Bench (3 tests criterion) :

23. `bench_derive_kek_64_mib` : Argon2id wall-clock < 5 s
24. `bench_encrypt_decrypt_happy` : AEAD < 1 ms
25. `bench_unlock_total` : derive_kek + AEAD + keyring fetch
    < 6 s

### 4.5 Critere d'acceptation

- `cargo test -p nexus-core-rs keystore` → 22 tests vert
- `cargo test --test keystore_integration` → 8 tests vert
- `cargo bench --bench keystore` : `derive_kek` < 5 s, numero
  archive dans `docs/rust/PATTERNS.md §T-keystore-bench-reference`
  (evite regression T22-pattern S19 bench absent)
- `cargo clippy -p nexus-core-rs -p nexus-launcher -p nexus-shell-
  daemon-core --all-targets -- -D warnings` clean
- `sbfb init --pin 1234` + `sbfb unlock --pin 1234` CLI flow
  manuel testable
- Working tree audit G5 dans commit body obligatoire

### 4.6 Commit cible

```
feat(sprint20): Phase A — encryption at rest keypair (Argon2id + AES-256-GCM + double layer OS keyring)

Livre le big rock Gate 2 prerequis : Ed25519 identity keypair daemon
desormais chiffre au repos via double layer. Argon2id(PIN + salt)
derive KEK, AES-256-GCM wrap la privee, blob v1 au chemin
~/.sbfb/keyring/<node_id>.enc. OS keyring (keyring-rs) stocke un
wrap supplementaire de la KEK pour defense-en-profondeur vs
same-user process malicieux (threat model T3 DPAPI user-scope
bypass Sygnia 2024 + SpecterOps 2026).

Fichiers :
- crates/nexus-core-rs/src/keystore.rs (nouveau, trait + impl)
- crates/nexus-core-rs/src/lib.rs (exports)
- crates/nexus-core-rs/Cargo.toml (deps argon2, aws-lc-rs, secrecy, keyring)
- crates/nexus-core-rs/benches/keystore.rs (nouveau, criterion bench)
- crates/nexus-launcher/src/unlock.rs (nouveau, flow CLI PIN)
- crates/nexus-launcher/src/main.rs (sbfb init/unlock commands)
- crates/nexus-shell-daemon/src/runtime.rs (consume Identity from KeyStore)
- docs/rust/PATTERNS.md (§Sprint 20.1 + §T-keystore-bench-reference)

Tests delta : +25 Rust (14 primitive + 8 integration + 3 bench).
Total Rust 538 → 563.

Design doc : .planning/research/S20_phase_A_encryption_at_rest_design.md
(alternatives HPKE/age/TPM considerees, rationale double layer,
parametrage Argon2id 64 MiB, bench reference CPU hardware 2026).

Closes HARDENING_ROADMAP §3 S20 item 1 (encryption at rest keypair).
Prerequis Gate 2 debloque A-S9 Checkpoint-seize risk.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

---

## 5. Phase B — Duress PIN + panic wipe (+15 tests)

### 5.1 Scope

Livre les **2 features defensives** qui reposent sur le keypair
Phase A :

1. **Duress PIN** : `sbfb init --duress-pin <pin2>` genere un
   fake Ed25519 keypair (jamais publie), stocke dans
   `<node_id>_duress.enc`. `KeyStore::unlock(pin)` retourne
   `Identity { mode: Duress }` quand PIN-duress entre. Daemon
   boot avec fake identity, noop responses.
2. **Panic wipe 5-tap** : shell shortcut `Ctrl+Shift+Alt+W`
   repete 5 fois dans 3 sec → zeroize RAM keypair +
   secure-unlink `<node_id>.enc` + `<node_id>_duress.enc` +
   state.sqlite + blob cache + `delete_credential()` OS keyring
   entry + forced exit.

### 5.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-core-rs/src/keystore.rs` | extension `init_duress`, `unlock_differential`, `wipe_all` |
| `crates/nexus-shell-daemon/src/noop_identity.rs` (nouveau) | routes `IrohNodeContext` avec fake endpoint pour mode Duress, noop gossip publish/subscribe |
| `web/src/components/PanicWipeKeybind.tsx` (nouveau) | React useEffect listener `window.addEventListener('keydown')` pour Ctrl+Shift+Alt+W detection 5-tap 3s window |
| `web/src/lib/daemon.ts` | nouveau endpoint `POST /panic/wipe` (loopback uniquement, peer creds required) |
| `crates/nexus-shell-daemon/src/api/panic.rs` (nouveau) | endpoint `/panic/wipe` : bind to `PanicWipeService` |
| `crates/nexus-shell-daemon/src/panic.rs` (nouveau) | `PanicWipeService::execute()` : zeroize + unlink + exit |
| `docs/security/DURESS.md` (nouveau) | threat model duress + legal warning + operator guide |
| Tests : duress unlock + duress noop + panic wipe integration + 5-tap keybind vitest | 15 tests |

### 5.3 Tests plan (15 tests)

Primitive duress (6 tests `keystore.rs`) :

1. `init_duress_creates_two_blobs` : `<node_id>.enc` +
   `<node_id>_duress.enc`
2. `unlock_normal_pin_returns_normal_identity`
3. `unlock_duress_pin_returns_duress_identity`
4. `unlock_wrong_pin_rejected_even_with_duress_setup`
5. `duress_keypair_different_from_normal`
6. `duress_blob_indistinguishable_size_from_normal` :
   ciphertext length identical (indistinguabilite)

Noop identity runtime (3 tests integration
`tests/duress_runtime.rs` nouveau) :

7. `daemon_boot_in_duress_mode_publishes_fake_curator_empty` :
   gossip emit visible mais empty
8. `daemon_boot_in_duress_mode_rejects_curator_subscribe_real`
   : signatures fake detectable localement mais pas cote peer
9. `daemon_boot_in_duress_mode_rejects_task_dispatch` : noop

Panic wipe (4 tests integration
`tests/panic_wipe_e2e.rs` nouveau) :

10. `panic_wipe_removes_both_blobs`
11. `panic_wipe_zeroizes_keypair_ram`
12. `panic_wipe_deletes_state_sqlite_and_blob_cache`
13. `panic_wipe_exits_process`

Frontend 5-tap (2 tests Vitest `web/src/components/_tests/PanicW
ipeKeybind.test.tsx`) :

14. `five_taps_within_3s_triggers_wipe` : `POST /panic/wipe`
    called
15. `four_taps_or_slow_does_not_trigger` : debounce respect

### 5.4 Critere d'acceptation

- `cargo test -p nexus-core-rs keystore::tests::duress` +
  `cargo test --test duress_runtime` + `cargo test --test panic_
  wipe_e2e` → 13 tests vert
- `cd web && npm run test:unit -- PanicWipeKeybind` → 2 tests
  vert (241 total Vitest = 239 + 2)
- `docs/security/DURESS.md` documente legal warning + operator
  runbook (section §3 how-to-activate, §4 legal risks, §5
  recovery-is-not-possible)
- Working tree audit G5 obligatoire
- Scan `scan-en-strings.sh` clean (pattern panique UI en francais)

### 5.5 Commit cible

```
feat(sprint20): Phase B — duress PIN (fake keypair noop) + panic wipe 5-tap gesture

Livre 2 features defensives A-S9 Checkpoint-seize :

1. Duress PIN : `sbfb init --duress-pin <pin2>` genere un fake
   keypair (jamais publie reel). Unlock avec ce PIN boote le
   daemon en mode Duress : fake identity gossip publish, noop
   responses task/curator. Indistinguable cote peer observer
   (pattern deniable par construction, vs GrapheneOS wipe-immediate
   qui est tell-tale).

2. Panic wipe 5-tap : shell Ctrl+Shift+Alt+W x5 en 3s → zeroize
   RAM + secure-unlink blobs + delete OS keyring entry + forced
   exit. Irreversible. Documente legal warning (obstruction
   evidence possible selon juridiction) dans docs/security/
   DURESS.md.

Fichiers :
- crates/nexus-core-rs/src/keystore.rs (extensions init_duress +
  unlock_differential + wipe_all)
- crates/nexus-shell-daemon/src/noop_identity.rs (nouveau)
- crates/nexus-shell-daemon/src/panic.rs (nouveau)
- crates/nexus-shell-daemon/src/api/panic.rs (nouveau endpoint)
- web/src/components/PanicWipeKeybind.tsx (nouveau keybind React)
- web/src/lib/daemon.ts (POST /panic/wipe)
- docs/security/DURESS.md (nouveau, threat + legal + runbook)

Tests delta : +13 Rust + 2 Vitest. Total Rust 563 → 576, Vitest
239 → 241.

Design doc : .planning/research/S20_phase_B_duress_panic_design.md
(rationale fake-keypair vs wipe-immediate, indistinguabilite
wire, 5-tap gesture ergonomie, legal implications).

Closes HARDENING_ROADMAP §3 S20 items 2+3 (duress PIN + panic
wipe).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

---

## 6. Phase C — PoW runtime wire gossip subscribe (carry S19 integre) (+10 tests)

### 6.1 Scope

Execute la promesse annoncee dans le commit body S19 `edfc51b`
« integration intentionnellement differee Sprint 20+ ». Wire
`subscribe_with_pow` au path `crates/nexus-shell-daemon-core/src/
iroh_runtime.rs::GossipClient::subscribe`, charge
`~/.sbfb/relay_pow_policy.toml`, enforce per-topic difficulty,
fail-close si proof invalide.

### 6.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` | remplace `gossip.subscribe(topic)` par `gossip.subscribe_with_pow(topic, policy.difficulty_for(topic))` |
| `crates/nexus-shell-daemon-core/src/pow_policy_loader.rs` (nouveau) | loader + hot-reload file-watcher `~/.sbfb/relay_pow_policy.toml` (pattern ConsentWatcher S16 + TokenRotator S18 + PinValidator S19) |
| `crates/nexus-shell-daemon-core/src/browse.rs` | subscribe paths browse/curator pass through `subscribe_with_pow` aussi |
| Tests integration `crates/nexus-shell-daemon-core/tests/pow_wire.rs` (nouveau) | 10 tests : happy subscribe + reject invalid proof + policy reload + fall-back default if policy absent + per-topic override |

### 6.3 Tests plan (10 tests)

1. `subscribe_with_valid_pow_proof_accepts`
2. `subscribe_with_invalid_pow_proof_rejects`
3. `subscribe_with_expired_pow_proof_rejects` (TTL 30 min S19)
4. `subscribe_without_policy_falls_back_default_2^18`
5. `subscribe_with_per_topic_override_applied`
6. `policy_hot_reload_on_file_change_detected`
7. `policy_malformed_toml_keeps_previous_policy` (pattern
   TokenRotator S18 D-1)
8. `browse_subscribe_passes_through_pow_wire`
9. `curator_subscribe_passes_through_pow_wire`
10. `concurrent_subscribers_no_proof_contention`

### 6.4 Critere d'acceptation

- `cargo test --test pow_wire` → 10 tests vert
- Grep `gossip\.subscribe\(` dans `crates/nexus-shell-daemon-core/
  src/**/*.rs` retourne 0 match hors `subscribe_with_pow` path
  (= full wire, pas bypass)
- Doc `docs/rust/PATTERNS.md §Sprint 20.3 PoW gossip wire
  runtime`

### 6.5 Commit cible

```
feat(sprint20): Phase C — PoW runtime wire gossip subscribe (scope integre carry S19 A-2)

Execute la promesse commit body S19 edfc51b : subscribe_with_pow
wire au path runtime GossipClient::subscribe dans
nexus-shell-daemon-core::iroh_runtime.rs. Primitive S19 (32 tests
inchanges) desormais runtime-active — tous les paths subscribe
(curator list + browse aggregator + task dispatch gossip) passent
par le PoW gate.

Fichiers :
- crates/nexus-shell-daemon-core/src/iroh_runtime.rs (wire subscribe)
- crates/nexus-shell-daemon-core/src/pow_policy_loader.rs (nouveau,
  hot-reload pattern)
- crates/nexus-shell-daemon-core/src/browse.rs (passe subscribe par pow wire)

Tests delta : +10 Rust integration. Total Rust 576 → 586.

Closes carry S19 A-2 (reclassifie scope S20 Phase C). Debloque S21
rate-limit per-(consumer, worker, model) qui dependait Sybil-
resistance minimale via PoW.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

---

## 7. Phase D — Structured output llguidance (+12 tests)

### 7.1 Scope

Integration `llguidance` crate Microsoft via feature flag
`llama-cpp-2` + build flag `-DLLAMA_LLGUIDANCE=ON`. Enforcement
schema JSON `task_response.schema.json` coord-side pour que les
workers LLM produisent toujours valid-schema responses avant
signature.

### 7.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-worker-core/Cargo.toml` | deps `llguidance = "0.7"` optional + `llama-cpp-2` avec feature `llguidance` |
| `crates/nexus-worker-core/src/llm.rs` | wire `llguidance::Constraint` dans `generate(prompt, grammar)` |
| `crates/nexus-core-rs/src/schemas/task_response.schema.json` (nouveau) | source-of-truth JSON Schema draft-07 |
| `crates/nexus-core-rs/src/schemas/mod.rs` (nouveau) | embed + parse schema + expose Lark-like via llguidance JSON Schema support |
| `docs/rust/PATTERNS.md` | section `§P30 structured output llama.cpp llguidance` + **warning prominent : « grammar != prompt injection defense »** (audited_findings HARDENING_ROADMAP note) |
| Tests : primitive schema encode + llguidance integration + worker generate E2E | 12 tests |

### 7.3 Tests plan (12 tests)

1. `schema_parses_as_valid_json_draft_07`
2. `schema_includes_domain_tag_task_response_v1`
3. `llguidance_constraint_builds_from_schema`
4. `generate_valid_response_passes_schema`
5. `generate_forced_structure_prevents_free_text`
6. `generate_p99_latency_under_10ms_on_rtx5080` (performance
   guard)
7. `worker_signs_response_only_after_schema_validation`
8. `worker_rejects_llm_output_that_bypasses_grammar`
   (defense-in-depth : meme si llguidance fail silently, validator
   final check)
9. `schema_version_bump_forward_reject` (pre-launch `v=1` only)
10. `grammar_prompt_injection_does_not_bypass_output_format` :
    test explicite qu'un prompt injection dans la user query
    N'ARRIVE PAS a changer le format (mais PEUT influencer le
    contenu = point warning doc)
11. `bench_grammar_overhead_under_1_percent` : llguidance 50
    µs/token ~ <1% vs token decode 1-10 ms
12. `schema_roundtrip_coord_worker` : worker produit, coord
    parse sans erreur

### 7.4 Critere d'acceptation

- `cargo test -p nexus-worker-core llm::tests::grammar` → 12 tests
  vert
- llama.cpp rebuild local `-DLLAMA_LLGUIDANCE=ON` documente dans
  `docs/shell/PATTERNS.md §P30` operator runbook
- Warning `docs/rust/PATTERNS.md §P30` : « structured output
  grammar n'est PAS une defense anti prompt injection (cf.
  HARDENING_ROADMAP audited_findings S19 2026-04-16) » prominent
  en haut de section

### 7.5 Commit cible

```
feat(sprint20): Phase D — structured output llama.cpp llguidance grammar (JSON schema enforce)

Integre llguidance crate Microsoft (Rust, 50 us/token, Lark-like
grammar + JSON Schema native) au worker LLM path via
llama-cpp-2 feature `llguidance` + llama.cpp build flag
-DLLAMA_LLGUIDANCE=ON. Les responses workers sont desormais
force-format valid-schema avant signature, eliminant la classe
"LLM garbled JSON → signature-fail chain break".

Rejet XGrammar (pas llama.cpp support), Outlines (Python IPC
brise Option G), GBNF natif (slower + pas Rust native).

Warning explicite PATTERNS.md §P30 : structured output grammar
n'est PAS une defense anti prompt injection. Grammar force le
format de sortie, pas le contenu. Prompt injection echappe
toujours a la grammar (cf. HARDENING_ROADMAP audited_findings
2026-04-16 "S21 grammar != prompt injection defense").

Fichiers :
- crates/nexus-worker-core/src/llm.rs (wire llguidance constraint)
- crates/nexus-core-rs/src/schemas/task_response.schema.json
- crates/nexus-core-rs/src/schemas/mod.rs
- docs/rust/PATTERNS.md (§P30 + warning prominent)

Tests delta : +12 Rust worker. Total Rust 586 → 598.

Design doc : .planning/research/S20_phase_D_structured_output_design.md
(llguidance vs XGrammar vs Outlines vs GBNF comparison, schema
versioning strategy, performance budget).

Closes HARDENING_ROADMAP §3 S20 item 4 (structured output).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

---

## 8. Phase E — Warrant canary auto-publish + dual-transport WSS TCP 443 fallback (+8 tests)

### 8.1 Scope

2 features infrastructure network :

1. **Warrant canary auto-publish scheduler** : au-dessus du
   publish manuel S18 E2 `04c9621`. Scheduler coord-side qui
   signe + publie un canary heartbeat mensuel automatiquement.
   **Verifier au kickoff Phase E si S18 a deja le scheduler**
   (risque duplicate). Grep `canary-monthly.yml`, `cron`,
   `scheduler.*canary`.
2. **Dual-transport detection + WSS TCP 443 fallback** : au boot
   daemon, probe UDP QUIC (3x dial in 10s) ; si all fail,
   bascule `RelayMode::Custom` avec `relay_wss_only = true`
   (force WebSocket over TCP 443). Log warn observable ops.
   Defense baseline anti-DPI ISP (full Tor/Snowflake = S24-25).

### 8.2 Fichiers touches

| Fichier | Role |
|---|---|
| `packages/nexus-coordinator/src/nexus_coordinator/canary_scheduler.py` (nouveau OU extension S18 E2 selon check) | scheduler monthly cron-like, signe Ed25519 + JCS, publie gossip topic `nexus-grid/warrant-canary/v1` |
| `packages/nexus-coordinator/src/nexus_coordinator/coordinator.py` | boot hook canary_scheduler start/stop |
| `crates/nexus-shell-daemon-core/src/transport_probe.rs` (nouveau) | UDP QUIC probe 3x 10s + fallback detection |
| `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` | utilise transport_probe au boot, configure `RelayMode` |
| Tests integration `packages/nexus-coordinator/tests/test_canary_scheduler.py` + `crates/nexus-shell-daemon-core/tests/transport_fallback.rs` | 8 tests |

### 8.3 Tests plan (8 tests)

Warrant canary scheduler (4 tests pytest) :

1. `scheduler_signs_and_publishes_monthly` : freeze time, advance
   30 days, verify publish call
2. `scheduler_skips_if_last_publish_under_30_days`
3. `scheduler_signs_with_maintainer_key_from_keyring` : cross-ref
   Phase A
4. `scheduler_logs_warn_if_publish_fails_but_retries_24h`

Transport probe + fallback (4 tests Rust) :

5. `probe_udp_quic_success_uses_default_transport`
6. `probe_udp_quic_timeout_3x_forces_wss_fallback`
7. `fallback_wss_still_connects_via_tcp_443`
8. `probe_rerun_on_config_reload_detected`

### 8.4 Critere d'acceptation

- Check preliminaire S18 E2 : si scheduler deja livre, cette
  feature devient UPGRADE (non duplicate). Document decision dans
  commit body.
- Tests pytest + Rust integration verts
- Doc `docs/shell/PATTERNS.md §Warrant canary auto-publish` +
  `docs/rust/PATTERNS.md §Transport probe + WSS fallback`

### 8.5 Commit cible

```
feat(sprint20): Phase E — warrant canary auto-publish + dual-transport WSS TCP 443 fallback

2 features infrastructure :

1. Warrant canary auto-publish scheduler (packages/nexus-
   coordinator/src/nexus_coordinator/canary_scheduler.py nouveau
   OR extension S18 04c9621 selon check). Automatise le publish
   mensuel + signature Ed25519+JCS + gossip topic.

2. Dual-transport probe : au boot, UDP QUIC probe 3x 10s ; si
   all fail, RelayMode::Custom relay_wss_only=true (WebSocket TCP
   443 fallback). Defense baseline anti-DPI ISP. Full Tor
   bridges + Snowflake-WebRTC reste hors scope S20 (cf. S24-25).

Fichiers :
- packages/nexus-coordinator/src/nexus_coordinator/canary_scheduler.py
- packages/nexus-coordinator/src/nexus_coordinator/coordinator.py
- crates/nexus-shell-daemon-core/src/transport_probe.rs
- crates/nexus-shell-daemon-core/src/iroh_runtime.rs

Tests delta : +4 coord + +4 Rust. Total 598 Rust → 602, coord
208+3 → 212+3.

Closes HARDENING_ROADMAP §3 S20 items 5+6 (canary auto-publish +
dual-transport WSS fallback).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

---

## 9. Phase F — Consolidation + verification + audit plan S21 (docs only)

### 9.1 Scope

- Update `CLAUDE.md §Etat actuel` : Sprint 20 CLOSED + Gate 2
  preparation + commits stack
- Update `docs/claude/SPRINT_LOG.md` : row S20 v1.2
- Update memory `nexus_grid_pivot.md` frontmatter
- `.planning/active/sprint20_verification.md` : fail-fast 32+ rows
- `.planning/active/sprint20_audit_plan.md` : tracks A-E + F-F +
  meta-track Radicle-v1.0 **re-carried S21**
- Migration planning `.planning/active/sprint20_*.md` +
  `.planning/active/sprint19_audit_findings.md` →
  `.planning/archive/v1.2/`

### 9.2 Commit cible

```
chore(sprint20): Phase F — wrap-up + verification + audit plan S21 + migrate planning
```

---

## 10. Fail-fast checklist (32 rows)

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | `git rev-parse --short HEAD` Phase F final | `git rev-parse --short HEAD` | 7-char SHA | — |
| 2 | Range S20 commits >= 7 | `git log --oneline 3a7f0a3..HEAD \| wc -l` | `>= 7` | — |
| 3 | `.planning/active/` vide post-F | `ls .planning/active/ \| wc -l` | `0` | — |
| 4 | `.planning/archive/v1.2/sprint20_*` >= 4 | `ls .planning/archive/v1.2/sprint20_*.md \| wc -l` | `>= 4` | — |
| 5 | Rust tests 538 → >= 602 | `cargo test --workspace --locked` somme | `>= 602` | — |
| 6 | `cargo fmt --all --check` silent | idem | exit 0 | — |
| 7 | `cargo clippy -D warnings` clean | idem | exit 0 | — |
| 8 | Python SDK 185 unchanged | idem | `185 passed` | — |
| 9 | Python coord 208 → >= 212 | idem | `>= 212 passed, 3 skipped` | — |
| 10 | Python app-gov 46 unchanged | idem | `46 passed` | — |
| 11 | `ruff format --check` clean | idem | exit 0 | — |
| 12 | `ruff check` clean | idem | exit 0 | — |
| 13 | Vitest 239 → >= 241 | idem | `>= 241 passed` | — |
| 14 | Playwright 38 unchanged | idem | `38 passed` | — |
| 15 | size-limit 7/7 | idem | all pass | — |
| 16 | Frontend build ok | idem | zero warnings | — |
| 17 | `scan-en-strings.sh` clean | idem | exit 0 | — |
| 18 | `keystore.rs` module present | `test -f crates/nexus-core-rs/src/keystore.rs` | exit 0 | — |
| 19 | `derive_kek` bench < 5s | `cargo bench --bench keystore 2>&1 \| grep "time:"` | `< 5s` | — |
| 20 | `DURESS.md` present | `test -f docs/security/DURESS.md` | exit 0 | — |
| 21 | `PanicWipeKeybind.tsx` present | `test -f web/src/components/PanicWipeKeybind.tsx` | exit 0 | — |
| 22 | PoW wire grep-verify (no bypass) | `grep -r "gossip\.subscribe\(" crates/nexus-shell-daemon-core/ \| grep -v "subscribe_with_pow\|test"` | 0 match | — |
| 23 | llguidance feature enabled | `grep "llguidance" crates/nexus-worker-core/Cargo.toml` | present | — |
| 24 | task_response.schema.json present | `test -f crates/nexus-core-rs/src/schemas/task_response.schema.json` | exit 0 | — |
| 25 | PATTERNS.md §P30 warning grammar != prompt injection | `grep "grammar.*prompt injection defense" docs/rust/PATTERNS.md` | >= 1 match | — |
| 26 | canary_scheduler present | `test -f packages/nexus-coordinator/src/nexus_coordinator/canary_scheduler.py` OR S18 integration confirm | exit 0 | — |
| 27 | transport_probe.rs present | `test -f crates/nexus-shell-daemon-core/src/transport_probe.rs` | exit 0 | — |
| 28 | Design doc S20 Phase A/B/C/D/E present | `ls .planning/research/S20_phase_*.md \| wc -l` | `>= 4` (sauf C = carry) | — |
| 29 | HARDENING_ROADMAP `last_validated` bumped (G2) | `grep "last_validated: 2026-04-\|last_validated: 2026-05" docs/security/HARDENING_ROADMAP.md` | present | — |
| 30 | `.gitignore` NOISE patterns ajoutees (P2-2 carry) | `grep "cc\.json\|test_libc\|docs/apps/" .gitignore` | >= 3 match | — |
| 31 | Cap G7 2/2 respecte verification Phase F | `grep "Cap G7 respecte : 2/2" .planning/active/sprint20_verification.md` | >= 1 match | — |
| 32 | Memory `nexus_grid_pivot.md` frontmatter tip sync | `grep "Tip \`" \~/.claude/.../nexus_grid_pivot.md \| grep -oE "[a-f0-9]+"` == HEAD_SHA | match | — |

Observed rempli en Phase F.

---

## 11. Git plan (commits ordonnes)

| # | Commit | Phase | SHA attendu |
|---|---|---|---|
| 1 | `chore(planning): open Sprint 20 — encryption at rest + duress + panic wipe + PoW wire + structured output + canary auto-publish + dual-transport` | Planning | post-`3a7f0a3` |
| 2 | `feat(sprint20): Phase A — encryption at rest keypair (Argon2id + AES-256-GCM + double layer OS keyring)` | A | — |
| 3 | `feat(sprint20): Phase B — duress PIN (fake keypair noop) + panic wipe 5-tap gesture` | B | — |
| 4 | `feat(sprint20): Phase C — PoW runtime wire gossip subscribe (scope integre carry S19 A-2)` | C | — |
| 5 | `feat(sprint20): Phase D — structured output llama.cpp llguidance grammar (JSON schema enforce)` | D | — |
| 6 | `feat(sprint20): Phase E — warrant canary auto-publish + dual-transport WSS TCP 443 fallback` | E | — |
| 7 | `chore(sprint20): Phase F — wrap-up + verification + audit plan S21 + migrate planning` | F | — |

7 commits S20 (1 planning + 5 feat + 1 wrap-up).

---

## 12. Scope cuts (repete pour accessibilite)

Cf. kickoff §8. En resume :

- Hardware keystore (TPM/SE/StrongBox) → S22+
- HPKE wrap-for-peer → S22+
- Rate-limit per-consumer → S21 (depend S20 Phase C)
- Client-side redaction SDK → S21
- Kudos-weighted admission → S22
- Tool-calling sandbox allow-list → S22
- Redundancy voting → S22
- Ephemeral workers + VRAM wipe → S23
- Honeypot Eclipse detection → S23
- DNS fallback DHT DoH/DoT → S24
- Arti Tor bridge → S25
- Domain fronting Snowflake-WebRTC → S25
- PQC migration ML-DSA + ML-KEM → S26+
- `actions/checkout@v4` pin SHA sweep → sprint ops futur

---

## 13. Risks (R1..R7) + mitigation

| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Phase A : Argon2id m=64 MiB trop lent sur Raspberry Pi 4 low-end (>5s UX inacceptable) | M | M | Bench Phase A obligatoire sur hardware target. Fallback m=32 MiB si >5s. Doc numero dans PATTERNS.md §T-keystore-bench-reference. |
| R2 | Phase A : `keyring-rs` version 3.6 API break vs research (crate versions evolue rapidement) | L | L | Check context7 au kickoff Phase A + Cargo.toml dependency resolution test. Fallback 3.5 acceptable. |
| R3 | Phase B : 5-tap gesture faux-positifs (user accidentel) | M | L | Window 3s + count precis 5 + log confirmation prompt modale OU simple (decision fail-safe au kickoff Phase B) |
| R4 | Phase B : mode Duress fuite via timing side-channel (decrypt Normal ~50µs vs Duress ~50µs blob meme taille, AES constant-time) | L | M | aws-lc-rs constant-time AES-GCM. Tests `duress_blob_indistinguishable_size_from_normal` verifie longueur ciphertext identique. |
| R5 | Phase C : iroh 0.97 API subscribe wire-point different que research S19 (breaking change mineur possible entre primitive livree et wire) | L | L | Confirm signature au kickoff Phase C. Fallback = update primitive PoW wrapper. |
| R6 | Phase D : llama-cpp-2 Rust bindings version bumps incompatibles ; `llguidance` feature nom change | L | M | Pin version exact dans Cargo.toml. Check context7 au kickoff Phase D. Fallback = construct llguidance via direct C FFI (ugly). |
| R7 | Phase E warrant canary : duplication S18 E2 `04c9621` (scheduler deja livre mais pas detecte) | M | L | Check Phase E kickoff : grep + lire `canary-monthly.yml` + module canary coord-side S18. Si scheduler deja livre, cette feature = UPGRADE (test coverage +, cron → systemd timer), documenter dans commit body. |

---

## 14. Checkpoint de cloture

Sprint 20 ferme quand :

1. 7 commits S20 landed (1 planning + 5 feat + 1 wrap-up)
2. 32/32 fail-fast checklist verte
3. `sprint20_verification.md` + `sprint20_audit_plan.md` ecrits
4. CLAUDE.md + SPRINT_LOG.md + memory `nexus_grid_pivot.md`
   updated post-Phase F (G6 carry-over manuel)
5. Planning files `sprint20_*.md` + `sprint19_audit_findings.md`
   migres `active/` → `archive/v1.2/`
6. Meta-1 Radicle-v1.0 tracking **re-carried S21** explicitement
   dans `sprint20_audit_plan.md §meta-track`
7. `.planning/active/` vide
8. Memory frontmatter description sync tip final
9. HARDENING_ROADMAP `last_validated` bump + audited_findings
   entry S20 open+close

---

**Note de placement** : ce plan est ecrit **meme commit** que
`sprint20_kickoff.md` + `sprint20_design_review.md` + `sprint20_
carry_summary.md`. Migrations S19 audit_findings staged pre-commit
via `git mv`.
