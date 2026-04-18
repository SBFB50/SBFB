# Sprint 20 Phase A — nexus-phase-auditor review

HEAD pre-commit: 1b1f9cb766434564cf877698f148615c5920e94c
Draft commit body: "feat(sprint20): Phase A — encryption at rest keypair (Argon2id + AES-256-GCM + double layer OS keyring)"
Timebox: 45m (initial) + 20m (post-fix re-audit)

## Verdict : PASS

(PASS = 0 P0/P1 ET >= 1 P2+ documente — rigor signal G4 satisfait)

Tous les P1 et P2 de la premiere passe ont ete resolus. 2 P3
carried over (non-bloquants, optionnels). Commit autorise.

---

## Dimensions (premiere passe — inchangees sauf post-fix)

### Security

- [x] **Pas de bloc `unsafe`** dans les 4 nouveaux fichiers.
- [x] **unwrap() production** : 3 appels dans `parse_blob` lignes
  860-862 (`try_into().unwrap()` sur slices `[u8;4]` fixes). Infaillibles
  par precondition statique (`blob.len() >= BLOB_HEADER_LEN + TAG_LEN`
  garantit que blob[24..28] etc. sont valides). Acceptable — commentaire
  inline absent (P3-3, carried over, optionnel).
- [x] **Loopback / PeerCreds** : le diff `runtime.rs` ne touche pas
  les routes HTTP ni le routeur. La modification est limitee au boot
  du endpoint iroh (lecture env var → `NodeConfig::with_secret_key`).
  Aucun nouveau chemin loopback ajoute. PeerCredsVerified non impacte.
- [x] **Wire format / JCS** : le blob est un format local (fichier
  `identity.enc`), pas un format wire reseau. Pas de serde_json sur
  ce path. La serialisation header est big-endian manuellement encode,
  pas JCS (legitimement — format binaire, pas JSON). OK.
- [x] **Zip path traversal** : aucun path extraction dans le diff.
- [x] **Secrets en clair** : `kek1` et `kek2` sont dans `SecretBox<[u8;32]>`.
  `final_kek` est aussi `SecretBox`. `secret_bytes` post-init est zeroized.
  Le `hex_str` dans `unlock.rs` est zeroized apres `set_var`. OK.
- [x] **PIN via arg CLI** : P2-5 resolu. `docs/rust/PATTERNS.md §T27`
  documente l'exposition shell history / ps avec reference Phase B
  rpassword. Tech debt formel present.
- [x] **SBFB_IDENTITY_SECRET_HEX env var** : visible dans
  `/proc/self/environ` pendant la fenetre spawn → daemon. Documente
  T24 dans PATTERNS.md §Sprint 20.2. La longueur d'exposition (entre
  `set_var` et `remove_var` dans le daemon child) est bornee au
  temps de startup du daemon. Acceptable comme tech debt documente.
- [x] **rotate_pin error mapping** : P2-1 resolu. `KeyStoreError::WrongPin`
  ajoute (`keystore.rs:193-200`). `rotate_pin()` mappe `UnlockError::
  AeadReject → KeyStoreError::WrongPin` (ligne 651-652). Semantique
  correcte.

### Patterns

- [x] **PATTERNS.md §Sprint 20.1** present et complet : double layer
  schema, threat model table, deviations D1 documentees (aes-gcm vs
  aws-lc-rs + `identity.enc` vs `<node_id>.enc`).
- [x] **PATTERNS.md §Sprint 20.2** present : key-handling discipline,
  SecretBox, zeroize RAII, T24 env var → UDS future.
- [x] **PATTERNS.md §T-keystore-bench-reference** present : bench
  reference 82 ms mesuree avec note calibration.
- [x] **T25 FIPS migration** : entree formelle `### T25` creee dans
  PATTERNS.md (lignes 1087-1108). Documente one-file swap derriere
  feature `fips`, cross-ref audit finding P2-6. Resolu.
- [x] **T26 Argon2id calibration gap** : entree formelle `### T26`
  creee dans PATTERNS.md (lignes 1110-1132). Documente 82ms vs target
  3s, fix path bump m=128 MiB ou t=6. Resolu.
- [x] **T27 PIN CLI exposure** : entree formelle `### T27` creee dans
  PATTERNS.md (lignes 1134-1151). Phase B promet rpassword interactif.
  Resolu.
- [x] **Pas de LOC estimees dans plan/kickoff** : verifie, aucune
  mention "LOC estimee" dans plan.md ou kickoff.md. HARDENING_ROADMAP
  a des tailles indicatives (~800 LOC etc.) mais ce sont des estimations
  de roadmap anterieures, pas dans le plan S20.
