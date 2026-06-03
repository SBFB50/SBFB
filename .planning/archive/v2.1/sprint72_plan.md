# Sprint 72 — Plan (Factory provider routing — ProviderRouter multi-LLM)

**Ecrit** : 2026-05-31 (revise apres arbitrage PO : D2 ollama-rs 0.3.4
partout + bump worker ; front UX intentions complete in-scope, Phase E).
**Tip master** : `636b9de` (audit S71 PASS).
**Roadmap** : Sprint 2/6, v2.1 Arc 3.5 « Factory Complete Vision ».

---

## §1 Etat verifie a l'entree

| Suite | Count | Commande | Observed |
|---|---|---|---|
| Rust nextest | 1528 (CI Linux canonique) | `cargo nextest run --workspace --locked` | |
| Rust doctests | ok | `cargo test --workspace --locked --doc` | |
| cargo fmt | 0 diff | `cargo fmt --all --check` | |
| cargo clippy | 0 warnings | `cargo clippy --workspace --all-targets --locked -- -D warnings` | |
| Vitest (shell `web/`) | 279 | `(cd web && npm run test:unit)` | |
| size-limit (shell `web/`) | 6/6 | `(cd web && npm run size)` | |
| factory-operator tsc | exit 0 | `(cd tools/factory-operator && npx tsc -b --noEmit)` | |
| factory-operator eslint | exit 0 | `(cd tools/factory-operator && npx eslint .)` | |
| release build | ok | `cargo build -p nexus-shell-daemon --release` | |
| **Total** | **~1813** | | |

Note : le shell React `web/` n'est PAS touche par S72 (Vitest/size-limit/
scan-en-strings sur `web/` = N/A). Le **front Operator**
`tools/factory-operator/` EST touche (Phase E, UX intentions complete) —
package standalone **sans suite de tests** (verifie `package.json` :
scripts `build`=`tsc -b && vite build`, `lint`=`eslint .`, pas de Vitest).
La fail-fast front = `tsc -b --noEmit` + `eslint .` (+ scan-en-strings si
le script s'applique au front Operator).

---

## §2 Decisions Day 0 (gelees)

| D# | Decision | Implication code |
|---|---|---|
| D1 | Trait `ExecutionTarget` enum-dispatch → `Pin<Box<dyn Stream<StreamChunk>>>` | NEW `crates/sbfb-factory/src/provider_router.rs` ; `lib.rs` (`mod provider_router`) ; `operator_server.rs:898` (appel `ExecutionTarget::run`) ; `llm_bridge.rs:95` (`spawn_claude_stream` reste, bras Claude) |
| D2 | `ollama-rs 0.3.4` partout (Factory dep + bump worker 0.2.6→0.3.4) | `crates/sbfb-factory/Cargo.toml` (+`ollama-rs 0.3.4 {stream}`) ; `provider_router.rs` bras `Ollama` (`generate_stream`) ; `crates/nexus-worker-core/Cargo.toml` (bump) + `src/llm/ollama.rs:239-254` (rename `GenerationOptions`→`ModelOptions` + import) ; §6.6 bump pin documente ; **4 tests quorum S71 re-verts (R7)** |
| D3 | NetworkProvider submit→poll → un seul `Done` (async, PO-14) | `provider_router.rs` bras `Network` (client `reqwest` submit + poll-loop `async_stream`) ; consomme `POST /api/v1/tasks/submit` + `GET /api/v1/tasks/{id}` (inchanges) |
| D4 | Cabler `provider` : `ChatSendRequest` → `ChatSession` → `handle_chat_stream` + UI selectable | `operator_server.rs:52` (`ChatSession +provider`), `:758` (persist au send), `:822-898` (lit provider, dispatch) ; gate SENSITIVE_ACTIONS reste AVANT dispatch ; `tools/factory-operator/` (UX intentions complete, Phase E) |
| D5 | 3 axes orthogonaux : `ExecutionTarget` (run) vs `Provider` prompt-adapt (process.rs) vs `LlmBackend` (worker) | `provider_router.rs` (nom `ExecutionTarget`) ; `docs/rust/PATTERNS.md §P55` (NEW) ; pas de modif `process.rs` |

---

## §3 Graphe de dependances inter-phases

```
Phase A (catalogue menace P2-H-1)  ──┐
                                     ├─→ Phase C (bump ollama-rs worker + ExecutionTarget + Claude + Ollama)
Phase B (dette : P2-F-3 3/3 + carries)─┘        │
                                                 ↓
                                          Phase D (Network + cablage backend provider)
                                                 ↓
                                          Phase E (front UX intentions complete)
                                                 ↓
                                          Phase F (wrap-up)
```

- **Phase A avant C/D** : P2-H-1 (catalogue menace Operator) doit etre
  ferme AVANT d'etendre la surface SSE (audit S71 trigger « avant
  extension surface Operator »). Dependance de gate, pas technique.
