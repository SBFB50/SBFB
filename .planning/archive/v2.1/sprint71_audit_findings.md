# Sprint 71 Audit Findings

Date: 2026-05-31
Auditor: Claude Opus 4.8 (session fraiche S72, Phase 0 Cas A, fallback portable
de `nexus-audit-gate` via `prompts/agent/audit-gate-checks.md`)
Sprint: 71
Diff: `201b24d..0b4e7f3` (38 commits, 105 fichiers, +13779/-1011)
Verdict: **PASS**

---

## 0. Perimetre et confirmation d'ingestion

Le sprint_start_sha est `201b24d` (S70 close), confirme depuis
`sprint71_kickoff.md §2.1` (« S70 clos au commit `201b24d` ») et §3 (le bloc
off-sprint `201b24d..d5ddb95` est explicitement DANS le perimetre de
l'audit-absorb). `201b24d` est ancetre direct lineaire de HEAD (`git merge-base
201b24d HEAD == 201b24d`). L'audit couvre donc `201b24d..0b4e7f3` au minimum,
ce qui INCLUT le bloc off-sprint (~14 commits `feat/fix(factory)` +
`docs(community)`) ET les phases 0 + A-D + chores E.

Note : 21 commits ahead d'origin/master mais 38 commits dans le perimetre
d'audit — l'ecart vient de quelques commits off-sprint deja passes origin
avant la divergence. **Working tree CLEAN au demarrage.** Rien n'est pousse
(pre-launch §2.3 actif : reconciliation locale libre, canonical editable, pas
de bump wire).

Le diff complet a ete ingere fichier par fichier (operator_server.rs,
auth.rs, llm_bridge.rs, dispatch_loop.rs, validator.rs, ollama.rs, llm/mod.rs,
runtime.rs (extraits), build_executor.rs, process.rs, sprint_history.rs,
terminal.rs (extraits), task.rs, dispatcher.rs, tests/operator_server.rs,
vite.config.ts). La reconciliation du bloc off-sprint est re-confirmee via
`sprint70_audit_findings.md` (audit-absorb, verdict CONDITIONAL PASS) +
`sprint71_offsprint_retro_review.md` (verdict RECONCILED).

---

## Track A — Suites

- Rust nextest : **1532 tests, 0 skipped, 0 fail apres re-run** (plan : 1528 /
  0 skip — l'ecart +4 est platform/`#[cfg]`-gate, Linux+GTK build le launcher
  vs Windows ; voir ci-dessous). 1 flake de timing isole (non-regression).
- Rust fmt : PASS (exit 0)
- Rust doctests : PASS (0 fail)
- Vitest : **N/A** — `web/` (shell React, suite Vitest/size-limit/
  scan-en-strings) non touche du sprint (`git diff --name-only 201b24d..HEAD --
  web/` = vide). Compte inchange 279. Conforme au plan (« front non touche »).
- size-limit / scan-en-strings : N/A (`web/` non touche)
- **Front Operator (`tools/factory-operator/`, TOUCHE : AgentChat.tsx,
  SprintHistory.tsx, vite.config.ts)** : verifie en supplement (pas dans la
  fail-fast §1 standard car package distinct sans suite Vitest). `tsc -b
  --noEmit` exit 0 (0 erreur de type) ; `eslint .` exit 0 (0 erreur, 3 warnings
  `react-refresh/only-export-components` sur les primitives shadcn vendorisees
  badge/button/tabs — pre-existantes, hors fichiers S71).

**Environnement d'audit** : suites rejouees via **Docker image `sbfb-ci`**
(rust 1.95, nextest 0.9.133), conformement a `feedback_wsl_before_push` +
`feedback_dual_platform`. Le poste Windows natif n'a PAS ete utilise comme
source de verite (les E2E worker-pump `dispatched_task_is_claimed_...` + miroir
S4 hang sur Windows natif — carry P2-A-1, artefact d'environnement, PAS une
regression : non classe).

