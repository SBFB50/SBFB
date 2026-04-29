# Sprint 38 — Audit plan (Sprint 37 post-mortem)

**Auditeur** : session fraiche (pas la session qui a code S37).
**Tip d'entree** : `c53f663` (S37 Phase B, dernier feat commit).
**Documents source** : `sprint37_kickoff.md` (D1..D5) +
`sprint37_plan.md` (§Phase A, §Phase B) +
`sprint37_verification.md` (32/32 fail-fast).

## Track A — Securite / crypto

- [ ] A-1 : hash-chain BLAKE3+JCS — verifier que `compute_entry_hash`
  utilise `canonical_bytes` avec le bon domain (`DOMAIN_KUDOS_V1`)
  et que le `HashableKudosEntry` exclut `entry_hash` (circularite)
- [ ] A-2 : verify_chain — verifier que la verification est complete
  (prev_hash + entry_hash recalcule) et qu'un tamper est detecte
- [ ] A-3 : unwrap_or_default remplaces — verifier que les 2 handlers
  http.rs retournent 500+log au lieu de silently default

## Track B — Architecture / code quality

- [ ] B-1 : log convergence — verifier que daemon et launcher ecrivent
  dans le meme `<root>/logs/` et pas chacun son path
- [ ] B-2 : validate_result retourne TaskRecord — verifier que le
  handler n'appelle plus `db.get_task()` en double
- [ ] B-3 : rowid tiebreaker (P2-REVIEW-B-1) — verifier que les
  queries SQL utilisent rowid comme tiebreaker et que l'invariant
  est documente

## Track C — Tests / coverage

- [ ] C-1 : delta tests cumule 936→946 (+10) — verifier chaque test
  ajoute teste une branche reelle pas un stub
- [ ] C-2 : mutex poisoned tests — verifier qu'ils empoisonnent le
  mutex correctement (panic dans thread avec guard tenu)
- [ ] C-3 : verify_chain_tampered — verifier que le tamper est un
  vrai UPDATE SQL pas un mock

## Track D — Process / meta

- [ ] D-1 : G8 preflights Phase A + B — verifier coherence preflight
  vs code livre (pas de drift plan→code non documente)
- [ ] D-2 : scope cuts — verifier que les 12 scope cuts §7 du kickoff
  ne sont pas violes
- [ ] D-3 : 2 MANDATORY 3/3 — verifier que P2-B-1-S34 (log convergence)
  et P2-C-1-S34 (.icns macOS) sont reellement fermes

## Track E — Dependencies

- [ ] E-1 : blake3 dep directe coordinator-rs — verifier qu'elle etait
  deja workspace dep et que l'ajout ne tire pas de nouvelles transitives
- [ ] E-2 : icns 0.3 + image 0.25 dans tools/png-to-icns — verifier
  que ces deps sont build-only (pas dans le runtime daemon/launcher)

## Track F — Doc coherence

- [ ] F-1 : HARDENING_ROADMAP compteurs — verifier 946 Rust / ~1949 total
- [ ] F-2 : CLAUDE.md etat actuel — verifier coherence avec verification.md
- [ ] F-3 : Phase review files present : 2/2 (A + B)
- [ ] F-4 : Phase preflight files present : 2/2 (A + B)

## Carries S38

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 6+/3 | exemption blocker externe |
| P2-REVIEW-C-1-S35 validator_loop tokio | 3/3 **MANDATORY** | D5 scope cut |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P3-grammar executor | 3/3+ | defer Rust pipeline |
| P3-watermark executor | 3/3+ | defer Rust pipeline |
| P2-REVIEW-A-1-S37 launcher logging test | 1/3 | Phase A review |
| P2-REVIEW-B-1-S37 rowid documentation | 1/3 | Phase B review |
