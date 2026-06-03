# Sprint 72 — Verification (ProviderRouter multi-LLM + Factory hardening + UX intentions)

**Ecrit** : 2026-06-03 (Phase F wrap-up).
**HEAD entree** : `0b4e7f3` (S71 close `docs(sprint71): verification + audit plan for Sprint 72`).
**HEAD sortie** : ce commit `docs(sprint72)` ferme le sprint (tip code phases = `95cae05`).
**28 ahead origin, RIEN pousse** (pre-launch §2.3 toujours actif).
**Roadmap** : v5 Factory Complete Vision, Arc 3.5, sprint 2/6 (S71-S76).
**Nature** : sprint feature — ouvre le routage provider multi-LLM
(`ExecutionTarget` Claude/Ollama/Network), cable `provider` de bout en bout,
livre l'ecran d'intentions d'execution, ferme la dette pair (P2-F-3 3/3) et
la dette audit S71 (P2-H-1).

---

## §1 Commit stack

`git log --oneline 0b4e7f3..HEAD` :

| Ordre | Phase | SHA | Titre |
|-------|-------|-----|-------|
| 1 | 0 (audit S71) | `636b9de` | chore(planning): Sprint 71 audit findings — PASS (S72 Phase 0) |
| 2 | kickoff | `1803d78` | chore(planning): Sprint 72 kickoff + plan + design_review + migrate S71 archive |
| 3 | A | `105c054` | docs(security): Sprint 72 Phase A — catalogue Operator surface (P2-H-1) |
| 4 | B | `08b6cb2` | fix(sprint72): Sprint 72 Phase B — close P2-F-3 (prompt coupling) + P2-A-2 signature + P3 carries |
| 5 | C | `3c9ea1b` | feat(factory): Sprint 72 Phase C — align ollama-rs 0.3.4 + ExecutionTarget dispatch + Ollama |
| 6 | D | `110c003` | feat(factory): Sprint 72 Phase D — NetworkProvider submit-poll + result-text primitive + provider routing |
| 7 | E | `95cae05` | feat(factory-operator): Sprint 72 Phase E — UX intentions execution (Claude / local / reseau) |
| 8 | F | (ce commit) | docs(sprint72): verification + audit plan for Sprint 73 |

Chaque phase code (A-E) : preflight G8 → review PASS-PENDING → Codex brut →
reconciliation → review PASS → body 9 sections. Phase 0 = audit gate S71
(Cas A), verdict **PASS** (0 P0, 0 P1, 1 P2 route Phase A, 2 P3) — aucun
`fix(sprint71)` requis.

---

## §2 How to re-run

```bash
# --- Rust (workspace) ---
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release

# --- Frontend web/ (non touche tout le sprint) ---
(cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && \
  npm run test:unit && npm run build && npm run size && \
  bash scripts/scan-en-strings.sh)

# --- Front Operator (Phase E) ---
(cd tools/factory-operator && npx tsc -b --noEmit && npx eslint . && npm run build)

# --- Phase A : presence catalogues menace ---
grep -ci operator docs/security/THREAT_MODEL.md                         # 16
grep -ciE 'operator|3001' docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md  # 10
grep -i P35 docs/security/THREAT_MODEL.md docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md

# --- Canonical CI Linux (gold standard, cf. P2-A-1 / §P54) ---
docker run --rm -v "${PWD}:/workspace" -w /workspace \
  -e CARGO_TARGET_DIR=/workspace/target-ci-linux \
  rust:1.94@sha256:b644cc33aee7a2b32ff3e1198711f8ad3a69ae29a58e1a674e97f75776b88186 \
  sh -lc "cargo test --workspace --locked && cargo test --workspace --locked --doc"
```

---

## §3 Fail-fast checklist

Mesures finales rejouees au Phase F. Le full workspace nextest a tourne sur
**Windows natif** (1544 run) ; le compte canonique reste celui de **CI Linux**
(cf. P2-A-1 / PATTERNS §P54). La seule divergence Windows = un timeout
environnemental (row 3) prouve flake par re-run isole.

