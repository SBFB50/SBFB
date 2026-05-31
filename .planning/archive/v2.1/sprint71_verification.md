# Sprint 71 — Verification (assainissement compute + securite Factory + reconciliation off-sprint)

**Ecrit** : 2026-05-31 (Phase E wrap-up).
**Tip master** : `ee8bf6a` a l'entree Phase E ; ce commit `docs(sprint71)`
ferme le sprint. **20 ahead origin, RIEN pousse** (pre-launch §2.3).
**Roadmap** : v5 Factory Complete Vision, Arc 3.5, sprint 1/6 (S71-S76).
**Nature** : sprint de consolidation post-arc. Zero feature speculative
— chaque item est un bug P0/P1, un carry, ou un test E2E manquant ancre
dans le code (kickoff §6.2.1 Regle 1).

---

## §1 Fail-fast checklist

Mesures finales rejouees au Phase E sur le tip `ee8bf6a`. Le full
workspace nextest a tourne **vert sur Windows natif cette fois**
(binaires des phases A-D deja chauds → pas de `os error 1455` au link) ;
le compte canonique reste celui de CI Linux (cf. P2-A-1 / PATTERNS §P54).

| # | Check | Critere | Observed |
|---|-------|---------|----------|
| 1 | `cargo fmt --all --check` | exit 0 | **PASS** (exit 0) |
| 2 | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning | **PASS** (0 warning, tous targets) |
| 3 | `cargo nextest run --workspace --locked` | 0 fail | **PASS** 1528 run / 1528 passed / 0 skipped (35.8 s) |
| 4 | `cargo test --workspace --locked --doc` | 0 fail | **PASS** (0 fail) |
| 5 | `cargo build -p nexus-shell-daemon --release` | OK | **PASS** (exit 0) |
| 6 | B-1 cle alignee (`dispatch_loop.rs`) | `task:` present, `tasks/` absent du writer | **PASS** writer `task:{}` (l.41) ; les 2 hits `tasks/` (l.39,137) sont des commentaires documentant l'ancien bug |
| 7 | B-1 round-trip | vert | **PASS** `dispatch_loop::tests::dispatch_loop_writes_to_doc` asserte `stored_key.starts_with("task:")` (le filtre `test(dispatched_key)` du plan ne matchait pas le nom reel — drift benin) |
| 8 | B-3 E2E cross-process | vert (ou skip Ollama documente) | **PASS** `dispatch_loop::tests::dispatched_task_is_claimed_and_executed_by_worker_engine` (located shell-daemon, pas test-harness — PLAN-ADAPT) ; vert ce run Windows + CI Linux. Caveat Windows-pump P2-A-1 documente §P54 |
| 9 | B-2 greedy seed | vert | **PASS** `engine::runtime::tests::verifiable_task_uses_greedy_seed` |
| 10 | B-2 deux workers meme hash | vert | **PASS** `validator::tests::two_honest_workers_same_hash` |
| 11 | B-2 quorum accepte deterministe | vert | **PASS** `validator::tests::quorum_accepts_deterministic_redundancy` |
| 12 | quorum rejette divergence | vert | **PASS** `validator::tests::quorum_rejects_nondeterministic_divergence` |
| 13 | RedundancyDispatcher resolu | 0 hit vivant | **PASS** module supprime ; 1 hit = commentaire `build_executor.rs:135` (« removed ») |
| 14 | execute_build tranche | cable ou retire | **PASS** garde dormant + documente (consommateur nomme S75 LT-7, `ROADMAP_COMMITMENTS.md` + §P53) |
| 15 | provider/backend documente | distinction documentee | **PASS** `docs/rust/PATTERNS.md §P53` (2 axes orthogonaux) |
| 16 | deps off-sprint au preflight | 3 deps scannees CVE | **PASS** `portable-pty 0.9.0` / `async-stream 0.3.6` / `futures 0.3.32` advisory-clean (preflight B §S1b + §P53 G13) |
| 17 | G2 SSE gate sensible | vert | **PASS** `sse_gates_sensitive_action` |
| 18 | G2 SSE happy-path | vert | **PASS** `sse_allows_nonsensitive` |
| 19 | G9 modele opus | 0 hit `"sonnet"` | **PASS** 0 hit ; defaut `claude-opus-4-8[1m]` + passthrough `req.model` |
| 20 | G9 modele cable | vert | **PASS** `chat_stream_uses_opus_model` |
| 21 | G7 token requis | vert | **PASS** `server_rejects_missing_token` (+ `auth::tests` 5) |
| 22 | G7 Host guard | vert | **PASS** `server_rejects_foreign_host` |
| 23 | G7 CORS restreint | 0 hit `allow_origin(Any)` | **PASS** 0 hit ; CORS pinne origine loopback connue |
| 24 | G12 timeout | vert | **PASS** `llm_bridge::tests::spawn_times_out` |
| 25 | G12 diagnostic | vert | **PASS** `llm_bridge::tests::missing_claude_diagnostic` |
| 26 | Contrat §4 amende | pilotage local gate explicite | **PASS** `RRV_FACTORY_CONTRACT.md` (pilotage agent local **gate** autorise, PO-2) |
| 27 | G6 terminal teste | >= 1 test | **PASS** 2 (`session_log_roundtrip`, `list_sessions_filters_correct_extension`) |
| 28 | G6 sprint_history teste | >= 1 test | **PASS** 3 (`parse_unified_diff_classifies_line_kinds`, `extract_section_stops_at_next_header`, `extract_verdict_reads_plan_adapt`) |
| 29 | G6 process teste | >= 1 test | **PASS** 3 (`resolve_kind_aliases`, `repo_root_resolves`, `providers_list_is_canonical`) |
| 30 | G5 retro-review present | existe | **PASS** `sprint71_offsprint_retro_review.md` (verdict RECONCILED) |
| 31 | G5 retro-Codex present | sortie brute codex | **PASS** `sprint71_offsprint_codex_review.md` |
| 32 | Vitest front | 0 fail | **N/A** — `web/` non touche (diff 201b24d..HEAD sur `web/` = 0) |
| 33 | size-limit | 6/6 | **N/A** — front non touche |
| 34 | scan-en-strings | 0 string EN | **N/A** — front non touche |

