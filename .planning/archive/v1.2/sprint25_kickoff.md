# Sprint 25 — Kickoff (Key rotation + C3 handoffs + D5 capabilities + P2 batch)

**Ecrit** : 2026-04-22 (session fraiche post-audit gate S24 `358f166`).
**Type** : **sprint implementation** (fondations securitaires pre-tool-calling :
rotation cles + pipeline guardrails multi-stage + capability gates).
**Tip master d'entree** : `358f166` (chore(sprint24): audit gate S24 PASS).
**Phase 0 audit Sprint 24** : **DEJA JOUE** — findings dans
`.planning/archive/v1.2/sprint24_audit_findings.md` (verdict **PASS**,
0 P0/P1, 2 P2 carry, 1 P3 nit). Migre vers `archive/v1.2/` dans ce
commit d'ouverture S25.

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan complet** (2026-04-22, 6 triggers scannes) :
  - `arti-client > 1.x stable` : **INACTIVE** (0.37.0 crates.io,
    projet Arti 1.5.0 mais crate API pre-1.0). Tor transport S26+.
  - `openai-agents-python > 0.7.0` : **INACTIVE** (v0.14.3 inchange).
  - `frost-ed25519 > 2.1` : **INACTIVE** (2.1.0 inchange).
  - `MCP spec revision Anthropic 2026+` : **ACTIVE** — vulnerabilite
    STDIO transport RCE avril 2026 (~200k serveurs, OX Security /
    The Register 2026-04-16). Spec version 2025-11-25 inchangee mais
    faille design confirmee (Anthropic decline fix architectural).
    **Impact S25** : renforce urgence D5 capability gates
    (`mcp_server_expose` gate-off-by-default) + B2 MCP server expose
    S26 devra integrer mitigations (sandbox STDIO, validation config,
    verified sources). D5 ce sprint = prereq B2.
  - `wasmtime LTS bump` : INACTIVE.
  - `microsoft/sudo > 24H2` : INACTIVE.
- **context7** `ed25519-dalek` (2026-04-22) — API `SigningKey::sign` /
  `VerifyingKey::verify` stable 2.x. Pas de primitive de rotation
  native (attendu — rotation = protocole applicatif, pas primitif
  crypto). Pattern : self-signed rotation announcement.
- **Code review** : `crates/nexus-core-rs/src/crypto.rs` (KeyPair sign/
  verify), `curator.rs` (CuratorList + CuratorListEntry sign/verify),
  `canonical.rs` (12 DOMAIN_*_V1 constants), `dispatcher.py`
  (input_chain: GuardrailChain), `guardrails.py` (Guardrail ABC +
  GuardrailChain), `docs/security/CAPABILITY_TOGGLES.md` (design
  complet D5).

---

## 1. Constat d'entree

### 1.1 D'ou on part

- **Tip** : `358f166` — S24 audit gate PASS, 0 P0/P1.
- **Working tree** : propre (post-migration `sprint24_audit_findings.md`
  → `archive/v1.2/`).
- **v1.2** : continuation security hardening. Pas de nouvelle version.

### 1.2 Ancrage HARDENING_ROADMAP

HARDENING_ROADMAP §3 S25 : "Tor transport phase 1 + per-app quota +
RAG + pluggable transports + D5 capabilities + A3 OS audit + B2 MCP +
C2 SDK + C5 streaming bridge". Sprint prescrit **FAT (~3700 LOC, +50%
norme)**. Arbitrage utilisateur 2026-04-22 : prioriser les 2 carries
formels (key rotation + C3 handoffs) + D5 capabilities (prereq B2 MCP,
reponse G2 trigger MCP vuln). Tor → S26 (arti 0.37.0 pre-1.0).
B2/A3/C2/C5/RAG → S26-S27.

### 1.3 Compteurs tests entree (tip `358f166`)

| Suite | Count | Notes |
|---|---|---|
| Rust nextest | 757 | all pass |
| Rust doctests | pass | |
| Python SDK | 185 | all pass |
| Python coord | 315 pass + 32 fail + 3 skip | 32 fail = stale PyO3 wheel (pre-existing) |
| Python gov | 46 | all pass |
| Vitest | 264 | all pass |
| Playwright | 43 | all pass |
| Size-limit | 7/7 | |
| **Total** | **~1621** | |

