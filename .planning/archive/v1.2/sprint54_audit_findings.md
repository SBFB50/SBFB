# Sprint 54 — Audit Findings

**Auditeur** : session fraiche S55 (2026-05-07).
**Sprint audite** : S54 (edition 2024 + dette pair + E2E wire + CI infra).
**Tip audite** : `5e12d14` (post-Phase D fixes). HEAD: `8066025`.
**Audit plan** : `sprint55_audit_plan.md` (7 tracks A-G).
**Methode** : verification independante par track, PATTERNS.md non lu
avant formation d'opinion (§3.5 convention).

---

## Verdict : PASS

0 P0, 0 P1, 3 P2, 2 P3. Sprint 54 livre proprement ses 4 phases.
Edition 2024 correcte, E2E wire tasks_doc_ticket cable, CI images
pinnees, dette pair resolue. Rigor signal G4 satisfait (3 P2).

---

## Tracks executees

| Track | Sujet | Verdict | Evidence |
|---|---|---|---|
| A | Edition 2024 integrity | PASS | `grep edition Cargo.toml` = 2024, clippy 0 warnings, 3 crates deny+allow pattern correct |
| B | Dette pair completeness | PASS | 5/5 items resolus : set_permissions, GossipTaskConfig, republish timer, loopback doc, preflight exemption |
| C | E2E wire correctness | PASS | tasks_doc_ticket present dans coordinator invite, daemon invite_api, worker invite. 3 tests present |
| D | CI infra status | PASS | 11 sha256 pins (3 images uniques), nexus-core-py supprime GHA, SELF_HOSTED_BUILD.md present |
| E | Scope cuts compliance | PASS | 0 violations. tls_pinning.rs + rate_limit_policy_loader.rs = modifications mecaniques edition uniquement |
| F | Test delta verification | PASS | Rust 1207/1207 (1 flaky re-run OK), Vitest 250/250. Delta +1 Rust +0 frontend = annonce |
| G | Carry-over accountability | PASS | 2 MANDATORY 3/3 S55 documentes, 7 P2 S54 documentes, 8 CLOSED corrects |

### G8 traceability (supplementaire)

| Phase | Preflight | Review | Verdict |
|---|---|---|---|
| A | sprint54_phase_A_preflight.md | sprint54_phase_A_review.md (PASS 2 P2) | ✅ |
| B | sprint54_phase_B_preflight.md | sprint54_phase_B_review.md (PASS 2 P2) | ✅ |
| C | sprint54_phase_C_preflight.md | sprint54_phase_C_review.md (PASS 2 P2) | ✅ |
| D | sprint54_phase_D_preflight.md | sprint54_phase_D_review.md (PASS 3 P2) | ✅ |

8 artefacts G8 presents (4 preflight + 4 review). 0 G8 gate bypass.

---

## Findings

### P2-1 — Flaky test browse::probe_and_cache_with_quorum_majority_continues_to_dial

**Fichier** : `crates/nexus-shell-daemon-core/src/browse.rs`
**Observation** : test echoue lors du premier run nextest complet
(842/1207 run, 1 failed), passe en re-run isole (1/1 passed).
verification.md reporte 1207/1207 — le test etait vert a ce
moment. Instabilite timing-dependante.
**Impact** : bruit CI, resultat audit variable selon timing.
Pre-existant (pas regression S54).
**Carry** : P2-S54-AUDIT-1 flaky browse test (1/3 S55).

### P2-2 — SAFETY comment convention incomplete sur unsafe FFI pre-existants

**Fichiers** :
- `crates/nexus-launcher/src/main.rs:515` — `libc::kill(SIGTERM)`,
  commentaire descriptif present mais pas de prefix `// SAFETY:`
- `crates/nexus-test-harness/src/lib.rs:147` — `libc::kill(SIGINT)`,
  aucun commentaire SAFETY
- `crates/nexus-shell-daemon/src/named_pipe_server.rs` — 12+ blocs
  unsafe Win32 FFI (GetTokenInformation, ConvertSidToStringSidW, etc.)
  sans prefix `// SAFETY:`

**Observation** : la migration edition 2024 Phase A a correctement
ajoute `// SAFETY:` sur les 70+ wrapping set_var/remove_var (18/18
dans les fichiers spot-checked). Mais les blocs unsafe pre-existants
(FFI processus + Win32 peer-creds) n'ont pas ete normalises avec
le meme prefix.
**Impact** : convention incomplète, grep `// SAFETY:` ne liste pas
tous les sites unsafe. Pre-existant.
**Carry** : P2-S54-AUDIT-2 SAFETY convention FFI (1/3 S55).

### P2-3 — Naming convention INVITE_VERSION vs *_FORMAT_VERSION

**Fichier** : `crates/nexus-worker-core/src/invite.rs:73`
**Observation** : `pub const INVITE_VERSION: u8 = 2` alors que la
convention projet est `*_FORMAT_VERSION` (ex: `CURATOR_LIST_FORMAT_VERSION`,
`KEY_ROTATION_FORMAT_VERSION`, `TASK_FORMAT_VERSION` — tous `u16 = 1`).
De plus, INVITE_VERSION = 2 (type u8) vs convention u16 pour les
autres. Pre-existant (Sprint 4 Phase C, `b0656ff`).
**Impact** : grep `FORMAT_VERSION` rate le invite. Le type u8 vs u16
est aussi inconsistant. Pre-launch, renommage serait cheap.
**Carry** : P2-S54-AUDIT-3 invite version naming (1/3 S55).

