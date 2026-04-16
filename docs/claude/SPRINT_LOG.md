# Sprint log — historique cross-version

Index synthetique de tous les sprints livres. Une ligne par sprint.
Detail des decisions, plans et verifications dans
[`.planning/archive/v{X}/sprint{N}_*.md`](../../.planning/archive/).

Pour la methodologie sprint elle-meme (lifecycle, audit gate,
conventions commit), voir [`README.md`](README.md).

Pour le sprint en cours, voir
[`.planning/active/`](../../.planning/active/).

---

## v1.2 — Security hardening (en cours)

| Sprint | Etat | Tip cloture | Nb commits | Docs |
|---|---|---|---|---|
| 16 | DONE + CONDITIONAL PASS levé | `d18e19e` (gate close landed) | 6 (Phase 0 gate + A-D + docs) + 7 (findings + C3 + D1 + C1/C2 + chore protocol + C4 + log update) | 6 docs (kickoff, plan, verification, audit_plan, audit_findings) + docs/security/ (README + THREAT_MODEL + RUNTIME_ISOLATION) |
| 17 | DONE + scope-cut Phase E acte | `<wrap-up>` (close + scope-cut + migrate) | 6 (A `297fd50` + B `c275ebd` + C `7dea299` + D `872f48a` + BLUEPRINT bonus `721686c` + F wrap-up) | 6 docs security (`ADVERSARIES.md` + 6 fiches T0-T5 dir `adversaries/` + `ATTACK_SCENARIOS.md` + `P2P_THREATS.md` + `COMPUTE_THREATS.md` + `HARDENING_ROADMAP.md` + `VALIDATED_BLUEPRINT.md`) + planning (kickoff/plan/verification/audit_plan) migre archive/v1.2/ |
| 18 | DONE + Gate 1 UNLOCKED + pivot E3 Radicle→Codeberg + audit gate S18 leve via 6 commits `677556f..1a606a3` (1 P1 + 4 P2 + P3 batch) | `4453bfd` (wrap-up) → `1a606a3` (audit gate close) | 8 + 6 audit fixes (A supply-chain + B `4ab0211` repro + C `9d0ad7a` multi-relai + D `94cccb2` wire+token + E1 `9f4d19f` driver + E2 `04c9621` canary + E3 `95807b1` Codeberg mirror + F `4453bfd` wrap-up + audit-P1 `677556f` D-1 wire TokenRotator + audit-P2 `0fb8458` F-1/F-2 docs hygiene + `9661485` A-1 drop `--workspace` cargo-deny + `6fe2dce` B-1 wheel SLSA attestation + `e223ec7` C-1 DHT quorum primitive-only clarification + audit-P3 `1a606a3` batch buildType URI + parse_version warn + RADICLE casing) | 11 docs planning (kickoff, plan, verification, audit_plan, audit_findings, **7 phase reviews B/C/D/E1/E2/E3 + F wrap-up review**) + `docs/release/MIRROR_FALLBACK.md` + `CANARY.txt` bootstrap + `scripts/verify-canary.sh` + `docs/release/REPRODUCIBLE_BUILDS.md` + `.github/workflows/` (supply-chain + canary-monthly + mirror-codeberg) — tous migres archive/v1.2/ |
| 19 | DONE + Eclipse-by-DHT defense runtime-active sous config opt-in `SBFB_PKARR_RELAYS` (canary, enforcement strict post-Gate 2) + carry Meta-1 Radicle-v1.0 re-carry S20 | `619059b` (Phase F) | 5 feat + 1 fix (Phase B follow-up) + 2 chore planning + 2 chore tooling G4 + 1 wrap-up (A `ab6985c` DHT quorum wire + B `edfc51b` PoW Hashcash + B follow-up `08f4e41` Cargo deps + canonical + lib + PATTERNS + C `540bb51` TLS SPKI pinning + D `f238d31` delayed upload queue + E `2fd4d72` pkarr docker image + F `619059b` wrap-up + chore planning `fe0a8fd` guardrails G1..G7 + chore planning `2fd6c60` Phase D wrap review + chore tooling `4216436` phase-auditor Write G4 + chore tooling `c609a03` TOOLING.md G4) | 11 docs planning (kickoff, plan, verification, audit_plan, supervision_log + 6 phase reviews A/B/C/D/E/F — F review carry active->archive dans commit ulterieur pattern S18) + `crates/nexus-core-rs/src/pow.rs` + `tls_pinning.rs` + `relay_test_cert.pem` fixture + `packages/nexus-coordinator/src/nexus_coordinator/upload_queue.py` + `docker/pkarr-relay/Dockerfile` + `.github/workflows/build-pkarr-image.yml` + `docs/release/PKARR_RELAY_OPS.md` §1-§7 — tous migres archive/v1.2/ sauf F review |

