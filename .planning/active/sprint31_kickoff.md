# Sprint 31 — Kickoff (task_runner réel + output filter E2E + Tor transport phase 1)

**Ecrit** : 2026-04-26 (session fraiche post-audit gate S30 `3e1cac0`).
**Type** : **sprint impair feature** (2 carries MANDATORY resolus +
Tor transport arti-client 2.0 + P2 batch S30 + G2 HARDENING update).
**Tip master d'entree** : `3e1cac0` (chore(planning): sprint 30
audit findings — verdict PASS, 0 P0/P1, 1 P3, 6 P2 carry).
**Phase 0 audit Sprint 30** : **DEJA JOUE** — findings dans
`.planning/active/sprint30_audit_findings.md` (verdict **PASS**,
0 P0/P1, 1 P3 nouveau, 6 P2 carry confirmations).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** (2026-04-26) : HARDENING_ROADMAP last_validated
  `2026-04-26` (S30 Phase D, meme jour). **3 triggers ACTIFS**
  inchanges depuis last_validated :

  1. **iroh 0.98.0** (2026-04-17, n0-computer/iroh) — trigger ACTIF.
     Breaking changes : types `#[non_exhaustive]`, `SecretKey::
     generate()` sans Rng, relay-v2 protocol avec Health frame,
     `Endpoint::online()` attend relay. iroh-gossip 0.98 suit.
     iroh-docs versioning separe (v0.29.0). Compatibilite iroh-blobs
     0.99 **non verifiee** (risque). Day 0 #3 interdit upgrade sauf
     sprint dedie. **SCOPE-CUT S32** (cf. D5).
     Source : GitHub releases iroh v0.98.0, crates.io.

  2. **arti-client 2.0.0** (2026-02-07, Tor Project) — trigger ACTIF.
     API : `TorClient::create_bootstrapped(config)` + `.connect(addr)`
     async tokio. Retourne `DataStream` (AsyncRead + AsyncWrite).
     ~15-20 deps transitives. API marquee "experimental" mais stable
     pour MVP. 0 CVE RustSec. LTS annonce branche 2.x. Config TOML
     via `TorClientConfig`. Pas de SOCKS daemon requis (API directe).
     **INTEGRE Phase C** comme feature principale.
     Source : blog.torproject.org arti_2_0_0_released, docs.rs
     arti-client, context7 arti (1597 snippets).

  3. **openai-agents-python 0.14.6** (2026-04-25) — trigger ACTIF.
     Informationnel uniquement, pas de dep directe SBFB.

  Triggers INACTIFS : frost-ed25519 (2.1.0), wasmtime, Tor PoW
  hspow, NIST PQC FIPS, NVIDIA H100 CCM, RFC 9591, MCP spec,
  microsoft/sudo.

- **G9 Codebase Exploration (2026-04-26)** :

  - **task_runner stub** : `task_runner.rs:9-17` retourne
    `TaskExecuteResult` vide (zero output/tokens/duration). Callsite
    unique `main.rs:105`. LlmBackend trait + OllamaBackend existent
    dans nexus-worker-core (~170 LOC production). Gap : copier
    factory pattern, remplacer stub par `generate()` call, mapper
    GenerateResponse → TaskExecuteResult. **~170 LOC**.

  - **output_filter.py** : `OutputFilter.filter(system_prompt,
    user_prompt, model_output) → FilterVerdict` existe (invisible
    text strip + prompt echo EED 0.85 + hot-reload policy). Wire
    sur **input** (PII redaction avant worker) mais **pas sur
    output** (post-verify result path). Gap : injecter dans le
    dispatcher post-verify, marquer results invalides rejected, 0
    kudos credit. Pattern PII guardrail. **~160 LOC**.

  - **iroh relay Tor** : iroh 0.97 `Endpoint::builder()` ne supporte
    pas de proxy config pour les relay connections. Tor transport
    phase 1 = **coordinator-side outbound HTTP** via arti (pas iroh
    relay). Les relay VPS servent deja de shield IP pour le layer P2P.
    Full iroh-over-Tor = phase 2 (S32+ fork evaluation ou upstream
    proxy API). **~500 LOC** phase 1.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 30 CLOSED. 4 phases A-D livrees :
