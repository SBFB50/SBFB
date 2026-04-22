# Sprint 25 Phase A — preflight G8

Date : 2026-04-22 | HEAD : `a6985b1` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : "pick deepest, no band-aid" — Phase A resolve root cause P2 items, pas de contournement
- feedback_context7_systematic.md : N/A (refactor code existant, pas de nouvelle lib/API)

## Scans (all clean)
- S1a OSS prior art : P2 cleanup batch (DNS concurrent + quarantine alerting). tokio::select! = pattern standard concurrent DNS. structlog alerting = pattern standard. APPROACH-ALIGNED — clean
- S1b deps : 0 nouvelles deps (hickory-resolver, tokio, structlog tous existants workspace). 0 CVE delta — clean
- S2 historiques : `git log --all --grep=... -- dns_fallback.rs quarantine_queue.py` = 0 decision historique traversee — clean
- S3 threat model : fast-path verified, HARDENING_ROADMAP §3 S25 aligned (DNS fallback = transport hardening, quarantine = compute defense) — clean
- S4 wire format : fast-path, Phase A ne touche pas canonical.rs ni *_VERSION. DNS = transport, quarantine = Python-only — clean

## Telemetrie preflight
- Duree totale : ~2m
- S1a : 30s / 2 patterns verifies (tokio::select!, structlog) / finding : clean
- S1b : 15s / 3 deps verifiees / finding : clean
- S2 : 20s / 2 fichiers, 0 commits matches / finding : clean
- S3 : fast-path / 15s
- S4 : fast-path / 10s

## Action
Proceder code phase A.
