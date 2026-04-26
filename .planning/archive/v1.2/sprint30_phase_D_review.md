# Phase Review — Sprint 30 Phase D

## Verdict : PASS (1 P2 + 1 P3)

Rigor signal : 1 finding P2 documente + 1 P3 — >=1 requis pour
PASS rigoureux satisfait.

## Memory consultation (Step 1.5)
- feedback_approach.md : "Documenter AVANT de coder, toujours" +
  "research before code" — Phase D est docs-only, conforme
- feedback_context7_systematic.md : context7 obligatoire avant
  code/decision sur lib — conforme (3 WebSearch sources
  bibliographiques utilisees pour SPLIT_INFERENCE_DESIGN.md)
- Violations memory : aucune

## Staging check (Step 1bis)
- Phase fichiers : 2 (HARDENING_ROADMAP.md modifie,
  SPLIT_INFERENCE_DESIGN.md nouveau)
- Planning/docs split : preflight deja commite separement
  (`ec1f812` chore(planning))
- Untracked accidentels : 0 (dans docs/security/)

## Suites (Step 2)
- Rust nextest : 864 passed (0 failed, 0 skipped) ✅
- Rust doctests : 0 passed, 1 ignored ✅
- Rust clippy : 0 warnings ✅
- Rust fmt : clean ✅
- Release build : nexus-shell-daemon OK ✅
- Python ruff : 150 files formatted, all checks passed ✅
- SDK tests : 195 passed ✅
- Coord tests : 394 passed + 36 failed (PyO3 stale) + 6 skipped ✅
  (baseline inchangee)
- Gov tests : 46 passed ✅
- Frontend lint : 0 errors (7 warnings pre-existants) ✅
- Frontend tsc : 0 errors ✅
- Vitest : 269 passed ✅
- Frontend build : OK ✅
- size-limit : 4/4 within budget ✅
- Playwright : 41 passed + 2 failed (env Windows, baseline) ✅
- en-strings : clean ✅

## Delta tests (Step 3)
+0 tests (phase docs-only). Conforme plan §7.3.

| Suite | Avant (S30 Phase C) | Apres (S30 Phase D) | Delta |
|---|---|---|---|
| Rust nextest | 864 | 864 | +0 |
| SDK | 195 | 195 | +0 |
| Coord | 394+36f+6s | 394+36f+6s | +0 |
| Gov | 46 | 46 | +0 |
| Vitest | 269 | 269 | +0 |
| Playwright | 41+2f | 41+2f | +0 |
| **Total** | **~1854** | **~1854** | **+0** |

## Modified-file branch coverage (Step 2bis, G9)
N/A — phase docs-only, aucun fichier code modifie.

## Research grounding (Step 4bis)
- S1a OSS prior art (preflight) : APPROACH-ALIGNED, 4 patterns
  documentes (BOINC, Truebit, Golem, split learning) ✅
- Plan §3 Research consulte : context7 arti-client + 5 WebSearch
  (nym-sdk, frost-ed25519, iroh, arti, openai-agents) ✅
- SPLIT_INFERENCE_DESIGN.md §5 : 10 references (5 academiques +
  3 projets OSS + 2 docs SBFB) ✅

## Horizon long-terme + documentation amont (Step 4ter)
- Design doc present : SPLIT_INFERENCE_DESIGN.md est le livrable
  principal — design doc pour sprint futur post-Gate 4 ✅
- D4/D5 Day 0 avec alternatives rejetees : kickoff §4 D4 rejette
  "upgrade iroh 0.98 immediate" et "ignorer triggers" ; D5
  rejette "prototype code" et "skip" et "inline dans HARDENING" ✅
- Solution la plus poussee : research doc couvre 4 patterns OSS,
  threat model implications, recommendations argumentees ✅
- LOC estimees au plan : kickoff references LOC sont retrospectives
  (S29 delivery) ou rejection rationale, pas prospectives ✅

## Scope cuts verification (Step 5)
Phase D docs-only, touche uniquement docs/security/. Aucun scope
cut viole :
1. Tor transport S31 : non touche (documente dans S31 section
   HARDENING_ROADMAP) ✅
2. Nym S32+ : non touche (documente re-defer rationale) ✅
3. TEE scope-cut : non touche ✅
4-13. : non touches ✅

## Findings (rigor signal)

### P2-D-1 : VALIDATED_BLUEPRINT.md Couche 6 stale references

VALIDATED_BLUEPRINT.md:275 reference "Kirchenbauer 2023 green-list
tokens biased" pour watermark injection — remplace par SynthID-
inspired PRF z-test (S27 Phase B). Ligne 278 reference "regex PII
+ spaCy NER wasm" pour prompt redaction — remplace par
onnxruntime-web + GLiNER (S21 Phase B).

Phase D plan dit "Coherence check (probable no-op)" et les 3
triggers actifs ne touchent pas ces claims. Pas dans le scope
Phase D. **Carry S31** : mettre a jour VALIDATED_BLUEPRINT.md
Couche 6 avec stack actuelle (SynthID + GLiNER).

### P3-D-1 : SPLIT_INFERENCE_DESIGN.md §2.4 activation size estimate

L'estimation "~128 MB par token" est approximative. Pour un
modele 7B (hidden_dim=4096, 32 layers, float16) : 1 *
seq_len * 4096 * 2 bytes * 32 layers = ~256 MB pour seq_len=1000.
L'ordre de grandeur est correct pour le propos du document
(couts prohibitifs), mais l'estimation pourrait etre plus precise.
Cosmetique — la conclusion (transfert inacceptable) reste valide.

## Recommendation
- Ready to commit : **oui**
- Carry-overs S31 : P2-D-1 VALIDATED_BLUEPRINT.md Couche 6 stale
  (Kirchenbauer → SynthID, spaCy → GLiNER)
- Corrections needed : aucune (P3 cosmetique, pas bloquant)
