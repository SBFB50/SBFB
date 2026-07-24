# Review r3 — Sprint 83 Phase K — Effets, retries et idempotence

## Verdict: PASS-PENDING

Round r3 = état après le round Codex externe r1 (verdict GAP, 5 P1 — tous
fermés) + re-review Workflow `wf_6fde88ab` (3 agents Opus 4.8 1M,
506 022 tokens) des fixes Codex + fermeture des 3 P2 de cette re-review.
Rapports : scratchpad session `rereview_k2/{code_fixes,claims,regression}.md`.

## Les 5 P1 Codex r1 — traitement

- **K-RETRY-01 (crash perd le timer de retry → run bloqué)** : FIX CODE —
  re-arm IDEMPOTENT du timer `er:` à chaque dispatch d'un node en échec
  retryable (`schedule_timer` INSERT OR IGNORE première-gagne, miroir du
  re-propose workflow-timeout) ; un crash entre le failure journalisé et
  son timer retarde le retry d'un backoff, ne le perd plus jamais. Test
  DISCRIMINANT `retry_timer_lost_in_crash_rearmed_on_resume` (état
  post-crash reproduit par la porte store directe) — **mutation ACTIVE
  jouée** : bloc re-arm retiré → test rouge, chemin normal
  (`node_timeout_and_retry_consume_plan_policies`) resté vert, restauration
  → vert (le piège mtime/os.replace a d'abord masqué le rebuild — touch).
- **K-CANCEL-01 (drain cancel n'atteint pas un orphelin)** : FIX CODE —
  branche `Err(UnknownAttempt)` du drain → `reap_orphan(handle journalisé)`
  + close, plus jamais un `cancel()` no-op. Test
  `cancel_drain_reaps_orphan_by_journaled_handle`. La claim « résultat
  conservé » est re-bornée « OBSERVÉ Ready au poll » (course post-poll
  documentée, code + THREAT).
- **K-RECEIPT-01 (digest jamais vérifié en lecture)** : RÉTROGRADATION
  HONNÊTE — row T-RUN-RECEIPT-FORGE-REPLAY reformulée : les 2 propriétés
  VÉRIFIÉES (anti-rejeu par contenu aux portes completion/reconcile ;
  journal digest-only) + borne (b) explicite : le matériel outbox n'est
  PAS re-vérifié contre son digest à la lecture — intégrité = écrivain
  unique + frontière fichier A11 (une vérif lecture serait défaite par le
  même adversaire) ; receipt vérifié+signé = Phase O.
- **K-PROC-01 (« whole tree, crash included » faux sous Unix)** :
  RÉTROGRADATION HONNÊTE — row « fermée Windows / bornée Unix » avec les
  3 bornes explicites (setsid échappe au process-group ; fenêtre
  spawn→persist du handle ; pgid réutilisable) + module-doc process.rs
  ALIGNÉ (les 4 « whole tree » rétrogradés — P2 K2-PROC-DOC-01 fermé).
- **K-FENCE-01 (check-and-append FencedFileActivity non atomique)** :
  RÉTROGRADATION HONNÊTE — la row dit désormais : le MÉCANISME est
  démontré (epoch présentée, application inférieure refusée, red control)
  mais la séquence lire-décider-append n'est pas atomique — non-nominal
  sous single-writer sérialisé ; une cible de PROD doit fournir un
  check-and-apply atomique dans SON backend.

## P2 Codex r1 fermés en code (au-delà du demandé)

- K-CAUSAL-01 : le reducer refuse une réconciliation AVANT le park
  (pré-autorisation `intent→reconcile→park→completion` fermée) — test
  `reconciliation_before_park_refused_by_reducer` ; l'ordre légitime
  vérifié intact.
- K-FS-01 : la feuille n'accepte que NotFound ou fichier RÉGULIER
  (FIFO/socket/device refusés au gate) — test durci
  `fs_effect_non_regular_leaf_refused` avec assertion DISCRIMINANTE
  `retryable:false` (le mutant produirait un échec Io retryable:true).
- K-WIN-01 : retour `ResumeThread` vérifié (≠ (DWORD)-1) ; échec →
  child tué.
- K-PROC-SCOPE-01 : l'env fait partie de `dedup_scope` (une reprise à env
  différent tombe en `TargetScopeChanged`) — test
  `process_env_is_part_of_dedup_scope`.
- K-DOC-01 : doc-comment `NodeFailed` + schema régénéré (l'ancienne
  phrase « never written » supprimée) ; JoinFired doc précisée
  (présence lue par le reducer, cohorte lue par la résolution).

## P2/P3 résiduels ASSUMÉS (documentés, pas de fix)

- K-FS-02 : collapse idempotent inatteignable après crash
  post-write/pré-completion avec un provider d'état RÉEL (le worktree
  digest a changé du fait de NOTRE write → précondition refuse la
  reprise). Résiduel de conception assumé : la précondition est un CAS
  d'état strict ; le déblocage est un geste opérateur (re-plan sur l'état
  courant) — même famille que T-RUN-RESTORE. Documenté ici + body.
