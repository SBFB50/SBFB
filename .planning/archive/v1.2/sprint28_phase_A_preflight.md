# Sprint 28 Phase A — preflight G8

Date : 2026-04-25 | HEAD : `a5cef06` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest technical option, research before code — S27 already researched SynthID-inspired PRF (Nature 2024 Google DeepMind), BIRA-resistant vs Kirchenbauer KGW
- feedback_context7_systematic.md : context7 unavailable (MCP tools not loaded), compensated by cargo registry source inspection for llama-cpp-2 0.1.143 API (LlamaSampler::logit_bias stable)

## Scans (all clean)
- S1a OSS prior art : 2 projets recherches (SynthID/Google DeepMind Nature 2024, Kirchenbauer KGW ICML 2023), APPROACH-ALIGNED — le bias additif PRF est le SOTA post-BIRA (arXiv:2509.23019 sept 2025), deja valide S27 Phase B — clean
- S1b deps : 0 nouvelle dep. watermark.rs utilise hmac+sha2 deja dans workspace. llama-cpp-2 0.1.143 pinne, LlamaSampler::logit_bias API stable. 0 delta — clean
- S2 historiques : 6 fichiers phase scannes, 0 commit DEVIATION/rejected sur ces fichiers. P37 scope-cuts documentes (Ollama wiring → post-API-hook, Tournament Sampling → post-v1.0) alignes avec plan Phase A (llama_cpp only) — clean
- S3 threat model : fast-path verified. Phase ne cree pas de nouveau composant securite ni wire format. Wiring existant watermark.rs → llama_cpp.rs. HARDENING_ROADMAP S28 aligned (watermark wiring) — clean
- S4 wire format : fast-path. output_token_ids existe deja dans ResultPayload avec #[serde(default)]. Aucun *_VERSION bump. Day 0 preserved. Pre-launch protocol preserved — clean

## Telemetrie preflight
- Duree totale : ~4m
- S1a : ~1m / 2 projets OSS consultes / finding : APPROACH-ALIGNED (clean)
- S1b : ~1m / 3 libs scannees (hmac, sha2, llama-cpp-2) / finding : clean
- S2 : ~1m / 6 fichiers, git log + archive grep / finding : clean
- S3 : fast-path / ~30s
- S4 : fast-path / ~30s

## Action
Proceder code phase A.
