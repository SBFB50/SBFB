# Sprint 20 — Kickoff (Encryption at rest + duress PIN + panic wipe + PoW wire + structured output + canary auto-publish + dual-transport)

**Ecrit** : 2026-04-16 (session fraiche post-S19 Phase F wrap-up +
audit gate leve).
**Type** : **sprint implementation** (sprint post-Gate 1 apres
consolidation transport S19, big rock encryption at rest = Gate 2
prerequis critique).
**Tip master d'entree** : `3a7f0a3` (test(sprint19) audit-P2 A-3
— probe_and_cache quorum majority continues to dial).
**Phase 0 audit Sprint 19** : **DEJA JOUE** — findings dans
`.planning/active/sprint19_audit_findings.md` (verdict **PASS**,
0 P0 + 0 P1 + 9 P2 + 2 P3 documentes). 8 P2 + 1 P3 leves via
commit `1af90b3` chore audit-P2 batch + 1 P2 leve via commit
`3a7f0a3` test audit-P2 A-3. Finding A-2 cap G7 DIFFERE kickoff
S20 D5 (ce kickoff — cf. §4 D5 + `sprint20_carry_summary.md`).

---

## Sources context7 + WebSearch consultees (pre-gel D1..D5)

**Deep research lance 2026-04-16** avant figer les D-decisions :

- **`kerkour.com/rust-cryptography-ecosystem-2026`** : article
  reference (contenu WebFetch vide au moment de lecture, a
  re-consulter Phase A).
- **`docs.rs/keyring`** (via context7 `/websites/rs_keyring_
  keyring`) : API `CredentialApi` + caveats thread safety +
  keyutils_persistent Linux hybrid pattern.
- **Sygnia 2024 + SpecterOps 2024-2026** : **DPAPI user-scope
  bypass same-user confirme**. Master keys dans LSASS
  extractibles via Mimikatz (`/unprotect` flag direct). Patch
  Nov 2023 ne ferme pas ce vecteur.
- **Signal Secure Value Recovery blog** : pattern Argon2id +
  MAC entanglement + passphrase wrap → KEK 32-byte → wrap
  Shared Preferences encryption key. Calibre ~3s/attempt.
- **RFC 9106 Argon2 + OWASP Password Storage Cheat Sheet** :
  Argon2id recommande (side-channel resistant). OWASP default
  m=19 MiB, t=2, p=1.
- **GrapheneOS features page + discuss.grapheneos.org/d/17901
  duress PIN idea + 1950.ai + androidauthority** : pattern
  PIN saisi n'importe ou credential demande → wipe hardware
  keystore + eSIM + forced shutdown irreversible. Hidden
  volume **forensically detectable** (terminology note ci-apres)
  via forensics 2025-2026 : Passware Kit 2025 supports VeraCrypt
  1.26.15 hidden partitions via Memory Analysis attack. Cf.
  paper « The investigator's friend and foe : A forensic analysis
  of GrapheneOS » (2026, ScienceDirect / ResearchGate).

  *Terminology note (acknowledgement Design Review Board G1
  finding D3)* : dans les docs suivantes (kickoff + plan +
  PATTERNS.md + DURESS.md Phase B), le terme « detectable » est
  prefere a « non-deniable ». « Non-deniable » au sens
  cryptographique = « impossible a denier » = « provably real »
  ; VeraCrypt hidden volumes sont « detectables » (forensics)
  mais revendiquent encore de la deniability. Le reject
  VeraCrypt est correct (detectability affaiblit la promesse
  de deniability jusqu'a la rendre non-defendable en practice),
  mais la terminologie « non-deniable » etait impropre.
- **arxiv 2501.10868 JSONSchemaBench + MLC blog XGrammar** :
  llguidance 50 µs/token, XGrammar 3.5x/10x faster mais **pas
  llama.cpp support** (vLLM/SGLang only), GBNF natif ~200µs+
  /token, Outlines Python.
- **llama.cpp/docs/llguidance.md** : `-DLLAMA_LLGUIDANCE=ON`
  cmake flag + ExternalProject_Add cargo build Rust library
  Microsoft.
- **`docs.rs/llguidance`** : crate Rust Microsoft, Lark-like
  grammar + JSON native, 50µs/token, integre `llama-cpp-2`
  crate via feature `llguidance`.
- **`aws-lc-rs` HPKE RFC 9180** : FIPS 140-3 support, aligne
  VALIDATED_BLUEPRINT S17 preference.

Frontmatter `docs/security/HARDENING_ROADMAP.md` :
`last_validated: 2026-04-16` (bump G2 au moment ouverture S20).
`audited_findings` etendu avec entree S20.

---

## 1. Constat d'entree

### 1.1 D'ou on part

Sprint 19 a livre le **durcissement de la chaine transport P2P**
(DHT quorum runtime wire carry S18 C-1, PoW Hashcash primitive +
envelope + policy loader, TLS SPKI cert pinning primitive + hot-
reload, delayed upload queue exponential jitter 0-5 min anti-
correlation SQLite WAL, pkarr relay self-hosted docker image +
ops doc §1-§7 publishable). Audit gate S19 Phase 0 joue par
session fraiche 2026-04-16, verdict **PASS** (0 P0 + 0 P1 + 9 P2
+ 2 P3 resolus inline via commits `1af90b3..3a7f0a3`).

Gate 1 (DnD Forge T0-T1 beta fermee) reste **UNLOCKED**. Sprint 20
attaque la **Gate 2 prerequisite critique** : encryption at rest
keypair + duress PIN + panic wipe + warrant canary auto-publish
+ dual-transport fallback + structured output llama.cpp +
integration runtime PoW wire carry S19 explicite au scope.

Les primitives S19 livrees sont `primitive-ready` mais pas toutes
`runtime-active` (cf. `docs/rust/PATTERNS.md §Sprint 19.1 Primitive
/ wire / enforcement separation`). Sprint 20 execute la promesse
d'incrementalite annoncee par ce pattern pour **PoW wire** (integre
au scope Phase C). Les autres (TLS iroh wire, DHT enforcement
strict) restent hors-scope S20 — cf. `sprint20_carry_summary.md`.