- **Phase B independante de A** (dette, ordonnee B apres A pour le commit
  lineaire) — reservee Regle 1 sprint pair, non-negociable.
- **Phase C : la migration worker (bloc 1) vient TOT** (R7) — bumper
  `ollama-rs` avant d'ecrire le bras Ollama (bloc 2) garantit une seule
  version cross-crate et re-verifie le quorum determinisme S71 d'abord.
  Phase D depend de l'enum `ExecutionTarget` + `ProviderStream` (C).
- **Phase D avant E** : le front (E) selectionne un `provider` que le
  backend (D) doit savoir router. **Le backend D lande INDEPENDAMMENT** —
  si E deborde, le routing backend reste livre (R4 mitigation).
- **Phase F apres tout** : wrap-up mesure le sprint complet.

---

## §4 Phase A — Catalogue menace Operator (P2-H-1) + reservation surface

### §4.1 Scope

Fermer **P2-H-1** (seul P2 de l'audit S71) AVANT d'etendre la surface
SSE. L'audit S71 a posé le trigger : « avant toute extension de la
surface Operator ». S72 (ProviderRouter) touche le chat SSE — c'est
l'extension. On documente la surface Operator `:3001` (write
`/api/artifacts/draft` + spawn `bypassPermissions` chat) dans les deux
catalogues menace canoniques (la defense EST deja en place+testee S71,
seul le catalogue accuse le retard). On anticipe que le dispatch reseau
S72 (NetworkProvider → daemon loopback) reste dans la frontiere loopback
durcie. Phase **docs/threat** — peu ou pas de code Rust.

### §4.2 Livrables

| Fichier | Role |
|---|---|
| `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` | Entree serveur Operator `:3001` : endpoint(s), trust tier, capacite write (`/api/artifacts/draft`) + spawn (`/api/chat/{id}/stream` bypassPermissions, `/api/terminal/ws`). Cross-ref `PATTERNS §P35`. Noter le NetworkProvider S72 (client du daemon loopback, pas surface entrante nouvelle). |
| `docs/security/THREAT_MODEL.md` | Entree menace Operator : CSRF/DNS-rebinding (mitige token X-SBFB-Token + Host + CORS loopback, S71 G7) + spawn-agent autonome (mitige gate SENSITIVE_ACTIONS, S71 G2). Ref `PATTERNS §P35`. |

### §4.3 Tests plan

Phase docs/threat — pas de nouveau test code (defense deja testee S71).
Verification = grep de presence documentaire (fail-fast §10).

### §4.4 Critere d'acceptation

```bash
grep -i "operator" docs/security/THREAT_MODEL.md            # >= 1 hit
grep -iE "operator|3001" docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md  # >= 1 hit
grep -i "P35" docs/security/THREAT_MODEL.md docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md
```

Condition binaire : les deux catalogues referencent la surface Operator ;
P2-H-1 exit (audit S71 Carry-Over) satisfaite.

### §4.5 Commit cible

