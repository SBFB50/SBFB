# Sprint 29 Phase D — preflight G8

Date : 2026-04-26 | HEAD : `6a23ebf` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, research OSS avant code, context7 obligatoire
- feedback_context7_systematic.md : context7 sur opentelemetry + ed25519-dalek + W3C Trace Context avant code — applique (3 context7 queries + 6 WebSearch)

## Scans (all clean)

- S1a OSS prior art : 4 projets recherches (OpenTelemetry Rust, tracing-opentelemetry, Spine-OSS, dd-trace-rs), APPROACH-ALIGNED — plan TraceProvider/TraceProcessor pattern miroir OTel SdkTracerProvider + SpanExporter. SignedCanaryProcessor valide par Spine-OSS (Ed25519 tamper-evident logging). Clean
- S1b deps : 3 libs scannees (opentelemetry 0.31.0 latest stable sept 2025, opentelemetry_sdk 0.31.0, ed25519-dalek 2.1 RUSTSEC clean), 0 CVE 2025-2026 — clean. Note API : Builder renomme SdkTracerProviderBuilder, BatchSpanProcessor background thread (pas async runtime).
- S2 historiques : 4 fichiers (nexus-trace-core new, runtime.rs, main.rs, Cargo.toml) + archive v1.2/ scannes, 0 decision rejetant tracing infra, HARDENING_ROADMAP confirme A2 TraceProvider S29 — clean
- S3 threat model : FULL SCAN (nouveau composant securite SignedCanaryProcessor). Composant local-only (events pas distribues P2P). Trace signing key Ed25519 en memoire broker process (pas de persistance separee). Aucune regression T0-T5. HARDENING_ROADMAP §3 S29 "A2 TraceProvider OTEL" aligne. THREAT_MODEL §9 per-mode (Phase B) couvre residual risks. Aucun nouveau vecteur reseau. — clean
- S4 wire format : fast-path verified. canonical.rs non touche. 14 DOMAIN_*_V1 existants dans canonical.rs inchanges. Nouveau DOMAIN_TRACE_EVENT_V1 vit dans nexus-trace-core (local, pas canonical.rs). Aucun *_VERSION bump. Day 0 D3 (TraceProvider opentelemetry 0.31) confirmee par implementation. Pre-launch protocol respecte. — clean

## Telemetrie preflight
- Duree totale : ~4m
- S1a : ~2m / 4 projets OSS consultes / finding : clean (APPROACH-ALIGNED)
- S1b : ~1m / 3 libs scannees / finding : clean
- S2 : ~30s / 4 fichiers + archive / finding : clean
- S3 : full / ~30s (composant local-only, pas de surface reseau)
- S4 : fast-path / ~15s

## Action
Proceder code phase D.