### 1.2 Ancrage HARDENING_ROADMAP §3 Sprint 20

La roadmap Phase D S17 specifie Sprint 20 items :

| Item | Source roadmap | Phase S20 |
|---|---|---|
| Keypair encryption at rest via Keychain/DPAPI/libsecret (~800 LOC) | §3 S20 | A |
| Duress PIN unlock (fake keypair → noop responses) (~500 LOC) | §3 S20 | B |
| Panic wipe 5-tap gesture (shell shortcut Ctrl+Shift+Alt+W) (~400 LOC) | §3 S20 | B (grouped avec duress) |
| Structured output llama.cpp grammar (JSON schema enforcement) (~300 LOC) | §3 S20 | D |
| Warrant canary auto-publish gossip heartbeat monthly (~200 LOC) | §3 S20 | E (grouped avec transport) |
| Dual-transport detection + WebSocket fallback TCP 443 (~300 LOC) | §3 S20 | E (grouped avec canary) |
| **PoW runtime wire gossip subscribe** (carry S19 `edfc51b` integre au scope S20) | `sprint19_audit_findings.md A-2` reclassifie | C |

Plus **Meta-1 Radicle-v1.0 tracking** + **P2-2 .gitignore NOISE
coverage** via carry G7 (cf. `sprint20_carry_summary.md`).

**Gate unlock** fin S20 : **preparation Gate 2** (TransLingua,
FamilyScan). Pas un Gate officiel HARDENING_ROADMAP §7 mais
prerequis critique A-S9 Checkpoint-seize elimine (`docs/security/
HARDENING_ROADMAP.md §3 S20 Goal`).

### 1.3 Compteurs de tests a l'entree (tip `3a7f0a3`)

| Suite | Count observe entree S20 |
|---|---|
| Rust workspace | 538 (test audit-P2 A-3 +1 vs 537 S19 close) |
| Python SDK | 185 |
| Python coordinator | 208 + 3 skipped |
| Python app-gov | 46 |
| Vitest unit | 239 |
| Playwright | 38 |
| size-limit | 7/7 |
| SPDX | 246+ |
| **Total** | **~1260 tests** |

**Delta Sprint 20 attendu : +65 a +90** (HARDENING_ROADMAP
projection : +65). Repartition estimee par phase dans plan.md §9.

### 1.4 Pre-launch protocol policy (rappel)

Sprint 19 a confirme la regle : `*_VERSION = 1` jusqu'au tag v1.0,
pas de tolerant decoder multi-version, `#[serde(default)]` legitime
uniquement pour runtime tolerance. Sprint 20 respecte : aucun item
ne touche un wire format existant.

L'**encryption at rest** produit un **format de blob local**
`~/.sbfb/keyring/<node_id>.enc` (format nouveau, v1). Pas de wire
protocol — fichier local uniquement. Format documente dans plan §A.

Le **duress PIN noop responses** ne change pas le wire : un peer
distant ne peut pas distinguer un SBFB-daemon duress d'un SBFB-
daemon normal par inspection gossip/relay (c'est le point — defense
par indistinguabilite).

---

## 2. Goal en une phrase

**Le projet elimine le checkpoint-seize risk (A-S9) en chiffrant
au repos le keypair d'identite via double-layer Argon2id(PIN) +
AES-256-GCM (Signal pattern), en ajoutant un duress PIN qui
desactive silencieusement l'identite reelle (fake keypair noop),
un panic wipe 5-tap irreversible, un warrant canary auto-publie
mensuel, un fallback transport WebSocket TCP 443 contre DPI ISP,
et en cablant au runtime la primitive PoW Hashcash S19 au path
gossip subscribe pour debloquer S21 rate-limit — critere SMART :
fail-fast checklist `sprint20_verification.md §Fail-fast` verte
(32+ rows executables) au Phase F wrap-up.**

---

