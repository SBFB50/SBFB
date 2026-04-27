# Sprint 32 Phase C — preflight G8

Date : 2026-04-27 | HEAD : `1a60033` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid, research before code — phase C = batch fixes targeted, approach aligned
- feedback_context7_systematic.md : context7 obligatoire avant code touchant lib/API — context7 indisponible cette session, WebSearch Ollama API confirme `num_predict` = correct param name

## Scans (all clean)
- S1a OSS prior art : Ollama native API confirme `num_predict` dans `options` (WebSearch + ollama_rs 0.2.6 source `GenerationOptions::num_predict(i32)`). APPROACH-ALIGNED — standard builder pattern, pas de recode from scratch — clean
- S1b deps : 0 nouvelle dep. ollama-rs 0.2.6 inchange. Pas de CVE. `num_predict` type `i32` vs `max_tokens: u32` = cast safe (valeurs realistes << i32::MAX) — clean
- S2 historiques : 5 fichiers cibles, 5 commits scannes (S31 Phase A task_runner, S31 Phase B coordinator, S30 Phase B http.rs COEP, S30 Phase D HARDENING). Archive scan : S18 canary DEVIATION non-related. Aucune decision historique ne contredit le wiring max_tokens, les FROST error tests, le Tor log fix, ou le HARDENING counter fix — clean
- S3 threat model : fast-path verified. Phase C n'introduit aucun nouveau composant securite ni wire format. FROST tests = test-only additions sur endpoints existants loopback-only. max_tokens = IPC interne. HARDENING = doc. Tor log = clarity fix. Pas de regression T0-T5 — clean
- S4 wire format : fast-path verified. Aucun fichier canonical.rs/schemas/ dans le perimetre. *_VERSION = 1 partout (schemas/mod.rs comment). Day 0 D1-D5 S32 non contredites. Pre-launch policy preserved — clean

## Telemetrie preflight
- Duree totale : ~4m
- S1a : ~2m / 1 WebSearch Ollama API + ollama_rs 0.2.6 source read / finding : APPROACH-ALIGNED
- S1b : ~30s / 1 lib (ollama-rs 0.2.6) / finding : clean
- S2 : ~30s / 5 fichiers + archive scan / finding : clean
- S3 : fast-path / ~20s
- S4 : fast-path / ~20s

## Action
Proceder code phase C.
