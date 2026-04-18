# Sprint 20 — Audit plan pour Sprint 21 Phase 0

**Écrit** : 2026-04-18 (Phase F wrap-up S20).
**Cible** : session fraîche Sprint 21 Phase 0 qui joue l'audit gate
avant ouverture kickoff S21.
**Range audit** : commits Sprint 20 `3a7f0a3..<Phase F tip>`.
**Verdict à produire** : `.planning/active/sprint20_audit_findings.md`
(PASS / CONDITIONAL PASS / FAIL — rigor signal G4 : ≥ 1 P2+ documenté
exigé, sinon CONCERN pas PASS).

Ce plan guide l'auditeur sur **7 tracks** (A-F + meta-track Radicle-
v1.0). Pattern permanent depuis Sprint 7. Calibration G4 et Working
tree audit G5 dans `.claude/agents/nexus-phase-auditor.md` et
`.claude/skills/nexus-phase-review/SKILL.md`. **Dimension G8
traceability** à inclure sur Phase E (premier pivot G8 effectif).

---

## Contexte d'entrée S21

- Tip S20 Phase F wrap-up (résolu post-commit dans verification.md §3
  row 1).
- Compteurs tests finals : **Rust 642** / 185 SDK / 213+3 coord / 46
  gov / 241 Vitest / 38 Playwright / 7/7 size / 246+ SPDX
  (~1371 tests, delta S20 +111 vs baseline 1260).
- Audit gate S19 levé via 2 commits `1af90b3..3a7f0a3` (9 P2 + 2 P3
  résolus inline). Verdict PASS. Audit findings à archiver avec le
  wrap-up S20 Phase F.
- **Pre-launch protocol policy codifiée CLAUDE.md** : aucun
  `*_VERSION` bumpé pendant S20, aucun tolerant decoder multi-version
  introduit. Confirmation grep `_VERSION` conseillée au Track-D audit.

## Commits S20 à auditer

```
b7d8d74 chore(gitignore): exclude .claude/narrate-action.lock runtime lockfile
6a3f199 feat(canary): Sprint 20 Phase E — federation foundations + WSS fallback observability
3c18908 chore(hooks): narrate-action mutex to cap Haiku subprocess stacking
e653619 chore(planning): Sprint 20 Phase E — G8 preflight + S1 finding E.6 inline absorption (post-crash re-validation)
b634c23 chore(skill): G8 robustness follow-up — 4 edge cases + auditor G8 traceability dimension
b6da3a4 chore(skill): eliminate node hook process leaks + orphan cleanup on SessionStart
e2e8595 chore(workflow): tighten bootstrap §7.1 G8 references after first real application
bd16e64 chore(planning): Sprint 20 Phase E — pivot G8 to federation foundations + WSS fallback
59225ee chore(workflow): introduce G8 phase pre-flight factual evolution check + nexus-phase-preflight skill
7ea68a6 chore(sprint20): Phase D audit P2-1 follow-up — honest llama_cpp.rs sampler comment
c85397b feat(sprint20): Phase D — structured output dual-backend LlmBackend (Ollama format + llama.cpp llguidance)
2e045f1 chore(planning): Sprint 20 Phase C — audit review archive (P2-C-SEC-1 levé in-phase)
16b94ba feat(sprint20): Phase C — PoW runtime wire gossip subscribe
c32ecb3 feat(sprint20): Phase B — duress PIN (fake keypair noop) + panic wipe 5-tap gesture
7ff22a0 chore(planning): Sprint 20 Phase B — duress PIN + panic wipe design doc
3380f76 fix(sprint18): token_rotation overlap window uses wall-clock to survive short-uptime
667e122 chore(tooling): gitattributes LF hardening + nextest adoption
c12878e chore(skill): sidecar unified terminal (narration tail + delivery reverse channel)
4f4a30a chore(skill): sidecar input terminal (live BTW injection via PostToolUse)
98139e3 chore(skill): narration terminal + gitignore comment fix
05271fa feat(sprint20): Phase A — encryption at rest keypair (Argon2id + AES-256-GCM + double layer OS keyring)
1b1f9cb chore(planning): open Sprint 20 — encryption at rest + duress + panic wipe + PoW wire + structured output + canary auto-publish + dual-transport
```

