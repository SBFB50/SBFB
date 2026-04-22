# Sprint 25 Phase D — preflight G8

Date : 2026-04-22 | HEAD : `a06a2d1` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, OSS prior art obligatoire, research before code — appliqué via S1a ci-dessous
- feedback_context7_systematic.md : context7 Typer + Semgrep queried — APIs stables, pattern existant codebase confirmé

## Scans (all clean)
- S1a OSS prior art : 3 axes recherchés (capability gate-off-by-default, TOML integrity hash, feature toggle patterns), APPROACH-ALIGNED — le plan suit le pattern microsoft/sudo (admin-only activation, OFF by default). Feature flag systems OSS (Togglz, FF4J, Flagsmith) couvrent le toggle web mais pas le gate OS-level admin privilege. Pas de lib prête qui combine TOML integrity + admin check + FastAPI decorator + Semgrep rule — glue code custom approprié pour le cas d'usage security-critical daemon. APPROACH-ALIGNED — clean
- S1b deps : Typer (déjà utilisé — quarantine.py/canary.py/invite.py), tomllib (Python 3.13 stdlib), hashlib/ctypes/os (stdlib). 0 nouvelle dep externe. Semgrep (outil CI externe, pas dep Python). context7 Typer : API `app.add_typer()` + `@app.command()` stable. context7 Semgrep : `pattern-either` + `pattern-not` + metavariables confirmés pour custom rules Python/FastAPI. 0 delta — clean
- S2 historiques : 0 DEVIATION/rejected/scope-cut sur fichiers Phase D cibles. CAPABILITY_TOGGLES.md écrit S22 hors-sprint (status design-only), implementation S25 explicitement planifiée HARDENING_ROADMAP §3. Aucune décision contradictoire — clean
- S3 threat model : fast-path verified. D5 adresse AD2 (malware user-mode) per CAPABILITY_TOGGLES.md §1. HARDENING_ROADMAP S25 confirme item. Aucune régression threat — clean
- S4 wire format : fast-path verified. Phase D ne touche ni canonical.rs ni schemas/ ni `*_VERSION`. `capabilities.toml` = nouveau fichier config (pas wire format). Day 0 D3 préservée. Pre-launch protocol non impacté — clean

## Telemetrie preflight
- Duree totale : ~3m
- S1a : ~2m / 3 axes OSS consultés (microsoft/sudo, feature-flag ecosystem, TOML integrity) / finding : APPROACH-ALIGNED clean
- S1b : ~1m / 3 libs scannées (Typer context7, Semgrep context7, tomllib stdlib) / finding : clean
- S2 : ~30s / git log + archive grep / finding : clean
- S3 : fast-path / ~15s
- S4 : fast-path / ~15s

## Action
Proceder code phase D.
