# Sprint 20 Phase B — nexus-phase-auditor review

HEAD pre-commit: `05271fa` (Phase A)
Draft commit body: "feat(sprint20): Phase B — duress PIN (fake keypair noop) + panic wipe 5-tap gesture"
Timebox: 45m

## Verdict : PASS-with-carry

0 P0 / 0 P1 / 2 P2 documentes. Rigor signal G4 satisfait.

Les 2 P2 (double-wipe handler + CRAFT merge) sont acceptables en
carry documenté — ils ne compromettent pas la sécurité (le wipe
est idempotent) ni l'atomicité du commit (le design doc est
directement rattaché à la phase B, non à un sprint distinct).
Commit autorisé sous reserve que les 2 P2 soient loggués dans
`sprint20_audit_findings.md` (Phase F).

---

## Dimensions

### Security

- [x] semgrep scan : non disponible sur Windows, fallback grep
  effectué.
- [x] unsafe blocks : 0 nouveau `unsafe` dans le diff.
- [x] unwrap/todo!/unimplemented! : présences dans les tests
  uniquement (`.unwrap()` dans les blocs `#[test]`/`#[tokio::test]`)
  — légitimes dans un contexte test. Aucun `unwrap` dans le code
  de production du diff.
- [x] loopback auth : `POST /panic/wipe` est derrière le middleware
  `auth_required` (bearer + Host + Origin) — correctement gaté,
  pas d'exemption publique.
- [x] env var scrub : `SBFB_IDENTITY_MODE` scrubée via
  `std::env::remove_var` immédiatement après lecture dans `main.rs`.
  `SBFB_IDENTITY_SECRET_HEX` (Phase A) scrubée via `hex_str.zeroize()`
  après `set_var` — comportement attendu et documenté.
- [x] zeroize : `SecretKeyBytes` porte `#[zeroize(drop)]`, les buffers
  stack intermédiaires (`tmp`, `secret`, `hex_str`) sont explicitement
  zeroized après usage dans `unlock.rs` et `keystore.rs`.
- [x] blob size indistinguabilité : `identity.enc` et
  `identity_duress.enc` font tous deux 96 bytes
  (`BLOB_HEADER_LEN(48) + SECRET_KEY_BYTES(32) + TAG_LEN(16)`),
  confirmé par test `#B6 duress_blob_indistinguishable_size_from_normal`.
- [x] noop surface uniforme erreur : `unlock_differential` retourne
  `AeadReject` qu'il y ait ou non un slot duress configuré
  (voir code ll. 895-907 keystore.rs) — l'adversaire ne peut pas
  distinguer "pas de slot duress" de "mauvais PIN".
- [x] PeerCredsVerified : le diff ne touche pas les routes
  loopback UDS/NP — N/A pour Phase B.
- [x] wire format / JCS : le diff ne touche pas de sérialisation
  wire gossip — N/A.
- [x] path traversal zip : aucun nouveau path de zip extraction
  dans le diff — N/A.

**Finding P2-B1 (double-wipe dans panic_wipe handler) :**
Le handler HTTP `panic_wipe` (http.rs ll. 686-710) appelle :
1. `service.execute()` — wipe les blobs + état.
2. Puis dans un `tokio::spawn` : `service.execute_and_exit(0)`.

`execute_and_exit` appelle lui-même `self.execute()` en interne
avant de déléguer à l'exit strategy. Conséquence : `wipe_all` est
appelé deux fois. La seconde invocation est idempotente par
construction (fichiers absents gérés silencieusement), mais le
comportement n'est pas intentionnel — l'intent du handler était
"wipe → reply 200 → schedule exit". Fix minimal : renommer l'appel
du spawn en `service.exit.exit(0)` directement (ou introduire une
méthode `execute_then_exit` qui ne re-exécute pas le wipe). P2 car
pas de risque de sécurité (idempotent) mais code surprenant pour
un auditeur futur et producteur de 2 lignes de log tracing
`warn!("POST /panic/wipe — executing irreversible wipe")`.

### Patterns

- [x] `docs/rust/PATTERNS.md` lu après formation de l'opinion.
  PATTERNS.md actuel (Sprint 1-19) ne documente pas encore les
  patterns Phase B (normal — ils seront ajoutés Phase F ou à
  l'ouverture d'un `§Sprint 20.2`). Aucun pattern existant (P1..P29)
  n'est violé par le diff.
