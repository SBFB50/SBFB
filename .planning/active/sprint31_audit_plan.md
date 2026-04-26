# Sprint 30 — Audit plan (pour S31 Phase 0)

**Date** : 2026-04-26
**Tip sortie S30** : sera le commit Phase E (post-migration)
**Auditeur** : session fraiche S31 Phase 0 (pas la meme session)

---

## 1. Mode d'emploi

L'auditeur est une session Claude Code fraiche. Ordre de lecture :

1. Ce fichier (`sprint31_audit_plan.md`)
2. `CLAUDE.md` (racine) — projet + pointeur workflow
3. `docs/claude/README.md` §3 + §8 (audit gate + comment auditer)
4. NE PAS lire les phase reviews (`sprint30_phase_{A..D}_review.md`)
   ni `sprint30_verification.md` AVANT d'avoir forme une opinion
   independante sur chaque Track
5. NE PAS rebattre les D1..D5 gelees du `sprint30_kickoff.md` §4

Timebox suggeree : 2-3h.
Delivrable : `.planning/active/sprint30_audit_findings.md` avec
verdict PASS / CONDITIONAL PASS / FAIL.

---

## 2. Dimensions d'audit

### Track A — P2 batch S29 audit (Phase A `a731811`)

1. **CONSENT-1 pure function** : verifier que `_populate_threat_fields`
   dans `consent.py` est desormais une pure function qui retourne un
   dict sans muter l'input. Verifier que les callers (GET/POST consent
   endpoints) mergent correctement le resultat.
2. **CONSENT-2 test coverage** : verifier que
   `test_consent_populate_pure_function` exerce le cas nominal et que
   les tests existants (regression) passent sans modification.
3. **OTEL-1 doc fix** : grep `"0.27"` dans HARDENING_ROADMAP.md et
   `"0.28"` dans nexus-trace-core/src/lib.rs — doit etre 0 matches.
4. **EXECUTOR-1 trace path** : verifier que `crates/nexus-executor/
   src/main.rs` a un commentaire inline expliquant le choix du chemin
   relatif (intentionnel) ou que le chemin est resolu depuis
   `ShellDaemonPaths`.
5. **TASK-RUNNER-1 defense-in-depth** : verifier que
   `crates/nexus-executor/src/task_runner.rs` a un commentaire
   defense-in-depth confirmant que le stub ne peut pas executer de
   code arbitraire.
6. **THREAT-MODEL-1 §9.5 gap note** : verifier que THREAT_MODEL.md
   §9.5 mentionne "output filter designed S23, wire e2e S31 target"
   ou equivalent.

### Track B — blob-serve COOP/COEP (Phase B `a63562e`)

1. **COOP-1 header present** : verifier que `blob_serve.rs` ou le
   middleware HTTP ajoute `Cross-Origin-Opener-Policy: same-origin`
   sur les reponses blob-serve.
2. **COEP-1 header present** : verifier que
   `Cross-Origin-Embedder-Policy: require-corp` est present.
3. **NOSNIFF-1 pre-existing** : verifier que
   `X-Content-Type-Options: nosniff` est toujours present (S13).
4. **SANDBOX-1 iframe** : verifier que `sandbox="allow-scripts"` sans
   `allow-same-origin` est toujours en place dans le code HTML iframe.
5. **CSP-1 unchanged** : verifier que CSP `connect-src 'none'` est
   toujours present (pas supprime par les ajouts COOP/COEP).
6. **CI-1 rust-ci.yml coverage** : verifier que `rust-ci.yml` inclut
   nexus-events-core dans sa matrice multi-OS ET que la matrice couvre
   au minimum Linux + macOS. Confirmer que `ci-cross-platform.yml`
   n'a pas ete cree (redondance documentee Phase B review P3).

**P2 carry Phase B review** :
- P2-REVIEW-B-1-S30 : Playwright COEP iframe test absent. Verifier
  si les tests existants exercent quand meme le chargement d'une app
  iframe.

### Track C — Warrant canary FROST DKG (Phase C `387b6b9`)

1. **DKG-1 generate_with_dealer** : verifier que `dkg.rs` appelle
   `frost_ed25519::keys::generate_with_dealer()` avec les params K/N.
   Verifier la validation (K > 0, N > 0, K <= N).
2. **DKG-2 serialization** : verifier que les `KeyPackage` et
   `PublicKeyPackage` sont serialises/deserialises JSON roundtrip
   (test `dkg_generate_serialize_roundtrip`).
3. **CEREMONY-1 full roundtrip** : verifier que
   `ceremony_full_roundtrip_3_participants` exerce round1 → round2 →
   aggregate avec K=2 signers sur N=3.
4. **CEREMONY-2 insufficient signers** : verifier que 1 signer sur
   K=2 produit une erreur explicite.
5. **CEREMONY-3 tamper detect** : verifier que la verification echoue
   quand le message est altere apres signature.
6. **COMPAT-1 Niveau 0/1** : verifier que la signature FROST aggregee
   est verifiable par un verificateur Ed25519 standard (`ed25519-dalek`
   `verify_strict` ou equiv). C'est le test critique pour la compat
   wire format.
7. **CONFIG-1 canary.toml.sample** : verifier que le fichier est
   parsable TOML et documente K, N, share paths.
8. **RUNBOOK-1 ops** : verifier que WARRANT_CANARY_HARDENING.md §4
   contient des commandes reelles (endpoints HTTP ou CLI) au lieu de
   pseudo-commandes `sbfb canary frost ...`.

**P2 carry Phase C review** :
- P2-REVIEW-C-1-S30 : HTTP integration tests FROST endpoints absents.
  Verifier si les endpoints sont au moins declares dans le routeur HTTP.

