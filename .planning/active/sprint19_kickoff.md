# Sprint 19 — Kickoff (PoW gossip + TLS pinning relays + DHT runtime wire + delayed upload queue)

**Ecrit** : 2026-04-16 (session fraiche post-S18 audit gate leve).
**Type** : **sprint implementation** (suite S18, chaine transport durci).
**Tip master d'entree** : `1a606a3` (chore(sprint18): audit-P3 batch
— buildType URI + parse_version warn + RADICLE casing).
**Phase 0 audit Sprint 18** : **DEJA JOUE** — findings dans
`.planning/archive/v1.2/sprint18_audit_findings.md` (migre via
`git mv` avec le 1er commit S19), verdict CONDITIONAL PASS leve
via 6 commits `677556f..1a606a3` (1 P1 + 4 P2 + batch P3). La
session fraiche qui demarre Sprint 19 Phase A verifie via
`git log` que le tip master courant >= `1a606a3` et ne rejoue pas
l'audit.

**Sources context7 + WebSearch a consulter en Phase A kickoff** :
- `rs_iroh` TLS / peer auth API (iroh 0.97 `RelayClient` +
  `RelayConfig` cert pinning primitives)
- `github.com/pubky/pkarr` relay self-hosting docker image
- Hashcash RFC + Bitcoin Lightning "invoice proof-of-work"
  references (difficulty adjust pattern)
- Tor/I2P delayed upload pattern (mixnet padding references)

---

## 1. Constat d'entree

### 1.1 D'ou on part

Sprint 18 a livre la baseline supply chain (cargo-deny + pip-audit
+ npm audit + wasmtime ban preemptif), les reproducible builds +
SLSA in-toto attestation, la **primitive** multi-relai federation
(`RelayMode::Custom` + 2 fallbacks) et la **primitive** DHT
pkarr quorum 2/3 (`dht_quorum::redundant_resolve` ~440 LOC +
13 tests), le wire TaskEntry coord-side complet (`is_open_source`
+ caps W/VRAM/h injectes AVANT signature canonical), la rotation
X-SBFB-Token **maintenant runtime-effective** (AuthState::Rotated
+ file-watcher tokens.json via D-1 audit fix `677556f`), le check
driver NVIDIA CVE au launcher startup, le warrant canary mensuel
Ed25519+JCS avec dead-man switch preserve, et le Codeberg prive
disaster-recovery mirror (pivot Radicle differe v1.0 avec flip
sequence self-contained MIRROR_FALLBACK.md §3).

**Gate 1 (DnD Forge beta fermee T0-T1) UNLOCKED effectivement**.

Sprint 19 est le **premier sprint apres Gate 1 unlock**. Son
mandat est clair : **durcir la chaine transport P2P** (TLS cert
pinning relais + PoW Hashcash gossip subscribe + delayed upload
queue anti-correlation) + **cabler au runtime** les primitives
livrees Phase S18 (DHT quorum wire browse aggregator) + **self-
hoster un pkarr relay** comme premier pas vers federation ONG
non solo-implementable S19+. Objectif : rendre l'eclipse-by-DHT
defense pleinement active, Sybil-resistance bootstrap minimale
via PoW, et anti-correlation traffic embryonnaire.

### 1.2 Ancrage HARDENING_ROADMAP §3 Sprint 19

La roadmap Phase D S17 specifie Sprint 19 items :

| Item | LOC roadmap | Source |
|---|---|---|
| PoW Hashcash per-gossip-subscribe (difficulty 2^18 ajustable) | ~400 | §3 S19 |
| TLS cert pinning relays (iroh upstream contrib) | ~200 | §3 S19 |
| Delayed upload queue (randomized 0-5min batching) | ~300 | §3 S19 |
| pkarr relay self-hosted (docker image + ops doc) | ~400 | §3 S19 |
| **Carry S18 C-1** Wire `redundant_resolve` browse aggregator + curator runtime | ~150 | S18 audit C-1 |

Total roadmap+carry : **~1450 LOC**. Plus **Meta-1 Radicle-v1.0
tracking** (zero-LOC, juste presence dans §3 items carry pour
resistance a la cloture S19).