**Resultat : 31/31 rows applicables PASS ; 32-34 N/A (front non touche
de tout le sprint).**

---

## §2 Delta tests

| Phase | Type | Rust delta | Detail |
|-------|------|-----------|--------|
| 0 | chore | +0 | audit-absorb retroactif + migration docs S70 (pas de code) |
| A | fix(compute) | +1 | E2E `dispatched_task_is_claimed_...` + `dispatch_loop_writes_to_doc` durci (assert `task:`) |
| B | fix(compute) | +8 net | **+11** task/runtime/ollama/validator/dispatcher (greedy seed + quorum deterministe) **−3** suppression module mort `redundancy` |
| C | fix(factory) | +14 | auth (5) + llm_bridge (2) + operator_server integration (7 : token/Host/CORS/SSE-gate/opus) |
| D | fix(factory) | +16 | terminal (2) + process (3) + sprint_history (3) + endpoints (8, dont 3 securite injection/traversal) |
| E | docs(sprint71) | +0 | wrap-up (ce commit) |
| **Total A-D** | | **+39** | |

| Suite | Entree S71 (S70 close `201b24d`) | Sortie S71 (`ee8bf6a`) | Net |
|-------|----------------------------------|------------------------|-----|
| Rust nextest | 1486 | **1528** | **+42** |
| Vitest | 279 | 279 | +0 |
| size-limit | 6/6 | 6/6 | 0 |

