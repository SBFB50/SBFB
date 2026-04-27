# Sprint 33 — Kickoff (multi-node readiness)

**Ecrit** : 2026-04-27 (session fraiche post-audit gate S32 `242200e`).
**Type** : **sprint impair feature** — pas de phase dette obligatoire
(§6.2.1 Regle 1 s'applique aux sprints pairs uniquement). Cependant,
P2-REVIEW-A-1 est MANDATORY 3/3 et doit etre adresse comme phase.
**Tip master d'entree** : `242200e` (chore(planning): sprint 32
audit findings — verdict PASS, 0 P0/P1, 3 P2, 2 P3).
**Phase 0 audit Sprint 32** : **DEJA JOUE** — findings dans
`.planning/active/sprint32_audit_findings.md` (verdict **PASS**,
0 P0/P1, 3 P2, 2 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** (2026-04-27) : HARDENING_ROADMAP last_validated
  `2026-04-27` (S32 Phase D, meme session). **0 triggers ACTIFS**
  depuis last_validated — la validation est du meme jour.

  Triggers surveilles :
  - iroh > 0.98 : pas de release 0.99 (crates.io 2026-04-27)
  - arti-client > 0.41 : pas de release 0.42 (crates.io 2026-04-27)
  - frost-ed25519 > 2.1 : stable 2.1.0 (inchange)
  - openai-agents-python 0.14.6 : informationnel, inchange

  Tous les autres triggers inactifs (wasmtime, Tor PoW, NIST PQC,
  NVIDIA H100 CCM, RFC 9591, MCP spec, microsoft/sudo).

- **Sprint 33 multi-node research** (commit `1a60033`, pre-kickoff) :
  3 agents paralleles (deploy-readiness, process-analysis, cross-compile)
  ont cartographie l'architecture de deploiement multi-noeuds, identifie
  4 blockers (CORS localhost-only, bearer token flow, systemd/Docker
  absent, stop/status CLI stubs), et propose D1..D5 et risk register.
  Document : `.planning/research/sprint33_multinode_research.md`.

- **CORS codebase exploration** (2026-04-27) : confirme les deux
  CORS configs localhost-only :
  - Daemon Rust : `http.rs:285` `loopback_cors_layer()` → `is_loopback_origin()`
    refuse tout sauf `http://127.0.0.1[:PORT]` et `http://localhost[:PORT]`
  - Coordinator Python : `app.py:121` `allow_origin_regex=r"^https?://(127\.0\.0\.1|localhost)(:\d+)?$"`
  Les deux bloquent l'acces browser depuis une IP publique VPS.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 32 CLOSED. 3 phases A-C livrees + Phase D wrap-up :
- Phase A : iroh stack upgrade 0.97→0.98 workspace-wide (4 crates)
- Phase B : rusqlite 0.36 + arti-client 0.41 dep activation (tor feature)
- Phase C : P2 batch carries audit S31 (max_tokens wire, FROST error
  tests, Tor boot log, HARDENING compteurs, Playwright COEP mock)
- Phase D : wrap-up + verification + audit plan S33 + migration

Audit gate S32 : **PASS** (0 P0/P1, 3 P2, 2 P3).

**Constat critique** : le projet a ~1883 tests, **tous single-machine**.
Aucun test ne spawn 2 daemons, aucun test ne verifie discovery pkarr
cross-reseau, aucun test ne pipe une tache d'un noeud a un autre.
Les tests e2e existants (`nexus-shell-daemon/tests/e2e.rs`,
`nexus-worker/tests/e2e.rs`) spawnent un binaire isole — pattern
transposable a 2 noeuds. La research multi-noeuds (commit `1a60033`)
documente 4 blockers a lever avant le premier test multi-daemon.

### §1.2 Ancrage HARDENING_ROADMAP

HARDENING_ROADMAP §3 roadmap Sprint 18-30 complete — S33 est
**post-roadmap**. Pas de ligne prescriptive S33. Le theme multi-node
readiness est drive par le constat operationnel §1.1, pas par le
threat model. Les mitigations B-Eclipse, B-Sybil, etc. sont en place
(S19-S22) mais jamais testees cross-daemon.

### §1.3 Compteurs tests entree (tip `242200e`)

| Suite | Count | Delta vs S32 sortie |
|---|---|---|
| Rust (cargo nextest) | 883 | 0 |
| SDK (pytest) | 195 (1 flaky Windows) | 0 |
| Coordinator (pytest) | 406 passed + 36 failed (PyO3 stale) + 6 skipped | 0 |
| Gov (pytest) | 46 | 0 |
| Vitest | 267 | 0 |
| Playwright | 42+2f (env) = 44 | 0 |
| size-limit | 7/7 | 0 |
| **Total** | **~1883** | **0** |

Note : le commit `242200e` (audit findings) ne touche aucun code.

### §1.4 Pre-launch protocol policy (rappel)

Pas de deploiement live. `*_FORMAT_VERSION` = 1 partout. Le fix CORS
est une **option de configuration runtime** (CLI flag `--cors-origin`),
pas un wire format change. Les invite tokens (format `nx1` + Base32)
restent inchanges. Aucun tolerant decoder, aucun bump version.

---

## §2 Goal en une phrase

Sprint 33 rend le reseau testable et deployable multi-noeuds pour la
premiere fois : fix CORS coordinator + daemon pour acces externe,
infrastructure deploy (systemd + install script), et premiers tests
multi-daemon end-to-end localhost.
**Critere SMART : 32+ rows fail-fast verts au verification.md (dont
rows 30-33 multi-noeuds), mesure binaire au Phase D wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 32

**DEJA JOUE** — commit `242200e`.

Verdict : **PASS** (0 P0/P1, 3 P2, 2 P3).

Findings integres dans ce kickoff :
- P2-A-1 : rand triple version (0.8+0.9+0.10) → carry 2/3
- P2-B-1 : tor-rtcompat implicit dep → carry 2/3
- P2-REVIEW-C-2 : daemon COEP E2E (mock vs real) → carry 2/3
  (tenter Phase C si test harness le permet)
- P3-iroh-comments : 7 commentaires stale "iroh 0.97" → Phase A nits
- P3-coordinator-comment : commentaire arti-client 2.0 → 0.41 → Phase A nits

ROADMAP_COMMITMENTS check (G7 Regle 3) :
- LT-1 a LT-5 : conditions latentes, aucun declenchement.
- LT-6 : **RESOLVED** S32 Phase A (iroh 0.98 deploye).

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — CORS : explicit opt-in pour acces externe

**Retenu** : Ajouter une option CLI `--cors-origin <ORIGIN>` aux deux
composants HTTP (daemon Rust + coordinator Python). L'option est
**opt-in explicite** : sans elle, le comportement loopback-only actuel
est preserve (zero regression securite). Multiple origins supportees
via repetition du flag.

Pour le **daemon Rust** (`http.rs:285`), la `loopback_cors_layer()` est
etendue : si `--cors-origin` est fourni, les origins sont ajoutees a la
liste d'allowlist. Sinon, seuls `127.0.0.1` et `localhost` sont acceptes.
La logique `is_loopback_origin()` reste le fallback.

Pour le **coordinator Python** (`app.py:121`), le `allow_origin_regex`
est remplace par une liste dynamique construite depuis `--cors-origin`
CLI + les loopback defaults. La regex localhost-only reste le fallback
si aucun origin externe n'est fourni.

**Rejete** :
- *CORS wildcard `*`* — desactive Same-Origin Policy pour tous les
  appelants. Incompatible avec le bearer token auth (credentials
  interdites avec `*`). Risque securite inacceptable.
- *Env var `CORS_ORIGINS` seule* — moins ergonomique que CLI flag pour
  les deploiements systemd (un override ExecStart est plus clair qu'un
  Environment=). Les deux sont supportes (env = fallback si CLI absent).
- *Reverse proxy Nginx/Caddy recommended* — valide pour production mais
  ajoute une couche de setup qui empeche le test multi-noeud rapide.
  Le fix CORS integre est le minimal viable ; le reverse proxy est
  documente comme recommandation production dans l'install script.

**Implications code** :
- `crates/nexus-shell-daemon/src/http.rs` : parametre `cors_origins: Vec<String>`
  dans la fn de construction router, extension de `loopback_cors_layer()`
- `crates/nexus-shell-daemon/src/main.rs` : CLI flag `--cors-origin`
  (clap `Vec<String>`)
- `packages/nexus-coordinator/src/nexus_coordinator/api/app.py` :
  parametre `cors_origins` dans `create_app()`
- `packages/nexus-coordinator/src/nexus_coordinator/cli/commands/start.py` :
  CLI option `--cors-origin`

### D2 — Multi-node test strategy : 2-daemon localhost

**Retenu** : Tests d'integration multi-daemon **sur localhost** (pas de
VPS requis pour CI). Deux daemons sont spawnes sur des ports ephemeres
avec des `NEXUS_GRID_ROOT` env distincts, chacun avec sa propre keypair
iroh. Les tests verifient :
1. Que les deux daemons repondent sur HTTP (smoke test)
2. Que la discovery pkarr fonctionne cross-daemon (gossip sync)
3. Qu'un blob publie sur le daemon 1 est recuperable depuis le daemon 2
4. Qu'une tache soumise sur le daemon 1 est dispatchee au worker du daemon 2

