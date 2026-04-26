# Sprint 28 — Audit plan (pour S29 Phase 0)

**Date** : 2026-04-26
**Tip sortie S28** : sera le commit Phase E (post-migration)
**Auditeur** : session fraiche S29 Phase 0 (pas la meme session)

---

## 1. Mode d'emploi

L'auditeur est une session Claude Code fraiche. Ordre de lecture :

1. Ce fichier (`sprint29_audit_plan.md`)
2. `CLAUDE.md` (racine) — projet + pointeur workflow
3. `docs/claude/README.md` §3 + §8 (audit gate + comment auditer)
4. NE PAS lire les phase reviews (`sprint28_phase_{A..D}_review.md`)
   ni `sprint28_verification.md` AVANT d'avoir forme une opinion
   independante sur chaque Track
5. NE PAS rebattre les D1..D5 gelees du `sprint28_kickoff.md` §4

Timebox suggeree : 2-3h.
Delivrable : `.planning/active/sprint28_audit_findings.md` avec
verdict PASS / CONDITIONAL PASS / FAIL.

---

## 2. Dimensions d'audit

### Track A — Watermark end-to-end wiring (Phase A `c5f35f7`)

1. **WIRE-1 compute_bias call site** : verifier que `llama_cpp.rs`
   appelle `compute_bias` dans le sampling loop quand
   `watermark.enabled = true`. Verifier que le bias n'est PAS applique
   quand `enabled = false` (gate `should_inject` ou equiv).
2. **WIRE-2 output_token_ids** : verifier que `runtime.rs` populate
   `output_token_ids` dans `TaskResult`/`GenerateResponse` avec les
   tokens effectivement generes (pas `vec![]` quand watermark actif).
   Verifier le serde roundtrip.
3. **CONFIG-1 watermark.toml.sample** : verifier que le fichier est
   parsable TOML et que les valeurs par defaut (enabled=false,
   delta=2.0, window_size=4) sont documentees en commentaire.
4. **SEED-1 trust_web_seeds.toml** : verifier que le fingerprint dummy
   `000...` a ete remplace par un placeholder etiquete `# PLACEHOLDER`
   (pas du zero-padding silencieux).
5. **P37-1 PATTERNS.md** : verifier que P37 mentionne `watermark.rs`
   comme source primaire ET `llama_cpp.rs` comme call site
   d'integration (les deux, pas un seul).

**P2 carry de Phase A review a verifier** :
- P2-REVIEW-1 : `generate_blocking` a 12 params. Verifier si
  `#[allow(clippy::too_many_arguments)]` est present et justifie.
- P2-REVIEW-2 : sampler chain rebuild per-step. Verifier si le hot
  path est acceptable pour < 100 req/s inference.

### Track B — Platform writers + ONNX fixture (Phase B `a43a1a1`)

1. **JOURNALD-1 cfg gate** : verifier que `JournaldWriter::write_event`
   est impl `#[cfg(target_os = "linux")]` et que le stub
   `#[cfg(not(target_os = "linux"))]` est preservee.
2. **OSLOG-1 cfg gate** : meme verification pour
   `OsLogWriter::write_event` avec `#[cfg(target_os = "macos")]`.
3. **FORMAT-1 structured fields** : verifier que `format_journal_fields`
   produit les 4 champs attendus (MESSAGE, PRIORITY, SBFB_EVENT_TYPE,
   SBFB_DETAILS) et que `format_oslog_message` produit un message
   lisible.
4. **ROUTING-1 init_platform_emitter** : verifier que la logique
   selectionne automatiquement le writer natif selon la plateforme
   (Linux → Journald, macOS → OsLog, other → Tracing).
5. **ONNX-1 mock** : verifier que le mock InferenceSession dans
   wrapper.test.ts exerce le pipeline PII (decodeSpans + greedyDedup +
   toFinding). Option B retenue (mock, pas fixture ONNX reelle) —
   verifier que le TODO S29 est documente si applicable.
6. **EVENT-1 event_type_name** : verifier que la fonction couvre
   toutes les variantes SecurityEvent (pas de match catch-all `_ =>`
   silencieux).

**P2 carry de Phase B review a verifier** :
- P2-B-1 : impls natives non testees fonctionnellement (Windows dev).
  Verifier que les format helpers compensent.
- P2-B-2 : `init_platform_emitter()` sans test direct. Verifier la
  trivialite (< 10 LOC, 3 branches cfg).

### Track C — Process isolation design doc (Phase C `ccbb6ca`)

1. **DOC-1 completude** : verifier que PROCESS_ARCHITECTURE.md contient
   au minimum 9 sections (intro, archi, IPC, lifecycle, state, fault,
   security, migration, questions).
2. **IPC-1 JSON-RPC justification** : verifier la presence d'une
   analyse comparative JSON-RPC vs gRPC avec chiffres
   (latence/serialization). Le kickoff D3 ⚠️ demandait cette analyse.
3. **COLD-1 budget** : verifier que le cold-start < 5s est documente
   comme cible avec benchmark RTX 5080 comme prereq S29.
4. **SECURITY-1 privilege reduction** : verifier que le doc specifie
   que l'executor n'a PAS acces au keypair identity (separation
   privileges broker/executor).
5. **FAULT-1 crash isolation** : verifier que le doc decrit le scenario
   crash executor sans crash broker (avec backoff re-spawn).

**P2 carry de Phase C review a verifier** :
- P2-C-1 : blob-serve reste dans le broker a S29. Verifier que le gap
  est documente dans le doc (§7.1 ou §9).
