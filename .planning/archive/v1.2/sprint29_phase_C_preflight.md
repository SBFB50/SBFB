# Sprint 29 Phase C — preflight G8

Date : 2026-04-26 | HEAD : `1f79c52` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, research before code, OSS prior art obligatoire — aligned (PROCESS_ARCHITECTURE.md design doc exists, D1 OSS research documented)
- feedback_context7_systematic.md : context7 obligatoire avant code — done (tokio UDS/NP API confirmed 1.49.0, JSON-RPC 2.0 spec stable)
- Tensions plan vs memory : aucune

## Scans (all clean)
- S1a OSS prior art : 6 projets recherches (BOINC, Golem/Yagna, Ollama, servo/ipc-channel, BUS/RT, iceoryx2), APPROACH-ALIGNED — broker/executor split with raw JSON-RPC 2.0 over UDS/NP matches established pattern of mature distributed compute systems. Raw serde_json approach (vs jsonrpsee) matches Delta Chat JSON-RPC bindings pattern. No LIB-EXISTS finding.
- S1b deps : 5 libs scannees (serde_json, tokio 1.40 full, clap, hmac 0.12, sha2 0.10), all already in workspace, 0 CVE on RustSec, 0 version delta — clean
- S2 historiques : 4 fichiers cibles, 8 commits scannes (S27 multi-forge, S25 key rotation, S18 canary, S7 daemon, S29-A events-core). Aucune decision historique ne contredit le split broker/executor. Archive scan : S18 DEVIATION canary auto-publisher = threat-model unrelated to process isolation. Memory feedback : no contradiction — clean
- S3 threat model : **full scan** (escalation : Phase C introduit nouveau composant securite process isolation + task_token HMAC-SHA256). PROCESS_ARCHITECTURE.md §7 couvre : (a) T0-T5 mapping complet (T0 inchange, T1 gain exploit executor != identity compromise, T2-T3 marginal, T4 VM S30+, T5 VM+TEE), (b) nouveaux vecteurs IPC channel (UDS fs perms / NP DACL S16), task_token replay (per-task + timestamp), executor crash (backoff + SecurityEvent), (c) aucun nouveau T necessaire. HARDENING_ROADMAP S29 D2 aligne. THREAT_MODEL §9 per-mode (Phase B) couvre. 0 regression — clean
- S4 wire format : fast-path verified. Phase C fichiers ne touchent pas canonical.rs ni schemas/. IPC formats (task.execute, health.report, executor.shutdown) = internes broker-executor, PAS wire P2P gossip. VERSION=1 preserves, Day 0 D1-D5 preservees, pre-launch policy respectee — clean

## Telemetrie preflight
- Duree totale : ~3m
- S1a : ~1m30s / 6 projets OSS consultes / finding : APPROACH-ALIGNED (clean)
- S1b : ~30s / 5 libs scannees / finding : clean (0 CVE, 0 delta)
- S2 : ~30s / 8 commits scannes / finding : clean
- S3 : full / ~30s / T0-T5 mapped, 0 regression
- S4 : fast-path / ~15s / VERSION=1, Day 0 preserved

## Action
Proceder code phase C.