**Limite d'audit documentee (compilation)** : la 1ere passe Docker (image
bare) a echoue a COMPILER le workspace complet sur `atk-sys` — la crate
`nexus-launcher` tire `tray-icon`/`muda` avec features GTK sur Linux, dont les
libs systeme (`libgtk-3-dev`, `libatk1.0-dev`, ...) sont absentes de l'image
CI bare. C'est une limite d'environnement, PAS un defaut de code (le pipeline
Woodpecker officiel build avec ces deps). La passe authoritaire a installe les
deps GTK puis rejoue `cargo nextest run --workspace --locked`. La crate
`nexus-launcher` n'est touchee par aucune ligne S71 (hors perimetre code).

**Flake de timing investigue (NON-regression)** : le 1er full-run a vu
**1 fail** : `nexus-worker::e2e::start_headless_boots_and_shuts_down_on_signal`
(`crates/nexus-worker/tests/e2e.rs:399`, « headless start should exit cleanly
after SIGINT; status=ExitStatus(unix_wait_status(2)) »). Diagnostic :
- Le test spawn le binaire worker, dort 800 ms, envoie `SIGINT` au groupe de
  process, attend l'exit propre. C'est un E2E **signal/timing** : sous charge
  parallele lourde (~30 binaires de test concurrents saturant le CPU du conteneur),
  les 800 ms de boot peuvent ne pas suffire a installer le handler `ctrl_c`, et le
  handler SIGINT par defaut tue le process (exit 2) avant le chemin gracieux.
- **Re-execute en isolation 3x (`--test-threads=1`) : 3/3 PASS** (~1.8 s chacun).
- **`crates/nexus-worker/` est totalement HORS perimetre S71** (`git diff
  201b24d..HEAD -- crates/nexus-worker/` = vide ; `e2e.rs` derniere touche
  Sprint 54 `1d010b0`). Le code teste n'a PAS change ce sprint.