**Faits saillants** :

- **Sprint 16** : loopback passe de `D` a `A-` via defense en
  profondeur — X-SBFB-Token 256-bit (`d7c265a`, launcher
  genere, perm 0600) + Host allowlist + Origin check
  (mitigation CVE-2025-49596 Anthropic MCP Inspector DNS
  rebinding, CVSS 9.4). UDS avec SO_PEERCRED (pattern Tailscale
  safesocket) + Named Pipes Windows avec DACL user-only via
  SDDL (`1cfde89`). GPU consent dialog 4 niveaux + whitelist L3
  manuelle + raccourci "Contribuer mon GPU" depuis Browse +
  caps W/VRAM/heures enforced worker-side via
  `should_accept_task` + `ConsentWatcher` (notify crate, 50 ms
  debounce) + usage.json daily counter reset minuit-local
  (`3247e88`). ProjectAnnouncement avec `is_open_source`
  derive automatiquement par le coordinator (true pour
  deploy-from-repo, false pour zip prive, non-user-settable
  pattern npm provenance/cosign) (`10bbc63`). Threat model
  STRIDE + LINDDUN livre dans `docs/security/` avec roadmap
  runtime isolation WSL2 / Virtualization.framework /
  systemd-nspawn pour Sprint 17+.

  **Audit gate joue en Phase 0 Sprint 17** (`0230589` findings) :
  verdict CONDITIONAL PASS avec 4 P1 identifies et fermes :
  - `795ebe9` C-3 : consent watcher fail-closed sur RwLock
    poisoned (ajout `continue;` dans Err arm)
  - `87cae71` D-1 : daemon `POST /publish` reject `is_open_source=true`
    sans chaine provenance (bypass non-user-settable casse par
    bearer-holder local)
  - `1aa6fed` C-1/C-2 : wire `is_open_source` + `estimated_watts`
    + `estimated_vram_mb` + `estimated_hours` end-to-end via
    Task canonical schema + runtime.rs lit directement
  - `d1e6971` chore(protocol) : drop pre-launch backward-compat
    scaffolding (PA_VERSION 5→1, decoder v==1 only, 6 tests
    zombies supprimes). Politique pre-launch codifiee dans
    CLAUDE.md §Pre-launch protocol policy.
  - `8e6fa35` C-4 : watcher preserve state sur `consent.json`
    remove (log + garde in-memory au lieu de silent revert L1).

  Compteurs finals : 430 Rust / 187 coord / 183 SDK / 46 gov /
  239 vitest / 38 Playwright / 7/7 size / 246+ SPDX (~1128 tests).

Detail : [`.planning/archive/v1.2/`](../../.planning/archive/v1.2/).