## 3. Phase 0 — Audit Sprint 19 (DEJA JOUE — verdict PASS)

**Status** : JOUE session 2026-04-16 (~2h, post-`bf7dd62`
wrap-up). Ne pas rejouer. Cf.
`.planning/active/sprint19_audit_findings.md` (reste dans
active/ jusqu'a migration Phase F S20).

**Commit stack du gate (leve)** :

```
3a7f0a3 test(sprint19): audit-P2 A-3 — probe_and_cache quorum majority continues to dial
1af90b3 chore(sprint19): audit-P2 batch — claim fix + PATTERNS callouts + ruff + F-review migrate
```

2 commits ont ferme le verdict PASS (0 P0 + 0 P1 + 9 P2 + 2 P3) :
- **P2-A1** : claim-drift « Eclipse-by-DHT pleinement active runtime »
  fixe dans 6 endroits (CLAUDE.md, SPRINT_LOG.md x2, sprint18/19
  verification.md x3).
- **P2-B1** : PoW bench wall-clock regression detection tech debt
  T22 logged.
- **P2-C1** : divergence plan fail-closed vs code fail-open pinset
  empty documentee PATTERNS.md §Sprint 19.3.
- **P2-C2** : TLS bootstrap pins n0 embed tech debt T21 logged.
- **P2-D1** : plaintext payload caveat promu en haut PATTERNS.md
  §P29.
- **P2-D2** : ruff format 2 fichiers Phase D corriges inline.
- **P2-E1** : Dockerfile `@sha256` digest pin tech debt T23 logged
  + TODO inline.
- **P2-A3** : test happy path quorum livre via `3a7f0a3` (+1 Rust).
- **P3-B2** : Design Review Board G1 extension crypto/spec
  (≥1 alternative concurrente recente) ajoutee docs/claude/README
  §6.1.1.
- **P3-A4** : `sprint19_phase_F_review.md` migre active→archive.

**Verdict final** : **PASS**. Sprint 20 Phase A non-bloque.

**Dette heritee Sprint 19 confirmee** (cf. `sprint20_carry_summary.md`) :
- **Meta-1 Radicle-v1.0 activation tracking** (re-carry S18→S19→S20).
- **P2-2 .gitignore NOISE coverage** (trivial inline ce kickoff).
- **PoW wire** reclassifie **scope S20 Phase C integre** (pas carry).
- **TLS wire T20** reclassifie **tech debt long-terme PATTERNS.md**.
- **DHT strict** reclassifie **post-Gate-2 HARDENING_ROADMAP**.

Cap G7 apres reclassification : **2/2 respecte**.

---

## 4. Decisions Day 0 (D1..D5)

### D1 — Architecture encryption at rest keypair : double layer

**Retenu** : **double layer** encryption at rest Ed25519 identity
keypair :

```
Ed25519 privée
   │ AES-256-GCM (aws-lc-rs, FIPS 140-3)
   │ KEK = Argon2id(PIN + 16-byte random salt)
   ▼
blob chiffre `~/.sbfb/keyring/<node_id>.enc` (format v1)
   │ OS keyring (keyring-rs) stocke un wrap supplementaire
   │ de la KEK (defense-en-profondeur)
   ▼
DPAPI / Keychain / Secret Service
```

En memoire :
- `secrecy::SecretBox<[u8; 32]>` pour la KEK decryptee
- `zeroize::Zeroize` implementations sur tous les types privee (Ed25519
  SecretKey, Argon2id output, PIN bytes)
- PIN jamais stocke en clair — seul l'Argon2id(PIN) hash transit

Format blob v1 :
```
magic: "SBFBK1" (6 bytes)
version: u8 (=1)
argon2_salt: [u8; 16]
argon2_params: m_cost u32 + t_cost u32 + parallelism u32
aead_nonce: [u8; 12]
ciphertext: Vec<u8> (Ed25519 privée 32 bytes + domain tag "IDKEY" wrap)
aead_tag: [u8; 16]
```

**Rejete** :

- **OS keyring seul** (DPAPI/Keychain/Secret Service direct) :
  confirmed gap T3 same-user process malicieux. Sygnia 2024
  « DPAPI downfall » : Mimikatz `/unprotect` flag bypass trivial
  sous context user authentifie. SpecterOps 2024-2026 :
  operational guidance DPAPI abuse = direct extract master keys
  LSASS. macOS Keychain : app signee pareil access
  `SecKeychainFindGenericPassword`. Linux Secret Service (DBus) :
  any process uid-matching peut read. **Insuffisant sans layer
  supplementaire**.
- **`age` file-based** (`str4d/rage` crate) : format solide,
  passphrase-protected identity files supportes. Mais ecosysteme
  oriente-fichier : hot-rotation KEK requires re-encrypt all files,
  notification watcher non-native. Choisi contre pour ergonomie
  rotation. `age-plugin-keystore` hybride existe (X25519/PQ hybrid
  via Secret Service) mais ajoute une dependency path complexe vs
  double layer direct.
- **HPKE RFC 9180** (`aws-lc-rs` / `rust-hpke` / `cryspen/hpke-rs`) :
  envelope encryption scheme hybride asymetrique (ECDH + HKDF +
  AEAD). Pour at-rest local **single-party** avec PIN-derived KEK,
  le setup asymetrique (generer+stocker ephemeral KEM) est overhead
  non-justifie : on n'a pas de second party ni de wrap-for-peer.
  Le pattern symetrique direct (Argon2id → KEK → AES-GCM) est
  strictement plus simple et couvre le meme threat model. Reporte
  S22+ si wrap-for-peer pattern apparait (restore keypair via pair
  trusted, pattern Signal SVR cross-device). Note Design Review
  Board G1 : le rejet initial etait lapidaire (« wire scope pas
  rest ») — cette version clarifie que HPKE **est** techniquement
  utilisable pour at-rest mais inutilement complexe pour le threat
  model single-party S20.
- **Hardware keystore TPM 2.0 / Secure Enclave / StrongBox** :
  ideal securite mais **platform-specific builds divergents**.
  Windows 11 TPM 2.0 minimum hardware varie, macOS Secure Enclave
  requires entitlement + notarization, Linux TPM-TSS userspace pas
  universel. **Reporte S22+** avec plan abstraction `trait KeyStore`
  + plusieurs impls selectees via build features.
- **scrypt / PBKDF2 / bcrypt** pour KDF :
  - scrypt : pas GPU-resistant moderne 2026 (ASIC bitcoin-scrypt
    commodity), remplace par Argon2id RFC 9106 (2022).
  - PBKDF2 : pas memory-hard, brute-force trivial GPU/ASIC sur
    PIN court.
  - bcrypt : pas memory-hard, max input 72 bytes contrainte.

**Implications code** :

- Nouveau module `crates/nexus-core-rs/src/keystore.rs` : trait
  `KeyStore` + `LocalFileKeyStore` impl (le double layer).
- Nouveau module `crates/nexus-shell-daemon/src/unlock.rs` : flow
  PIN prompt + Argon2id derive + decrypt keypair + boot
  `IrohNodeContext`.
- Bump dep `argon2 = "0.5"` (RustCrypto), `aws-lc-rs = "1.18"`
  (FIPS track), `keyring = "3.6"` (confirmer version via context7
  Phase A research fresh).
- `sbfb-launcher` CLI : `sbfb init --pin` setup initial, `sbfb
  unlock` flow interactif.

### D2 — Argon2id parameters

**Retenu** : **m=64 MiB, t=3, p=1** (Argon2id RFC 9106).

Target calibration : **~3 secondes/attempt** sur hardware moderne
2026 (RTX 5080 dev / Raspberry Pi 4 low-end consumer). Bench
obligatoire Phase A kickoff — si >5s sur Pi 4, fallback m=32 MiB.

**Rejete** :

- **m=19 MiB, t=2, p=1** (OWASP default 2024) : calibre pour
  **passphrase** (entropy ~40+ bits). Pour **PIN court 4-6 digits
  = <20 bits d'entropie**, insuffisant contre brute-force GPU
  moderne (RTX 5080 ~500k attempts/sec Argon2id @ 19 MiB, crack
  PIN 6-digit en <2 sec). Bump m=64 MiB impose 10x plus memory
  bandwidth bottleneck → GPU asymmetric advantage reduit.
- **scrypt** : deja rejete D1 (pas GPU-resistant moderne 2026).
- **PBKDF2-SHA256 1M iterations** (NIST SP 800-132) : pas memory-
  hard, brute-force trivial GPU.
- **Argon2d** : side-channel vulnerable, RFC 9106 §3.1 recommande
  Argon2id sauf cas crypto-currency specifique.
- **Argon2i** : slower que Argon2id a security equivalente,
  side-channel resistant mais pas meilleur que id pour notre
  threat model (pas de co-tenant timing attack pre-launch single-
  user).

**Implications code** : constants dans
`crates/nexus-core-rs/src/keystore.rs` :

```rust
pub const ARGON2_MEM_COST: u32 = 64 * 1024; // 64 MiB in KiB
pub const ARGON2_TIME_COST: u32 = 3;
pub const ARGON2_PARALLELISM: u32 = 1;
```

Test bench Phase A : `cargo bench --bench keystore`. Assert
`derive_kek` wall-clock < 5s sur CI runner (GitHub Actions
standard). Documenter numero dans PATTERNS.md §KEK derive
bench reference (evite regression T22-pattern S19).

### D3 — Duress PIN pattern : fake keypair noop (pas wipe immediate)

**Retenu** : **GrapheneOS-inspired mais ADAPTE** au modele SBFB
daemon :

Quand PIN entre = **PIN duress** (second PIN configure a `sbfb init
--duress-pin`) :

1. **Ne pas wiper immediatement** le blob reel (distinction vs
   GrapheneOS Android).
2. Decoder un **blob alternatif** `~/.sbfb/keyring/<node_id>_
   duress.enc` contenant un **fake Ed25519 keypair** (genere a
   `sbfb init`, jamais publie sur le reseau reel).
3. Booter le daemon avec ce fake keypair : le daemon accepte
   requetes, publie sur gossip avec fake identity, renvoie noop
   responses (pas de data reelle, pas de kudos, pas de curator
   subscribe reel).
4. Signal indistinguishable au peer observer : un peer distant
   ne peut pas savoir qu'il parle a un SBFB-daemon duress (defense
   par indistinguabilite).