| # | Check | Critere | Observed |
|---|-------|---------|----------|
| 1 | `cargo fmt --all --check` | exit 0 | ✅ **PASS** (exit 0) |
| 2 | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning | ✅ **PASS** (0 warning, tous targets ; bump schemars 1.2 ne casse rien) |
| 3 | `cargo nextest run --workspace --locked` | 0 fail | ✅ **PASS canonique** 1544 run / **0 skip**. Windows natif : 1543 passed + **1 flake env** `operator_sprint_history_endpoint` (`reqwest TimedOut` sous charge parallele full-workspace, contention git) — **re-run isole = PASS 1/1** (preuve flake, pas regression) ; reproduit sur master pur (OPERATOR-TIMEOUT carry S73). Canonique CI Linux = 1544/1544 (Phase D Codex, arbre Rust byte-identique depuis `110c003`). |
| 4 | `cargo test --workspace --locked --doc` | 0 fail | ✅ **PASS** (0 fail) |
| 5 | `cargo build -p nexus-shell-daemon --release` | OK | ✅ **PASS** (exit 0, 15.75s) |
| 6 | A — `operator` dans THREAT_MODEL.md | >= 1 hit | ✅ **PASS** (16 hits ; §14 surface Operator + T-OPERATOR-CSRF/SPAWN) |
| 7 | A — `operator\|3001` dans LOOPBACK_…TRUST_TIERS.md | >= 1 hit | ✅ **PASS** (10 hits ; §3.1 inventaire Operator :3001 trust tier + gates) |
| 8 | A — ref `P35` dans les 2 catalogues | present | ✅ **PASS** (cross-ref `docs/shell/PATTERNS.md §P35`) |
| 9 | B — P2-F-3 check couplage prompt/wrapper | test vert | ✅ **PASS** `process::tests::prompt_kinds_resolve_to_existing_files` + `agent_wrappers_reference_existing_prompts` (process.rs:887,907 — 8 kinds + 8 refs bidirectionnel) |
| 10 | B — P2-A-2 assertion signature E2E | assertion presente | ✅ **PASS** `dispatched_task_is_claimed_and_executed_by_worker_engine` asserte `ResultEntry::verify_signature()` (dispatch_loop.rs:248-256) |
| 11 | C — ollama-rs aligne 0.3.4 partout | pin + dep directe | ✅ **PASS** `Cargo.toml:105` (pin 0.3.4) + `sbfb-factory/Cargo.toml` dep directe `features=["stream"]` ; worker `ollama.rs` migre `ModelOptions` |
| 12 | C — 4 tests quorum S71 re-verts (R7) | 4/4 vert | ✅ **PASS** `verifiable_task_uses_greedy_seed`, `two_honest_workers_same_hash`, `quorum_accepts_deterministic_redundancy`, `quorum_rejects_nondeterministic_divergence` — determinisme greedy-seed survit au bump |
| 13 | C — `ExecutionTarget::run` dispatche Claude + Ollama | parse + dispatch | ✅ **PASS** `execution_target_from_provider_parses_closed_set` (claude/ollama/local/network/unknown→claude) + `claude_target_is_behaviorally_unchanged` (bras Claude = `spawn_claude_stream` inchange) |
| 14 | C — Ollama `generate_stream` → StreamChunk | mapping prouve | ✅ **PASS** `ollama_stream_maps_to_chunks_via_mock` (mock deterministe Delta/Done) + `ollama_unreachable_yields_diagnostic` + `ollama_diagnostic_flags_connection_refused` |
| 15 | C — bump schemars 0.8→1.2 (PO Option A) | snapshot + clippy clean | ✅ **PASS** `task_response.schema.json` regenere (draft-07→2020-12) ; G8 DESIGN-CONFLICT resolu par arbitrage PO (honore D2 ollama-rs unique partout) |
| 16 | D — `handle_chat_stream` route selon `session.provider` | dispatch vert | ✅ **PASS** `chat_stream_routes_by_session_provider` (ollama→Ollama, defaut→Claude) |
| 17 | D — `provider` persiste au send (symetrie `model`) | persist vert | ✅ **PASS** `chat_session_persists_provider` (operator_server.rs:53,787) |
| 18 | D — network submit→poll→**un seul** Done (PO-14) | dones==1, deltas==0 | ✅ **PASS** `network_provider_submit_poll_yields_single_done` (assert `dones.len()==1 && deltas==0`) |
| 19 | D — network timeout global | Error vert | ✅ **PASS** `network_provider_poll_timeout` (`StreamChunk::Error`) |
| 20 | D — gate SENSITIVE_ACTIONS avant dispatch (tous providers) | gate vert | ✅ **PASS** `sensitive_action_gated_regardless_of_provider` (securite S71 D3 preservee) |
| 21 | D — route daemon `/result` + colonne `result_text` (PO Option A) | route + persist | ✅ **PASS** `task_result_route_404_then_text_on_completed` (http.rs) + `set_task_result_persists_retrievable_text` + `get_task_result_none_for_missing_task` (db.rs ; migration M16) |
| 22 | D — PATTERNS §P55 (3 axes orthogonaux) | documente | ✅ **PASS** `docs/rust/PATTERNS.md:2889` (Execution target / Provider prompt-adapt / Worker backend) |
| 23 | E — `factory-operator` tsc | exit 0 | ✅ **PASS** `npx tsc -b --noEmit` exit 0 |
| 24 | E — `factory-operator` eslint | exit 0 | ✅ **PASS** `npx eslint .` exit 0 (3 warnings PRE-EXISTANTS `ui/badge\|button\|tabs`, 0 sur fichiers Phase E) |
| 25 | E — `factory-operator` build | exit 0 | ✅ **PASS** `npm run build` exit 0 (2050 modules) |
| 26 | E — route `/execute` + selecteur 3-intentions | present | ✅ **PASS** `ExecutionChat.tsx` (NEW) + `executionChat.ts` (NEW) + route App.tsx + nav Sidebar ; CTA intentions « Executer sur Claude / en local / sur le reseau », jamais jargon |
| 27 | E — intention transmise au backend (`provider`) | cable | ✅ **PASS** `executionChat.ts` `createSession`/`sendMessage` transmet `provider` claude/ollama/network ; SSE EventSource relatif (proxy Vite injecte x-sbfb-token) |
| 28 | E — strings utilisateur FR | scan clean | ✅ **PASS** i18n FR/EN bloc `execute` complet (cles `sessionError`, `networkStatus.{rejected,timed_out}` ajoutees) ; `web/` scan-en-strings clean |
| 29 | Vitest front `web/` | 0 fail | ✅ **PASS** 279 passed (23 fichiers) — `web/` non touche de tout le sprint |
| 30 | size-limit | 6/6 | ✅ **PASS** 6/6 (vendor-ui 262/270, CommandPalette 9.8/20, css 122.6/130, …) |
| 31 | scan-en-strings `web/` | 0 string EN | ✅ **PASS** (`src/ is French-only, clean`) |