- Conclusion : flake de timing sous charge parallele, pas une regression S71.
  Non classe P0/P1 (meme logique que le hang worker-pump Windows P2-A-1 :
  artefact d'environnement, pas un bug code). Un re-run `--no-fail-fast` ne
  produit aucun autre fail.

**Findings** : aucun bloquant. 0 regression. 1 flake timing en code intouche,
prouve non-deterministe (3/3 PASS isole). Candidat de robustesse pour le carry
(augmenter le delai de boot ou attendre un READY au lieu d'un `sleep(800ms)`) —
mais hors scope S71 (code S54).

---

## Track B — Security

Surface securite la plus dense du sprint (bloc Factory Phase C + reconciliation
Phase D). Scan complet du diff `201b24d..HEAD` sur `*.rs *.ts *.tsx` +
revue manuelle de chaque endpoint write/spawn.

Patterns scannes (resultats negatifs exhaustifs) :
- **Secrets hardcodes** : 0. Les 2 seuls litteraux token-like (`TEST_TOKEN`
  dans `tests/operator_server.rs:9` et `http.rs:2180`) sont des fixtures de
  test injectees via `SBFB_AUTH_TOKEN` — pas de vrai secret. Scan
  `AKIA|ghp_|pat_|password|secret=|api_key|BEGIN PRIVATE` = 0 hit en code
  produit.
- **`unsafe`** : 2 occurrences, toutes en code de test (`auth.rs:308,311`
  `std::env::set_var/remove_var`, requis par l'API env unsafe de l'edition
  2024, isole par-process par nextest, commente). 0 `unsafe` en code produit.
- **`unwrap()`/`expect()`** : tous en code de test, ou sur `Regex::new(const)`
  (regex constante, infaillible), ou `Mutex::lock().unwrap()` (mutex empoisonne
  = bug, convention standard). Aucun `unwrap` produit sur entree non fiable.
- **`Command::new`** : `claude_exe` (spawn agent, garde par auth + gate
  SENSITIVE_ACTIONS), `git` (process.rs / sprint_history.rs, avec
  `--end-of-options` + validation rev). Aucun `sh -c`, aucune interpolation
  shell, aucun child_process JS.
- **`innerHTML`/`dangerouslySetInnerHTML`/`eval`/`allow-same-origin`** : 0 hit
  ajoute.

Verifications positives (conformes au modele de menace, alignees T0-T5) :
- **G2 SSE gate (ex-P0)** : `handle_chat_stream` (`operator_server.rs:858-879`)
  applique le filtre `SENSITIVE_ACTIONS` AVANT `spawn_claude_stream`. Un
  dernier message user sensible (`shell`/`commit`/`push`/`PASS`) renvoie
  `requires_gate` via `sse_gate(...)` et ne spawn JAMAIS l'agent
  `bypassPermissions`. Identique a `handle_chat_message`/`handle_chat_send`.
  Aucun autre chemin ne spawn `bypassPermissions` non garde (seul
  `spawn_claude_stream` le porte, atteint uniquement apres le gate). Teste :
  `sse_gates_sensitive_action` (asserte que `--permission-mode` ne fuit PAS).
- **G7 auth (ex-P1)** : middleware `auth::auth_required` applique a TOUTES les
  routes (`operator_server.rs:152-155`). Token `X-SBFB-Token` compare en
  `constant_time_eq` (`auth.rs:215-224`), Host guard loopback
  (`is_loopback_host`), Origin guard loopback (si present). Token 256-bit
  OsRng, ecriture atomique tmp+rename, `0600` Unix. CORS pinne via predicate
  `is_loopback_origin` (0 `allow_origin(Any)`). Teste : `server_rejects_missing
  _token` (401), `server_rejects_foreign_host` (403), `cors_restricts_origin`
  (403), `token_request_succeeds` (200). Le proxy Vite injecte le token
  server-to-server (`vite.config.ts`) — le navigateur ne lit jamais `~/.sbfb`.
- **G9 modele (ex-P1)** : `default_model() == "claude-opus-4-8[1m]"`
  (`operator_server.rs:294`), 0 hit `"sonnet"`. `req.model` passthrough avec
  defaut frozen. Teste : `chat_stream_uses_opus_model` (asserte opus-4-8,
  rejette `--model sonnet`).
- **G12 spawn (ex-P1)** : timeout d'inactivite (`llm_bridge.rs`) avec
  `start_kill` + `wait` (pas de zombie) + `kill_on_drop` filet de securite,
  diagnostic clair si binaire introuvable. Teste : `spawn_times_out`,
  `missing_claude_diagnostic`.
- **Phase D — git option injection (ex-P1 du retro-Codex)** : `is_safe_git_rev`
  (`operator_server.rs:219-223`) rejette tout rev a `-` initial / whitespace /
  control. Defense en profondeur : `--end-of-options` sur tous les `git
  log/diff` (`sprint_history.rs:942,947`). Teste : `operator_commit_diff_
  rejects_option_injection` + `operator_audit_rejects_option_injection`
  (asserte 400 ET qu'aucun fichier n'est ecrit).
- **Phase D — drive-prefix traversal (ex-P2)** : `handle_terminal_session_
  content` (`operator_server.rs:964-1003`) rejette `..`, `/`, `\`, `:`, +
  backstop structurel `path.parent() == term_dir`. Teste : `operator_terminal_
  session_content_rejects_traversal` (`..` percent-encode + `C:` drive-prefix).
- **`verifiable` dans canonical bytes signes** (`task.rs`) : le mode d'execution
  (greedy vs sampling) fait partie de l'identite signee — un MITM applicatif ne
  peut pas servir un autre mode sans casser la signature. Teste :
  `task_canonical_includes_verifiable`. `#[serde(default)]` justifie (tolerance
  runtime, pas compat historique — conforme pre-launch §2.3).
- **Threat boundary D5** : « process local hostile lit le token » reste
  hors-scope assume (sandbox OS niveau noeud), documente `PATTERNS §P35` +
  kickoff §5 D5, aligne sur le modele daemon loopback deja accepte projet-wide.
  Confirme.

**Findings** : 1 finding documentation-lag (voir Track H, **P2-H-1**). Aucun
P0/P1 securite.

---

## Track C — Patterns

Opinion formee depuis le code (Tracks A/B) AVANT lecture de PATTERNS.md
(opinion-first, anti-anchoring).

- `docs/rust/PATTERNS.md §P53` (quorum deterministe B-2 + axes provider/backend
  D8 + dead-module cleanup + deps G13) : present, decrit fidelement le code lu.
- `docs/rust/PATTERNS.md §P54` (B-1 dispatch key + B-3 E2E + caveat
  Windows-pump P2-A-1 + signature-gap P2-A-2) : present, exact, honnete sur les
  caveats. Documente que le E2E asserte `results.len()==1` mais PAS la signature
  (P2-A-2) — coherent avec le code lu (`dispatch_loop.rs:234`).
- `docs/shell/PATTERNS.md §P35` (Factory Operator loopback hardening,
  cross-ref P27) : present, complet — couvre auth/SSE-gate/model/spawn + la
  frontiere de menace D5. La duplication auth deliberee (auth.rs vs
  daemon-core) est documentee comme dette suivie (`auth.rs:21-30` + P35), pas
  une duplication accidentelle (sensibilite P2-C-1 satisfaite).

Tech debt cree/resolu :
- Resolu : `RedundancyDispatcher` (module mort retire), `mod redundancy;`
  retire de `lib.rs`. `execute_build` garde dormant avec consommateur NOMME S75
  (LT-7) dans `ROADMAP_COMMITMENTS.md` + §P53 — conforme a la decision D8
  (DEPRECATED+ROADMAP si appelant nommable).
- Cree (documente) : distinction provider (prompt-adaptation : claude/codex/
  gpt/local/human) vs backend (LlmBackend : Ollama/llama_cpp), 2 axes
  orthogonaux, NON unifies (`process.rs:24-34` + §P53).

Provider/backend axes et serde_json vs JCS : pas de nouvelle duplication
canonical introduite. `verifiable` ajoute aux canonical bytes via le chemin
existant `task_canonical_bytes` (JCS), pas de second mecanisme.

**Findings** : aucun. Patterns documentes, alignes sur le code, dette tranchee.

---

## Track D — Scope

16/16 scope cuts (plan §8 / verification.md §3) verifies par grep sur le code
ajoute S71.

Grep cible (`git diff 201b24d..HEAD -- '*.rs' | grep '^+'`) sur :
`ProviderRouter|tree-sitter|trait Provider|NetworkProvider|sharding|Petals|
Parallax|fork_project|search_network` (en excluant commentaires/refs
S72-S76/deferrals) = **0 hit en code actif**.

- #1 ProviderRouter : D4 cable seulement defaut modele + passthrough `req.model`
  — aucun trait router. Confirme.
- #2-6 routage reseau / FTS5 reindex / SearchResult enrichment / barre shell /
  SearchManifest : absents.
- #7-9 fork/projet cible/templates : `repo_root` pointe toujours nexus, pas de
  commande search/open/fork, templates static + static-reader seuls.
- #10-11 GPU partage / quorum cross-MACHINE : S71 prouve cross-PROCESS
  same-machine greedy ; limite cross-GPU documentee §P53.
- #12 sharding / #13 logprobs-watermark / #14 kudos per-task / #15 tree-sitter
  / #16 packaging : absents (logprobs inerte `[0u8;32]`, token bootstrap =
  securite dev pas onboarding).

