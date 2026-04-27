# Sprint 31 Phase A — preflight G8

Date : 2026-04-26 | HEAD : `ed2c433` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest option, research OSS before code, context7 systematic
- feedback_context7_systematic.md : context7 obligatoire pour ollama-rs (done S1b)

## Scans (all clean)
- S1a OSS prior art : 1 projet reference (nexus-worker-core OllamaBackend S20), ollama-rs context7 API confirmee, APPROACH-ALIGNED — le plan wire directement ollama-rs dans l'executor (plus leger que dependre du worker-core entier), pattern standard pour un process executor isolé
- S1b deps : ollama-rs workspace `"0.2"` — 0 CVE RustSec, API `GenerationRequest::new()` + `generate().await` stable (context7 61 snippets, score 92.4). schemars liste dans plan mais probablement inutile (executor ne fait pas de schema enforcement, c'est le role worker-core). tokio deja workspace dep — clean
- S2 historiques : 3 fichiers cibles (task_runner.rs, main.rs, Cargo.toml), git log scan — 1 commit pertinent (S29 Phase D TraceProvider, pas de conflit). Archive scan : 0 rejection/deviation sur executor+ollama — clean
- S3 threat model : fast-path verified — pas de nouveau composant securite, pas de wire format. HARDENING_ROADMAP S31 prescrit task_runner implementation, aligne — clean
- S4 wire format : fast-path verified — aucun fichier canonical.rs/schemas touche. TaskExecuteParams/Result = IPC interne, pas wire format reseau. VERSION=1 inchange, Day 0 D1 preservee — clean

## Note mineure (non-bloquante)
Plan §5.2 liste `schemars` dans les deps a ajouter. L'executor ne fait pas de schema enforcement (c'est le role de worker-core via FormatType::StructuredJson). Si l'implementation confirme que schemars n'est pas necessaire, ne pas l'ajouter. Pas un finding — juste une simplification.

## Telemetrie preflight
- Duree totale : ~3m
- S1a : ~1m30 / 1 projet OSS reference + context7 ollama-rs / finding : APPROACH-ALIGNED
- S1b : ~30s / 2 libs scannees (ollama-rs, schemars) / finding : clean
- S2 : ~30s / 3 fichiers, 1 commit scanne / finding : clean
- S3 : fast-path / ~15s
- S4 : fast-path / ~15s

## Action
Proceder code phase A.