Le test harness vit dans un nouveau crate `nexus-test-harness`
(`crates/nexus-test-harness/`) avec un `DaemonCluster` struct :

```rust
pub struct DaemonCluster { nodes: Vec<DaemonHandle> }
pub struct DaemonHandle { proc: Child, root: TempDir, http_port: u16 }
```

**Phase A delivre le smoke test minimal (row 30)**, Phase C delivre le
harness complet + rows 31-33.

**Rejete** :
- *Docker Compose multi-container* — ajoute Docker comme dependance CI.
  Les binaires Rust se spawnent nativement, pas besoin de conteneurs pour
  des tests localhost. Docker reste pour le pkarr-relay self-hosted.
- *Tests VPS-based (SSH + cloud)* — trop lent pour CI, flaky reseau.
  Les tests localhost suffisent pour valider la logique P2P (iroh gere
  le NAT traversal via relay, pas via port binding direct).
- *Mocking du layer P2P* — perd la valeur du test multi-noeud. Le but
  est de tester la stack reelle, pas des abstractions.

**Implications code** :
- `crates/nexus-test-harness/` (nouveau crate workspace)
- `crates/nexus-test-harness/src/lib.rs` : `DaemonCluster`, `DaemonHandle`
- `crates/nexus-test-harness/tests/` : tests d'integration multi-daemon
- `scripts/test-multi-node.sh` : script smoke test rapide (CI-friendly)