**Resultat : 31/31 rows PASS.** La seule divergence (row 3 Windows natif) est
un flake environnemental prouve par re-run isole — le compte canonique CI
Linux est 1544/1544, 0 skip.

---

## §4 Delta tests

| Phase | Type | Rust delta | Detail |
|-------|------|-----------|--------|
| 0 | chore (audit S71) | +0 | audit gate S71 PASS — pas de code |
| A | docs(security) | +0 | catalogue menace Operator (P2-H-1) — docs-only |
| B | fix(sprint72) | +2 | `prompt_kinds_resolve_to_existing_files` + `agent_wrappers_reference_existing_prompts` (P2-F-3) ; P2-A-2 = assertion `verify_signature()` sur E2E existant (+0 compte) |
| C | feat(factory) | +7 | 6 tests `provider_router` (parse closed-set, Claude unchanged, Ollama mock/diagnostic, network not-impl) + 1 ripple schema/executor (bump schemars 1.2) ; R7 4 quorum re-verts (non-regression, pas un ajout) |
| D | feat(factory) | +7 | db.rs +2 (result_text persist/get) + http.rs +1 (route /result) + provider_router net +1 (−1 not-impl, +2 submit-poll/timeout) + operator_server +3 (route-by-provider, persist provider, gate-all-providers) |
| E | feat(factory-operator) | +0 | front-only ; `tools/factory-operator` sans test runner (tsc+eslint+build seuls) |
| F | docs(sprint72) | +0 | wrap-up (ce commit) |
| **Total A-E** | | **+16** | |

