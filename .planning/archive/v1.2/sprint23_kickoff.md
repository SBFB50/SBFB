# Sprint 23 — Kickoff (Ephemeral workers + escalating PoW + honeypot + redundancy voting + contribution families foundation)

**Écrit** : 2026-04-20 (session fraîche post-audit gate S22 `6a514fc`).
**Type** : **sprint implementation** (anti-infiltration defense-in-depth
worker-side + redundancy voting foundation Gate 3 + design docs LT-3
contribution families).
**Tip master d'entrée** : `2438c59` (chore(claude): process bankruptcy).
**Phase 0 audit Sprint 22** : **DÉJÀ JOUÉ** — findings dans
`.planning/archive/v1.2/sprint22_audit_findings.md` (verdict **PASS**,
0 P0 + 0 P1 + 8 P2 + 4 P3 documentés carry S23). Migré vers
`archive/v1.2/` dans ce commit d'ouverture S23.

---

## 1. Constat d'entrée

- **Tip** : `2438c59` — 1 commit process chore au-dessus du gate S22
  `6a514fc`.
- **Compteurs tests** : 710 Rust / 185 SDK / 263+3 coord / 46 gov /
  264 Vitest / 38 Playwright / 7/7 size / 246+ SPDX (~1509 total).
- **Working tree** : propre (`git status --short` vide).
- **Version cible** : v1.2 (continuation hardening, pas de nouvelle
  version — le thème anti-infiltration est dans la continuité
  directe de Sybil-resistance S22).
- **G2 trigger check** : `last_validated: 2026-04-20` dans
  HARDENING_ROADMAP frontmatter. Trigger pertinent S23 =
  `openai-agents-python > 0.7.0` (B1 guardrails) → **sans objet**
  car B1 différé S24 (arbitrage Option B 2026-04-20). Aucun trigger
  actif bloquant S23.
- **G6 memory carry-over** : `sprint22_verification.md §5` déjà
  fusionné dans `nexus_grid_pivot.md` tip `6a514fc` par S22 Phase F.
  Memory à jour.

---

## 2. Goal

Durcir la résilience worker-side contre worker infiltré (honey-worker,
extraction modèle, Sybil joining massif) en livrant 4 primitives
anti-infiltration + redundancy voting foundation pour Gate 3. Poser
les design docs contribution families (LT-3 post-v1.0) + endpoint
observabilité fairness.

**Critère SMART : 28+ rows fail-fast verts au `verification.md`,
mesure binaire au Phase F wrap-up.**

---

## 3. Phase 0 — Audit gate Sprint 22

**Verdict** : PASS (0 P0 + 0 P1).
**Commit stack gate** :
```
6a514fc chore(sprint22): audit gate S22 — findings (verdict PASS, no blocking fix) + Phase F review
```
**P2 carry S23** absorbés Phase A ci-dessous (cleanup batch).
**LT-2 Radicle** : sorti cap G7 (trigger tag v1.0 only). Aucune
action S23.
**LT-3/LT-4** : hors-sprint, design-only S23.

---

## 4. Décisions Day 0 (D1..D5 gelées)

### D1 — Ephemeral worker lifecycle : restart-based + VRAM wipe inter-task

**Retenu** : le worker process se restart après `N` tasks
(configurable, default 50) via auto-exit + systemd/superviseur
restart. Entre chaque tâche, `cudaMemset` efface la VRAM visible
(mitigation extraction poids modèle par task N+1 injectée).

**Rejeté** :
- **Process-pool recycling** (nouveau processus par task, pool de K
  prêts) : latence cold-start Ollama ~3-8s par spawn inacceptable
  pour task throughput. Pattern Google Borg (preemptible VMs) ne
  s'applique pas — nos workers sont single-GPU single-process.
- **TEE-only isolation** (NVIDIA CCM H100) : hardware non-disponible
  chez contributeurs bénévoles RTX consumer. Roadmap S30+ via
  attestation TEE provider (trait existant S20 Phase E.5). Carry
  long-terme, pas substitut.
- **Memory encryption at rest only** (sans VRAM wipe) : couvre le
  disque (livré S20 keystore) mais pas la surface GPU la plus
  exploitable (CUDA memory visible inter-process sans isolation
  MIG).

**Implications** :
- Nouveau module `crates/nexus-worker-core/src/ephemeral.rs`
- Config `worker.toml` champ `max_tasks_before_restart: u32`
- Intégration au state machine `WorkerState` existant
  (`engine/runtime.rs`)
- Dep : `cudarc` 0.12 pour `cudaMemset` safe binding (ou raw FFI
  si cudarc trop lourd)

### D2 — PoW escalation : ramp géométrique per-(consumer, model)