Les refs S75/cross-machine dans le code (`execute_build` dormant, ROADMAP,
PATTERNS) sont des deferrals documentes, pas des implementations (confirme
Codex Phase B + ma relecture).

**Findings** : aucun. 16/16 scope cuts respectes, 0 fuite.

---

## Track E — Tests Delta

- Annonce (verification.md §2) : +39 Rust A-D (A +1, B +8 net, C +14, D +16),
  +0 Vitest. Exit 1528 vs entree 1486 = +42 (ecart +3 explique : surface de
  test minimale landee avec le bloc off-sprint + arrondi baseline Phase A).
- Observe : nextest exit **1528 passed / 0 skipped** (Docker CI). Front +0
  (web/ non touche).

Decomposition verifiee par lecture des suites :
- A : `dispatch_loop_writes_to_doc` (durci, assert `task:`) +
  `dispatched_task_is_claimed_and_executed_by_worker_engine` (E2E).
- B : task (canonical verifiable), runtime (greedy seed), ollama
  (deterministic_options x2), validator (B-2 properties x3), dispatcher
  (verifiable propagation) ; **−3** = suppression du module `redundancy` et ses
  tests. Aucun appelant orphelin (grep `mod redundancy` = retire, `redundancy_
  factor` est une colonne DB distincte, non liee).
