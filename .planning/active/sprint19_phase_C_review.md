# Sprint 19 Phase C — nexus-phase-auditor review

**HEAD pre-commit** : `08f4e41`
**Draft commit title** : `feat(sprint19): Phase C — TLS cert pinning relays (SPKI hash validate)`
**Timebox** : ~55 min
**Auditor** : nexus-phase-auditor (session 2026-04-16)

## Verdict : PASS

**Promu CONCERN → PASS** après intégration des 2 mitigations
code/commit recommandées par l'auditeur avant commit :

- **P2-1 FIXED** : `.planning/research/S19_phase_C_tls_cert_pinning_design.md`
  (~1031 lignes, threat model T2-T5 + alternatives + limitations
  + contribution upstream iroh) désormais stagé dans le commit
  Phase C. La règle "design doc AVANT code" est satisfaite dans
  l'historique git `git show <sha-phase-C>` voit le raisonnement
  accompagner le code.
- **P2-2 FIXED** : `static ENV_GUARD: Mutex<()>` ajouté au module
  `tests` de `tls_pinning.rs` ligne 526, acquisition `let _lock =
  ENV_GUARD.lock().unwrap();` en tête des 2 tests env-mutants
  (`relay_pins_file_path_honours_custom_env_override` ligne 861
  + `relay_pins_file_path_falls_back_to_sbfb_home` ligne 873).
  Pattern aligné avec `relay_config.rs:241`, `pkarr_resolver.rs:255`,
  `relay_pow_policy.rs:232`. Race condition `SBFB_HOME` /
  `SBFB_RELAY_PINS_FILE` cross-module fermée.

**P3-1 reporté Phase F wrap-up** : `sprint19_plan.md §6.3` contient
`// "base64url-..." 44 chars` dans le snippet Rust preview ; la
valeur correcte est 43 chars (SHA-256 = 32 bytes, base64url no-pad
= ⌈32×4/3⌉ = 43). Le code livré est correct (43 chars partout).
Cosmétique planning-only, non-bloquant.

## Verdict initial (pré-intégration mitigations) : CONCERN

0 finding P0, 0 finding P1. Commit autorisé avec 2 findings P2 et
1 finding P3 à logguer. Verdict initial conservé ci-dessous pour
trace audit.

---

## Dimensions

### Security

- **`#![forbid(unsafe_code)]` préservé** : confirmé ligne 31 de
  `lib.rs`. Le module `tls_pinning.rs` est safe Rust pur, zéro
  bloc `unsafe`. **PASS**.
- **Aucun secret hardcodé** : la constante `TEST_CERT_SPKI_SHA256 =
  "Aq1c_N_zjopBnfg-mcHBozX8dgA64izVtd_zgdDioXs"` est un test
  vector volontaire (self-signed test cert CN=relay.test.invalid),
  pas une vraie clé relay. **PASS**.
- **unwrap() en tests uniquement** : tous les `unwrap()` sont dans
  `#[cfg(test)]`. Zéro `unwrap()` dans le code de production.
  **PASS**.
- **Pattern fail-open/fail-closed** : correctement implémenté et
  documenté (module doc comment + design doc §5.4). Pinset vide →
  warn + WebPKI seul. Pinset non-vide + relay absent →
  `PinError::NoPin` (fail-closed). **PASS**.
- **Backup pin RFC 7469 §4.3** : test
  `validate_accepts_backup_pin_when_primary_expired` couvre le
  scénario. **PASS**.
- **Pas d'injection dans iroh relay client** : intentionnellement
  absent (déviation 1 documentée + T20 tech debt). Le
  `PinValidator::validate()` n'est pas appelé au handshake TLS en
  production — documenté explicitement dans le module doc.
  **PASS** (risque résiduel trackable et documenté).
- **Loopback/wire/zip** : aucun de ces domaines touchés. **PASS**.
- Semgrep non disponible sur l'environnement — fallback grep
  effectué.

### Patterns

- **T20 documenté** (`PATTERNS.md` lignes 974-1018) : tech debt
  explicitement tracé avec fix path deux-voies (upstream PR +
  forked path fallback), référence design doc §5.1 et commit
  cible. Conforme au pattern de tracking tech debt. **PASS**.