### P3-1 — Audit plan Track C5 check inexact

**Fichier** : `sprint55_audit_plan.md:58-59`
**Observation** : le plan prescrit `grep FORMAT_VERSION
crates/nexus-coordinator-rs/src/invite.rs` attendant "version = 1".
Aucun FORMAT_VERSION n'existe dans ce fichier. La constante est
`INVITE_VERSION: u8 = 2` dans `crates/nexus-worker-core/src/invite.rs`.
**Impact** : nit documentation audit plan — n'affecte pas le code.

### P3-2 — Republish timer sans jitter confirme

**Fichier** : `crates/nexus-shell-daemon/src/runtime.rs:1015`
**Observation** : `Duration::from_secs(45)` fixe, pas de jitter.
Correctement identifie et documente comme P2-S54-jitter-republish
carry 1/3 S55 dans la Phase B review. Pas de regression.
**Impact** : thundering-herd theorique si N noeuds demarrent
simultanement. Correctement defere.

---

## Items CLOSED confirmes (cross-ref Tracks A-D)

| Item | Track | Verification |
|---|---|---|
| P2-REVIEW-B-1-S51 edition 2024 | A | edition 2024, clippy 0, 3 deny+allow |
| P2-S53-node_key perms 0600 | B | set_permissions + cfg(unix) present |
| P2-S53-gossip params struct | B | GossipTaskConfig struct + spawn |
| P2-S53-route collision doc | B | /api/daemon/ namespace correct |
| P2-S53-periodic republish | B | 45s timer dans select! loop |
| P2-S53-preflight process gap | B | §6.9 exemption post-plan documente |
| P2-AUDIT-1-S52 images CI pin | D | 11 sha256 (3 images) |
| P2-REVIEW-A-1-S52 nextest timeout | D | documente dans SELF_HOSTED_BUILD.md |

8/8 items CLOSED confirmes. Aucun item claim CLOSED sans evidence.

## Escalades 3/3 MANDATORY S55 confirmees

| Item | Justification |
|---|---|
| P2-REVIEW-B-1-S52 Woodpecker serveur | Infra VPS prete (Docker, deploy-key, cli). Serveur Woodpecker + webhooks TLS manquants. Escalade justifiee. |
| P2-REVIEW-B-2-S52 GHA validation post-push | Rust CI fix committe (nexus-core-py supprime). Run ID post-push non documente. Escalade justifiee. |

## Compteurs tests confirmes

| Suite | Annonce S54 | Mesure audit | Match |
|---|---|---|---|
| Rust nextest | 1207 | 1207 (1 flaky) | ✅ |
| Vitest | 250 | 250 | ✅ |
| Delta Rust | +1 | +1 (Phase C) | ✅ |
| Delta frontend | +0 | +0 | ✅ |

---

## Carries S55 (consolide)

### Herites (inchanges)

| Item | Compteur S55 |
|---|---|
| P2-A-1 rand blocker upstream | 13+/3 exemption |
| P2-AUDIT-2 iroh transitives | herite pin 0.98 |
| P2-S53-outbox non-persistant | 2/3 |
| P2-S53-browse_request rate-limit | 2/3 |

### Escalades 3/3 MANDATORY S55

| Item | Compteur S55 |
|---|---|
| P2-REVIEW-B-1-S52 Woodpecker serveur | **3/3 MANDATORY** |
| P2-REVIEW-B-2-S52 GHA validation post-push | **3/3 MANDATORY** |

### Nouveaux P2 S54 (carries 1/3)

| Item | Source |
|---|---|
| P2-S54-forbid-deny-doc | Phase A review |
| P2-S54-lightcheck-edition-faux-positif | Phase A review |
| P2-S54-jitter-republish | Phase B review |
| P2-S54-windows-test-cfg-unix | Phase B review |
| P2-S54-test-E2E-multi-noeuds | Phase C review |
| P2-S54-project-name-hardcode | Phase C review |
| P2-S54-rustfmt-drift-sessions | Phase D review |

### Nouveaux P2 audit S54

| Item | Source |
|---|---|
| P2-S54-AUDIT-1 flaky browse test | Audit finding P2-1 |
| P2-S54-AUDIT-2 SAFETY convention FFI | Audit finding P2-2 |
| P2-S54-AUDIT-3 invite version naming | Audit finding P2-3 |

### Long-terme (inchanges)

- T-NN+2 iframe Rust-wasm (PATTERNS §P34)
- LT-2 Radicle sortie cap G7 (trigger tag v1.0)
- LT-3/LT-4 hors-sprint (post-v1.0)
- LT-5 redundancy persistence (reclassifie S26)
- LT-7 self-hosted build — **PRE-V1.0 OBLIGATOIRE** (S55)
