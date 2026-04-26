# Sprint 29 — Kickoff (Process isolation MVP + audit prep + TraceProvider)

**Ecrit** : 2026-04-26 (session fraiche post-audit gate S28 `102bc9f`).
**Type** : **sprint implementation + audit prep** (broker/executor split +
THREAT_MODEL §9 residual risks + TraceProvider tracing + responsible disclosure).
**Tip master d'entree** : `102bc9f` (chore(planning): sprint 28 audit gate —
findings verdict PASS, 0 P0/P1, 5 P2/1 P3).
**Phase 0 audit Sprint 28** : **DEJA JOUE** — findings dans
`.planning/active/sprint28_audit_findings.md` (verdict **PASS**,
0 P0/P1, 5 P2, 1 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** (2026-04-26) : HARDENING_ROADMAP last_validated
  `2026-04-26` (delta 0j). Les 12 triggers re-scannes par S28 Phase D
  = tous INACTIVE ou sans impact S29. Pas de re-fetch necessaire.

- **G9 WebSearch (2026-04-26, agent general-purpose)** :
  - **Rust IPC SOTA** : `jsonrpsee v0.26.0` (Parity, 2025-08-11) =
    SOTA JSON-RPC 2.0 Rust, pre-1.0 mais production-proven (polkadot-sdk,
    zkSync Era, Forest Filecoin). Pas de built-in IPC transport UDS/NP
    mais custom transports supportes. **Aucun CVE** dans RustSec.
    Alternative retenue : raw `serde_json` + `tokio` UDS/NP (pattern
    Delta Chat JSON-RPC bindings, zero dep supplementaire — serde_json
    deja dans le workspace).
  - **opentelemetry Rust** : version courante **0.31.0** (2025-09-25),
    PAS 0.27 comme cite par HARDENING_ROADMAP §3 S29. Breaking changes
    majeurs 0.27→0.28 (global shutdown removed, `TracerProvider` →
    `SdkTracerProvider`, async runtime plus requis pour batch processors).
    Traces API stable depuis 0.28. MSRV 1.75.0. **PLAN-ADAPT finding** :
    A2 TraceProvider doit cibler opentelemetry 0.31, pas 0.27.
    Recommended stack : `opentelemetry = "0.31"` + `opentelemetry_sdk
    = "0.31"` + `opentelemetry-otlp = "0.31"` (features `http-proto` +
    `reqwest-blocking-client`). 1.0 pas encore publie (retard vs roadmap
    "early 2025").
  - **Audit prep best practices** : Trail of Bits checklist
    (blog.trailofbits.com/2018/04/06/) : Review Goals Document + clean
    codebase (`-D warnings` = deja fait) + "batteries included" package
    (BUILDING.md, frozen commit, scope markers) + documentation suite
    (architecture, actor/privilege map, inline comments). Cure53 :
    Work Packages format, 10-15 person-days, white-box. Trail of Bits
    `audit-prep-assistant` tool (github.com/trailofbits/skills).
  - **W3C Trace Context** : `traceparent` header standard cross-process.
    OTel env carrier spec (Beta) : `TRACEPARENT`/`TRACESTATE` env vars
    pour subprocess spawn. Alternative per-request : embedded dans
    JSON-RPC request metadata. Rust `TraceContextPropagator` + Python
    `opentelemetry-api 1.x` W3C TC par defaut.
  - **IPC benchmarks** (3tilley.github.io/posts/simple-ipc-ping-pong/) :
    Linux stdio 4.8µs, Windows stdio 28.5µs (6x) — negligeable vs
    inference 100ms+/token. Named Pipe + JSON-RPC = latence < 50µs.

- **G9 Codebase Exploration (2026-04-26, agent Explore)** :
  - **Process isolation readiness** : daemon monolithique 8700 LOC core
    (`nexus-shell-daemon-core`, 14 modules publics). Zero `std::process::
    Command` ni `tokio::process`, zero subprocess spawning. UDS/NP
    existants = auth layer (peer creds), PAS IPC RPC. PROCESS_
    ARCHITECTURE.md (540 LOC, S28 Phase C) = design complet avec 3
    JSON-RPC methods, cold-start <5s target, pool mode N=1. **Readiness
    ~40% (design 100%, code 0%).**
  - **SecurityEvent enum** : 12 variantes dans `nexus-events-core`.
    **Manque** : `ExecutorCrash`, `BrokerCrash` (references dans
    PROCESS_ARCHITECTURE.md §6.1 mais absents de l'enum). A ajouter
    Phase C.
  - **TraceProvider** : **zero integration OpenTelemetry** dans le
    workspace. Deps actuelles : `tracing = "0.1"` + `tracing-subscriber
    = "0.3"`. Pas de `opentelemetry` ni `tracing-opentelemetry` dans
    aucun Cargo.toml. Crate `nexus-events-core` = pipeline `SecurityEvent
    → EventWriter trait → 4 impls` (JsonFileWriter, TracingWriter,
    JournaldWriter, OsLogWriter). Architecture propre, extensible.
  - **THREAT_MODEL §9** : section existante "Revue et evolution" (regles
    de mise a jour), PAS de per-mode residual risk analysis. Aucun delta
    consent-level (L1/L2/L3/L4) documentes. §8 residuals R1-R6
    s'appliquent uniformement. Gap confirme : agents_sudo B4 = nouveau
    contenu, pas un refactor.
  - **Cold-start** : zero benchmark code. PROCESS_ARCHITECTURE.md §4.3
    dit "prerequis S29 benchmark reel avant implementation".
  - **Tests baseline** : 828 Rust nextest / 835 lib tests total (incl.
    feature-gated) / 674 Python test functions / 292 JS/TS test files.

---

## 1. Constat d'entree

### 1.1 D'ou on part

- **Tip** : `102bc9f` — S28 audit gate PASS (0 P0/P1, 5 P2/1 P3).
- **Working tree** : propre.
- **v1.2** : continuation security hardening + Gate 3 prerequisites.
  Pas de nouvelle version (meme theme).

### 1.2 Ancrage HARDENING_ROADMAP

HARDENING_ROADMAP §3 S29 prescrit : "External audit + remediation buffer"
(~2620 LOC total) incluant :
- Audit execution (~50-100k$ budget, 4-8 semaines)
- Remediation findings (~1500 LOC estime)
- Public disclosure + security.txt (~200 LOC)
- agents_sudo A2 TraceProvider (~600 LOC + 40 tests)
- agents_sudo B4 THREAT_MODEL §9 (~320 LOC)

**Note realisme S29** (resout P2-D-1 carry) : l'audit externe est un
engagement de 4-8 semaines avec un tiers. S29 prepare l'engagement
(scope freeze, BUILDING.md, THREAT_MODEL §9) et livre les prerequis
code (TraceProvider, process isolation). L'audit lui-meme demarre
fin S29 ou S30 selon timeline vendor. La remediation sera absorbee dans
le sprint post-audit (S30 ou S31). Le budget ~1500 LOC remediation
est une estimation — il sera affine quand les findings arriveront.

**S28 deferred items** revendicant S29 :
- D2 broker/executor implementation (design-only S28)
- D3 Windows RPC (co-landing D2)
- C4 task-scoped sandbox (co-landing D2)

**Arbitrage S29** (post-G9 factual) :
1. D2 broker/executor : **absorbe** — c'est la phase technique principale.
   PROCESS_ARCHITECTURE.md est le design complet, l'implementation suit.
2. D3 Windows RPC : **deferred S30** — le split fonctionne via Named Pipe
   existant (S16). L'upgrade vers Windows RPC est une optimisation, pas
   un prerequis. Co-landing VM S30+.
3. C4 task-scoped sandbox : **deferred S30** — depends de D2 stable. Le
   split processus S29 est la fondation ; le sandbox per-task est la
   couche suivante.
4. A2 TraceProvider : **absorbe** avec PLAN-ADAPT (opentelemetry 0.31
   au lieu de 0.27).
5. B4 THREAT_MODEL §9 : **absorbe**.

Scope S29 redimensionne : ~1800-2200 LOC code + ~300 LOC docs =
~2100-2500 LOC total. Sprint d'**implementation technique** qui livre
le premier split processus du daemon + infrastructure tracing pre-audit.

### 1.3 Compteurs tests entree (tip `102bc9f`)

| Suite | Count | Notes |
|---|---|---|
| Rust nextest | 828 | all pass |
| Rust doctests | pass | |
| Python SDK | 195 | all pass |
| Python coord | 391 pass + 36 fail + 6 skip | 36 fail = stale PyO3 wheel |
| Python gov | 46 | all pass |
| Vitest | 269 | all pass |
| Playwright | 41 pass + 2 fail | 2 fail = env coordinator not running |
| Size-limit | 7/7 | |
| **Total** | **~1814** | (+1 vs sortie S28 — delta Vitest ±1 count method) |

### 1.4 Pre-launch protocol policy

Pas de deploiement live. `*_FORMAT_VERSION` restent a 1. S29 introduit
un nouveau domain `DOMAIN_TRACE_EVENT_V1` (design-only pre-launch stable
— pas de wire P2P, evenements locaux uniquement). `DOMAIN_IPC_REQUEST_V1`
et `DOMAIN_IPC_RESPONSE_V1` sont des formats internes broker↔executor,
pas des wire formats P2P gossip. Aucun `*_VERSION` bump sur les formats
existants.

---

## 2. Goal

Implementer le premier split processus broker/executor du daemon
(PROCESS_ARCHITECTURE.md → code), livrer l'infrastructure tracing
pre-audit (TraceProvider opentelemetry 0.31), documenter les risques
residuels per-configuration (THREAT_MODEL §9), et preparer
l'engagement audit externe (responsible disclosure + BUILDING.md +
scope freeze).

**Critere SMART : 30+ rows fail-fast verts au `verification.md`,
mesure binaire au Phase E wrap-up.**

---

## 3. Phase 0 — Audit gate Sprint 28

**Verdict** : PASS (0 P0/P1, 5 P2, 1 P3).
**Commit** : `102bc9f` — `sprint28_audit_findings.md` dans
`.planning/active/`.
**P2 carry S29** : 8 items documentes dans §6.

---

## 4. Decisions Day 0 (D1..D5 gelees)

### D1 — Process isolation : broker/executor split raw JSON-RPC 2.0

**Retenu** : implementer le split broker/executor prescrit par
`PROCESS_ARCHITECTURE.md` (S28 Phase C design doc). IPC via raw
`serde_json` + `tokio` UDS (Linux/macOS) / Named Pipe (Windows) avec
protocole JSON-RPC 2.0. Nouveau binaire `nexus-executor` dans
`crates/nexus-executor/`.

Le broker (`nexus-shell-daemon` refactore) garde : keypair Ed25519,
bearer auth, gossip, curator pipeline, browse aggregator, blob-serve,
state persistence, consent state. L'executor n'a acces ni a la
keypair ni au master token — token ephemere `task_token` per-task
(HMAC-SHA256 derive).

3 methodes JSON-RPC : `task.execute` (broker→executor), `health.report`
(executor→broker notification), `executor.shutdown` (broker→executor).

Pool mode N=1 par defaut (single-GPU RTX 5080). Spawn-on-demand mode
pour tests. Crash executor → backoff exponentiel 1s→30s cap +
`SecurityEvent::ExecutorCrash`. Heartbeat 10s executor→broker.

§Research consulte : G9 WebSearch `jsonrpsee v0.26.0` (Parity) vs raw
serde_json. jsonrpsee = SOTA mais ajoute ~500KB binaire + dep pour un
canal local ou on controle les deux cotes. `serde_json` = zero dep
supplementaire (deja workspace), pattern Delta Chat
(delta.chat/en/2025-02-11). IPC bench : UDS 4.8µs, Named Pipe 28.5µs
(Windows) — negligeable vs inference 100ms+/token. G9 Explore :
daemon monolithique 8700 LOC, zero subprocess spawning existant.

§Research consulte : PROCESS_ARCHITECTURE.md §3.2 analyse comparative
JSON-RPC 2.0 vs gRPC : latence 2-5µs vs 1-2µs (negligeable), zero
codegen vs `.proto` + tonic, 0 KB delta vs ~500 KB (tonic + prost).

**Rejete** :
- **jsonrpsee** : dep lourde (~500KB binaire, Parity ecosystem) pour un
  canal purement local ou les deux endpoints sont controles. Pas de
  transport UDS built-in — necessiterait un adapter custom de toute
  facon. Sur-engineering pour N=1 executor.
- **gRPC / tonic** : codegen protobuf, dep transitive, streaming non
  necessaire (broker envoie request, executor retourne result complet).
  Decision gelee PROCESS_ARCHITECTURE.md §3.2.
- **Shared memory** : viole la frontiere de securite processus. Le
  split vise justement l'isolation memoire entre broker et executor.
- **Monolithe maintenu** : contradicts HARDENING_ROADMAP §3 S28 D2.
  Deferred S28→S29, ne peut pas re-defer sans violer §6.2.1.

**Implications** :
- Nouveau crate `crates/nexus-executor/` (binaire)
- Refactor `crates/nexus-shell-daemon-core/` : extraire module IPC broker
- `crates/nexus-events-core/` : ajouter `ExecutorCrash` + `BrokerCrash`
  variantes SecurityEvent
- Tests : spawn executor en subprocess, IPC roundtrip, crash recovery

### D2 — Cold-start benchmark : Ollama 7B sur RTX 5080

**Retenu** : benchmark reel avant code (prereq explicite
PROCESS_ARCHITECTURE.md §4.3 Q1). Mesurer : (1) spawn process + connect
IPC, (2) Ollama model load (cache chaud), (3) premier token inference.
Cible < 5s. Si > 5s : evaluer `keep_alive: "5m"` Ollama warm-start.

Le benchmark est un test Rust integre dans `nexus-executor` crate
(pas un script externe) qui spawne le binaire executor, connecte le
socket IPC, envoie une requete `task.execute` minimale, mesure le
temps jusqu'au premier token retourne.

**Rejete** :
- **Benchmark externe (CLI script)** : pas reproductible, pas versionne,
  pas dans la CI. Un test Rust = reproductible + assertion < 5s.
- **Skip benchmark** : le design doc dit explicitement "prerequis S29,
  pas une garantie". Coder le split sans valider le budget cold-start
  risque une architecture non-viable.
- **Benchmark coordinator-side** : le bottleneck est l'executor, pas le
  coordinator. Mesurer au plus pres du spawn.

**Implications** :
- Benchmark dans `crates/nexus-executor/benches/cold_start.rs`
- Prerequis Ollama running + model 7B en cache sur machine dev
- Resultat documente dans commit body Phase A

### D3 — TraceProvider : opentelemetry 0.31 backend-agnostic (PLAN-ADAPT)

**Retenu** : `TraceProvider` trait unifie dans crate `nexus-trace-core`
(nouveau crate workspace). 3 backends :

1. **BatchLogProcessor** : JSON structured → fichier (default, zero dep
   supplementaire, extension naturelle de JsonFileWriter existant)
2. **OtelProcessor** : OTLP/gRPC via `opentelemetry 0.31` +
   `opentelemetry-otlp 0.31` (Grafana Tempo / Jaeger)
3. **SignedCanaryProcessor** : Ed25519-signed trace events
   (`DOMAIN_TRACE_EVENT_V1` nouveau domain design-only pre-launch stable)

W3C Trace Context propagation cross-process : `traceparent` header
embarque dans chaque requete JSON-RPC broker→executor (per-request
correlation, pas env vars one-shot).

**PLAN-ADAPT** : HARDENING_ROADMAP §3 S29 cite `opentelemetry 0.27`.
La version courante est **0.31.0** (2025-09-25) avec breaking changes
majeurs depuis 0.28 (global shutdown removed, `SdkTracerProvider`,
batch processors threads dedies). L'implementation cible 0.31. Evidence
OSS : opentelemetry-rust GitHub releases, migration guide 0.28. Source :
G9 WebSearch 2026-04-26.

§Research consulte : G9 WebSearch opentelemetry Rust 0.31.0. Traces API
stable depuis 0.28. OTLP exporter beta (0.31.1, 2026-03-19). Python
opentelemetry-api 1.x = W3C TC par defaut. Env carrier spec = Beta.

**Rejete** :
- **opentelemetry 0.27** : 4 versions en retard, breaking changes
  0.27→0.28 non-triviaux. Citer une version obsolete = dette des
  l'introduction.
- **tracing-opentelemetry only** : bridge tracing→OTel = sous-couche
  utile, mais ne couvre pas le signing Ed25519 ni le batch log structre.
  Le trait TraceProvider est plus flexible.
- **Zero OpenTelemetry** : le HARDENING_ROADMAP prescrit A2 pour
  l'audit pre-Cure53/ToB. L'auditeur attend de la tracing formelle.

**Implications** :
- Nouveau crate `crates/nexus-trace-core/` (lib)
- Workspace deps : `opentelemetry = "0.31"`, `opentelemetry_sdk = "0.31"`,
  `opentelemetry-otlp = "0.31"`
- Consumer Phase C : IPC requests portent `traceparent`
- Consumer Phase D : TraceProvider wired dans broker + executor startup

### D4 — THREAT_MODEL §9 : per-mode residual risk documentation

**Retenu** : refactor `docs/security/THREAT_MODEL.md` — renommer §9
"Revue et evolution" en §10, ajouter nouveau §9 "Residual risks
per-configuration" avec 6 sous-sections :

- 9.1 Consent GPU 4 niveaux (S16C) : L1=zero→aucun risque compute,
  L4=full→R-compute-all active
- 9.2 Loopback 3 trust tiers (S22F/S25/LT-4) : AUTO/CONFIRM_PROMPT/
  BIOMETRIC_GATE avec deltas de surface
- 9.3 Duress PIN (S20B) : mode normal vs duress, threat delta per AD
- 9.4 Rate-limit tiers (S22A) : no-limit→full-limit progression
- 9.5 Pipeline guardrails disabled combos (B1 S23) : ce qui casse si
  guardrails OFF
- 9.6 Capability toggles (D5 S25) : per-capability risk matrix

Annotations in-product : `consent.json` field `residual_threats_
acknowledged` + `level_threat_note` (deja livres S22 Phase F D1 design).
UI `GpuConsentDialog.tsx` affiche `level_threat_note` tooltip.

§Research consulte : G9 Explore THREAT_MODEL.md = 392 lignes, §9
actuel = regles de mise a jour (pas de per-mode risks). §8 R1-R6
s'appliquent uniformement. Gap confirme.

**Rejete** :
- **Threat model monolithique** : l'auditeur Cure53/ToB attend des
  residuals per-configuration pour evaluer les modes degraded. Un
  modele uniforme ne capture pas la posture de securite reelle d'un
  user L1 vs L4.
- **Per-mode risks dans un doc separe** : fragmentation. §9 integre
  dans THREAT_MODEL.md = single source of truth.

**Implications** :
- Update `docs/security/THREAT_MODEL.md` (+~200 LOC)
- Update `web/src/components/GpuConsentDialog.tsx` (tooltip, +~50 LOC)
- Update `packages/nexus-coordinator/` consent endpoint (+~20 LOC)
- Prereq audit externe (B4 spec HARDENING_ROADMAP)

### D5 — Scope disposition : audit engagement + deferrals

**Retenu** :

1. **Responsible disclosure** : creer `SECURITY.md` (racine) +
   `.well-known/security.txt` (RFC 9116) + `BUILDING.md` (build
   instructions "batteries included" pour l'auditeur). Pattern Trail
   of Bits checklist.

2. **Scope freeze audit** : au Phase E wrap-up, documenter le tip
   commit exact comme "audit scope freeze point" dans
   `EXTERNAL_AUDIT_SCOPE.md §2.7` (resout P2-D-2 carry). Ce tip +
   BUILDING.md + THREAT_MODEL.md = package envoyable au vendor.

3. **D3 Windows RPC** : deferred S30 — le split broker/executor S29
   utilise le Named Pipe existant (S16 Phase B, DACL user-only SDDL).
   L'upgrade vers Windows RPC (SID caller authentifie) est une
   optimisation post-split, co-landing VM S30+.

4. **C4 task-scoped sandbox** : deferred S30 — depend de D2 stable
   (1 sprint de stabilisation). Le split S29 est la fondation.

5. **Nym mixnet** : deferred S30+ — trigger `nym-sdk beta stable`
   toujours INACTIVE.

6. **Tor transport** : deferred S30+ — `arti-client > 1.x stable`
   trigger INACTIVE.

7. **GPU lockup defense** : deferred S30+ — dep A4 process roles
   (prerequis D2 S29, actionnable S30).

8. **blob-serve executor dedie** : deferred S30+ — PROCESS_
   ARCHITECTURE.md §9 Q4, Option B S30+.

§Research consulte : G9 WebSearch Trail of Bits audit prep checklist.
"Batteries included" package = frozen commit + BUILDING.md + scope
markers. security.txt RFC 9116 = standard.

**Rejete** :
- **Audit engagement S29** : la timeline 4-8 semaines ne fit pas dans
  un sprint de code. S29 prepare, S30 execute l'engagement.
- **Skip responsible disclosure** : Trail of Bits checklist l'attend.
  200 LOC de docs, pas de raison de defer.
- **D3/C4 S29** : scope creep. Le split D2 seul est deja ~800-1200 LOC.
  Co-landing D3+C4 = ~2500 LOC supplementaires, bust le budget sprint.

**Implications** :
- Nouveaux `SECURITY.md`, `BUILDING.md`, `.well-known/security.txt`
- Update `docs/security/EXTERNAL_AUDIT_SCOPE.md` §2.7

### Acknowledged review findings (G1)

Scoring : D1 ✅, D2 ⚠️, D3 ❌, D4 ⚠️, D5 ✅.
Rigor signal G4 satisfait (2 ⚠️ + 1 ❌ sur 5).

**D1 ✅** — Minor : narrative Rust-first implicite (serde_json + tokio
SONT Rust-natifs). Acknowledged : le draft ne cite pas explicitement
"Rust-native alternative considered" mais le choix raw serde_json +
tokio est par definition Rust-first (zero crate protocol externe).
Pas de changement requis.

**D2 ⚠️** — Timing ambiguite benchmark Phase A vs pre-kickoff spike.
Decision : adjust — le benchmark EST Phase A (pas pre-kickoff). C'est
un **blocking gate** : Phase C code ne commence PAS avant que le
resultat Phase A soit documente dans le commit body. Decision tree
explicite :
- Benchmark ≤ 5s → pool mode Phase C (plan nominal)
- Benchmark > 5s et ≤ 10s → evaluer Ollama `keep_alive: "5m"` warm-start
- Benchmark > 10s → pivot spawn-on-demand ONLY + re-design pool

**D3 ❌** — 3 concerns :
1. **Date anomaly** : NON-ISSUE. `cargo search opentelemetry --limit 1`
   (2026-04-26) confirme `opentelemetry = "0.31.0"` comme version
   courante. Release 2025-09-25 (7 mois) = version la plus recente
   disponible. "Version courante" = version la plus recente sur
   crates.io, pas "publiee cette semaine".
2. **PLAN-ADAPT authority** : G9 WebSearch EST l'autorite pre-D-decision
   (§6.10). PLAN-ADAPT (§6.9) require ">=1 projet OSS nomme avec
   source verifiable" — opentelemetry-rust GitHub releases +
   migration guide 0.28 (github.com/open-telemetry/opentelemetry-rust/
   blob/main/docs/migration_0.28.md) = evidence. Le HARDENING_ROADMAP
   fut ecrit S17 (2026-02-15) quand 0.27 etait courant ; 4 versions
   plus tard, 0.27 est techniquement obsolete. PLAN-ADAPT corrige.
3. **Rust-first gap** : ADJUST — documenter les gaps explicites de
   `tracing` seul vs TraceProvider + OTel :
   - `tracing-subscriber` = structured logging + file appender MAIS
     pas de rotation avec signing Ed25519 (pas de `SignedCanaryProcessor`)
   - `tracing-subscriber` = pas d'export OTLP natif (bridge
     `tracing-opentelemetry` est une sous-couche, pas un remplacement)
   - `tracing` seul ne couvre pas le batch log structre avec rotation
     signable (JsonFileWriter existant dans nexus-events-core ne
     s'integre pas au framework tracing)
   - Le trait `TraceProvider` abstrait sur 3 backends (BatchLog, OTel,
     Signed). `tracing-opentelemetry` couvre le cas OTel mais pas les
     2 autres. Conclusion : OTel est UN backend du trait, pas le
     framework entier.
4. **1.0 roadmap source** : opentelemetry-rust GitHub issue #1678
   (github.com/open-telemetry/opentelemetry-rust/issues/1678) —
   roadmap "early 2025" citee dans l'issue, 1.0 non publiee a date
   (2026-04-26). Acknowledged, source ajoutee.

Decision : proceed (Option C reviewer) — opentelemetry 0.31.0 pinne,
PLAN-ADAPT authority = G9 evidence (§6.9/§6.10).

**D4 ⚠️** — S22 annotations non implementees dans le code production.
Decision : adjust — verification confirmee :
- `residual_threats_acknowledged` : zero match dans
  `packages/nexus-coordinator/` (design-only S22 Phase F D1)
- `level_threat_note` : zero match dans `web/src/components/` (idem)
Phase B scope ajuste pour **inclure le backfill** des annotations S22
(+~70 LOC supplementaires : schema consent.json, endpoint coord,
composant React). Ce n'est pas du scope creep — c'est de la dette
S22 non livree que Phase B D4 absorbe naturellement.

**D5 ✅** — Aucun concern. Pas de changement.

---

## 5. Phase outline A..E

### Phase A — P2 batch S28 audit (8 items) + cold-start benchmark

Absorbe les 8 P2 de l'audit gate S28. Code + docs + benchmark :

- P2-REVIEW-1 : ajouter commentaire justificatif
  `#[allow(clippy::too_many_arguments)]` dans `llama_cpp.rs:258`
- P2-REVIEW-2 : documenter load assumption sampler chain rebuild dans
  `llama_cpp.rs` (commentaire inline : acceptable < 100 req/s)
- P2-B-1 : CI Linux/macOS writers — **scope-cut** (blocked on CI infra,
  pas actionnable sur Windows dev). Documenter dans commit body.
- P2-B-2 : ajouter test direct `init_platform_emitter()` dans
  `nexus-events-core` (trivial 7 LOC, 3 branches cfg)
- P2-C-1 : blob-serve isolation gap — sera naturellement documente par
  Phase C (broker garde blob-serve). Ajouter commentaire dans code.
- P2-C-2 : **cold-start benchmark RTX 5080** — mesurer spawn + IPC
  connect + Ollama 7B load + first token. Cible < 5s. Resultat dans
  commit body.
- P2-D-1 : ecrire **Note realisme S29-S30** dans HARDENING_ROADMAP §3
- P2-D-2 : ajouter §2.7 "Version verification at RFP time" dans
  `EXTERNAL_AUDIT_SCOPE.md`

Commit cible : `feat(sprint29): Sprint 29 Phase A — P2 batch S28 +
cold-start benchmark RTX 5080`

### Phase B — THREAT_MODEL §9 + responsible disclosure docs

Livrables docs + code frontend/backend :

- THREAT_MODEL.md §9 "Residual risks per-configuration" (6 sous-sections
  consent GPU / loopback tiers / duress / rate-limit / guardrails /
  capabilities)
- Renommer §9 actuel "Revue et evolution" en §10
- `GpuConsentDialog.tsx` : afficher `level_threat_note` tooltip
- `consent.json` endpoint : populate `residual_threats_acknowledged`
- `SECURITY.md` (racine) : responsible disclosure policy
- `.well-known/security.txt` (RFC 9116)
- `BUILDING.md` : instructions build "batteries included" pour auditeur

Commit cible : `feat(sprint29): Sprint 29 Phase B — THREAT_MODEL §9
per-mode risks + responsible disclosure`

### Phase C — Process isolation MVP : broker/executor split

Le gros de l'implementation technique :

- Nouveau crate `crates/nexus-executor/` (binaire) :
  - CLI : `nexus-executor --ipc-path <path> [--spawn-on-demand]`
  - IPC client : connect UDS/Named Pipe, JSON-RPC 2.0 handler
  - Task execution : dispatch vers worker-core engine
  - Health heartbeat 10s → broker
  - Graceful shutdown on `executor.shutdown` method
- Refactor `crates/nexus-shell-daemon-core/` :
  - Nouveau module `ipc_broker.rs` : spawn executor subprocess,
    UDS/NP server, JSON-RPC 2.0 dispatch, health monitoring,
    crash detection + backoff respawn
  - Update `runtime.rs` : ajouter executor lifecycle au boot sequence
  - Task routing : broker recoit task du coordinator proxy, forward
    via IPC a l'executor, retourne result
- `crates/nexus-events-core/` : ajouter `ExecutorCrash` +
  `BrokerCrash` variantes SecurityEvent
- W3C Trace Context : `traceparent` header dans chaque JSON-RPC
  request (preparation Phase D TraceProvider)
- Tests : spawn subprocess, IPC roundtrip, crash recovery, heartbeat
  timeout, task token ephemere

Commit cible : `feat(sprint29): Sprint 29 Phase C — process isolation
broker/executor split JSON-RPC 2.0 IPC`

### Phase D — TraceProvider opentelemetry 0.31

Infrastructure tracing pre-audit :

- Nouveau crate `crates/nexus-trace-core/` (lib) :
  - Trait `TraceProvider` + `TraceProcessor`
  - `BatchLogProcessor` : JSON structured → fichier (default)
  - `OtelProcessor` : OTLP bridge via opentelemetry 0.31
  - `SignedCanaryProcessor` : Ed25519-signed trace events
  - `set_trace_processors()` / `add_trace_processor()` API
  - `DOMAIN_TRACE_EVENT_V1` domain separation (design-only
    pre-launch stable)
- Wire dans broker startup (`runtime.rs`)
- Wire dans executor startup (`nexus-executor/src/main.rs`)
- W3C Trace Context extraction depuis JSON-RPC `traceparent` header
- Consumer : SecurityEvent emission routes vers TraceProvider
- Tests : processor pipeline, signed event roundtrip, OTel mock
  exporter, traceparent propagation

Commit cible : `feat(sprint29): Sprint 29 Phase D — TraceProvider
opentelemetry 0.31 backend-agnostic`

### Phase E — Wrap-up

Livrables :
- `sprint29_verification.md`
- `sprint30_audit_plan.md`
- `sprint29_carry_summary.md`
- Migration active → archive/v1.2/
- Updates CLAUDE.md (compteurs, etat), SPRINT_LOG, memory
- Scope freeze audit : documenter tip commit dans
  `EXTERNAL_AUDIT_SCOPE.md §2.7`

Commit cible : `chore(sprint29): Phase E — wrap-up + verification +
audit plan S30 + migration`

---

## 6. Items carry/dette

### Carry S29 (absorbes ou re-confirmes)

| ID | Description | Source | Reports | Status S29 |
|---|---|---|---|---|
| P2-REVIEW-1 | generate_blocking 12 params commentaire | S28 Phase A review | 1/3 | Phase A |
| P2-REVIEW-2 | Sampler chain load assumption commentaire | S28 Phase A review | 1/3 | Phase A |
| P2-B-1 | CI Linux/macOS writers | S28 Phase B review | 1/3 | **scope-cut** (blocked CI infra, carry S30 2/3) |
| P2-B-2 | init_platform_emitter test direct | S28 Phase B review | 1/3 | Phase A |
| P2-C-1 | blob-serve isolation gap | S28 Phase C review | 1/3 | Phase C (documente, pas resolu — Phase C broker garde blob-serve, resolution S30+) |
| P2-C-2 | Cold-start benchmark RTX 5080 | S28 Phase C review | 1/3 | Phase A |
| P2-D-1 | Note realisme S29-S30 | S28 Phase D review | 1/3 | Phase A |
| P2-D-2 | Version note at RFP time | S28 Phase D review | 1/3 | Phase A |

### Hors cap — items long-terme

| ID | Description | Condition | Status |
|---|---|---|---|
| T-NN+2 | iframe Rust-wasm (PATTERNS §P34) | tract opset 19 / ort wasm32-browser / gline-rs wasm-bindgen | Triggers inactifs |
| LT-1 | Kudos-v2 fairness reform | v1.0 + design doc + Gini > 0.70 | Latent |
| LT-2 | Radicle activation | tag v1.0 | Latent |
| LT-3 | Contribution family Sybil matrix | v1.0 + 3+ contrib non-compute | Latent |
| LT-4 | OS biometric gate | v1.0 + S30 FROST N1 + partnership | Latent |
| LT-5 | Redundancy persistence SQLite | multi-worker deploy OR v1.0 | Latent |
| LT-6 | iroh neighborhood enrichment | iroh > 0.97 OR v1.0 | Latent |

### Check ROADMAP_COMMITMENTS (G7 Regle 3)

Resultat : toutes les conditions de declenchement sont latentes
(tag v1.0 non pose, iroh toujours 0.97, Gini non mesurable pre-prod,
pas de multi-worker deploy, pas de partnership ONG formelle). Aucun
item LT ne redevient carry actif.

### Sprint impair — pas de phase dette obligatoire

S29 est impair → §6.2.1 Regle 1 ne s'applique pas. Les 8 P2 carry
sont absorbes en Phase A, pas dans une phase dette dediee.

---

## 7. Scope cuts — ce que Sprint 29 NE fait PAS

1. **D3 Windows RPC** → S30 (Named Pipe S16 suffit, co-landing VM)
2. **C4 task-scoped sandbox** → S30 (depend D2 stable)
3. **CI Linux/macOS writers** → S30 (P2-B-1, blocked CI infra, 2/3)
4. **Nym mixnet integration** → S30+ (SDK beta trigger INACTIVE)
5. **Tor transport** → S30+ (arti pre-1.0 trigger INACTIVE)
6. **Arti library-embed** → S30+ (conditionnel arti >= 1.0)
7. **Domain fronting implementation** → S30+ (legal review prereq)
8. **GPU lockup defense** → S30+ (dep A4 process roles post-D2)
9. **C1 SQLiteSession abstraction** → S30+
10. **Streaming bridge C5** → S30+
11. **blob-serve executor dedie** → S30+ (PROCESS_ARCHITECTURE §9 Q4)
12. **Full Gate 3 showcase app** → post-Gate 3
13. **Audit execution** → S30 (S29 prepare, engagement 4-8 sem)
14. **Remediation audit** → post-findings (S30 ou S31)
15. **opentelemetry 1.0 pin** → post-1.0 release (delayed)

---

## 8. Tracabilite scope

Table mappant les items "What's NOT" S28 au sprint de prise en charge :

| Item S28 scope cut | Sprint cible | Phase |
|---|---|---|
| D2 broker/executor implementation → S29 | **S29 Phase C** | absorbed |
| D3 Windows RPC → S29 | S30 | deferred (Named Pipe suffit S29) |
| C4 task-scoped sandbox code → S29 | S30 | deferred (depend D2 stable) |
| Nym mixnet integration → S30+ | S30+ | unchanged |
| MIG partitioning → post-v1.0 | post-v1.0 | unchanged |
| Tor transport → S30+ | S30+ | unchanged |
| Arti library-embed → S30+ | S30+ | unchanged |
| Domain fronting implementation → S30+ | S30+ | unchanged |
| GPU lockup defense → S29+ | S30+ | deferred (dep A4 process roles) |
| C1 SQLiteSession abstraction → S29+ | S30+ | deferred |
| Streaming bridge C5 → S29+ | S30+ | deferred |
| Full Gate 3 showcase app → post-Gate 3 | post-Gate 3 | unchanged |

---

## 9. Risk register

| ID | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R-S29-1 | Cold-start benchmark > 5s invalide l'archi pool mode | MED | HIGH | Phase A benchmark AVANT code Phase C. Si > 5s : evaluer Ollama `keep_alive` warm-start. Si > 10s : fallback spawn-on-demand only + re-design pool. |
| R-S29-2 | Refactor daemon split casse des tests existants | MED | MED | Tests existants (828 Rust) doivent rester verts. Le broker expose la meme API HTTP — les tests HTTP ne changent pas. Les tests internes qui mockent le runtime devront s'adapter. |
| R-S29-3 | opentelemetry 0.31 breaking API vs workspace deps | LOW | MED | OTEL confine dans crate `nexus-trace-core`, pas de contamination workspace. Feature-gate si conflit dep transitif. |
| R-S29-4 | Named Pipe perf Windows dev goulot (28µs vs 4.8µs Linux UDS) | LOW | LOW | 28µs << 100ms inference. Si mesure montre > 1ms : investiguer async Named Pipe tokio. |
| R-S29-5 | SecurityEvent enum breaking change (ajout ExecutorCrash/BrokerCrash) | LOW | LOW | Ajout de variantes = non-breaking en serde. Aucun decoder externe (pre-launch). |

---

## 10. Audit gate pattern — rappel

- Phase 0 audit S28 DONE (PASS, `102bc9f`).
- Phase E wrap-up produira `sprint29_verification.md` +
  `sprint30_audit_plan.md`.
- Phase 0 Sprint 30 (prochain audit) = audit independant de S29.

---

## 11. Checkpoint de validation

Questions pour arbitrage utilisateur AVANT plan detaille :

1. **D1** : Le split broker/executor via raw serde_json + tokio UDS/NP
   est-il acceptable, ou faut-il jsonrpsee pour la conformite JSON-RPC
   2.0 stricte ?

2. **D2** : Le cold-start benchmark comme Phase A prereq est-il le bon
   timing, ou faut-il un spike research dedie pre-kickoff ?

3. **D3** : opentelemetry 0.31 PLAN-ADAPT vs HARDENING_ROADMAP 0.27 —
   la justification (4 versions en retard, breaking changes 0.28) est-
   elle suffisante ?

4. **D4** : THREAT_MODEL §9 avec 6 sous-sections — faut-il ajouter
   la dimension process isolation (broker/executor) comme sous-section
   9.7 puisque Phase C l'implemente ce sprint ?

5. **D5** : L'engagement audit Trail of Bits S30 (vs S29) est-il
   acceptable ? S29 = preparation, S30 = envoi RFP + execution.