Titre : `docs(security): Sprint 72 Phase A — catalogue Operator surface (P2-H-1)`
Body 9 sections (cf. `.claude/templates/commit_body_phase.txt`).
Codex verification obligatoire (docs-only n'exempte pas, §4.5.6 README).
Carry closure : P2-H-1 ferme.

---

## §5 Phase B — Dette pair (Regle 1) : P2-F-3 3/3 + carries compute

### §5.1 Scope

Phase dette **reservee** (sprint pair, §6.2.1 Regle 1) —
**non-negociable, non-convertible en feature**. Items :

1. **P2-F-3 (3/3 MANDATORY)** : couplage wrappers `.claude/agents/*.md`
   → `prompts/agent/*.md` (4 refs : `phase-review.md`, `phase-auditor.md`,
   `preflight.md`, `audit-gate-checks.md`). **Verifier d'abord l'etat
   reel** (§G9) : si un garde-fou existe deja, documenter clos. Sinon,
   poser un check mecanique borne.
2. **P2-A-2** : E2E cross-process S71 asserte `results.len()==1` mais pas
   la signature. Ajouter `ResultEntry::verify_signature()`.
3. **P3-A-3 / P3-B-1 / P3-B-2** : cosmetiques — fix si peu couteux, sinon
   re-doc.

### §5.2 Livrables

| Fichier | Role |
|---|---|
| `prompts/agent/` + test ou lint | P2-F-3 : check mecanique que chaque `prompts/agent/<kind>.md` reference par un wrapper `.claude/agents/*.md` existe. Options : (a) test Rust dans `sbfb-factory` (process.rs sait resoudre `prompt_filename`) iterant les kinds ; (b) extension hook lint-planning ; (c) contrat documente `AGENT_SYSTEM.md` + test. Trancher au preflight selon l'etat reel. |
| `docs/agent/AGENT_SYSTEM.md` | P2-F-3 : si check mecanique, documenter le contrat de stabilite prompt↔wrapper. |
| `crates/nexus-shell-daemon/src/dispatch_loop.rs` (ou test E2E) | P2-A-2 : ajouter `verify_signature()` sur le `ResultEntry` lu dans `dispatched_task_is_claimed_and_executed_by_worker_engine`. |
| `docs/rust/PATTERNS.md` | P3-B-2 : confirmer note `sha256` misnomer (§P53) ; P3-B-1 `as i32` re-doc si non fixe. |

### §5.3 Tests plan

1. `prompt_wrappers_reference_existing_files` (ou nom equivalent) —
   verifie que pour chaque kind reference par un wrapper, le fichier
   `prompts/agent/<kind>.md` existe (P2-F-3 exit binaire). **Si deja
   resolu, documenter clos sans ce test.**
2. Assertion `result.verify_signature()` ajoutee dans le E2E existant
   `dispatched_task_is_claimed_and_executed_by_worker_engine` (P2-A-2).

### §5.4 Critere d'acceptation

```bash
cargo nextest run -p sbfb-factory --locked        # P2-F-3 check vert (si test)
cargo nextest run -p nexus-shell-daemon --locked  # E2E + assertion signature vert
```

Condition binaire : P2-F-3 ferme (check en place OU clos documente,
**plus jamais carry**) ; P2-A-2 assertion signature presente ; P3
tranches (fix ou re-doc).

### §5.5 Commit cible

Titre : `fix(planning): Sprint 72 Phase B — close P2-F-3 (prompt coupling) + carries`
(ou `test(...)` selon contenu reel). Body 9 sections. Carry closure :
P2-F-3 (3/3 → CLOSED), P2-A-2 (CLOSED), P3 tranches.

---

## §6 Phase C — Bump `ollama-rs` worker (migration quorum) + `ExecutionTarget` + Claude + Ollama

### §6.1 Scope

Deux blocs ordonnes — la **migration worker vient en premier (R7)** :

**Bloc 1 — Migration `ollama-rs` 0.2.6→0.3.4 (D2)** : bumper le pin
workspace, ajouter `ollama-rs 0.3.4 {features=["stream"]}` en dep directe
de `sbfb-factory`, et migrer `nexus-worker-core/src/llm/ollama.rs` (rename
`GenerationOptions`→`ModelOptions` + import `ollama_rs::models::
ModelOptions`). L'API builders `.temperature()`/`.seed(i32)`/`.options()`/
`.system()`/`generate()` survit (context7 0.3.4 verifie) — migration
mecanique. **Preflight Phase C re-verifie l'API seed/options 0.3.4 (S1b)
avant la 1ere ligne** ; si l'API a change de maniere non anticipee →
DESIGN-CONFLICT remonte.

**Bloc 2 — `ExecutionTarget` + Claude + Ollama (D1/D2)** : NEW
`provider_router.rs` : enum `ExecutionTarget` + type-alias `ProviderStream`
(`Pin<Box<dyn Stream<Item=StreamChunk> + Send>>`) + `from_provider()` +
`run()`. Bras `Claude` = `spawn_claude_stream` (INCHANGE, idle-timeout D6
S71). Bras `Ollama` = `ollama-rs 0.3.4` `generate_stream(GenerationRequest)`
→ chaque `GenerationResponse.response` → `StreamChunk::Delta`, le chunk
final → `StreamChunk::Done` ; idle-timeout + diagnostic Ollama injoignable.
(Bras `Network` ajoute Phase D — vide/`todo` ici.)

### §6.2 Livrables

| Fichier | Role |
|---|---|
| `crates/nexus-worker-core/Cargo.toml` | Bump `ollama-rs` 0.2.6→0.3.4 (decision PO §6.6). |
| `Cargo.toml` workspace (si pin centralise) | Bump pin `ollama-rs` 0.3.4. |
| `crates/nexus-worker-core/src/llm/ollama.rs:239-254` | Rename `GenerationOptions`→`ModelOptions` + import `ollama_rs::models::ModelOptions`. `deterministic_options` inchange fonctionnellement. |
| `crates/sbfb-factory/Cargo.toml` | +`ollama-rs = { version = "0.3.4", features = ["stream"] }`. |
| `crates/sbfb-factory/src/provider_router.rs` | NEW. `enum ExecutionTarget { Claude{model}, Ollama{model}, Network{project_id,model} }`, `type ProviderStream`, `from_provider`, `run`. Bras Claude = `spawn_claude_stream` boxe ; bras Ollama = `ollama-rs generate_stream` → StreamChunk. |
| `crates/sbfb-factory/src/lib.rs` | `mod provider_router;`. |

### §6.3 Tests plan

1. **4 tests quorum S71 re-verts (R7, critere binaire migration)** :
   `verifiable_task_uses_greedy_seed`, `two_honest_workers_same_hash`,
   `quorum_accepts_deterministic_redundancy`,
   `quorum_rejects_nondeterministic_divergence` — restent verts post-bump.
   Plus `deterministic_options_wire_temperature_and_seed` (ollama.rs:398).
2. `execution_target_from_provider_parses_closed_set` — `"claude"`→Claude,
   `"ollama"`/`"local"`→Ollama, `"network"`→Network, defaut/inconnu→Claude.
3. `claude_target_is_behaviorally_unchanged` — bras Claude = meme sequence
   `StreamChunk` que `spawn_claude_stream` direct (via `SBFB_CLAUDE_BIN`
   stub, comme tests llm_bridge S71).
4. `ollama_stream_maps_to_chunks` — un Ollama stub/feature-gate produit
   `Delta` ... `Done` depuis `generate_stream`. Skip propre si Ollama
   absent (gate disponibilite, comme B-3 S71).
5. `ollama_unreachable_yields_diagnostic` — Ollama injoignable →
   `StreamChunk::Error` diagnostic.

### §6.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-worker-core --locked   # 4 tests quorum + deterministic_options verts (R7)
cargo nextest run -p sbfb-factory --locked
cargo clippy --workspace --all-targets --locked -- -D warnings   # bump ne casse rien
```

Condition binaire : ollama-rs aligne 0.3.4 partout ; **4 tests quorum S71
verts** ; `ExecutionTarget::run` dispatche Claude + Ollama ; bras Claude
byte-equivalent S71 ; mapping Ollama `generate_stream` → StreamChunk
prouve (stub/skip).

### §6.5 Commit cible

Titre : `feat(factory): Sprint 72 Phase C — align ollama-rs 0.3.4 + ExecutionTarget dispatch + Ollama`
Body 9 sections. G8 traceability (preflight C — phase introduit un bump
dep + nouveau composant : format complet S1a/S1b recommande, README §6.9).
Pre-launch protocol (bump pin, pas de wire). Codex verification. Carry
closure (R7 quorum re-verifie).

---

## §7 Phase D — NetworkProvider submit→poll + cablage backend `provider`

### §7.1 Scope

Bras `Network` (D3) : client `reqwest` qui soumet une `TaskSubmission`
(prompt assemble + model) via `POST /api/v1/tasks/submit` au daemon
loopback, recupere `task_id`, poll `GET /api/v1/tasks/{task_id}`
(`async_stream::stream!` + `tokio::time::interval` 2s) jusqu'a
`completed`/`rejected`. Emet un progress (label statut) par tick, puis
**un seul** `StreamChunk::Done` (PO-14, jamais Delta token-par-token).
Timeout global. Cabler `provider` **backend** (D4) : `ChatSession
+provider`, persiste au send (comme `model` S71), lu au stream, dispatch
`ExecutionTarget`. Gate SENSITIVE_ACTIONS reste AVANT dispatch (inchange).
`PATTERNS §P55` (3 axes) ecrit ici. **Le backend Phase D lande
INDEPENDAMMENT du front Phase E.**

### §7.2 Livrables

| Fichier | Role |
|---|---|
| `crates/sbfb-factory/src/provider_router.rs` | Bras `Network` : client submit + poll-loop `async_stream` → un `Done`. Endpoint daemon overridable (`SBFB_DAEMON_ENDPOINT`, defaut loopback). Auth token daemon si requis (R3, verifie preflight). |
| `crates/sbfb-factory/src/operator_server.rs` | `:52` `ChatSession { ..., provider: String }` ; `:758` persister `req.provider` au send (symetrie `model`) ; `:822-898` `handle_chat_stream` lit `session.provider`, construit `ExecutionTarget`, `.run()`. Gate SENSITIVE_ACTIONS conserve avant dispatch. |
| `docs/rust/PATTERNS.md` | §P55 NEW : 3 axes orthogonaux (`Provider` prompt-adapt / `LlmBackend` runtime / `ExecutionTarget` chat routing). D5. |

### §7.3 Tests plan

1. `chat_session_persists_provider` — un `chat-send` avec
   `provider:"ollama"` persiste le provider dans la session.
2. `chat_stream_routes_by_session_provider` — `session.provider=="ollama"`
   dispatche le bras Ollama (pas Claude) ; `"claude"` (defaut) → Claude.
3. `network_provider_submit_poll_yields_single_done` — un daemon stub qui
   accepte submit puis renvoie `completed` au 2e poll produit exactement
   un `StreamChunk::Done` (pas de Delta token). PO-14.
4. `network_provider_poll_timeout` — daemon stub qui ne complete jamais →
   `StreamChunk::Error` (timeout global).
5. `sensitive_action_gated_regardless_of_provider` — action sensible
   (`commit`/`push`/`shell`/`PASS`) → `requires_gate` AVANT dispatch quel
   que soit `provider`. Securite S71 D3 preservee.

### §7.4 Critere d'acceptation

```bash
cargo nextest run -p sbfb-factory --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Condition binaire : `handle_chat_stream` route selon `session.provider` ;
gate SENSITIVE_ACTIONS avant dispatch (tous targets) ; network
submit→poll→un Done prouve ; provider=claude inchange.

### §7.5 Commit cible

Titre : `feat(factory): Sprint 72 Phase D — NetworkProvider submit-poll + wire provider routing`
Body 9 sections. Pre-launch protocol (NetworkProvider consomme
`TaskSubmission`/`TaskStatus` inchanges — pas de bump wire). §P55
documente. Codex verification.

---

## §8 Phase E — Front UX intentions COMPLETE (decision PO)

### §8.1 Scope

Implementation COMPLETE du selecteur d'intentions dans
`tools/factory-operator/` : CTA « Executer sur Claude / en local / sur le
reseau » (intentions, **jamais** jargon `provider/kind` — UX obligatoire
CLAUDE.md) mappant vers `ChatSendRequest.provider` ; etats reseau riches
(« en cours sur le reseau » / progress pendant le poll NetworkProvider).
Respecte la frontiere `tools/factory-ui/src/readonly` (socle S70). Depend
de Phase D (backend provider cable).

### §8.2 Livrables

| Fichier | Role |
|---|---|
| `tools/factory-operator/src/` (composant chat/selecteur) | Selecteur d'intentions (Radix Select deja en dep) — 3 intentions mappant vers `provider` claude/ollama(local)/network ; etats reseau (loading « en cours sur le reseau »). |
| `tools/factory-operator/src/` (i18n) | Cles i18next FR/EN pour les 3 intentions + etats (react-i18next deja en place). Strings utilisateur FR. |
| `tools/factory-operator/src/api-client` (ou equivalent) | Transmettre `provider` selectionne dans le `POST /chat/{id}/send`. |

### §8.3 Tests plan

Le front Operator n'a **pas de test runner** (package.json : pas de
Vitest). Verification = `tsc -b --noEmit` (0 erreur type) + `eslint .`
(0 erreur) + revue manuelle de la transmission `provider`. Si le PO veut
des tests front, ce serait un ajout d'infra (Vitest) hors quick win —
documenter en carry si demande. Pas de nouveau test Rust.

