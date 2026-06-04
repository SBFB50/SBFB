# Sprint 73 — Verification (Recherche reseau cablee — FTS5 fraicheur + SearchResult enrichi + barre shell + guardrail securite + dette)

**Ecrit** : 2026-06-04 (Phase F wrap-up).
**HEAD entree** : `087e781` (audit findings S72 PASS — S73 Phase 0).
**HEAD sortie** : ce commit `docs(sprint73)` ferme le sprint (tip code phases = `9472085`).
**37 ahead origin, RIEN pousse** (pre-launch §1.4 toujours actif).
**Roadmap** : v5 Factory Complete Vision, Arc 3.5, sprint **3/6** (S71-S76).
**Nature** : sprint impair a **forte charge dette** — ouvre la recherche reseau
(reindex FTS5 a chaud + triplet provenance + barre shell), corrige un invariant
securite (guardrail AVANT persist), et ferme **7 P2 dette** dont **P2-A-1
worker-pump 3/3 MANDATORY** (plus jamais carry).

---

## §1 Commit stack

`git log --oneline 087e781..HEAD` :

| Ordre | Phase | SHA | Titre |
|-------|-------|-----|-------|
| 1 | 0 (audit S72) | `087e781` | chore(planning): Sprint 72 audit findings — PASS (S73 Phase 0) |
| 2 | kickoff | `845bea6` | chore(planning): Sprint 73 kickoff + plan + design_review + migrate S72 archive |
| 3 | A | `6f5ff30` | fix(sprint73): Sprint 73 Phase A — guardrail before result_text persist (2 paths) + Operator tier + hardening-roadmap recadre |
| 4 | (chore) | `5361fd8` | chore(planning): normalize Sprint 73 review verdict header + artifact refs |
| 5 | B | `a4e1542` | fix(sprint73): Sprint 73 Phase B — close P2-A-1 worker-pump 3/3 (multi_thread) + test debt + NetworkProvider/Operator hardening |
| 6 | C | `47c9ff7` | feat(search): Sprint 73 Phase C — hot incremental FTS5 reindex on feed ingest (freshness) |
| 7 | D | `0f86e5a` | feat(search): Sprint 73 Phase D — enrich SearchResult with provenance triplet (M17 UNINDEXED) + SearchManifest design note |
| 8 | E | `9472085` | feat(shell): Sprint 73 Phase E — wire Browse search bar to GET /api/daemon/search |
| 9 | F | (ce commit) | docs(sprint73): verification + audit plan for Sprint 74 |

Chaque phase code (A-E) : preflight G8 → review PASS-PENDING → Codex brut →
reconciliation → review PASS → body 9 sections. Phase 0 = audit gate S72
(Cas A), verdict **PASS** (0 P0, 0 P1, 12 P2, 10 P3) — aucun `fix(sprint72)`
requis ; les 12 P2 routes au plan S73 (Phase A doc lot + Phase B dette + Phase F
preflight).

---

## §2 How to re-run

```bash
# --- Rust (workspace) — Windows natif ---
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release

# --- Worker-pump closure gate (P2-A-1) : cargo test SHARED-PROCESS (le mode
#     ou le hang original se manifestait — PAS nextest qui isole par process) ---
cargo test -p nexus-shell-daemon -p nexus-worker-core --locked   # Windows natif

# --- Canonique CI Linux (gold standard) : sbfb-ci + libgtk-3-dev (atk-sys) ---
MSYS_NO_PATHCONV=1 docker run --rm \
  -v "${PWD}:/workspace" -w /workspace -e CARGO_TARGET_DIR=/tmp/ci-target \
  sbfb-ci:latest bash -c "apt-get update -qq && apt-get install -y -qq libgtk-3-dev && \
    cargo nextest run --workspace --locked && cargo test --workspace --locked --doc"

# --- Frontend web/ (Phase E) ---
(cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && \
  npm run test:unit && npm run build && npm run size && \
  bash scripts/scan-en-strings.sh)

# --- Front Operator (Phase B : infra Vitest NEW) ---
(cd tools/factory-operator && npx vitest run)

# --- Phase A : claims doc corrigees ---
grep -n "AVANT" docs/security/THREAT_MODEL.md             # §14 guardrail avant persist
grep -niE "operator|3001" docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md  # tier formel §2.1/§8.1

# --- Phase D : design note SearchManifest present ---
test -f .planning/research/s73_searchmanifest_index_node_design.md
```