### D3 — Deploy packaging : systemd + install script Linux

**Retenu** : Templates systemd pour les 3 binaires (daemon, worker,
coordinator) + script `scripts/install-node.sh` qui detecte l'OS,
installe les deps (Rust, Python/uv si coordinator), clone le repo,
build `--release`, genere la keypair, et cree les units systemd.

Le script est **Linux-first** (Ubuntu/Debian cible). macOS launchd est
documente en commentaire mais pas genere automatiquement (population
Mac secondaire pre-v1.0). Windows utilise le launcher existant
(`nexus-launcher`).

**Rejete** :
- *Dockerfile pour daemon/worker* — Docker ajoute une couche d'overhead
  pour des binaires statiques Rust. Le pattern existant
  `docker/pkarr-relay/` reste mais n'est pas etendu aux 3 binaires
  principaux. Revision post-v1.0 si la communaute le demande.
- *Snap/Flatpak/AppImage* — overhead de packaging trop eleve pour le
  nombre actuel de contributeurs (1). Revision post-v1.0.
- *Nix flake* — excellente reproductibilite mais barrier d'entree pour
  les contributeurs non-Nix. Revision post-v1.0 si contributeur Nix
  propose un PR.

**Implications code** :
- `scripts/install-node.sh` : script installation (~150 lignes)
- `configs/systemd/nexus-daemon.service` : template systemd daemon
- `configs/systemd/nexus-worker.service` : template systemd worker
- `configs/systemd/nexus-coordinator.service` : template systemd coordinator