### §8.4 Critere d'acceptation

```bash
(cd tools/factory-operator && npx tsc -b --noEmit)   # exit 0
(cd tools/factory-operator && npx eslint .)          # exit 0
# strings FR si scan-en-strings applicable au front Operator :
bash web/scripts/scan-en-strings.sh   # (ou equivalent ciblant factory-operator)
```

Condition binaire : le front compile + lint propres ; l'intention
selectionnee est transmise au backend (`provider`) ; strings utilisateur
en francais.

### §8.5 Commit cible

Titre : `feat(factory-operator): Sprint 72 Phase E — UX intentions execution (Claude / local / reseau)`
Body 9 sections. Scope cuts respectes (packaging/onboarding reste S74).
Codex verification. Note : front-only, suites lourdes Rust N/A (exemption
documentee — seul le front est touche).

---

## §9 Phase F — Wrap-up

### §9.1 Scope

Ecrire les deux livrables obligatoires de cloture + mises a jour.

### §9.2 Livrables

| Fichier | Role |
|---|---|
| `.planning/active/sprint72_verification.md` | Self-report fail-fast rempli (9 sections canoniques §2.3). |
| `.planning/active/sprint73_audit_plan.md` | Feuille de route audit S73 (tracks A-I, route les P2/P3 des phase reviews §4.4 README). |
| `docs/rust/PATTERNS.md` | §P55 (3 axes) si pas deja Phase D ; tech debt S72. |
| `docs/shell/PATTERNS.md` | Si nouveau pattern (NetworkProvider client). |
| `CLAUDE.md` | Etat actuel S72 DONE. |
| `docs/claude/SPRINT_LOG.md` | Row S72. |
| memory `nexus_grid_pivot.md` + `MEMORY.md` | Tip + compteurs + carries S73. |