**Retenu** : difficulté PoW augmente géométriquement (×2 par tranche
de K tasks, configurable) par tuple `(consumer_id, model_id)`.
Difficulté de base = S19 Hashcash 2^18 (~100ms CPU moderne). Reset
quotidien minuit UTC.

**Rejeté** :
- **Adaptive basé charge réseau** (difficulté globale auto-ajustée
  style Bitcoin) : require consensus global absent par design P2P.
  Chaque worker voit un subset de tasks, pas le réseau entier.
- **Flat difficulty elevée** (2^24+ pour tous) : punit les
  legitimate users au même titre que les attackers. Proportionnalité
  requise (HARDENING_ROADMAP §Gate 3 principe).
- **Equi-X memory-hard** (Tor 2023 post-Hashcash) : impl Rust
  `equix 0.3` existe (crate `equix`), mais ajoute dep crypto
  custom non-RFC (Blake2b + 60KB mémoire per-solve). Over-
  engineered pour notre threat model : l'escalation géométrique
  suffit — un Sybil à 1000 identités paie 2^28 à la 10e tranche
  par identité, intenable même avec GPU. Equi-X serait pertinent
  si la base était fixe ET basse.

**Implications** :
- Extension `crates/nexus-core-rs/src/pow.rs` (struct
  `EscalatingPolicy` + `difficulty_for(consumer, model, count)`)
- Storage compteur coord-side dans ledger existant (champ additionnel
  `task_count_daily` par consumer/model dans SQLite compute_ledger)
- Wire-point : `gossip.rs:join_topic_with_pow` accepte
  `DifficultyTarget` dynamic au lieu de const

### D3 — Redundancy voting : fixed factor 3-worker majority

**Retenu** : champ `redundancy_factor: u8` dans `Task` wire format
(valeurs 1/3/5, default 1 = pas de redundancy). Quand factor > 1,
le coordinator dispatch à N workers et agrège via majority vote
(bitwise hash comparison BLAKE3 sur le résultat canonique). Mismatch
= quarantine des outliers + alerte curator.

**Rejeté** :
- **Factor configurable continu** (float, ex. 2.5 = probabilistic
  dispatch) : complexité inutile pré-v1.0. 1/3/5 discret couvre
  100% use-cases identifiés. Runtime overhead O(N) workers est
  acceptable pour N≤5.
- **Coordinator-free consensus** (workers votent entre eux via
  gossip) : require multi-round protocol gossip supplémentaire,
  latence ×3, pas de trust anchor pour trancher mismatch. Le
  coordinator est déjà trust anchor task dispatch.
- **Homomorphic comparison** (compare sans révéler) : crypto lourde,
  aucune lib production-ready Rust 2026 pour ce use-case exact.
  Overkill : les résultats sont déjà signés, la comparaison
  canonique hash suffît.

**Implications** :
- Extension wire format `Task` : champ `redundancy_factor` (pre-
  launch protocol = redéfinit v1 pas bump)
- Nouveau module `packages/nexus-coordinator/src/nexus_coordinator/
  redundancy.py` (dispatch + collect + vote + quarantine route)
- Intégration `dispatcher.py` au path de dispatch existant

### D4 — Honeypot Eclipse detection : canary peer rotation + alert

**Retenu** : le coordinator plante K peers canari (dummy node_id
signés, rotation toutes les 6h) dans le DHT. Si un node_id
worker réapparaît dans le neighborhood de >80% des canary peers
sur 3 rotations consécutives (= 18h), alerte Eclipse potentielle
envoyée au curator pour review manuelle.

**Rejeté** :
- **Traffic analysis passive** (observer le routing table sans
  canary actif) : require accès aux routing tables iroh interne
  (pas exposé API 0.97). Canary peer = seul moyen observable.
- **Automatic quarantine on detection** (pas d'alerte, quarantine
  directe) : false positive rate élevé sur réseau petit (<50
  nodes) — neighborhood naturellement overlap. Review manuelle
  curator seulement. Automation post-Gate 3 quand le réseau est
  > 200 nodes.
- **Gossip-level heartbeat monitoring** (détecter via absence de
  gossip diversity) : indirect, latent, détectable par l'attacker
  (il peut simuler gossip diversity en relayant). Canary peer
  rotation est un signal positif : si l'attacker contrôle le DHT
  segment, il DOIT router vers les canary IDs pour rester crédible.

**Implications** :
- Nouveau module coord-side `honeypot.py` (canary peer factory +
  rotation scheduler + co-location alert)
- Extension `browse_aggregator` daemon-side pour reporter
  neighborhood snapshots (endpoint `/diagnostic/neighborhood`)