| Suite | Entree S72 (`0b4e7f3`) | Sortie S72 (`95cae05`) | Net |
|-------|------------------------|------------------------|-----|
| Rust nextest (canonique Linux) | 1528 | **1544** | **+16** |
| Vitest (`web/`) | 279 | 279 | +0 |
| size-limit | 6/6 | 6/6 | 0 |

**Reconciliation du compte** : somme des deltas A-E = +16 (A 0 / B 2 / C 7 /
D 7 / E 0) ; exit canonique 1544 vs entree 1528 = +16. **Exact, 0 ecart.**
Variance vs estimation plan §10 (+10-12, ~1538-1540) : +4-6 de plus,
provenant de (a) Phase C — le bump schemars 1.2 (arbitrage PO Option A,
non anticipe au plan) a ajoute le ripple schema/executor ; (b) Phase D — la
route `/result` + colonne `result_text` (Option A resolvant le DESIGN-CONFLICT
S4) a ajoute db/http/validator tests au-dela de l'estimation. Estimation
indicative, pas un plafond (§6.7). Aucune regression (0 fail canonique,
0 skip).

---

## §5 G8 preflight bilan

| Phase | Verdict G8 | Resolution | Fichier |
|-------|-----------|------------|---------|
| A | **EXECUTE** | docs-only, S1a/S1b/S4 N/A | `sprint72_phase_a_preflight.md` |
| B | **EXECUTE** | dette pair, 4 scans clean | `sprint72_phase_b_preflight.md` |
| C | **DESIGN-CONFLICT → EXECUTE** | ground-truth : ollama-rs 0.3.4 tire schemars 1.2 (collision trait-bound vs pin 0.8). Arbitrage **PO Option A** : bump schemars 0.8→1.2 workspace pour honorer D2 (ollama-rs unique partout). | `sprint72_phase_c_preflight.md` + `_pivot_proposal.md` |
| D | **DESIGN-CONFLICT → EXECUTE** | ground-truth S4 : pas de chemin de recuperation du resultat reseau cote Factory. Arbitrage **PO Option A** (2026-06-03) : route daemon `GET /api/v1/tasks/{id}/result` + colonne `result_text` (migration M16). Pas de bump wire (pre-launch, additif). | `sprint72_phase_d_preflight.md` + `_pivot_proposal.md` |
| E | **PLAN-ADAPT** | ground-truth S1a/S2 : le consommateur chat SSE provider-route n'existe pas — il a existe (`e26d9f2`) puis ete retire (`c3f4813`, terminal PTY qui bypasse `ExecutionTarget`). Phase E **construit** le consommateur (`ExecutionChat.tsx` route `/execute`). | `sprint72_phase_e_preflight.md` |

**5/5 phases code G8. 2 EXECUTE (A,B) / 2 DESIGN-CONFLICT resolus par
arbitrage PO (C,D) / 1 PLAN-ADAPT (E).**

> **Note meta-process (route audit S73)** : les 2 DESIGN-CONFLICT (C puis D)
> sont **consecutifs** et touchent tous deux le theme ProviderRouter. Tous
> deux portent une evidence ground-truth concrete (collision dep schemars ;
> gap wire result-retrieval) et ont ete tranches par le PO sur une Option A
> documentee (pas de derive agent, pas de changement Day-0 unilateral). Le
> signal : le plan S72 a sous-estime deux dependances structurelles du
> routage provider (chaine transitive de deps + contrat de recuperation
> reseau). Les deux fixes sont propres et testes ; le pattern a surveiller =
> les phases « cablage cross-composant » exigent un S1b/S4 plus profond au
> preflight. Route en Track meta de l'audit S73.