- **Sprint 19.3 ajouté** (`PATTERNS.md` lignes 1129-1185) :
  pattern canonique TLS pinning documenté avec 4 invariants +
  anti-pattern HPKP + concrete check + référence audit. Conforme
  au format des patterns existants (S18.1, S19.1, S19.2).
  **PASS**.
- **Pattern hot-reload** : implémentation correcte — watch parent
  dir (non le fichier), debounce 50ms, swap `Arc<RwLock<>>`
  atomique, reload fail → keep previous. Aligné avec
  `ConsentWatcher` S16 et `TokenRotator` S18. **PASS**.
- **Pattern primitive/wire/enforcement** : Phase C respecte le
  pattern S19.1 — primitive testée en isolation, wire différée
  T20. **PASS**.
- **`deny_unknown_fields`** sur `RelayPin` et `RelayPinsFile` :
  **PASS** (lignes 212, 250).
- **Pattern drift détecté (P2-2)** : tests env-mutants sans
  `static ENV_GUARD: Mutex<()>`. Fix appliqué avant commit (cf.
  §Verdict ci-dessus).

### Scope-cuts

Liste des scope cuts du kickoff §6 (termes grep-ables) :
`Encryption at rest`, `Duress PIN`, `Rate-limit sliding-window`,
`Kudos-weighted`, `Structured output`, `Client-side redaction`,
`Federated ONG`, `ML-DSA-65`, `Domain fronting`,
`actions/checkout@v4`.

Résultat : **aucun terme scope-cut trouvé** dans les fichiers
stagés Phase C (`tls_pinning.rs`, `RELAY_PIN_BOOTSTRAP.md`,
`PATTERNS.md`, `Cargo.toml`, design doc).

Les modifications à `docs/security/HARDENING_ROADMAP.md` et
`COMPUTE_THREATS.md` sont antérieures (drift méthodologique hors
commit Phase C). Elles touchent des items S23/S25/S26 pour les
clarifier/corriger, mais aucun scope-cut S19 n'est envahi. **PASS**.

### Tests-delta

- **Annoncé** : +17 (surplus par rapport aux +8 du plan §9)
- **Réel mesuré** : `cargo test -p nexus-core-rs tls_pinning` →
  17 passed, 0 failed, 0 ignored
- **Total workspace** : 537 pass (520 baseline Phase B follow-up
  → 537, delta +17 exact)
- **Zéro test skipped/ignored** dans le module `tls_pinning` :
  **PASS**

Delta annoncé = delta réel. **PASS**.

Les 17 tests couvrent :

- Extraction SPKI PEM + DER + garbage input (3 tests)
- Validation matching/mismatch/NoPin/empty-fail-open/expired/
  backup-pin (6 tests)
- Loader valid JSON / missing file / invalid JSON / unknown
  version / unknown field (5 tests)
- Hot-reload write+rename atomic (1 test)
- Path resolution env override + SBFB_HOME fallback (2 tests)

### Research-grounding

Deps nouvelles introduites par le diff Phase C :

| Dep | Version workspace | Résolution Cargo.lock | Trace research |
|---|---|---|---|
| `x509-parser` | `"0.17"` | `0.17.0` | Design doc §5.3 + §7.5 : URL docs.rs + rationale "zero-copy, fuzzed, déjà transitive dep rustls". **PASS** |
| `base64` | `"0.22"` | `0.22.1` | Workspace `Cargo.toml` commentaire Sprint 19 Phase C + design doc §5.3 code snippet. **PASS** |
| `notify` | `"6"` (workspace déjà déclarée) | `6.1.1` | Design doc §4.4 : réutilisation du pattern TokenRotator S18, déjà workspace dep. **PASS** |
| `chrono` | workspace déjà déclarée | — | Design doc §5.3 code snippet + Cargo.toml commentaire "cohérence warrant canary S18 Phase E2". **PASS** |