- **Sprint 17** : sprint **recherche pure** livre (0 code, ~4823
  LOC docs). Phase A `297fd50` taxonomie T0-T5 + 12 attack
  scenarios + 6 fiches adversaires. Phase B `c275ebd` P2P attack
  surface 7 vecteurs (Sybil/Eclipse/gossip/DHT/BGP/traffic/ISP).
  Phase C `7dea299` GPU compute-sharing 7 classes menace
  (prompt leak / spoof / theft / extract / inject / side-channel /
  DoS). Phase D `872f48a` hardening roadmap (matrix 27 threats
  + framework scoring I×L/E + roadmap Sprint 18-30 sequencee
  + quick-wins + big-rocks + dependency graph + gates 1-4
  unlocking). Commit bonus `721686c` `VALIDATED_BLUEPRINT.md`
  : design long-terme 13 couches (host / identity / transport /
  overlay / sybil / storage / compute / runtime / deploy / trust /
  opsec / formal-verif / research) avec chaque brique OSS validee
  contre docs 2026 + advisories + CVE via WebSearch + context7
  MCP (50+ briques, 8 ajoutes, 9 retirees, 3 zones rouges
  documentees : wasmtime 12 CVE avril 2026, libp2p-gossipsub
  CVE-2026-33040/34219, libcrux semantic gaps Symbolic Software
  7 avril 2026). Phase E `RELEASE_GATES.md` + `PARTNERSHIPS.md`
  + `DISCLOSURE.md` (~750 LOC) **scope-cut officialise** :
  redondance partielle avec BLUEPRINT (gates + partnerships +
  disclosure pattern couverts), items Phase E restants
  ONG-facing (enforcement formel, outreach templates, SLA CVE
  workflow, audit vendor couts negocies) reportes a sprint
  OpSec dedie futur quand fondation multi-juridiction en place.
  Phase F wrap-up livre verification + audit plan S18 + updates
  CLAUDE.md + SPRINT_LOG + memory + migration planning
  active -> archive/v1.2/. Pattern "recherche avant code"
  (Zcash / Signal / Briar / Tor). Phase 0 audit S16 deja joue
  pre-S17, verdict PASS, tip entree `d18e19e`.

  Position post-S17 vs OSS state-of-the-art 2026 (documente dans
  `VALIDATED_BLUEPRINT.md`) : **match** sur crypto primitives
  (aws-lc-rs FIPS 140-3), PQC hybride (Signal PQXDH 2024),
  app sandbox (Shopify/Fastly via Wasmtime), supply chain
  (Mozilla/Kubernetes), transport anonyme (Briar via Arti embed),
  at-rest + duress (Briar + VeraCrypt), formal verif critical
  path (Signal Triple Ratchet SPQR), fuzzing (Mozilla/Google),
  TEE attestation (Confidential Computing Consortium), traffic
  shaping (Mullvad DAITA). **Structurellement superieur** sur
  memory safety (Rust entier vs Signal Java+C / Tor C / SecureDrop
  Python). **Leader unique OSS** sur 3 dimensions :
  compute-sharing defense-in-depth (7 classes adressees),
  verified P2P app deploy (multi-builder + SLSA L3+), runtime
  GPU consent per-task. Tests inchanges ~1128 (sprint recherche
  pure).

