# Sprint 52 — Audit plan (Sprint 51 post-mortem)

**Auditeur** : session fraiche (pas la session qui a code S51).
**Tip d'entree** : `54e8af0` (S51 Phase B, dernier feat commit).
**Documents source** : `sprint51_kickoff.md` (D1..D4) +
`sprint51_plan.md` (§Phase A, §Phase B) +
`sprint51_verification.md` (23/23 fail-fast).

---

## Mode d'emploi

Lire dans l'ordre : (1) ce fichier, (2) sprint51_plan.md,
(3) sprint51_kickoff.md §D1..D4. Ne PAS lire le code source
avant d'avoir parcouru les tracks ci-dessous et forme une
opinion. Timebox : 2-3h. Livrable : `sprint51_audit_findings.md`.

## Track A — Legacy deletion completeness (Phase A)

- [ ] A-1 : verifier que `git ls-files nexus/ tests/*.py worker/
  pyproject.toml uv.lock` retourne 0 fichier.
- [ ] A-2 : verifier que `git ls-files .github/workflows/
  build-wheels.yml` retourne 0.
- [ ] A-3 : verifier que `scripts/ci-smoke/` contient exactement
  4 scripts (attestation-schema, pkarr-relay-healthcheck,
  reproducible-build, supply-chain-green).
- [ ] A-4 : verifier que supply-chain-green.sh n'a plus de
  section pip-audit.

## Track B — CI workflows coherence (Phase A)

- [ ] B-1 : verifier que ci.yml ne reference plus Python/pytest/
  maturin/ruff/uv.
- [ ] B-2 : verifier que release.yml ne reference plus
  nexus-core-py/maturin/wheel/PyPI.
- [ ] B-3 : verifier que build-pkarr-image.yml reference
  `scripts/ci-smoke/` (pas `tests/ci-smoke/`).
- [ ] B-4 : verifier que release-attest.sh ne reference plus
  nexus-core-py.

## Track C — Carries resolution (Phase B)

- [ ] C-1 : P2-REVIEW-A-1-S48 canary reload size cap — verifier
  que MAX_DURESS_ACK_MESSAGE_LEN et MAX_HEADLINE_LEN existent
  dans le code Rust avec tests.
- [ ] C-2 : P2-REVIEW-B-1-S48 auth.rs set_var — verifier que
  tous les set_var dans auth.rs sont dans #[cfg(test)].
- [ ] C-3 : P2-AUDIT-A-1-S48 doc accuracy — verifier que
  `_reload_policy_locked` n'apparait dans aucun fichier .rs/.ts.

## Track D — Documentation coherence (Phase C)

- [ ] D-1 : CLAUDE.md — nexus/ supprime de la structure, Python
  supprime du Stack, carries S52 mis a jour.
- [ ] D-2 : HARDENING_ROADMAP.md — last_validated mis a jour S51.
- [ ] D-3 : SPRINT_LOG.md — row S51 presente.

## Track E — Process / meta

- [ ] E-1 : G8 preflights 2/2 presents (A + B tous EXECUTE).
- [ ] E-2 : scope cuts 8/8 respectes (diff --stat).
- [ ] E-3 : Phase reviews 2/2 presents (A + B).
- [ ] E-4 : Delta tests cumule coherent : 0 delta (soustractif).
- [ ] E-5 : Sprint impair — pas de phase dette obligatoire (confirme).
- [ ] E-6 : clippy fix print_stub deplace (items_after_test_module).

## Track F — .gitignore hygiene

- [ ] F-1 : verifier que .gitignore contient `packages/` et
  `tests/` (remnants non-traces).
- [ ] F-2 : verifier que .gitignore ne reference plus
  `crates/nexus-core-py/target/`.
- [ ] F-3 : verifier que `git status --short` est clean apres
  le commit wrap-up.

---

## Carries S52

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-A-1-S50 dispatch join order | 2/3 | S50 review |
| P2-REVIEW-B-1-S51 unsafe set_var futur | 1/3 | NEW S51 |
| P2-REVIEW-A-2-S51 docs legacy orphelines | 1/3 | NEW S51 |

**Note S52 pair** : phase dette obligatoire (§6.2.1 Regle 1).
P2-REVIEW-A-1-S50 dispatch join order a 2/3 — si non adresse S52,
il passe a 3/3 MANDATORY S53.

---

## Verdict global attendu

- PASS : 0 P0, 0 P1 → S52 Phase A demarre direct
- CONDITIONAL PASS : 1-3 P1 → fix(sprint51): ... avant S52 Phase A
- FAIL : >= 1 P0 ou >= 3 P1 → re-conception partielle

## Out of scope pour l'audit

- D1..D4 gelees du kickoff (ne pas rebattre)
- Pin iroh 0.98 (Day 0 #3)
- Scope cuts S51 (decision sprint, pas audit)
- 21 docs legacy (carry P2-REVIEW-A-2-S51, pas audit)