**Gate unlock** fin S19 : Eclipse-by-DHT defense pleinement
active (wiring DHT quorum au runtime browse/curator). Pas un
Gate officiel HARDENING_ROADMAP §7 mais debloque S21 rate-limit
(prerequis Sybil-resistance minimale via PoW).

### 1.3 Le declencheur runtime wire DHT

Post-S18 audit finding C-1 (P2) : la primitive `dht_quorum::
redundant_resolve` est **livree + testee** (13 tests Rust) mais
**non-wirée en production**. Grep confirme que les call sites
existants `discovery.rs`, `nexus-shell-daemon-core` browse
aggregator, curator runtime utilisent encore des lookups pkarr
single-node. La promesse "Eclipse-by-DHT defense" de la kickoff
S18 §1.2 + HARDENING_ROADMAP §7 Gate 1 est **partiellement
inactive en runtime** — la primitive est la, le glue avec les
appelants iroh-relay 0.97 per-pkarr-relay lookup manque (~150 LOC).

Sprint 19 est le sprint naturel pour ce wiring : (a) le code
existe, (b) la primitive est testee, (c) le design est
deliberement generique sur le resolver (`QuorumResolver` trait),
(d) c'est la condition pour considerer Eclipse-by-DHT comme
"defense active" plutot que "defense armee mais non-engagee".

### 1.4 Compteurs de tests a l'entree (tip `1a606a3`)

| Suite | Count observe entree S19 |
|---|---|
| Rust workspace | 478 (473 tests + 5 doc-tests) |
| Python SDK | 183 |
| Python coordinator | 187 + 3 skipped |
| Python app-gov | 46 |
| Vitest unit | 239 |
| Playwright | 38 |
| size-limit | 7/7 |
| SPDX | 246+ |
| **Total** | **~1176 tests** |

**Delta Sprint 19 attendu : +45 a +55** (HARDENING_ROADMAP
projection : +45). Repartition estimee : +25 Rust (PoW primitive
+ TLS pinning + DHT wire + delayed queue), +10 Coordinator
(delayed upload integration), +5 Web (Playwright smoke PoW +
pkarr relay health), +5 ops (pkarr docker smoke).

### 1.5 Pre-launch protocol policy (rappel)

Sprint 18 a confirme la regle : `*_VERSION = 1` jusqu'au tag v1.0,
pas de tolerant decoder multi-version. Sprint 19 respecte : aucun
item liste ci-dessus ne touche un wire format. PoW Hashcash est
un **nouveau champ optionnel** dans gossip subscribe messages
(`#[serde(default)]` legitime pour runtime robustness si un
publisher S18-era envoie sans). TLS pinning est purement local
config. Delayed upload queue est pipeline interne, pas de wire.
DHT quorum wire change le code path lookup, pas le format.

---

## 2. Goal en une phrase

**Le projet durcit la chaine transport P2P en imposant un cost-
of-identity minimal via PoW Hashcash sur chaque gossip subscribe
(difficulty 2^18 ajustable per-relai), en pinnant les certificats
TLS des relais iroh (contrib upstream si possible), en retardant
randomly 0-5 minutes les publications pour empecher la correlation
traffic, en self-hostant un premier pkarr relay docker (premier
pas vers federation ONG), et en cablant au runtime la primitive
DHT quorum 2/3 S18 dans le browse aggregator + curator runtime
pour activer pleinement la defense Eclipse-by-DHT — debloquant
S21 rate-limit per-consumer fin S21 comme brique Gate 2.**

---

## 3. Phase 0 — Audit Sprint 18 (DEJA JOUE — verdict CONDITIONAL PASS → LEVE)

**Status** : JOUE session 2026-04-15 (~2h30, post-`4453bfd`
wrap-up). Ne pas rejouer. Cf.
`.planning/archive/v1.2/sprint18_audit_findings.md` (migre avec
ce 1er commit S19).

**Commit stack du gate (leve)** :

