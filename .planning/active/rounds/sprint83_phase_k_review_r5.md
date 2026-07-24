# Review r5 — Sprint 83 Phase K — Effets, retries et idempotence

## Verdict: PASS-PENDING

Round r5 = review Workflow POST-CORRECTIF sur l'arbre r7→r8 (exigée par
Codex r8, P1 processuel K-ROUND-REVIEW-AFTER-FIX-08 — le canon §4.5.8
veut la review AVANT le round Codex suivant ; le r8 avait été lancé sur
l'arbre corrigé sans ce round durable). Workflow wf_e402653d, 2 agents
Opus 4.8 1M (delta-r8 + final-proofs), verdicts : **delta-r8 = AUCUN
FINDING** ; **final-proofs = CLEAN technique (0 P0/P1/P2), 1 P3
précision-doc FERMÉ** dans la foulée. Succède à la review r4
(PASS-PENDING, round HISTORIQUE aux compteurs de son époque — jamais
réécrit). PASS-PENDING = review OK, Codex r9 de scellement attendu.

## Le delta r7→r8 vérifié INDÉPENDAMMENT (agent delta-r8)

- Gate `#[cfg(windows)]` suffixe `.exe` (process.rs) vérifié contre la
  SOURCE std 1.95 `resolve_exe` : l'ensemble accepté = {absolu} ∩
  {`.exe`-suffixé sous Windows} n'est JAMAIS ré-résolu par std
  (`has_exe_suffix` → chemin utilisé tel quel, ni sibling ni PATH).
  Ordre des gates sans trou (drive-relative refusé par le gate 1 ;
  UNC/verbatim `.exe` acceptés tels quels).
- Claims docs EXACTES, pas surclamées : « tries the `path.exe` SIBLING
  before `path` », « bifurcation pilotable par le sibling » = le
  comportement réel de std (sibling-if-exists puis fallback).
- Test `process_absolute_without_exe_refused_on_windows` NON-vacueux et
  discriminant par le flag retryable : gate présent → `Refused`
  non-retryable `vec![false]` ; gate retiré → std tente `tool.exe` puis
  `tool`, absents → `Io` retryable `vec![true]` → ROUGE. Les voisins
  `current_exe()` verts pour la BONNE raison (un binaire de test finit
  réellement en `.exe`).
- Unix : gate et test `#[cfg(windows)]` non compilés — comportement
  strictement inchangé.
- Bord sous-la-barre noté (pas un finding) : le fichier littéralement
  nommé `.exe` est refusé (`extension()` = None) là où std l'accepterait
  — divergence dans le sens SÛR.
- Suite exécutée par l'agent : engine+core **227 run + 7 skip** confirmé.

## État final des preuves (agent final-proofs)

- **Staging** : 15 M + 4 crate untracked + preflight + rounds codex
  r1..r8 + review r1..r5 — aucune anomalie, `sprint83_plan_po.md`
  toujours EXCLU (chore séparé), canoniques = étape de promotion.
- **Gates** : check-frontier-contracts EXIT=0 (census 29 frozen) ;
  goldens `git diff --stat HEAD -- '*.golden.json'` VIDE — hash
  `run_event_terminal` `1ba2be10…` re-confirmé INDÉPENDAMMENT ;
  `grep -rn "dated J/K" docs/` = 0.
- **Compteurs périmés** : uniquement dans les rounds HISTORIQUES gelés
  (review r4 : 226+7/14, « r7 en cours » — correct au titre de son
  époque) ; rien hors rounds/ ne pollue le canonique.
- **[P3] réconciliation mutations FERMÉ** : les slugs « parse-laxiste »
  et « contains_key » (jouées boucle r2, documentées review r3) n'avaient
  pas d'annotation `Mutation killed:` sur leurs tests — annotations
  posées (`reconcile_recovered_rejects_foreign_receipt`,
  `reconciliation_effect_id_mismatch_refused_by_reducer`), doc-only,
  fmt + 227/227 re-verts. Le compte « 15 » = les mutations ACTIVES de la
  boucle review/Codex r1→r8 ; le fichier effects.rs en documente ~25 au
  total (sur-ensemble, plancher couvert).

## Vérification §7.4 (arbre r8)

- Windows : fmt 0 ; clippy workspace -D warnings OK ; nextest
  2363/2364 + 7 skip — unique rouge = dispatch_loop convergence (classe
  STANDING carry 7, solo PASS 1.8s) ; doctests OK ; release OK.
- Docker sbfb-ci : fmt OK ; nextest 2367/2369 + 7 skip — 2 rouges =
  e2e sigint + worker start_headless (classes STANDING carry 7, solo
  Docker 2/2 PASS) ; le run r7 quasi-identique côté Linux (delta r8 =
  cfg(windows)-inerte) avait donné 2369/2369 0-ROUGE.
- Engine+core : **227 run + 7 skip** ; **15 mutations actives tuées**
  dans la boucle (annotées dans les tests) ; census 29 ; 0 nœud lock
  neuf (992 avant/après) ; web 548 + operator 201 verts (blocs
  complets, inchangés depuis — 0 fichier front touché).

## Codex reconciliation (8 rounds joués, r9 attendu)

r1 GAP 5 P1 → fixes/rétrogradations. r2 GAP 2 P1 → receipt lié +
state-binding. r3 GAP 1 P1 → racine canonicalisée. r4 GAP 1 P1+2 P2+1 P3
→ cwd figé+JCS+freeze. r5 GAP 2 P1+1 P2+1 P3 → program+fenced
log+claims+docs V. r6 GAP 1 P1+1 P2 → ABSOLU-only+feuille nue. r7 GAP
1 P1 → suffixe `.exe` Windows (fait std source). r8 = **CLEAN technique
(0 P0, 0 P1 technique, 0 P2, 0 P3)** — les attaques
casse/point-final/espace/ADS/8.3/résiduel-std toutes CONFIRMÉES fermées,
régressions r1-r6 nulles, claims alignées ; unique P1 PROCESSUEL =
la présente review manquait → FERMÉ par ce round. r9 = scellement
attendu sur l'arbre inchangé (verdict au canonique).

## RESTE À FAIRE avant commit

1. Codex r9 (scellement — l'arbre ne bouge plus, hormis les 2
   annotations doc-only de ce round).
2. Promotion des canoniques : ce fichier → `sprint83_phase_k_review.md`
   avec `## Verdict: PASS` + réconciliation r9 ;
   `sprint83_phase_k_codex_review.md` = copie byte-identique du r9.
3. Commit atomique `feat(workflow-engine): Sprint 83 Phase K` (body 9
   sections prêt).
4. Post-commit : chore(planning) `sprint83_plan_po.md` + MAJ memory.
