# Sprint 69 — Audit Plan (pour Phase 0 S69)

**Ecrit** : 2026-05-22 (Phase E Sprint 68).
**Sprint audite** : Sprint 68 (Proof Cards + Publish Gate).
**Tip attendu** : commit Phase E docs(sprint68).

---

## Tracks a auditer

### Track 1 — Suites verification

Relancer la fail-fast checklist 30/30 du verification.md S68.
Verifier les compteurs : 1419 Rust / 279 Vitest / 6/6 size-limit.
Verifier la coherence delta annonce (verification.md §S2) vs reel
(nextest output).

### Track 2 — Security review

Scanner les 4 phases code (A-D) pour vulnerabilites :
- Phase A : ProofCard compute — formule score deterministe,
  formula_version field. Verifier que la formule ne peut pas etre
  jouee (THREAT_MODEL §12 T-PROOFCARD-FORMULA-GAME, 4 vecteurs).
  Verifier que les risk factors sont correctement calcules.
  Endpoint HTTP proof-card : verifier absence d'injection dans
  project_id path parameter.
- Phase B : Preview ephemere — POST /api/v1/preview/load accepte
  des bytes arbitraires (zip). Verifier la taille max (10 MB),
  l'eviction TTL, l'isolation blob-serve. Publish path : verifier
  que le pre-validation sbfb-manifest empeche la publication
  d'un manifest invalide.
- Phase C : Gates FG4-FG7 — FG5 sandbox utilise dunce::canonicalize
  + prefix check. Verifier que le path traversal est correctement
  bloque (Windows backslash, symlinks, .. sequences). FG6 secrets :
  verifier que les patterns regex couvrent les cas communs (AKIA,
  ghp_, gho_, PEM). Diff engine : verifier pas d'injection via
  noms de fichiers.
- Phase D : ProofCard UI — composant React. Verifier que les
  donnees ProofCard sont echappees correctement dans le rendu
  (pas de XSS via risk_factors ou layer names). Trust-wording
  compliance : aucun terme interdit.

### Track 3 — Patterns review

Verifier `docs/rust/PATTERNS.md` :
- Pas de nouveau pattern documente S68 (confirmer)
- P52 BlobStore et P51 raw-op toujours coherents avec l'impl
- sbfb-factory gates patterns (FG4-FG7) documentes ou implicites

Verifier `docs/shell/PATTERNS.md` pour coherence.

### Track 4 — Scope cuts compliance

Verifier que les 14 scope cuts listes dans verification.md §S4
sont effectivement respectes :
- Pas de SearchManifest wire format (grep SearchManifest dans code)
- Pas de page React /factory (grep factory dans web/src/pages/)
- Pas de Babel app (grep babel dans crates/ ou examples/)
- Pas de @dev index tree-sitter (grep tree-sitter dans Cargo.toml)
- Pas de FG8/FG9/FG10 (grep FG8 FG9 FG10 dans code, seulement
  dans docs)
- Factory ne depend pas du daemon (grep Cargo.toml)

### Track 5 — Tests delta coherence

Verifier que le delta tests annonce dans chaque commit body
correspond aux tests reellement ajoutes :
- Phase A body dit +11 Rust +1 Vitest → verifier
  (note : body Phase A disait initialement +10, corrige
  retrospectivement Phase B → P2-I-2 carry)
- Phase B body dit +14 Rust → verifier
- Phase C body dit +10 Rust → verifier
- Phase D body dit +0 Rust +8 Vitest → verifier
- Total cumulatif 1419 Rust / 279 Vitest = entree 1384/270 + 35/9

### Track 6 — Review files quality

Verifier les 4 fichiers review Phase A-D :
- Chaque review a verdict PASS (pas PASS-PENDING residuel)
- Chaque review a une section Codex reconciliation
- Les fichiers codex_review sont bruts (pas reecrits par Claude)
- Les preflight ont des verdicts coherents (EXECUTE ou PLAN-ADAPT)

### Track 7 — Carry-overs + ROADMAP_COMMITMENTS

Verifier la coherence des carries :
- P2-C-2 path traversal Windows : etait 1/3, traite Phase C
  dunce::canonicalize → devrait etre CLOSED/RESOLVED
- P2-I-2 delta body : etait 2/3, vient de S68 Phase A retro-
  correction → verifier 3/3 atteint ou pas
- Carries exemption (P2-A-1, P2-AUDIT-2, P2-G-1, T-NN+2) :
  verifier que le statut est inchange
- LT-2 Radicle : tag v1.0 pose, trigger toujours PENDING ?
- LT-5, LT-7 : statut inchange attendu

Verifier les 3 carries Phase D :
- P2-D-1 : BrowsedProject proofCardQuery wiring non teste en
  integration (composant teste unitairement). Verifier si teste S69.
- P2-D-2 : ProofCardData type sans validation Zod runtime
  (cast direct `as`, daemon trusted local). Verifier si corrige S69.
- P2-D-3 : THREAT_MODEL XSS mention implicite (React auto-escape
  mitigue, pas documente explicitement). Verifier si documente S69.

Verifier ROADMAP_COMMITMENTS pour coherence v4 Arc 2 :
- S68 devait livrer Proof Cards + publish path → confirmer livre
- S69 attend : Babel dogfood + pilote ferme + Gate 1

### Track 8 — HARDENING review

Verifier `docs/security/HARDENING_ROADMAP.md` :
- S68 pas d'entree specifique hardening
- Pas de regression sur les mitigations existantes
- THREAT_MODEL.md §12 (ProofCard surface) ajoute Phase D : verifier
  completude T-PROOFCARD-FORMULA-GAME (4 vecteurs + mitigations)
- Preview store (Phase B) : verifier qu'aucun threat manque pour
  le vecteur ephemeral preview abuse (DoS via uploads massifs)

### Track 9 — Meta-process

Verifier la discipline de process du sprint :
- 5 phases A-E avec G8 preflight systematique (5/5)
- Commit discipline : chaque phase a un commit atomique avec body
  riche (9 sections obligatoires)
- Pas de band-aid fix
- Pas d'amend
- Pas de force push
- Memory mise a jour post-phase
- Plan sequentiel suivi (superviseur actif)
- Codex gate §4.5 respecte (4 phases code A-D)
- Hook lightcheck operationnel tout au long du sprint
- Agent Teams supervisor deploye et gates G-SPAWN..G-POST respectes

---

## Critere verdict

| Verdict | Condition |
|---|---|
| **PASS** | 0 P0, 0 P1, >= 1 P2+ documente |
| **CONDITIONAL PASS** | 0 P0, 0 P1 mais conditions a surveiller S69 |
| **FAIL** | >= 1 P0 ou >= 1 P1 non resoluble dans l'audit |

Rigor signal G4 : PASS exige >= 1 P2+ documente. 0 P0/P1 et 0 P2+ = CONCERN (pas PASS).
