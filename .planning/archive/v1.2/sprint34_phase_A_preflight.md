# Sprint 34 Phase A — preflight G8

Date : 2026-04-27 | HEAD : `ec50151` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : "chercher projets OSS avant coder from scratch" → S1a vérifiable, phase dette pas de code from scratch
- feedback_context7_systematic.md : context7 obligatoire pour zip crate (new dep) et frost-ed25519 3.0 eval → appliqué Phase A implementation

## Scans (all clean)
- S1a OSS prior art : Phase dette (rand unification, COEP E2E test, frost eval) — standard Cargo dependency management + integration testing, APPROACH-ALIGNED — clean
- S1b deps : frost-ed25519 actuellement 2.2.0 (résolu dans range "2.1"), 3.0.0 disponible (trigger G2). zip crate = new dep, version 2.x stable. 0 CVE critical — clean
- S2 historiques : 1 décision trouvée (`04c9621` S18 E2 canary auto-publisher rejeté threat-model) — Phase A ne touche pas le scheduler, compatible. Reversion check: 0 revert trouvé, décision toujours active. frost-ed25519 references dans sprint20/30 = API usage, pas rejet design — clean
- S3 threat model : fast-path verified — Phase A n'introduit ni nouveau composant sécurité ni nouveau wire format. HARDENING_ROADMAP aligned (frost trigger documenté) — clean
- S4 wire format : fast-path — 0 fichier canonical.rs/schemas touché, VERSION=1 préservé, Day 0 preserved — clean

## Telemetrie preflight
- Durée totale : ~3m
- S1a : 30s / phase dette standard, APPROACH-ALIGNED
- S1b : 45s / 2 libs scannées (frost-ed25519, zip) / clean
- S2 : 60s / 3 fichiers phase scope + archive scan / clean
- S3 : fast-path / 15s
- S4 : fast-path / 15s

## Action
Procéder code Phase A.
