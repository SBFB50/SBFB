# Review r1 — Sprint 83 Phase K — Effets, retries et idempotence

## Verdict: CONCERN

Review Workflow `wf_ca6215e9` — 11 agents Opus 4.8 1M (8 dimensions +
3 lentilles adversariales), 2 072 247 tokens, 296 tool uses. Diff vs HEAD
`a23510e8` (arbre sale = Phase K non committée). Rapports par dimension :
scratchpad session `review_k/{orchestrator,store,reducer_core,adapters,
tests,scope,security,frontier,adv_refutation,adv_completeness,adv_design}.md`.

Agrégat : **0 P0**, **2 P1 CONFIRMÉS** (convergents 4+ agents, tenus par la
lentille réfutation), **~6 P2**, **~10 P3**. Scope INTÉGRAL confirmé (dim
scope) : K.2/K.3/K.4 livrés, PA-1..6 + DS-1/DS-2 conformes, 10 carries table
§4 tous traités ou re-routés explicitement, 0 defer silencieux, 0 bump wire,
census 29 intact. Chemin Target byte-identique à J confirmé (0 join dans les
4 goldens, snapshots run_event*.golden.json inchangés).

## P1 — confirmés (corrigés, cf. review canonique)

- **P1-A — Évasion worktree par symlink FEUILLE.** `confine()`
  (`fs_effect.rs`) ne canonicalise que le PARENT ; le composant final est
  rejoint brut et `File::create`/`fs::read` suivent un symlink terminal →
  écriture hors worktree via un lien feuille tracké. La row THREAT
  T-RUN-FS-EFFECT-ESCAPE « symlink sortant refusé AVANT tout octet » + le
  module-doc étaient donc FAUX pour ce cas ; le test ne couvrait que `..`.
  Convergent : adapters/security/scope/frontier + adv-refutation +
  adv-design. `fs_effect.rs:158-162,194-203` ; `THREAT_MODEL.md`.
- **P1-B — Charte « reads NO environment variable » FAUSSE.** `process.rs`
  lisait `std::env::var_os(PATH/HOME/…)` pour l'allowlist, contredisant la
  charte du crate (`lib.rs:104`, re-affirmée `fs_effect.rs:13` +
  `THREAT_MODEL.md`) → contrat d'hôte caché. Convergent : adapters/security
  + adv-refutation + adv-design (DESIGN-CONFLICT). `process.rs:249-253`.

## P2 (corrigés ou documentés)

- **Adaptateurs fs_effect/process non fencing-aware** : la fermeture
  target-side de T-RUN-DOUBLE-EMIT clause 2 (v26) n'est démontrée que par
  la cible echo de référence — borne honnête ajoutée à la row (être
  fencing-aware = propriété DU BACKEND).
- **input JCS exposé via env `SBFB_EFFECT_INPUT`** (lisible /proc/pid/environ
  co-user) — résidu ajouté à T-RUN-PROCESS-SPAWN-INJECTION.
- **llms.txt** gardait « restore/gc/lease-force dated J/K » alors que K les
  re-route Phase V (deux docs rank-1 contradictoires) — corrigé.
- **Rationales « mutation killed » inexactes** (`double_command_single_
  transition`, `non_idempotent_without_receipt_becomes_unknown`) : tests
  valides mais doc-comments décrivant le mauvais mécanisme — réécrits.

## P3 (corrigés ou actés)

- Mode non-park de `unknowable_outcome` laissait la row `attempts` ouverte
  (asymétrie avec le park) — clos via `close_open_attempt` /
  `AttemptOutcome::Unknown`.
- `Facts.join_fired` stockait un `Vec<String>` jamais lu par le reducer +
  doc sur-affirmé — simplifié en `BTreeMap<String,()>`, doc corrigé,
  `_runtime` → `runtime`.
- `reconcile_unknown(ResultRecovered)` acceptait n'importe quelle string —
  validation de binding `ActivityReceipt` (run_id/node/effect_id/hash) +
  `ReconcileReceiptMismatch`.
- Fuite d'un child `CREATE_SUSPENDED` si `job.assign`/`resume` échoue —
  child tué avant propagation.
- 2 gardes causales du reducer (attempt-sans-intent, park-sans-intent) non
  testées — 2 tests ajoutés.
- `reap_orphan` / résolution d'orphelin (jambe crash-recovery K.4) sans
  test — test `orphan_attempt_reaped_and_decided_by_journaled_claim` ajouté
  (branches idempotent/non-idempotent).
- Cohorte de join figée testée seulement pour `Any` — test quorum(2)
  multi-source ajouté.
- Happy-path `ProcessActivity` (exit 0 → Ready) non testé — test ajouté.
- **QUESTION orchestrator tranchée FAIL-CLOSED** : une défaillance terminale
  sous `route-error-port` SANS arête `error` wirée (classifier, ou leaf
  laissé non-wiré) était avalée (run complété silencieusement) — 3b ferme
  désormais `node_failed` quand il n'y a nulle part où router
  (`has_error_edge`), test `route_error_port_without_edge_fails_closed`.
- `record_late_completion` sans `require_running` (durcissement défensif,
  non atteignable — seul appelant est le drain de cancel, run encore
  Running) : ACTÉ P3, jumeau public `spike-faults` utilisé par les tests J.
- Écart doc DS-2 (`windows-sys 0.61`) vs code (`windows 0.57`, aligné SOTA
  `local_worker.rs`, 0 nœud lock neuf) : rationale plus fort, acté au body.