```
1a606a3 chore(sprint18): audit-P3 batch — buildType URI + parse_version warn + RADICLE casing
e223ec7 fix(sprint18): audit-P2 C-1 — clarify DHT quorum primitive vs runtime wire status
6fe2dce fix(sprint18): audit-P2 B-1 — add wheel SLSA attestation to release matrix
9661485 fix(sprint18): audit-P2 A-1 — drop --workspace from cargo-deny job
0fb8458 fix(sprint18): audit-P2 F-1 + F-2 — resolve docs hygiene discrepancies
677556f fix(sprint18): audit-P1 D-1 — wire TokenRotator into shell-daemon HTTP router
```

6 commits ont ferme le verdict CONDITIONAL PASS (0 P0 + 1 P1 + 5
P2 + 6 P3) :
- **D-1 (P1)** : wire TokenRotator via `AuthState::Rotated(Arc<
  RwLock<TokenRotator>>)` + `notify` file-watcher sur
  `tokens.json` (pattern S16 ConsentWatcher). Rotation 24h passe
  de primitive livree a effective au runtime. +4 tests Rust
  (Rotated accepts current+previous, post-overlap reject, Static
  non-regression, file-watcher tokens.json reload).
- **F-1+F-2 (P2)** : 4 docs hygiene discrepancies (phase_E1_
  review presence + file count 9→10 + "5 reviews" → "6 reviews"
  omit E1 + placeholders `<wrap-up>`/`<A>`/`<this>` resolus
  aux SHAs reels).
- **A-1 (P2)** : drop `arg: --workspace` cargo-deny (default
  depuis v0.14+, rejete par versions modernes type 0.19.2).
- **B-1 (P2)** : wheel `nexus-core-py` ajoute a la matrix
  `release.yml` avec attestation SLSA in-toto (parite worker/
  daemon/launcher).
- **C-1 (P2)** : clarifier `verification.md §Gate 1` que DHT
  quorum est livre comme **primitive prete, runtime wiring
  S19+**. Pas de wire au browse aggregator ce sprint
  (carry-over **inclus dans ce kickoff comme Phase A**).
- **P3 batch (1a606a3)** : 3 nits cosmetiques cumules (B-2 build-
  Type URI standard SLSA `slsa.dev/build-type/custom` vs
  `container-based-build` non-applicable, E1-1 `parse_version`
  log warn sur segment non-numerique, E3-1 RADICLE_PROJECT_NAME
  casing align `sbfb` lowercase).

**Verdict final** : **PASS**. Sprint 19 Phase A non-bloque.

**Dette heritee Sprint 18 confirmee** :
- **C-1 P2 carry** : wire `redundant_resolve` au browse
  aggregator + curator runtime. Design generique sur
  `QuorumResolver` trait, wiring estime ~150 LOC + 5 tests.
  **Phase A S19 item 5**.
- **Meta-1 P2 carry** : item Radicle-v1.0 activation tracking.
  Owner : FlowUP. Deadline : jour du tag v1.0 (probablement
  sprint release v1.0). Runbook : `docs/release/MIRROR_FALLBACK.
  md §3.1-3.8` self-contained (5 secrets GHA + workflow YAML +
  rotation procedure). **Note au §6 items carry ci-dessous**.

**Tech debt S18 loggee dans PATTERNS.md** (P3 non-fixes
inline) :
- E1-1 a ete fixe `1a606a3` (warn log) — pas tech debt restante.
- E3-2 P3 `actions/checkout@v4` non-pin SHA — reporte sprint
  security ops futur (quand pin SHA policy etend aux 4 workflows
  GHA en une fois).

---

## 4. Decisions Day 0 (D1..D5)

### D1 — Perimetre items + ordre des phases

**Retenu** : 5 items (4 roadmap + 1 carry S18). Regroupes en 5
phases A-E + F wrap-up selon couplage technique et risque.

| Phase | Items couverts | Rationale regroupement |
|---|---|---|
| A — DHT quorum runtime wire | Carry S18 C-1 | Quick win (~150 LOC + 5 tests), design prete, livrable day 1, active Eclipse-by-DHT defense |
| B — PoW Hashcash gossip subscribe | Item 1 | Primitive crypto + integration iroh-gossip 0.97 subscribe path, testable isole |
| C — TLS cert pinning relays | Item 2 | Touche iroh relay client config (potentiellement contrib upstream), smoke test contre n0 relays |
| D — Delayed upload queue | Item 3 | Pipeline coord-side (publish.rs + task submit), independant transport |
| E — pkarr relay self-hosted | Item 4 | Ops docker image + doc deploy, zero code Rust dans SBFB repo (infra-as-docs) |
| F — Verification + audit plan S20 | consolidation + migration PARA | Fin de sprint standard |