- [x] **Pre-launch protocol policy** respectee : BLOB_VERSION = 0x01,
  pas de decoder multi-version, commentaire inline sur la politique.

### Working tree audit (G5)

- [x] **PHASE** : 16 fichiers au total dans le diff (14 originaux
  + `sprint20_phase_A_review.md` + `sprint20_plan.md` updates).
  Les fichiers attendus par plan §4.2 sont tous presents. Le fichier
  review et le plan patche sont CRAFT mais staged ensemble car la
  phase inclut le cycle audit complet.
  - `crates/nexus-core-rs/src/keystore.rs` (NEW) ✓
  - `crates/nexus-core-rs/tests/keystore_integration.rs` (NEW) ✓
  - `crates/nexus-core-rs/benches/keystore.rs` (NEW) ✓
  - `crates/nexus-core-rs/src/lib.rs` ✓
  - `crates/nexus-core-rs/Cargo.toml` ✓
  - `crates/nexus-launcher/src/unlock.rs` (NEW) ✓
  - `crates/nexus-launcher/src/main.rs` ✓
  - `crates/nexus-launcher/Cargo.toml` ✓
  - `crates/nexus-shell-daemon/src/runtime.rs` ✓
  - `crates/nexus-shell-daemon/Cargo.toml` ✓
  - `crates/nexus-shell-daemon-core/src/browse.rs` (fmt collateral) ✓
  - `docs/rust/PATTERNS.md` ✓
  - `Cargo.toml` (workspace deps) ✓
  - `Cargo.lock` ✓
  - `.planning/active/sprint20_plan.md` (addendum §3.1 aes-gcm P1 fix) ✓
  - `.planning/active/sprint20_phase_A_review.md` (ce fichier) ✓
- [x] **CRAFT** : `sprint20_plan.md` et `sprint20_phase_A_review.md`
  sont des fichiers planning. Ils sont inclus dans le meme commit
  phase car le patch plan est le fix P1 mandatory et la review est
  le gate document. Pattern justifiable — un split commit separatif
  pour 2 lignes d'addendum plan aurait cree plus de bruit qu'il
  n'en aurait elimine.
- [x] **DEBT** : 0 scope cut touche. ✓
- [x] **NOISE** : `.claude/settings.local.json` et `.claude/worktrees/`
  sont untracked ET dans `.gitignore`. 0 NOISE stage. ✓
- [x] **Section "Working tree audit"** : confirmee presente dans le
  draft commit body fourni par l'executeur.

### Scope-cuts

Items scope-cut §12 grepes dans le diff :
- `tpm`, `strongbox`, `secure.enclave` : 0 match ✓
- `hpke` : 0 match (sauf Cargo.toml commentaire refus D1 expliquant
  pourquoi rejete) ✓
- `rate.limit` : 0 match ✓
- `pqc`, `ml.kem`, `ml.dsa` : 0 match ✓
- `tor.bridge`, `snowflake`, `domain.front` : 0 match ✓

**Aucun scope creep detecte.** ✓

### Tests-delta

- [x] **Annonce post-fix** : +28 Rust (15 prim + 8 int + 5 launcher)
  = 538 → 566 (le test `param_downgrade_attack_rejected` ajoute
  par fix P2-4 porte le primitif de 14 a 15).
- [x] **Compte dans le diff** :
  - `keystore.rs` : 15 fonctions `#[test]` (lignes 925, 935, 944,
    954, 969, 987, 1004, 1025, 1038, 1054, 1072, 1089, 1103, 1124,
    1147) ✓
  - `keystore_integration.rs` : 8 fonctions `#[test]` (attributs en
    debut de ligne, ligne 44 est un commentaire de doc non compte) ✓
  - `unlock.rs` : 5 fonctions `#[test]` ✓
  - Total : 28. Delta annonce correct. ✓
- [x] **Rust workspace total annonce** : 566. Conforme. ✓

### Research-grounding

- [x] **argon2 = "0.5"** : trace dans plan §3.1 (RFC 9106 + OWASP
  2024 + RustSec advisory-db clean). PASS ✓
- [x] **keyring = "3.6"** : trace dans plan §3.1 (context7
  `/websites/rs_keyring_keyring`, API trait, cross-platform). PASS ✓
- [x] **secrecy = "0.10"** : trace dans plan §3.1 (advisory-db clean).
  Nom explicite dans la liste des crates verifies. PASS ✓
- [x] **zeroize = "1.8"** : trace dans plan §3.1 (advisory-db clean +
  mention dans plan §4.2 `bump existing zeroize = "1.8"`). PASS ✓
- [x] **tempfile = "3.13"** : deja present comme dev-dep avant ce
  diff (hoisted to workspace). Version inchangee. PASS ✓