### §9.3 Tests plan

Phase docs-only — +0 tests. Re-mesure des compteurs.

### §9.4 Critere d'acceptation

100% fail-fast verts (§10), 2 docs planning ecrits, PATTERNS a jour,
memory a jour, SPRINT_LOG row.

### §9.5 Commit cible

Titre : `docs(sprint72): verification + audit plan for Sprint 73`
(titre SANS « Sprint N Phase X » → cloture docs, non gate phase-impl).
Body : verification 100%, audit_plan 9 tracks, PATTERNS, memory, SPRINT_LOG.

---

## §10 Delta tests estime

| Phase | Rust | Front | Detail |
|---|---|---|---|
| A | +0 | +0 | docs/threat-only (defense deja testee S71) |
| B | +1-2 | +0 | P2-F-3 check (1) + P2-A-2 assertion (sur test existant) ; P3 cosmetique |
| C | +4-5 | +0 | from_provider parse + Claude equivalence + Ollama stream map + Ollama diagnostic ; **4 tests quorum S71 re-verts (R7, pas un ajout — non-regression)** |
| D | +5 | +0 | session persist provider + route-by-provider + network submit-poll single-Done + network timeout + sensitive-gated-all-providers |
| E | +0 | tsc+eslint | front Operator UX intentions (pas de test runner — verif tsc/eslint) |
| F | +0 | +0 | wrap-up docs |
| **Total** | **+10-12** | tsc/eslint front | |
| **Sortie estimee** | **~1538-1540 Rust** | **279 Vitest (web inchange)** | **~1825** |