### Track D — G2 HARDENING + split inference (Phase D `9c8ffc9`)

1. **G2-1 last_validated** : grep `last_validated` dans
   HARDENING_ROADMAP.md — doit mentionner 2026-04-26 et S30.
2. **G2-2 triggers documentes** : verifier que les 3 triggers ACTIFS
   (iroh 0.98.0, arti-client 2.0.0, openai-agents 0.14.6) sont
   documentes dans `audited_findings` avec dates et sources.
3. **G2-3 S31 entry** : verifier que HARDENING_ROADMAP §3 contient
   une section S31 mentionnant "Tor transport phase 1" avec
   arti-client 2.0.0.
4. **G2-4 S30 statut reel** : verifier que §3 S30 documente le statut
   reel des 4 items prescrits (Nym re-defer, TEE scope-cut, warrant
   canary Phase C, split inference Phase D).
5. **SPLIT-1 structure** : verifier que SPLIT_INFERENCE_DESIGN.md a
   les 5 sections annoncees (contexte, patterns existants, threat model,
   recommendations, references).
6. **SPLIT-2 grounding** : verifier que les patterns cites (BOINC,
   Truebit, Golem, split learning) sont documentes avec des references
   verifiables (pas de hallucination).
7. **BLUEPRINT-1 coherence** : verifier que VALIDATED_BLUEPRINT.md
   n'est pas en conflit avec les 3 triggers (probable no-op confirme
   Phase D — triggers = deps, pas menaces).

**P2 carry Phase D review** :
- P2-REVIEW-D-1-S30 : VALIDATED_BLUEPRINT Couche 6 stale
  (Kirchenbauer→SynthID, spaCy→GLiNER). Verifier si les references
  sont correctes dans l'etat actuel.

### Track E — Process compliance

1. **G8-1 preflights** : verifier que les 4 fichiers preflight
   (phase_A/B/C/D) existent avec verdict EXECUTE.
2. **G8-2 reviews** : verifier que les 4 reviews phase (A/B/C/D)
   existent avec verdict PASS et au moins 1 P2 chacun (rigor signal).
3. **COMMIT-1 format** : verifier que les 4 commits feat/docs suivent
   le pattern `feat|docs(sprint30): Sprint 30 Phase X — titre` avec
   body riche.
4. **G1-1 design review** : verifier que
   `sprint30_design_review.md` existe et contient le scoring D1..D5.
5. **SCOPE-1 cuts respected** : verifier que les 13 scope cuts du
   kickoff §7 n'ont pas ete violes (aucun fichier dans le diff qui
   touche un scope-cut).
6. **PRE-LAUNCH-1** : verifier que `*_VERSION = 1` partout dans
   `crates/nexus-core-rs/src/`. Pas de bump. Pas de tolerant decoder.
   Pas de test "legacy decode" introduit.
7. **ROADMAP-1** : verifier que commit `c50976a` (roadmap v1.0
   Alexandria) est coherent avec le kickoff §7 scope cuts et ne
   promet pas des livrables pour des sprints ou ils ne sont pas prevus.

### Track F — Meta-track carries

1. **CARRY-1 compteurs** : verifier que P2-REVIEW-B-2 et
   P2-REVIEW-C-1 sont documentes a 2/3 reports dans le carry summary.
   Si non resolus S31, ils deviennent MANDATORY S32.
2. **CARRY-2 LT-6** : verifier que LT-6 (iroh > 0.97) est note
   "trigger met" mais bloque par Day 0 #3.
3. **CARRY-3 new carries** : verifier que les 4 nouveaux carries
   (Playwright COEP, VALIDATED_BLUEPRINT stale, confidence_score,
   HTTP FROST tests) sont documentes avec source commit.

---

## 3. Track G1 presence

Verifier que `sprint30_design_review.md` existe dans archive/v1.2/.
Absent = **P1**. Present avec scoring 5/5 = OK. Present sans scoring
= P2.

---

## 4. Track HARDENING drift

Comparer HARDENING_ROADMAP §3 ligne S30 (items prescrits) vs ce que
S30 a reellement livre :

| Item prescrit | Livre ? | Justification si non |
|---|---|---|
| Nym mixnet phase 1 | Non | Scope-cut kickoff §7.2 (SDK paused crates.io) |
| TEE H100 attestation | Non | Scope-cut kickoff §7.3 (pas hardware partenaire) |
| Split inference research | Oui | Phase D `9c8ffc9` SPLIT_INFERENCE_DESIGN.md |
| Warrant canary Niveau 1 | Oui | Phase C `387b6b9` DKG + ceremony code wiring |

Drift sur 2 items non-livres : les deux ont scope-cut justifie dans le
kickoff. Pas de drift non-justifie.

---

## 5. Verdict global attendu

- **PASS** : 0 P0, 0 P1 → S31 Phase A demarre direct
- **CONDITIONAL PASS** : 1-3 P1 fixables → S31 Phase A bloque
  tant que les `fix(sprint30): ...` ne sont pas landed
- **FAIL** : >= 1 P0 ou >= 3 P1 → re-conception partielle

---

## 6. Out of scope pour l'audit

NE PAS rebattre :
- D1..D5 gelees (trusted dealer DKG, GitHub Actions multi-OS,
  COOP/COEP headers, G2 refresh, split inference design doc)
- Scope cuts (Tor S31, Nym S32+, TEE scope-cut, etc.)
- Choix de pin iroh 0.97 (Day 0 #3)
- Pre-launch protocol policy (pas de bump VERSION)

---

## 7. Livrable final attendu

Format : `sprint30_audit_findings.md` dans `.planning/active/`.
Sections : Auditeur, Verdict global, Track A..F findings avec
severity P0/P1/P2/P3 par finding, carry-overs pour S31 avec
compteur reports.