- Phase A : P2 batch S29 (7 items, consent pure fn refactor, executor
  trace comment, task_runner defense-in-depth, THREAT_MODEL §9.5 gap
  note, otel doc fixes)
- Phase B : dette pair blob-serve COOP/COEP headers + CI cross-
  platform GitHub Actions 3 OS
- Phase C : warrant canary Niveau 1 FROST DKG code wiring (dkg.rs +
  ceremony.rs + 5 CLI + 4 HTTP endpoints + ops runbook)
- Phase D : G2 HARDENING_ROADMAP refresh + SPLIT_INFERENCE_DESIGN.md

Audit gate S30 : **PASS** (0 P0/P1, 1 P3 nouveau, 6 P2 carry
confirmations). Roadmap v1.0 Alexandria S31-S35 commite (`c50976a`).

### §1.2 Ancrage HARDENING_ROADMAP

HARDENING_ROADMAP §3 Sprint 31 prescrit :

| Item prescrit | Statut G2/G9 S31 | Decision |
|---|---|---|
| Tor transport phase 1 (arti >= 1.0 debloque) | arti 2.0.0 LTS stable, API confirmee | **INTEGRE Phase C** (scope : coordinator outbound HTTP, pas iroh relay) |
| §9.5 output filter wire E2E (carry 2/3) | Code existe, gap = placement post-verify | **INTEGRE Phase B** |
| task_runner implementation reelle (carry 2/3) | Stub, LlmBackend factory existe | **INTEGRE Phase A** |
| P2 carries S30 Phase B/C reviews | 4 P2 a 1/3, 1 P3 a 1/3 | **INTEGRE Phase D** (batch) |

**iroh 0.98 upgrade** (roadmap Alexandria S31 Phase C) : **SCOPE-CUT
S32**. Justification :
- S31 a deja 2 carries MANDATORY + Tor feature = 3 hard deliverables
- iroh-blobs 0.99 compatibilite non verifiee (risque cascade)
- Tor phase 1 fonctionne avec iroh 0.97 (coordinator HTTP, pas relay)
- S32 est un sprint pair → phase dette obligatoire, ideal pour
  l'upgrade dedie
- Day 0 #3 : "upgrade **volontaire**" = sprint dedie, pas opportuniste

### §1.3 Compteurs tests entree (tip `3e1cac0`)

| Suite | Count | Delta vs S30 entree |
|---|---|---|
| Rust (cargo nextest) | 864 | +8 |
| SDK (pytest) | 195 | 0 |
| Coordinator (pytest) | 394 passed + 36 failed (PyO3 wheel stale) + 6 skipped | +1 |
| Gov (pytest) | 46 | 0 |
| Vitest | 269 | 0 |
| Playwright | ~43 (41+2f env) | 0 |
| size-limit | 7/7 | 0 |
| **Total** | **~1854** | **+9** |

Les 36 coord failures = PyO3 wheel stale (meme root cause depuis
S16). Les 2 PW failures = env (coordinator not running). 1 SDK flaky
(test_concurrent_store_same_sha256_dedup_safe, passe au re-run).

### §1.4 Pre-launch protocol policy (rappel)

Pas de deploiement live. `*_FORMAT_VERSION` = 1 partout. Pas de
bump version, pas de tolerant decoder multi-version. Aucun nouveau
wire format introduit S31 (Tor transport = config + runtime, pas
wire). Cf. `CLAUDE.md §Pre-launch protocol policy`.

---

## §2 Goal en une phrase

Sprint 31 rend le worker fonctionnel (task_runner reel via Ollama),
wire le output filter end-to-end (§9.5 THREAT_MODEL), et ajoute le
premier transport Tor anonymisant (arti-client 2.0 coordinator-side).
**Critere SMART : 30+ rows fail-fast vertes au verification.md,
mesure binaire au Phase E wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 30

**DEJA JOUE** — commit `3e1cac0`.

Verdict : **PASS** (0 P0/P1, 1 P3 nouveau, 6 P2 carry confirmations).

Findings integres dans ce kickoff :
- P3-AUDIT-1 : WebAppFrame.tsx orphelin `allow-same-origin` →
  nettoyage Phase D batch