5. **Panic wipe separe** : gesture 5-tap shell (cf. Phase B) =
   VRAIE destruction + exit forced. Duress n'implique pas wipe.

Rationale vs GrapheneOS wipe-immediate :
- SBFB n'est pas un OS mobile (pas eSIM a wiper, pas keystore
  hardware a scramble).
- Le daemon peut tourner **apres** duress-unlock sans signaler
  (fake gossip activity meme masque le trafic reel).
- Wipe immediate = tell-tale — un adversaire observing qui a force
  un PIN et voit le daemon crash instantanement sait qu'un PIN
  duress existe. Fake noop = deniable par construction.

**Rejete** :

- **Wipe immediat GrapheneOS strict** : tell-tale (cf. rationale
  ci-dessus). Approche correcte pour OS mobile (state management
  stack), mauvaise pour daemon P2P.
- **VeraCrypt hidden volume / deniable encryption filesystem** :
  forensics 2025-2026 **detectent** le hidden volume via
  ciphertext size analysis + header entropy distribution +
  memory analysis (Passware Kit 2025 supporte VeraCrypt 1.26.15
  hidden partitions). Cf. paper « The investigator's friend and
  foe : A forensic analysis of GrapheneOS » 2026 (ScienceDirect).
  La detectability affaiblit la promesse de deniability jusqu'a
  la rendre non-defendable en practice. Attracts adversarial
  pressure « prove there's no hidden volume » — pire que pas de
  defense declaree.