- C : auth (5) + llm_bridge (2 : timeout + diagnostic) + integration (7).
- D : terminal (2) + sprint_history (3) + process (3) + endpoints (8 dont 3
  securite injection/traversal).

Le delta +42 (somme par phase +39 + 3 residuels off-sprint) est coherent et
explique. Aucune phase a delta zero injustifiee (Phase 0 et E sont docs-only,
declarees). Aucun test « mock-for-integration » : le E2E B-3 utilise un vrai
`Engine` worker (StubBackend pour l'hermeticite LLM, pas pour le routage —
le routage dispatch→claim→result est reel cross-component).

**Findings** : aucun. Delta coherent, decomposition verifiee.

---

## Track F — Review Files

- Phases code : 4 (A-D).
- Preflight presents : 4/4 (A EXECUTE, B PLAN-ADAPT, C SCOPE-CUT-CONSISTENT,
  D PLAN-ADAPT — verdicts reels lus depuis les fichiers, confirmes
  verification.md §4).
- Reviews presents : 4/4, tous **`## Verdict: PASS`** (format EXACT, pas
  d'espace avant `:`) — A:16, B:8, C:3, D:3. Le commit `ee8bf6a` a corrige le
  format Phase C (espace avant `:`) — verifie corrige.
- Codex reviews presents : 4/4 (A-D, sorties brutes) + 1 retro-Codex off-sprint.
- Artefacts reconciliation : `sprint71_offsprint_retro_review.md` (verdict
  RECONCILED) + `sprint71_offsprint_codex_review.md` (brut) +
  `sprint70_audit_findings.md` (audit-absorb, CONDITIONAL PASS).

Identite sprint/phase coherente entre preflight, review, codex et sujet de
commit (`Sprint 71 Phase X`). Le verdict offsprint `## Verdict : RECONCILED`
(espace avant `:`) est un artefact de reconciliation special, pas une review de
phase standard — les 4 reviews de PHASE respectent toutes le format strict.

**Findings** : aucun bloquant. Voir **P3-F-1** (note meta cosmetique : le body
Phase D `f19ed83` recap C comme « EXECUTE » alors que le preflight C reel =
SCOPE-CUT-CONSISTENT). Auto-declare par verification.md §4 note meta. Aucun
impact code, verdicts reels traces dans les fichiers preflight.

---

## Track G — Carry-Overs

- Items CLOSED S71 (12) : B-1, B-2, B-3, G1, G2, G5, G6, G7, G9, G12, G13, D8.
  Chacun verifie (code + test) cf. Tracks A/B/C/E. Tous fermes in-sprint.
- Nouveaux S72 : P2-A-1 (worker-pump Windows), P2-A-2 (E2E sans signature),
  P3-A-3, P3-B-1, P3-B-2, 3xP2/3xP3 Phase C, 3xP2/1xP3 Phase D — tous
  documentes verification.md §5 + routes S72.
- Reconduits : P2-A-1(rand) et P2-AUDIT-2(iroh) exemptes blocker amont ;
  T-NN+2 exempte (upstream wasm) ; P2-F-3 a 2/3 (non escalade) ; LT-2 trigger
  PENDING ; LT-5/LT-7 horizon long/partiel.

Aucun item n'atteint 3 reports sans exemption — pas d'escalade G7 requise.

**Findings** : aucun. Discipline de carry respectee, routage complet.

---

## Track H — HARDENING

Le bloc Factory Phase C ajoute une **nouvelle surface reseau locale** : un
serveur HTTP Operator (`:3001`) qui ECRIT des fichiers
(`/api/artifacts/draft`) et SPAWN des agents `bypassPermissions`
(`/api/chat/{id}/stream`, `/api/terminal/ws`). C'est la principale extension
de surface d'attaque de S71.

Etat de la couverture documentaire menace :
- La defense est **implementee et testee** (token+Host+CORS, gate SSE, timeout
  spawn — Track B) et son rationale d'ingenierie est capture dans
  `docs/shell/PATTERNS.md §P35` (complet, incluant la frontiere D5).
- MAIS **aucun fichier de `docs/security/` n'a ete touche dans tout le
  perimetre S71** (`git diff --name-only 201b24d..HEAD -- docs/security/` =
  vide). En consequence :
  - `THREAT_MODEL.md` n'a aucune entree Operator/bypassPermissions/spawn/`:3001`
    (seule la ligne generique daemon X-SBFB-Token existe — grep
    `operator|bypassPermissions|spawn|3001|llm_bridge|artifact.*draft` = 0).
  - `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` — dont la raison d'etre est de cataloguer
    les endpoints loopback et leur niveau de confiance — ne liste PAS le serveur
    Operator `:3001` (grep `operator|sbfb-factory|3001|spawn` = 0).

Zones rouges (R-iroh-audit P0 / R-wasmtime-cve P0 / R-libcrux-hax P2 /
R-pyodide-escape) : inchangees, aucun changement de statut non documente.
HARDENING_ROADMAP.md : pas de pre-requis S71 non livre identifie.

**Findings** : **P2-H-1** (documentation menace en retard sur le code). La
defense est faite et eprouvee ; seul le CATALOGUE menace canonique
(THREAT_MODEL + LOOPBACK_ENDPOINTS_TRUST_TIERS) ne reference pas encore la
nouvelle surface write+spawn de l'Operator. Classe **P2** (et non P1) : la
frontiere de confiance EST documentee (PATTERNS §P35 couvre token+Host+CORS+
SSE-gate+spawn+D5), seul le catalogue security/ accuse le retard — c'est une
completude documentaire, pas une frontiere non defendue. Le plan d'audit S71
Track 8 anticipait precisement ce finding (« ou ouvrir un finding si la doc
menace est en retard sur le code »).

---

## Track I — Meta-Process

- Phase 0 audit-absorb = deviation PO-3 documentee (kickoff §3).
  `sprint70_audit_findings.md` couvre S70 + le bloc off-sprint, verdict
  **CONDITIONAL PASS**, mappe chaque P0/P1 vers une phase A-D. Confirme.
- G8 : 4/4 phases code, 0 DESIGN-CONFLICT. 2 PLAN-ADAPT (B : gap backend
  Ollama qui n'attachait jamais `GenerationOptions` — evidence structurelle
  reelle ; D : crate binary-only → tests inline + harness HTTP — evidence
  structurelle reelle). Non consecutifs (separes par C). Pas de derive du plan.
- Commit discipline : 4 phase commits, sujets EXACTS `fix(scope): Sprint 71
  Phase X — title`. Bodies = **9 sections exactes** chacun (`## Contexte`,
  `## Fichiers`, `## Delta tests`, `## Verification §7.4`, `## Scope cuts`,
  `## G8 traceability`, `## Pre-launch protocol`, `## Codex verification`,
  `## Carry closure...`). Section `## Codex verification` presente partout.
- Pas de `--no-verify`, pas de `--amend` sur les phase commits (historique
  lineaire, parents coherents). Pas d'emoji (seuls em-dash/fleches/§
  typographiques).
