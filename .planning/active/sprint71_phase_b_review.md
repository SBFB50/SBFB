# Sprint 71 Phase B — Review Deep (B-2 deterministic quorum + D8 cleanup + G13)

Date: 2026-05-30
HEAD: `2f9238d` (diff dans le working tree NON commité)
Reviewer: review-deep (Claude)
Preflight referencé: `sprint71_phase_b_preflight.md` (verdict PLAN-ADAPT)

## Verdict: PASS

> Review Claude OK puis **reconcilie avec Codex GPT 5.5** (10/10
> livrables CONFIRMES, 0 GAP, 0 PARTIEL — voir §Codex reconciliation) :
> code correct, branch coverage complete, scope cuts honores,
> wire-format conforme pre-launch, deviation TaskSubmission justifiee
> et bornee. Aucun P0/P1. 2 P3 documentables ci-dessous. Committable.
>
> (Etat transitoire PASS-PENDING avant Codex conserve dans l'historique :
> la sequence review PASS-PENDING -> Codex -> reconciliation -> PASS a
> ete respectee.)

Evidence : chaque affirmation cite `fichier:ligne` ou une sortie de
commande inline. Suites re-jouees independamment (pas confiance
aveugle au main thread) — voir §Branch coverage.

---

## Correctness (ligne par ligne)

Tous les hunks de `git diff HEAD` lus. Le code fait ce que le plan
adapté (preflight §Plan Adaptation) annonce, sans band-aid.

- **`task.rs:145-167`** — champ `verifiable: bool` `#[serde(default)]`,
  doc-comment explicite (signed, contraste `redundancy_factor`,
  rationale runtime-tolerance). Initialisé `false` dans `new()`
  (`task.rs:247`). Builder `with_verifiable` (`task.rs:264-270`).
  **NON retiré de `task_canonical_bytes`** (vérifié : `task.rs:42-44`
  ne contient que `obj.remove("redundancy_factor")`, pas
  `"verifiable"`) → champ bien dans les bytes signés. Conforme S4.
- **`llm/mod.rs:188-199`** — `seed: Option<u32>` ajouté, doc précise
  "inert sur llama_cpp / load-bearing sur Ollama" + avertit
  explicitement "NOT the per-task watermark PRF seed". Init `None`
  (`mod.rs:226`). Builders `with_seed` (`mod.rs:247-251`) et
  `deterministic` (`mod.rs:259-263`) — ce dernier pose
  `temperature=Some(0.0)` + `seed=Some(seed)`, **ne touche PAS** les
  champs `watermark_*`. Correct.
- **`ollama.rs:180-189`** — `deterministic_options(&params)` câblé dans
  `req_build`, n'attache `.options()` que si `Some`. Helper
  `ollama.rs:238-252` : retourne `None` ssi `temperature` ET `seed`
  absents (best-effort préservé), sinon construit
  `GenerationOptions::default().temperature(t).seed(s as i32)`. Le
  commentaire `as i32` (`ollama.rs:247-249`) documente la
  reinterpretation bit-for-bit. Correct.
- **`runtime.rs:1043-1047`** — l'inline `GenerateParams::new(...)...`
  remplacé par `build_generate_params(&task_entry.task, &self.worker_config.watermark)`.
  Helper `runtime.rs:1244-1258` : `.with_watermark(...)` PUIS, si
  `task.verifiable`, `.deterministic(deterministic_seed(&task.task_id))`.
  L'ordre préserve la config watermark (appliquée avant, non clobberée).
  `deterministic_seed` (`runtime.rs:1267-1270`) = 4 premiers octets LE
  de `blake3_hash(task_id)` → `u32`. Correct.
- **`validator.rs:84-100`** — doc-comment ajouté sur `validate_quorum`,
  **logique inchangée** (vérifié : le corps `validator.rs:101-175` est
  identique au pré-diff, seuls les `///` au-dessus sont nouveaux). La
  doc explique le misnomer colonne `sha256` (heritage S55) et que
  l'exact-match n'est utile que si workers déterministes. Correct.