**Note sur `x509-parser`** : deux versions coexistent dans
`Cargo.lock` (`0.17.0` et `0.18.1`). La `0.18.1` est tirée par une
dep transitive (probablement `rustls` ou `webpki`). Le pin workspace
`"0.17"` résout correctement le package direct de `nexus-core-rs`.
Pas de CVE connue sur `x509-parser 0.17.0` au 2026-04-16 (aucun
advisory rustsec actif trouvé). **PASS**.

**APIs crypto/specs standardisées utilisées** :

- RFC 7469 §2.4 (SPKI pin) : trace explicite dans design doc
  §3.5 + §7.1. **PASS**.
- RFC 5280 §4.1.2.7 (SubjectPublicKeyInfo) : trace dans design
  doc §7.1. **PASS**.
- `base64url` RFC 4648 : trace dans design doc §7.1. **PASS**.

Aucune API crypto nouvelle non-tracée. **PASS**.

### Horizon long-terme + documentation amont

- **Design doc présent** :
  `.planning/research/S19_phase_C_tls_cert_pinning_design.md`
  (~1031 lignes, écrit pre-implementation) — **PASS** sur la
  substance.
- **Alternatives rejetées citées** : D3 du kickoff énumère 4
  alternatives (full cert pin, CA chain, DANE, CT monitoring)
  avec rationale de rejet. Design doc §3 couvre 6 alternatives
  avec analyse détaillée. **PASS**.
- **Solution la plus poussée** : SPKI hash est la solution
  recommandée post-HPKP (OWASP, RFC 7469 §2.4, Tor Browser
  pattern). Le choix `x509-parser` over `spki` crate est justifié
  (§5.3). Pas de crypto maison. **PASS**.
- **Estimations LOC dans plan/kickoff** : le diff
  `sprint19_plan.md` *supprime* les estimations LOC (`~180 LOC`,
  `~320 LOC`, etc.) qui existaient avant Phase C. Les deux
  mentions LOC qui restent dans le design doc (`~150 LOC de iroh
  internals`, lignes 661 et 835) sont des estimations de code
  *tiers à fork*, pas d'estimation du code SBFB produit — cas
  limite acceptable selon §6.7. **PASS**.

---

## Findings

### P2 (fixés avant commit)

- **P2-1 FIXED** :
  `.planning/research/S19_phase_C_tls_cert_pinning_design.md` est
  désormais stagé dans le commit Phase C. La règle "design doc
  AVANT code" est satisfaite dans l'historique git.
- **P2-2 FIXED** : `static ENV_GUARD: Mutex<()>` ajouté au module
  `tests` de `tls_pinning.rs`, acquis en tête des 2 tests
  env-mutants. Pattern aligné avec les 3 modules du crate qui
  mutent déjà des env vars. Race condition cross-module fermée.

### P3 (reporté Phase F wrap-up)

- **P3-1** : `sprint19_plan.md §6.3` ligne 403 contient
  `// "base64url-..." 44 chars` dans le snippet Rust preview —
  la valeur correcte est 43 chars. Le code implémenté est correct
  (`tls_pinning.rs` lignes 218, 278, 594 disent bien 43). La
  discordance est dans le doc planning uniquement, sans impact
  sur la sécurité. Fix cosmétique optionnel en Phase F P3 batch.

---

## Recommendation finale

**Commit Phase C autorisé tel quel après application des 2
mitigations P2**. Le P3-1 est noté pour P3 batch Phase F.

Tests vérifiés post-mitigations :

- `cargo fmt --all --check` : clean
- `cargo clippy --workspace --all-targets --locked -- -D warnings` :
  0 warning
- `cargo test -p nexus-core-rs tls_pinning` : 17 pass / 0 fail /
  0 ignored
- `cargo test --workspace --locked` : 537 pass / 0 fail (pas de
  régression)

La déviation architecturale annoncée (absence d'injection dans
`node.rs`) est correctement documentée dans T20 + module doc +
design doc §5.1. Elle ne constitue pas un finding — c'est une
décision délibérée conforme au pattern "primitive d'abord, wire
ensuite" établi S19 (Phase A S18→S19 + Phase B S19→S20+).