- Arbitrage §11 : mono-sprint acte, Phase D a tenu (reconciliation complete,
  pas partielle — retro-review + retro-Codex + retro-audit + 16 tests des 5
  surfaces off-sprint). Pas de carry de reconciliation residuel vers S72.
- Stash WIP terminal S71 (G1) resolu (drop, asciicast `.cast` HEAD conserve).
  2 stashes restants sont du debris dev pre-existant hors-scope S71.

**Findings** : aucun bloquant. Discipline process respectee de bout en bout.

---

## Summary

| Severity | Count | Items |
|----------|-------|-------|
| P0 | 0 | — |
| P1 | 0 | — |
| P2 | 1 | P2-H-1 (threat doc lag : Operator surface absente de THREAT_MODEL + LOOPBACK_ENDPOINTS_TRUST_TIERS) |
| P3 | 2 | P3-F-1 (body Phase D recap C=EXECUTE vs reel SCOPE-CUT-CONSISTENT, cosmetique, auto-declare) ; P3-OS-1 (operator_server.rs:519 OR dupliquee `starts_with("## Verdict") || starts_with("## Verdict")` — no-op benin, PRE-EXISTANT S70 `69e3a06`, hors perimetre code S71 mais dans un fichier touche) |

Rigor signal G4 : 0 P0/P1 + 1 P2 + 2 P3 documentes avec evidence negative
exhaustive par track. Satisfait (>= 1 P2+). PAS un CONCERN (audit non
superficiel : 9 tracks, evidence fichier:ligne / SHA).