- 6 P2 carry confirmations → §6 ci-dessous

ROADMAP_COMMITMENTS check (G7 Regle 3) :
- LT-1 a LT-5 : conditions latentes, aucun declenchement.
- **LT-6** : trigger "iroh > 0.97" met (iroh 0.98.0 2026-04-17).
  MAIS Day 0 #3 pin bloque. iroh 0.98 scope-cut S32 (cf. §1.2).
  LT-6 sera reactivee comme carry actif au kickoff S32 quand le
  pin sera leve. Reste dans ROADMAP_COMMITMENTS avec note
  "condition met, upgrade scheduled S32".

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — task_runner reel : wire LlmBackend dans nexus-executor

**Retenu** : Copier le pattern `LlmBackend` trait + `OllamaBackend`
impl depuis `nexus-worker-core/src/llm/` vers `nexus-executor/`.
Wire `execute_task()` dans `task_runner.rs` : build backend on boot
via `--ollama-endpoint` CLI arg, remplacer le stub par un appel
`backend.generate(params)`, mapper `GenerateResponse` →
`TaskExecuteResult` (text, prompt_tokens, completion_tokens,
duration). Error path : `LlmBackendError` → JSON-RPC error response
au broker.

**Rejete** :
- *Garder le stub un sprint de plus* — carry 2/3, MANDATORY S32 si
  non resolu. Le worker ne fait litteralement rien sans ca. Pas de
  raison valide de differer.
- *Wire llama.cpp direct (feature llm_llama_cpp)* — l'executor est
  un child process isole. Ollama via HTTP est plus simple, plus
  testable, et ne necessite pas de compiler llama.cpp dans
  l'executor. llama.cpp = worker-side only.
- *Wire les deux backends (Ollama + llama.cpp)* — scope creep. Phase
  1 = Ollama seul. llama.cpp executor support = carry S32+ si
  demande.
- *IPC pass-through vers worker-core* — le worker-core est dans un
  process different (worker binary). L'executor doit avoir son propre
  backend. L'IPC broker route les tasks, pas les LLM calls.

**Implications code** : `crates/nexus-executor/src/task_runner.rs`
(rewrite ~80 LOC), `crates/nexus-executor/src/main.rs` (CLI arg +
backend init ~40 LOC), `crates/nexus-executor/Cargo.toml` (deps
ollama-rs + schemars + tokio). Tests : stub-mode e2e + generate mock.

### D2 — §9.5 output filter wire E2E : injection post-verify coordinator

**Retenu** : Injecter `OutputFilter.filter(system_prompt, user_prompt,
result.result_text)` dans le result dispatch path du coordinator,
post-verification signature 3-layer. Results invalides marques
`rejected` (meme semantique que verify fail), 0 kudos credit.
Pattern identique a `PiiInputGuardrail` : guardrail ABC adapter
`OutputSafetyGuardrail.check()` dans le pipeline declaratif
`GuardrailChain`. Logging tripwire evidence (invisible_text,
prompt_echo, risk_score) dans audit log.

**Rejete** :
- *Filter worker-side (avant signature)* — le worker est
  potentiellement malveillant (C-ResultSpoof threat model §9.5).
  Filtrer worker-side = faire confiance au worker pour se filtrer
  lui-meme. Le coordinator est la trust boundary.
- *Filter client-side (iframe SDK)* — meme raison : le client
  affiche le resultat, il ne decide pas s'il est safe. Defense en
  profondeur client-side = carry S34 (Phase B polish).
- *Garder le gap un sprint de plus* — carry 2/3, MANDATORY S32.
  THREAT_MODEL §9.5 documente "output filter OFF" comme residual.
  Le code existe, il suffit de le brancher.
- *Re-design OutputFilter* — le design actuel (invisible text +
  prompt echo EED) est valide et teste. Pas de redesign pour wirer.

**Implications code** :
`packages/nexus-coordinator/src/nexus_coordinator/api/verify.py` ou
nouveau `result_guardrails.py` (~50 LOC injection), context threading
(~20 LOC), logging (~30 LOC), tests (~60 LOC fixtures invisible text
+ echo + clean + edge).