### D4 — P2-REVIEW-A-1 MANDATORY : hook LOC guard

**Retenu** : Ajouter un check dans le hook `phase-precommit-lightcheck.sh`
qui grep les fichiers `sprint*_plan.md` dans le staging area pour
detecter les patterns d'estimation LOC (`~NNN LOC`, `environ NNN lignes`,
`budget LOC`, etc.). Si detecte, le hook bloque le commit avec un
message pointant vers `docs/claude/README.md §6.7`.

Ce check est **mecanique** — il empeche la recidive du pattern "LOC
estimation en amont" qui a ete reporte 3 sprints consecutifs sans
resolution. La convention §6.7 est deja documentee ; il manquait
l'enforcement.

**Rejete** :
- *Documentation-only (pas de hook)* — §6.7 existe deja et n'a pas
  empeche 3 recidives. L'enforcement mecanique est la seule option qui
  resout un MANDATORY 3/3.
- *Semgrep rule au lieu de hook* — les fichiers `.planning/` sont du
  Markdown, pas du code. Semgrep est optimise pour AST, pas pour prose.
  Un grep dans le hook est plus simple et plus fiable.
- *Suppression pure de la convention* — les LOC estimations en amont
  sont un anti-pattern documente (§6.7 rationale : plafond psychologique,
  tronque la solution deep, metrique non-pertinente). Supprimer la regle
  plutot que l'enforcer serait un recul.

**Implications code** :
- `.claude/hooks/phase-precommit-lightcheck.sh` : ajout check LOC guard
  (~15 lignes bash)

### D5 — Fail-fast checklist extension : rows multi-noeuds

**Retenu** : 4 nouvelles rows permanentes dans la fail-fast checklist
(verification.md §Checklist) :

| # | Check | Commande | Critere |
|---|---|---|---|
| 30 | 2-daemon localhost smoke | `scripts/test-multi-node.sh` | both HTTP respond |
| 31 | Cross-node discovery | via test harness | peer found |
| 32 | Cross-node blob transfer | via test harness | hash match |
| 33 | Cross-node task pipe | via test harness | result valid |

Row 30 est **MANDATORY** quand du code P2P change (gossip, blobs,
discovery, wire format). Rows 31-33 suivent le meme trigger.

**Rejete** :
- *Rows conditionnelles (skip si pas de change P2P)* — retenu pour les
  phases pure-docs/frontend. Mais quand du code P2P change, les 4 rows
  sont obligatoires.
- *Plus de 4 rows (mobile test, VPS latency, etc.)* — scope creep. Le
  multi-noeud localhost est le minimum viable. Les tests VPS et mobile
  sont des tests manuels pre-v1.0, pas des rows fail-fast CI.

**Implications** : aucun code — ce sont des conventions checklist.
L'implementation est dans D2 (test harness) et D3 (smoke script).

### Acknowledged review findings (G1)

Scoring : D1 ✅, D2 ✅, D3 ⚠️, D4 ⚠️, D5 ✅.
Rigor signal G4 satisfait (2 ⚠️ sur 5, 3 G4 signals).