---

## Verdict : PASS

0 P0, 0 P1, 1 P2 documente (+2 P3). La reconciliation du bloc off-sprint est
complete (RECONCILED), les phases A-D ferment 12 gaps (1 P0 + 6 P1 +
compute/dette) avec code + tests + non-regression. La securite Factory est
implementee et eprouvee. Les suites rejouent en CI Linux / Docker avec **0
regression** (full workspace 1532 tests ; 1 flake de timing isole en code
intouche S54, prouve 3/3 PASS en isolation — non-regression ; compte de
reference du plan 1528). Le seul P2 (P2-H-1) est un retard documentaire du
catalogue menace canonique, pas une frontiere non defendue — il NE bloque PAS
le kickoff S72 (la defense est en place et testee). **S72 kickoff debloque.**

---

## Carry-Over To Sprint 72

- **P2-H-1** : owner = S72 (track securite/HARDENING). Trigger = avant toute
  extension de la surface Operator (ProviderRouter cable le chat Factory sur le
  routage — il TOUCHE cette surface). Exit = entree Operator `:3001` ajoutee a
  `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` (endpoint + trust tier + write/spawn
  capability) ET une entree menace (CSRF/rebinding + spawn-agent) dans
  `THREAT_MODEL.md` referencant `PATTERNS §P35`.
- **P2-A-2** : owner = S72 phase compute. Trigger = quand le E2E cross-process
  est etendu. Exit = le E2E asserte `ResultEntry::verify_signature()` sur le
  result lu (pas seulement `results.len()==1`).
- **P2-A-1 (worker-pump Windows)** : owner = S72 investigation surfaces worker.
  Trigger = si un test worker-pump doit tourner sur Windows natif. Exit =
  root-cause du hang iroh-docs pump Windows, ou exemption formelle CI-Linux-only.
- **P3-F-1** : cosmetique, pas de trigger bloquant (recap body, verdicts reels
  dans les fichiers preflight).
- **P3-OS-1** : pre-existant S70. Trigger = prochaine modification de
  `handle_artifact_draft`. Exit = collapse `starts_with("## Verdict") ||
  starts_with("## Verdict")` en un seul predicat (ou corriger la 2e branche si
  une variante `## Verdict :` espacee etait intendue).

Aucun finding ne necessite un commit `fix(sprint71)` prealable au kickoff S72.