- **Sprint 18** : quick wins + supply chain baseline + multi-relai
  phase 1 + Gate 1 unlock. Phase A supply-chain CI (cargo-deny +
  pip-audit + npm audit + wasmtime pin 43.0.1+ D2 contre 12 CVE
  avril 2026), Phase B `4ab0211` reproducible builds (`--locked`
  + `SOURCE_DATE_EPOCH` + SHA256 SLSA in-toto attestation Ed25519),
  Phase C `9d0ad7a` multi-relai federation iroh `RelayMode::Custom`
  n0 + 2 fallbacks round-robin + DHT pkarr 3-paralleles quorum 2/3,
  Phase D `94cccb2` coord-side TaskEntry wire (`is_open_source` +
  estimated_watts/vram_mb/hours injectes dans canonical AVANT sign)
  + X-SBFB-Token rotation auto, Phase E1 `9f4d19f` NVIDIA driver
  CVE check launcher startup (NVD scrape + cache 24h), Phase E2
  `04c9621` warrant canary mensuel Ed25519 signe (gossip topic
  `nexus-grid/warrant-canary/v1` + `CANARY.txt` bootstrap pubkey
  `80b439cb...` FlowUP persistante + `scripts/verify-canary.sh` +
  `canary-monthly.yml` GHA VERIFIER par design — stocker cle en
  GHA secret casserait dead-man switch), Phase E3 `95807b1`
  Codeberg prive disaster-recovery mirror (pivot Radicle:
  `codeberg.org/SBFB/SBFB` + `mirror-codeberg.yml` push --mirror
  auth via `http.extraheader`, `MIRROR_FALLBACK.md` §1-§7
  self-contained avec §3 flip sequence v1.0 complet 3.1-3.8 pour
  activation Radicle `gsaslis/mirror-to-radicle@v0.2.0` SHA
  `514707f3` + 5 secrets GHA au go-public). Phase F wrap-up livre
  verification + audit plan S19 + migration planning. Delta tests
  Rust : **+44** (430 → 474), cumul ~1172 tests. Gate 1 (DnD Forge
  beta fermee) **effectivement UNLOCKED** — supply chain + repro
  builds + multi-relai + wire complete + driver check + canary +
  mirror redundancy = criteres `HARDENING_ROADMAP §7` remplis.
  Pivot notable : repo GitHub `SBFB50/SBFB` prive pre-launch +
  Radicle P2P public-only = Codeberg prive maintenant, flip
  Radicle differe au tag v1.0 go-live avec runbook self-contained.

  **Audit gate S18 joue en Phase 0 Sprint 19** (2026-04-15, session
  fraiche post-`4453bfd`) : verdict CONDITIONAL PASS avec 0 P0 + 1
  P1 + 5 P2 + 6 P3. Fermeture via 6 commits :
  - `677556f` D-1 (P1) : wire `TokenRotator` via `AuthState::
    Rotated(Arc<RwLock<TokenRotator>>)` + `notify` file-watcher
    sur `tokens.json` (pattern S16 ConsentWatcher). Rotation 24h
    passe de primitive livree a effective au runtime. +4 tests
    Rust (Rotated accepts current+previous, post-overlap reject,
    Static non-regression, file-watcher reload).
  - `0fb8458` F-1+F-2 (P2) : resolve 4 docs hygiene discrepancies
    (phase_E1_review presence + file count 9→10 + "5 reviews"→"6
    reviews" + tip placeholders `<wrap-up>`/`<A>` resolus reel).
  - `9661485` A-1 (P2) : drop `arg: --workspace` du job cargo-deny
    (default depuis v0.14, rejete par versions modernes).
  - `6fe2dce` B-1 (P2) : ajouter le wheel `nexus-core-py` a la
    matrix `release.yml` avec attestation SLSA in-toto (parite
    avec nexus-worker / nexus-shell-daemon / nexus-launcher).
  - `e223ec7` C-1 (P2) : clarifier `verification.md §Gate 1` que
    DHT quorum est livre comme **primitive prete, runtime wiring
    S19+**. Pas de wire au browse aggregator ce sprint (carry-
    over).
  - `1a606a3` P3 batch (3 nits) : buildType URI SLSA slsa.dev/
    build-type/custom + `parse_version` warn sur segment
    non-numerique + RADICLE_PROJECT_NAME casing align.

  Compteurs finals post-gate : **478 Rust** (474→478 +4 audit fix)
  / 183 SDK / 187+3 coord / 46 gov / 239 Vitest / 38 Playwright /
  7/7 size / 246+ SPDX (~1176 tests). **CONDITIONAL PASS LEVE**.
  Carry-overs S19 : C-1 wire `redundant_resolve` au browse
  aggregator + Meta-1 Radicle-v1.0 activation tracking.

- **Sprint 19** : durcissement chaine transport P2P. Phase A
  `ab6985c` **wire DHT quorum** au runtime (carry S18 C-1) —
  `PkarrQuorumResolver` + `PkarrRelayClient` wrap cables au
  browse aggregator + curator runtime via canary opt-in
  `SBFB_PKARR_RELAYS` env var (disabled par defaut), **Eclipse-
  by-DHT defense runtime-active sous config** (enforcement strict
  par defaut = post-Gate 2 ; flip S18 verification §Gate 1
  `[~]→[x]` acte le wiring primitive→runtime, pas l'activation
  universelle). Phase B `edfc51b` + follow-up `08f4e41` PoW Hashcash
  primitive (`crates/nexus-core-rs/src/pow.rs` SHA256 leading-zeros
  difficulty target + domain separation `DOMAIN_POW_V1` + nonce
  solve loop single-threaded CI-friendly) + gossip subscribe
  integration (`subscribe_with_pow` wrap + `relay_pow_policy.toml`
  loader + per-relai difficulty override + default 2^18 ~100ms
  CPU moderne 2026). Phase C `540bb51` TLS cert pinning relays
  (`tls_pinning.rs` SPKI hash extract DER RFC 7469 pattern HPKP
  concept + `PinValidator` fail-closed sur pinset empty + fixture
  `relay_test_cert.pem` deterministe + doc PATTERNS.md section
  rotation procedure). Phase D `f238d31` delayed upload queue
  (`packages/nexus-coordinator/src/nexus_coordinator/upload_queue.
  py` async queue + scheduler 30s flush loop + exponential jitter
  0-5min anti-correlation + integration `api/tasks.py` pipe
  gossip emit async au lieu de direct + tests pytest couvrant
  distribution range + persistence + concurrent submit). Phase E
  `2fd4d72` pkarr relay self-hosted docker image (`docker/pkarr-
  relay/Dockerfile` non-root user UID 10001 + tini + healthcheck,
  `.github/workflows/build-pkarr-image.yml` push `ghcr.io/SBFB50/
  pkarr-relay` + Trivy scan, `docs/release/PKARR_RELAY_OPS.md`
  §1-§7 self-contained : rationale + provisioning Hetzner CX11
  + systemd + nginx Let's Encrypt + smoke test + monitoring +
  rotation SPKI cert cross-ref Phase C). Phase F wrap-up livre
  verification + audit plan S20 + migration planning active →
  archive/v1.2/. Delta tests S19 : **+82** (+59 Rust PoW+TLS+
  DHT wire, +21 coord delayed upload, +2 SDK helpers) — cumul
  **~1259 tests**. Gate S19 = criteres HARDENING_ROADMAP §3 S19
  remplis, prerequis S21 rate-limit disponible (Sybil-resistance
  minimale via PoW). Pas de zone rouge nouvelle.

  **Audit gate S19** : a jouer en Phase 0 Sprint 20. Audit plan
  livre `.planning/archive/v1.2/sprint19_audit_plan.md` avec 6
  tracks (A DHT wire + B PoW Hashcash + C TLS pinning + D
  delayed upload + E pkarr relay + F wrap-up) + meta-track
  Radicle-v1.0 activation tracking re-carry S20. Pattern
  permanent depuis Sprint 7. Compteurs d'entree S20 : **537
  Rust** / 185 SDK / 208+3 coord / 46 gov / 239 Vitest / 38
  Playwright / 7/7 size / 246+ SPDX (~1259 tests).

---

## v1.1 — Verified deploy + bridge bidirectionnel + CPU watchdog (S14-15)

| Sprint | Etat | Tip cloture | Nb commits | Docs |
|---|---|---|---|---|
| 14 | DONE + CONDITIONAL PASS levé | `f6015b3` (A-1 commit_sha fix landed) | 5 + 1 (gate) | 5 docs (kickoff, plan, verification, audit_plan, audit_findings) |
| 15 | DONE | `4da0043` (Phase E docs) | 5 (A-D + docs) | 4 docs (kickoff, plan, verification, audit_plan) |

**Faits saillants** :

- **Sprint 14** : premier deploy verified-from-source. Le coordinator
  clone le repo, verifie SBFB.json (Keyoxide pattern Ed25519), zip
  le contenu, signe `provenance.json` (SLSA L1). ProjectAnnouncement
  v4 ajoute `provenance_hash` et `repo_url`. Multi-forge (GitHub,
  GitLab, Codeberg, Gitea generique). Badge "Verifie" cote shell.
  Audit conditional PASS leve via `542479f` (commit_sha SHA pinning
  full 40 hex).

- **Sprint 15** : bridge devient bidirectionnel via `sbfb-bridge-event`
  (host → iframe push, fire-and-forget). CPU watchdog via heartbeat
  `sbfb-bridge-heartbeat` (1s) + timeout 5s + overlay "App ne repond
  plus". CLI `sbfb init <type> <path>` scaffolds 3 templates
  (html/react/pyodide). E2E Playwright avec iframe reelle qui charge
  le vrai SDK. Compteurs : ~934 tests total (+26 ce sprint).

Detail : [`.planning/archive/v1.1/`](../../.planning/archive/v1.1/).

---

## v1.0 — Pivot SBFB → P2P → universal render → bridge postMessage (S0-13)

| Sprint | Etat | Tip cloture | Nb commits | Docs |
|---|---|---|---|---|
| 0 | DONE | `stabilize/compute` mergée | 9 | - |
| 1 | DONE | `e631325` | - | - |
| 2 | DONE + audité rétro | `ed2ea76` | 6 | audit rétro dans `audit_sprint2/` |
| 3 | DONE | `9476be8` | 12 (W1..W12) | `sprint3_verification.md` |
| 4 | DONE | `3b5c162` | 9 | `sprint4_kickoff`, `_plan`, `_verification`, `_verify_prompt` |
| 5 | DONE | `cdf4467` | 9 | `sprint5_kickoff` (monolithique), `_plan`, `_verification` |
| 6 | DONE + CONDITIONAL PASS levé | `504c6aa` puis `2926383` post-gate | 8 + 10 (gate) | 4 docs + `audit_findings` |
| 7 | DONE | `9cc0796` | 8 | 4 docs + attend `audit_findings` du Sprint 8 Phase 0 |
| 8 | DONE + CONDITIONAL PASS levé | `9339bb6` | 7 | 4 docs + `audit_findings` |
| 9 | DONE + CONDITIONAL PASS levé | `eb81c27` puis `48b332a` post-gate | 7 + 2 (gate) | 4 docs + `audit_findings` |
| 10 | DONE | `d07bfcf` (pre-Phase F) | 5 | 4 docs (kickoff, plan, verification, audit_plan) |
| 11 | DONE + CONDITIONAL PASS levé | `999fec6` puis `f2c94e3` post-gate | 6 + 2 (gate) | 4 docs + `audit_findings` |
| 12 | DONE + CONDITIONAL PASS levé | `bf3f009` puis `53a9e32` post-gate | 7 + 1 (gate) | 5 docs (kickoff, plan, verification, audit_plan, audit_findings) |
| 13 | DONE | `08853ff` (Phase E docs) | 6 (planning + A-D + docs) | 4 docs (kickoff, plan, verification, audit_plan) |

**Faits saillants** :

- **Sprint 6** est le premier a avoir les 4 docs planning complets des
  le demarrage.
- **Sprint 7** est le premier cycle complet de l'audit gate pattern
  (instaure post-S6 retro).
- **Sprint 10** est le premier sprint ops (CI/CD + 3 VPS bootstrap, pas
  de code applicatif).
- **Sprint 11** est le premier P2P end-to-end (publish + discovery +
  render plein ecran).
- **Sprint 12** est le premier rendu universel cross-node (archive zip
  → daemon blob-serve → iframe sandboxee).
- **Sprint 13** est le premier avec bridge iframe ↔ reseau
  (postMessage + open source enforcement + launcher Rust minimal).

Detail : [`.planning/archive/v1.0/`](../../.planning/archive/v1.0/).

---

## Conventions

### Quand mettre a jour ce log

- A la cloture du sprint N (Phase E commit) — ajouter la row dans la
  section v1.x correspondante avec etat `DONE`
- A la levee d'une CONDITIONAL PASS — mettre a jour l'etat avec
  `+ CONDITIONAL PASS levé` et le tip post-gate
- A l'ouverture d'une nouvelle version majeure — creer une nouvelle
  section `## v1.x — theme (Sx-Sy)` au-dessus de v1.x-1

### Quand creer un nouveau dossier `archive/v1.x/`

Quand le sprint qui s'ouvre adresse un theme suffisamment distinct
du precedent pour justifier une release majeure. Exemples :
- v1.0 → v1.1 : passage de "ca marche end-to-end" a "ca marche
  verifiable cryptographiquement"
- v1.1 → v1.2 : passage de "feature complete" a "production hardening"

Decision prise en kickoff §1 du sprint qui ouvre la nouvelle version.

### Migration historique

Avant Sprint 16, tous les `sprint{N}_*.md` vivaient a plat dans
`.planning/`. La migration vers `active/` + `archive/v{X}/` a ete
faite en Sprint 16 Phase 0 pour eviter que `docs/claude/README.md`
§10 ne devienne ingerable a 30+ sprints.