**D3 ⚠️ G4-SIGNAL-2** : "Zero systemd proof-of-concept codebase."
Decision : acknowledge — les templates systemd sont des fichiers de
configuration statiques simples (ExecStart + User + Restart). Pas de
feature avancee (socket activation, cgroup). Le risque est minimal.
Phase B commit body documentera "systemd templates Ubuntu/Debian 22.04+,
autres init-systems = PR welcome post-v1.0" (G4-SIGNAL-3 absorbe).

**D3 ⚠️ G4-SIGNAL-3** : "No init-system exploration (runit, OpenRC, s6)."
Decision : acknowledge — scope explicitement Ubuntu/Debian pre-v1.0.
Note ajoutee au commit body Phase B.

**D4 ⚠️ G4-SIGNAL-1** : "Semantic gap check 3 (WARN deviation) vs D4
intent (BLOCK presence)."
Decision : **CORRIGE** — D4 implementation est un **nouveau check 6**
(pas un renforcement de check 3). Check 3 reste WARN sur deviation
post-code. Check 6 ajoute un BLOCK preventif sur patterns `~NNN LOC`,
`budget LOC`, `environ NNN lignes` dans `sprint*_plan.md` staged.
Distinction : check 3 = retroactif (LOC commit vs estimation),
check 6 = proactif (estimation en amont dans plan).
Exception HARDENING_ROADMAP.md (bornes indicatives §6.7) preservee
en excluant ce fichier du grep.

---

## §5 Plan Phase outline A..D

### Phase A — CORS fix + P2-REVIEW-A-1 + P3 nits batch

- Fix CORS daemon Rust : `--cors-origin` CLI flag + extension
  `loopback_cors_layer()` (D1)
- Fix CORS coordinator Python : `--cors-origin` CLI option + origins
  dynamiques dans `create_app()` (D1)
- P2-REVIEW-A-1 MANDATORY : hook LOC guard dans
  `phase-precommit-lightcheck.sh` (D4)
- P3 nits batch : 7 commentaires stale "iroh 0.97" + 1 commentaire
  arti-client 2.0→0.41 (8 fichiers, zero impact runtime)
- Commit cible : `feat(sprint33): Sprint 33 Phase A — CORS external
  access + LOC guard + P3 nits`

### Phase B — Deploy infrastructure (systemd + install script)

- Templates systemd pour daemon, worker, coordinator (D3)
- Script `scripts/install-node.sh` (D3)
- Commit cible : `feat(sprint33): Sprint 33 Phase B — deploy infra
  systemd + install script`

### Phase C — Multi-node test harness + smoke test

- Crate `nexus-test-harness` avec `DaemonCluster` (D2)
- Script `scripts/test-multi-node.sh` (row 30)
- Tests d'integration multi-daemon (rows 31-33) (D2)
- Tenter P2-REVIEW-C-2 COEP E2E avec daemon reel si harness supporte
  (sinon carry 3/3 MANDATORY S34)
- Commit cible : `feat(sprint33): Sprint 33 Phase C — multi-node test
  harness + 2-daemon smoke`

### Phase D — Wrap-up + verification + audit plan S34

Standard wrap-up :
- sprint33_verification.md (fail-fast 32+ rows)
- sprint33_carry_summary.md
- sprint34_audit_plan.md
- SPRINT_LOG.md row S33
- CLAUDE.md §Etat actuel update
- Memory update nexus_grid_pivot.md + MEMORY.md
- HARDENING_ROADMAP.md update (last_validated S33)
- Migration active/ → archive/v1.2/
- Commit cible : `chore(sprint33): Phase D — wrap-up + verification
  + audit plan S34 + migration`

---

## §6 Items carry/dette (G7)

### Carry S32 — traitement S33