- `record_late_completion` sans `require_running` (défensif,
  non-atteignable — jumeau spike-faults public utilisé par les tests J).
- Orphan test : les 2 incarnations partagent le flag idempotent (la prod
  lit le flag JOURNALISÉ ; discriminé par mutation reap_orphan).
- Écart preflight DS-2 : `windows` 0.57 (SOTA local_worker) au lieu de
  `windows-sys` — rationale plus fort, 0 nœud lock neuf confirmé.
- REFERENCE.md corrigé : `outcome_unknown` n'est PAS neuf (le spike E le
  produit) — K ajoute les producteurs timeout/orphan.

## Vérification §7.4 (état r3)

- Windows : fmt 0 ; clippy workspace -D warnings OK ; nextest workspace
  **2351/2351 + 6 skip** (état post-fixes Codex ; relance engine post-r3 :
  **104/104 + 6 skip**) ; doctests OK ; release OK.
- Docker sbfb-ci : fmt OK ; crates workflow **216/216 + 6 skip**
  (état post-fixes Codex ; re-run à l'état r3 en cours au moment de ce
  round, verdict au canonique) ; workspace 2345/2345 canonique + classe
  flake e2e daemon documentée (re-run solo 2/2 PASS).
- Web : bloc complet + operator verts (aucun fichier web/ touché).
- Gates : frontier clean ; census 29 ; goldens E/J byte-identiques ;
  0 nœud lock neuf ; arbre sans résidu d'agent (diff-stat 15 M + 4
  untracked crate + artefacts planning).

## Codex reconciliation (3 rounds)

- **Codex r1 = GAP (5 P1)** → 2 fixes code (K-RETRY-01 self-heal timer,
  K-CANCEL-01 drain reap) + 3 rétrogradations honnêtes (K-RECEIPT-01,
  K-PROC-01 Unix, K-FENCE-01) + 4 P2 fermés en code.
- **Codex r2 = GAP (2 P1)** → K-RECONCILE-RAW-01 (receipt opaque acceptait
  un faux succès) FERMÉ : `ResultRecovered` EXIGE un `ActivityReceipt`
  parsable+lié (plus de bras `if let Some`) ; K-FS-STATE-BIND-01
  (précondition FS non liée) FERMÉ : état attendu folded dans
  `dedup_scope`. + P2 fermés : K-CAUSAL-02 (effect_id lié dans le
  reducer), K-PROC-SCOPE-02 (dedup_scope JCS injectif + failure_retryable),
  K-CANCEL-ERR-02 (drain propage les vraies erreurs), K-CANCEL-TEST-02
  (handle journalisé prouvé). + 5 rétrogradations doc P3.
- **Codex r3 = GAP (1 P1)** → K-FS-ROOT-REL : `worktree_root` RELATIF dans
  `dedup_scope` pouvait désigner un autre worktree après un resume dans un
  autre cwd. FERMÉ : la racine est CANONICALISÉE (absolue) dans
  `dedup_scope` (fallback raw si non-résolue — l'effet échoue de toute
  façon au `confine`).

## RESTE À FAIRE avant commit (non exécuté — contexte session serré)

1. **1 round Codex de confirmation** sur le fix r3 (attendu CLEAN ou
   P2/P3 documentés).
2. **§7.4 dual-platform complet** : Windows workspace (fmt/clippy/nextest/
   doctest/release) + Docker sbfb-ci (les précédents runs étaient verts à
   l'état r2 : Win 2354/2354, Docker crates 219/219 ; re-run requis après
   le fix root canonicalisation r3).
3. **Promotion des canoniques** : ce fichier → `sprint83_phase_k_review.md`
   avec `## Verdict: PASS` ; `sprint83_phase_k_codex_review.md` = copie
   byte-identique du dernier round Codex.
4. **Commit atomique** `feat(workflow-engine): Sprint 83 Phase K` (body
   9 sections, stager code + docs + preflight + rounds + canoniques).
5. **Post-commit** : MAJ `nexus_grid_pivot.md` + `MEMORY.md` ; commit
   séparé `chore(planning)` pour `sprint83_plan_po.md` (vue PO ajoutée
   hors-phase, untracked).