- **`redundancy.rs`** — supprimé via `git rm` (status `D`), `pub mod
  redundancy` retiré de `lib.rs:32`. Grep `redundancy` sur le crate :
  AUCUN hit `RedundancyDispatcher` / `redundancy::` résiduel — seuls
  des `redundancy_factor` (champ distinct légitime). Compilation propre.
- **`build_executor.rs:126-140`** — doc "Dormant entry point" ajoutée,
  **pas de `#[deprecated]`** (vérifié : le diff n'ajoute que des `///`).
  Le wrapper `execute_build → execute_build_with_timeout` reste appelable
  sans warning sous `-D warnings`. Correct (un `#[deprecated]` aurait
  cassé le build comme le brief le craignait — évité).
- **`process.rs:24-33`** — doc PROVIDERS (axe prompt-adaptation vs
  backend exécution, "NOT unified"). Const inchangée. Correct.
- **`PATTERNS.md` §P53** + **`ROADMAP_COMMITMENTS.md` LT-7** — cohérents
  avec le code, datés 2026-05-30. Correct.

**Verdict Correctness : clean.** Aucun band-aid. Le fix attaque la
root cause (Ollama ne câblait pas `GenerationOptions`), pas un
contournement.

---

## Branch coverage

Suites re-jouées indépendamment (ciblé, pas re-build workspace) :

```
cargo nextest run -p nexus-core-rs -p nexus-coordinator-rs -E '<new tests>'
  → 9 passed, 561 skipped
cargo nextest run -p nexus-worker-core -E '<new tests>'
  → 3 passed, 187 skipped
```

Couverture des branches challengées :

1. **`verifiable` true ET false** : `build_generate_params` testé sur
   les DEUX chemins (`runtime.rs:1325-1356` : `det` → `temp=Some(0.0)`
   + `seed=Some(...)` ; `plain` → `temp=None` + `seed=None`). ✓
2. **Quorum accept ET reject à factor=2** : `quorum_accepts_deterministic_redundancy`
   (`validator.rs:500-522`, 2 honnêtes identiques → Accepted) ET
   `quorum_rejects_nondeterministic_divergence` (`validator.rs:524-549`,
   2 divergents → QuorumRejected). Math vérifiée : factor=2 →
   `majority_threshold = 2/2 = 1` (`validator.rs:135`) → accept ssi
   `best_count > 1` (`validator.rs:142`), donc 2 d'accord passent, 1+1
   divergents échouent. ✓
3. **Craft path submission→Task** : `submit_propagates_verifiable_flag`
   (`dispatcher.rs:174-194`) — `sub.verifiable=true` → `entry.task.verifiable`
   + `verify_signature()` OK ; submission par défaut → `false`. Couvre
   la déviation TaskSubmission de bout en bout. ✓
4. **Ollama options posées ET absentes** : `deterministic_options_wire_temperature_and_seed`
   (`ollama.rs:397-409`, assert via Serialize : `temperature=0.0`,
   `seed=7`) ET `best_effort_params_attach_no_options`
   (`ollama.rs:411-419`, params nus → `None`). ✓
5. **Seed stabilité** : `verifiable_task_uses_greedy_seed` assert
   `deterministic_seed("task-det") == deterministic_seed("task-det")`
   (idempotent) ET `!= deterministic_seed("task-other")` (discriminant).
   ✓ — répond au point #7 du brief : même `task_id` → même blake3 →
   même `u32`, donc tous les workers d'un task convergent.
6. **Wire roundtrip + default** : `task_wire_verifiable_roundtrip`
   (`task.rs`) ET `task_wire_default_verifiable_false` (JSON minimal
   omettant le champ → `false`). ✓
7. **Canonical/signature discriminance** : `task_canonical_includes_verifiable`
   (bytes diffèrent) ET `task_entry_different_verifiable_different_signature`
   (signatures diffèrent + verify OK). ✓

**Verdict Branch coverage : complète.** Les 5 paires accept/reject —
true/false — present/absent — stable/discriminant — signed/verify sont
toutes verrouillées. Aucun trou de branche identifié.

---

## Sécurité (S3 deep)

- **Greedy/seed n'ouvre pas de surface** : confirmé. L'attaquant qui
  veut passer le quorum doit produire le MÊME `result_text` que les
  honnêtes → reproduire l'argmax greedy exact → bon modèle + bon prompt
  + bon contexte = le travail honnête. Le déterminisme n'aide pas
  l'attaquant (il ne raccourcit aucun calcul). Le seed est PUBLIC
  (dérivé du `task_id`, non secret) — le connaître n'aide pas à forger.
- **seed déterministe ≠ watermark_seed PRF** : la confusion est
  activement prévenue dans le code, pas juste évitée. `mod.rs:194-197`
  ("NOT the per-task watermark PRF seed [`Self::watermark_seed`]"),
  `runtime.rs:1264` ("distinct from the per-task watermark PRF seed"),
  PATTERNS §P53. `deterministic()` ne touche pas `watermark_seed`
  (`mod.rs:259-263`), champs orthogonaux. ✓
- **Rejet outliers préservé** : `validate_quorum` corps inchangé
  (`validator.rs:130-172`) — comptage par `sha256`, majorité stricte
  `best_count > majority_threshold`, log+warn des divergents
  (`validator.rs:145-154`), rejet si pas de majorité
  (`validator.rs:165-172`). `quorum_rejects_nondeterministic_divergence`
  verrouille la propriété. Aucune régression sur un threat couvert
  (C-ResultSpoof S23, collusion residual M Sybil).
- **Collusion** : vecteur inchangé par B-2 (deux malveillants pouvaient
  déjà converger sur un faux texte commun avant B-2). Mitigations
  existantes (Ed25519 par ResultEntry, majorité stricte) intactes.

**Verdict Sécurité : clean.** Le forçage greedy/seed DURCIT le quorum
inference (le rendait utilisable), sans nouvelle surface. Limite
cross-GPU honnêtement documentée (PATTERNS §P53, carry S75).

---

## Scope cuts

Grep des lignes AJOUTÉES du diff pour tokens interdits
(`ProviderRouter|sharding|shard|cross-machine|shared_gpu|logprobs_hash|
tree-sitter`) :

- Unique hit : `PATTERNS.md:974` "the real cross-machine quorum proof is
  scope-cut to **S75**" — c'est une référence qui HONORE le scope cut
  #11, pas une violation.
- Aucune ligne de code ne touche : cross-machine réel (S75),
  logprobs/watermark verification (V2 — les mentions watermark sont les
  clarifications anti-confusion), ProviderRouter (S72), GPU partagé
  (S75), sharding (S76).

**Verdict Scope cuts : tous honorés.** #11 (cross-machine→S75), #13
(logprobs→V2), #1 (ProviderRouter→S72) intacts.

---

## Wire / pre-launch

- **`verifiable` change les canonical bytes** : confirmé voulu
  (`task_canonical_includes_verifiable` assert `bytes_plain != bytes_det`).
  Conforme pre-launch policy (redéfinition libre de la v1 courante,
  aucun nœud tiers en prod).
- **`TASK_FORMAT_VERSION` reste 1** : vérifié `task.rs:61` inchangé. ✓
- **Aucun zombie "legacy decode"** : les 2 tests pré-existants flaggés
  par le preflight ont été ré-examinés :
  - `task_canonical_bytes_contain_the_four_consent_fields`
    (`task.rs:746-763`) utilise `text.contains(...)` sur des champs
    spécifiques, PAS une égalité de forme JSON exacte → ajouter
    `verifiable` ne le casse pas, ce N'EST PAS un zombie.
  - `task_wire_default_factor_1` (`task.rs:802-816`) est un test de
    runtime-tolerance (JSON minimal → défauts), pattern explicitement
    LÉGITIME par la pre-launch policy — pas un zombie figeant "v1 sans
    verifiable". Le nouveau `task_wire_default_verifiable_false` le
    mirroite correctement.
  → **Aucun zombie à supprimer.** Le diff n'introduit aucun décodeur
  multi-version tolérant.
- **`#[serde(default)]`** : justifié runtime-tolerance dans la doc des
  3 sites (`task.rs:166-167`, `types.rs:95-97`, mirroir `is_open_source`).

**Verdict Wire/pre-launch : conforme.**

---

## D8 (honnêteté du nettoyage)

- **RedundancyDispatcher 0 appelant vivant** : confirmé par grep
  (aucun `RedundancyDispatcher` / `redundancy::` résiduel hors le champ
  `redundancy_factor`). Reversion S55 (`0cb576d`, quorum DB-backed
  supersède l'in-memory) → **retrait pur justifié**. Fichier `git rm` +
  `lib.rs:32` mod retiré. Compile propre.
- **execute_build marqueur dormant honnête** : pas de `#[deprecated]`
  (vérifié dans le diff — uniquement `///`). L'appel interne
  `execute_build → execute_build_with_timeout` (`build_executor.rs:140`)
  reste valide sous `-D warnings` (worker-core a compilé clean lors du
  run nextest ci-dessus). Décision "conserve, pas retire" cohérente :
  LT-7/S75 = consommateur futur NOMMÉ (ROADMAP_COMMITMENTS LT-7).
  Contraste explicite avec RedundancyDispatcher documenté
  (`build_executor.rs:135-140` + PATTERNS §P53). **Honnête** : ne
  prétend pas que le code est branché, le marque dormant.

**Verdict D8 : honnête et borné.**

---

## Déviation TaskSubmission

Le brief signale `TaskSubmission.verifiable` + câblage `submit_task` +
3 struct-literals (http.rs, db.rs, dispatch_loop.rs) comme DÉVIATION
hors-scope plan §6.

**Tranche : justifiée et bornée — complétion légitime de B-2, PAS scope
creep.**

- Sans elle, `verifiable` n'est atteignable que par construction directe
  `Task` (jamais via le craft path coordinateur `submit_task`). Or le
  chemin de soumission réel passe par `TaskSubmission` (`types.rs:91-99`)
  → `submit_task` (`dispatcher.rs:88` câble `verifiable: submission.verifiable`).
  La déviation rend B-2 atteignable end-to-end par un client — c'est le
  but déclaré de B-2 ("un caller peut demander du compute déterministe").
- **Bornée** : `types.rs` (+1 champ `#[serde(default)]`),
  `dispatcher.rs` (+1 câblage +1 test), et 3 struct-literals `Task`
  corrigés (http.rs:4077, db.rs:1228, dispatch_loop.rs:85) — ces
  derniers sont des corrections de COMPILATION obligatoires (ajouter un
  champ non-`Default` à une struct construite par littéral force la
  mise à jour de tous les sites). Ce ne sont pas de la fonctionnalité
  nouvelle, juste la conséquence mécanique du champ.
- Couverte par `submit_propagates_verifiable_flag` (true ET false).

**Verdict Déviation : acceptée.** Scope creep aurait été d'ajouter une
route HTTP `/verifiable` ou un ProviderRouter — rien de tel.

---

## Livrables

Plan §6 B.3 tests attendus vs réels :
- `verifiable_task_uses_greedy_seed` ✓ (`runtime.rs:1324`)
- `two_honest_workers_same_hash` ✓ (`validator.rs:466`)
- `quorum_accepts_deterministic_redundancy` ✓ (`validator.rs:500`)
- `quorum_rejects_nondeterministic_divergence` ✓ (`validator.rs:524`)
- Trace G13 CVE ✓ (PATTERNS §P53 "Off-sprint deps validated" : portable-pty
  0.9.0 / async-stream 0.3.6 / futures 0.3.32 + ollama-rs 0.2.6, advisory-clean)

Bonus vs plan (issus du PLAN-ADAPT, attendus) : tests Ollama
`deterministic_options_*` + tests wire `task_*verifiable*` + déviation
`submit_propagates_verifiable_flag`.

**Verdict Livrables : tous présents + bonus PLAN-ADAPT.**

---

## Patterns

- §P53 ajouté (PATTERNS.md) : couvre déterminisme quorum, axes
  provider/backend, dead-module cleanup, G13. Cohérent avec le code
  (vérifié les refs `validator.rs`, `llm/llama_cpp.rs:327`,
  `process.rs`). Cross-refs corrects.
- LT-7 mis à jour (ROADMAP_COMMITMENTS) avec revue S71 Phase B datée.
- Doc-comments code suivent le pattern existant (`is_open_source` /
  `redundancy_factor`) — phrasing runtime-tolerance réutilisé.

**Verdict Patterns : conforme.**

---

## Findings

### P0 / P1 (bloquants)
**Aucun.**

### P2 (documentables)
**Aucun.**

### P3 (cosmétique / suivi)
- **P3-B-1** — `as i32` cast du seed `u32` (`ollama.rs:247-249`) : déjà
  commenté (reinterpretation bit-for-bit, valeur exacte indifférente car
  tous les workers dérivent le même `u32`→`i32`). Pas un bug — la
  reinterpretation est lossless et identique cross-worker. Note seulement
  pour mémoire : si un jour Ollama interprétait le signe du seed
  différemment d'un seed positif, l'invariant "tous identiques" tiendrait
  quand même. Non-actionnable.
- **P3-B-2** — Colonne DB `task_results.sha256` reste un misnomer (stocke
  `result_text` brut pour l'inference, héritage build-task S55).
  Documenté (validator.rs:86-90 + PATTERNS §P53), non renommé
  (wire DB local, édition libre mais cosmétique non prioritaire pré-tag).
  Carry connu, pas introduit par B.

---

## Action

- **PASS-PENDING** : la review Claude valide le code, les branches, la
  sécurité, le wire, le scope, la déviation et les livrables. Aucun
  P0/P1/P2.
- **Gate suivante OBLIGATOIRE avant commit** : `codex exec` (review
  Codex GPT, BLOQUANTE review→commit). Ce verdict n'est PAS committable
  seul.
- Le commit body de Phase B doit citer `sprint71_phase_b_preflight.md`
  et documenter le delta "plan proposait X / S1a a identifié Y / adapté
  à Z" (Ollama gap), comme prévu par le preflight §Plan Adaptation.

---

## Codex reconciliation

- **Outil** : `codex exec` (OpenAI GPT 5.5, cross-model — pas un agent
  Claude). Prompt : `.git/CODEX_SPRINT71_PHASE_B.txt` (10 livrables).
  Sortie BRUTE non réécrite : `sprint71_phase_b_codex_review.md`.
- **Verdict Codex** : **10/10 livrables CONFIRMÉS, 0 GAP, 0 PARTIEL.**
  Codex a relu le working tree (`git diff HEAD`, fichiers sur disque) et
  cité fichier:ligne pour chaque livrable. Il a en plus **exécuté
  lui-même** les tests ciblés (`verifiable_task_uses_greedy_seed`,
  `deterministic_options*`, `best_effort_params_attach_no_options`, les
  3 tests quorum, `submit_propagates_verifiable_flag`) et confirmé qu'ils
  passent.
- **Contrôles transverses Codex** :
  - *Scope cuts* — Codex note "PARTIEL au sens littéral" : des lignes
    ajoutées **documentent** S75/cross-machine (`build_executor.rs`,
    `ROADMAP_COMMITMENTS.md`, `PATTERNS.md`) mais **aucune ne les
    implémente**. Ce sont des deferrals/dormant refs qui HONORENT les
    scope cuts — **pas un GAP** (documenter un scope cut est exactement
    la consigne). Aucun ajout `ProviderRouter`/`S72`/`sharding`/`S76`/
    `watermark V2` dans le diff code.
  - *Legacy decode* — CONFIRMÉ : `TASK_FORMAT_VERSION` reste 1,
    `task_wire_default_verifiable_false` décode un JSON sans `verifiable`
    à `false` (runtime tolerance, pas zombie).
  - *Seed vs watermark* — CONFIRMÉ : seed déterministe distinct du PRF
    `watermark_seed`, séparation explicite dans le code et la doc.
- **GAPs P0/P1** : aucun. **GAPs P2/P3** : aucun (au-delà des 2 P3 déjà
  documentés ci-dessus : cast `as i32` lossless commenté, misnomer
  `sha256` carry S55).
- **Suites** : aucune correction Codex requise → pas de re-run nécessaire
  (les suites workspace étaient déjà vertes : 1498 passed, 0 skipped).
- **Promotion** : verdict review promu PASS-PENDING → **PASS**. Séquence
  review PASS-PENDING → Codex → reconciliation → PASS respectée. Le
  fichier `sprint71_phase_b_codex_review.md` n'a PAS été modifié.