| ID | Description | Reports | Resolution S33 | Phase |
|---|---|---|---|---|
| P2-REVIEW-A-1 | LOC plan meta-process | **3/3 MANDATORY** | Hook LOC guard | A |
| P2-A-1 | rand triple version (0.8+0.9+0.10) | 2/3 | Carry confirme | — |
| P2-B-1 | tor-rtcompat implicit dep | 2/3 | Carry confirme | — |
| P2-REVIEW-C-2 | daemon COEP E2E | 2/3 | Tenter Phase C | C |
| P3-grammar | grammar executor wire | 2/3 | Carry confirme | — |
| P3-watermark | watermark executor wire | 2/3 | Carry confirme | — |
| P3-iroh-comments | 7 commentaires stale "iroh 0.97" | 2/3 | Fix Phase A | A |
| P3-coordinator-comment | commentaire arti-client 2.0 → 0.41 | 1/3 | Fix Phase A | A |

### Items differes S34+

| ID | Description | Reports apres S33 | Sprint cible | Justification |
|---|---|---|---|---|
| P2-A-1 | rand triple version | **2/3** | S34 | Upstream frost-ed25519 dep sur rand 0.8. Pas d'action SBFB possible (ni fork ni patch raisonnable). Report justifie par blocker externe. |
| P2-B-1 | tor-rtcompat implicit dep | **2/3** | S34 | Fonctionnel via resolution implicite PreferredRuntime. Expliciter si arti-client change API. Report justifie par risque faible. |
| P2-REVIEW-C-2 | daemon COEP E2E | **2/3 ou RESOLVE** | S34 si non resolu C | Tenter via test harness Phase C. Si le harness ne supporte pas le blob-serve HTTP, carry 3/3 MANDATORY S34. |
| P3-grammar | grammar executor wire | **2/3** | S34+ | Ollama ne supporte pas GBNF natif. Wire quand backend change. |
| P3-watermark | watermark executor wire | **2/3** | S34+ | Defense-in-depth, SynthID inject worker-side pas executor-side. |

### Items long-terme (ROADMAP_COMMITMENTS)

| ID | Condition | Status |
|---|---|---|
| LT-1 | v1.0 + design doc + Gini > 0.70 | Latent |
| LT-2 | tag v1.0 | Latent |
| LT-3 | v1.0 + 3+ contrib non-compute | Latent |
| LT-4 | v1.0 + N1 FROST + partnership | Latent |
| LT-5 | multi-worker deploy OR v1.0 | Latent |
| LT-6 | iroh > 0.97 OR v1.0 | **RESOLVED** S32 |

Note LT-5 : le test multi-daemon S33 ne constitue PAS un "multi-worker
deploy" au sens LT-5 (qui vise la production avec state persistent).
LT-5 reste latent.

---

## §7 Scope cuts

Ce que S33 ne fait PAS :

1. **VPS deployment effectif** — S33 prepare l'infra (systemd, script)
   mais ne deploie pas sur VPS. Deployment reel = operation manuelle
   post-sprint.
2. **Mobile browser testing** — le shell React responsive est teste
   en desktop. Test iPhone/Android = test manuel pre-v1.0.
3. **iroh relay over Tor** — scope-cut S34+ (iroh 0.98 n'ajoute pas
   de proxy config Endpoint)
4. **Nym mixnet** — re-defere (SDK paused crates.io)
5. **TEE H100 attestation** — scope-cut (pas hardware partenaire)
6. **DKG distribue FROST** — post-v1.0 (trusted dealer suffisant N=3)
7. **CI multi-node VPS** — les tests CI restent localhost. CI VPS =
   infrastructure post-v1.0.
8. **Docker daemon/worker** — binaires Rust statiques, Docker non
   justifie pre-v1.0
9. **stop/status CLI** — stubs existants suffisants. Graceful shutdown
   = Ctrl+C + systemd KillSignal=SIGINT.