### D3 — Tor transport phase 1 : arti-client 2.0 coordinator outbound

**Retenu** : Integrer `arti-client 2.0.0` pour anonymiser les
connexions HTTP sortantes du coordinator (task dispatch, gossip
publish, HTTP fetch). Scope phase 1 :
- `TorTransport` wrapper Rust dans `nexus-core-rs` (create
  `TorClient::create_bootstrapped()`, expose `connect(addr)`)
- Configuration opt-in `[tor]` section dans config TOML (disabled
  par defaut, `enabled = false`)
- PyO3 binding minimal : `tor_connect(host, port) → stream` pour
  usage coordinator-side Python
- Wire dans le coordinator pour les outbound HTTP requests
  (via `httpx` transport adapter ou `aiohttp` connector)
- Pas de modification des connexions iroh relay (phase 2 S32+)
- Test : connection through Tor, latency baseline, fallback si Tor
  indisponible

**Rejete** :
- *iroh relay over Tor (full stack)* — iroh 0.97 Endpoint::builder()
  n'expose pas de proxy config. Necessite fork iroh ou upstream
  contribution. Scope S32+ apres evaluation. Les relay VPS servent
  deja de shield IP pour le P2P layer.
- *SOCKS proxy mode (arti proxy -p 9150)* — overhead daemon
  supplementaire, pas necessaire avec l'API directe
  TorClient::connect(). Plus simple, moins de surface d'attaque.
- *Skip Tor S31* — HARDENING_ROADMAP §3 S31 le prescrit. arti 2.0
  LTS stable depuis 2 mois. Carry depuis S25 (4 sprints). Le
  coordinator expose l'IP de l'operateur sur chaque HTTP sortant.
- *I2P/Nym hybride* — Nym SDK paused (S30 scope-cut). I2P pas dans
  le threat model SBFB. arti-client = Tor standard = meilleur
  rapport couverture/risque pour ONG target.

**Implications code** : `crates/nexus-core-rs/src/tor_transport.rs`
(NEW ~200 LOC), `crates/nexus-core-py/src/lib.rs` (binding ~30 LOC),
`packages/nexus-coordinator/src/nexus_coordinator/tor_client.py`
(wrapper ~80 LOC), `configs/tor.toml.sample` (NEW),
`docs/security/HARDENING_ROADMAP.md` (S31 entry update).
Deps : `arti-client = "2.0"`, `tor-rtcompat = "2.0"` (tokio runtime).

### D4 — P2 batch S30 carries + G2 HARDENING update

**Retenu** : Batch des items P2/P3 a 1/3 resolvables + G2 update :
- P3-AUDIT-1 : supprimer `web/src/components/app/WebAppFrame.tsx` +
  `WebAppFrame.test.tsx` (composant orphelin S11, `allow-same-origin`
  non-conforme, 0 import production)
- P2-REVIEW-D-1-S30 : VALIDATED_BLUEPRINT.md refresh Couche 6
  (Kirchenbauer → SynthID, spaCy → GLiNER) — doc fix ~10 LOC
- P3-REVIEW-D-1-S30 : confidence_score field dans
  SPLIT_INFERENCE_DESIGN.md — doc fix ~5 LOC
- HARDENING_ROADMAP.md : `last_validated: S31`, S31 entry update
  (Tor delivered, iroh 0.98 deferred S32, carries resolved)