### 1.4 Pre-launch protocol policy

Pas de deploiement live. `*_FORMAT_VERSION` restent a 1. Pas de
tolerant decoder multi-version. Cf. `CLAUDE.md §Pre-launch protocol
policy`. Key rotation (Phase B) ajoute `DOMAIN_KEY_ROTATION_V1` +
`KEY_ROTATION_FORMAT_VERSION = 1` — nouveau wire format pre-launch
stable.

---

## 2. Goal

Solder les 2 carries formels S24 (key rotation ceremony + C3
handoffs semantic), poser le systeme de capability gates (D5)
comme prereq au MCP server expose S26 (reponse trigger G2 vuln
MCP avril 2026), et nettoyer les P2 DNS/quarantine.

**Critere SMART : 25+ rows fail-fast verts au `verification.md`,
mesure binaire au Phase E wrap-up.**

---

## 3. Phase 0 — Audit gate Sprint 24

**Verdict** : PASS (0 P0/P1, 2 P2 carry, 1 P3 nit).
**Commit** : `358f166` — `.planning/active/sprint24_audit_findings.md`
migre vers `archive/v1.2/` dans ce commit d'ouverture.
**P2 carry S25** : P2-E-1 (TLS per-endpoint) + P2-E-2 (DoH concurrent)
absorbes Phase A ci-dessous.

---

## 4. Decisions Day 0 (D1..D5 gelees)

### D1 — Key rotation ceremony : self-signed rotation announcement + gossip revocation

**Retenu** : nouveau struct Rust `KeyRotationAnnouncement` (dans
`nexus-core-rs`) contenant `old_public_key`, `new_public_key`,
`timestamp`, `reason: String`, **signe par l'ancienne cle** (preuve
de possession). Publication sur gossip topic dedie
`nexus-grid/key-rotation/v1`. Nouveau DOMAIN separation
`DOMAIN_KEY_ROTATION_V1`. Workers observent les 2 cles pendant une
fenetre de transition configurable (defaut 7 jours). Apres expiration,
ancienne cle = rejetee. Revocation list = in-memory `HashMap<PublicKey,
RevocationEntry>` dans le shell-daemon (pas de persistence SQLite —
pre-v1.0, 0 node tiers). `CuratorListEntry::verify_signature` mis a
jour pour checker la revocation list avant accept.

Pattern inspire de : SSH key rotation (old key signs the new key),
Keybase device revocation (signed revocation statement), Matrix key
backup rotation (self-signed cross-signing).

**Rejete** :
- **Certificate Authority model** (CA signe les cles) : centralise,
  contredit Day 0 #1 "zero central authority". Les curators n'ont
  pas de CA au-dessus d'eux.
- **Web of Trust PGP-style** (peers co-signent) : complexite N^2,
  pas de UX viable pour un reseau P2P automatise. Adapte humains
  manuels, pas bots.
- **Rotation automatique sur timer** (key expiry fixe) : unnecessary
  pre-v1.0 (0 node externe), la compromise est le trigger pas le
  temps. Post-v1.0, envisageable via LT commitment.

**Implications** :
- Nouveau `crates/nexus-core-rs/src/key_rotation.rs` (struct +
  sign/verify + revocation cache)
- Nouveau `DOMAIN_KEY_ROTATION_V1` dans `canonical.rs`
- Update `curator.rs` : `verify_signature` check revocation
- Wire gossip subscribe dans `shell-daemon-core` (pattern
  `pow_policy_loader.rs` S20 Phase C)
- PyO3 binding `nexus_core.verify_key_rotation` pour coord-side

### D2 — C3 handoffs : StageGuardrailMap multi-stage pipeline

