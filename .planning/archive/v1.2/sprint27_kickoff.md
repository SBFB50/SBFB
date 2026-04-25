# Sprint 27 — Kickoff (Output watermark SynthID + Couche 3 mature multi-forge + trust-web ONG bootstrap + Gate 3 hardening + P2 batch S26)

**Ecrit** : 2026-04-25 (session fraiche post-audit gate S26 `982cfd1`).
**Type** : **sprint implementation** (watermark model output + Couche 3
Sybil-resistance mature + Gate 3 prerequisites showcase apps).
**Tip master d'entree** : `22374f3` (chore(planning): migration S26
audit_findings active → archive/v1.2/).
**Phase 0 audit Sprint 26** : **DEJA JOUE** — findings dans
`.planning/archive/v1.2/sprint26_audit_findings.md` (verdict **PASS**,
0 P0/P1, 7 P2 documentes).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** (2026-04-25, verification des 12 triggers
  HARDENING_ROADMAP depuis `last_validated` 2026-04-22) :
  - `iroh release > 0.97` : **INACTIVE** (toujours 0.97.0).
  - `wasmtime LTS bump` : **INACTIVE** pour notre codebase. Note :
    6 nouveaux CVE avril 2026 (CVE-2026-34941 heap OOB,
    CVE-2026-34942 panic UTF-16, CVE-2026-34945 host data leak
    Winch 64-bit, CVE-2026-34983 use-after-free Linker clone,
    CVE-2026-35195 OOB write string transcoding, CVE-2026-34988
    pooling allocator data leak). Patches dans 43.0.1 — notre pin
    43.0.1+ depuis S18 couvre deja. R-wasmtime-cve P0 mis a jour.
  - `arti-client > 1.x stable` : **INACTIVE** (toujours pre-1.0).
  - `frost-ed25519 > 2.1` : **INACTIVE** (toujours 2.1.0).
  - `MCP spec revision Anthropic 2026+` : **INACTIVE** (spec
    2025-11-25 inchangee). Design flaw disclosure avril 2026 (The
    Register) = surface STDIO, ne touche pas notre Streamable HTTP
    local-only.
  - `openai-agents-python > 0.7.0` : **ACTIVE** (SDK passe a v2.x
    openai requis, ajout Sandbox Agents, WebSocket transport). Mais
    impacts sur S23-S25 (guardrails + hooks + SDK) deja livres. Pas
    d'impact S27 — notre fork du pattern guardrails est autonome
    (`guardrails.py` Guardrail ABC, pas d'import openai-agents).
  - `RFC 9591 erratum` : **INACTIVE**.
  - `microsoft/sudo > 24H2` : **INACTIVE**.
  - `NIST PQC FIPS 203/204` : **INACTIVE**.
  - `NVIDIA H100 CCM driver` : **INACTIVE**.
  - `Sprint S+2 trigger` : S29 entries = audit externe Cure53/ToB
    + Nym mixnet. Non-bloquant pour S27.
- **WebSearch "Kirchenbauer watermark LLM 2026"** (pre-gel D1) :
  Kirchenbauer ICML 2023 (arXiv:2301.10226) = green/red-list logit
  biasing. **Vulnerable BIRA attack** (arXiv:2509.23019, sept 2025)
  — Bidirectional Iterative Rewriting Attack sur le biais
  statistique green-list. Deja documente dans S22 kickoff D4 et
  `canary_input.py:11-12`.
- **WebSearch "SynthID text watermarking Google DeepMind 2024"**
  (pre-gel D1) : SynthID-Text (Nature 2024, Google DeepMind).
  Tournament Sampling — unifies distortionary et non-distortionary.
  Open source oct 2024 via Responsible GenAI Toolkit + Hugging Face
  (`google-deepmind/synthid-text` GitHub). Aussi vulnerable aux
  attaques meaning-preserving (paraphrasing, back-translation),
  mais plus robust que KGW. Detection par z-test statistique sur
  token distributions.
- **WebSearch "MarkLLM watermarking toolkit 2024"** (pre-gel D1) :
  MarkLLM (EMNLP 2024 Demo, THU-BPM/MarkLLM GitHub). 9 algorithmes
  (KGW + Christ familles), 12 evaluation tools. Python-only, pas de
  binding Rust. Toolkit academique, pas production-ready. PyPI
  `markllm`.
- **Codebase grep "watermark|BIRA|kirchenbauer"** (pre-gel D1) :
  `canary_input.py:2-12` documente la distinction design
  watermark-input (canari known-answer prompt probe) vs
  watermark-output (Kirchenbauer green-list). HARDENING_ROADMAP §3
  S27 ligne 616-617 prescrit encore "Kirchenbauer 2023 green-list"
  — **stale** (BIRA non integre dans la formulation §3, bien
  qu'identifie en S22).

---

## 1. Constat d'entree

### 1.1 D'ou on part

- **Tip** : `22374f3` — S26 DONE, audit PASS (0 P0/P1, 7 P2).
- **Working tree** : propre (post-migration audit_findings).
- **v1.2** : continuation security hardening. Pas de nouvelle version.

### 1.2 Ancrage HARDENING_ROADMAP

HARDENING_ROADMAP §3 S27 prescrit : "Watermark model + Couche 3
mature + Gate 3 push" (~1500 LOC total).

Items :
- Watermark injection opt-in (Kirchenbauer 2023) — ~500 LOC
- Couche 3 mature (multi-forge cross-validate + trust-web Amnesty)
  — ~700 LOC
- Gate 3 showcase-specific hardening items — ~300 LOC

**Arbitrage S27** :
1. Technique watermark changee : Kirchenbauer → SynthID-inspired
   (BIRA vulnerability, cf. D1). Implementation scoped : detection
   coordinator-side + injection llama.cpp backend opt-in.
   Ollama backend deferred (API ne supporte pas logit hooks).
2. Couche 3 : DelegationCert primitive existe (S23), design RFC
   existe (`CONTRIBUTOR_ATTESTATION_RFC.md`). Gap = multi-forge
   parser git-log offline + trust-web bootstrap seed.
3. P2 batch S26 audit : 3 items (A-1, C-1, D-1) en Phase A.
4. Gate 3 showcase hardening : JsonFileWriter rotation (P2-C-2) +
   TracingWriter rename (P2-C-3) + Gate 3 prerequisites doc update.

### 1.3 Compteurs tests entree (tip `22374f3`)

| Suite | Count | Notes |
|---|---|---|
| Rust nextest | 802 | all pass |
| Rust doctests | pass | |
| Python SDK | 193 | all pass |
| Python coord | 377 pass + 45 fail + 6 skip | 45 fail = stale PyO3 wheel |
| Python gov | 46 | all pass |
| Vitest | 264 | all pass |
| Playwright | 27 pass + 16 fail | 16 fail = env PyO3 wheel |
| Size-limit | 7/7 | |
| **Total** | **~1752** | |

### 1.4 Pre-launch protocol policy

Pas de deploiement live. `*_FORMAT_VERSION` restent a 1. Pas de
tolerant decoder multi-version. S27 n'introduit PAS de nouveau wire
format P2P gossip. Le watermark est un mecanisme local
worker-coordinator. La Couche 3 multi-forge parser est un module
offline local. Aucun `*_VERSION` bump.

---

## 2. Goal

Livrer la suite technique Gate 3 : output watermark SynthID-inspired
pour detection compute-theft worker-side, Couche 3 Sybil-resistance
mature (multi-forge cross-validation offline + trust-web bootstrap ONG),
hardening gaps Gate 3 showcase (rotation audit trail + naming). P2 batch
S26 audit en Phase A.

**Critere SMART : 25+ rows fail-fast verts au `verification.md`,
mesure binaire au Phase E wrap-up.**

---

## 3. Phase 0 — Audit gate Sprint 26

**Verdict** : PASS (0 P0/P1, 7 P2 documentes).
**Commit** : `982cfd1` — `sprint26_audit_findings.md` migre vers
`archive/v1.2/` dans `22374f3`.
**P2 carry S27** : 3 items (A-1 validate_stage_guard_map wire,
C-1 emit_capability_event logging, D-1 TaskHandlerDescriptor
description) absorbes Phase A ci-dessous.
**P2 observes non carry** : 4 items (B-1 MCP lifespan, C-2
JsonFileWriter rotation, C-3 EtwWriter rename, E-1 LOC estimates)
absorbes ou documentes dans phases B-D ci-dessous.

---

## 4. Decisions Day 0 (D1..D5 gelees)

### D1 — Output watermark : SynthID-inspired z-test detection + llama.cpp injection opt-in

**Retenu** : watermark output model-side pour detection compute-theft
(C-ComputeTheft, COMPUTE_THREATS §4). Complementaire au canary-input
S22 (watermark-INPUT prompt probe). Architecture 2 composantes :

1. **Detection coordinator-side** (Python) : z-test statistique sur
   les tokens retournes par le worker. Inspiree de SynthID Tournament
   Sampling (Nature 2024, Google DeepMind). Implementation :
   pseudo-random function (PRF) hash(token_id, context_hash, secret)
   genere un score par token. Test statistique binomial sur la
   proportion de tokens "green" (score > threshold). Le coordinator
   partage le secret avec les workers opt-in. Workers sans opt-in :
   le test est non-concluant (pas de rejection, fallback sur
   canary-input).

2. **Injection worker-side** (Rust, LlamaCppBackend only) : logit
   bias additionnel durant le sampling. Le worker ajoute un delta
   `+delta_logit` aux tokens "green" (determines par la meme PRF).
   Opt-in via `watermark.enabled = true` dans
   `configs/worker.toml`. Ollama backend : deferred (l'API
   `ollama-rs` n'expose pas de hook logit pre-sampling).

**Rejete** :
- **Kirchenbauer KGW green-list** (ICML 2023, arXiv:2301.10226) :
  vulnerable BIRA attack (arXiv:2509.23019, sept 2025). Bidirectional
  Iterative Rewriting Attack exploite le biais statistique
  deterministe du green-list partitioning. Documente et rejete en
  S22 kickoff D4 + `canary_input.py:11-12`.
- **MarkLLM toolkit integration** : Python-only (THU-BPM/MarkLLM,
  EMNLP 2024). Pas de binding Rust. Toolkit academique, non
  production-ready. 9 algorithmes dont KGW (meme vulnerabilite
  BIRA). Pas d'integration Ollama/llama.cpp.
- **SynthID SDK direct** (`google-deepmind/synthid-text`) : dep
  Google-specifique (TensorFlow/JAX). Notre stack = Ollama +
  llama.cpp, pas Google. On retient le CONCEPT (Tournament Sampling
  + z-test detection) mais pas la lib. Implementation maison
  minimaliste, ~300 LOC Python detection + ~200 LOC Rust injection.
- **Detection-only sans injection** : non-concluant sur workers
  non-watermarked. La valeur est dans la combinaison
  injection+detection. Detection seule = canary-input suffit deja.
- **Full SynthID Tournament Sampling** : le Tournament Sampling
  modifie la distribution CDF complete des tokens, pas juste un
  bias additif. Plus complex a implementer dans le sampling pipeline
  llama.cpp. Le bias additif (green-list PRF-based, SANS le
  partitioning deterministe KGW vulnerable BIRA) est un compromis
  robuste et simple.

**Implications** :
- Nouveau `packages/nexus-coordinator/src/nexus_coordinator/watermark_detector.py`
  (~300 LOC)
- Update `crates/nexus-worker-core/src/llm/llama_cpp.rs` logit bias
  hook (~200 LOC)
- Nouveau `configs/watermark.toml.sample`
- Secret management : le coordinator genere un secret Ed25519-derived
  (reuse keypair existant) partage via le task dispatch (champ
  `watermark_seed` dans Task, pas de nouveau wire format — pre-launch
  protocol redefinit le canonical v1)

### D2 — Couche 3 multi-forge cross-validate : git-log parser offline + SQLite LRU

**Retenu** : implementation de la Couche 3 Sybil-resistance decrite
dans `docs/security/CONTRIBUTOR_ATTESTATION_RFC.md`. Architecture :

1. **Parser git-log offline** (Rust, nexus-core-rs) : execute
   `git log --show-signature --format=<custom>` sur un clone local.
   Parse les signatures GPG/SSH (RFC 4880 / RFC 8709) pour extraire
   le fingerprint de l'auteur. Chaque commit signe = une attestation
   de contribution. Aggregation par auteur : nb commits signes,
   premiere/derniere date, forges distinctes.

2. **Cross-forge aggregation** : le parser traite des repos clones
   depuis GitHub, GitLab, Codeberg, Gitea (multi-forge S14). Un
   meme contributeur (meme fingerprint Ed25519/GPG) ayant signe des
   commits sur 2+ forges recoit un score de confiance plus eleve
   (diversification geographique + juridictionnelle du trust anchor).

3. **SQLite LRU cache** (Rust) : les resultats du parser sont caches
   dans `~/.sbfb/attestation_cache.db` (pattern quarantine_queue.py
   S21 Phase D). TTL 7 jours. Evite de re-parser les repos a chaque
   requete.

4. **Integration DelegationCert** : le trust-score cross-forge
   alimente le champ `trust_level` de `DelegationCert` (S23
   `attestations/delegation.rs`). Le cert est signe par le delegateur
   (coordinateur ou ONG anchor).

**Rejete** :
- **Live forge API polling** (GitHub API / GitLab API) : latence
  (API rate limits 5000 req/h GitHub), necessite token OAuth
  (contradicts zero-admin), et les forges self-hosted (Gitea) n'ont
  pas d'API standardisee pour les signatures commit. Le clone +
  git-log offline est universel.
- **Blockchain attestation** : overhead delirant pour un reseau
  pre-v1.0. Contradicts zero-admin + AGPL-3.0 philosophy. Aucun
  projet OSS comparable n'utilise blockchain pour les attestations
  de contribution (Protocol Guild = Ethereum mais pour la
  distribution de fonds, pas pour l'attestation).
- **Radicle native verification** : Radicle utilise `git-ref` signe
  Ed25519 (pas GPG). Integration Radicle est track LT-2
  (conditionnel tag v1.0). Le parser GPG/SSH offline couvre les 4
  forges actuelles.

**Implications** :
- Nouveau `crates/nexus-core-rs/src/attestations/forge_parser.rs`
  (~400 LOC)
- Nouveau `crates/nexus-core-rs/src/attestations/trust_cache.rs`
  (~150 LOC)
- Update `crates/nexus-core-rs/src/attestations/delegation.rs`
  (trust_level field)
- Nouveau `crates/nexus-core-rs/src/attestations/trust_web.rs`
  (~150 LOC)

### D3 — Trust-web bootstrap : Amnesty-class ONG seed keys

**Retenu** : les trust anchors de la Couche 3 sont des organisations
verifiables (Amnesty International, Human Rights Watch, Committee to
Protect Journalists, Electronic Frontier Foundation). Le bootstrap
fonctionne comme suit :

1. **Seed keys hardcoded** : fichier `configs/trust_web_seeds.toml`
   avec les fingerprints Ed25519 des ONG (generes lors du partenariat
   formel S28 — pour S27, placeholder avec la cle FlowUP comme seul
   anchor bootstrap). Chaque entree : org_name, fingerprint, forge
   URLs, description.

2. **Delegation chain** : les ONG signent des `DelegationCert` pour
   les contributeurs qu'elles reconnaissent. Chaque cert contient :
   delegator_fingerprint, delegatee_fingerprint, trust_level (1-5),
   timestamp, expiry.

3. **Propagation gossip** : les certs sont publies via gossip topic
   `nexus-grid/trust-web/v1` (nouveau topic, pattern canary S18 E2).
   Les nodes collectent et verifient les certs. Web-of-trust :
   confiance transitive avec decay (level -1 par hop, minimum 1).

**Rejete** :
- **Centralized CA model** : contradicts P2P decentralise. Un CA
  = single point of failure + censure possible.
- **Keybase-style social proofs** : Keybase est defunct (acquis
  Zoom 2020, stagnation). Le model social-proof requiert des APIs
  tierces instables.
- **PKI/x509** : overhead disproportionne. Les certs x509 sont
  concus pour TLS, pas pour le trust P2P. Ed25519 + JCS canonical
  (pattern existant S22 Couche 1+2) est suffisant.
- **Auto-discovery trust sans anchor** : web-of-trust sans anchor
  = Sybil trivial (un attaquant genere N identites qui se font
  mutuellement confiance). Les ONG sont des anchors verifiables
  hors-reseau (reputationnels, institutionnels).

**Implications** :
- Nouveau `configs/trust_web_seeds.toml`
- Nouveau `crates/nexus-shell-daemon-core/src/trust_web.rs` (~200 LOC)
- Update `crates/nexus-shell-daemon/src/runtime.rs` (gossip subscribe
  trust-web topic)
- Nouveau gossip topic `nexus-grid/trust-web/v1`

### D4 — P2 batch : 3 items S26 audit + 4 items observes en Phase A

**Retenu** : resoudre en Phase A les 3 P2 carry de l'audit S26 + les
4 P2 observes non-carry :

| ID | Fix | Fichier |
|---|---|---|
| P2-A-1 | Wire `validate_stage_guard_map()` dans `Dispatcher.__init__` | `dispatcher.py` |
| P2-C-1 | Ajouter `logger.debug()` dans except `_emit_capability_event` | `capability_store.py` |
| P2-D-1 | Ajouter `description: str = ""` a `TaskHandlerDescriptor` + capturer `fn.__doc__` | `decorators.py` + `app.py` |
| P2-C-2 | JsonFileWriter rotation taille-based (`max_bytes` 10 MiB + `.1`/`.2` suffixes) | `nexus-events-core/lib.rs` |
| P2-C-3 | Rename `EtwWriter` → `TracingWriter` (reflet du comportement reel cross-platform) | `nexus-events-core/lib.rs` |
| P2-B-1 | Commenter le pattern `__aenter__/__aexit__` explicite MCP lifespan | `api/app.py` |
| P2-E-1 | (Informatif) Ne pas inclure estimations LOC dans plan S27 — integre dans les conventions | N/A |

Total : ~80 LOC de fix + ~40 LOC de tests.

**Rejete** :
- **Distribuer les P2 dans les phases B-D** : pattern Phase A P2
  batch etabli depuis S25. Cleanup d'abord, features ensuite.
- **Defer S28** : items < 30 LOC chacun, defer = gaming G7.

### D5 — Gate 3 scope : hardening docs + showcase apps reframing

**Retenu** : le scope Gate 3 S27 est la maturation des primitives
existantes et le reframing des showcase apps (pivot Gate 3 showcase →
triplette Alexandria / Surveillance foret / D&D, cf.
`docs/apps/LAUNCH_SHOWCASE.md`) :

1. **HARDENING_ROADMAP update** : mettre a jour la ligne S27
   (Kirchenbauer → SynthID, ajouter note BIRA, update compteurs).
   Gate 3 redefinie : "Alexandria, showcase apps" remplace
   "Gate 3 showcase, NEXUS cold-case".
2. **COMPUTE_THREATS update** : mettre a jour §4.4 watermark
   (SynthID remplace KGW, BIRA note, detection z-test).
3. **Gate 3 prerequisites checklist** : documenter dans
   `HARDENING_ROADMAP §7` les items restants pour Gate 3 unlock
   (post-S27 + audit externe S29). Alexandria = premiere app
   showcase (stockage distribue, MCP tools, pas de GPU requis).
4. **PATTERNS.md update** : ajouter pattern P37 watermark detector
   + P38 trust-web (si applicable, sinon defer).

5. **Self-distribution design doc** : nouveau
   `docs/release/SELF_DISTRIBUTION.md` — spec pour le concept "le
   protocole est son propre premier contenu" (SBFB binaries = blobs
   sur le reseau SBFB). Design doc consumee par sprint d'implem
   ~S30 (release prep pre-v1.0).

Items differes :
- Alexandria app implementation → post-Gate 3 (sprint dedie)
- Surveillance foret / D&D apps → post-Alexandria
- Self-distribution implementation → ~S30 (consomme le design doc)
- Full audit externe Cure53/ToB → S29

**Rejete** :
- **Showcase app in S27** : Gate 3 n'est pas encore unlock (manque
  audit externe S29). Les apps seraient prematurees.
- **Self-distribution code in S27** : les binaires ne sont pas encore
  stables (hardening en cours). Le design doc capture la vision, le
  code attend les binaires stables.
- **Scope creep docs** : la self-distribution est un design doc
  cible (~200 lignes), pas un document long-life lourd.

---

## Acknowledged review findings (G1)

Scoring : D1 ✅, D2 ✅, D3 ⚠️, D4 ✅, D5 ✅.
Rigor signal G4 satisfait (1 ⚠️ sur 5).

D3 ⚠️ : DelegationCert format non independamment verifie contre
standards attestation externes (C2PA Claim structures). Decision :
**adjust** — ajouter dans Phase C une sous-tache documentation du
format signature DelegationCert avec mapping C2PA
(`spec.c2pa.org/specifications/1.4/attestations/`). Le format
existant (S23 `attestations/delegation.rs`) sera documente formellement
avant l'extension trust-web, garantissant la migration S28 ONG.

Recommendation MED (R-S27-4 logit-bias + llguidance) : integree au
risk register et planifiee comme test integration Phase B.

Recommendation LOW (LT-2 Radicle 1.7.0+) : deja trackee dans
ROADMAP_COMMITMENTS, pas d'action S27.

---

## 5. Plan Phase outline A..E

### Phase A — P2 batch S26 audit (7 items)

**Scope** : resoudre les 7 P2 (3 carry + 4 observes) listes dans D4.
**Critere** : tous les tests existants passent + tests de regression
ajoutees pour chaque fix.
**Commit** : `feat(sprint27): Sprint 27 Phase A — P2 batch S26 audit 7 fixes`

### Phase B — Output watermark SynthID-inspired

**Scope** : watermark detector coordinator-side (z-test PRF-based) +
injection llama.cpp backend opt-in (logit bias delta).
**Critere** : tests unitaires detection (watermarked vs non-watermarked
output) + tests integration injection llama.cpp (mock) + config sample.
**Commit** : `feat(sprint27): Sprint 27 Phase B — output watermark SynthID-inspired z-test detection + llama.cpp injection opt-in`

### Phase C — Couche 3 multi-forge cross-validate + trust-web bootstrap

**Scope** : git-log --show-signature parser Rust + SQLite LRU cache +
cross-forge aggregation + DelegationCert trust_level + trust-web ONG
bootstrap seed config + gossip subscribe trust-web topic.
**Critere** : tests parser (GPG + SSH sigs) + tests cache + tests
DelegationCert avec trust_level + config sample.
**Commit** : `feat(sprint27): Sprint 27 Phase C — Couche 3 multi-forge cross-validate + trust-web ONG bootstrap`

### Phase D — Gate 3 showcase hardening + docs update

**Scope** : HARDENING_ROADMAP update SynthID + COMPUTE_THREATS §4.4
update + Gate 3 prerequisites checklist + PATTERNS.md update (si
applicable).
**Critere** : pas de test code (docs-only). Coherence interne des
docs verifiee par grep cross-references.
**Commit** : `docs(sprint27): Sprint 27 Phase D — Gate 3 showcase hardening docs + Gate 3 prerequisites update`

### Phase E — Wrap-up

**Scope** : verification.md + audit_plan S28 + memory update +
migration planning active → archive/v1.2/ (si sprint pair S28 ouvre
dans la foulee).
**Critere** : 25+ rows fail-fast verts, compteurs tests mis a jour,
memory nexus_grid_pivot.md et MEMORY.md a jour.
**Commit** : `chore(sprint27): Phase E — wrap-up + verification + audit plan S28 + migration`

---

## 6. Items carry/dette

### Carry S27 (absorbes)

| ID | Description | Source | Reports | Status S27 |
|---|---|---|---|---|
| P2-A-1 | validate_stage_guard_map non wiree | S26 audit | 1/3 | Phase A |
| P2-C-1 | emit_capability_event catch silencieux | S26 audit | 1/3 | Phase A |
| P2-D-1 | TaskHandlerDescriptor sans description | S26 audit | 1/3 | Phase A |
| P2-C-2 | JsonFileWriter sans rotation | S26 audit | 1/3 | Phase A |
| P2-C-3 | EtwWriter naming trompeur | S26 audit | 1/3 | Phase A |
| P2-B-1 | MCP lifespan aenter/aexit fragile | S26 audit | 1/3 | Phase A |

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

```bash
grep -A 5 "Condition de declenchement" docs/release/ROADMAP_COMMITMENTS.md
```

Resultat : toutes les conditions de declenchement sont latentes
(tag v1.0 non pose, iroh toujours 0.97, Gini non mesurable pre-prod,
pas de multi-worker deploy, pas de partnership ONG formelle). Aucun
item LT ne redevient carry actif.

---

## 7. Scope cuts — ce que Sprint 27 NE fait PAS

1. **Tor transport phase 1** → S28+ (arti-client toujours pre-1.0)
2. **Arti library-embed** → S28+ (conditionnel arti >= 1.0)
3. **Domain fronting implementation** → S28+ (legal review prereq)
4. **GPU lockup defense** → S28+ (dep A4 process roles)
5. **A4 process role tagging** → S28 (cluster D feature D2/D3)
6. **C1 SQLiteSession abstraction** → S28+ (pas prioritaire Gate 3)
7. **Ollama backend watermark injection** → S28+ (API limitation,
   pas de logit hook). Detection-only pour Ollama workers.
8. **SynthID Tournament Sampling complet** → S28+ (CDF modification
   complexe, bias additif simple suffit pour MVP)
9. **Platform writers complets** (journald, oslog) → S28 sprint pair
   (phase dette obligatoire §6.2.1 Regle 1)
10. **ONNX end-to-end CI fixture** (P2-B-1 S22 carry) → S28 phase
    dette
11. **Streaming bridge C5** → S28+
12. **Full Gate 3 showcase app** → post-Gate 3 (post-audit externe S29)

---

## 8. Tracabilite scope

Table mappant les items "What's NOT" S26 au sprint de prise en charge :

| Item S26 scope cut | Sprint cible | Phase |
|---|---|---|
| Tor transport → S27+ | S28+ | deferred (arti pre-1.0) |
| Domain fronting impl → S27+ | S28+ | deferred (legal) |
| GPU lockup → S27+ | S28+ | deferred (A4 prereq) |
| A4 process roles → S27+ | S28 | D2 broker/executor split |
| C1 SQLiteSession → S27+ | S28+ | deferred |
| Platform writers → S27 | S28 | phase dette (sprint pair) |
| 8 events wire restants → S27 | absorbed S27+ | incremental |
| HARDENING_ROADMAP S27 items | **S27** | Phases B-D |

---

## 9. Risk register

| ID | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R-S27-1 | Watermark bias additif insuffisant pour detection robuste (paraphrasing attack) | MED | MED | Canary-input S22 reste la defense primaire, watermark = defense-in-depth complementaire. Test de robustesse avec paraphrasing simule. |
| R-S27-2 | git-log parser GPG/SSH coverage incomplete (signatures non-standard) | LOW | MED | Scope limit GPG + SSH (RFC 4880 + 8709). Signatures X.509/S/MIME = scope cut S28+. |
| R-S27-3 | Trust-web sans ONG reelles (placeholder FlowUP) = pas de verification cross-org | MED | LOW | Architecture en place, bootstrap seed remplacable. S28 outreach ONG. |
| R-S27-4 | llama.cpp logit bias integration casse le structured output (llguidance S20 Phase D) | MED | HIGH | Test integration logit bias + llguidance. Si conflit : bias OFF quand grammar active (fallback). |
| R-S27-5 | S28 sprint pair + 12 scope cuts = dette accumulation | LOW | MED | Phase dette obligatoire S28 (§6.2.1 Regle 1). |

---

## 10. Audit gate pattern — rappel

- Phase 0 audit S26 DONE (PASS, `982cfd1`).
- Phase E wrap-up produira `sprint27_verification.md` +
  `sprint28_audit_plan.md`.
- Phase 0 Sprint 28 (prochain audit) = audit independant de S27.

---

## 11. Checkpoint de validation

Questions pour arbitrage utilisateur AVANT plan detaille :

1. **D1** : SynthID-inspired (z-test + bias additif) est-il le bon
   compromis vs Kirchenbauer rejete ? L'approche bias additif PRF
   (pas de Tournament Sampling complet) offre robustesse BIRA tout
   en restant implementable dans le sampling llama.cpp.

2. **D2** : Couche 3 via git-log --show-signature offline parser
   (GPG/SSH) est-il suffisant ? Alternative : API forge (mais
   rate-limits + tokens OAuth + pas universel Gitea).

3. **D3** : Trust-web bootstrap avec placeholder FlowUP (pas d'ONG
   reelle S27) — l'architecture prepare-t-elle correctement le
   partenariat S28 ?

4. **D4** : Les 7 P2 batch sont-ils tous pertinents Phase A, ou
   certains devraient-ils etre differes ?

5. **D5** : Le scope docs Gate 3 showcase (update HARDENING_ROADMAP +
   COMPUTE_THREATS + Gate 3 prerequisites) est-il suffisant, ou
   faut-il ajouter des items code Gate 3 showcase-specifiques ?
