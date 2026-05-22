# Sprint 68 — Audit Plan (pour Phase 0 S68)

**Ecrit** : 2026-05-21 (Phase E Sprint 67).
**Sprint audite** : Sprint 67 (Factory Foundation).
**Tip attendu** : commit Phase E docs(sprint67).

---

## Tracks a auditer

### Track 1 — Suites verification

Relancer la fail-fast checklist 29/29 du verification.md S67.
Verifier les compteurs : 1384 Rust / 270 Vitest / 6/6 size-limit.
Verifier la coherence delta annonce (verification.md §S2) vs reel
(nextest output).

### Track 2 — Security review

Scanner les 4 phases code (A-D) pour vulnerabilites :
- Phase A : sbfb-manifest validation, feed operations validation
  (hex-64 pubkey, project_id). Verifier que les validations sont
  suffisantes (pas de regex bypass).
- Phase B : FTS5 search — T-SEARCH-INJECTION sanitizer (strip
  HTML, reject NUL bytes, FTS5 syntax escape). Verifier que le
  sanitizer couvre les vecteurs THREAT_MODEL §11.
- Phase C : sbfb-factory secret scanner (regex patterns). Verifier
  que les patterns AKIA/ghp_/gho_/PEM sont bien implementes.
  Path traversal rejection (validate `..` et symlinks).
- Phase D : factory.provenance.json — hash BLAKE3, pas de
  signature Ed25519 (prevu S68+). Verifier que le hash est
  deterministe (meme inputs = meme output).

### Track 3 — Patterns review

Verifier `docs/rust/PATTERNS.md` :
- P52 BlobStore enum pattern documente (Phase D)
- P51 raw-op store+forward reference dans les nouveaux crates
- Coherence entre les patterns documentes et l'implementation
  reelle dans sbfb-manifest, sbfb-factory, search.rs

Verifier `docs/shell/PATTERNS.md` pour coherence.

### Track 4 — Scope cuts compliance

Verifier que les 14 scope cuts listes dans verification.md §S4
sont effectivement respectes :
- Pas de preview ephemere (POST /api/v1/preview/load)
- Pas de page React /factory
- Pas de Proof Cards
- Pas de SearchManifest wire format
- Pas de publish path factory→daemon
- Factory ne depend pas du daemon (grep Cargo.toml)

### Track 5 — Tests delta coherence

Verifier que le delta tests annonce dans chaque commit body
correspond aux tests reellement ajoutes :
- Phase A body dit +11 → verifier 11 nouveaux tests identifies
- Phase B body dit +8 Rust +1 Vitest → verifier
- Phase C body dit +11 → verifier (plan estimait +10)
- Phase D body dit +5 → verifier
- Total cumulatif 1384 Rust / 270 Vitest = entree 1349/269 + 35/1

### Track 6 — Review files quality

Verifier les 4 fichiers review Phase A-D :
- Chaque review a verdict PASS (pas PASS-PENDING residuel)
- Chaque review a une section Codex reconciliation
- Les fichiers codex_review sont bruts (pas reecrits par Claude)
- Les preflight ont des verdicts coherents (EXECUTE ou PLAN-ADAPT)

### Track 7 — Carry-overs + ROADMAP_COMMITMENTS

Verifier la coherence des carries :
- P2-THREAT-MODEL-FEED-SURFACE : etait 2/3 entree S67, traite
  Phase B → devrait etre CLOSED 3/3 MANDATORY
- P2-66-2 BlobStore : etait 1/3, traite Phase D P52 → CLOSED
- P2-66-1 feed republish : etait 1/3, traite Phase D note → CLOSED
- P2-66-3 Phase A body format : etait CLOSED en S66 → confirmer
- P2-C-2 path traversal Windows : NEW 1/3 (Phase C review)
- Carries exemption (P2-A-1, P2-AUDIT-2, P2-G-1, T-NN+2) :
  verifier que le statut est inchange

Verifier ROADMAP_COMMITMENTS pour coherence v4 Arc 2.

### Track 8 — HARDENING review

Verifier `docs/security/HARDENING_ROADMAP.md` :
- S67 n'a pas d'entree specifique (pas de hardening sprint)
- Pas de regression sur les mitigations existantes
- THREAT_MODEL.md §11 (search surface) ajoute Phase B : verifier
  completude T-SEARCH-INJECTION + T-CURATOR-VOUCH + T-SEARCH-DOS

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
- Phase E docs-only = pas de Codex (acceptable)
- Hook lightcheck operationnel tout au long du sprint

---

## Critere verdict

| Verdict | Condition |
|---|---|
| **PASS** | 0 P0, 0 P1, >= 1 P2+ documente |
| **CONDITIONAL PASS** | 0 P0, 0 P1 mais conditions a surveiller S68 |
| **FAIL** | >= 1 P0 ou >= 1 P1 non resoluble dans l'audit |

Rigor signal G4 : PASS exige >= 1 P2+ documente. 0 P0/P1 et 0 P2+ = CONCERN (pas PASS).