**Retenu** : type alias `StageGuardrailMap = Dict[str, GuardrailChain]`
mappant les 5 lifecycle stages (S24 Phase C hooks) a des
`GuardrailChain` optionnelles. Le `Dispatcher` accepte un
`stage_guards: StageGuardrailMap` en plus de l'actuel `input_chain`.
A chaque fire de `HookRunner`, le hook runner verifie si un chain
est defini pour ce stage et l'execute. Chains delivrees S25 :
- `input` (pre-dispatch) : PiiInputGuardrail + CanaryInputGuardrail
  (existant, renommage `input_chain` → stage_guards["on_task_dispatched"])
- `output` (post-result) : OutputSafetyGuardrail (existant dans
  validator.py ad-hoc, migre vers chain)
- Stages `on_claim_broadcast`, `on_validator_post_task`,
  `on_quarantine_enqueue` : chain vide (None) — extensible S26+.

Pattern inspire de : middleware pipeline ASP.NET Core (request +
response pipelines separes), Kong API Gateway (request/response
transform chains), openai-agents-python (input_guardrail /
output_guardrail distinction native v0.14.3).

**Rejete** :
- **Single global chain** (meme guardrails input et output) : PII
  redaction ne s'applique pas a l'output (le model genere, pas
  l'utilisateur). Output safety ≠ input safety. Forcer les 2 dans
  1 chain = no-ops inutiles ou branching dans chaque guardrail.
- **Per-guardrail stage annotation** (`@stage("input")` decorator) :
  couplage guardrail→stage, un guardrail ne devrait pas savoir a
  quel stage il tourne. La separation chain-level est plus propre.
- **AOP/aspect decorators** : deja rejete S24 D2 pour les hooks.
  Meme raison : implicite, hard to debug, execution order opaque.

**Implications** :
- Update `guardrails.py` : `StageGuardrailMap` type alias
- Update `dispatcher.py` : `stage_guards` parameter, integration
  avec `HookRunner`
- Update `validator.py` : OutputSafetyGuardrail migre vers
  output chain au lieu d'etre inline
- Tests contract : 2 chains (input + output) × scenarios

### D3 — D5 capabilities : nexus-admin gate-off-by-default

**Retenu** : implementer le design `docs/security/CAPABILITY_TOGGLES.md`
(S22 hors-sprint, status design-only) en 4 composants :
1. `nexus-admin` Typer CLI Python (`packages/nexus-coordinator/src/
   nexus_coordinator/cli/commands/capability.py`) — `list`, `enable`,
   `disable`, `info`, `audit-trail`.
2. `CapabilitiesStore` singleton (`capability_store.py`) — load
   `~/.sbfb/capabilities.toml` + verify `integrity_hash` SHA-256 +
   fallback all-OFF on tamper detect.
3. `@require_capability(name)` FastAPI decorator — 403 si disabled.
4. Semgrep rule `.semgrep/capability_gate.yml` — PR-block si endpoint
   `/tool/`, `/rag/`, `/mcp/` sans decorator.

Admin privilege check : `os.geteuid() == 0` (Unix), `IsUserAnAdmin()`
+ Mandatory Integrity Level High (Windows). Pattern exactement
`microsoft/sudo` (capability OFF par defaut, admin-only activation).

**Event logging** : `capability_changed` event via `structlog` (pas
ETW/journald — A3 OS audit channel deferred S26). Quand A3 land,
le log sera reroute vers les writers platform-native sans changer
l'interface.

**Rejete** :
- **Config.toml inline flag** (`~/.sbfb/config.toml` boolean) :
  editable par malware user-mode sans privilege escalation. Pas
  de gate admin. Detecte par CAPABILITY_TOGGLES.md §1 threat
  analysis.
- **Environment variable** (`SBFB_ENABLE_MCP=1`) : ephemere, pas
  d'audit trail, pas d'anti-tamper. Un process enfant herite les
  vars = pas de confinement.
- **Feature flags compile-time** (`--features mcp_server`) :
  oblige rebuild binaire pour changer. Pas de runtime toggle.
  Inutilisable pour operator qui veut activer/desactiver sans
  redeployer.

**Implications** :
- Nouveau `packages/nexus-coordinator/src/nexus_coordinator/
  capability_store.py` + `cli/commands/capability.py`
- Nouveau `.semgrep/capability_gate.yml`
- Update `docs/security/CAPABILITY_TOGGLES.md` status
  design-only → implemented S25

### D4 — P2 cleanup batch : DNS concurrent + quarantine alerting

**Retenu** : absorber 3 P2 issus de S24 audit :
- **P2-E-1** : `dns_fallback.rs` `build_resolver` utilise
  `endpoints[0].tls_name` pour tous les IPs. Refactor : chaque
  endpoint porte son propre `tls_name`, le resolver les itere
  individuellement.
- **P2-E-2** : `resolve_node` essaie DoH puis DoT sequentiellement.
  Refactor : `tokio::select!` lance les 2 en concurrence, premiere
  reponse gagne. Worst-case passe de 2×timeout a 1×timeout.
- **P2-D-2** : quarantine enqueue = silent. Ajouter emission
  d'un structured log + hook `on_quarantine_enqueue` event qui
  inclut le worker_id + reason. Le curator peut observer via
  son endpoint `/api/quarantine/` (deja existant S21 Phase D).

**Rejete** :
- **P2-D-1 redundancy persistence** : in-memory → SQLite est un
  refactor significatif (pattern S21 quarantine, mais applique a
  une structure differente). Defer S26 — in-memory suffisant
  pre-v1.0 (0 node externe, pas de state a survivre un restart).

**Implications** :
- Update `crates/nexus-core-rs/src/dns_fallback.rs` (P2-E-1 + P2-E-2)
- Update `packages/nexus-coordinator/src/nexus_coordinator/
  quarantine_queue.py` (P2-D-2 alerting via hooks)
- Pas de nouveau fichier

### D5 — Scope management : ce que Sprint 25 NE fait PAS

**Retenu** : les items suivants sont differes pour garder le sprint
dans la norme ~2500 LOC :
1. **Tor transport phase 1** → S26 (arti-client 0.37.0 pre-1.0,
   risque instabilite API)
2. **B2 MCP server expose** → S26 (prereq D5 ce sprint, S26
   combine D5 livre + mitigations vuln MCP avril 2026)
3. **A3 OS audit channel** → S26 (ETW/journald/oslog, structlog
   fallback suffisant S25)
4. **C2 @task_handler SDK** → S26+ (dep B3 Pydantic auto-derivation)
5. **C5 streaming bridge** → S26+ (dep D5 + Playwright matrix)
6. **RAG sanitization** → S26+ (dep D5 `rag_retrieval` capability)
7. **Per-app rate budget** → S26+ (coordinator extension, pas de
   dep urgente)
8. **Pluggable transports lyrebird** → S26 (couple avec Tor phase 1)
9. **Domain fronting implementation** → S26+ (legal review prereq)
10. **P2-D-1 redundancy persistence** → S26 (refactor significatif)
11. **P2-E-1-iroh neighborhood** → S26 (enrichment non-bloquant)

---

## 4.5 Design Review Board findings (G1)

**Report** : `.planning/active/sprint25_design_review.md` (2026-04-22).
**Verdict** : 0 ❌ + 0 ⚠️ + 5 ✅. Proceder Phase A.

Scoring : D1 ✅, D2 ✅, D3 ✅, D4 ✅, D5 ✅.

### Acknowledged review findings

Aucun angle mort identifie (0 ⚠️). Recommandations non-bloquantes
integrees dans le plan :
- **D1** : monitoring ed25519-dalek 2.2+ breakage → Phase B devra
  verifier latest version avant code.
- **D2** : tests regression input_chain migration → Phase C plan §C.2
  inclut backward compat.
- **D3** : Windows MIL testing → Phase D tests incluent mock
  admin_check.
- **D4** : timeout logging concurrent DNS → Phase A tests incluent
  failure logging.

---

## 5. Phase outline

### Phase A — P2 cleanup batch DNS concurrent + quarantine alerting

- **Scope** : P2-E-1 per-endpoint TLS name dns_fallback.rs + P2-E-2
  concurrent DoH/DoT tokio::select! + P2-D-2 quarantine alerting
  via hooks on_quarantine_enqueue + HARDENING_ROADMAP last_validated
  update 2026-04-22
- **Critere** : `cargo nextest run dns_fallback` vert (P2-E-1 + P2-E-2),
  `uv run pytest tests/test_quarantine_queue.py` vert (P2-D-2)
- **Commit** : `feat(sprint25): Phase A — P2 batch DNS concurrent
  fallback + quarantine curator alerting`

### Phase B — Key rotation ceremony + revocation announcement

- **Scope** : `key_rotation.rs` (KeyRotationAnnouncement struct +
  sign/verify + RevocationCache), `DOMAIN_KEY_ROTATION_V1` canonical,
  gossip subscribe wire shell-daemon-core, PyO3 binding
  `verify_key_rotation`, update `curator.rs` verify_signature
  check revocation, tests contract
- **Critere** : 20+ tests Rust (sign/verify rotation, revocation
  cache, expired transition, curator verify with revocation, PyO3
  round-trip), `cargo nextest run -p nexus-core-rs` vert
- **Commit** : `feat(sprint25): Phase B — key rotation ceremony
  Ed25519 self-signed + gossip revocation list`

### Phase C — C3 handoffs StageGuardrailMap

- **Scope** : `StageGuardrailMap` type dans guardrails.py, update
  dispatcher.py stage_guards integration, migration validator.py
  OutputSafetyGuardrail vers output chain, HookRunner integration
  chain execution, tests contract 2 chains × scenarios
- **Critere** : 15+ tests coord (input chain preserved, output chain
  new, stage routing, chain-absent passthrough, error resilience),
  `uv run pytest packages/nexus-coordinator/tests/` vert
- **Commit** : `feat(sprint25): Phase C — C3 handoffs StageGuardrailMap
  multi-stage guardrail pipeline`

### Phase D — D5 capabilities gate-off-by-default

- **Scope** : `capability_store.py` (CapabilitiesStore singleton +
  TOML parser + integrity_hash verify + fallback all-OFF),
  `cli/commands/capability.py` (Typer CLI list/enable/disable/info/
  audit-trail), `@require_capability` decorator, admin privilege
  check cross-OS, `.semgrep/capability_gate.yml`, update
  CAPABILITY_TOGGLES.md status
- **Critere** : 25+ tests (store load/verify/tamper, CLI
  enable/disable round-trip, decorator 403/200, admin check mock,
  Semgrep rule match/no-match), `uv run pytest` + `semgrep --test`
  vert
- **Commit** : `feat(sprint25): Phase D — D5 capability toggles
  nexus-admin + capabilities.toml + @require_capability decorator`

### Phase E — wrap-up + verification + audit plan S26

- **Scope** :
  - verification.md (25+ rows fail-fast)
  - audit_plan S26
  - SPRINT_LOG.md + CLAUDE.md updates
  - memory update tip + compteurs
- **Critere** : 25+ rows fail-fast verts, docs coherents
- **Commit** : `chore(sprint25): Phase E — wrap-up + verification
  + audit plan S26 + migration planning archive/v1.2/`

---

## 6. Items carry/dette — reclassification S24 → S25

| Item | Source | Phase S25 | Classification |
|---|---|---|---|
| Key rotation ceremony | S24 D5 scope-cut | Phase B | [x] carry confirme S25 |
| C3 handoffs semantic | S24 D5 scope-cut | Phase C | [x] carry confirme S25 |
| P2-E-1 TLS per-endpoint | audit_findings §P2 | Phase A | [x] resolve S25 |
| P2-E-2 DoH concurrent | audit_findings §P2 | Phase A | [x] resolve S25 |
| P2-D-2 quarantine alerting | S23 audit → S24 defer | Phase A | [x] resolve S25 |
| P2-D-1 redundancy persistence | S23 audit → S24 defer | — | [deferred] → S26 (refactor significatif) |
| P2-E-1-iroh neighborhood | S23 audit → S24 defer | — | [deferred] → S26 |
| T-NN+2 iframe Rust-wasm | S22 carry | — | [deferred] PATTERNS §P34 |
| LT-2 Radicle | ROADMAP_COMMITMENTS | — | hors cap (trigger tag v1.0) |
| LT-3/LT-4 | ROADMAP_COMMITMENTS | — | hors-sprint (post-v1.0) |

**Cap G7 bilan** : 2/2 slots carry consommes et resolus ce sprint
(key rotation Phase B + C3 handoffs Phase C). 0 nouveau carry
introduit — objectif sprint net-zero carry.

---

## 7. Scope cuts — ce que Sprint 25 NE fait PAS

1. **Tor transport phase 1** → S26 (arti-client 0.37.0 pre-1.0)
2. **B2 MCP server expose** → S26 (prereq D5 livre ce sprint)
3. **A3 OS audit channel** → S26 (structlog fallback suffisant)
4. **C2 @task_handler SDK** → S26+
5. **C5 streaming bridge** → S26+
6. **RAG sanitization** → S26+
7. **Per-app rate budget** → S26+
8. **Pluggable transports** → S26
9. **Domain fronting impl** → S26+
10. **P2-D-1 redundancy persistence** → S26
11. **P2-E-1-iroh neighborhood** → S26

---

## 8. Tracabilite scope — mapping carry S24 → S25

| Item carry | Source | Phase S25 | Status |
|---|---|---|---|
| Key rotation ceremony | S24 D5 + HARDENING §3 S24 | Phase B | [x] confirme |
| C3 handoffs semantic | S24 D5 + HARDENING §3 S24 | Phase C | [x] confirme |
| D5 capabilities implem | HARDENING §3 S25 amendement | Phase D | [x] confirme |
| P2-E-1 TLS per-endpoint | audit_findings S24 | Phase A | [x] confirme |
| P2-E-2 DoH concurrent | audit_findings S24 | Phase A | [x] confirme |
| P2-D-2 quarantine alerting | S23 audit → S24 defer | Phase A | [x] confirme |
| Tor transport | HARDENING §3 S25 | — | [deferred] → S26 |
| B2 MCP server | HARDENING §3 S25 | — | [deferred] → S26 |
| A3 OS audit | HARDENING §3 S25 | — | [deferred] → S26 |
| C2/C5/RAG/rate budget | HARDENING §3 S25 | — | [deferred] → S26-S27 |

**Cap G7 bilan** : 0 nouveau carry (objectif net-zero). 2/2 entrants
resolus Phases B+C.

---

## 9. Risk register (R1..R5)

| ID | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Key rotation gossip msg incompatible existing subscribe | Low | Medium | Nouveau topic dedie, pas de modification topics existants |
| R2 | C3 handoffs casse le path input_chain existant | Medium | High | Migration backward-compatible : input_chain renomme, tests regression |
| R3 | capabilities.toml integrity_hash collisions SHA-256 | Negligible | Low | SHA-256 = 256-bit, collision pratiquement impossible pre-quantum |
| R4 | Admin privilege check bypass Windows Medium IL | Low | Medium | Double check IsUserAnAdmin + MIL High (defense-in-depth, cf. CAPABILITY_TOGGLES.md §4.1) |
| R5 | tokio::select! DNS concurrent race condition | Low | Low | select! est cancel-safe pour les futures DNS resolver (read-only) |

---

## 10. Audit gate pattern — rappel

- Phase E produira `sprint25_verification.md` + `sprint25_audit_plan.md`
- Sprint 26 Phase 0 jouera l'audit gate en session fraiche
- Convention permanente depuis Sprint 7

---

## 11. Checkpoint de validation

- [x] Audit gate S24 PASS (0 P0/P1)
- [x] G2 trigger check : 1 ACTIVE (MCP vuln avril 2026, renforce
      D5 urgence — D5 livre ce sprint, B2 MCP S26)
- [x] G6 memory carry-over : items S24 verification §5 deja captures
      dans nexus_grid_pivot.md tip `358f166`
- [x] G7 cap carry-overs : 2/2 resolus (key rotation Phase B +
      C3 handoffs Phase C). 0 nouveau carry.
- [x] D1..D5 rediges
- [x] sprint25_carry_summary.md genere (correction gap process S24)
- [x] G1 Design Review Board scoring report (0 ❌ + 0 ⚠️ + 5 ✅)
- [x] Acknowledged review findings (0 ⚠️, recommandations non-bloquantes)