- [x] **aes-gcm = "0.10"** : **P1 resolu**. `sprint20_plan.md §3.1`
  contient maintenant une entree explicite (lignes 104-116) avec :
  rationale substitut aws-lc-rs (NASM Windows build constraint),
  algorithme byte-identique RFC 5116, cross-check RustSec advisory-db
  2026-04-16 (0 finding actif sur `aes-gcm` / `aes` / `ghash`),
  usage production (`age`, `openmls`, `signal-rs`), migration path
  T25 PATTERNS.md §Sprint 20.1. Trace conforme au standard project.
  PASS ✓

### Horizon long-terme + documentation amont

- [x] **Design doc present** : `sprint20_design_review.md` (G1 Board)
  preexistant au code. `sprint20_kickoff.md §D1..D5` documentent le
  design avec alternatives rejetees (OS keyring seul, `age`, HPKE,
  TPM, scrypt/PBKDF2/bcrypt). ✓
- [x] **Alternatives rejetees citees** : D1 liste 5 alternatives
  avec rationale detaille. D2 liste 4 alternatives KDF. ✓
- [x] **Solution la plus poussee** : double layer Argon2id + AES-256-GCM
  + OS keyring est le pattern Signal-grade documente dans RFC 9106
  + OWASP 2024. La deviation aes-gcm vs aws-lc-rs est acceptable
  pre-launch (algorithme identique, API compatible, path migration
  T25 documente). ✓
- [x] **Aucune LOC estimee** : plan.md/kickoff.md ne contiennent pas
  "LOC estimee". HARDENING_ROADMAP §3 a des tailles indicatives (~800
  LOC) qui sont des projections anterieures a S20, pas des estimations
  dans ce plan. ✓

---

## §Post-fix verification (re-audit 2026-04-16)

### Verification P1

**P1-Research (aes-gcm)** — RESOLU ✓

`sprint20_plan.md §3.1` lignes 104-116 contiennent maintenant une
entree `**aes-gcm = "0.10"**` (RustCrypto) avec :
- rationale NASM Windows build constraint (aws-lc-sys incompatible)
- algorithme byte-identique RFC 5116 + RFC 5297
- RustSec advisory-db cross-check 2026-04-16 : 0 finding actif sur
  `aes-gcm` / `aes` / `ghash`
- usages production references (`age`, `openmls`, `signal-rs`)
- migration path T25 PATTERNS.md §Sprint 20.1

La trace est conforme au standard project §4bis (nom lib + rationale
+ advisory check date). P1 leve.

### Verification P2

**P2-1 (rotate_pin WrongPin variant)** — RESOLU ✓

`keystore.rs:193-200` : `KeyStoreError::WrongPin` ajoute avec doc
inline expliquant la semantique (variant separe d'`Argon2Params`
pour que les appelants puissent pattern-matcher un evenement wrong-PIN
vs un parametre KDF invalide).

`keystore.rs:651-652` : `UnlockError::AeadReject` mappe correctement
sur `KeyStoreError::WrongPin`. Les autres variantes (`BlobMalformed`,
`Argon2Kdf`, `KeyringEntryMissing`, `Io`, `NotInitialized`) mappent
sur des variantes semantiquement distinctes. Mapping complet et
correct.

**P2-2 (SBFB_IDENTITY_SECRET_HEX_ENV single source of truth)** — RESOLU ✓

`nexus-core-rs/src/keystore.rs:150` : source unique `pub const
SBFB_IDENTITY_SECRET_HEX_ENV: &str = "SBFB_IDENTITY_SECRET_HEX"`.

`nexus-core-rs/src/lib.rs:75` : re-exporte via `pub use keystore::{...,
SBFB_IDENTITY_SECRET_HEX_ENV, ...}`.

`nexus-launcher/src/unlock.rs:43` : importe depuis `nexus_core_rs::
SBFB_IDENTITY_SECRET_HEX_ENV` (pas de constante locale). ✓

`nexus-shell-daemon/src/runtime.rs:63-65` : commentaire documentant
l'import partage + `use nexus_core_rs::SBFB_IDENTITY_SECRET_HEX_ENV`
(pas de constante locale). ✓

Divergence silencieuse impossible desormais.

**P2-3 (T26 Argon2id calibration gap)** — RESOLU ✓

`docs/rust/PATTERNS.md:1110-1132` : entree `### T26` presente avec
mesure 82 ms, analyse du gap vs target 3s, explication pourquoi le
schema reste sur (double layer + keyring), fix path (m=128 MiB ou
t=6 apres telemetrie Raspberry Pi 4), note sur les blobs anciens
(self-describing params, pas de migration necessaire). Cross-ref
audit finding P2-3.