Plus : le chore `<post-b7d8d74>` fmt fix `frost.rs` residual Phase E
hygiene, et la Phase F wrap-up `<HEAD>` elle-même (ce commit).

---

## Track A — Phase A (encryption at rest keypair)

**Commit** : `05271fa feat(sprint20): Phase A — encryption at rest
keypair (Argon2id + AES-256-GCM + double layer OS keyring)`.

**Pré-revue** : `sprint20_phase_A_review.md` verdict PASS post-fix
(0 P0/P1 + 6 P2 résolus + 3 P3 carried over). Audit session fraîche
doit re-scanner indépendamment, ne pas faire confiance au verdict
review.

### A-1 Design doc `S20_phase_A_encryption_at_rest_design.md` absent

Le commit body `05271fa` annonce `.planning/research/S20_phase_A_encryption_at_rest_design.md`
mais le fichier est **absent** dans `.planning/research/` (vérifié Phase F
row 28 fail-fast). Phase B + D design docs présents, Phase A non.

**Priorité** : P2. Impact : traçabilité des alternatives rejetées (HPKE,
age, TPM, scrypt/PBKDF2/bcrypt) repose uniquement sur kickoff §D1
§D2. Acceptable mais non-conforme §G3 + §6.7 documentation amont.

**Fix attendu** : créer rétroactivement le design doc dans un chore
ou accepter l'absence comme limitation documentée dans kickoff §D1
(6 pages de rationale suffisent peut-être — laisser l'auditeur trancher).

### A-2 aes-gcm vs aws-lc-rs deviation pre-launch

Plan §3.1 documente la déviation NASM Windows build. Tech debt T25
PATTERNS.md §Sprint 20.1 trace le path migration `feature = "fips"`.

**Audit check** : vérifier que T25 est bien référencée depuis
HARDENING_ROADMAP §3 (ligne FIPS compliance T0-T1 beta). Grep
`T25.*PATTERNS` dans `HARDENING_ROADMAP.md`.

### A-3 Bench 82 ms vs target 3 s

T-keystore-bench-reference documente l'écart (RTX 5080 dev = 82 ms).
T26 fix path = bump m=128 MiB ou t=6 après télémétrie Pi 4. Attendu
audit : vérifier que T26 a un owner + deadline visible.

### A-4 `.unwrap()` en prod non-commentés (P3-3 review)

`keystore.rs:860-862` 3× `try_into::<[u8;4]>().unwrap()` sans
commentaire `// infallible: guaranteed by length check above`. P3
carried over. Audit check : voter si P2 ou P3 — l'invariant est
tenu par `blob.len() >= BLOB_HEADER_LEN + TAG_LEN` mais sans
assertion explicite au call-site.

### A-5 `SBFB_IDENTITY_SECRET_HEX` env var exposure T24

Pattern dev/smoke-test. Env var visible dans `/proc/self/environ` +
Windows Task Manager. Documenté T24 PATTERNS §Sprint 20.2. Fix path
= UDS sidecar secret channel (S22+). Audit check : re-confirmer
que le scope dev est clair dans la doc.

---

## Track B — Phase B (duress PIN + panic wipe)

**Commit** : `c32ecb3 feat(sprint20): Phase B — duress PIN (fake
keypair noop) + panic wipe 5-tap gesture`.

**Pré-revue** : `sprint20_phase_B_review.md` verdict PASS-with-carry
(0 P0/P1 + 2 P2 documentés à carry ici).

### B-1 Double-wipe dans `panic_wipe` HTTP handler (P2-B1 review)

`crates/nexus-shell-daemon/src/http.rs:686-710` : le handler appelle
`service.execute()` **puis** `tokio::spawn` avec
`service.execute_and_exit(0)` qui ré-exécute `execute()`. Idempotent
(wipe_all silencieux sur fichiers absents) mais non-intentionnel +
2 lignes log tracing dupliquées.

**Priorité** : P2. **Fix attendu** : remplacer le body du spawn par
`service.exit.exit(0)` direct (ou introduire `execute_then_exit` qui
ne ré-exécute pas le wipe). 3 lignes.

### B-2 CRAFT design doc inclus dans commit phase (P2-B2 review)

`.planning/research/S20_phase_B_duress_panic_design.md` stagé dans le
commit phase au lieu d'un `chore(planning)` séparé préalable.
Dérogation G5 mineure documentée dans body commit section Working
tree audit.

**Priorité** : P2 (discipline G5, pas sécurité). Action : logger ici
et confirmer que la procédure G5 §Working tree audit reste respectée
pour les futures phases.

### B-3 Délta tests +18 vs 20 observés (P3-B3 review)

Body commit annonce `+18` mais review compte 20 tests Phase B dans
le diff. Explication probable : 2 tests existants modifiés plutôt que
nouveaux. Non-bloquant.

### B-4 Timing side-channel `unlock_differential` (kickoff §5 + D3)

Le slot Duress nécessite un deuxième `derive_kek` Argon2id (même
params, PIN différent). L'auditeur peut observer un profil timing
~2x KDF sur un attempt raté pure-chance vs wrong-PIN+duress fallback.
Scope cut documenté S23+ (parallel KDF cancel). Audit check :
vérifier que ce scope cut reste documenté, pas oublié.

---

## Track C — Phase C (PoW runtime wire gossip subscribe)

**Commit** : `16b94ba feat(sprint20): Phase C — PoW runtime wire
gossip subscribe` + `2e045f1 chore(planning): Sprint 20 Phase C —
audit review archive (P2-C-SEC-1 levé in-phase)`.

**Pré-revue** : `sprint20_phase_C_review.md` verdict PASS-with-carry
(0 P0/P1 + 1 P2-SEC-1 **levé in-phase** via le chore `2e045f1`, 1
P2-PLAN-1 carry ici).

### C-1 Plan §6.2 wire-point divergence (P2-C-PLAN-1 review)

Plan §6.2 cite `crates/nexus-shell-daemon-core/src/iroh_runtime.rs::
GossipClient::subscribe()` comme wire-point. Le vrai call-site est
`crates/nexus-shell-daemon/src/runtime.rs::spawn_gossip_subscribe_task`.
`browse.rs::subscribe()` dans `-core` = appel `CuratorRuntime::subscribe`
(gestion attention set, pas gossip transport).

**Priorité** : P2 (traçabilité docs, pas fonctionnel). **Fix attendu** :
Phase F aurait dû corriger §6.2 + §6.4 plan (docs only). Action pour
l'auditeur : vérifier que la correction a été faite dans ce commit
Phase F. Si non, ouvrir un P2 pour chore docs immédiat.

### C-2 `gossip.subscribe\b` grep no-bypass verify

Row 22 fail-fast Phase F : 0 match dans `crates/nexus-shell-daemon-core/
src/` et `crates/nexus-shell-daemon/src/` hors `subscribe_with_pow`.
Audit check : re-courir le grep au moment du gate, confirmer l'absence
de régression.

### C-3 Canary broadcast `main.rs:237` non enveloppé PoW (P3-C3 review)

Documenté dans body commit. Scope Phase E (warrant canary). Non-
bloquant. Audit check : vérifier que Phase E a bien adressé (grep
`canary.*publish` avec/sans `wrap_payload_with_pow`).

---

## Track D — Phase D (structured output dual-backend)

**Commit** : `c85397b feat(sprint20): Phase D — structured output
dual-backend LlmBackend (Ollama format + llama.cpp llguidance)` +
`7ea68a6 chore(sprint20): Phase D audit P2-1 follow-up — honest
llama_cpp.rs sampler comment`.

**Pré-revue** : `sprint20_phase_D_review.md` verdict PASS-with-carry
(0 P0/P1 + 2 P2 dont P2-1 **levé via `7ea68a6`**, P2-2 carry ici).

### D-1 Kickoff §D4 version llguidance 0.7 vs livrée 1.7 (P2-2 review)

Kickoff §D4 spécifie `llguidance = "0.7"`. Livrée `llguidance = "1.7"`
(bump majeur constaté via context7 2026-04-18). Design doc + Cargo.toml
inline comment à jour, kickoff reste 0.7.

**Priorité** : P2 (traçabilité audit). **Fix attendu** : mise à jour
`sprint20_kickoff.md §D4` → `llguidance = "1.7"` + note "bumped à la
session Phase D 2026-04-18". Trivial docs only. Phase F a-t-elle
adressé ? Si non, ouvrir P2.

### D-2 Logit-bias wire llama.cpp absent (P2-1 review, levé)

`apply_matcher_mask` au Sprint 20 ne pousse **pas** de logit-bias au
sampler. Matcher avance via `ff_tokens` + `consume_token` post-sélection.
Validation finale `validate_task_response` = garde-fou effectif. Wire-
level enforcement token-rejeté-avant-sampling = carry S21.

Commit `7ea68a6` a ajouté la note honnête Sprint 20 dans §P30 +
corrigé le commentaire `llama_cpp.rs:307-308`. Audit check : grep
`Sprint 20 état.*logit-bias wire` dans PATTERNS.md, confirmer la note
présente.

### D-3 `serde_json::to_value(RootSchema).expect` sans commentaire INFALLIBLE (P3-1 review)

`crates/nexus-core-rs/src/schemas/task_response.rs:146`. Invariant :
`serde_json::to_value` sur un `RootSchema` est purement structurel →
infaillible. Convention projet : ajouter `// INFALLIBLE: ...`. P3
cosmétique.

### D-4 `expand_tilde` path traversal local (P3-2 review)

`crates/nexus-worker-core/src/llm/llama_cpp.rs:442-448`. Pas de
validation `path.components().any(|c| c == ParentDir)`. Config
locale trust-root opérateur → impact faible. P3 cosmétique. Audit
check : documenter ou valider.

### D-5 Delta tests +27 vs +12 plan (P3-3 review)

Over-delivery +15. Non-bloquant. Reconciliation commit body
recommandée.

---

## Track E — Phase E (warrant canary federation + WSS fallback)

**Commit** : `6a3f199 feat(canary): Sprint 20 Phase E — federation
foundations + WSS fallback observability` + plannings
`bd16e64` (pivot G8 plan update) + `e653619` (preflight post-crash
re-validation).

**Pré-revue** : `sprint20_phase_E_review.md` verdict PASS (0 P0/P1 +
3 P2 documentés à carry ici).

**Dimension supplémentaire obligatoire** : **Pivot retrospective + G8
traceability** (cf. README.md §6.9 garde-fou 7 + skill nexus-phase-
preflight Step 7). Premier pivot G8 effectif du projet.

### E-1 Pivot retrospective G8 (dimension supplémentaire)

Vérifications minimales :

- `.planning/archive/v1.2/sprint20_phase_E_pivot_proposal.md` présent
  et référencé par le commit body Phase E.
- `.planning/archive/v1.2/sprint20_phase_E_preflight.md` présent +
  verdict final SCOPE-CUT-CONSISTENT (post-crash re-validation).
- Commit `bd16e64` antérieur à Phase E code (`6a3f199`) et met à jour
  plan §8 Phase E → pivot Option C.
- Finding S1 E.6 (UDP QUIC probe → WSS fallback observability-only
  car `relay_wss_only` client-side n'existe pas iroh 0.97) absorbé
  inline.
- 7 sous-tâches E.1-E.7 effectivement livrées dans le diff (grep
  fichiers canary/{signer,frost,duress_ack,attestation}.rs +
  transport_probe.rs + canary_registry.py + api/canary.py +
  WARRANT_CANARY_HARDENING.md).
- Dead-man switch intégrité : aucune `CanarySigner::sign` appelée
  par un scheduler/cron/GHA ; CLI manuel uniquement (`sbfb canary`
  + `sbfb canary ack`).

### E-2 `canary_wire_bytes` utilise `serde_json::to_vec` non-JCS (P2-1 review)

**Préexistant à Phase E** (S18 E2 `04c9621` `canary.rs:212-213`
identique). Phase E ne l'introduit pas mais ne le corrige pas.
Pattern SBFB `docs/rust/PATTERNS.md §P-wire sprint 2` préfère JCS
partout.

**Priorité** : P2. Impact sécurité nul (signature couvre canonical_bytes
JCS, broadcast envelope JSON pur) mais ambiguïté cross-language pour
subscribers Python qui re-sérialisent. Fix = migrer vers `serde_jcs::
to_vec` + tech debt entry. Carry S21 ou chore S20 post-F.

### E-3 `CanaryRegistry` sans vérif Ed25519 at ingest (P2-2 review)

`canary_registry.py` `POST /api/canary/observed` accepte des payloads
sans vérifier la signature. Delibéré + documenté (observational-only,
`WARRANT_CANARY_HARDENING.md §2 T-canary-registry-spoof`). Attaquant
local avec bearer token loopback pourrait injecter des observations
fake → masquer un vrai pubkey stale.

**Priorité** : P2. Mitigation actuelle = bearer token loopback + trust
root CANARY.txt bootstrap pubkeys. Fix long-terme = verify Ed25519 at
ingest (tech debt entry). Carry S21 décision : est-ce une maturité
acceptable pre-launch ?

### E-4 LOC dans kickoff §1.2 ambigus (P2-3 review)

Tableau HARDENING_ROADMAP dans kickoff §1.2 contient `~800 LOC`,
`~500 LOC`, etc. Politique `docs/claude/feedback_approach.md`
interdit LOC estimée dans plan (retrospective OK, estimation = P2
anti-pattern).

**Priorité** : P2. Action audit : vérifier la provenance. Si ce sont
des projections roadmap historiques (pré-S20, legitimate), fermer
inline. Sinon, retirer.

### E-5 `expect()` en prod dans FROST impl (review note)

`crates/nexus-shell-daemon-core/src/canary/frost.rs:300` et `:315`
justifiés inline (invariants locaux, trusted_dealer self-produced).
Pattern P26 (expect-as-invariant) respecté. Audit check : voter si
les justifications sont suffisantes ou demander re-phrasing `unreachable!`.

### E-6 `frost.rs:154` formatting residual (Phase E hygiene)

`cargo fmt --all --check` a retourné exit 1 avant Phase F sur 10
lignes de `frost.rs:154`. Fixé via chore séparé pre-Phase F (3 insertions
/ 7 deletions). Audit check : confirmer que le chore est bien présent
dans le range `3a7f0a3..HEAD` et séparé de Phase F docs-only. Question
à l'auditeur : faut-il resserrer la discipline pre-commit pour que
`cargo fmt --check` soit dans `nexus-phase-review` Step 6 ?

---

## Track F — Phase F (wrap-up docs only — ce commit)

**Commit** : `<HEAD>` — chore(sprint20): Phase F — wrap-up + verification
+ audit plan S21 + migrate planning.

### F-1 Sprint20 Phase A design doc absent (row 28 fail-fast)

Cf. A-1 Track A. Body commit Phase A `05271fa` réfère `.planning/
research/S20_phase_A_encryption_at_rest_design.md` jamais écrit.
Question audit : est-ce acceptable rétro ou demander un chore
rétro-actif pour écrire le design doc (alternatives HPKE/age/TPM/KDF
listées dans kickoff §D1-§D2 suffisent-elles) ?

### F-2 Migration `active/` → `archive/v1.2/` (11 fichiers)

Vérifier que `.planning/active/` est **vide** post-commit et que les
11 fichiers + `sprint20_verification.md` + `sprint20_audit_plan.md`
sont tous présents dans `.planning/archive/v1.2/`.

Fichiers attendus :
- sprint20_kickoff.md
- sprint20_plan.md
- sprint20_design_review.md
- sprint20_carry_summary.md
- sprint20_phase_A_review.md
- sprint20_phase_B_review.md
- sprint20_phase_C_review.md
- sprint20_phase_D_review.md
- sprint20_phase_E_review.md
- sprint20_phase_E_pivot_proposal.md
- sprint20_phase_E_preflight.md
- sprint20_verification.md
- sprint20_audit_plan.md

Phase F review sera créé par cette session fraîche S21 Phase 0 (pattern
S18/S19) et ajouté dans un chore séparé post-audit.

### F-3 Memory + CLAUDE.md + SPRINT_LOG.md updated

Audit check :
- `memory/nexus_grid_pivot.md` frontmatter tip sync avec HEAD Phase F.
- `memory/MEMORY.md` row SBFB pivot résumé S20.
- `CLAUDE.md §Etat actuel` ajout Sprint 20 CLOSED + compteurs tests
  finaux + commits + carry Meta-1 S21 + archive path `archive/v1.2/`
  étendu S16-20.
- `docs/claude/SPRINT_LOG.md` row S20 ajoutée sous v1.2.

---

## Meta-track — Radicle-v1.0 activation tracking (re-carry S18→S19→S20→S21)

**Owner** : FlowUP.
**Deadline** : jour du tag `v1.0` go-live (pas de date calendrier —
pattern annual-ish tant que v1.0 pas tag).
**Runbook** : `docs/release/MIRROR_FALLBACK.md §3.1-3.8` (8 sous-
sections self-contained, 5 secrets GHA, action
`gsaslis/mirror-to-radicle@514707f3` v0.2.0).
**Statut S20** : `docs/release/MIRROR_FALLBACK.md` non touché S20.
Audit check : grep `radicle` dans diff S20 `3a7f0a3..HEAD` → vérifier
0 changement wire. Confirmer re-carry explicite S21 dans ce document.

---

## Calibration G4 (rigor signal auditor)

Le verdict final S20 audit est attendu **PASS** si :

- 0 P0 + 0 P1 identifiés.
- ≥ 1 P2+ documenté et résolu OU carry explicite avec owner +
  priorité dans `sprint20_audit_findings.md`.
- Sinon verdict CONCERN (re-auditer dimension manquée).

Pré-revues phases signalent **10 P2** total carry-over S21 candidats
(A-1 design doc, B-1 double-wipe, B-2 CRAFT, C-1 plan divergence,
D-1 kickoff llguidance version, E-2 JCS broadcast, E-3 registry
verify, E-4 LOC kickoff, E-6 frost fmt residual discipline, F-1
design doc A-1). L'auditeur peut en résoudre inline (batch chore)
ou laisser ouvertes pour le gate fix. Ne pas confondre "documenter
un P2" et "le résoudre" (pré-revues phases ≠ audit indépendant
session fraîche).

---

## Fichiers attendus à l'audit session fraîche

L'auditeur Sprint 21 Phase 0 doit pouvoir consulter :

- `.planning/archive/v1.2/sprint20_*.md` (ci-dessus 13 fichiers +
  `sprint20_phase_F_review.md` à créer durant l'audit)
- `.planning/archive/v1.2/sprint19_audit_findings.md` (verdict gate
  S19 levé — déjà migré par Phase F S20)
- Code du range `3a7f0a3..<Phase F HEAD>` (22+ commits)
- `docs/rust/PATTERNS.md` §Sprint 20.1 + §T25 + §T26 + §T27 +
  §P30 + §P31 + §P32 (ajouts S20)
- `docs/shell/PATTERNS.md` section Transport probe + WSS fallback
- `docs/security/HARDENING_ROADMAP.md` (ligne S30 ajoutée + ligne
  S25-30 Niveau 1 enforcement + `last_validated: 2026-04-18`)
- `docs/security/WARRANT_CANARY_HARDENING.md` (nouveau, Phase E.7)
- `docs/security/DURESS.md` (nouveau, Phase B)
- `memory/nexus_grid_pivot.md` frontmatter + SPRINT_LOG.md row S20

---

## Checkpoint S21 kickoff

Le kickoff S21 ne commence **qu'après** :

1. `sprint20_audit_findings.md` livré avec verdict explicite.
2. Tous les P0/P1 levés via commits `fix(sprint20): <track-N>
   description` ou `chore(sprint20): <track-N> batch`.
3. Meta-1 Radicle-v1.0 re-carry S21 confirmé.
4. `sprint20_phase_F_review.md` créé et migré archive.

Pattern audit gate permanent depuis Sprint 7.