**Rejete** :

- **Ordre alternatif "PoW first"** : Phase B bloquerait Phase A
  qui est un quick win. On commence par le livrable le plus rapide
  (A wire DHT) pour lander un signal positif early + desactiver
  la dette C-1 des j1.
- **Fusion B+C "1 phase transport durci"** : B touche gossip
  primitive (crypto), C touche relay config (TLS). Tests et risk
  profiles distincts, livraison atomique par phase = meilleur
  pattern debugging.
- **Phase E pkarr relay dans 1 phase separee** : zero code Rust
  dans repo (juste docker + doc), mais touche infrastructure
  ops (registry image, CI build-docker). Justifiable phase
  distincte pour tracabilite.

**Rationale ordre A→F** : (a) wire DHT d'abord (quick win +
active defense existante), (b) PoW prerequis S21 rate-limit
(premier item HARDENING_ROADMAP §3 S19), (c) TLS pinning apres
PoW pour eviter conflicts sur meme iroh config, (d) delayed
upload independant (peut paralleliser si besoin mais pour
simplicite lineaire), (e) pkarr relay last (infra-as-docs sans
code Rust, bon item de fin).

### D2 — PoW Hashcash difficulty initial

**Retenu** : difficulty **2^18** (~262144 hashes, ~100ms CPU
moderne 2026 sur Hashcash SHA256 single-threaded) comme
**baseline**, **ajustable per-relai** via config
`relay_pow_policy.toml` (format TOML, pattern existant SBFB).

Formula challenge-response : `HashPuzzle(subscribe_topic +
nonce + timestamp) < target(difficulty)`.

**Rejete** :

- **Difficulty 2^20** (~1M hashes, ~400ms) : trop cher pour un
  noeud mobile ou raspberry pi (proverbial T0/T1 adversary vs
  T4 maintainer-ops — le target est le **spammer botnet**, pas
  le noeud legit). 2^18 est le compromis documente dans PoW
  Hashcash literature (spec RFC 6110 reference + Tor rend-
  point PoW 2023).
- **Difficulty 2^16** (~65K hashes, ~25ms) : trop bas, un botnet
  peut encore flood raisonnablement. 2^18 est le seuil ou un
  attacker a 10k bots Raspberry-Pi-equiv doit passer ~15min par
  subscribe burst (rate-limit emergent).

**Source** : Tor PoW rendez-vous point 2023, Lightning Network
invoice PoW, RFC 6110 Hashcash anti-spam. **Phase B plan
detaillera** l'implementation SHA256 via `sha2` crate (deja dep).

### D3 — TLS pinning strategy

**Retenu** : pin le **public key SPKI hash** (pattern OWASP +
HPKP deprecated mais concept valide) stocke dans
`~/.sbfb/relay-pins.json` (pattern config partage avec
`relays.json` S18), verifie cote client iroh avant handshake
WebSocket.

Format pin entry :
```json
{
  "relay_url": "https://relay.iroh.network",
  "spki_sha256": "base64-hex-...",
  "added_at": "2026-04-16",
  "source": "bootstrap" // ou "user-override"
}
```

Contrib iroh upstream **optionnelle** : si iroh 0.97 expose deja
`RelayClient::with_cert_validator(custom)`, on wrap. Sinon, on
forke le connect path en S19 et on open PR upstream pour S20+
(aligne avec VALIDATED_BLUEPRINT couche 3 "transport anonyme").

**Rejete** :

- **Pin full certificate** : rotation cert = full break pour tous
  les clients = UX cassee lors de Let's Encrypt renew. Pin SPKI
  survive Let's Encrypt renew car le key reste le meme.
- **Pin CA chain uniquement** : sans pin SPKI, une CA compromise
  (pas improbable 2026, WebPKI sous pression) casse l'auth.