**Reconciliation du compte** : la somme des deltas par phase A-D = +39 ;
l'exit mesure (1528) vs S70 close (1486) = +42. Les +3 residuels = la
surface de test minime qui a lande avec le **bloc off-sprint** Factory
avant la Phase A (bloc reconcilie en Phase D) + l'arrondi
workspace-vs-sous-suite de la Phase A (le body B a baseline le workspace
a « ~1490 »). **1528 est le chiffre liant**, re-mesure au Phase E
(1528 passed, 0 skipped). Aucune regression (0 fail, 0 skip).

---

## §3 Scope cuts compliance

Les 16 scope cuts du plan §12 / kickoff §8. Chaque body de phase a
grep-confirme 0 hit ; Codex a confirme que les refs S75/cross-machine
dans le code sont des deferrals/dormant, pas des implementations.

| # | Item | Sprint cible | Respecte |
|---|------|--------------|----------|
| 1 | ProviderRouter multi-LLM | S72 | OUI — D4 cable seulement defaut+passthrough, pas de router |
| 2 | Chat Factory route reseau | S72 | OUI — aucune logique routage |
| 3 | Pont feed-distant → reindex FTS5 | S73 | OUI — absent |
| 4 | Enrichissement `SearchResult` | S73 | OUI — absent |
| 5 | Barre recherche shell cablee | S73 | OUI — absent |
| 6 | Decision SearchManifest | S73 | OUI — absent |
| 7 | `sbfb-factory search/open/fork` | S74 | OUI — absent |
| 8 | Notion projet cible distinct nexus | S74 | OUI — `repo_root` pointe nexus |
| 9 | Templates etendus (react, pyodide) | S74 | OUI — static + static-reader seuls |
| 10 | GPU partage cross-machine | S75 | OUI — preuve cross-PROCESS seulement |
| 11 | Quorum redundancy>1 cross-MACHINE reel | S75 | OUI — cross-PROCESS same-machine greedy ; limite cross-GPU documentee §P53 |
| 12 | Sharding pipeline | S76 STRETCH | OUI — absent |
| 13 | logprobs/watermark verification | V2 compute | OUI — greedy seed seul, machinerie logprobs inerte |
| 14 | Dashboard kudos per-task | S75 | OUI — absent |
| 15 | @dev index tree-sitter | S71+ post-Gate 1 | OUI — absent |
| 16 | Packaging produit Factory | S74 | OUI — token bootstrap = securite dev, pas onboarding |

**Resultat : 16/16 scope cuts respectes.**

---

## §4 G8 preflight bilan