- Wire : pas de nouveau wire format (uses existing gossip subscribe)

### D5 — B1 guardrails refactor : distribué S24 Phase A (arbitrage Option B)

**Retenu** : B1 (trait `Guardrail` unifié + `GuardrailChain` +
retrofit 6 primitives) atterrit S24 Phase A comme prérequis naturel
de A1 `TaskDispatchHooks`. S23 ne touche PAS l'architecture
guardrails. Les 6 primitives silos (pii_redactor, output_filter,
quarantine_queue, rate_limit, pii_iframe, canary_input) restent
fonctionnelles en l'état.

**Rejeté** :
- **Option A (B1 dédié S23)** : ~3220 LOC total (dépasse norme
  ~2500), pression scope-cut empirique documentée 3× en S22.
- **Option C (B1 défer S27)** : laisse 6 silos 5+ sprints, chaque
  nouveau guardrail S24-S26 devra être retrofitté ultérieurement.
  Coût retrofit croissant.

**Implications** :
- S23 zero code sur l'unification guardrails
- Design doc `GUARDRAILS_ARCHITECTURE.md` (livré S22 hors-sprint)
  reste référence stable pour S24 kickoff
- S24 kickoff G2 trigger `openai-agents-python > 0.7.0` à vérifier
  avant gel D1 B1

---

## 4.5 Design Review Board findings (G1)

**Report** : `.planning/active/sprint23_design_review.md` (2026-04-20).
**Verdict** : 0 ❌ + 2 ⚠️ + 3 ✅. Procéder Phase A.

### Acknowledged review findings

- **⚠️ D2-G1-1 (Equi-X re-eval)** : ACKNOWLEDGED. La ramp géométrique
  SHA256 2^18 base suffit pour le threat model Gate 2 actuel (réseau
  <200 nodes). Si Sybil volume post-Gate-2 dépasse un seuil empirique,
  Equi-X memory-hard (crate `equix 0.3`) sera re-évalué. Ajout trigger
  S24 audit checklist. Pas de changement D2 S23.
- **⚠️ D3-G1-1 (SecureDrop cite)** : ACKNOWLEDGED. BOINC/Folding@Home
  result validator majority est le meilleur précédent technique (10+
  ans production, hash comparison ×3). Amendement Phase D design note.
  Pas de changement architecture D3.

---

## 5. Phase outline

### Phase A — P2 audit cleanup batch + process fix

- **Scope** : absorber les 6 P2 + 2 P3 triviaux du gate S22
  - P2-S22A-1 : retirer `dashmap` dep directe worker-core
  - P2-S22A-3 : update PATTERNS.md §P33 (RwLock post-wire)
  - P2-B-2 : wrapper.ts L309 commentaire obsolète "scaffold"
  - P2-E-1 : rename `_reload_policy_locked` → `_reload_policy_inner`
  - P2-E-2 : README §6.7 amend (LOC = borne scoping, pas métrique)
  - P2-Meta-hook-1 : clarifier bypass_audit_trail.log forward-only
  - P3-C-1 : re-export DOMAIN_PROVENANCE_V1 + DOMAIN_WARRANT_CANARY_V1
  - P3-B-4 : toFloat32Array test branches defensives (optionnel)
- **Critère** : `cargo nextest run --workspace` vert, `npm run
  test:unit` vert, ruff check vert
- **Commit** : `feat(sprint23): Phase A — P2 cleanup batch S22 audit
  findings + README §6.7 LOC convention amend`

### Phase B — Ephemeral workers (restart + VRAM wipe)

- **Scope** : `ephemeral.rs` lifecycle + `cudaMemset` inter-task +
  config `worker.toml` + state machine integration + tests
- **Critère** : 8+ tests unit (lifecycle state transitions, wipe mock,
  config parse, restart trigger)
- **Commit** : `feat(sprint23): Phase B — ephemeral worker lifecycle
  restart + VRAM cudaMemset wipe inter-task`

### Phase C — Escalating PoW difficulty ramp

- **Scope** : `EscalatingPolicy` struct + `difficulty_for()` +
  daily reset + storage compteur coord-side + wire dynamic difficulty
- **Critère** : 10+ tests (geometric ramp, reset, per-consumer
  isolation, overflow protection, integration pow.rs)
- **Commit** : `feat(sprint23): Phase C — escalating PoW geometric
  ramp per-(consumer, model) with daily reset`

### Phase D — Redundancy voting 3-worker majority

- **Scope** : `redundancy.py` module + Task wire extension
  `redundancy_factor` + dispatcher integration + BLAKE3 hash vote
  + quarantine route outliers
- **Critère** : 12+ tests (dispatch multi, collect, majority vote
  pass, mismatch quarantine, factor=1 passthrough)