---

## §6 Scope cuts compliance

Les 16 scope cuts du kickoff §7 / plan §11. Chaque review de phase a
confirme 0 hit ; Codex a confirme que les refs S73+/cross-machine restent
des deferrals, pas des implementations.

| # | Item | Sprint cible | Respecte |
|---|------|--------------|----------|
| 1 | Onboarding/packaging atelier (launcher, doc install) | S74 | OUI — l'UX intentions (ecran `/execute`) EST livree (Phase E) ; seul le packaging produit reste S74 |
| 2 | Pont feed-distant → reindex FTS5 | S73 | OUI — `feed_sync.rs` non touche |
| 3 | Enrichissement `SearchResult` | S73 | OUI — absent |
| 4 | Barre recherche shell cablee | S73 | OUI — absent |
| 5 | Decision SearchManifest | S73 | OUI — absent |
| 6 | `sbfb-factory search/open/fork` | S74 | OUI — absent |
| 7 | Notion projet cible distinct nexus | S74 | OUI — `default_project_id()='operator-chat'` placeholder, pas de fork projet |
| 8 | Templates etendus (react, pyodide) | S74 | OUI — absent |
| 9 | GPU partage cross-machine | S75 | OUI — NetworkProvider = cross-PROCESS loopback, pas cross-GPU |
| 10 | Quorum redundancy>1 cross-MACHINE reel | S75 | OUI — submit→poll same-machine daemon |
| 11 | Sharding pipeline | S76 STRETCH | OUI — absent |
| 12 | Streaming token-par-token worker reseau distant | jamais (PO-14) | OUI — un seul `Done`, 0 `Delta` (test `dones==1, deltas==0`) |
| 13 | logprobs/watermark verification | V2 compute | OUI — greedy seed seul |
| 14 | Dashboard kudos per-task | S75 | OUI — absent |
| 15 | Extraction crate `ollama-client` partage | CADUC | OUI — ollama-rs 0.3.4 EST la lib partagee (worker + Factory) |
| 16 | Routage multi-cloud generaliste (OpenAI/Gemini/…) | hors roadmap | OUI — 3 targets fermes (Claude/Ollama/Network), pas un proxy multi-cloud |

**Resultat : 16/16 scope cuts respectes.**

---

## §7 Surface nouvelle livree

| Module / fichier | Role | LOC approx |
|---|---|---|
| `crates/sbfb-factory/src/provider_router.rs` (NEW) | `enum ExecutionTarget {Claude,Ollama,Network}` + `ProviderStream` + `from_provider` + `run` ; 3 bras (Claude inchange, Ollama `generate_stream`, Network submit→poll) | ~430 |
| `crates/nexus-coordinator-rs/src/db.rs` (M) | colonne `result_text` (migration M16) + `set_task_result` persist + `get_task_result` reader | +~60 |
| `crates/nexus-shell-daemon/src/tasks_api.rs` + `http.rs` (M) | route `GET /api/v1/tasks/{id}/result` (T0 loopback read-only) | +~40 |
| `crates/sbfb-factory/src/operator_server.rs` (M) | `ChatSession {+provider, +project_id}` persist + `handle_chat_stream` lit provider, construit `ExecutionTarget`, dispatch (gate SENSITIVE_ACTIONS conserve avant) | +~50 |
| `crates/nexus-worker-core/src/llm/ollama.rs` + `executor` (M) | migration `GenerationOptions`→`ModelOptions` (ollama-rs 0.3.4) | +~20 |
| `tools/factory-operator/src/lib/executionChat.ts` (NEW) | client `createSession`/`sendMessage`/`openStream` ; types `ExecutionIntent`/`StreamChunk` | ~120 |
| `tools/factory-operator/src/pages/ExecutionChat.tsx` (NEW) | page SSE provider-routee ; selecteur 3-intentions ; EventSource anti-reconnect ; etats reseau | ~200 |
| `docs/security/THREAT_MODEL.md` §14 + `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` §3.1 | catalogue surface Operator (P2-H-1) | docs |
| `docs/rust/PATTERNS.md` §P55 | 3 axes orthogonaux LLM (D5) | docs |