**P2-4 (test param_downgrade_attack_rejected)** — RESOLU ✓

`keystore.rs:1124-1144` : test `param_downgrade_attack_rejected`
present. Logique : init → read orig `m_cost` bytes[24..28] → flip a
une valeur differente (16 si orig != 16, sinon 32 — gere le cas ou
ARGON2_MEM_COST vaut MIN_M_COST) → `unlock("1234")` → expect
`UnlockError::AeadReject`. Le test est un test "negative" sur la
propriete de securite documentee (AAD inclut le header complet →
toute modification de params invalide le AEAD open).

Note : la logique de flip adaptatif (if orig == 16 then 32 else 16)
est correcte et robuste aux valeurs de constantes futures.

**P2-5 (T27 PIN CLI tech debt)** — RESOLU ✓

`docs/rust/PATTERNS.md:1134-1151` : entree `### T27` presente.
Documente l'exposition `ps auxe` + `HISTFILE` + Windows Task Manager,
le scope (Phase A dev/smoke-test), le fix path (Phase B `rpassword`
interactif + `--pin-fd <fd>` pour CI/batch), cross-ref audit P2-5.

**P2-6 (T25 FIPS migration tech debt)** — RESOLU ✓

`docs/rust/PATTERNS.md:1087-1108` : entree `### T25` presente.
Documente le contexte du swap aes-gcm → aws-lc-rs, le caractere
non-regrressif (algorithme byte-identique), le fix path (feature
`fips` + one-file swap), cross-ref kickoff §D1.

### Nouvelles observations post-fix

Aucun nouveau finding introduit par les corrections. Verification
spot :

- `rotate_pin` rewrite : le mapping exhaustif de toutes les variantes
  `UnlockError` vers `KeyStoreError` est complet (les 6 variantes
  d'`UnlockError` sont toutes couvertes). Aucun `_ =>` catch-all
  silencieux.
- `SBFB_IDENTITY_SECRET_HEX_ENV` : aucune constante locale residuelle
  dans `unlock.rs` ou `runtime.rs` (seul import depuis `nexus_core_rs`).
- T25/T26/T27 : les 3 entrees sont correctement numerotees
  (pas de collision avec T24 preexistant), documentees avec cross-refs,
  et non-bloquantes pour la phase.
- `param_downgrade_attack_rejected` : le test utilise `make_store_no_keyring`
  (pattern coherent avec les autres tests primitifs qui n'ont pas acces
  a un OS keyring reel en CI).

### P3 carries (non-bloquants, non-fixes — attendu)

- **P3-1** : `write_atomic` laisse un fichier orphelin `.enc.tmp`
  sur disk si `fs::rename` echoue. Blob chiffre, pas de plaintext
  expose. Cleanup optionnel. — `keystore.rs:888-901`

- **P3-2** : Divergence compteur tests plan §4 (+25 annonce dans
  le plan v1) vs reel (+28 post-fix incluant le test downgrade et
  les 5 tests launcher sous-comptes). Plan §4.4 n'a pas ete mis a
  jour avec le delta reel. Pas bloquant — livraison superieure.
  — `sprint20_plan.md §4.4`

- **P3-3** : `unwrap()` sur `try_into::<[u8;4]>()` dans `parse_blob`
  lignes 860-862. Infaillible par precondition (longueur verifiee
  ligne 845 : `blob.len() < BLOB_HEADER_LEN + TAG_LEN`), mais
  commentaire `// infallible: guaranteed by length check above`
  absent. Confusant pour futurs lecteurs.
  — `keystore.rs:860-862`

---

## Findings (post-fix — P3 only)

- **P3-1** : `write_atomic` sans cleanup `.enc.tmp` sur echec rename.
  — `crates/nexus-core-rs/src/keystore.rs:888-901`

- **P3-2** : Plan §4.4 annonce "+25 tests" mais reel = +28 (tests
  launcher sous-comptes + param_downgrade ajoute). Delta sup, pas
  bloquant. — `sprint20_plan.md §4.4`

- **P3-3** : `unwrap()` sur slice `try_into::<[u8;4]>()` sans
  commentaire "infallible". — `keystore.rs:860-862`

---

## Recommendation

**Commit autorise.** 0 P0, 0 P1, 0 P2 residuels.

Les 6 findings P2 de la premiere passe sont tous resolus avec des
fixes propres et bien documentes. Les 3 P3 carried over sont
optionnels et non-bloquants. La rigor signal G4 est satisfait
(>= 1 P2+ documente et resolu).

P3-2 (delta plan §4.4) peut etre corrige inline dans le commit
body ou laisse comme note retrospective — au choix de l'executeur.
