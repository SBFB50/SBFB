# Sprint 76 — Phase G review (wrap-up + Arc 3.5 close)

> Produit par Workflows ultracode (review fan-out 5 dimensions + adversarial,
> puis re-review focalisee 2 agents apres correction). NE remplace PAS Codex.

## Verdict: PASS

Review Workflow PASS-PENDING (round 2) + Codex GPT-5.5 reconcilie (0 GAP) =
verdict promu **PASS**. Committable.

## Round 1 — review fan-out (5 dimensions) → CONCERN

Le fmt-fix, les docs longue-vie et la posture acceptance LIVE etaient propres et
honnetes (verification.md marque #26/#30 DIFFERE-materiel-operateur, jamais PASS ;
THREAT_MODEL v9 = bloc versions 0 row dupliquee ; PATTERNS §P62/P38 numeros libres ;
SPRINT_LOG hashes valides ; 0 bump wire). **MAIS** une dimension a confirme un **P1
en code neuf** + des P2 :

| Sev | Finding | Fix |
|---|---|---|
| **P1** | `b3_live_pc_vps.sh` palier 2 soumettait `redundancy_factor=2` SANS `verifiable:true` → le worker echantillonne (`task.rs`) et le dispatcher saute le cohort gate (`dispatcher.rs:70` : gate SSI `verifiable && redundancy>1`) → 2 workers divergent → **quorum hash-exact JAMAIS forme** → poll BLOCK 30s. Le banner imputait a tort la non-convergence a un worker non-homogene. | **ROOT-FIX** : `VERIFIABLE=true` ssi `REDUNDANCY>=2`, cable a `"verifiable":%s` du submit. Banner/verdict/header citent `verifiable` comme prerequis. |
| P2 | `die()` appele (validation REDUNDANCY) AVANT sa definition → entree invalide = `die: command not found` exit 127. | `die()` hoiste AVANT le bloc de validation. |
| P2 | CLAUDE.md « +31 Vitest (A+7,B+10,E+1,F+1) » somme a 19 (melange base 367 off-sprint avec breakdown phases). | « +19 depuis l'entree phases 379 ; 367→398 = +31 dont +12 off-sprint ». |
| P2 | `sprint77_audit_plan.md` §3 omet le carry B10-PARITE (route `sprint76_phase_b_review.md:28/:268`). | Ajoute au §3 (DOC-P2, 2 miroirs hand-maintained). |
| P3 | `REDUNDANCY=02` -> JSON invalide `redundancy_factor:02`. | Normalisation base-10 `$((10#$REDUNDANCY))`. |

## Round 2 — re-review focalisee (2 agents) → PASS-PENDING

- **P1 harness root-fixe, confirme contre le code** : `types.rs:101` (`verifiable`
  `#[serde(default)]` false), `dispatcher.rs:70` (cohort gate SSI
  `verifiable && redundancy>1`), `task.rs:234-247` (signed canonical, greedy vs
  sampling), `runtime.rs:1341` (`if task.verifiable { params.deterministic(seed) }`).
  Sans `verifiable=true` le quorum ne peut PAS se former ; avec, les workers
  homogenes convergent. **Palier 1 NON-REGRESSE** : `redundancy_factor:1,verifiable:false`
  = semantiquement identique a l'ancien (champ omis → serde default false). `die()`
  defini avant usage. `bash -n` clean, 0 hazard `set -e`/quoting/JSON.
- **3 P2 doc resolus** : CLAUDE Vitest arithmetique reconciliee (7+10+1+1=19,
  379+19=398, 367→398=+31 dont +12 off-sprint) ; B10-PARITE present
  `sprint77_audit_plan.md:319` fidele a la source ; verification.md row #30 + §5
  mentionnent `verifiable:true` auto.
- **P3 leading-zero blinde** : `$((10#$REDUNDANCY))` ("02"→2, "007"→7) ; cases
  testees ACCEPT 1/2/02/007, REJECT abc/""/0/-1.
- Aucun nouveau P0/P1/P2 introduit.

## Gate dual-platform (AVANT push)

- Windows : fmt 0 (apres fix `http.rs:8531`) + clippy 0 + nextest **1804/1804**
  0-skip + doctests 0 + release 0.
- Docker canonique `sbfb-ci` rust:1.94 : fmt **0** + clippy 0 + nextest
  **1808/1808** 0-skip (+4 cfg(unix)) + doctests 0.
- Web : lint 0-err + tsc 0 + Vitest **398/398** + coverage 87.2/79.01/85.92/88.52
  + size 6/6 + scan clean.
- Les fixes review touchent uniquement `b3_live_pc_vps.sh` (script non-compile,
  jamais en CI) + `.md` → compteurs Rust/Vitest inchanges (re-verif non requise) ;
  `nexus-shell-daemon` re-teste 398/398 apres le fmt-fix whitespace.

## Codex reconciliation

Codex GPT-5.5 (`codex exec`, output brut `sprint76_phase_g_codex_review.md`) :
**3/6 CONFIRMED, 3 PARTIAL, 0 GAP**.
- **CONFIRMED** : D3 (verification.md 0 faux-vert, #26/#30 DIFFERE, bilan 36/38),
  D4 (audit_plan carries complets + 3 two-report fermes), D5 (docs longue-vie
  structure exacte).
- **PARTIAL = limites sandbox Codex, pas des defauts** : D1 (rustfmt wrapping
  confirme ; preuve « violation 1.94 reelle » non re-jouable car Docker
  inaccessible + toolchain 1.94 absente — verifiee independamment cote livreur),
  D6 (arithmetique 1804/398 + Cargo.lock vide confirmes par Codex ; seul Docker
  1808 + fmt 1.94 dependent du self-report car Docker hors-sandbox).
- **D2 PARTIAL (0 GAP) = clarification de scope, traitee** : Codex note que le
  harness ne soumet pas `required_runtime` donc n'exerce pas l'auto-claim-gate
  du dispatcher (couvert par le test Phase C
  `dispatcher_routes_replicas_to_homogeneous_cohort`). Le harness prouve le
  quorum via `verifiable` + homogeneite operateur. Documente honnetement
  (SCOPE NOTE dans `b3_live_pc_vps.sh` + verification.md §5 ; choix delibere
  pour eviter une fragilite tuple-mismatch en run manuel).
- **`INVITE_FORMAT_VERSION=2`** signale par Codex (pre-existant, inchange S76) :
  corrige la section `## Pre-launch protocol` du commit body (« 0 bump wire EN
  S76 », pas « tous a 1 »).

Aucun GAP P0/P1 ; les 3 PARTIAL sont resolus (sandbox-replay + scope documente).
Suites non re-jouees (les corrections post-Codex sont des commentaires `.sh` +
`.md`, 0 code compile). Verdict final PASS.