- **Skipper TLS pinning S19 et attendre S20** : mais HARDENING_
  ROADMAP §3 S19 liste item explicitement, et S20 est encryption
  at rest big-rock (deja plein). Pin SPKI est quick-win ~200 LOC.

**Source** : OWASP Cheat Sheet Pinning + Chromium security model
post-HPKP-deprecation (pattern Tor Browser verify-cert helper).

### D4 — Delayed upload queue range

**Retenu** : queue **0-5 minutes batch window** avec
**randomized jitter exponential** distribution (median ~90s,
tail 5min). Batching window interne : toutes les 30s, flush les
messages dont le jitter est ecoule.

Trigger : `POST /project/task/submit` → push queue avec
`delivery_at = now + exponential_random(mean=90s, max=300s)`.
Scheduler interne coord-side pop et emit vers iroh-gossip.

**Rejete** :

- **Range 0-30 minutes** (Tor-style rendez-vous) : trop long
  pour UX live (DnD Forge users attendent task response en <2min
  mediane). 0-5min est le compromis publish-anonymity vs
  interactive.
- **Range 0-60s** : insuffisant pour anti-correlation (un
  observer reseau peut correler 60s facilement). 5min minimum
  tail.
- **No batching (immediate flush)** : casse l'entire point de
  la queue (0 delay = 0 anti-correlation).

**Source** : Tor Circuit Padding spec + VALIDATED_BLUEPRINT
couche 10 opsec "publish delay randomization" reference Mullvad
DAITA traffic shaping.

### D5 — pkarr relay self-hosted image

**Retenu** : **Docker image** `ghcr.io/SBFB50/pkarr-relay:v1.0`
build depuis pkarr/server upstream `Dockerfile`, tags
sem-version suivent pkarr upstream. Ops doc
`docs/release/PKARR_RELAY_OPS.md` couvre : deploy Hetzner CX11
(ou equivalent ~5 EUR/mois), `systemd` service unit, reverse
proxy nginx + Let's Encrypt, volume persistence (`pkarr.db`),
smoke-test `pkarr-cli publish/resolve` post-deploy.

Le relay n'est **pas** deploye dans ce sprint (c'est ops, pas
code). Ce sprint livre **l'image docker packagee + la doc
deploy**, pour qu'un maintainer (FlowUP ou contributeur) puisse
spin up un relai en ~30min.

**Rejete** :

- **Binary distribution seul** (sans docker) : moins portable,
  plus friction deploy. Docker = standard 2026 ops.
- **k8s manifest** : overkill pour 1 relay solo-ops. k8s viendra
  si/quand S25+ federation ONG multi-cloud.
- **Deploy le relai dans le sprint** : non, sprint implementation
  code + doc ops. Deploiement reel est decision user separe
  (cout hosting + ops energy).

**Source** : pkarr upstream Dockerfile pattern + VALIDATED_
BLUEPRINT couche 4 "overlay DHT" mentions pkarr federation
S19-S22 sequence.

---

## 5. Plan Phase outline

### Phase 0 — Audit Sprint 18 (DEJA JOUE, verdict PASS)

Migration `sprint18_audit_findings.md` + `sprint18_phase_F_
review.md` → `archive/v1.2/` via `git mv` dans le 1er commit
S19 (pattern `f75b2c6` S17 open).

### Phase A — DHT quorum runtime wire + carry S18 C-1 (~150 LOC, +5 tests)

**Scope** :
- `crates/nexus-shell-daemon-core/src/browse.rs` : remplacer
  lookup single pkarr par `dht_quorum::redundant_resolve` avec
  3 resolvers pkarr paralleles (n0-relay-1/2/3 via `RelayMap`
  S18 federation)
- `crates/nexus-core-rs/src/curator.rs` (ou equivalent curator
  runtime) : meme wire
- Tests integration : 2 scenarios happy (2/3 agree → accept) +
  degraded (1/3 agree → NoMajority + log warn), 3 tests unit
  wire
- Doc `docs/rust/PATTERNS.md` : section "DHT quorum wire"
  (pattern "primitive + wire separation", ref S18 audit C-1)

**Livrable commit** : `feat(sprint19): Phase A — DHT quorum
runtime wire (browse aggregator + curator)`

