---
written: 2026-04-20  # Sprint 22 hors-sprint (post Phase B `e9530c2`)
last_validated: 2026-04-20
triggers_revalidate:
  - "openai-agents-python release > 0.7.0 (breaking API changes Agent / Runner / Guardrail decorator)"
  - "microsoft/sudo release publique beyond Windows 11 24H2 inbox (audit trail ETW schema changes)"
  - "MCP spec revision Anthropic 2026+ (convert_schemas_to_strict semantics)"
  - "Sprint S+2 commence vs sprint cible (ex : S24 commence → re-scan S26 integration)"
  - "Gate 2 unlock effectif S22 Phase F (apps TransLingua/FamilyScan/EHPAD-Lien publiées)"
---

# Research — 18 features openai-agents-python + microsoft/sudo : matrice d'intégration S22 Phase F → LT-4

**Origine** : session orchestrateur 2026-04-20 (hors-sprint S22, post Phase B
`e9530c2` GLiNER span-logits decoder). L'utilisateur a demandé une analyse
objective des deux repos suivants pour identifier des **features produit**
(pas roadmap) potentiellement applicables à SBFB :

- [openai/openai-agents-python](https://github.com/openai/openai-agents-python) —
  framework agent LLM Python (agents, handoffs, guardrails, sessions,
  tracing, function tools, streaming, MCP servers).
- [microsoft/sudo](https://github.com/microsoft/sudo) — sudo pour Windows
  (élévation process depuis console unelevated, 3 modes trade-off
  UX/sécurité, RPC handle forwarding, ETW audit, opt-in OS Settings).

Cette note trace le mapping factuel 18 features → sprints/phases/
carry-overs du planning existant (`docs/security/HARDENING_ROADMAP.md` §3
S22-S30 + `docs/release/ROADMAP_COMMITMENTS.md` LT-1/2/3 + S22
kickoff/plan en cours). Pattern recherche : symétrique à
`.planning/research/S22_contribution_family_sybil_matrix.md` (commit
`dbc4ceb`) — synthèse 4 agents parallèles indépendants écrivant vers
disque via Write tool.

---

## 1. Les 18 features identifiées

Analyse deep 2026-04-20 des deux repos via context7 (`/openai/openai-
agents-python`, `/websites/urchade_github_io_gliner` référence
décodeur, doc Microsoft Learn sudo, blog devblogs.microsoft.com). Chaque
feature est décrite comme **primitive produit** (ce qu'elle permet),
pas comme implémentation (delta technique laissé aux design docs
respectifs par sprint).

### Cluster A — Observability (4 features)

- **A1 — Lifecycle hooks API** : `RunHooks` class avec `on_agent_start
  / on_agent_end / on_tool_start / on_tool_end / on_handoff`, injectable
  via `Runner.run(hooks=)`. Permet consumer extérieur d'observer
  événements typés sans coupler callsites. Pour SBFB : API
  `TaskDispatchHooks` (on_claim_broadcast / on_task_dispatched /
  on_result_received / on_validator_post_task / on_quarantine_enqueue)
  remplace l'observabilité ad-hoc dispatcher.py S20 Phase D.
- **A2 — Tracing provider backend-agnostic** : `TraceProvider` global +
  `BatchTraceProcessor` default (OpenAI backend) + `add_trace_processor()
  / set_trace_processors()`. Permet export OTEL Grafana Tempo / Jaeger
  ou custom processor (exemple : Ed25519-signed trace events pour
  audit chain-of-custody judiciaire).
- **A3 — OS audit channel** : `sudo_events/` module dédié écrit vers
  ETW Windows. Permet événements critiques (panic wipe, rotation
  token, quarantine drop, Sybil admission reject) visibles SIEM
  entreprise (Splunk/Sentinel/QRadar) sans parser logs applicatifs.
  Pour SBFB : `nexus-events-core` avec writers platform-native
  (`tracing-etw` Windows, `sd_journal_send` Linux, `os_log` macOS).
- **A4 — Process role tagging** : sudo documente hierarchy explicite
  unelevated-client / elevated-broker-middle / target-child. Permet
  antivirus corporate et policy AppArmor/SELinux/Landlock d'appliquer
  règles différentiées par rôle. Pour SBFB : `ProcessRole` enum
  (Launcher / Daemon / Worker / OllamaRuntime / IframeBlobServe)
  injecté au spawn via env var HMAC-signée, base cgroups v2 / Job
  Object / launchd label.

### Cluster B — Guardrails & Policy (4 features)

- **B1 — Guardrails decorator+tripwire** : `@input_guardrail /
  @output_guardrail` → `GuardrailFunctionOutput(output_info,
  tripwire_triggered: bool)` → exception typée
  `InputGuardrailTripwireTriggered / OutputGuardrailTripwireTriggered`.
  Permet pipeline déclaratif composable (chain multi-checkers avec
  typed abort), remplace logique if/else éparse. Pour SBFB : refactor
  rétroactif des 6 primitives S16-S22 (pii_redact, output_filter,
  quarantine_queue, canary_input, rate_limit, invisible_text) vers un
  contrat `Guardrail` unifié + SDK wrapper iframe via bridge P24
  whitelist extension.
- **B2 — MCP server exposition** : `Agent(mcp_servers=[server])` consume
  MCP tools externes ou expose agent SBFB comme MCP server. Permet
  **expansion majeure du canal d'accès** : utilisateur Claude
  Desktop / ChatGPT invoque SBFB comme outil sans jamais installer le
  client SBFB. 3 methods bridge S13 (task_submit, storage_get,
  storage_set) mappées vers 3 MCP tools avec JSON schema strict +
  rate-limit per-session S22 Phase A reuse.
- **B3 — Output type Pydantic auto-derivation** : `Agent(output_type=
  Pydantic)` → JSON Schema auto + validation final_output. Pour
  SBFB : S20 Phase D `task_response.schema.json` écrit manuellement +
  `schemars` Rust dérivation, coté Python coord schema manuel =
  drift-prone. Dérivation Pydantic `TaskResponsePydantic.
  model_json_schema()` avec test contract fail-fast drift.
- **B4 — Per-mode residual risk documentation** : sudo docs listent
  explicite threats résiduels par mode (`normal` = medium-integrity
  drive elevated ack ; `disableInput` = mitige stdin ; `forceNewWindow`
  = isolation max). Pour SBFB : `THREAT_MODEL.md §9 "Residual risks
  per-configuration"` avec sous-sections par feature configurable
  (consent GPU 4 niveaux, loopback modes, duress PIN, rate-limit
  tiers, guardrails disabled combos). Annotations in-product via
  `consent.json` field `residual_threats: [...]` visible UI launcher.

### Cluster C — SDK abstractions + Streaming (5 features)

- **C1 — SQLiteSession abstraction** : `SQLiteSession(id, db_path)` →
  `get_items / add_items / pop_item / clear_session`. Pour SBFB :
  abstraction unifiée des 5 stores SQLite ad-hoc (`quarantine_queue`,
  `kudos`, `canary_registry`, `contributor_registry` S22 Phase C,
  `upload_queue`). Permet undo natif (`pop_item`) cross-module + CLI
  admin unifiée `sbfb session <module> list/pop/clear`.
- **C2 — `@function_tool` auto-schema SDK** : decorator auto-schema
  Python typed + Pydantic Field constraints. Pour SBFB : `@task_handler`
  SDK generating automatically task request/response schema depuis
  Pydantic class + validation runtime + manifest auto-export. Réduit
  boilerplate app publisher ~10x (publier une app = 10 lignes vs
  100+ actuellement).
- **C3 — Handoffs semantic** : `handoff(agent, on_handoff=callback,
  input_filter=fn, is_enabled=bool|fn)`. Pour SBFB : dispatcher worker
  re-assignment explicite (worker rate-limited → handoff autre worker
  avec `input_filter` re-redact PII policy-target + `on_handoff` crédit
  no-show kudos + `is_enabled` skip low-reputation/Sybil-reject).
  Remplace round-robin dispatcher.py actuel.
- **C4 — Sandbox per-run task-scoped** : `SandboxRunConfig(session=
  sandbox)` async context manager sandbox éphémère per-execution.
  Pour SBFB : iframe per-task avec TTL 30s post-task + fresh Pyodide
  interpreter + cache LRU `(app_hash, task_id)`. Critique Gate 3+ apps
  T3+ (PolitiScan, NEXUS cold-case, LibanLive) pour chain-of-custody
  judiciaire (pas de leak évidence cross-dossier).
- **C5 — Streaming bridge events** : `Runner.run_streamed()` →
  `result.stream_events()` discriminated union `raw_response_event /
  run_item_stream_event`. Pour SBFB : **nouvelle méthode whitelist
  bridge P24** `task_submit_streaming` avec wire format
  `bridge.schema.json` discriminated union `{token / tool_called /
  pii_masked / done / error}`. UX majeure apps LLM (visible token-par-
  token vs attente résultat final 10-30s).

### Cluster D — Process & OS integration (5 features)

- **D1 — Three-mode trade-off doc pattern** : sudo 3 modes documentés
  avec trade-off UX/security explicite + threat per mode. Pour SBFB :
  enrichir 4 niveaux consent GPU S16 Phase C avec `threat_note:
  &'static str` per niveau + extension pattern aux 3 endpoints
  loopback risky S16/S20 (panic wipe, rotate token, unlock duress) via
  nouveau `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md`.
- **D2 — Broker/executor process split** : sudo middle-tier elevated
  bridge isolé du target-child. Pour SBFB : split strict broker
  (peer-cred gated, long-lived, surface minimale) vs executor
  (short-lived, load model Ollama/llama.cpp, crash indep) — alignement
  naturel avec `RUNTIME_ISOLATION.md §3 Phase C` WSL2 / Virtualization.
  framework / systemd-nspawn (HARDENING §3 S28).
- **D3 — Windows RPC + OS-authenticated handle forward** : sudo utilise
  `RpcBindingInqAuthClient*` = OS fournit SID caller authentifié +
  handle forwarding auto. Pour SBFB : remplace `named_pipe_server.rs`
  SDDL DACL manuel S16 Phase B par Windows RPC = délégation OS auth
  (~300 LOC custom supprimés, surface attaque réduite).
- **D4 — OS biometric gate (UAC / Windows Hello / TouchID / polkit)** :
  sudo utilise UAC single gate non-bypassable par process browser-
  level. Pour SBFB : appliquer gate biométrique sur 5 ops critiques
  (panic wipe, duress unlock, token rotate force, consent tier bump
  `L1→L4`, federation canary FROST cosign S30 Niveau 1). Protège
  populations Gate 4 (journalistes/activistes/ONG) contre malware
  user-mode browser-injected.
- **D5 — Capability toggle opt-in OS-level** : sudo entièrement OFF
  jusqu'à activation Settings → System → Advanced → Developer
  Features. Pour SBFB : `capabilities.toml` avec `[capability.
  tool_calling / streaming_bridge / federation_canary /
  rag_retrieval / mcp_server_expose]` toutes `enabled = false` par
  défaut. Activation via binaire séparé `nexus-admin` Typer CLI qui
  vérifie admin privilege OS-level (EUID Unix / `IsUserAnAdmin()` +
  Integrity Level High Mandatory Label Windows).

---

## 2. Bénéfices produit par persona

Le mapping feature → bénéfice est validé factuellement (pas
spéculatif) en référence aux threats couverts dans `THREAT_MODEL.md`
+ `ADVERSARIES.md` + `HARDENING_ROADMAP.md`.

### Consumers (utilisateurs finaux apps)

- **Streaming live tokens** (C5) : UX ChatGPT-style pour apps LLM.
- **Fiabilité handoff worker** (C3) : si worker tombe, task bascule
  sans re-exposer PII ni perte.
- **Zéro fuite cross-task** (C4) : apps médicales/juridiques isolent
  dossiers cryptographiquement.
- **Consentement éclairé** (B4, D1) : comprendre ce qu'on active
  avant de l'activer.

### App publishers (contributeurs code)

- **SDK 10× plus simple** (C2) : publier app = 10 lignes Python vs
  boilerplate 100+ lignes manuel.
- **Filtres métier custom** (B1) : apps spécialisées (médical,
  juridique, finance) ajoutent leurs règles.
- **Dashboards custom** (A1, A2) : instrumenter son app + export
  OTEL vers Grafana/Datadog/Jaeger habituel.

### Admin DSI / entreprise (adoption institutionnelle)

- **Compatibilité SIEM** (A3) : events SBFB dans Splunk/Sentinel/
  QRadar sans parse logs.
- **Whitelisting AV précis** (A4) : process tagués par rôle,
  policies IT granulaires.
- **Features risky off par défaut** (D5) : install vanilla = surface
  minimale, admin toggle pour activer.
- **SLA meilleur** (D2) : crash Ollama ne coupe pas P2P.
- **Install Windows fiable** (D3) : API Microsoft officielle vs
  SDDL manuel source bugs.

### Utilisateurs à haut risque (journalistes, avocats, ONG — Gate 3/4)

- **Anti-malware browser** (D4) : ops critiques requièrent biométrie
  OS non-bypassable.
- **Chain-of-custody judiciaire** (C4) : isolation cryptographique
  dossiers distincts = attendu tribunaux.
- **Minimal surface par défaut** (D5) : zéro feature risky active
  sans activation explicite.
- **DPIA/RGPD compliance** (B4) : résidus documentés formels par
  config choisie.

### Workers compute (contributeurs GPU)

- **Kudos justes en handoff** (C3) : rate-limit légitime ≠ no-show
  volontaire.
- **Undo natif** (C1) : rollback ops via `pop_item` cross-module.
- **Clarté modes consent** (D1) : risque par niveau lisible UI.

### Écosystème global

- **SBFB utilisable Claude/ChatGPT** (B2) : expansion canaux d'accès
  ~100× (agents LLM externes deviennent consumers potentiels sans
  installer client SBFB).
- **Schema drift réduit** (B3) : moins de bugs prod cross-boundary
  app/coord.

**Bénéfice différenciant principal** : B2 MCP + C5 streaming
élargissent le marché cible ~100×. D4 biométrie + C4 sandbox
rendent SBFB défendable face aux populations vulnérables
ciblées Gate 4. Les deux axes revendiqués par la plateforme ne
sont aujourd'hui que partiellement adressés.

---

## 3. Matrice d'intégration sprint-by-sprint

Produite par 4 agents parallèles indépendants (cluster A/B/C/D),
chaque agent lisant `HARDENING_ROADMAP.md` + `ROADMAP_COMMITMENTS.md`
+ sprint kickoff/plan en cours + `README.md §6.2.1+§6.7+§6.9` +
`PATTERNS.md`. Contrainte commune : **préférer la solution la plus
poussée** (feedback `memory/feedback_approach.md`) + **pre-launch
protocol policy respectée** (zéro bump `*_VERSION` jusqu'à v1.0) +
**cap G7** (max 2 carry-overs/sprint).

| Sprint | Feature | Cluster | Mode intégration | Deps bloquantes |
|---|---|---|---|---|
| **S22 Phase F** wrap | D1 Three-mode trade-off doc | D | Absorbé Phase F (doc-only) | — |
| **S22 Phase F** opportuniste | B3 Pydantic auto-derivation | B | Phase F opportuniste OU carry S23 | — |
| **S23 chore hors-sprint** | D5-design `CAPABILITY_TOGGLES.md` | D | Amendement HARDENING §3 S23 (pattern `dbc4ceb`) | — |
| **S23 chore hors-sprint** | B1-design `GUARDRAILS_ARCHITECTURE.md` | B | Amendement HARDENING §3 S23 | — |
| **S23** | B1 Guardrails refactor (pipeline + 6 primitives) | B | Item net-new S23 | B1-design landed |
| **S24** | A1 Lifecycle hooks API (`TaskDispatchHooks`) | A | Amendement §3 S24 (consumer natural = re-run sampling) | S22 Phase A ✓ |
| **S24** | C3 Handoffs semantic dispatcher | C | Phase dédiée S24 | S22 Phase A ✓, S22 Phase C, S23 redundancy voting |
| **S25** | D5-implem `nexus-admin` CLI + `capabilities.toml` | D | Amendement §3 S25 (prérequis tool-calling) | D5-design |
| **S25** | A3 OS audit channel `nexus-events-core` (ETW/journald/oslog) | A | Item net-new S25 | A1 landed |
| **S25** | B2 MCP server exposition | B | Extension RAG phase S25 | B1 + B3 landed |
| **S25** | C2 `@task_handler` SDK auto-schema | C | Phase SDK S25 | B3 landed |
| **S25** | C5 Streaming bridge events (`bridge.schema.json` + `task_submit_streaming`) | C | Phase bridge S25 | S20 Phase D ✓, S21 Phase B ✓ |
| **S26** | C1 SQLiteSession abstraction (crate `nexus-session-store` + migrate 5 stores) | C | Thème principal S26 (refactor cross-module) | S22 Phase C stabilisé |
| **S26** | A4 Process role tagging (cgroups/Job Object/launchd) | A | Amendement §3 S26 (GPU lockup prérequis) | — |
| **S28** | D2 Broker/executor process split + `PROCESS_ARCHITECTURE.md` | D | Amendement §3 S28 Phase A | S20 Phase A/B ✓ |
| **S28** | D3 Windows RPC + OS-auth handle forward | D | Amendement §3 S28 Phase B | D2 co-landing |
| **S28** | C4 Sandbox per-run task-scoped | C | Phase dédiée S28 (cohérence isolation) | C5 landed (correlation ID stable cross-iframe) |
| **S29** | A2 Tracing provider backend-agnostic (OTEL + Ed25519 signed processor) | A | Amendement §3 S29 (pre-audit prep) | A1 + A3 landed |
| **S29** | B4 Per-mode residual risk doc (`THREAT_MODEL §9`) | B | Amendement §3 S29 (pre-audit Cure53/ToB) | Tous modes S16-S28 landed |
| **LT-4 post-v1.0** | D4 OS biometric gate (Hello/TouchID/polkit) | D | Nouvelle entrée `ROADMAP_COMMITMENTS.md` | Tag v1.0 + S30 FROST N1 + partnership OpSec review |

---

## 4. Sprint-load check (warnings capacity)

| Sprint | Charge additive | Warning |
|---|---|---|
| S22 Phase F | +D1 doc + peut-être B3 | OK (Phase F existante déjà +50 LOC wrap + process fixes P2-S21-4/5 ; +D1 ~150 LOC docs ne casse pas) |
| S23 | +B1 refactor + 2 design docs chore hors-sprint | **Attention** — S23 déjà chargé (~2220 LOC per §3 S23 actuel : ephemeral workers + escalating PoW + honeypot + redundancy voting + Couche 3 design + contribution families + /diagnostic/fairness). B1 refactor = +~800 LOC + tests. **Cap budget sprint (norme ~1500-2500 LOC) en limite supérieure.** |
| S24 | +A1 + C3 (+~1100 LOC) | OK (§3 S24 actuel ~1400 LOC, +A1 ~400 + C3 ~700 = ~2500 LOC, dans norme) |
| S25 | +D5-implem + A3 + B2 + C2 + C5 (5 features, +~3000 LOC) | **FAT** — §3 S25 actuel ~1200 LOC Tor+RAG+quota, +~3000 LOC cluster = ~4200 LOC total. **Dépasse largement norme.** Split recommandé en S25a + S25b OU priorisation stricte (D5+B2 prioritaires pour unlock tool-calling ; A3+C2+C5 carry S26-S27). |
| S26 | +C1 + A4 (+~1200 LOC) | OK (§3 S26 actuel ~800 LOC Tor complete + curator + GPU lockup, +C1 ~800 + A4 ~400 = ~2000 LOC, dans norme) |
| S28 | +D2 + D3 + C4 (+~1700 LOC) | **Attention** — §3 S28 actuel ~2200 LOC Nym+MIG+audit prep, +~1700 cluster = ~3900 LOC. **Dépasse norme ~40%**. D3 peut défer S30 ou LT si context7 Windows RPC crate immature. |
| S29 | +A2 + B4 (+~800 LOC) | OK (§3 S29 actuel ~1700 LOC remediation buffer, +~800 = ~2500 LOC, dans norme) |

**Risques concrets** :

1. **S25 surcharge 5 features** (cross-clusters) → split ou
   priorisation. Recommandation factuelle : **D5-implem +
   B2 MCP** sont les 2 features qui débloquent tool-calling (prévu
   S25 via HARDENING §3 S25 ligne 389 "RAG sanitization pipeline").
   **A3 + C2 + C5** carry S26-S27 acceptable.
2. **S28 surcharge 3 features + Nym/MIG existant** → D2+C4
   co-landing obligatoire (cohérence thème isolation), D3 Windows-
   specific défer S30 ou LT-4 (éviter bloquer Gate 4 prep avec
   dep crate Windows RPC immature).
3. **B1 S23 refactor structurant** — si S23 capacité saturée
   (~2220 LOC actuel), B1 ~800 LOC peut passer ~3000 LOC. Design
   doc hors-sprint pré-S23 (chore commit) + décision arbitrage
   user au kickoff S23.

---

## 5. Graphe de dépendances critique

```
B3 (S22F ou S23) ──→ C2 (S25)   [partagent type TaskResponse]
                 └─→ B2 (S25)   [MCP tool schema source]

B1 (S23) ─────────→ B2 (S25)    [MCP = guardrail chain entry]
              └───→ B4 (S29)    [residual risk doc B1 disable]
              └───→ C5 (S25)    [stream event = guardrail pipeline]

A1 (S24) ─────────→ A2 (S29)    [events typés flow provider]
              └───→ A3 (S25)    [event enum partagée]

D5-design (S23) ──→ D5-implem (S25) ──→ tool-calling réactivation
                                                  (PR-block Semgrep enforcement)

D2+D3 co-landing (S28) — RPC utile ssi broker/executor split existe

C5 (S25) ─────────→ C4 (S28)    [streaming survive re-création iframe task-scoped]

D1 (S22F) ────────→ B4 (S29)    [threat_note vocabulaire réutilisé]
```

---

## 6. Cap G7 + pre-launch policy bilan

- **0 slot carry-over formel consommé pre-v1.0**. Toutes les features
  entrent via :
  - Amendements HARDENING §3 S22-S29 existants (pattern `88eee23`)
  - Items net-new sprint (pattern S22 Phase C Couches 1+2)
  - Chore hors-sprint (pattern `dbc4ceb` + `88eee23`)
  - Phase F absorption S22 (doc-only, D1)
  - Entrée LT-4 post-v1.0 pour D4 biométrie (hors cap G7 formel,
    pattern LT-1/2/3)

- **Pre-launch protocol respectée** : aucune feature ne bump `BLOB_
  VERSION / TASK_RESPONSE_VERSION / CANARY_VERSION / ANNOUNCEMENT_
  VERSION / PROVENANCE_VERSION / CURATOR_LIST_VERSION / AGE_WITNESS_
  VERSION / CONTRIBUTOR_ATTESTATION_VERSION / DELEGATION_CERT_
  VERSION`. Nouveaux wire format introduits design-only pre-launch
  stable :
  - `DOMAIN_TRACE_EVENT_V1` (A2 trace provider processor)
  - `DOMAIN_OS_AUDIT_EVENT_V1` (A3 nexus-events-core)
  - `bridge.schema.json` extension P24 (C5 streaming, pattern P24
    whitelist extended additively across sprints — never bumped)
  - `capabilities.toml` schema (D5 opt-in toggle)
  - `PROCESS_ROLE_SIGNING_V1` (A4 env var HMAC anti-spoof)

- **Day 0 sprints gelées** : aucune feature ne re-bat de D1..D5
  sprint kickoff acté. S22 Day 0 (D1 Sybil 3 couches / D2 scope γ
  hybride / D3 NVML baseline / D4 watermark canari / D5 cap G7 +
  LT-2) préservé intégralement.

---

## 7. Actions procédurales immédiates

Les actions suivantes sont **procédurales non-controversées**
(précédent `dbc4ceb` + `88eee23`) et peuvent être exécutées dans le
même commit `chore(planning)` que ce research doc :

1. **Phase F S22** : amender `.planning/active/sprint22_plan.md §9`
   avec item D1 + fichier `docs/security/LOOPBACK_ENDPOINTS_TRUST_
   TIERS.md` (doc-only, ~150 LOC).
2. **HARDENING §3 amendements** :
   - S22 Phase F : mention D1 via §9.1 item
   - S23 : amendement "B1 guardrails refactor + 2 design docs
     chore hors-sprint pré-kickoff" + scope warning capacity
   - S24 : amendement "A1 TaskDispatchHooks + C3 handoffs semantic
     dispatcher"
   - S25 : amendement "D5-implem + A3 + B2 + C2 + C5 (5 features,
     split/priorisation requise au kickoff)"
   - S26 : amendement "C1 SQLiteSession abstraction (crate dédiée)
     + A4 Process role tagging"
   - S28 : amendement "D2+D3 broker/executor split + C4 sandbox
     per-run (cohérence isolation)"
   - S29 : amendement "A2 TraceProvider + B4 per-mode residual
     risk doc"
   - Frontmatter `last_validated` bump + `audited_findings` nouvelle
     entrée 2026-04-20
   - `triggers_revalidate` nouveau trigger "openai-agents-python
     release > 0.7.0" + "MCP spec revision Anthropic"
3. **ROADMAP_COMMITMENTS** : ajouter entrée LT-4 "D4 OS biometric
   gate cross-platform" + update index table.
4. **Design doc stubs** :
   - `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` (D1, ~150
     LOC, body substantiel car absorption S22 Phase F imminente)
   - `docs/security/CAPABILITY_TOGGLES.md` (D5, ~200 LOC, body
     design substantiel car S23 chore pré-kickoff + S25 consumer)
   - `docs/security/GUARDRAILS_ARCHITECTURE.md` (B1, ~250 LOC, body
     design substantiel car S23 kickoff vient)

**Arbitrages utilisateur différés** (3 options factuellement
viables, à décider au kickoff S23 + S25) :

- **B1 timing** : S23 dédié (risque sur-capacité) OU split S24-S25
  OU défer S27 (après Sybil mature).
- **S25 split** : 5 features → priorisation {D5+B2} prioritaires
  (tool-calling unlock) vs {A3+C2+C5} carry S26-S27.
- **D5 enforcement** : Semgrep custom rule PR-block (strict) OU
  audit CI manuel (souple) OU les deux.

Ces 3 arbitrages sont laissés au kickoff des sprints concernés
(pattern S22 Phase A R1 arbitrage post-G8 pivot Option C
`60adceb`).

---

## 8. Conformité au principe horizon long-terme (§6.7)

Chaque feature retenue adopte la solution la plus poussée :

- A1 — trait Rust + PyO3 binding + multi-hook fan-out typed
  (vs callbacks ad-hoc).
- A2 — OTEL SDK + W3C Trace Context propagation cross-process +
  signed canary processor (vs log JSON structured only).
- A3 — 3 writers platform-native ETW/journald/oslog (vs single
  backend).
- A4 — `ProcessRole` enum signée HMAC + multi-OS tagging
  (cgroups/Job Object/launchd label) + policy Landlock/AppArmor/
  SELinux base.
- B1 — refactor rétroactif 6 primitives vers contrat unifié + SDK
  wrapper iframe (vs pipeline ad-hoc réparé).
- B2 — MCP server spec strict + consent GPU integration + rate-
  limit per-session reuse (vs opening generic tool registry).
- B3 — `datamodel-code-generator` dérivation + test contract
  fail-fast drift (vs schema manuel vérifié à l'œil).
- B4 — section `THREAT_MODEL §9` structurée 9.1-9.5 + annotations
  runtime `consent.json` field + UI exposition (vs appendix doc
  seul).
- C1 — crate Rust `nexus-session-store` + PyO3 binding (vs helper
  Python unique).
- C2 — decorator introspection + manifest auto-export + lien
  typé TaskResponse (vs générateur manuel).
- C3 — design doc long-life + trait Handoff + 3 paramètres
  orthogonaux (on_handoff/input_filter/is_enabled) (vs
  round-robin amélioré).
- C4 — iframe per-task + fresh Pyodide + TTL + cache LRU +
  alignement runtime isolation WSL2 conditional Gate 3+ (vs
  flag AppManifest seul).
- C5 — wire format `bridge.schema.json` formel + discriminated
  union + 3-browser Playwright test + buffering window design
  pour output_filter InvisibleText cross-chunk (vs chunk naïf).
- D1 — `threat_note` Rust attribute + UI exposition + extension
  loopback endpoints design doc (vs commentaires README).
- D2 — design doc `PROCESS_ARCHITECTURE.md` long-life + split
  crate + alignment RUNTIME_ISOLATION.md (vs refactor inline
  silencieux).
- D3 — `windows-rs` Windows RPC delegation (~300 LOC custom
  supprimés) + alignment D2 split (vs patch SDDL amélioré).
- D4 — 3 crates cross-platform + 5 endpoints critiques + LT-4
  post-v1.0 (vs feature Windows-only seule).
- D5 — `nexus-admin` binaire séparé + admin privilege check
  cross-OS + Semgrep PR-block enforcement (vs flag config
  applicatif).

---

## 9. Références

- `docs/security/HARDENING_ROADMAP.md` §3 S22-S30 (amendements cible)
- `docs/release/ROADMAP_COMMITMENTS.md` LT-1/2/3 (pattern LT-4)
- `.planning/active/sprint22_plan.md §9` (Phase F amendement D1)
- `.planning/research/S22_contribution_family_sybil_matrix.md`
  (pattern research doc + commit `dbc4ceb`)
- `memory/feedback_approach.md` (principe deepest technical option)
- `docs/claude/README.md §6.2.1 + §6.7 + §6.9`
- Sources externes (context7 fresh 2026-04-20) :
  - `/openai/openai-agents-python` (Agent, Runner, Guardrails,
    Sessions, Hooks, Tracing, function_tool, Handoffs, Streaming,
    MCP)
  - `/websites/urchade_github_io_gliner` (decoder.py canonical,
    utilisé pour S22 Phase B preflight G8)
  - [Microsoft Learn Sudo for Windows](https://learn.microsoft.com/en-us/windows/sudo/)
    (3 modes forceNewWindow/disableInput/normal)
  - [devblogs.microsoft.com Introducing Sudo for Windows](https://devblogs.microsoft.com/commandline/introducing-sudo-for-windows/)
    (RPC + handle forwarding architecture)