Estimation indicative (pas un plafond, pas de LOC §6.7). Les 4 tests
quorum S71 ne sont PAS un ajout — ils sont re-verts comme critere de
non-regression de la migration (R7).

---

## §11 Scope cuts (reprise kickoff §7)

1. Onboarding/packaging atelier (launcher, doc install) → **S74** (l'UX
   intentions complete EST in-scope S72 Phase E ; seul le packaging reste).
2. Pont feed-distant → reindex FTS5 → **S73**.
3. Enrichissement `SearchResult` → **S73**.
4. Barre recherche shell cablee → **S73**.
5. Decision SearchManifest → **S73** (selon audit S72).
6. `sbfb-factory search/open/fork` → **S74**.
7. Notion projet cible distinct nexus → **S74**.
8. Templates etendus (react, pyodide) → **S74**.
9. GPU partage cross-machine → **S75**.
10. Quorum redundancy>1 cross-MACHINE reel → **S75**.
11. Sharding pipeline → **S76 STRETCH**.
12. Streaming token-par-token worker reseau distant → **jamais (PO-14)**.
13. logprobs/watermark verification → **V2 compute**.
14. Dashboard kudos per-task → **S75**.
15. Extraction crate `ollama-client` partage → **CADUC** (ollama-rs adopte
    partout, la lib EST le crate partage — plus d'extraction a prevoir).
16. Routage multi-cloud generaliste (OpenAI/Gemini/...) → **hors roadmap**.

---

## §12 Risks (reprise kickoff §9)

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Bras Claude change le comportement S71 (regression gate/timeout) | Moyen | Eleve | `spawn_claude_stream` reste corps du bras Claude, inchange ; test d'equivalence (Phase C) ; gate SENSITIVE_ACTIONS avant dispatch (Phase D test). |
| R2 | OllamaProvider non testable sans Ollama reel | Moyen | Moyen | Test mapping `generate_stream`→StreamChunk sur stub/feature gate ; E2E reel gate disponibilite (skip propre). |
| R3 | NetworkProvider exige token daemon S16 que l'Operator n'a pas | Moyen | Moyen | Preflight Phase D verifie l'auth de `/api/v1/tasks/submit` ; reutiliser le token daemon loopback si requis (pattern S71 C). |
| R4 | Scope creep front UX complet (override PO) | Eleve | Eleve | **Accepte PO** ; mitige en phasant le front (Phase E) separement du backend (Phase D) — backend lande independamment. Vrais scope cuts stricts (recherche S73, fork S74, packaging S74). |
| R5 | UX intentions incompatible dispatch async (fausse promesse live) | Moyen | Moyen | Etat « en cours sur le reseau » pendant le poll, jamais de Delta token WAN (PO-14, fail-fast #20). |
| R6 | P2-F-3 deja resolu OU plus gros que prevu | Faible | Faible | Phase B verifie l'etat reel d'abord (§G9) ; exit binaire ; check mecanique borne. |
| R7 | Bump ollama-rs 0.2.6→0.3.4 casse le determinisme greedy-seed du quorum worker | Moyen | Eleve | Migration mecanique (rename `GenerationOptions`→`ModelOptions`, API survit — context7 0.3.4) ; faite TOT (Phase C bloc 1) ; **4 tests quorum S71 = critere binaire** ; preflight Phase C re-verifie API seed 0.3.4 (S1b) ; DESIGN-CONFLICT si API non anticipee. |

---

## §13 Checkpoint de cloture

- [ ] Fail-fast checklist §10 : 100% rows applicables verts
- [ ] Phases A-F landed (A docs/threat + B fix dette + C feat migration+dispatch + D feat backend + E feat front + F docs)
- [ ] P2-H-1 ferme (Phase A) — catalogues menace referencent Operator
- [ ] P2-F-3 ferme (Phase B, 3/3, exit binaire) — **plus jamais carry**
- [ ] P2-A-2 assertion signature ajoutee (Phase B)
- [ ] ollama-rs aligne 0.3.4 partout + **4 tests quorum S71 verts (R7)**
- [ ] `ExecutionTarget` + 3 providers (Claude inchange / Ollama / Network)
- [ ] `provider` cable de bout en bout + UI intentions complete, gate SENSITIVE_ACTIONS preserve
- [ ] Front Operator tsc+eslint propres, strings utilisateur FR
- [ ] Pas de bump wire (PO-14, pre-launch) ; bump pin ollama-rs documente §6.6
- [ ] `sprint72_verification.md` + `sprint73_audit_plan.md` ecrits
- [ ] `docs/rust/PATTERNS.md §P55` (3 axes) a jour
- [ ] Memory `nexus_grid_pivot.md` + `MEMORY.md` a jour + SPRINT_LOG row S72
- [ ] G8 preflight phases code (A docs allege ; C migration = format complet S1b)
- [ ] Codex verification chaque phase (zero exemption §4.5.6)
