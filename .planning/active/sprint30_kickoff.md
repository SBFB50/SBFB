# Sprint 30 — Kickoff (Dette pair + warrant canary Niveau 1 + G2 remediation)

**Ecrit** : 2026-04-26 (session fraiche post-audit gate S29 `dcdda7e`).
**Type** : **sprint pair dette + feature Niveau 1** (CI cross-platform
MANDATORY + blob-serve + canary DKG wiring + G2 HARDENING refresh +
split inference research doc).
**Tip master d'entree** : `dcdda7e` (chore(planning): sprint 29 audit
gate — findings verdict PASS, 0 P0/P1, 6 P2, 3 P3).
**Phase 0 audit Sprint 29** : **DEJA JOUE** — findings dans
`.planning/active/sprint29_audit_findings.md` (verdict **PASS**,
0 P0/P1, 6 P2, 3 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** (2026-04-26) : HARDENING_ROADMAP last_validated
  `2026-04-26` (S28 Phase D). **3 triggers ACTIFS** depuis
  last_validated :

  1. **iroh 0.98.0** (2026-04-17, n0-computer/iroh) — trigger
     "iroh release > 0.97" ACTIF. Le crate `iroh` principal a
     publie 0.98.0. Impact : Day 0 #3 interdit upgrade (sprint
     dedie), LT-6 condition de declenchement partiellement remplie
     (verifier si API neighborhood amelioree). Pas d'action S30,
     awareness documentee dans HARDENING_ROADMAP refresh.
     Source : [crates.io/iroh](https://crates.io/crates/iroh),
     [github.com/n0-computer/iroh/releases](https://github.com/n0-computer/iroh/releases).

  2. **arti-client 2.0.0** (2026-02-07, Tor Project) — trigger
     "arti-client release > 1.x stable" ACTIF. Premier release
     **stable** d'arti. API : `TorClient::create_bootstrapped(config)`
     + `tor_client.connect(("host", port))`. LTS annonce pour
     branche 2.x. Impact : HARDENING_ROADMAP "Tor transport phase 1
     (S30+, conditionnel arti >= 1.0)" devient faisable. Scope trop
     large pour S30 (dette + carries), reporte S31.
     Source : [abit.ee/en/soft/browsers/tor-arti-200](https://abit.ee/en/soft/browsers/tor-arti-200-rust-anonymity-privacy-hidden-services-onion-ip-over-onion-update-release-en),
     context7 `/git_gitlab_torproject_org/tpo_core_arti`.

  3. **openai-agents-python 0.14.6** (2026-04-25) — trigger
     "> 0.7.0" ACTIF. Sandbox Agents, WebSocket transport, `openai
     v2.x` requis. Impact : informationnel uniquement — SBFB ne
     depend pas directement de ce package. API surface changes
     documentees pour reference future guardrails.
     Source : [github.com/openai/openai-agents-python/releases](https://github.com/openai/openai-agents-python/releases).

  Triggers INACTIFS : frost-ed25519 (toujours 2.1.0), wasmtime,
  Tor PoW hspow, NIST PQC FIPS, NVIDIA H100 CCM (GA CUDA 12.4
  mais pas de nouveau driver 2026), RFC 9591, MCP spec,
  microsoft/sudo.

  **Sprint S+2 trigger** : S30 est le sprint cible S28+2 → re-scan
  §3 S30 effectue. Items S30 HARDENING_ROADMAP evalues ci-dessous
  §1.2.

- **G9 WebSearch (2026-04-26)** :
  - **nym-sdk** : publication crates.io **pausee** depuis v1.20.4.
    Stream module et travaux recents non publies. Reprise prevue
    avec "Lewes Protocol". Import depuis Git uniquement.
    Source : [nym.com/docs/developers/rust](https://nym.com/docs/developers/rust).
    → Nym mixnet phase 1 re-defere S32+.

  - **arti-client 2.0 API** : `TorClient::create_bootstrapped()`
    → connection TCP anonymisee, `.connect(addr)` async.
    Standalone SOCKS proxy `arti proxy -p 9150`. Onion service
    support. Deps : `arti-client`, `tor-rtcompat`,
    `futures::io::{AsyncReadExt, AsyncWriteExt}`.
    Source : context7 Arti (1597 snippets, reputation High, score 85).

  - **frost-ed25519 2.1.0** : API stable. `generate_with_dealer()`
    pour trusted dealer DKG. `frost::round1/round2/aggregate` pour
    signature interactive. ZF crate audite ToB 2023. Pas de release
    > 2.1.0 en avril 2026.
    Source : [crates.io/frost-ed25519](https://crates.io/crates/frost-ed25519).

- **G9 Codebase Exploration (2026-04-26)** :
  - **Canary FROST existant** : `canary/frost.rs` contient
    `FrostCanarySigner` (S20 Phase E.2). Trait `CanarySigner`.
    `AttestationProvider` trait + `NoopAttestation` impl.
    `CanaryRegistry` coord-side. DKG ceremony code = **absent**
    (seul le signing est implemente, pas la generation de shares).
  - **Platform writers** : `JournaldWriter` + `OsLogWriter` dans
    `nexus-events-core/src/lib.rs` cfg-gated. Aucun CI cross-
    platform existant (Windows-only dev).
  - **blob-serve** : CSP `connect-src 'none'` dans
    `blob_serve.rs`. Pas de COOP/COEP. Isolation gap = manque
    `Cross-Origin-Opener-Policy` + `Cross-Origin-Embedder-Policy`.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 29 CLOSED. 4 phases A-D livrees :
- Phase A : P2 batch S28 (5/8 fermes) + cold-start benchmark RTX
  5080 (497 ms warm << 5000 ms target)
- Phase B : THREAT_MODEL §9 per-mode residual risks + SECURITY.md
  + BUILDING.md + security.txt RFC 9116
- Phase C : process isolation broker/executor split JSON-RPC 2.0
  IPC (nexus-executor crate ~495 LOC + ipc_broker ~550 LOC)
- Phase D : TraceProvider opentelemetry 0.31 backend-agnostic
  (nexus-trace-core crate ~733 LOC)

Audit gate S29 : **PASS** (0 P0/P1, 6 P2, 3 P3). 7 P2 carry S30
documentes dans `sprint29_carry_summary.md`.

### §1.2 Ancrage HARDENING_ROADMAP

HARDENING_ROADMAP §3 Sprint 30 prescrit :

| Item prescrit | Statut G2/G9 S30 | Decision |
|---|---|---|
| Nym mixnet phase 1 (carry S28) | SDK paused crates.io, Lewes Protocol non publie | **RE-DEFERE S32+** |
| TEE H100 attestation | Pas de partenaire hardware ONG | **SCOPE-CUT** |
| Split inference research | Docs-only, faisable | **INTEGRE Phase D** |
| Warrant canary Niveau 1 enforcement | frost-ed25519 2.1 stable, DKG code absent | **INTEGRE Phase C** (scope reduit : code wiring sans TEE, sans recrutement mainteneurs) |

**Tor transport phase 1** (S30+, conditionnel arti >= 1.0) : arti
2.0.0 stable depuis 2026-02-07 → **faisable** mais scope trop
large pour dette sprint pair. Reporte **S31** comme feature
principale.

### §1.3 Compteurs tests entree (tip `dcdda7e`)

| Suite | Count | Delta vs S28 entree |
|---|---|---|
| Rust (cargo nextest) | 856 | +28 |
| SDK (pytest) | 195 | 0 |
| Coordinator (pytest) | 393 passed + 36 failed (PyO3 wheel stale) + 6 skipped | +2 |
| Gov (pytest) | 46 | 0 |
| Vitest | 269 | 0 |
| Playwright | ~43 (41+2f env) | 0 |
| size-limit | 4/4 | 0 |
| **Total** | **~1845** | **+31** |

Les 36 coord failures = PyO3 wheel stale (meme root cause depuis
S16, pas regression). Les 2 PW failures = env Windows (pas
regression).

### §1.4 Pre-launch protocol policy (rappel)

Pas de deploiement live. `*_FORMAT_VERSION` = 1 partout. Pas de
bump version, pas de tolerant decoder multi-version. Les nouveaux
formats introduits ce sprint (canary DKG ceremony config) suivent
la meme regle. Cf. `CLAUDE.md §Pre-launch protocol policy`.

---

## §2 Goal en une phrase

Sprint 30 livre la dette pair obligatoire (CI cross-platform
MANDATORY + blob-serve isolation) + le wiring code warrant canary
Niveau 1 FROST DKG + la remediation G2 HARDENING_ROADMAP (3
triggers actifs) + le research doc split inference.
**Critere SMART : 30+ rows fail-fast vertes au verification.md,
mesure binaire au Phase E wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 29

**DEJA JOUE** — commit `dcdda7e`.

Verdict : **PASS** (0 P0/P1, 6 P2, 3 P3).

Findings integres dans ce kickoff :
- P2-AUDIT-1 : HARDENING_ROADMAP.md:734 "opentelemetry 0.27" stale
  → fix Phase A
- 5 P2 carries S29 → §6 ci-dessous
- 3 P3 : cosmetiques, fix opportuniste

ROADMAP_COMMITMENTS check (G7 Regle 3) :
- LT-1 a LT-5 : conditions latentes, aucun declenchement.
- **LT-6** : trigger "iroh > 0.97" partiellement rempli (iroh
  0.98.0 publie 2026-04-17). MAIS Day 0 #3 pin iroh 0.97 bloque
  l'upgrade. LT-6 reste dans ROADMAP_COMMITMENTS avec note
  "condition met, awaits pin lift in dedicated upgrade sprint".
  Pas de re-activation carry.

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — Warrant canary Niveau 1 : trusted dealer DKG + CLI wiring

**Retenu** : Trusted dealer DKG via `frost-ed25519 2.1`
`generate_with_dealer()`. Ajouter les CLI commands dans le module
canary du shell daemon :
- `sbfb canary frost trusted-dealer --k 2 --n 3` (genere shares +
  pubkey package)
- `sbfb canary frost round1` / `round2` / `aggregate` (signing
  ceremony interactive)
Config TOML `configs/canary.toml.sample` pour localiser les shares.
`NoopAttestation` maintenu (TEE scope-cut). Pas de recrutement
mainteneurs (ops post-v1.0). Le code prepare le chemin complet
Niveau 0 → Niveau 1 mais ne l'active pas en production sans
mainteneurs recrutes.

**Rejete** :
- *DKG distribue (`frost::keys::dkg::part1/part2/part3`)* — multi-
  machine orchestration, overkill pre-v1.0 avec N=3. WARRANT_
  CANARY_HARDENING.md §4.2 prescrit trusted dealer initial, DKG
  distribue post-v1.0.
- *Skip Niveau 1* — HARDENING_ROADMAP §3 S30 le prescrit. Sans le
  code wiring, S31+ ne peut pas activer le flip.
- *TEE H100 attestation backend* — pas de hardware partenaire.
  `NoopAttestation` suffisant pre-v1.0. TEE = LT-4 condition (c).
- *Attestation via fichier signe simple* — n'apporte rien vs
  `NoopAttestation` pre-v1.0 (pas de TEE = pas d'attestation
  hardware-bound = signature software seule = meme trust model que
  single-key canary).

**Implications code** : `crates/nexus-shell-daemon-core/src/canary/
frost.rs` (extend DKG ceremony), `crates/nexus-shell-daemon/src/`
(CLI endpoint), `configs/canary.toml.sample` (new),
`docs/security/WARRANT_CANARY_HARDENING.md` (ops runbook §4
update).

### D2 — CI cross-platform : GitHub Actions multi-OS workflow

**Retenu** : GitHub Actions workflow `.github/workflows/
ci-cross-platform.yml` avec matrice `ubuntu-latest` +
`macos-latest`. Scope : `cargo nextest run -p nexus-events-core`
(crate avec les platform writers cfg-gated) + `cargo clippy` +
`cargo fmt --check`. Pas de build complet workspace (iroh deps
longues a compiler cross-platform). Trigger : push sur `master` +
PR.

**Rejete** :
- *Build complet workspace cross-platform* — iroh + pyo3 + ollama
  deps = 15-20 min build Linux/macOS, overkill pour valider 2
  platform writers.
- *Docker Linux container sur Windows* — ne teste pas macOS, et
  Docker Desktop Windows a des limitations avec les tests Rust
  natifs (mount volumes, syscall compat).
- *Mock-only sur Windows* — ne valide pas que le code compile et
  s'execute sur les OS cibles. 3/3 reports justement parce que
  mock-only ne suffit pas.
- *CI heberge (GitLab runner, Buildkite)* — overhead infrastructure,
  GitHub Actions gratuit pour open-source.

**Implications code** : `.github/workflows/ci-cross-platform.yml`
(new), ajustements cfg-gate eventuels dans
`crates/nexus-events-core/src/lib.rs`.

### D3 — blob-serve isolation : COOP/COEP + CSP upgrade

**Retenu** : Ajouter les headers de securite manquants dans
`blob_serve.rs` :
- `Cross-Origin-Opener-Policy: same-origin` (isole la fenetre)
- `Cross-Origin-Embedder-Policy: require-corp` (bloque les
  resources cross-origin non-CORP)
- `X-Content-Type-Options: nosniff` (si absent)
- Verification que `sandbox="allow-scripts"` sans
  `allow-same-origin` est toujours en place (defense iframe).
Documentation du gap restant : full process isolation (blob-serve
dans un subprocess separe) = item long-terme, pas S30.

**Rejete** :
- *Full process isolation blob-serve* — rewrite architectural
  (spawn blob-serve dans un processus child, IPC, lifecycle),
  estimable a > 1000 LOC, depasse le scope dette.
- *Statu quo* — 2/3 reports, la dette doit progresser. Les headers
  COOP/COEP sont un quick-win factuel (3 lignes) qui ferme le gap
  pour la surface iframe.
- *CSP `script-src 'none'`* — casserait les apps qui executent du
  JS (React, Pyodide). Le modele est `allow-scripts` sans
  `allow-same-origin`, les headers COOP/COEP renforcent
  l'isolation sans casser le modele.

**Implications code** : `crates/nexus-shell-daemon-core/src/
blob_serve.rs` (headers), tests unitaires blob-serve.

### D4 — G2 HARDENING_ROADMAP refresh : 3 triggers + S30 update

**Retenu** : Mise a jour complete de `HARDENING_ROADMAP.md` :
- `last_validated: 2026-04-26` (date du scan G2 S30 kickoff)
- §3 S30 : statut reel des 4 items (Nym re-defer, TEE scope-cut,
  split inference Phase D, warrant canary Niveau 1 Phase C)
- Nouvelle entree §3 S31 : "Tor transport phase 1 avec arti 2.0"
  (nouvellement faisable, deplace depuis S30+)
- `audited_findings` : ajouter entree 2026-04-26 S30 avec les 3
  triggers actifs documentes
- `VALIDATED_BLUEPRINT.md` : verifier coherence si impacte par les
  3 triggers (probable non — triggers sont deps, pas menaces)

**Rejete** :
- *Upgrade iroh 0.98 immediate* — Day 0 #3 (iroh 0.97 pinne,
  upgrade volontaire = sprint dedie). Documenter awareness only.
- *Tor transport S30* — scope trop large (dette + carries + Niveau
  1). Report S31 comme feature principale.
- *Ignorer les triggers* — 3 triggers actifs non-documentes = dette
  de fraicheur documentaire, G2 §6.8 l'interdit.

**Implications code** : `docs/security/HARDENING_ROADMAP.md`
(updates), eventuellement `docs/security/VALIDATED_BLUEPRINT.md`.

### D5 — Split inference research : design doc only

**Retenu** : Document `docs/security/SPLIT_INFERENCE_DESIGN.md`
qui capture les findings de la recherche split inference
(HARDENING_ROADMAP §3 S30 item). Contenu : modeles split-inference
existants (BOINC verification, Truebit interactive verification,
Golem task markets), applicabilite au modele SBFB (coordinateur
distribue, pas de serveur central), implications threat model
(C-PromptLeak via partitionnement), recommendations pour sprint
futur. Pas de code.

**Rejete** :
- *Prototype code* — pas de budget avec dette + carries + Niveau 1.
  Le research doc est le livrable prescrit par HARDENING_ROADMAP.
- *Skip* — HARDENING_ROADMAP §3 S30 le prescrit. Un design doc
  factuel cree une base pour le sprint dedie futur.
- *Integration dans HARDENING_ROADMAP inline* — trop dense pour
  etre un paragraphe, merite un document dedie avec sections
  structurees (patterns, threat model, recommendations).

**Implications code** : `docs/security/SPLIT_INFERENCE_DESIGN.md`
(new).

### Acknowledged review findings (G1)

Scoring : D1 ✅, D2 ✅, D3 ✅, D4 ⚠️, D5 ⚠️.
Rigor signal G4 satisfait (2 ⚠️ sur 5).

**D4 ⚠️** : "D4 decision text lacks explicit alternative comparison
for arti-client choice". Decision : adjust — Phase D HARDENING_ROADMAP
§3 S31 entry ajoutera 1 ligne rationale "arti-client 2.0.0 LTS +
API stable (pas de hybride I2P/Snowflake requis pour phase 1 scope)".

**D5 ⚠️** : "no bibliography pre-allocated for split inference
research doc". Decision : adjust — Phase D sera precedee d'un pre-
select de 3-5 sources cles (Truebit Verify whitepaper, BOINC docs,
Golem API, 1-2 papers academiques split learning) pour grounding
immediat du design doc.

---

## §5 Plan Phase outline A..E

### Phase A — P2 batch S29 audit

Fixes les items P2/P3 de l'audit S29 + carries resolvables :
- P2-AUDIT-1 : HARDENING_ROADMAP.md "opentelemetry 0.27" → "0.31"
- P3-AUDIT-1 : nexus-trace-core lib.rs:8 docstring "0.28+" →
  "0.31"
- P2-REVIEW-B-1 : consent.py `_populate_threat_fields` refactor
  (pure function, pas mutation in-place)
- P2-REVIEW-D-1 : executor trace log path → resoudre depuis
  `ShellDaemonPaths` OU documenter l'asymetrie intentionnelle
- Document P2-REVIEW-B-2 (§9.5 output filter) : wire gap status
  update dans THREAT_MODEL §9.5
- Document P2-REVIEW-C-1 (task_runner stub) : defense-in-depth
  note ajoutee, carry confirme

Commit cible : `feat(sprint30): Sprint 30 Phase A — P2 batch S29
audit (7 items)`

### Phase B — Phase dette (sprint pair §6.2.1 Regle 1)

MANDATORY : CI cross-platform + blob-serve isolation.
- P2-B-1-S28 : GitHub Actions workflow multi-OS (D2)
- P2-C-1-S28 : blob-serve COOP/COEP headers (D3)
- Fixes P3 cosmetiques opportunistes si touche le meme fichier

Commit cible : `feat(sprint30): Sprint 30 Phase B — dette pair CI
cross-platform + blob-serve isolation`

### Phase C — Warrant canary Niveau 1 code wiring

Feature principale du sprint :
- DKG trusted dealer CLI (`generate_with_dealer` wrapper)
- Signing ceremony CLI (round1/round2/aggregate wrappers)
- Config `canary.toml.sample` (share paths, pubkey package path,
  K/N params)
- Ops runbook update dans WARRANT_CANARY_HARDENING.md §4
- Tests : DKG roundtrip, signing ceremony 3-participant, tamper
  detection

Commit cible : `feat(sprint30): Sprint 30 Phase C — warrant canary
Niveau 1 FROST DKG code wiring`

### Phase D — G2 remediation + split inference research

Docs-heavy phase :
- HARDENING_ROADMAP.md refresh (D4) : last_validated, triggers,
  S30 statut, S31 Tor transport entree
- SPLIT_INFERENCE_DESIGN.md (D5) : research doc
- VALIDATED_BLUEPRINT.md : coherence check post-triggers
- P3 cosmetiques restants si applicable

Commit cible : `docs(sprint30): Sprint 30 Phase D — G2 HARDENING
refresh + split inference research`

### Phase E — Wrap-up + verification + audit plan S31

Standard wrap-up :
- sprint30_verification.md (fail-fast 30+ rows)
- sprint30_carry_summary.md
- sprint31_audit_plan.md (audit plan pour S31 Phase 0)
- SPRINT_LOG.md row S30
- CLAUDE.md §Etat actuel update
- Memory update nexus_grid_pivot.md + MEMORY.md

Commit cible : `chore(sprint30): Phase E — wrap-up + verification
+ audit plan S31 + migration`

---

## §6 Items carry/dette (G7)

### Carry S29 — resolution prevue

| ID | Description | Reports | Resolution S30 | Phase |
|---|---|---|---|---|
| P2-B-1-S28 | CI Linux/macOS writers | **3/3 MANDATORY** | CI GitHub Actions multi-OS | B |
| P2-C-1-S28 | blob-serve isolation gap | 2/3 | COOP/COEP headers | B |
| P2-REVIEW-B-1 | consent.py mutation pattern | 1/3 | Refactor pure function | A |
| P2-REVIEW-B-2 | §9.5 output filter not wired | 1/3 | Document gap status | A (doc) |
| P2-REVIEW-C-1 | task_runner.rs stub | 1/3 | Document defense-in-depth | A (doc) |
| P2-REVIEW-D-1 | executor trace log path | 1/3 | Fix ou doc asymetrie | A |
| P2-AUDIT-1 | HARDENING_ROADMAP 0.27→0.31 | 0/3 | Fix doc | A |

### Items differes S31+

| ID | Description | Reports | Sprint cible | Justification |
|---|---|---|---|---|
| P2-REVIEW-B-2 | §9.5 output filter not wired | 2/3 | S31 | Wire end-to-end necessite changes multi-crate (pas dette-compatible) |
| P2-REVIEW-C-1 | task_runner.rs stub | 2/3 | S31 | Implementation reelle = feature, pas dette (Ollama/llama.cpp IPC dispatch) |

### Phase dette S30 (§6.2.1 Regle 1 — sprint pair)

Phase B reservee exclusivement aux items differes :
- P2-B-1-S28 (3/3 MANDATORY) : absorbe
- P2-C-1-S28 (2/3) : absorbe

### Items long-terme (ROADMAP_COMMITMENTS — inchanges sauf LT-6)

| ID | Condition | Status |
|---|---|---|
| LT-1 | v1.0 + design doc + Gini > 0.70 | Latent |
| LT-2 | tag v1.0 | Latent |
| LT-3 | v1.0 + 3+ contrib non-compute | Latent |
| LT-4 | v1.0 + N1 FROST + partnership | Latent |
| LT-5 | multi-worker deploy OR v1.0 | Latent |
| LT-6 | iroh > 0.97 OR v1.0 | **Trigger met** (iroh 0.98.0 2026-04-17) — bloque par Day 0 #3 pin, reste latent |

---

## §7 Scope cuts

Ce que S30 ne fait PAS :

1. **Tor transport phase 1** — reporte S31 (arti 2.0 stable mais
   scope dette + carries + Niveau 1 ne laisse pas de place)
2. **Nym mixnet phase 1** — re-defere S32+ (SDK paused crates.io)
3. **TEE H100 attestation** — scope-cut (pas de hardware
   partenaire). Prerequis LT-4
4. **DKG distribue FROST** — post-v1.0 (trusted dealer suffisant
   N=3, cf. WARRANT_CANARY_HARDENING §4.2)
5. **Recrutement mainteneurs cross-juridiction** — ops post-v1.0,
   code S30 prepare le chemin
6. **iroh 0.98 upgrade** — Day 0 #3 (sprint dedie)
7. **Upgrade openai-agents-python** — pas de dep directe SBFB
8. **task_runner implementation reelle** — feature pas dette, S31
9. **§9.5 output filter wire end-to-end** — multi-crate, S31
10. **Full process isolation blob-serve** — rewrite architectural,
    LT scope
11. **Tor PoW spec update** — trigger inactif
12. **MCP spec revision** — trigger inactif
13. **CI full workspace cross-platform** — scope CI = nexus-events-
    core uniquement (platform writers)

---

## §8 Tracabilite scope

Table mappant les items S29 "What's NOT" sur leur traitement S30 :

| Item S29 scope-cut | Sprint + Phase S30 | Status |
|---|---|---|
| CI Linux/macOS (3/3 MANDATORY) | S30 Phase B | **INTEGRE** |
| blob-serve isolation (2/3) | S30 Phase B | **INTEGRE** |
| consent.py mutation (1/3) | S30 Phase A | **INTEGRE** |
| §9.5 output filter (1/3) | S30 Phase A (doc) → S31 (wire) | DOCUMENTE |
| task_runner stub (1/3) | S30 Phase A (doc) → S31 (impl) | DOCUMENTE |
| executor trace log path (1/3) | S30 Phase A | **INTEGRE** |
| HARDENING_ROADMAP 0.27→0.31 | S30 Phase A | **INTEGRE** |

---

## §9 Risk register

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | GitHub Actions macOS runner indisponible | Low | Medium | Fallback ubuntu-only si macOS flaky, macOS en allow-failure |
| R2 | frost-ed25519 API break entre 2.1 et code S30 | Very Low | Medium | Pin exact `=2.1`, audit ToB 2023 stable |
| R3 | COOP/COEP casse des apps iframe existantes | Medium | Medium | Test regression Playwright sur apps demo (hello-world-app) |
| R4 | Scope creep Niveau 1 (recrutement → code → ops) | Medium | High | Scope strict = code wiring uniquement, 0 ops |
| R5 | Sprint pair trop charge (dette + feature + docs) | Medium | Medium | Phase D docs-only absorbe la charge, Phase C = seule feature |
| R6 | arti 2.0 defer S31 conteste par audit gate | Low | Low | S30 documente le defer avec evidence (scope dette + carries) |
| R7 | LT-6 iroh 0.98 trigger met mais non actionnable | Low | Low | Document awareness, pas d'action tant que pin Day 0 #3 en place |

---

## §10 Audit gate pattern — rappel

Phase 0 audit S29 **jouee** — verdict PASS, commit `dcdda7e`.
Phase E produira :
- `sprint30_verification.md` (self-report fail-fast)
- `sprint31_audit_plan.md` (plan pour S31 Phase 0)
- `sprint30_carry_summary.md`

---

## §11 Checkpoint de validation

5 questions pour arbitrage user AVANT le plan detaille :

1. **D1 Niveau 1** : le scope "code wiring sans recrutement ni TEE"
   est-il suffisant pour S30, ou faut-il un scope plus ambitieux
   (ex: DKG distribue) ?
2. **D2 CI** : GitHub Actions multi-OS avec scope nexus-events-core
   uniquement — suffisant pour fermer le carry 3/3 ?
3. **D3 blob-serve** : COOP/COEP headers = fermeture du carry 2/3
   ou faut-il aller plus loin (process isolation partielle) ?
4. **D4 HARDENING_ROADMAP** : reporter Tor transport S31 malgre
   arti 2.0 stable — acceptable vu la charge dette ?
5. **D5 Split inference** : design doc seul — suffisant ou
   prototype minimal souhaite ?