### Phase B — PoW Hashcash gossip subscribe (~400 LOC, +15 tests)

**Scope** :
- `crates/nexus-core-rs/src/pow.rs` : primitive SHA256 Hashcash
  `solve_challenge(topic, nonce, difficulty) -> Proof` +
  `verify_challenge(proof, difficulty) -> bool`, deterministe
  single-threaded (CI-friendly)
- Integration gossip subscribe : `GossipClient::subscribe_with_
  pow(topic, difficulty)` wrap le path subscribe existant
- `relay_pow_policy.toml` loader (format similar `relays.json`
  S18) + default policy `{"default_difficulty": 262144, "relay_
  overrides": {}}`
- Tests Rust : 10 tests primitive (solve+verify + edge cases
  nonce overflow / difficulty 0 / difficulty max) + 5 tests
  integration subscribe (happy path + reject on invalid proof
  + fallback no-pow if relay policy omits)
- Bench `cargo bench --bench pow` : verify 2^18 difficulty =
  <200ms sur CPU moderne (warn si >500ms, ajust difficulty S21+)

**Livrable commit** : `feat(sprint19): Phase B — PoW Hashcash
gossip subscribe (difficulty 2^18 per-relai)`

### Phase C — TLS cert pinning relays (~200 LOC, +8 tests)

**Scope** :
- `crates/nexus-core-rs/src/tls_pinning.rs` : SPKI hash extract
  from PEM/DER cert + `PinValidator` struct + `validate(cert,
  pinset) -> Result`
- Integration iroh relay client : wrap `RelayClient::builder()`
  avec custom TLS validator (si iroh 0.97 expose hook ; sinon
  fork connect path + TODO upstream PR)
- `~/.sbfb/relay-pins.json` loader + bootstrap default pins
  (SPKI hash des 3 relais n0 connus au moment S19 kickoff,
  fetched via `openssl s_client -connect` documented in doc)
- Tests : 5 tests primitive (extract SPKI + validate match/
  mismatch + empty pinset fail-close), 3 tests integration
  (connect ok / rejet / user-override)
- Doc `docs/rust/PATTERNS.md` : section "TLS cert pinning" +
  rotation procedure (quand n0 roll cert) + CONTRIBUTING.md
  note "PR upstream iroh si custom validator non-expose"

**Livrable commit** : `feat(sprint19): Phase C — TLS cert
pinning relays (SPKI hash validate)`

### Phase D — Delayed upload queue (~300 LOC, +10 tests)

**Scope** :
- `packages/nexus-coordinator/src/nexus_coordinator/upload_
  queue.py` : async queue + `schedule(task, jitter_max=300s)`
  method + internal scheduler 30s flush loop
- Integration `api/tasks.py` : `POST /project/task/submit` pipe
  dans queue au lieu de direct gossip emit
- Tests : 8 tests pytest (happy path jitter range, max 5min
  respect, scheduler flush 30s granularity, queue persistence
  on coord restart, concurrent submit thread-safe), 2 tests
  integration full-loop (submit + poll task status apres delay)
- Metric log : `upload_queue_delay_seconds` histogram (pas
  Prometheus obligatoire, juste log INFO pour diagnostic S20+)
- Doc `docs/shell/PATTERNS.md` : section "Delayed upload queue"
  + rationale anti-correlation + UX trade-off (DnD Forge response
  latency expected +30-90s mediane)

**Livrable commit** : `feat(sprint19): Phase D — delayed upload
queue (0-5min exponential jitter)`

### Phase E — pkarr relay self-hosted image (~400 LOC equivalent docker + doc)

**Scope** :
- `docker/pkarr-relay/Dockerfile` (~30 LOC) : base sur pkarr
  upstream Dockerfile, tag `FROM rust:1.94-slim AS builder ...`,
  healthcheck exposed
- `.github/workflows/build-pkarr-image.yml` (~50 LOC) : trigger
  push sur `master` + tag `v*`, push vers
  `ghcr.io/SBFB50/pkarr-relay:<version>`, scan Trivy inline,
  permissions `packages: write`