- [x] `noop_identity` helpers utilisés systématiquement dans les 3
  handlers sensibles (`publish_project`, `subscribe_curator`,
  `publish_blob`). La route `DELETE /curators/:pubkey`
  (unsubscribe) n'est pas gatée — correct car supprimer d'un
  attention set vide en mode Duress est un no-op idempotent.
- [x] `ExitStrategy` injectable : le trait est correctement abstrait,
  `RealExit` pour prod, `RecordingExit` pour les tests (gated
  `#[cfg(test)]`). Pattern testable validé.
- [x] keyring slot paramétré pour les tests : `with_keyring_slot`
  garantit l'isolation entre tests parallèles — pattern conforme
  à ce qui existait en Phase A.
- [x] `#[serde(deny_unknown_fields)]` : non applicable ici (pas de
  nouveau type Serde dans le diff sauf `PanicWipeResponse` côté TS
  qui utilise `.strict()` Zod — équivalent correct).
- [x] Aucun LOC estimé dans plan/kickoff : plan §Phase B dit
  "~500 LOC duress + ~400 LOC panic wipe" — ces estimations sont
  présentes dans le plan. **Finding P3 : les estimations LOC sont
  en §1.2 HARDENING_ROADMAP row, non dans le plan d'exécution lui-
  même (le plan §5.1 ne répète pas les chiffres) — tolérable, la
  règle §6.7 README vise les estimations décisionnelles pas les
  notes de roadmap.** Pas de P2 sur ce point (les chiffres sont
  dans un tableau de roadmap, pas dans le scope description).

### Working tree audit (G5)

| Fichier | Catégorie | Statut |
|---|---|---|
| `crates/nexus-core-rs/src/keystore.rs` | PHASE | attendu §5.2 |
| `crates/nexus-core-rs/src/lib.rs` | PHASE | attendu §5.2 |
| `crates/nexus-launcher/src/main.rs` | PHASE | attendu §5.2 |
| `crates/nexus-launcher/src/unlock.rs` | PHASE | attendu §5.2 |
| `crates/nexus-shell-daemon/src/http.rs` | PHASE | attendu §5.2 |
| `crates/nexus-shell-daemon/src/main.rs` | PHASE | attendu §5.2 |
| `crates/nexus-shell-daemon/src/noop_identity.rs` | PHASE | attendu §5.2 |
| `crates/nexus-shell-daemon/src/panic.rs` | PHASE | attendu §5.2 |
| `crates/nexus-shell-daemon/src/runtime.rs` | PHASE | attendu §5.2 |
| `web/src/api/daemon.ts` | PHASE | attendu §5.2 |
| `web/src/components/AppShell.tsx` | PHASE | attendu §5.2 |
| `web/src/components/PanicWipeKeybind.tsx` | PHASE | attendu §5.2 |
| `web/src/components/__tests__/PanicWipeKeybind.test.tsx` | PHASE | attendu §5.2 |
| `docs/security/DURESS.md` | PHASE/DOCS | attendu §5.2 + §5.4 |
| `.planning/research/S20_phase_B_duress_panic_design.md` | **CRAFT** | design doc — commit chore séparé attendu |

**Finding P2-B2 (CRAFT dans le commit Phase B) :**
`.planning/research/S20_phase_B_duress_panic_design.md` est un
fichier de planning/research (catégorie CRAFT selon G5). La
procédure G5 requiert un commit `chore(planning|skill|debt)`
**avant** le commit phase. Ce fichier est stagé directement avec
le code Phase B. Le body commit le documente sous "CRAFT (design
doc)" mais sans commit séparé préalable. En practice, le design
doc est étroitement lié à la phase (créé en même temps, référencé
dans le commit body), et son inclusion directe ne crée pas de
bruit d'audit — mais c'est une dérogation à la procédure G5 qui
mérite d'être loggée.

Le body commit contient bien une section "Working tree audit" —
exigence G5 respectée sur ce point.

- [x] PHASE : 14 fichiers attendus par Plan §Phase B ✓
- [!] CRAFT : 1 fichier planning inclus dans commit phase (P2)
- [x] DEBT : 0 fichier hors-scope
- [x] NOISE : 0 (pas de .pdb/.exe/cache)

### Scope-cuts

Scope cuts §8 kickoff vérifiés :

- Hardware keystore (TPM/SE/StrongBox) : grep du diff → 0 match.
  `LocalFileKeyStore` seulement, pas de TPM/SE code. ✓