- **Soft-delete PIN (ecrase en RAM seulement)** : cold-extract
  disk defait. Pas une defense contre T4 adversary (border /
  coercion) qui ecrase le disk live.
- **Platform-managed eSIM wipe** (GrapheneOS pattern) : out-of-
  scope SBFB cross-platform (desktop only).

**Warning legal obligatoire** documente dans `docs/security/
DURESS.md` (nouveau, Phase B) : destruction d'evidence peut
constituer obstruction dans certaines juridictions. Fake-keypair
noop **n'est pas** destruction, c'est indisponibilite — less
legally exposed.

**Implications code** :

- Option `sbfb init --duress-pin=<pin>` genere un fake keypair +
  encode dans `<node_id>_duress.enc` avec **meme KEK derivation
  Argon2id mais different PIN**.
- Module `crates/nexus-core-rs/src/keystore.rs` : `KeyStore::
  unlock(pin) -> Result<Identity, UnlockError>` retourne `Identity
  { keypair, mode: Normal | Duress }`. Le caller (`sbfb-launcher`)
  interprete Duress et route `IrohNodeContext` avec fake endpoint.
- Le **daemon** ne sait pas qu'il est en mode duress — c'est le
  launcher qui setup le routing noop (pattern defense-in-depth :
  fuite d'info daemon-side ne compromet pas).

### D4 — Structured output llama.cpp : llguidance

**Retenu** : **`llguidance`** crate Rust Microsoft via feature
flag `llama-cpp-2` et build flag `-DLLAMA_LLGUIDANCE=ON` llama.cpp.

Scope minimal S20 Phase D :

- Task envelope schema JSON coord-side (signed TaskEntry wire
  format + TaskResult wire format). Llama.cpp enforce la grammar
  au decode-time pour que les worker responses soient toujours
  valid-schema avant signature.
- Document `docs/rust/PATTERNS.md §P30 structured output llama.cpp`
  — **avec warning explicite** : « structured output grammar
  **n'est PAS** une defense anti prompt injection (cf. HARDENING_
  ROADMAP audited_findings 2026-04-16 "S21 grammar ≠ prompt
  injection defense") ». Grammar force le format, pas le contenu.

Performance budget : `llguidance` 50 µs/token, negligible vs
llama.cpp token decode ~1-10 ms per token on RTX 5080. Pas de
regression UX attendue.

**Rejete** :

- **GBNF natif llama.cpp** : slower ~200 µs+/token, pas de Rust
  binding native (C API via `llama-cpp-2` bindings mais pas
  integration aussi propre que feature `llguidance`).
- **XGrammar** (mlc-ai) : **pas supporte par llama.cpp** au 2026-
  04-16 (verifie MLC blog + arxiv 2501.10868). Integre vLLM/SGLang
  seulement. Stack SBFB Option G = llama.cpp via worker Rust → hors-
  scope.
- **Outlines (Python)** : overhead IPC coord → worker LLM
  (serialize grammar + tokens cross-language), brise l'Option G
  (separation Rust workspace vs Python workspace stricte). Non
  aligne.
- **JSON Mode / tool_use OpenAI-compat** : requires compat layer
  custom coord-side. Pas de standard, chaque backend different
  (Ollama, llama.cpp server, vLLM). Llguidance = directement
  llama.cpp-friendly.
- **LMQL / Guidance Python** : overhead IPC pareil que Outlines.

**Implications code** :

- `Cargo.toml` workspace : `llguidance = "1.7"` optional dep dans
  `nexus-worker-core` (bumpe depuis `0.7` annonce initialement au
  kickoff — version courante verifiee context7 `/guidance-ai/
  llguidance` 2026-04-18 Phase D session fraiche, levee in-phase
  cf. audit P2-2 Phase D).
- `llama-cpp-2` deja pinne workspace (ou a ajouter) avec feature
  `llguidance`.
- `crates/nexus-worker-core/src/llm.rs` : wire `llguidance::
  Constraint` dans le path `llm.generate(prompt, grammar)`.
- Schema JSON dispatch task response : fichier
  `crates/nexus-core-rs/src/schemas/task_response.schema.json`
  (source-of-truth), converti Lark-like via llguidance builtin
  JSON Schema support.

### D5 — Cap G7 carry-overs reclassification

**Retenu** (cf. `sprint20_carry_summary.md`) :

| # | Item | Classification S20 |
|---|---|---|
| 1 | Meta-1 Radicle-v1.0 activation tracking | **Carry confirme** §Meta-track S20 |
| 2 | P2-2 .gitignore NOISE coverage | **Carry confirme** — chore inline ce kickoff |
| 3 | PoW runtime wire gossip subscribe | **Scope S20 Phase C integre** (pas carry) |
| 4 | TLS wire iroh T20 | **Tech debt long-terme** PATTERNS.md §T20 (pas carry) |
| 5 | DHT canary → enforcement strict | **Post-Gate-2** HARDENING_ROADMAP (pas carry) |

Cap G7 respecte : **2/2**.

**Rejete** :

- Laisser les 5 items comme carry-overs sans reclassification :
  depasse cap G7 = 2, declencherait P1 auditor S20 Phase F. Le
  cap existe precisement pour rendre visible ce glissement
  (finding A-2 S19 audit a attrape le pattern).
- Livrer les 5 items dans S20 scope : impossible (TLS wire iroh
  bloque par upstream 0.97 absence hook, DHT strict requires
  federation Pkarr consolide post-Gate-2).
- Abandonner PoW wire (DEPRECATED.md) : casse promesse commit
  body `edfc51b` S19 + bloque S21 rate-limit. Non-starter.

**Implications** : section §6 Items carry/dette ci-dessous + plan
§Phase C integration PoW wire explicite.

### Acknowledged review findings (G1)

Rapport Design Review Board G1 : `.planning/active/sprint20_design
_review.md` (reviewer agent Explore independant, 15 min timebox,
2026-04-16). **Scoring** : D1 ✅, D2 ✅, D3 ⚠️, D4 ✅, D5 ✅. Rigor
signal G4 satisfait (1 ⚠️ sur 5, pas 100% ✅).

**D1 ✅** : HPKE rejet initial lapidaire (« wire scope pas rest »).
**Decision** : **adjust** — clarification detaillee ajoutee au
paragraphe D1 ci-dessus. HPKE est techniquement utilisable pour
at-rest mais le setup asymetrique (ECDH + HKDF) est overhead
non-justifie pour le threat model single-party PIN-derived KEK.
Rationale Signal SVR cross-device reporte S22+ si wrap-for-peer
pattern apparait.

**D2 ✅** : noted, no action required. Sources RFC 9106 + OWASP
2024-2025 + Signal Argon2id 2024 verifiees recentes, toutes
alternatives modernes evaluees.

**D3 ⚠️** : terminologie « non-deniable » ambigue + source
GrapheneOS research 2026 non-linkee. **Decision** : **adjust** —
remplacement « non-deniable » → « detectable » dans les 2
occurrences (§Sources + D3 Rejete), lien explicite au paper
« The investigator's friend and foe : A forensic analysis of
GrapheneOS » (2026 ScienceDirect) ajoute, note terminologique
documentee. Reject VeraCrypt reste **correct sur le fond**
(forensics 2025-2026 Passware Kit detecte hidden volumes VeraCrypt
1.26.15 via Memory Analysis) — seulement la terminologie
reseme.

**D4 ✅** : noted, no action required. Excellente rigueur selon
reviewer. 4 alternatives modernes 2025+ (GBNF, XGrammar, Outlines,
JSON Mode OpenAI) avec raisons explicites du rejet. arxiv
2501.10868 JSONSchemaBench (jan 2025) confirme llguidance p99
0.5ms.

**D5 ✅** : noted, no action required. Pattern organisationnel
(cap G7), non-applicable a la regle G1 extension crypto/spec.
Cap 2/2 respecte.

**Aucun blocage P0/P1 pour Phase A S20**. Tous les ⚠️ sont points
d'amelioration redactionnelle acknowledge et adjusted inline dans
ce kickoff avant gel. Design Review Board mission accomplie.

---

## 5. Plan Phase outline

### Phase 0 — Audit Sprint 19 (DEJA JOUE, verdict PASS)

`sprint19_audit_findings.md` reste dans `active/` jusqu'a
migration Phase F S20 (pattern S18→S19).

### Phase A — Encryption at rest keypair (big rock) (+25 tests)

Scope : trait `KeyStore` + `LocalFileKeyStore` impl double layer
(Argon2id + AES-256-GCM + OS keyring wrap). Bench KEK derive <5s.
Tests cross-platform Windows/macOS/Linux.

Livrable commit : `feat(sprint20): Phase A — encryption at rest
keypair (Argon2id + AES-256-GCM + double layer OS keyring)`

### Phase B — Duress PIN + panic wipe (+15 tests)

Scope : `KeyStore::unlock(pin)` differential Normal/Duress path,
launcher route daemon vers fake `IrohNodeContext` en mode duress.
Panic wipe 5-tap shell gesture `Ctrl+Shift+Alt+W` : zeroize RAM
+ secure-unlink `<node_id>.enc` + `<node_id>_duress.enc` +
state.sqlite + blob cache + forced exit.

Livrable commit : `feat(sprint20): Phase B — duress PIN (fake
keypair noop) + panic wipe 5-tap gesture`

### Phase C — PoW runtime wire gossip subscribe (carry S19 integre) (+10 tests)

Scope : wire `subscribe_with_pow` au path `crates/nexus-shell-
daemon-core/src/iroh_runtime.rs::GossipClient::subscribe()`.
Lecture `~/.sbfb/relay_pow_policy.toml` + enforce per-topic
difficulty. Tests fail-close si publisher sans proof valide.

Livrable commit : `feat(sprint20): Phase C — PoW runtime wire
gossip subscribe (scope integre carry S19 A-2)`

### Phase D — Structured output llguidance (+12 tests)

Scope : integration `llguidance` feature `llama-cpp-2` +
enforcement schema JSON `task_response.schema.json` + warning
doc PATTERNS.md §P30 (grammar ≠ prompt injection defense).

Livrable commit : `feat(sprint20): Phase D — structured output
llama.cpp llguidance grammar (JSON schema enforce)`

### Phase E — Warrant canary auto-publish + Dual-transport WSS fallback TCP 443 (+8 tests)

Scope :
- Warrant canary **scheduler automation** (gossip heartbeat
  mensuel signe Ed25519+JCS, au-dessus du publish manuel S18
  E2 `04c9621`). Check exactement ce que S18 a livre (risque
  duplicate) au kickoff Phase E.
- Dual-transport detection : au boot daemon, probe UDP QUIC →
  si bloque (timeout repete 3x en 10s), bascule `RelayMode::
  WebSocket` over TCP 443. Log warn.

Livrable commit : `feat(sprint20): Phase E — warrant canary
auto-publish + dual-transport WSS TCP 443 fallback`

### Phase F — Consolidation + verification + audit plan S21 (docs only)

Update CLAUDE.md + SPRINT_LOG.md row S20 + memory fusion +
`sprint20_verification.md` + `sprint20_audit_plan.md`. Migration
PARA tous sprint20_*.md → `archive/v1.2/`.

Livrable commit : `chore(sprint20): Phase F — wrap-up +
verification + audit plan S21 + migrate planning`

---

## 6. Items carry/dette

### Items carry confirmes S20 (cap G7 = 2/2)

- [x] **Meta-1 Radicle-v1.0 activation tracking** : re-carry
  S18→S19→S20 confirme. Owner FlowUP, deadline jour tag v1.0.
  Runbook `docs/release/MIRROR_FALLBACK.md §3.1-3.8` self-
  contained. Phase F S20 re-carry S21 si v1.0 pas tag.
- [x] **P2-2 .gitignore NOISE coverage** : inline ce chore
  ouverture S20 (pattern `*.exe`, `*.pdb`, `cc.json`, `/node_
  modules/` racine, `/site/`, `/docs/apps/`).

### Items reclassifies (NON-carry — cf. `sprint20_carry_summary.md`)

- [scope] **PoW runtime wire gossip subscribe** → integre S20
  Phase C directement.
- [tech-debt] **TLS pinning wire iroh T20** → `docs/rust/PATTERNS.
  md §T20` long-terme (iroh 0.97 limitation upstream).
- [post-Gate-2] **DHT canary → enforcement strict** → HARDENING_
  ROADMAP.md §3 post-S22 (reevalue kickoff S23+ si prerequis
  federation Pkarr consolide remplis).

---

## 7. Tracabilite scope

Items **nouveaux Sprint 20** :
- Keypair encryption at rest double layer (Phase A)
- Duress PIN fake keypair noop (Phase B)
- Panic wipe 5-tap gesture (Phase B)
- Warrant canary auto-publish scheduler (Phase E, check S18
  overlap avant)
- Dual-transport WSS TCP 443 fallback (Phase E)
- Structured output llguidance (Phase D)

Items **carry/dette** :
- PoW runtime wire gossip subscribe (carry S19, scope integre
  Phase C)
- Meta-1 Radicle-v1.0 activation tracking (carry S18→S19→S20)

Items **differes** :
- TLS wire iroh T20 → tech debt long-terme (iroh 0.97 upstream)
- DHT strict → post-Gate-2 roadmap
- Rate-limit per-consumer → S21 (depend S20 PoW wire Phase C)
- Client-side redaction SDK → S21
- Kudos-weighted admission → S22
- Sandbox tool-calling allow-list → S22
- Hardware keystore (TPM 2.0 / Secure Enclave / StrongBox) → S22+
  (abstraction `trait KeyStore` deja Phase A = prep pour impls)
- ML-DSA-65 + ML-KEM-1024 PQC → S26+ (attention HNDL liability
  audited_findings HARDENING_ROADMAP)
- Domain fronting + Tor bridges → S24-25
- `actions/checkout@v4` pin SHA sweep → sprint ops futur

---

## 8. Scope cuts (PAS dans ce sprint)

Cf. §7 ci-dessus pour detail. En resume :

- **Hardware keystore** : scope-cut Phase A (abstraction `trait
  KeyStore` livree mais impls TPM/SE/StrongBox = S22+)
- **HPKE envelope for keypair peer-restore** : Signal SVR-style
  cross-device restore → S22+ (pas besoin pre-launch single-
  user)
- **Rate-limit per-consumer** : S21 (depend Phase C S20)
- **Client-side redaction SDK** : S21
- **Kudos-weighted gossip admission** : S22
- **Tool-calling sandbox allow-list strict** : S22
- **Redundancy voting Task.redundancy_factor** : S22
- **Ephemeral workers + VRAM wipe** : S23
- **Honeypot Eclipse detection** : S23
- **Re-run sampling + DNS fallback DHT** : S24
- **Arti Tor bridge integration** : S25
- **Domain fronting Snowflake-WebRTC** : S25
- **PQC migration ML-DSA + ML-KEM** : S26+ (audited_findings
  HARDENING_ROADMAP note HNDL liability — pourrait accelerer)
- **`actions/checkout@v4` pin SHA sweep** : sprint ops futur

---

## 9. Audit gate pattern — rappel

Phase 0 Sprint 19 audit joue pre-S20 session 2026-04-16, verdict
PASS apres 2 commits `1af90b3..3a7f0a3`. Phase F S20 produit
`sprint20_audit_plan.md` pour Sprint 21 Phase 0. Pattern permanent
depuis Sprint 7.

Meta-1 Radicle-v1.0 tracking re-carry explicite dans
`sprint20_audit_plan.md` §meta-track (prevu Phase F).

**Rigor signal G4** : verdict audit S20 Phase 0 (session S21
fraiche) exigera ≥1 P2+ documente. Si 0 finding = CONCERN pas
PASS.

**Design Review Board G1** : agent Explore independant execute
sur ce draft D1..D5. Rapport dans
`.planning/active/sprint20_design_review.md`. Planner
acknowledge chaque ⚠️ / ❌ dans §4 « Acknowledged review findings
» (section ci-dessus, remplie apres reception rapport).

---

## 10. Checkpoint de validation

Status : **draft en attente Design Review Board G1 output**.
Points de validation souhaitables (non-bloquants si user confirme
approche autonome) :

1. **D1 double-layer vs single OS keyring** : OK double layer ou
   preference simplification single layer (risque T3 same-user
   accepted) ?
2. **D2 Argon2id m=64 MiB** : OK ou preference m=128 MiB (plus
   secure, ~6s/attempt risky mobile) / m=32 MiB (compromise Pi 4
   ~1-2s) ?
3. **D3 Duress fake keypair noop vs wipe immediate** : OK fake
   noop (deniable indistinguishable) ou preference wipe immediate
   (tell-tale mais binaire) ?
4. **D4 llguidance vs GBNF natif** : OK llguidance (50µs/token Rust
   integration) ou prefere GBNF natif (simpler, slower) ?
5. **D5 reclassification 5 carry-overs** : OK les 3 reclassifies
   (PoW scope, TLS tech debt, DHT post-Gate-2) ou preference autre
   categorisation ?

**Fichiers untracked root** (`cc.json`, `test_libc.exe/pdb`,
`site/`, `node_modules/`, `docs/DND_P2P_DESIGN.md`, `docs/VISION_
USE_CASES.md`, `docs/apps/`, `.claude/settings.local.json`,
`.claude/worktrees/`) : **traite via carry P2-2 inline dans
commit chore(planning) ouverture S20** (ajout patterns
`.gitignore`). Resolu.

---

**Note de placement** : ce kickoff est ecrit directement dans
`.planning/active/` avec `sprint20_plan.md` +
`sprint20_design_review.md` (produit par agent G1) +
`sprint20_carry_summary.md`. `sprint19_audit_findings.md`
migre archive/v1.2/ dans ce meme commit `chore(planning): open
Sprint 20`.
