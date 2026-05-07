# Sprint 55 — Kickoff (CI self-hosted build + LT-7 foundation)

**Ecrit** : 2026-05-07 (post-audit gate S54 PASS `734da72`).
**Type** : **sprint impair** — pas de phase dette obligatoire (§6.2.1
Regle 1). 2 items 3/3 MANDATORY a traiter Phase A.
**Tip master d'entree** : `ee0e54c`.
**Phase 0 audit Sprint 54** : **DEJA JOUE** — `734da72` PASS
(0 P0, 0 P1, 3 P2, 2 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-07 (0j). 5 fichiers
  security avec triggers_revalidate. 0 trigger actif pertinent pour
  le theme S55. HARDENING_ROADMAP frais (S54). WARRANT_CANARY trigger
  frost-ed25519 > 2.1 stale (upgrade 3.0 S34, trigger non-MAJ).
  Pas de pre-research supplementaire.

- **Woodpecker CI** (context7 `woodpecker-ci/woodpecker` 2026-05-07) :
  v3.14.0 stable. Docker Compose server + agent. GitHub OAuth app
  requis. Caddy reverse proxy recommande (auto Let's Encrypt TLS).
  Agent secret via `openssl rand -hex 32`. gRPC port 9000 pour
  communication agent→server (h2c:// si via proxy). Tag `latest`
  supprime en v3 — utiliser tag `v3` ou version pinned. Pas de
  plugin privileged par defaut en v3. Pipeline existant
  `.woodpecker/ci-linux.yml` compatible v3 sans modification.

- **Rust reproducible builds** (WebSearch 2026-05-07) :
  rust-lang/rust#129080 — 62/81 sub-tasks completes. SOURCE_DATE_EPOCH
  + remap-path-prefix couvrent ~95% des cas x86_64-linux homogene.
  Cross-platform (Linux/macOS/Windows) non reproductible. MVP S55 :
  x86_64-linux seulement, pin rustc version string.

- **ROADMAP_COMMITMENTS check** :
  - LT-7 self-hosted build : **PRE-V1.0 OBLIGATOIRE**. S55 target.
    Condition active. SELF_HOSTED_BUILD.md §8 scope MVP : build
    executor + dispatcher routing + quorum SHA256 + CLI + test E2E.
  - LT-1 Kudos-v2 : trigger Gini > 0.70. Pas de donnees. Latent.
  - LT-2..LT-5 latents. LT-6 RESOLVED S32.
  - 1 condition declenchee (LT-7).

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 54 CLOSED + audit PASS (`734da72`). Le workspace est en
edition 2024. Le P2P iroh est valide cross-machine (LAN Win-Mac,
WAN dev-VPS Helsinki). Le GAP E2E tasks_doc_ticket est ferme —
prerequis LT-7 debloque. L'infra CI VPS est prete (Docker 29.4.3,
images CI pullees, deploy-key GitHub, woodpecker-cli 3.14.0).

**Etat technique (tip `ee0e54c`)** :
- Workspace clean, edition 2024, Rust 1.94
- Pipeline Woodpecker ecrit (`.woodpecker/ci-linux.yml`, 12 steps,
  images pinnees SHA256). Agent non deploye. Serveur non deploye.
- GHA Rust CI corrige (nexus-core-py supprime), run non valide
  post-push master
- Dispatcher : 205 LOC, `task_type: String` libre, zero routing
  par type. Tous les tasks traites identiquement.
- Validator : 226 LOC, single-result acceptance, zero quorum logic.
  Signe + existence + status guard uniquement.
- Task struct : `metadata: BTreeMap<String, String>` pret pour les
  parametres build (zero changement wire format)
- 1 test flaky (browse probe_and_cache, timing-dependent)

**Carries entrants S55** :

| Item | Compteur | Source |
|---|---|---|
| P2-REVIEW-B-1-S52 Woodpecker serveur | **3/3 MANDATORY** | S54 escalade |
| P2-REVIEW-B-2-S52 GHA validation | **3/3 MANDATORY** | S54 escalade |
| P2-A-1 rand blocker upstream | 13+/3 | exemption externe |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-S53-outbox non-persistant | 2/3 | S53 Phase F review |
| P2-S53-browse_request rate-limit | 2/3 | S53 Phase G review |
| P2-S54-forbid-deny-doc | 1/3 | Phase A review |
| P2-S54-lightcheck-edition-faux-positif | 1/3 | Phase A review |
| P2-S54-jitter-republish | 1/3 | Phase B review |
| P2-S54-windows-test-cfg-unix | 1/3 | Phase B review |
| P2-S54-test-E2E-multi-noeuds | 1/3 | Phase C review |
| P2-S54-project-name-hardcode | 1/3 | Phase C review |
| P2-S54-rustfmt-drift-sessions | 1/3 | Phase D review |
| P2-S54-AUDIT-1 flaky browse test | 1/3 | S54 audit |
| P2-S54-AUDIT-2 SAFETY convention FFI | 1/3 | S54 audit |
| P2-S54-AUDIT-3 invite version naming | 1/3 | S54 audit |

### §1.2 Ancrage roadmap

S54 a ferme le GAP E2E (prerequis LT-7) et prepare l'infra VPS
(Docker, images, deploy-key). S55 est le sprint qui livre LT-7
Tier 1 (Woodpecker CI operationnel) et pose les fondations Tier 2
(build executor + quorum).

SELF_HOSTED_BUILD.md §1.1 strategie 3 etages :
- **Tier 1 CI Woodpecker** : S52 config + S54 images pin + **S55
  server + E2E validation**
- **Tier 2 Build worker SBFB** : **S55 foundation** (dispatcher
  routing + executor + quorum)
- **Tier 3 Reseau autonome** : post-S55 (N builders + auto-deploy)

ROADMAP_COMMITMENTS LT-7 : "PRE-V1.0 OBLIGATOIRE. Pas de tag v1.0
sans self-hosted build operationnel."

### §1.3 Compteurs tests entree (tip `ee0e54c`)

| Suite | Count |
|---|---|
| Rust nextest | 1207 |
| Rust doctests | 6 passed, 1 ignored |
| Vitest | 250 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 6/6 |
| **Total** | **~1463** |

**Post-S55 attendu** : ~1475+ (build executor tests, quorum tests,
integration test build E2E).

### §1.4 Pre-launch protocol policy (rappel)

Phase B-C touchent le dispatcher et validator (ajout routing
task_type "build" et quorum accumulation). Pre-launch : pas de
bump TASK_FORMAT_VERSION — les parametres build passent par
`metadata: BTreeMap<String, String>` existant, zero changement
de la struct Task. Le champ `task_type: String` accepte deja
n'importe quelle valeur.

---

## §2 Goal

Sprint 55 rend le reseau SBFB capable de se compiler via sa propre
infra : Woodpecker CI operationnel sur VPS (Tier 1 complet), build
executor + quorum SHA256 dans le protocole (Tier 2 foundation), et
GHA valide comme fallback. Les 2 items 3/3 MANDATORY (Woodpecker
serveur + GHA validation) sont fermes.
**Critere SMART : 24+ rows fail-fast verts au verification.md, mesure
binaire au Phase E wrap-up. Woodpecker serveur accessible HTTPS.
`task_type: "build"` dispatch + quorum SHA256 dans tests.**

---

## §3 Phase 0 — Audit gate S54

**DEJA JOUE** : commit `734da72` PASS
(0 P0, 0 P1, 3 P2, 2 P3).
Audit findings dans `.planning/active/sprint54_audit_findings.md`.
16 carries documentes pour S55 (cf. §1.1 ci-dessus).

---

## §4 Decisions Day 0 (D1..D4 gelees)

### D1 — Woodpecker serveur via Docker Compose + Caddy auto-TLS

**Retenu** : deployer le serveur Woodpecker sur le VPS sbfb-eu
(`135.181.42.188`) via Docker Compose (serveur + agent side-by-side).
Caddy en reverse proxy pour TLS automatique (Let's Encrypt). GitHub
OAuth app pour acces repo + webhooks. SQLite backend (pas besoin de
PostgreSQL pour un seul agent). Configuration systemd pour demarrage
automatique.

**Rejete** :
- Nginx + Certbot : plus de configuration manuelle, renouvellement
  cert explicite. Caddy auto-TLS est plus simple et les docs
  Woodpecker fournissent un exemple Caddy natif.
- Woodpecker binaire natif (sans Docker) : plus complexe pour les
  mises a jour. Docker Compose est le deploiement standard et
  permet le rollback instantane via tag version.
- Forgejo Actions : necessiterait une instance Forgejo complete.
  Woodpecker est deja configure (pipeline S52, images pinnees S54).
- Traefik au lieu de Caddy : plus complexe, labels Docker requis.
  Caddy est plus simple pour un seul service.

**Implications code** : `configs/woodpecker/docker-compose.yml` (NEW),
`configs/woodpecker/Caddyfile` (NEW),
`configs/systemd/woodpecker.service` (NEW). Pas de code applicatif.

### D2 — GHA validation via push + run documentation

**Retenu** : push master au remote GitHub apres les commits code du
sprint (Phase D ou E), declencher le workflow GHA, documenter le run
ID dans le commit body wrap-up. Le fix nexus-core-py est deja commite
(S54 Phase D). Le push valide l'ensemble du code S54+S55.

**Rejete** :
- API workflow_dispatch : complexite supplementaire sans benefice.
  Le push standard declenche automatiquement les workflows.
- Reporter a S56 : item a 3/3 MANDATORY, reporter interdit (§6.2.1
  Regle 2).

**Implications code** : 0 fichiers code. Action GitHub uniquement.

### D3 — LT-7 build task routing + executor tmpdir MVP

**Retenu** : ajouter un branch `task_type == "build"` dans le
dispatcher (crates/nexus-coordinator-rs). Le build task encode ses
parametres dans `metadata: BTreeMap<String, String>` (cles
`build.repo`, `build.commit`, `build.binary`, `build.target`,
`build.source_date_epoch`). Zero changement struct Task, zero bump
TASK_FORMAT_VERSION (pre-launch policy).

Build executor dans nexus-worker-core : module `build_executor.rs`
qui (a) clone le repo dans un tmpdir, (b) execute `cargo build
--release --locked -p <binary>` avec SOURCE_DATE_EPOCH + remap-path-
prefix, (c) calcule SHA256 du binaire, (d) soumet le ResultEntry.

Quorum SHA256 dans le validator : accumulation de N resultats
(redundancy_factor) avant comparaison. Majorite SHA256 identique →
accepte. Divergence → rejet + alerte. Nouveau statut DB
`AwaitingQuorum` entre `Dispatched` et `Completed`.

**Rejete** :
- Binaire separe `nexus-builder` : overkill pour le MVP. Un module
  dans nexus-worker-core suffit. Separation binaire en Tier 3.
- Nix sandbox : courbe d'apprentissage significative. tmpdir est
  suffisant pour demontrer le protocole. Podman rootless en Tier 2.
- Full podman rootless des S55 : overhead d'integration. Le MVP
  tmpdir valide la chaine dispatch→build→quorum. L'isolation
  hermetique vient apres.
- Streamer les logs build : complexite significative (websocket ou
  SSE depuis le worker). Le MVP retourne le resultat final. Les
  logs streames sont scope Tier 3.
- Cross-platform builds : x86_64-linux seulement en MVP. Les builds
  multi-target (Windows, macOS) necessite des workers dediees par
  OS et la reproductibilite cross-platform n'est pas resolue
  upstream (rust-lang/rust#129080).

**Implications code** : `crates/nexus-worker-core/src/build_executor.rs`
(NEW ~200-300 LOC), `crates/nexus-coordinator-rs/src/validator.rs`
(quorum extension ~80-100 LOC), `crates/nexus-coordinator-rs/src/db.rs`
(migration AwaitingQuorum ~30 LOC), 5+ tests.

### D4 — P2 batch selection (4 items quick)

**Retenu** : resoudre 4 items P2 quick en une phase :
- `P2-S54-jitter-republish` : ajouter jitter ±15s aleatoire au
  timer 45s dans `runtime.rs` (prevention thundering-herd)
- `P2-S54-project-name-hardcode` : extraire "sbfb" hardcode dans
  `invite_api.rs` vers une constante configurable
- `P2-S54-AUDIT-2 SAFETY convention FFI` : ajouter `// SAFETY:`
  aux blocs unsafe pre-existants (libc::kill dans launcher +
  test-harness, Win32 FFI dans named_pipe_server)
- `P2-S54-AUDIT-3 invite version naming` : renommer
  `INVITE_VERSION` → `INVITE_FORMAT_VERSION`, type u8 → u16,
  aligner avec la convention projet

**Rejete** :
- Inclure outbox persistant (2/3) : necessite design (format
  fichier, rotation, recovery). > 500 LOC. Carry S56.
- Inclure browse_request rate-limit (2/3) : necessite design
  (per-peer tracking, decay). > 500 LOC. Carry S56.
- Inclure test E2E multi-noeuds : depend de l'infra VPS Woodpecker,
  mieux en S56 quand le pipeline est operationnel.
- Inclure windows-test-cfg-unix : investigation CI cross-platform
  hors scope theme CI self-hosted build linux.

**Implications code** : `runtime.rs` (jitter 1 ligne),
`invite_api.rs` (constante), `invite.rs` worker (rename + type),
`launcher/main.rs` + `test-harness/lib.rs` + `named_pipe_server.rs`
(SAFETY comments). ~50-80 LOC total.

### Acknowledged review findings (G1)

Scoring : D1 ✅, D2 ⚠️, D3 ✅, D4 ⚠️.
Rigor signal G4 satisfait (2 ⚠️ sur 4).

D2 ⚠️ (nexus-core-py fix reference + workflow_dispatch) :
acknowledge — le fix est confirme dans `be633c3` (38 lignes
rust-ci.yml, 0 occurrences restantes dans HEAD). Le reviewer
n'a pas trouve le commit car le titre utilise "Rust CI fix"
et pas "nexus-core-py". Concernant workflow_dispatch vs push :
le reviewer a raison que c'est un choix de process, pas une
contrainte architecturale. Decision maintenue (push est le
declencheur naturel apres commit, 0 setup supplementaire).

D4 ⚠️ (task roster vs decisions architecturales) : acknowledge
— le reviewer a raison que les 4 items P2 sont des taches, pas
des decisions architecturales. Le §D4 documente la **selection**
(quels items inclure dans le batch et lesquels reporter) ce qui
est une decision de priorisation. Le framing "Day 0" est un
abus de format kickoff, pas un abus de substance.

---

## §5 Plan Phase outline A..E

### Phase A — Woodpecker serveur + GHA validation (3/3 MANDATORY)

**But** : deployer le serveur Woodpecker sur VPS sbfb-eu et valider
le pipeline E2E. Pusher master vers GitHub et documenter le run ID
GHA. CLOSE les 2 items 3/3 MANDATORY.

- docker-compose.yml (server + agent)
- Caddyfile (TLS auto Let's Encrypt)
- systemd service woodpecker.service
- GitHub OAuth app creation + webhook config
- VPS deploy + pipeline E2E validation
- Push master → GHA run → documenter run ID
- Commit : `feat(sprint55): Sprint 55 Phase A — Woodpecker server
  deploy + GHA validation`

### Phase B — LT-7 build executor + dispatcher routing

**But** : ajouter le support `task_type: "build"` dans le protocole.
Dispatcher routing + build executor tmpdir MVP.

- `build_executor.rs` : clone repo, cargo build, SHA256, ResultEntry
- Dispatcher routing : task_type == "build" → build path
- Build metadata validation (cles requises dans metadata BTreeMap)
- Tests unitaires : build executor mock, dispatcher routing
- Commit : `feat(sprint55): Sprint 55 Phase B — LT-7 build executor
  + dispatcher task_type routing`

### Phase C — LT-7 quorum SHA256 + integration test

**But** : implementer la verification quorum pour les build tasks.
Integration test avec build dispatch E2E.

- Validator quorum : accumulation N resultats, SHA256 comparison
- DB migration : statut AwaitingQuorum
- Quorum evaluation : 2/3 match → accepted, diverge → rejected
- Integration test : build task dispatch → executor → quorum
- Commit : `feat(sprint55): Sprint 55 Phase C — LT-7 quorum SHA256
  validation + build E2E test`

### Phase D — P2 batch quick carries

**But** : resoudre 4 items P2 quick S54 pour prevenir l'accumulation.

- jitter-republish : ±15s random sur timer 45s
- project-name-hardcode : constante configurable
- SAFETY convention FFI : comments sur unsafe pre-existants
- invite version naming : INVITE_VERSION → INVITE_FORMAT_VERSION
- Commit : `feat(sprint55): Sprint 55 Phase D — P2 batch quick
  carries (jitter + SAFETY + naming)`

### Phase E — Wrap-up + verification + audit plan S56

**But** : cloturer le sprint.

- CLAUDE.md : update S55 CLOSED, carries S56
- HARDENING_ROADMAP : update last_validated S55
- verification.md : 24+ fail-fast rows
- sprint56_audit_plan.md : 7+ tracks
- Commit : `chore(sprint55): Phase E — wrap-up + verification +
  audit plan S56`

---

## §6 Items carry/dette

### Carries confirmes S55

- [phase A] **P2-REVIEW-B-1-S52** Woodpecker serveur 3/3 MANDATORY :
  **ADRESSE Phase A** → CLOSE attendu.
- [phase A] **P2-REVIEW-B-2-S52** GHA validation 3/3 MANDATORY :
  **ADRESSE Phase A** → CLOSE attendu.
- [phase D] **P2-S54-jitter-republish** 1/3 :
  **ADRESSE Phase D** → CLOSE attendu.
- [phase D] **P2-S54-project-name-hardcode** 1/3 :
  **ADRESSE Phase D** → CLOSE attendu.
- [phase D] **P2-S54-AUDIT-2 SAFETY convention FFI** 1/3 :
  **ADRESSE Phase D** → CLOSE attendu.
- [phase D] **P2-S54-AUDIT-3 invite version naming** 1/3 :
  **ADRESSE Phase D** → CLOSE attendu.
- [carry] **P2-A-1** rand blocker upstream 13+/3 : exemption externe.
- [carry] **P2-AUDIT-2** iroh transitives : herite pin 0.98.
- [carry] **P2-S53-outbox non-persistant** 2/3 : carry S56
  (necessite design > 500 LOC).
- [carry] **P2-S53-browse_request rate-limit** 2/3 : carry S56
  (necessite design > 500 LOC).
- [carry] **P2-S54-forbid-deny-doc** 1/3 : carry S56 (docs).
- [carry] **P2-S54-lightcheck-edition-faux-positif** 1/3 : carry S56.
- [carry] **P2-S54-windows-test-cfg-unix** 1/3 : carry S56.
- [carry] **P2-S54-test-E2E-multi-noeuds** 1/3 : carry S56.
- [carry] **P2-S54-rustfmt-drift-sessions** 1/3 : carry S56.
- [carry] **P2-S54-AUDIT-1 flaky browse test** 1/3 : carry S56.

### Carries residuels post-S55

| Item | Compteur S56 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 14+/3 | exemption |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-S53-outbox non-persistant | 3/3 **MANDATORY** | S53 review |
| P2-S53-browse_request rate-limit | 3/3 **MANDATORY** | S53 review |
| P2-S54-forbid-deny-doc | 2/3 | Phase A review |
| P2-S54-lightcheck-edition-faux-positif | 2/3 | Phase A review |
| P2-S54-windows-test-cfg-unix | 2/3 | Phase B review |
| P2-S54-test-E2E-multi-noeuds | 2/3 | Phase C review |
| P2-S54-rustfmt-drift-sessions | 2/3 | Phase D review |
| P2-S54-AUDIT-1 flaky browse test | 2/3 | S54 audit |

**Attention S56 pair** : outbox et browse_request passent a 3/3
MANDATORY. S56 devra les inclure dans le plan obligatoire + la
phase dette pair. 4 items passent a 2/3.

---

## §7 Scope cuts

1. **LT-7 cross-platform builds** — Tier 3 (MVP = x86_64-linux only)
2. **LT-7 toolchain bundle iroh-blobs** — Tier 3
3. **LT-7 auto-update reseau** — Tier 3
4. **LT-7 build log streaming** — Tier 3 (MVP = resultat final)
5. **LT-7 podman rootless sandbox** — Tier 2 post-MVP (S56+)
6. **Outbox persistant fichier** — S56 (> 500 LOC, 3/3 MANDATORY S56)
7. **Browse_request rate-limit** — S56 (> 500 LOC, 3/3 MANDATORY S56)
8. **Test E2E multi-noeuds automatise** — S56
9. **Windows test cfg(unix) CI** — S56
10. **forbid-deny-doc PATTERNS** — S56
11. **Lightcheck edition faux-positif** — S56
12. **rustfmt drift sessions** — S56
13. **Flaky browse test investigation** — S56
14. **Pre-v1.0 apps Protocol Explorer + Ideas Hub** — S56-S57
15. **LT-1 Kudos-v2 fairness reform** — S58+

---

## §8 Tracabilite scope (S54 → S55)

| S54 scope cut | S55 disposition |
|---|---|
| LT-7 self-hosted build foundation | **Phases B-C** |
| Test E2E multi-noeuds automatise | Scope cut reporte S56 |
| Outbox persistant fichier | Scope cut reporte S56 (3/3 MANDATORY) |
| Browse_request rate-limit per-peer | Scope cut reporte S56 (3/3 MANDATORY) |
| VPS TLS + nginx | **Phase A** (Caddy au lieu de nginx) |
| VPS monitoring + alerting | Scope cut S56+ |
| systemd service VPS | **Phase A** (woodpecker.service) |
| LT-1 Kudos-v2 fairness reform | Scope cut S58+ |
| Events SSE daemon-native | post-v1.0 |
| MCP server Rust | post-v1.0 |
| Pagination SQL-side LIMIT/OFFSET | S56+ |
| Test infra mk_state() refactoring | S56+ |

---

## §9 Risk register

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | VPS RAM insuffisante pour Woodpecker server + agent + builds Rust (~4-6GB peak) | Medium | High | 8GB VPS. Si memoire insuffisante : scope cut Phase B/C builds vers S56, garder server-only |
| R2 | GitHub OAuth app creation necessite intervention browser utilisateur | Certain | Low | Etape documentee, faisable en 5 min |
| R3 | Reproductibilite SHA256 echoue meme environnement (non-determinisme residuel Rust) | Medium | Medium | SOURCE_DATE_EPOCH + remap-path-prefix. MVP accepte faux-negatifs documentes |
| R4 | Build executor tmpdir pas assez isole (acces filesystem host) | Low | Medium | MVP documente la limitation. Podman rootless en Tier 2 (S56) |
| R5 | GHA run echoue sur tests flaky macOS/Ubuntu | Medium | Low | Documenter les flaky connus, re-run 2-3x |
| R6 | Quorum DB migration complexe (AwaitingQuorum status) | Low | Medium | Migration simple ALTER TABLE, status enum extension |

---

## §10 Audit gate pattern — rappel

Phase 0 S54 jouee (PASS `734da72`). Phase E produira
sprint56_audit_plan.md pour la session fraiche S56.

---

## §11 Checkpoint de validation

1. **D1** : Woodpecker server Docker Compose + Caddy TLS ?
   → oui (VPS prete, images pullees, cli installe)
2. **D2** : GHA validation via push + run documentation ?
   → oui (fix nexus-core-py deja commite, push simple)
3. **D3** : LT-7 build task routing + executor tmpdir MVP ?
   → oui (wire format pret, dispatcher/validator clean, design doc
   detaille SELF_HOSTED_BUILD.md)
4. **D4** : P2 batch 4 items quick ?
   → oui (tous < 10 LOC individuellement, zero design requis)