- `docs/release/PKARR_RELAY_OPS.md` (~250 lignes) : §1 rationale,
  §2 provisioning Hetzner CX11 (commands copy-paste), §3
  systemd unit template, §4 nginx reverse proxy + Let's Encrypt,
  §5 smoke test `pkarr-cli publish/resolve`, §6 monitoring
  baseline (disk + network logs), §7 rotation SPKI cert (cross-
  ref S19 Phase C TLS pinning)
- Pas de code Rust dans ce repo — sauf un smoke test ops
  `tests/ci-smoke/pkarr-relay-healthcheck.sh` (~20 LOC bash)
  qui curl le `/healthz` endpoint docker local + asserts
  response
- Tests ops : 1 test CI (docker build succeeds sur Linux amd64),
  pas de test runtime (deploy real = user-driven ops)

**Livrable commit** : `feat(sprint19): Phase E — pkarr relay
self-hosted docker image + ops doc`

### Phase F — Consolidation + verification + audit plan S20 (~250 LOC docs)

**Scope** :
- Update `CLAUDE.md §Etat actuel` : Sprint 19 CLOSED + status
  Eclipse-by-DHT defense active + commits stack
- Update `docs/claude/SPRINT_LOG.md` : row S19 v1.2
- Update memory `nexus_grid_pivot.md` frontmatter description
- `.planning/active/sprint19_verification.md` : checklist
  fail-fast (CI green, 1176+45 tests, scope respecte)
- `.planning/active/sprint19_audit_plan.md` : tracks A-E +
  meta-track Radicle-v1.0 tracking **re-carried** pour Sprint 20
  Phase 0
- Migration planning `.planning/active/sprint19_*.md` →
  `.planning/archive/v1.2/` dans le wrap-up commit

**Livrable commit** : `chore(sprint19): Phase F — wrap-up +
verification + audit plan S20 + migrate planning`

---

## 6. Scope cuts (PAS dans ce sprint)

**Encryption at rest keypair** : Sprint 20 big-rock
(`HARDENING_ROADMAP §3 S20`). Keychain/DPAPI wrapping. Gate 2
prerequis.

**Duress PIN + panic wipe** : Sprint 20 (`§3 S20`).

**Rate-limit sliding-window per-consumer** : Sprint 21 (`§3 S21`).
Depend S19 PoW Hashcash (sans Sybil-resistance minimale, rate-
limit contournable).

**Kudos-weighted gossip admission** : Sprint 22 (`§3 S22`).
Depend S19 PoW + S21 rate-limit.

**Structured output llama.cpp grammar** : Sprint 20 (`§3 S20`).

**Client-side redaction SDK + output filter** : Sprint 21
(`§3 S21`).

**Federated ONG-run pkarr relays concrets** : Sprint 22+ outreach
(non solo-implementable, necessite partnership Amnesty/HRW). S19
livre juste l'image docker pour que SBFB ops ou un contributeur
self-hoste un relai single.

**ML-DSA-65 + ML-KEM-1024 hybrid (PQC migration)** : Sprint 26+
(`VALIDATED_BLUEPRINT couche 1`). Ed25519 acceptable jusque-la.

**Domain fronting + Tor bridges** : Sprint 24-25
(`§3 S24-25`). Depend S19 multi-relai + TLS pinning mature.

**`actions/checkout@v4` pin SHA (S18 E3-2 P3)** : reporte sprint
security ops futur, pattern etendu aux 4 workflows GHA en une
fois.

**Wheel `nexus-core-py` attestation SLSA** : DEJA FIXE S18
`6fe2dce` B-1. Non-carry.

---

## 7. Tracabilite scope

Items **nouveaux Sprint 19** :
- PoW Hashcash primitive + gossip integration (nouveau)
- TLS cert pinning SPKI (nouveau)
- Delayed upload queue (nouveau)
- pkarr relay docker image + ops doc (nouveau)

Items **carry/dette** :
- DHT quorum runtime wire (carry S18 C-1 P2)
- Meta-1 Radicle-v1.0 activation tracking (carry S18 Meta-1 P2,
  re-carry S20 si v1.0 pas S20)

Items **differes** :
- Rate-limit per-consumer → S21
- Encryption at rest → S20
- Kudos-weighted admission → S22
- Federated ONG pkarr → S22+ (partnership)
- `actions/checkout@v4` pin SHA → sprint ops futur