- **Commit** : `feat(sprint23): Phase D — redundancy voting
  3-worker majority Task.redundancy_factor + quarantine outliers`

### Phase E — Honeypot Eclipse detection + fairness observability

- **Scope** :
  - `honeypot.py` canary peer factory + rotation 6h + co-location
    alert + daemon neighborhood endpoint
  - `/diagnostic/fairness` endpoint (Gini + top-5% + churn-rate)
- **Critère** : 10+ tests (rotation, alert threshold, false positive
  guard, Gini calculation, edge cases empty ledger)
- **Commit** : `feat(sprint23): Phase E — honeypot Eclipse canary
  peer detection + fairness observability diagnostic endpoint`

### Phase F — Design docs contribution families + wrap-up

- **Scope** :
  - `docs/fairness/CONTRIBUTION_FAMILIES_V1.md` (Option F 3 couches
    asymétriques, design-only)
  - `docs/fairness/KUDOS_V2_WIRE.md` (spec wire, design-only)
  - Couche 3 `DelegationCert` Rust struct format finalization
    (~100 LOC, design-only module)
  - verification.md + audit_plan S24 + migration planning
- **Critère** : 28+ rows fail-fast verts, docs review, PATTERNS
  updated
- **Commit** : `chore(sprint23): Phase F — contribution families
  design docs + Couche 3 cert format + wrap-up + verification +
  audit plan S24`

---

## 6. Scope cuts — ce que Sprint 23 NE fait PAS

1. **B1 guardrails refactor** → S24 Phase A (D5, arbitrage Option B)
2. **Couche 3 DelegationCert implem runtime** → S25-S27 (design-only
   S23 Phase F, implem multi-forge cross-validate séquencée)
3. **Contribution families implem code** → post-v1.0 LT-3 (design
   docs seulement S23, triggers empiriques Gini > 0.70)
4. **Traffic padding** → S28 (aligné Nym mixnet integration)
5. **Exponential cooldown per-identity** → DÉFERÉ (redondant Couche 1
   age gate S22, node_id <7j déjà bloqué)
6. **Honeypot auto-quarantine** → post-Gate 3 (false positive rate
   sur petit réseau <50 nodes)
7. **P2-B-1 ONNX end-to-end CI fixture model** → S24 Track B
   (infrastructure jsdom + 45 MB model, non-résolvable inline)
8. **T-NN+2 iframe Rust-wasm** → PATTERNS §P34 tech debt (triggers
   a/b/c/d non-activés)

---

## 7. Traçabilité scope — mapping carry S22 → S23

| Item carry | Source | Phase S23 | Status |
|---|---|---|---|
| P2-S22A-1 dashmap unused | audit_findings §3 | Phase A | [x] confirmé |
| P2-S22A-3 PATTERNS §P33 | audit_findings §3 | Phase A | [x] confirmé |
| P2-B-2 wrapper.ts comment | audit_findings §3 | Phase A | [x] confirmé |
| P2-E-1 `_reload_policy_locked` | audit_findings §3 | Phase A | [x] confirmé |
| P2-E-2 LOC estimations | audit_findings §3 | Phase A | [x] confirmé |
| P2-Meta-hook-1 bypass log | audit_findings §3 | Phase A | [x] confirmé |
| P3-C-1 DOMAIN re-export | audit_findings §3 | Phase A | [x] confirmé |
| Redundancy voting carry S22 | HARDENING §3 S23 | Phase D | [x] confirmé |
| T-NN+2 iframe Rust-wasm | carry_summary §1 | hors cap formel | [deferred] PATTERNS §P34 |

**Cap G7 bilan** : 0/2 slots carry formels consommés (P2 batch =
cleanup pas carry, redundancy voting = roadmap item pas carry report).
T-NN+2 hors cap.

---

## 8. Audit gate pattern — rappel

- Phase F produira `sprint23_verification.md` + `sprint23_audit_plan.md`
- Sprint 24 Phase 0 jouera l'audit gate en session fraîche
- Convention permanente depuis Sprint 7

---

## 9. Checkpoint de validation

Avant de rédiger le plan détaillé :

- [x] Audit gate S22 PASS confirmé (0 P0 + 0 P1)
- [x] G2 trigger check : aucun trigger actif pour S23
- [x] G6 memory carry-over : fusionné par S22 Phase F
- [x] G7 cap carry-overs : 0/2 slots, T-NN+2 hors cap
- [x] D1..D5 rédigés
- [x] B1 timing arbitré par user (Option B → S24)
- [x] G1 Design Review Board scoring report (`sprint23_design_review.md`)
- [x] Acknowledged review findings (2 ⚠️, 0 ❌)