| Phase | Verdict | Fichier |
|-------|---------|---------|
| 0 | deviation §3 documentee (audit-absorb, PO-3) | `sprint70_audit_findings.md` |
| A | **EXECUTE** | `sprint71_phase_a_preflight.md` |
| B | **PLAN-ADAPT** (backend Ollama gap S1a — Ollama n'attachait jamais `GenerationOptions`) | `sprint71_phase_b_preflight.md` |
| C | **SCOPE-CUT-CONSISTENT** (5 scans S1a/S1b/S2/S3/S4) | `sprint71_phase_c_preflight.md` |
| D | **PLAN-ADAPT** (crate binary-only → tests inline + harness HTTP ; extension `.cast`) | `sprint71_phase_d_preflight.md` |
| E | N/A — clôture docs sans code (le hook lightcheck Check 8 ne s'arme pas : titre sans « Sprint N Phase X ») | — |

**4/4 phases code G8, 0 DESIGN-CONFLICT, 1 EXECUTE / 1 SCOPE-CUT-CONSISTENT
/ 2 PLAN-ADAPT.** PLAN-ADAPT non-consecutifs (B puis D, separes par C
EXECUTE-class) — pas de signal meta (< 2 consecutifs).

> **Note meta-process (P3, pour audit S72)** : le body de la Phase D
> (`f19ed83`) recap les verdicts G8 comme « C=EXECUTE » alors que le
> preflight C reel = **SCOPE-CUT-CONSISTENT**. Inexactitude cosmetique
> du recap, sans impact code. Verdicts reels confirmes ci-dessus depuis
> les fichiers preflight.

---

## §5 Carries

### Carries / gaps CLOSED Sprint 71

| ID | Gravite | Phase cloture | Detail |
|----|---------|---------------|--------|
| B-1 dispatch key | bug routage | A (`2f9238d`) | Writer aligne `task:` ; 1ere tache reellement vue par un worker |
| B-3 E2E cross-process | gap E2E | A (`2f9238d`) | 1er E2E dispatch→claim→exec→result |
| G1 WIP terminal | dette flottante | A (`2f9238d`) | Stash `.cast`→`.log` resolu (drop, asciicast HEAD conserve) |
| B-2 quorum deterministe | bug quorum | B (`0daff81`) | Greedy seed-fixe ; deux workers honnetes → meme hash → quorum accepte |
| D8 modules morts | dette structurelle | B (`0daff81`) | `RedundancyDispatcher` retire ; `execute_build` dormant documente ; provider/backend clarifies |
| G13 deps off-sprint | dette audit | B (`0daff81`) | 3 deps scannees CVE (clean) |
| G2 SSE bypass gate | **P0** securite | C (`a0337c6`) | SSE gate SENSITIVE_ACTIONS (plus de spawn `bypassPermissions` non garde) |
| G7 CORS Any + zero auth | **P1** securite | C (`a0337c6`) | Token X-SBFB-Token + Host guard + CORS restreint |
| G9 modele hardcode sonnet | **P1** regle | C (`a0337c6`) | `claude-opus-4-8[1m]` defaut + passthrough |
| G12 spawn sans timeout | **P1** robustesse | C (`a0337c6`) | Timeout subprocess + diagnostic `claude` resolu |
| G5 bloc off-sprint non reconcilie | **P1** process | D (`f19ed83`) | retro-review (11 dim) + retro-Codex brut + retro-audit |
| G6 surfaces 0 test | **P1** couverture | D (`f19ed83`) | terminal/process/sprint_history/endpoints testes (+16) |

Le retro-review off-sprint (`sprint71_offsprint_retro_review.md`) est
**RECONCILED** : les P0/P1 du bloc (~14 commits, +5500 lignes) = exactement
le scope des phases A-D, fermes in-sprint (pas en `fix(sprint70)` prealable,
deviation PO-3). Phase D a aussi corrige in-phase 2 findings securite du
retro-Codex : **git option injection (P1)** + **drive-prefix (P2)**.

### Nouveaux carries S72 (P2/P3 non bloquants)

| ID | Gravite | Source | Route S72 |
|----|---------|--------|-----------|
| P2-A-1 worker-pump iroh-docs hang Windows natif | P2 | Phase A review | Documente §P54 ; E2E worker-pump = CI Linux only ; candidat investigation surfaces worker |
| P2-A-2 E2E n'asserte pas la signature result | P2 | Phase A review | Optionnel `ResultEntry::verify_signature()` ; signature couverte par units task/validator |
| P3-A-3 `task_id` partage entre 2 tests | P3 | Phase A review | Cosmetique |
| P3-B-1 `as i32` cast seed u32 | P3 | Phase B review | Cosmetique (seed deterministe, pas de perte fonctionnelle) |
| P3-B-2 colonne DB `sha256` misnomer | P3 | Phase B review | Stocke `result_text` brut pour inference ; documente §P53 |
| 3×P2 + 3×P3 Phase C | P2/P3 | Phase C review | Documentes (rigor signal) |
| 3×P2 + 1×P3 Phase D | P2/P3 | Phase D review | Documentes (rigor signal) |

### Carries reconduits (exemptes / hors-scope)

| ID | Compteur | Statut |
|----|----------|--------|
| P2-A-1 (rand upstream) | exemption | Blocker amont, hors scope agent |
| P2-AUDIT-2 (iroh transitives) | herite | Pin iroh 0.98 |
| T-NN+2 (iframe Rust-wasm) | deferred | PATTERNS §P34, upstream wasm |
| P2-F-3 (prompt file coupling) | 2/3 | Non escalade, differe S72 |
| LT-2 (Radicle) | trigger PENDING | Tag v1.0 pas pousse origin |
| LT-5 (redundancy persistence) | reclass S26 | Post-v1.0 horizon long |
| LT-7 (worker quorum build E2E) | partiel | E2E routing Phase A ; quorum build cross-machine → S75 |

---

## §6 Commits Sprint 71

| Ordre | Phase | SHA | Titre |
|-------|-------|-----|-------|
| 1 | chore | `d4bcceb` | redirect S71 — Factory Complete Vision arc (roadmap v5 + intake) |
| 2 | chore | `e92e7d8` | sharding design SOTA addendum (S76 research) |
| 3 | chore | `1190d18` | Sprint 71 kickoff + plan + design_review |
| 4 | Phase 0 | `2ec72e8` | Sprint 71 audit-absorb findings + migrate S70 docs |
| 5 | A | `2f9238d` | fix(compute): Sprint 71 Phase A — align dispatch key + first cross-process E2E |
| 6 | B | `0daff81` | fix(compute): Sprint 71 Phase B — deterministic quorum (greedy seed) + dead module cleanup |
| 7 | C | `a0337c6` | fix(factory): Sprint 71 Phase C — gate SSE + opus-4-8 + token auth + spawn timeout |
| 8 | D | `f19ed83` | fix(factory): Sprint 71 Phase D — reconcile off-sprint block + harden git rev injection |
| 9 | chore | `ee8bf6a` | fix Phase C review verdict format (space before colon) |
| 10 | E | (ce commit) | docs(sprint71): verification + audit plan for Sprint 72 |

Phases code A-D : type `fix` (consolidation — chaque phase ferme un bug
ancre, pas une feature). Phase D recategorisee `test→fix` (a inclus le
fix securite git option injection). Chaque phase A-D : preflight → review
PASS-PENDING → Codex brut → reconciliation → review PASS → body 9 sections.

---

## §7 Arbitrage §11 — decision actee

**Mono-sprint.** La Phase D (reconciliation, R8 = le risque pilote) a
**tenu dans le budget** : retro-review + retro-Codex + retro-audit +
couverture des 5 surfaces off-sprint (+16 tests) livres sans deborder.
Le point de bascule (reconciliation partielle + completion S71-bis/S72)
**n'a pas ete declenche**. S71 reste un seul sprint ; l'arc S72-S76 n'est
pas decale.

---

## §8 Checkpoint de cloture

- [x] Fail-fast checklist §1 : 31/31 rows applicables verts (32-34 N/A front)
- [x] Phases 0 + A-D landed (4 fix + chores), bodies 9 sections phases code
- [x] `sprint70_audit_findings.md` produit (Phase 0 audit-absorb, deviation PO-3)
- [x] Docs S70 migres `active/` → `archive/v2.1/`
- [x] `sprint71_verification.md` + `sprint71_audit_plan.md` ecrits (ce commit)
- [x] `sprint71_offsprint_retro_review.md` + `_codex_review.md` (RECONCILED)
- [x] `docs/rust/PATTERNS.md §P54` (B-1/B-3) + `docs/shell/PATTERNS.md P35` (Factory securite)
- [x] `RRV_FACTORY_CONTRACT.md §4` amende (PO-2)
- [x] B-1 ferme, B-2 quorum deterministe accepte, securite Factory gatee, bloc off-sprint reconcilie
- [x] Stash WIP terminal S71 resolu (drop, asciicast HEAD) — 2 stashes restants pre-existants hors-scope S71
- [x] Arbitrage §11 acte : mono-sprint (Phase D a tenu)
- [x] Memory `nexus_grid_pivot.md` + `MEMORY.md` a jour (ce commit) + SPRINT_LOG row S71
- [x] 4/4 phases code G8 (0 DESIGN-CONFLICT), 16/16 scope cuts respectes

**S71 CLOSED. Arc 3.5 (Factory Complete Vision) demarre ; S72
ProviderRouter debloque.**