---

## 8. Audit gate pattern — rappel

Phase 0 Sprint 18 audit joue pre-S19 session 2026-04-15, verdict
CONDITIONAL PASS leve apres 6 commits `677556f..1a606a3`. Phase F
S19 produit `sprint19_audit_plan.md` pour Sprint 20 Phase 0.
Pattern permanent depuis Sprint 7.

Meta-1 Radicle-v1.0 tracking re-carried dans `sprint19_audit_
plan.md` explicitement (cf. §5 Phase F scope) pour eviter perte
apres cloture S19.

---

## 9. Estimations LOC

| Phase | LOC code | LOC tests | LOC docs | Total |
|---|---|---|---|---|
| 0 — Audit S18 | 0 | 0 | 0 (migre existant) | 0 |
| A — DHT quorum wire | ~100 | ~50 | ~20 | ~170 |
| B — PoW Hashcash gossip | ~280 | ~120 | ~30 | ~430 |
| C — TLS cert pinning | ~140 | ~60 | ~20 | ~220 |
| D — Delayed upload queue | ~200 | ~80 | ~30 | ~310 |
| E — pkarr relay docker+doc | ~50 (Dockerfile+YAML) | ~20 (smoke bash) | ~250 | ~320 |
| F — Consolidation + verif + audit plan | 0 | 0 | ~250 | ~250 |
| **Total** | **~770** | **~330** | **~600** | **~1700** |

**Delta tests** : +45 (Rust ~30, coord ~10, ops smoke ~5).
Compteur final estime : **~1221 tests** (1176 + 45).

LOC total (~1700) > roadmap estimate (~1450) : ecart vient de
(a) tests volume (roadmap compte ~45 tests = ~135 LOC vs plan
estime ~330 LOC incluant fixtures et setup), (b) docs pkarr ops
non-comptees par roadmap, (c) subphase decoupage Phase A carry
S18 (150 LOC dans roadmap S18 non-comptee dans S19 estimate
original).

---

## 10. Checkpoint de validation

Status : **draft**, a discuter avant Phase A si besoin. Hypothese
retenue : l'utilisateur demande cadre standard S19 HARDENING_
ROADMAP + carry S18 C-1. Si ecart constate Phase A kickoff
session fraiche, re-ouvrir discussion Day-0.

Points de validation souhaitables (non-bloquants si user confirme
l'approche "autonome") :

1. **D2 PoW Hashcash difficulty 2^18 initial** : OK baseline ou
   preference 2^16 (permissif) / 2^20 (restrictif) ?
2. **D3 TLS pinning SPKI hash (pattern HPKP-concept) vs full-cert
   pin** : OK SPKI ou prefere full-cert ?
3. **D4 Delayed upload queue range 0-5min** : OK ou prefere 0-15min
   (plus anti-correlation, UX degradee) / 0-2min (UX preservee,
   anti-correlation faible) ?
4. **D5 pkarr relay docker uniquement (no real deploy ce sprint)**
   : OK livrable image + doc S19 et deploy real reporte S20 ou
   prefere deploy real inclus ce sprint ?
5. **Ordre des phases A→F** : OK DHT wire d'abord (quick win) ou
   prefere PoW first (item P0 roadmap) ?
6. **Fichiers untracked root** (`cc.json`, `test_libc.exe/pdb`,
   `site/`, `node_modules/`, `docs/DND_P2P_DESIGN.md`,
   `docs/VISION_USE_CASES.md`, `docs/apps/`) : decision par defaut
   **laisser traine S19** (hors scope transport hardening, non-
   bloquant). Sprint futur dedie docs apps les integrera.

---

**Note de placement** : ce kickoff est ecrit directement dans
`.planning/active/` (Sprint 18 deja migre archive/v1.2/ via
wrap-up `4453bfd` + audit findings + phase F review migres avec
le 1er commit S19). Le seul mouvement S19-day-1 est `git mv
sprint18_audit_findings.md sprint18_phase_F_review.md →
archive/v1.2/` (**deja fait** — staged pour inclusion dans le
1er commit chore(planning) S19).