- P2-C-2 : benchmark cold-start non mesure. Verifier que c'est un
  prereq explicite dans le doc.

### Track D — External audit scope + HARDENING_ROADMAP (Phase D `727a780`)

1. **SCOPE-1 in/out** : verifier que EXTERNAL_AUDIT_SCOPE.md distingue
   clairement scope in (crypto, wire, auth, transport, sandbox) et
   scope out (UI React, docs, CI, tests).
2. **VENDOR-1 matrix** : verifier la presence d'une comparaison
   Cure53/Trail of Bits avec au minimum criteres focus/budget/duree.
3. **ROADMAP-1 S28 line** : verifier que HARDENING_ROADMAP §3 S28
   reflete le sprint reel (watermark wiring + dette + design docs) et
   non l'aspirationnel initial (Nym + MIG + D2/D3/C4).
4. **ROADMAP-2 Nym carry** : verifier que §3 S30 mentionne Nym avec
   note de deferral "G9 2026-04-25 SDK beta".
5. **ROADMAP-3 last_validated** : verifier que `last_validated` est
   mis a jour a 2026-04-26 (ou 2026-04-25 selon la date du commit).
6. **GATE3-1** : verifier que Gate 3 checklist items S28 sont
   documentes (watermark wiring, PROCESS_ARCHITECTURE, audit scope).

**P2 carry de Phase D review a verifier** :
- P2-D-1 : S29-S30 sans "Note realisme". Verifier dans §3.
- P2-D-2 : versions crates sans note "at S28, verify at engagement".

### Track E — G1 Design Review Board

1. Verifier que `sprint28_design_review.md` existe dans
   `.planning/archive/v1.2/` (post-migration).
2. Verifier que le scoring D1-D5 est present (5 scores ✅/⚠️/❌).
3. Verifier que le kickoff §4 "Acknowledged review findings" repond
   a chaque ⚠️.
4. Rigor signal G4 : au moins 1 ⚠️ attendu (pas de rubber-stamp 5/5).

### Track F — G8 traceability

1. Verifier que les 4 phases A-D ont chacune un
   `sprint28_phase_{X}_preflight.md` dans `.planning/archive/v1.2/`.
2. Verifier que les 4 phases A-D ont chacune un
   `sprint28_phase_{X}_review.md` dans `.planning/archive/v1.2/`.
3. Verifier la coherence verdict G8 × commit (4 EXECUTE → 4 commits
   phase livres, 0 DESIGN-CONFLICT → 0 pivot_proposal).
4. Phase review files present : 4/4 reviews (A, B, C, D).
   Ratio < 4/4 = P2.

### Track G — Sprint pair phase dette (§6.2.1 Regle 1)

1. S28 est un sprint pair → phase dette obligatoire.
2. Verifier que Phase B est etiquetee "Phase dette" dans le plan.
3. Verifier que les items differes SC-9 (platform writers) et SC-10
   (ONNX CI fixture) sont resolus dans Phase B.
4. Verifier les compteurs reports : SC-9 (2/3 → resolu), SC-10
   (5+/3 → resolu, etait en escalade obligatoire).

### Track H — HARDENING_ROADMAP drift (P2 informatif)

1. Comparer HARDENING_ROADMAP §3 S28 (items prescrits originaux) vs
   livrables reels :
   - Nym mixnet : prescrit → scope-cut S30+ avec G9 justification ✅
   - MIG : prescrit → scope-cut post-v1.0 avec G9 justification ✅
   - D2/D3/C4 process isolation : prescrit → design-only (PROCESS_
     ARCHITECTURE.md) avec justification prereqs ✅
   - External audit prep : prescrit → livre (EXTERNAL_AUDIT_SCOPE.md) ✅
2. Chaque deferral doit avoir une justification dans kickoff §7 ou §D5.
3. Drift non justifie = P2.

### Meta-track — Findings phase reviews routes

Phase A review : P2-REVIEW-1, P2-REVIEW-2 → Track A ci-dessus
Phase B review : P2-B-1, P2-B-2 → Track B ci-dessus
Phase C review : P2-C-1, P2-C-2 → Track C ci-dessus
Phase D review : P2-D-1, P2-D-2 → Track D ci-dessus
Total : 8 findings P2 documentes par l'auditeur de phase, tous routes.

---

## 3. Calibration rigor G4

L'audit DOIT trouver au minimum 1 P2+ pour verdict PASS. Sinon
verdict CONCERN et re-audit dimension supplementaire.

---

## 4. Pre-launch protocol check

Verifier :
- `*_VERSION = 1` partout (aucun bump S28)
- Aucun tolerant decoder multi-version introduit
- Aucun test "legacy decode" zombie introduit
- Watermark wiring = interne worker, pas wire format P2P gossip
- Platform writers = audit trail local

---

## 5. Out of scope

NE PAS rebattre :
- D1 watermark bias additif (pas Tournament Sampling) — decision gelee
- D2 libsystemd/oslog crate choice — decision gelee
- D3 JSON-RPC 2.0 pour IPC (sauf si analyse Phase C contredit)
- D4 Trail of Bits recommandation — decision gelee
- D5 Nym/MIG deferrals — decision gelee
- Les scope cuts kickoff §7 (12 items)
- Les pins de dep (iroh 0.97)

---

## 6. Livrable attendu

`sprint28_audit_findings.md` dans `.planning/active/` avec :
1. Verdict global (PASS / CONDITIONAL PASS / FAIL)
2. Une section par Track (A-H + Meta)
3. Findings tries par severite (P0 → P3)
4. Commits fix si CONDITIONAL PASS
5. P2 a logger en tech debt
