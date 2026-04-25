# Phase Review — Sprint 27 Phase A

## Verdict : PASS

(Rigor signal : 2 findings P2 documentes — >=1 requis pour PASS rigoureux)

## Memory consultation (Step 1.5)
- feedback_approach.md : no band-aid, pick deepest — Phase A P2 batch = cleanup, aligne. Respecte.
- Aucune zone specifique applicable (pas kudos/governance/deploy/lib).
- Tensions plan vs memory : aucune.

## Staging check (Step 1bis)
- Phase fichiers : 10 M + 1 new (preflight)
- Planning/docs split : chore(planning) fait commit `5586d0b` avant Phase A
- Untracked accidentels : 0

## Suites (Step 2)
- Rust fmt : clean
- Rust clippy : 0 warning workspace
- Rust nextest : 803 pass (+1 vs 802 baseline)
- Rust doctest : pass (1 ignored, pre-existant)
- Rust release build : nexus-shell-daemon OK
- Python ruff : format clean, lint clean
- Python SDK : 195 pass (+2 vs 193 baseline)
- Python coord : 419 pass + 14 fail + 6 skip (14 fail = test_files pre-existant env + 3 test_apps entry-point, pas regression)
- Python gov : 46 pass (apres env fix)
- Frontend lint : 0 errors (7 warnings pre-existants)
- Frontend tsc : clean
- Frontend vitest : 264 pass
- Frontend build : OK
- Frontend size-limit : OK
- Frontend Playwright : 41 pass + 2 fail (entry-point env, pas regression)
- scan-en-strings : clean

## Modified-file branch coverage (Step 2bis, G9)
- `lib.rs` : `with_max_bytes()` constructeur → tested by `json_file_writer_rotation` PASS
- `lib.rs` : `rotate()` fn privee → tested by `json_file_writer_rotation` PASS
- `lib.rs` : `if self.path.exists() / if meta.len() >= self.max_bytes` branches → tested by `json_file_writer_rotation` PASS
- `dispatcher.py` : `validate_stage_guard_map(stage_guards)` 1 LOC wiring → tested by `test_dispatcher_rejects_invalid_stage_guard_key` PASS
- `capability_store.py` : `_log.debug(...)` remplace `pass` → defensive logging, comportement identique (CONCERN acceptable)
- `decorators.py` : `"description": fn.__doc__ or ""` 1 LOC → tested by `test_task_handler_captures_docstring` PASS
- `registry.py` : `"description": meta.get("description", "")` 1 LOC → tested by `test_task_handler_descriptor_has_description` PASS
- `api/apps.py` : `"description": th.description` 1 LOC → manifest response field, exercised indirectly by existing manifest tests PASS
- `api/app.py` : comment only, no logic change PASS

## Delta tests (Step 3)
- Rust : 802 -> 803 (+1 json_file_writer_rotation, net: +2 new -1 renamed etw→tracing)
- SDK : 193 -> 195 (+2 task_handler_captures_docstring, task_handler_descriptor_has_description)
- Coord : +1 (test_dispatcher_rejects_invalid_stage_guard_key)
- Total delta phase : +4 (plan disait +3 — ecart = le rename etw→tracing produit tracing_writer_compiles qui est un +1 net car l'ancien etw_writer_compiles est supprime)

## Commit body validation (Step 4)
- Format titre : `feat(sprint27): Sprint 27 Phase A — P2 batch S26 audit 7 fixes` PASS
- Contexte present : oui (7 P2 documentes individuellement) PASS
- Delta tests : +4 reel vs +3 annonce body — body dit "+4" avec detail, PASS
- Scope cuts honoured : "aucun scope cut viole" PASS
- Co-Authored-By present : oui PASS

## Research grounding (Step 4bis)
- 4bis-A : S1a = N/A (P2 batch cleanup, pas de design decision). Preflight documente. PASS
- 4bis-B : Aucune dep ajoutee/modifiee. Aucune API externe touchee. PASS

## Horizon long-terme + documentation amont (Step 4ter)
- Pas de nouveau module structurant : N/A
- D1..D5 : Phase A ne cree pas de D — D4 kickoff documente les 7 items PASS
- Solution la plus poussee : N/A (cleanup fixes, pas de choix technique)
- LOC estimates : le kickoff §1.2 + D1 + D2 contiennent des LOC prospectives — P2 process (cf. finding ci-dessous)

## Scope cuts verification (Step 5)
- Tor transport : 0 fichiers diff PASS
- Arti library-embed : 0 PASS
- Domain fronting : 0 PASS
- GPU lockup defense : 0 PASS
- A4 process roles : 0 PASS
- Ollama backend watermark : 0 PASS
- SynthID Tournament Sampling : 0 PASS
- Platform writers complets : 0 PASS
- Streaming bridge C5 : 0 PASS
- Full Gate 3 showcase app : 0 PASS

## Findings

### P2-review-1 — JsonFileWriter rotation TOCTOU window

`lib.rs:119-124` : le check `metadata(&self.path).len() >= self.max_bytes`
suivi de `rotate()` n'est pas atomique. Un writer concurrent pourrait
voir le fichier avant la rotation et ecrire dans le fichier qui va
etre renomme. En pratique, le `SECURITY_EMITTER` singleton (`OnceLock`)
serialise les appels `emit_event` — le TOCTOU est unexploitable dans
l'architecture actuelle. Carry-over : si multi-writer devient un cas
(LT-5 redundancy persistence), ajouter un file lock.

### P2-review-2 — LOC estimates prospectives dans kickoff S27

`sprint27_kickoff.md` §1.2, D1, D2 contiennent des estimations LOC
prospectives (~1500 LOC total, ~500 LOC, ~700 LOC, etc.). Convention
§6.7 interdit les LOC prospectives dans plan/kickoff. Ces estimations
sont dans le kickoff passe (pas dans Phase A code), donc non-bloquant
pour ce commit. Process finding carry-over : supprimer les LOC
estimates dans les prochains kickoffs.

## Recommendation
- Ready to commit : **oui**
- Carry-overs S28 : P2-review-1 (TOCTOU multi-writer) si LT-5 activated
- Corrections needed : aucune