10. **Build CI merge (build-binaries.yml)** — reporte S34 (scope test
    d'abord, CI packaging ensuite)
11. **Cross-node task execution reel (Ollama)** — le test multi-daemon
    utilise des stubs. L'execution Ollama reelle est testee single-node
    (S31 Phase A task_runner).
12. **Output filter client-side (iframe defense-in-depth)** — S35+

---

## §8 Tracabilite scope

Table mappant les items S32 "What's NOT" sur leur traitement S33 :

| Item S32 scope-cut | Sprint + Phase S33 | Status |
|---|---|---|
| iroh relay over Tor | S34+ | SCOPE-CUT (inchange) |
| Nym mixnet phase 1 | S34+ | RE-DEFERE |
| TEE H100 attestation | post-v1.0 | SCOPE-CUT |
| DKG distribue FROST | post-v1.0 | SCOPE-CUT |
| Onion service hosting | post phase 1 | SCOPE-CUT |
| Full process isolation | LT | SCOPE-CUT |
| openai-agents-python | informationnel | INCHANGE |
| llama.cpp executor | S34+ | SCOPE-CUT |
| Output filter client-side | S35+ | SCOPE-CUT |
| iroh 1.0 wait | pas de release | INCHANGE |
| rusqlite 0.39 | 0.36 suffisant | SCOPE-CUT |

Items nouveaux S33 non dans S32 :
- Multi-node readiness (D1-D5) → **INTEGRE** S33

---

## §9 Risk register

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | iroh discovery localhost ne fonctionne pas en 2-daemon (meme machine, ports differents) | Medium | High | iroh 0.98 supporte multi-instance via keypair distinctes. Si discovery DHT echoue localhost, fallback sur direct connect via NodeAddr connu. |
| R2 | CORS fix introduit une regression securite loopback | Low | High | Le fix est opt-in explicite. Sans `--cors-origin`, le comportement est identique a pre-S33. Tests unitaires CORS existants preserves. |
| R3 | systemd unit templates non testes sur distro autre que Ubuntu | Medium | Low | Templates generiques (`ExecStart`, `User`, `WorkingDirectory`). Pas de feature systemd avancee. Test manuel sur Ubuntu CI. |
| R4 | Test harness trop lent pour CI (2 daemons = 2× boot time) | Medium | Medium | Boot time daemon ~2s (release build). 2 daemons = ~4s. Acceptable en CI. Si trop lent, paralleliser le spawn. |
| R5 | P2-REVIEW-C-2 COEP E2E non resolvable meme avec test harness | High | Low | Si le test harness ne peut pas spawner un blob-serve HTTP, carry 3/3 MANDATORY S34. Le COEP mock Playwright (S32 Phase C) reste une couverture partielle. |
| R6 | Hook LOC guard trop strict (false positives sur du texte legitime) | Low | Low | Le pattern grep est cible sur `~NNN LOC` et `budget LOC`, pas sur toute mention de "LOC". False positives = override manuel avec commentaire justificatif. |

---

## §10 Audit gate pattern — rappel

Phase 0 audit S32 **jouee** — verdict PASS, commit `242200e`.
Phase D produira :
- `sprint33_verification.md` (self-report fail-fast)
- `sprint34_audit_plan.md` (plan pour S34 Phase 0)
- `sprint33_carry_summary.md`

---

## §11 Checkpoint de validation

5 questions pour arbitrage user AVANT le plan detaille :

1. **D1 CORS** : opt-in explicite via `--cors-origin` (daemon + coord).
   Sans le flag, loopback-only preserve. Pas de wildcard. Suffisant ?
2. **D2 Multi-node test** : 2-daemon localhost, crate `nexus-test-harness`.
   Pas de VPS en CI. Rows 30-33 dans fail-fast. Acceptable ?
3. **D3 Deploy infra** : systemd + install script Linux. Pas Docker pour
   les 3 binaires. macOS launchd documente mais pas genere. OK ?
4. **D4 LOC guard** : hook lightcheck grep `~NNN LOC` dans plans.
   Enforcement mecanique du §6.7. Resolve P2-REVIEW-A-1 3/3. OK ?
5. **D5 Fail-fast** : 4 rows permanentes multi-noeuds, obligatoires
   quand code P2P change. Trop ou pas assez ?
