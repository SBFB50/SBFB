# Phase Review — Sprint 42 Phase B

## Verdict : PASS

Rigor signal : 2 findings (1 P2, 1 P3) documentes.

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest — aligné. Port 1:1 fonctionnel du Python S14.
- sprint14_keyoxide_decision.md : deploy from source (clone+Keyoxide+SLSA L1) — respecté.
- feedback_context7_systematic.md : aucune nouvelle dep externe.

## Staging check (Step 1bis)
- Phase fichiers : 7 (forge.rs, provenance.rs, deploy.rs, lib.rs, http.rs, main.rs, Cargo.toml)
- Planning split : chore(planning) fait pour preflight.md (`edaf1b3`)
- Untracked accidentels : `tools/babel-scraper/` pre-existant, hors scope

## Suites (Step 2)
- Rust fmt : PASS
- Rust clippy workspace : PASS
- Rust nextest workspace : 1081 tests (1 flaky pre-existant daemon-core quorum)
- Rust doctests : PASS
- Rust release build : PASS
- Python ruff : PASS
- Python SDK : 195 passed
- Python coord : 409 passed, 36 failed (PyO3 wheel stale — pre-existant)
- Frontend lint+tsc+vitest+build+size : PASS

## Delta tests (Step 3)
- Rust workspace : 1060 -> 1081 (+21)
  - forge.rs : +8 (normalize, detect GitHub/GitLab/Codeberg/Gitea/unknown)
  - provenance.rs : +4 (generate+verify, wrong key, tampered, blake3 deterministic)
  - deploy.rs : +9 (SHA validation, zip validation, zip creation+append, dir_size, SBFB.json parse+missing)

## Modified-file branch coverage G9 (Step 2bis)
- forge.rs : tous les bras ForgeType testés (GitHub/GitLab/Codeberg/Gitea/Unknown) PASS
- provenance.rs : generate+verify+tamper+blake3 testés PASS
- deploy.rs : validate_zip, zip_directory, add_to_zip, dir_size, read_sbfb_json, is_valid_sha testés PASS
- http.rs : seules modifs = pub(crate) sur 2 fonctions + 2 routes ajoutées — pas de nouvelle logique PASS

## Research grounding (Step 4bis)
- S1a : F-Droid / SLSA framework / Reproducible Builds — APPROACH-ALIGNED. Port 1:1 du Python validé S14.
- S1b : 0 nouvelle dep externe. zip et tempfile déplacés de dev-deps à deps.

## Scope cuts verification (Step 5)
- 8/8 scope cuts respectés. Deploy handlers portés, pas de modification aux routes restantes.

## Findings

- **P2** : Les handlers deploy utilisent `state.pow_keypair` pour signer la provenance. Ce keypair est le keypair Ed25519 du daemon (identité iroh). En Python, `coord.keypair` était utilisé — c'est le même keypair en pratique (même identité node), mais la correspondance n'est pas vérifiée programmatiquement. Post-v1.0 quand coordinator Python est supprimé (S45), ce sera le seul path et la question disparaît. Carry S43+ : documenter que `pow_keypair` = identité provenance dans PATTERNS.md.
- **P3** : `deploy_private` ne déclenche pas de publish announcement — le caller devra appeler `/publish` séparément. Le Python fait pareil (le private deploy ne publie que si explicitement demandé). Comportement cohérent.

## Recommendation
- Ready to commit : oui
- Carry-overs S43+ : P2 pow_keypair = provenance identity documentation