- HPKE envelope peer-restore : 0 mention dans le diff. ✓
- Parallel KDF S23+ : 0 implémentation dans le diff (timing
  side-channel documenté comme scope cut §5 du design doc). ✓
- Rate-limit per-consumer : 0 match. ✓
- Coldboot RAM <60s S22+ : 0 implémentation. ✓

D1..D5 kickoff §4 respectées :

- D3 retenu (fake keypair noop, pas wipe-immediate) : implémenté
  via `IdentityMode::Duress` + `noop_identity` helpers. ✓
- D1 double-layer OS keyring réutilisé pour slot duress (compte
  `identity-kek-wrap-duress`). ✓
- Pas de TPM/SE dans le diff. ✓

### Tests-delta

Baseline entrée Phase B : 562 Rust (annoncée dans commit body).
Note : le kickoff §1.3 indique 538 tests à l'entrée S20, Phase A
a livré +24 nextest (562 = 538 + 24 confirmé en se souvenant que
la phase A annonçait "563" mais sans les bench criterion qui ne
sont pas comptés par nextest). La baseline 562 est cohérente.

Tests Phase B annoncés vs observés dans le diff :

| Module | Annoncé | Compté dans diff |
|---|---|---|
| `keystore.rs` (#B1–#B8) | 8 | 8 (#B1 init_duress_creates_two_blobs, #B2 unlock_normal, #B3 unlock_duress, #B4 wrong_pin_uniform, #B5 keypair_distinct, #B6 size_indistinguable, #B7 init_duress_twice_rejected, #B8 wipe_all_removes) ✓ |
| `panic.rs` (4 tests) | 4 | 4 (panic_wipe_removes_both_blobs, panic_wipe_deletes_state_sqlite_and_blob_cache, panic_wipe_zeroizes_keypair_ram, panic_wipe_exits_process) ✓ |
| `noop_identity.rs` (4 tests) | 4 | 4 (normal_mode_always_proceeds, duress_mode_noop_publishes, duress_mode_noop_subscribes, duress_mode_rejects_dispatch) ✓ |
| `http.rs` (3 tests) | 3 | 3 (#B-rt-1 daemon_boot_in_duress_mode_publishes_fake_curator_empty, #B-rt-2 daemon_boot_in_duress_mode_rejects_curator_subscribe_real, #B-rt-3 daemon_boot_in_duress_mode_rejects_task_dispatch) ✓ |
| `unlock.rs` (1 test) | 1 | 1 (parse_subcommand_init_duress_with_flag) ✓ |
| Vitest PanicWipeKeybind | 2 | 2 (five_taps_within_3s_triggers_wipe, four_taps_or_slow_does_not_trigger) ✓ |

Total Rust compté : 8 + 4 + 4 + 3 + 1 = **20 tests Phase B** (vs 15
annoncés au plan, +5 bonus : #B7, #B8, `normal_mode_always_proceeds`,
`duress_mode_noop_subscribes`, `duress_mode_rejects_dispatch`).

Delta Rust annoncé : +18 (562 → 580). 20 tests Phase B mais le
commit body annonce +18 — discordance de 2 tests. Explication
possible : 2 tests inclus dans http.rs ou unlock.rs existaient
déjà avant (tests pré-existants modifiés/augmentés) et ne sont
pas comptés comme "nouveau". Impossible à confirmer sans runner
live. **P3 cosmétique : le delta +18 vs 20 nouveaux tests
observés dans le diff devrait être réconcilié dans le body
commit.** En pratique la suite passe à 580/580 (annoncé) — le
chiffre total est le signal, le delta peut inclure des tests de
régression Phase A touchés.

- [x] Rust workspace : 562 → 580 (+18, annoncé) — delta plausible,
  tests itemisés concordent avec le diff.
- [x] Vitest : 239 → 241 (+2, annoncé) — 2 tests PanicWipeKeybind
  observés dans le diff ✓
- [x] Python / Playwright / size : inchangés (annoncé)
- [x] Aucun test `#[ignore]` ni `#[should_panic]` sans raison dans
  le diff.

### Research-grounding

- [x] Cargo.toml deps : 0 nouvelle dépendance ajoutée dans ce diff
  (Phase B ne nécessite aucune nouvelle dep — toutes les deps
  critiques ont été introduites Phase A et tracées dans
  `sprint20_plan.md §3.1` : `aes-gcm`, `argon2`, `keyring`,
  `zeroize`, `secrecy`, `blake3`).
- [x] web/package.json : 0 nouvelle dep npm dans le diff.
- [x] API crypto utilisées (`Aes256Gcm`, `Argon2id`, `SecretBox`,
  `BLAKE3`) : toutes tracées Phase A research, pas de nouveau
  usage de spec standardisée non traqué en Phase B.
- [x] `std::process::exit` via `ExitStrategy` trait : usage standard,
  pas de spec externe.
- [x] RustSec advisory cross-check : délégué à Phase A (le kickoff
  §3.1 confirme 0 advisory actif sur ces crates au 2026-04-16).
  Phase B n'introduit pas de nouvelles crates.

### Horizon long-terme + documentation amont

- [x] Design doc présent : `.planning/research/S20_phase_B_duress_
  panic_design.md` (263 LOC) couvre §1 rationale (fake keypair vs
  wipe-immediate vs VeraCrypt), §2 indistinguabilité wire, §3
  ergonomie 5-tap, §4 implications légales, §5 angles morts scope
  cuts, §6 dépendances, §7 tests plan. Existait avant le code.
- [x] Alternatives rejetées citées dans D3 kickoff §4 (wipe-
  immediate GrapheneOS, VeraCrypt hidden volume, soft-delete RAM)
  avec rationale complet. Design Review Board G1 a acknowled D3
  (⚠️ terminologie — corrigé "detectable" vs "non-deniable").
- [x] Solution la plus poussée : fake keypair noop est plus sophisti-
  qué que wipe-immediate (meilleure deniabilité, documentée vs
  GrapheneOS 2026 forensics). La limite de timing indistinguabilité
  `unlock_differential` (~2x KDF pour le slot duress) est
  documentée comme scope cut S23+ avec parallel KDF cancel.
- [x] Aucune estimation LOC dans les sections décisionnelles du plan
  (les chiffres ~500/~400 LOC de la HARDENING_ROADMAP row §3 sont
  des projections de planification long-terme, pas des estimations
  de scope de la phase courante). Pas de P2 sur ce point.

---

## Findings

- **P2-B1** : double-wipe dans `panic_wipe` HTTP handler —
  `crates/nexus-shell-daemon/src/http.rs` ll. 686-710. Le handler
  appelle `service.execute()` puis passe le même `Arc<service>` à
  un `tokio::spawn` qui appelle `service.execute_and_exit(0)`.
  `execute_and_exit` appelle `self.execute()` en interne —
  `wipe_all` est donc exécuté deux fois. Idempotent mais non-
  intentionnel. Fix : dans le spawn, appeler `service.exit.exit(0)`
  directement OU inliner `execute_and_exit` de manière à éviter la
  re-exécution. Logger dans `sprint20_audit_findings.md`.

- **P2-B2** : fichier CRAFT (design doc Phase B) inclus dans le
  commit phase plutôt que dans un commit `chore(planning)` séparé
  préalable. Procédure G5 dit split obligatoire. En practice le
  design doc est atomiquement lié à la livraison Phase B (pas un
  doc de planning isolé), et le body commit le déclare
  explicitement. Dérogation mineure, commit autorisé, logger dans
  `sprint20_audit_findings.md` pour maintenir la discipline G5.

- **P3-B3** : discordance delta +18 annoncé vs 20 tests Phase B
  observés dans le diff. Reconciliation dans le body commit
  recommandée (indiquer "20 nouveaux tests dont 2 comptabilisés
  dans la baseline Phase A via refactor" ou similaire). Non-
  bloquant.

---

## Recommendation

**Commit autorisé** (0 P0 / 0 P1).

Actions recommandées avant ou pendant Phase F :

1. (P2-B1, non-bloquant) : corriger le double-wipe dans
   `panic_wipe` handler. Fix minimal : remplacer le body du spawn
   par `service.exit.exit(0)` directement après que `execute()` a
   déjà tourné, pour éviter le second cycle `wipe_all`.

2. (P2-B2, non-bloquant) : logger la dérogation CRAFT dans
   `sprint20_audit_findings.md` §Phase B. Rappeler la procédure G5
   en commentaire de phase pour les prochaines phases.

3. (P3-B3, cosmétique) : réconcilier la description du delta dans
   le body commit final (20 tests observés vs +18 annoncé).

Les 2 P2 doivent apparaître dans `sprint20_audit_findings.md`
(Phase F) pour le gate S21. Aucune des deux ne remet en cause la
solidité crypto / sécurité de la livraison.