> **Note process (Docker-sur-Windows)** : sans `libgtk-3-dev` le
> `cargo test --no-run` echoue a la compilation (`atk-sys` / GTK stack tire par
> le tray-icon launcher) — et un exit code non-zero peut etre **masque** par un
> `| tail`. `MSYS_NO_PATHCONV=1` + `bash -c` (pas `-l`) + `CARGO_TARGET_DIR`
> isole (eviter la contention avec le target Windows natif tournant en
> parallele) sont obligatoires.

---

## §3 Fail-fast checklist

Mesures finales rejouees au Phase F (2026-06-04). Le full workspace a tourne sur
**Windows natif** (1566 nextest) **et** **Docker Linux canonique** (sbfb-ci,
rustc 1.95.0, 1570 nextest). Le compte canonique reste **CI Linux**. La
divergence Windows/Linux (+4) = tests `#[cfg(unix)]` (UDS peer-cred, e2e
unix-gates) absents sous Windows — structurel, 0 skip sur les deux.

| # | Check | Critere | Observed |
|---|-------|---------|----------|
| 1 | `cargo fmt --all --check` | exit 0 | ✅ **PASS** (exit 0) |
| 2 | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning | ✅ **PASS** (0 warning, tous targets) |
| 3 | `cargo nextest run --workspace --locked` | 0 fail | ✅ **PASS** — **canonique CI Linux 1570/1570, 0 skip** (sbfb-ci rustc 1.95.0) ; Windows natif **1566/1566, 0 skip** (+4 = `#[cfg(unix)]` Linux-only, structurel). 0 fail sur les deux. |
| 4 | `cargo test --workspace --locked --doc` | 0 fail | ✅ **PASS** (Windows + Docker Linux, 0 fail) |
| 5 | `cargo build -p nexus-shell-daemon --release` | OK | ✅ **PASS** (exit 0, 15.98s) |
| 6 | A — guardrail HTTP rejet | vert (0 persist) | ✅ **PASS** `submit_result_rejected_by_guardrail_persists_nothing` — trip → 400 + `get_task_result().result_text.is_none()` + status non `completed` + 0 kudos (`http.rs:1516-1530`) |
| 7 | A — guardrail accepte | vert | ✅ **PASS** `submit_result_accepted_persists_after_guardrail` — pass → 200 + `result_text` lisible |
| 8 | A — validator_loop rejet | vert | ✅ **PASS** `validator_loop_rejected_result_not_persisted` — gossip rejet → skip persist + 0 kudos (chemin gossip n'avait **aucun** guardrail avant S73) |
| 9 | A — quorum guardrail texte agree | vert | ✅ **PASS** `quorum_guardrail_runs_on_agreed_text` (redundancy>1, guardrail sur `best_hash` agree) + `validator_loop_accepted_result_persisted` |
| 10 | A — claim doc corrigee | « AVANT persist » | ✅ **PASS** `THREAT_MODEL §14:789-793` (« `default_output_chain` tourne AVANT `set_task_result` ») + `LOOPBACK §3` (GET /result reordonne) |
| 11 | A — Operator tier formel | rows §2.1/§8.1 | ✅ **PASS** `LOOPBACK §2.1` (Operator `:3001` T0 uniformement + gate SENSITIVE_ACTIONS) + §8.1 couverture (P2-TIER-MODEL) |
| 12 | B — worker-pump multi_thread Windows | vert (carry CLOSED) | ✅ **PASS** — inclus dans nextest 1566 (0 skip, 0 `#[ignore]`) **ET** `cargo test` shared-process (gate d'origine du hang) : `engine_claims_and_executes_tasks_on_registered_doc`, `rate_limit_gate_reloads_live_policy`, dispatch E2E tous **ok**, 190 passed / 0 ignored / exit 0 |
| 13 | B — worker-pump Docker Linux | vert | ✅ **PASS** — inclus dans nextest Linux 1570 (0 skip) |
| 14 | B — zombie de-hardcode | vert master pur | ✅ **PASS** `audit_commit_valid_phase_commit` + `_non_phase_commit` (repo git fixture self-contained, plus de SHA S70 `6fb95df` hardcode) |
| 15 | B — operator_server serialise | vert Windows | ✅ **PASS** `operator_sprint_history_endpoint` passe (Windows natif 20.6s, timeout 30s configurable ferme OPERATOR-TIMEOUT) + serial_test `#[serial(sbfb_env)]` |
| 16 | B — factory-operator Vitest | vert (infra NEW) | ✅ **PASS** **7 passed** (2 fichiers ; jsdom + MockEventSource ; gate short-circuit + StreamChunk mapping) |
| 17 | B — NetworkProvider last_err | vert | ✅ **PASS** `last_err` memorisee + surfacee au timeout (`provider_router.rs:386`) |
| 18 | B — model-picker non-Claude | vert | ✅ **PASS** `non_claude_providers_do_not_inherit_the_claude_model` + `claude_provider_keeps_the_frozen_opus_model` (per-provider, plus de defaut `claude-opus-4-8` aux providers Ollama/Network) |
| 19 | C — reindex chaud | hit sans reboot | ✅ **PASS** `feed_ingest_indexes_entry_hot` — `upsert_feed_entry` apres insert → `search()` trouve l'entree sans reboot |
| 20 | C — idempotent | 1 ligne | ✅ **PASS** `reindex_hot_is_idempotent` — re-upsert meme `seq` → no-op rewrite |
| 21 | C — helper partage | vert | ✅ **PASS** `extract_index_fields_shared_with_rebuild` (anti-derive hot/rebuild) |
| 22 | C — reader non bloque | vert | ✅ **PASS** test interleave (WAL, 1 stmt, single std::Mutex serialise writer/reader) + `busy_timeout(5s)` explicite |
| 23 | D — triplet provenance | 4 champs | ✅ **PASS** `search_result_carries_provenance_triplet` — hit `ReleasePublished` porte repo_url+commit_sha+archive_hash+provenance_hash (name bridge `artifact_hash`→`archive_hash`) |
| 24 | D — migration M17 | rebuild repopule | ✅ **PASS** `migration_m17_recreates_index_unindexed` (DROP/recreate FTS5 + 5 col UNINDEXED ; `rebuild_from_feed` repopule, 0 perte feed) |
| 25 | D — UNINDEXED | vert | ✅ **PASS** `enriched_fields_unindexed_not_matchable` — un MATCH sur un hash ne retourne pas |
| 26 | D — JSON endpoint | 5 champs | ✅ **PASS** `search_handler_json_includes_triplet` — reponse JSON porte les 5 cles (toujours-presentes, `null` si absent) |
| 27 | D — design note present | present | ✅ **PASS** `.planning/research/s73_searchmanifest_index_node_design.md` (NEW, noeud-index opt-in, 7 modeles OSS, 0 code wire) |
| 28 | E — `web/` lint+tsc | exit 0 | ✅ **PASS** `npm run lint` (5 warnings react-refresh PRE-EXISTANTS, 0 error) + `tsc --noEmit` exit 0 |
| 29 | E — Vitest `web/` | 0 fail | ✅ **PASS** **289 passed** (24 fichiers ; 279→289, +10 : daemon +6 / `Browse.test.tsx` NEW +4) |
| 30 | E — build + size | 6/6 | ✅ **PASS** `npm run build` (built 7.07s) + `npm run size` **6/6** (vendor-ui 262.3/270, CommandPalette 9.8/20, css 122.8/130) |
| 31 | E — scan-en-strings | 0 string EN | ✅ **PASS** (`src/ is French-only, clean`) |
| 32 | E — barre tape /search | vert | ✅ **PASS** `searchBrowse_calls_daemon_search_endpoint` (GET `/api/daemon/search?q=`, URLSearchParams, bearer authFetch) |
| 33 | F — skills preflight amendees | present | ✅ **PASS** S1b graphe transitif (`Cargo.lock` + `cargo tree -d`) + S4 trace wire producteur→consommateur — dans `prompts/agent/preflight.md` + `.claude/skills/nexus-phase-preflight/SKILL.md` + `.claude/agents/nexus-phase-preflight-deep.md` |
| 34 | F — 2 docs planning | present | ✅ **PASS** `sprint73_verification.md` (ce fichier) + `sprint74_audit_plan.md` |

**Resultat : 34/34 rows PASS.** Compte canonique CI Linux 1570/1570 (0 skip) ;
Windows natif 1566/1566 (0 skip) ; worker-pump P2-A-1 vert sous nextest **et**
`cargo test` shared-process (le gate d'origine du hang).

---

## §4 Delta tests

| Phase | Type | Rust delta (Windows) | Detail |
|-------|------|----------------------|--------|
| 0 | chore (audit S72) | +0 | audit gate S72 PASS — pas de code |
| A | fix(sprint73) | +5 | guardrail 2 chemins : 2 HTTP (reject/accept) + 2 validator_loop (reject/accept) + 1 quorum (`quorum_guardrail_runs_on_agreed_text`) |
| B | fix(sprint73) | +7 | dette : poll-diagnostic +1 + sync-fs +2 + model-picker +4 ; worker-pump = 7 tests **convertis** `multi_thread` (non un ajout net) ; `factory-operator` Vitest +7 (infra NEW, hors compte Rust) |
| C | feat(search) | +5 | `feed_ingest_indexes_entry_hot`, `reindex_hot_is_idempotent`, `extract_index_fields_shared_with_rebuild`, interleave-reader, `rebuild_from_feed` repair |
| D | feat(search) | +5 | triplet provenance, M17 UNINDEXED, UNINDEXED-not-matchable, JSON endpoint, null-pour-op-non-release |
| E | feat(shell) | +0 Rust / +10 Vitest | front-only (0 `.rs` touche) ; `web/` Vitest 279→289 (daemon +6 / `Browse.test.tsx` +4) |
| F | docs(sprint73) | +0 | wrap-up (ce commit) |
| **Total A-E** | | **+22 Rust (Windows)** | + Vitest +10 + factory-operator +7 |

| Suite | Entree S73 (`087e781`) | Sortie S73 (`9472085`) | Net |
|-------|------------------------|------------------------|-----|
| Rust nextest (**canonique CI Linux**) | 1544 | **1570** | **+26** |
| Rust nextest (Windows natif) | 1544 | **1566** | **+22** |
| Vitest (`web/`) | 279 | **289** | **+10** |
| `factory-operator` Vitest | 0 (pas de runner) | **7** | **+7** (infra NEW) |
| size-limit | 6/6 | 6/6 | 0 |

**Reconciliation du compte** : la somme des deltas par phase A-E **mesures
Windows** = +22 (A 5 / B 7 / C 5 / D 5 / E 0), exit Windows 1566 vs entree 1544.
Le compte **canonique CI Linux** est **1570** (+26) : l'ecart **+4 vs Windows**
= tests `#[cfg(unix)]` (UDS peer-cred `auth.rs`, e2e unix-gates) qui ne compilent
pas sous Windows — **structurel** (apparait des Phase B : 1556 Win / 1560 Linux),
**0 skip sur les deux plateformes**. La decomposition exacte des 4 tests
`#[cfg(unix)]` est routee a l'audit S74 Track E (`nextest list` 2 plateformes).
Aucune regression (0 fail canonique, 0 skip).

---

## §5 G8 preflight bilan

| Phase | Verdict G8 | Resolution | Fichier |
|-------|-----------|------------|---------|
| A | **EXECUTE** | securite reorder + doc lot, 4 scans clean ; G1 design_review present (gate Phase A) | `sprint73_phase_a_preflight.md` |
| B | **EXECUTE** | dette pure, 4 scans clean ; D6 worker-pump = root-cause runtime flavor (multi_thread) | `sprint73_phase_b_preflight.md` |
| C | **EXECUTE** | reindex hot D1, SQLite reel 3.49.2 (pas 3.50.x kickoff), test#4 reformule single-Mutex | `sprint73_phase_c_preflight.md` |
| D | **EXECUTE** | triplet UNINDEXED D2 + defer SearchManifest D3, 0 wire (M17 local) | `sprint73_phase_d_preflight.md` |
| E | **SCOPE-CUT-CONSISTENT** | drift plan→reel : le JSON est une **enveloppe** `{results,total,took_ms}` (pas « SearchResult 7+5 » nu) ; les 4 champs provenance serialises toujours-en-`null` → Zod `.nullable()` pas `.optional()` (finding preflight load-bearing) | `sprint73_phase_e_preflight.md` |

**5/5 phases code G8. 4 EXECUTE (A,B,C,D) / 1 SCOPE-CUT-CONSISTENT (E).** Zero
DESIGN-CONFLICT (contraste S72 = 2 DESIGN-CONFLICT consecutifs ; les amendements
preflight S1b/S4 de Phase F repondent a cette lecon meta S72).

**Codex 5/5 phases** (zero exemption §4.5.6) : A 7/7 (Run 1 PARTIAL
`ResultValidator` guardrail-less → ferme Run 2 par gate `#[cfg(test)]`),
B 8/8 0 GAP, C 7/7 0 GAP, D 8/8 0 GAP, E 4/4 (gpt-5.5) 0 GAP.

---

## §6 Scope cuts compliance

Les 14 scope cuts du kickoff §7 / plan §7. Chaque review de phase a confirme
0 hit ; Codex a confirme que les refs S74+/cross-machine restent des deferrals.

| # | Item | Sprint cible | Respecte |
|---|------|--------------|----------|
| 1 | SearchManifest reseau-large (op feed + gossip + wire signe) | post-launch (D3) | OUI — DEFERE, feed-local + design note ; `PublicFeedOperation` = 4 variantes (SearchManifestPublished = commentaire forward-compat seul), 0 struct/fn/serde tag |
| 2 | `sbfb-factory search/open/fork` | S74 | OUI — absent |
| 3 | Notion projet cible distinct nexus | S74 | OUI — `repo_root` pointe nexus |
| 4 | `reseau→atelier` clone/reconstruction blob | S74 | OUI — le triplet est **retourne** (display-only), aucun fork cable |
| 5 | Templates etendus (react, pyodide) | S74 | OUI — absent |
| 6 | GPU partage cross-machine | S75 | OUI — absent |
| 7 | Quorum redundancy>1 cross-MACHINE reel | S75 | OUI — absent |
| 8 | Sharding pipeline | S76 STRETCH | OUI — absent |
| 9 | Tantivy | gate post-S75 | OUI — gele, FTS5 reste l'engine |
| 10 | @dev tree-sitter / source-only | post-Gate 1 | OUI — absent |
| 11 | Rate-limit per-client search | S74+ (re-eval Phase E) | OUI — re-evalue : endpoint loopback single-user derriere `auth_required` (residual T-SEARCH-DOS acceptable pre-launch, THREAT_MODEL §11) ; carry S74 si trafic le justifie |
| 12 | Webhook/SSE feed push | S74+ | OUI — reindex sur chemin pull/gossip existant, pas un push |
| 13 | Streaming token-par-token WAN | jamais (PO-14) | OUI — absent |
| 14 | Pagination boutons barre recherche | S74+ | OUI — champ recherche simple, pas de prev/next |

**Resultat : 14/14 scope cuts respectes.** Zero bump wire (`FEED_FORMAT_VERSION=1`,
M17 = schema SQLite local).

---

## §7 Surface nouvelle livree

| Module / fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/validator.rs` | split `validate_result_pre_guardrail` (no persist) / `validate_result_post_guardrail` (seul appelant prod de `set_task_result`) ; `ResultValidator` gate `#[cfg(test)]` |
| `crates/nexus-shell-daemon/src/http.rs` (`coordinator_submit_result`) | reorder pre → `default_output_chain().run()` → post ; sur trip, 400 + 0 persist + 0 kudos |
| `crates/nexus-shell-daemon/src/validator_loop.rs` | guardrail injecte sur le chemin gossip (aucun avant S73) |
| `crates/nexus-coordinator-rs/src/search.rs` | `upsert_feed_entry` (hot reindex, rowid=seq, INSERT OR REPLACE) + helper `extract_index_fields` partage + `SearchResult` +5 champs triplet UNINDEXED + name bridge `artifact_hash`→`archive_hash` |
| `crates/nexus-coordinator-rs/src/db.rs` | migration **M17** (DROP/recreate `search_index` + 5 col UNINDEXED) + `busy_timeout(5s)` explicite |
| `crates/nexus-shell-daemon/src/feed_sync.rs` | appel `upsert_feed_entry` apres `insert_feed_entry` Ok (meme lock scope, best-effort warn-on-fail) |
| `crates/sbfb-factory/src/provider_router.rs` + `operator_server.rs` | `last_err` surface + per-provider model (Claude pinne, Ollama/Network env-overridable) |
| `crates/sbfb-factory/src/daemon_client.rs` | `resolve_daemon` async (`spawn_blocking`, plus de `std::fs` en contexte async) |
| `crates/sbfb-factory/tests/process_cli.rs` | fixtures git self-contained (de-hardcode SHA S70) |
| `tools/factory-operator/` (vitest.config, setup.ts, *.test.ts) | infra Vitest NEW (jsdom + MockEventSource) — 7 tests |
| `web/src/api/daemon.ts` | `searchBrowse()` + `SearchResultSchema`/`SearchResponseSchema` Zod `.strict()` (enveloppe, 4 provenance `.nullable()`) |
| `web/src/pages/Browse.tsx` | barre recherche dediee + `useQuery` + rendu hits provenance + `isHttpsUrl` garde XSS |
| `.planning/research/s73_searchmanifest_index_node_design.md` (NEW) | design forme correcte SearchManifest (D3 defer) |
| `docs/security/THREAT_MODEL.md` §14/§11 + `LOOPBACK §2.1/§3/§8.1` + `HARDENING_ROADMAP §3` | guardrail AVANT persist + Operator tier formel + recadre |
| `docs/rust/PATTERNS.md` §P56 (+ §P53/§P54/§P55 lot P3) | FTS5 hot reindex + triplet UNINDEXED + nits doc corriges |
| `prompts/agent/preflight.md` + skill + agent-deep | S1b graphe transitif + S4 trace wire (P2-PREFLIGHT-*) |

---

## §8 Findings carry-over for memory (G6)

Max 5 items a persister (fusion manuelle au kickoff S74) — surfaces par une
scrutiny adversariale multi-agent du diff (Phase F) ; details + file:line dans
`sprint74_audit_plan.md` :

1. **Tripwire S74 AVANT browse-indexing prod** : `search_index` rowid est
   partage entre les upserts feed (rowid=seq) et `index_entry` browse (auto
   rowid, **test-only** aujourd'hui). Cabler le browse-indexing prod S74 **doit**
   partitionner l'espace rowid (sinon un upsert feed clobbe une ligne browse) +
   re-appliquer l'invariant `is_open_source⇒provenance_hash` (`public_feed.rs:285`)
   au chemin browse. Tripwire doc `search.rs:241-244` + PATTERNS §P56.
2. **Freshness narrative incomplete (P2 candidat)** : un op `ReleasePublished`
   indexe `description=''` (le payload n'a ni reason ni project_name) → **un
   projet publie n'est pas cherchable full-text** par son nom. Seuls les ops
   porteurs de `reason` (CuratorVouched/SourceBecameStale) le sont. L'indexation
   du nom de projet est un gap a trancher S74.
3. **Guardrail-before-persist = convention d'appelant, pas type (P2/P1 candidat)**
   : l'invariant D5 repose sur 2 appelants qui executent le guardrail entre
   `pre` et `post` ; `PendingResultPersist` ne porte aucune preuve. Seul le gate
   `#[cfg(test)]` sur `ResultValidator` empeche une re-composition guardrail-less.
   + le chemin quorum laisse le texte rejete dans `task_results` (interne, non
   recuperable via GET /result) sans statut terminal (zombie/re-dispatch). Surface
   **recuperable** correctement gatee — a trancher P2 precision-doc vs disponibilite.
4. **Phase E (P2 auditor-found, hors self-report)** : `SearchResultsView` n'a
   aucune branche `query.isError` ; un drift Rust↔Zod (`callDaemon` THROW
   `ApiProtocolError`) yield un `LoadingSkeleton` **infini** (pas une carte
   d'erreur). + scheme-guard non normalise sur les 3 ancres `repo_url`
   pre-existantes (P2-3).
5. **Process** : (a) worker-pump P2-A-1 **CLOSED sur les 2 gates** (nextest +
   `cargo test` shared-process) — le hang se manifestait sous `cargo test`
   partage, PAS nextest. (b) Docker-sur-Windows exige `libgtk-3-dev` (atk-sys),
   sinon compile-fail masque par `| tail`. (c) Phases C/D ont **defere** leur
   verif Docker Linux au wrap-up Phase F (faite : 1570/1570 couvre M17 +
   hot-reindex). (d) `nexus-phase-review-deep` + `nexus-process-supervisor`
   toujours non enregistres → fallback general-purpose + hooks backstop D17.

---

## §9 Checkpoint de cloture

- [x] Fail-fast checklist §3 : **34/34 rows PASS** (canonique CI Linux 1570/1570, 0 skip ; Windows natif 1566/1566, 0 skip ; +4 = `#[cfg(unix)]` structurel)
- [x] Phases A-F landed (A guardrail+doc, B dette, C reindex, D enrichment, E shell, F wrap-up)
- [x] **P2-A-1 worker-pump 3/3 CLOSED par fix `multi_thread` cross-platform** (vert Windows nextest + Windows `cargo test` shared-process **+** Docker Linux nextest) — **plus jamais carry**
- [x] P2-RESULT-TEXT-GUARDRAIL-ORDER ferme (guardrail AVANT persist sur HTTP + validator_loop ; surface recuperable jamais `completed` sur trip ; claims THREAT_MODEL §14 + LOOPBACK §3 corrigees)
- [x] 12 P2 audit S72 traites (Phase A doc lot 3 + Phase B dette/hardening 7 + Phase F preflight 2)
- [x] Reindex FTS5 a chaud (entree feed-distante cherchable sans reboot, idempotent O(1))
- [x] SearchResult enrichi triplet provenance (UNINDEXED, M17) — prerequis fork S74
- [x] Barre recherche shell cablee `GET /api/daemon/search` (champ Browse, FR, garde XSS)
- [x] D3 SearchManifest : defer documente + design note forme correcte present (0 code wire)
- [x] Pas de bump wire (pre-launch ; M17 = schema local ; `FEED_FORMAT_VERSION=1`)
- [x] 5/5 phases code G8 (4 EXECUTE + 1 SCOPE-CUT-CONSISTENT, 0 DESIGN-CONFLICT) ; Codex 5/5 phases (A-E)
- [x] `sprint73_verification.md` + `sprint74_audit_plan.md` ecrits (ce commit)
- [x] Skills/agents preflight amendees (P2-PREFLIGHT-TRANSITIVE-DEPTH + WIRE-CONTRACT-DEPTH, 3 fichiers) ; PATTERNS §P53/§P54/§P55 P3 lot corrige
- [x] Memory `nexus_grid_pivot.md` + `MEMORY.md` a jour (ce commit) + SPRINT_LOG row S73

**S73 CLOSED. Arc 3.5 (Factory Complete Vision) 3/6 ; S74 (atelier fork)
debloque sous reserve de l'audit gate S73** (`sprint74_audit_plan.md` route ~10
P2 candidats + 3 candidats P1 a trancher — les invariants headline tiennent).