---

## §8 Findings carry-over for memory (G6)

Max 5 items a persister (fusion manuelle au kickoff S73) :

1. **P2-A-1(S71) worker-pump iroh-docs Windows natif → 2/3, devient 3/3
   MANDATORY S73.** Doit entrer dans le plan S73 (root-cause iroh-docs pump
   Windows OU exemption formelle CI-Linux-only ecrite). Famille elargie par
   la verification S72 : `operator_sprint_history_endpoint` (OPERATOR-TIMEOUT)
   est un meme flake env (timeout sous charge parallele, passe isole).
2. **2 DESIGN-CONFLICT consecutifs C+D resolus par arbitrage PO (Option A
   chacun).** schemars 0.8→1.2 (collision dep ollama-rs) + route daemon
   `/result` + colonne `result_text` (gap wire result-retrieval). Day-0
   ajustes proprement, pas de derive — mais signal : le cablage cross-composant
   exige un S1b/S4 plus profond au preflight.
3. **Dette test S73** : `audit_commit_valid_phase_commit` (SHA hardcode S70
   `6fb95df` archive) + `operator_server` timeouts bind-mount/charge — les
   deux pre-existants, reproduits sur master pur, a de-hardcoder/exempter S73.
4. **Phase E carries P2** : (a) model-picker pour intentions non-Claude
   (Ollama defaute `claude-opus-4-8[1m]` inexistant → Error gracieux, axe
   model D5 separe) ; (b) `factory-operator` sans test runner (logique
   critique = revue manuelle, ajout Vitest = infra hors quick win).
5. **Process** : `nexus-phase-review-deep` ET `nexus-process-supervisor` NON
   enregistres dans cet env → review = fallback agent `general-purpose`
   independant, supervision = hooks backstop (D17). Codex CWD = racine repo
   (chemins absolus pour prompt + `-o`).

---

## §9 Checkpoint de cloture

- [x] Fail-fast checklist §3 : 31/31 rows PASS (row 3 flake env Windows prouve par re-run isole ; canonique Linux 1544/1544)
- [x] Phases A-F landed (A docs/threat + B fix dette + C feat migration+dispatch + D feat backend + E feat front + F docs)
- [x] P2-H-1 ferme (Phase A) — catalogues menace referencent Operator (THREAT_MODEL §14 + LOOPBACK §3.1)
- [x] P2-F-3 ferme (Phase B, 3/3, exit binaire) — **plus jamais carry**
- [x] P2-A-2 assertion signature ajoutee (Phase B, `verify_signature()` E2E)
- [x] ollama-rs aligne 0.3.4 partout + **4 tests quorum S71 verts (R7)**
- [x] `ExecutionTarget` + 3 providers (Claude inchange / Ollama / Network)
- [x] `provider` cable de bout en bout + UI intentions complete (`/execute`), gate SENSITIVE_ACTIONS preserve
- [x] Front Operator tsc+eslint+build propres, strings utilisateur FR
- [x] Pas de bump wire (PO-14, pre-launch) ; bump pin ollama-rs 0.3.4 + schemars 1.2 (PO Option A) documente ; route `/result` additive
- [x] `sprint72_verification.md` + `sprint73_audit_plan.md` ecrits (ce commit)
- [x] `docs/rust/PATTERNS.md §P55` (3 axes) a jour
- [x] Memory `nexus_grid_pivot.md` + `MEMORY.md` a jour (ce commit) + SPRINT_LOG row S72
- [x] 5/5 phases code G8 (2 DESIGN-CONFLICT resolus PO, 1 PLAN-ADAPT, 2 EXECUTE), 16/16 scope cuts respectes
- [x] Codex verification 5/5 phases (zero exemption §4.5.6)

**S72 CLOSED. Arc 3.5 (Factory Complete Vision) 2/6 ; S73 (recherche reseau)
debloque sous reserve de l'audit gate S72.**