- P2-REVIEW-C-1-S30 : HTTP integration tests FROST endpoints — ~4
  tests exercant POST /api/canary/frost/* (endpoints declares mais
  non testes en integration HTTP)

**Rejete** :
- *P2-REVIEW-B-1-S30 Playwright COEP iframe test* — necessite env
  Playwright avec coordinator running. Les 2 PW env failures actuelles
  montrent que l'env n'est pas stable. Differe S34 Phase B polish.
- *Skip batch* — les items doc (VALIDATED_BLUEPRINT, confidence_score)
  sont trivaux. WebAppFrame cleanup ferme un gap securite (meme
  orphelin). HTTP FROST tests ferment un gap couverture. Ne pas les
  faire = les carry a 2/3 pour rien.

**Implications code** : `web/src/components/app/WebAppFrame.tsx` +
`WebAppFrame.test.tsx` (DELETE), `docs/security/VALIDATED_BLUEPRINT.md`
(edit ~10 LOC), `docs/security/SPLIT_INFERENCE_DESIGN.md` (edit ~5
LOC), `docs/security/HARDENING_ROADMAP.md` (update S31 entry),
`crates/nexus-shell-daemon/tests/` ou inline tests HTTP FROST (~80
LOC tests).

### D5 — iroh 0.98 upgrade : SCOPE-CUT S32

**Retenu** : Reporter l'upgrade iroh 0.97 → 0.98 au Sprint 32.

**Justification** :
1. S31 a 2 carries MANDATORY (task_runner 2/3 + output filter 2/3)
   + 1 feature principale (Tor transport) = 3 hard deliverables
2. Compatibilite iroh-blobs 0.99 avec iroh 0.98 non verifiee —
   risque de cascade si les versions sont incompatibles
3. Breaking changes iroh 0.98 (types `#[non_exhaustive]`,
   `SecretKey::generate()`, relay-v2) affectent le workspace entier
   (nexus-core-rs, shell-daemon-core, worker-core)
4. Tor phase 1 scope = coordinator outbound HTTP, ne touche pas iroh
   relay → pas de prerequis iroh 0.98
5. S32 est un sprint pair → phase dette obligatoire (Regle 1 §6.2.1),
   ideal pour absorber l'upgrade dedie
6. LT-6 sera reactivee au kickoff S32 comme carry actif quand le pin
   sera leve

**Roadmap Alexandria deviation** : le roadmap prescrivait iroh 0.98
en S31 Phase C. Le kickoff a autorite sur le roadmap (§8 du roadmap :
"Ce document n'est PAS un kickoff sprint"). La deviation est
documentee, pas silencieuse.

**Implications** : Day 0 #3 (iroh 0.97 pinne) reste en vigueur
pendant S31. Sera leve au kickoff S32 avec son propre D-decision.

### Acknowledged review findings (G1)

Scoring : D1 ✅, D2 ✅, D3 ⚠️, D4 ���, D5 ✅.
Rigor signal G4 satisfait (1 ⚠️ sur 5).

**D3 ⚠️** : "Tor transport coordinator-only scope significantly
narrower than HARDENING_ROADMAP §3 S31 prescription which specifies
'wire iroh relay HTTPS fallback over Tor SOCKS5'". Decision : adjust —
Phase C scope documente explicitement que iroh relay Tor = phase 2
(S32+) car iroh 0.97 n'expose pas de proxy config. HARDENING_ROADMAP
§3 S31 entry sera mise a jour Phase D pour refleter le scope reel
livre (coordinator outbound HTTP, pas iroh relay).

---

## §5 Plan Phase outline A..E

### Phase A — task_runner reel (P2-REVIEW-C-1 resolu)

Carry MANDATORY resolu : wire LlmBackend dans l'executor.
- Rewrite `task_runner.rs` : build OllamaBackend, generate(), map
  response
- CLI `--ollama-endpoint` dans main.rs
- Tests : stub-mode fallback + generate mock + error path
- Commit cible : `feat(sprint31): Sprint 31 Phase A — task_runner
  reel executor wire LlmBackend Ollama`

### Phase B — §9.5 output filter E2E (P2-REVIEW-B-2 resolu)

Carry MANDATORY resolu : wire OutputFilter dans le result path.
- Injection post-verify dans dispatcher
- OutputSafetyGuardrail adapter dans GuardrailChain
- Context threading (system_prompt, user_prompt)
- Nettoyage WebAppFrame.tsx orphelin (P3-AUDIT-1, meme commit
  car touche web/)
- Tests : invisible text tripwire + echo + clean + edge
- Commit cible : `feat(sprint31): Sprint 31 Phase B — output filter
  E2E wire + WebAppFrame orphelin cleanup`

### Phase C — Tor transport phase 1 arti-client 2.0

Feature principale S31 :
- `tor_transport.rs` dans nexus-core-rs (TorTransport wrapper)
- PyO3 binding `tor_connect(host, port)`
- `tor_client.py` coordinator wrapper
- Config `[tor]` opt-in (disabled par defaut)
- Wire coordinator outbound HTTP via arti
- Tests : Tor bootstrap + connect + fallback + latency baseline
- Commit cible : `feat(sprint31): Sprint 31 Phase C — Tor transport
  phase 1 arti-client 2.0 coordinator outbound`

### Phase D — P2 batch S30 + G2 HARDENING update

Docs + tests + cleanup batch :
- Supprimer WebAppFrame.tsx + test (si pas fait Phase B)
- VALIDATED_BLUEPRINT.md refresh Couche 6
- SPLIT_INFERENCE_DESIGN.md confidence_score
- HARDENING_ROADMAP.md S31 entry update
- HTTP integration tests FROST endpoints
- Commit cible : `feat(sprint31): Sprint 31 Phase D — P2 batch S30
  carries + G2 HARDENING update`

### Phase E — Wrap-up + verification + audit plan S32

Standard wrap-up :
- sprint31_verification.md (fail-fast 30+ rows)
- sprint31_carry_summary.md
- sprint32_audit_plan.md
- SPRINT_LOG.md row S31
- CLAUDE.md §Etat actuel update
- Memory update nexus_grid_pivot.md + MEMORY.md
- Migration active/ → archive/v1.2/
- Commit cible : `chore(sprint31): Phase E — wrap-up + verification
  + audit plan S32 + migration`

---

## §6 Items carry/dette (G7)

### Carry S30 — resolution prevue

| ID | Description | Reports | Resolution S31 | Phase |
|---|---|---|---|---|
| P2-REVIEW-C-1 | task_runner stub | **2/3** | Wire reel Ollama | A |
| P2-REVIEW-B-2 | §9.5 output filter not wired | **2/3** | Wire E2E post-verify | B |
| P3-AUDIT-1 | WebAppFrame.tsx orphelin | 1/3 | Supprimer | B |
| P2-REVIEW-D-1-S30 | VALIDATED_BLUEPRINT stale | 1/3 | Doc fix | D |
| P3-REVIEW-D-1-S30 | confidence_score field | 1/3 | Doc fix | D |
| P2-REVIEW-C-1-S30 | HTTP FROST tests | 1/3 | Tests | D |

### Items differes S32+

| ID | Description | Reports apres S31 | Sprint cible | Justification |
|---|---|---|---|---|
| P2-REVIEW-B-1-S30 | Playwright COEP iframe test | 2/3 | S34 | Env Playwright instable, polish phase |
| iroh 0.98 upgrade | Lever Day 0 #3 + LT-6 | - | S32 | Risque cascade, sprint pair dette |

### Phase dette S31 (§6.2.1 Regle 1)

S31 est impair — pas de phase dette obligatoire. Les carries 2/3
sont resolus en Phase A et B comme features, pas comme dette.

### Items long-terme (ROADMAP_COMMITMENTS — inchanges)

| ID | Condition | Status |
|---|---|---|
| LT-1 | v1.0 + design doc + Gini > 0.70 | Latent |
| LT-2 | tag v1.0 | Latent |
| LT-3 | v1.0 + 3+ contrib non-compute | Latent |
| LT-4 | v1.0 + N1 FROST + partnership | Latent |
| LT-5 | multi-worker deploy OR v1.0 | Latent |
| LT-6 | iroh > 0.97 OR v1.0 | **Trigger met** (iroh 0.98.0) — bloque par Day 0 #3 pin, **scheduled S32** |

---

## §7 Scope cuts

Ce que S31 ne fait PAS :

1. **iroh 0.98 upgrade** — scope-cut S32 (risque cascade, sprint pair
   dette, carries prioritaires). Day 0 #3 maintenu.
2. **iroh relay over Tor** — scope-cut S32+ (iroh 0.97 pas de proxy
   config, necessite fork evaluation ou upstream contribution)
3. **Nym mixnet phase 1** — re-defere S33+ (SDK paused crates.io)
4. **TEE H100 attestation** — scope-cut (pas hardware partenaire)
5. **DKG distribue FROST** — post-v1.0 (trusted dealer suffisant N=3)
6. **Recrutement mainteneurs** — ops post-v1.0
7. **Playwright COEP iframe test** — S34 Phase B polish (env instable)
8. **Onion service hosting** — post phase 1 Tor (phase 2)
9. **Full process isolation blob-serve** — LT rewrite architectural
10. **openai-agents-python upgrade** — pas de dep directe SBFB
11. **llama.cpp executor support** — S32+ si demande
12. **Output filter client-side (iframe defense-in-depth)** — S34
    Phase B polish (defense en profondeur, pas critical path)

---

## §8 Tracabilite scope

Table mappant les items S30 "What's NOT" sur leur traitement S31 :

| Item S30 scope-cut | Sprint + Phase S31 | Status |
|---|---|---|
| Tor transport phase 1 | S31 Phase C | **INTEGRE** |
| task_runner implementation | S31 Phase A | **INTEGRE** |
| §9.5 output filter wire E2E | S31 Phase B | **INTEGRE** |
| iroh 0.98 upgrade | S32 | SCOPE-CUT (risque) |
| Nym mixnet phase 1 | S33+ | RE-DEFERE |
| TEE H100 attestation | post-v1.0 | SCOPE-CUT |
| DKG distribue FROST | post-v1.0 | SCOPE-CUT |
| Full process isolation blob-serve | LT | SCOPE-CUT |
| Tor PoW spec update | trigger inactif | INCHANGE |
| MCP spec revision | trigger inactif | INCHANGE |
| CI full workspace cross-platform | resolu S30 | DONE |

---

## §9 Risk register

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | arti-client deps lourdes (15-20 transitives) augmentent build time | Medium | Low | Feature-gate `tor` dans Cargo.toml, compile uniquement si enabled |
| R2 | arti bootstrap lent (~10-30s premiere connexion Tor) | High | Medium | Connexion async au startup, pas bloquant. Fallback direct si Tor timeout 30s |
| R3 | OllamaBackend executor : Ollama pas running sur machine de test | Medium | Low | Stub-mode test sans Ollama reel. Integration test optionnel avec `OLLAMA_ENDPOINT` env |
| R4 | OutputFilter false positive sur contenu legitime | Low | Medium | Config threshold dans output_filter_policy.toml. Log tripwire evidence pour tuning |
| R5 | iroh 0.98 scope-cut conteste par audit gate S31 | Low | Low | Justification documentee §1.2 + D5. Roadmap deviation explicite |
| R6 | P2-REVIEW-B-1-S30 Playwright atteint 2/3 → MANDATORY S32 | Medium | Low | S32 kickoff integre resolution ou exemption blocker externe (env instable) |
| R7 | arti TorClient tokio runtime conflit avec iroh tokio | Low | High | Les deux utilisent tokio 1.x. Shared runtime. Tester trot dans Phase C |

---

## §10 Audit gate pattern — rappel

Phase 0 audit S30 **jouee** — verdict PASS, commit `3e1cac0`.
Phase E produira :
- `sprint31_verification.md` (self-report fail-fast)
- `sprint32_audit_plan.md` (plan pour S32 Phase 0)
- `sprint31_carry_summary.md`

---

## §11 Checkpoint de validation

5 questions pour arbitrage user AVANT le plan detaille :

1. **D1 task_runner** : wire Ollama uniquement, pas llama.cpp — le
   delivrable cle est `nexus-executor --ollama-endpoint
   http://localhost:11434` recoit un task et retourne une vraie
   inference. Suffisant ?
2. **D2 output filter** : injection coordinator-side post-verify.
   Le worker reste non-filtre (trust boundary = coordinator).
   Acceptable pour phase 1 ?
3. **D3 Tor** : scope reduit a coordinator outbound HTTP (pas iroh
   relay). Les relay VPS shieldent deja l'IP P2P. iroh relay Tor =
   phase 2 S32+. Acceptable ?
4. **D5 iroh 0.98** : scope-cut S32 malgre roadmap Alexandria.
   Justification : risque cascade + carries prioritaires + Tor
   fonctionne sans. OK ?
5. **D4 batch** : supprimer WebAppFrame orphelin + 4 items P2/P3
   doc/tests. Trop ou pas assez ?
