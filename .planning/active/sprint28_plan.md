# Sprint 28 — Plan d'execution detaille

**Ecrit** : 2026-04-25
**Kickoff** : `.planning/active/sprint28_kickoff.md`
**D1..D5** : gelees apres G1 review (D1 ✅, D2 ⚠️ ack, D3 ⚠️ ack,
D4 ✅, D5 ⚠️ ack)

---

## 1. Phase A — P2 batch S27 audit (4 items)

### 1.1 P2-B-1 : Watermark sampling pipeline wiring

**Fichiers touches** :
- `crates/nexus-worker-core/src/llm/llama_cpp.rs` — ajouter call
  `compute_bias` dans le sampling loop
- `crates/nexus-worker-core/src/engine/runtime.rs` — populer
  `output_token_ids` dans TaskResult
- `crates/nexus-worker-core/src/llm/watermark.rs` — verifier API
  publique (compute_bias, should_inject, prf_score)

**Specification** :
1. Dans `llama_cpp.rs`, avant la selection de token dans `generate()` :
   - Verifier `should_inject(&config)` (config = WorkerConfig section
     watermark)
   - Si actif : appeler `compute_bias(token_id, context_hash, secret,
     delta)` pour chaque token du vocabulaire → obtenir le bias array
   - Ajouter le bias aux logits avant softmax/sampling
   - Accumuler les token_ids generes dans un Vec<u32>
2. Dans `runtime.rs`, au moment de construire le TaskResult :
   - Populer `output_token_ids` avec le Vec accumule (ou vec![] si
     watermark desactive)
3. Gate : `watermark.enabled = true` dans WorkerConfig (defaut false)

**Tests** :
- Test unitaire : `llama_cpp.rs` avec mock backend verifiant que
  `compute_bias` est appele quand enabled=true
- Test unitaire : `runtime.rs` verifiant que `output_token_ids` est
  non-vide quand watermark=true
- Test integration : round-trip bias injection → detection z-test
  (si realisable sans GPU)

### 1.2 P2-B-2 : watermark.toml.sample

**Fichier** : `configs/watermark.toml.sample` (nouveau)

```toml
# Watermark configuration — opt-in per-worker
# Section integree dans worker.toml [watermark]

[watermark]
# Enable watermark injection in sampling pipeline (LlamaCpp backend only)
enabled = false

# Logit bias delta for green tokens (PRF score > 0.5)
delta = 2.0

# Context window size for PRF hash computation
window_size = 4
```

### 1.3 P2-C-1 : Fingerprint seeds.toml reel

**Fichier** : `configs/trust_web_seeds.toml`

Remplacer le fingerprint dummy `80b439cb0000...abcd` par le vrai
fingerprint Ed25519 FlowUP. Si pas de cle Ed25519 GPG/SSH FlowUP
disponible au moment de l'implementation : documenter un placeholder
explicitement etiquete `# PLACEHOLDER — replace with real Ed25519
fingerprint at go-live` (mieux que du zero-padding silencieux).

### 1.4 P2-D-1 : P37 path correction PATTERNS.md

**Fichier** : `docs/rust/PATTERNS.md` section P37

Corriger le chemin : P37 doit mentionner `watermark.rs` (computation
PRF + bias) comme source primaire, avec `llama_cpp.rs` comme call site
d'integration (post-Phase A wiring). Les deux fichiers sont pertinents :
watermark.rs = module, llama_cpp.rs = consumer.

### 1.5 Commit Phase A

```
feat(sprint28): Sprint 28 Phase A — watermark end-to-end wiring + P2 batch S27 audit
```

Body riche : delta tests, 4 P2 resolus (B-1/B-2/C-1/D-1), scope cuts
respectes.

---

## 2. Phase B — Phase dette (sprint pair, §6.2.1 Regle 1)

### 2.1 SC-9 : Platform writers journald + oslog

**Fichiers touches** :
- `crates/nexus-events-core/Cargo.toml` — ajout deps optionnelles
- `crates/nexus-events-core/src/lib.rs` — remplacement stubs

**Specification journald (Linux)** :
1. Ajouter dep `libsystemd` >= 0.7 avec `[target.'cfg(target_os =
   "linux")'.dependencies]`
2. Remplacer stub `JournaldWriter` :
   ```rust
   #[cfg(target_os = "linux")]
   impl EventWriter for JournaldWriter {
       fn write_event(&self, event: &SecurityEvent) -> Result<(), EventError> {
           use libsystemd::logging::journal_send;
           let fields = vec![
               ("MESSAGE", format!("{:?}", event)),
               ("PRIORITY", "6"), // LOG_INFO
               ("SBFB_EVENT_TYPE", event.event_type()),
               ("SBFB_DETAILS", serde_json::to_string(event)?),
           ];
           journal_send(fields.iter().map(|(k, v)| (k, v.as_str())))
               .map_err(|e| EventError::WriteFailed(e.to_string()))
       }
   }
   ```
3. `#[cfg(not(target_os = "linux"))]` : garder le stub fallback

**Specification oslog (macOS)** :
1. Ajouter dep `oslog` >= 0.2 avec `[target.'cfg(target_os =
   "macos")'.dependencies]`
2. Remplacer stub `OsLogWriter` :
   ```rust
   #[cfg(target_os = "macos")]
   impl EventWriter for OsLogWriter {
       fn write_event(&self, event: &SecurityEvent) -> Result<(), EventError> {
           use oslog::OsLog;
           let log = OsLog::new("com.sbfb.security", "events");
           log.with_level(oslog::Level::Default, &format!("{:?}", event));
           Ok(())
       }
   }
   ```
3. `#[cfg(not(target_os = "macos"))]` : garder le stub fallback

**Specification init_emitter routing** :
- Mettre a jour `init_emitter` pour selectionner automatiquement le
  writer natif : Linux → JournaldWriter, macOS → OsLogWriter,
  Windows/other → TracingWriter (defaut existant)

**Tests** :
- Test existant `stub_writers_noop` : adapter pour verifier que les
  stubs sont toujours utilises sur les plateformes non-cibles
- Nouveau test : `journald_writer_format` (mock journal_send, verifier
  les champs structures)
- Nouveau test : `oslog_writer_format` (mock OsLog, verifier subsystem)
- Tous les tests compilent et passent sur Windows (cfg gate → stubs)

### 2.2 SC-10 : ONNX CI fixture

**Fichiers touches** :
- `web/src/sdk/pii/__tests__/` — nouveau test fixture
- Possible `web/test-fixtures/` — mini model ONNX

**Specification** :
1. Creer un mini model ONNX (< 1 MB) qui simule l'interface GLiNER
   (input: token IDs + attention mask, output: span logits). Le
   model n'a pas besoin d'etre accurate — juste les bonnes shapes
   pour exercer le pipeline decoder.
2. Option A : export Python d'un model random-weights avec torch.onnx
3. Option B : si export trop complexe, creer un mock
   `onnxruntime-web` dans les tests (pas de vrai model, mock
   `InferenceSession.run()`) avec un TODO ONNX fixture reelle

**Risk R-S28-3** : si la creation du mini model depasse ~100 LOC ou
necessite des deps Python lourdes : scope-cut a Option B (mock) +
TODO S29.

**Tests** :
- Vitest : test end-to-end PII detection avec fixture/mock
- Verification que le pipeline decoder (decodeSpans + greedyDedup +
  toFinding) est exerce

### 2.3 Commit Phase B

```
feat(sprint28): Sprint 28 Phase B — platform writers journald/oslog + ONNX CI fixture
```

Body riche : delta tests, SC-9/SC-10 resolus, scope cuts respectes.

---

## 3. Phase C — Process isolation design doc

### 3.1 PROCESS_ARCHITECTURE.md

**Fichier** : `docs/security/PROCESS_ARCHITECTURE.md` (nouveau)

**Sections du design doc** :

1. **Introduction + motivation** : pourquoi separer broker et executor
   (fault isolation, privilege reduction, crash containment, prep VM
   runtime isolation)

2. **Architecture cible** :
   - Broker (`nexus-shell-daemon` refactore) : long-lived, surface
     minimale (bearer auth + routing + gossip subscribe + state
     persistence). Pas de GPU access, pas de model loading.
   - Executor (`nexus-executor` nouveau binaire) : spawn par broker,
     accede GPU, charge modele Ollama/llama.cpp, execute task, retourne
     resultat via IPC. Crash independant du broker.

3. **IPC boundary** :
   - Canal : Unix domain socket (Linux/macOS) / Named Pipe (Windows)
   - Protocole : JSON-RPC 2.0 (simplicite, debuggabilite, logs
     lisibles). Analyse comparative vs gRPC/protobuf :
     - JSON-RPC : ~2-5µs serialization serde_json sur payload <1KB,
       zero codegen, human-readable logs
     - gRPC : ~1-2µs protobuf, codegen obligatoire, binary logs
     - Sur UDS local : la difference est negligeable (<5µs), le
       bottleneck est le model inference (100ms+)
     - Decision : JSON-RPC 2.0 retenu (simplicite > micro-optimization
       sur IPC local)

4. **Executor lifecycle** :
   - Pool mode : N executors pre-spawned (N = nombre de models
     caches), idle timeout 5min
   - Spawn-on-demand mode : executor cree a la demande, killed apres
     task completion + 30s grace period
   - Recommendation : pool mode pour production (cold-start amortise),
     spawn-on-demand pour dev/test
   - Cold-start budget : < 5s premier token (benchmark RTX 5080 +
     Ollama 7B a mesurer S29 Phase D2 pre-commit)

5. **State ownership** :
   - Broker owns : keypair identity, gossip subscriptions, bearer
     tokens, curator lists, consent state, routing table
   - Executor owns : model runtime, GPU memory, sampling state,
     watermark injection state
   - Shared (via IPC) : task request/response, watermark config,
     health status

6. **Fault isolation** :
   - Executor crash → broker log + re-spawn avec backoff exponentiel
     (1s, 2s, 4s, max 30s)
   - Broker crash → all executors orphaned → executor self-exit
     apres 60s sans heartbeat broker
   - OOM executor → Linux OOM killer cible executor (cgroup isolee),
     broker survive

7. **Security implications** :
   - Executor n'a PAS acces au keypair identity (privilege reduction)
   - Broker forward le bearer token au executor pour le contexte task
     (token ephemere per-task, pas le master token)
   - Named Pipe DACL per-executor (pattern S16)

8. **Migration path** :
   - S29 Phase D2 : implementation broker (refactor shell-daemon) +
     executor (nouveau binaire)
   - S29 Phase C4 : task-scoped sandbox (per-task iframe + per-task
     executor)
   - S30+ : VM wrapper (WSL2/Virtualization.framework/systemd-nspawn)
     autour de l'executor

9. **Open questions (unknowns)** :
   - Cold-start Ollama 7B RTX 5080 : besoin benchmark reel
   - cgroup isolation Windows : pas de cgroups natifs, alternative =
     Job Objects (Win32)
   - Model cache partage entre executors : shared filesystem ou
     IPC model-pull ?

### 3.2 Commit Phase C

```
docs(sprint28): Sprint 28 Phase C — process isolation PROCESS_ARCHITECTURE.md design doc
```

Body riche : design doc complet, 9 sections, scope Phase C respecte
(design-only, pas de code).

---

## 4. Phase D — External audit scope + HARDENING_ROADMAP update

### 4.1 EXTERNAL_AUDIT_SCOPE.md

**Fichier** : `docs/security/EXTERNAL_AUDIT_SCOPE.md` (nouveau)

**Sections** :

1. **Objectif** : audit externe independant pre-Gate 3 unlock
2. **Scope in** :
   - Crypto primitives : Ed25519 (keypair, canary, DelegationCert),
     AES-256-GCM (keystore), Argon2id (KEK derivation), FROST
     (canary threshold), HMAC-SHA256 (watermark PRF)
   - Wire formats : canonical JCS (Task, CuratorList, CanarySigned,
     AgeWitness, ContributorAttestation, DelegationCert)
   - Auth : bearer token (X-SBFB-Token), UDS/Named Pipe peer creds,
     Host/Origin allowlist
   - Transport : iroh 0.97 (gossip, blobs, DHT pkarr), TLS SPKI
     pinning, PoW Hashcash
   - Sandbox : blob-serve iframe CSP, postMessage bridge 3 methodes
   - Process : broker/executor split (si livre S29)
3. **Scope out** :
   - UI React (pas de surface de securite directe)
   - Docs/planning
   - CI/CD (GitHub Actions)
   - Test infrastructure
4. **Vendor matrix** :
   | Critere | Cure53 | Trail of Bits |
   |---|---|---|
   | Focus | Web + infra + API | Crypto + protocol + formal |
   | Budget | $20-50k | $50-100k |
   | Duree | 2-4 semaines | 4-8 semaines |
   | Deliverables | Report + remediation check | Report + formal analysis |
   | Best fit SBFB | Auth + transport + sandbox | Crypto + wire formats |
5. **Recommendation** : Trail of Bits (crypto/protocol focus aligne
   avec surface SBFB = 7 crypto primitives + 6 wire formats + novel
   watermark PRF scheme). Budget $50-80k, 4 semaines.
6. **Pre-conditions S29** :
   - PROCESS_ARCHITECTURE.md livre (S28 Phase C)
   - THREAT_MODEL §9 per-mode residual risk (S29 B4)
   - Stabilite wire formats (pre-launch protocol v1 figee)
7. **Timeline** : S29 Phase A scope finalize → RFP send → engagement
   → S29 Phase D audit execution → findings → remediation buffer

### 4.2 HARDENING_ROADMAP update

**Fichier** : `docs/security/HARDENING_ROADMAP.md`

Updates :
1. §3 S28 : mettre a jour le titre et les items pour refleter le
   sprint reel (watermark wiring + dette + process isolation design
   + audit scope) au lieu de l'aspirationnel (Nym + MIG + D2/D3/C4)
2. §3 S28 Nym : ajouter note "G9 2026-04-25 : SDK beta, deferred
   S30+ post-Gate 3"
3. §3 S28 MIG : ajouter note "G9 2026-04-25 : A100/H100 only,
   deferred post-v1.0 quand workers enterprise disponibles"
4. `last_validated` : mettre a jour a 2026-04-25 avec description S28
5. §3 S30 : ajouter "Nym mixnet integration phase 1" (deplace de S28)

### 4.3 Gate 3 checklist update

Si applicable : mettre a jour Gate 3 prerequisites checklist pour
refleter la completion du watermark wiring (Phase A) et l'audit
scope doc (Phase D).

### 4.4 Commit Phase D

```
docs(sprint28): Sprint 28 Phase D — external audit scope + HARDENING_ROADMAP update
```

Body riche : 2 docs nouveaux, HARDENING_ROADMAP mis a jour, Nym/MIG
deferrals documentes.

---

## 5. Phase E — Wrap-up

### 5.1 Livrables

1. `sprint28_verification.md` — fail-fast checklist 20+ rows
2. `sprint29_audit_plan.md` — plan pour audit independant S28
3. `sprint28_carry_summary.md` — carry-overs S29
4. Migration `.planning/active/sprint{27,28}_*.md` →
   `.planning/archive/v1.2/` (sauf S29 audit_plan qui reste pour
   S29 Phase 0)
5. Updates :
   - `CLAUDE.md` §Etat actuel (compteurs, carries)
   - `docs/claude/SPRINT_LOG.md` (row S28)
   - Memory `nexus_grid_pivot.md` (tip + compteurs)
   - Memory `MEMORY.md` (index)

### 5.2 Commit Phase E

```
chore(sprint28): Phase E — wrap-up + verification + audit plan S29 + migration
```

---

## 6. Fail-fast checklist (preview)

| # | Check | Phase |
|---|---|---|
| 1 | `compute_bias` call site dans llama_cpp.rs | A |
| 2 | `should_inject` gate watermark.enabled | A |
| 3 | `output_token_ids` populated runtime.rs | A |
| 4 | Test unitaire bias injection mock | A |
| 5 | Test unitaire output_token_ids non-vide | A |
| 6 | `configs/watermark.toml.sample` parsable | A |
| 7 | `trust_web_seeds.toml` fingerprint reel ou placeholder etiquete | A |
| 8 | P37 PATTERNS.md path correct (watermark.rs + llama_cpp.rs) | A |
| 9 | JournaldWriter impl `#[cfg(target_os = "linux")]` | B |
| 10 | OsLogWriter impl `#[cfg(target_os = "macos")]` | B |
| 11 | Stubs fallback preserves sur plateformes non-cibles | B |
| 12 | init_emitter routing auto platform | B |
| 13 | Test mock journald_writer_format | B |
| 14 | Test mock oslog_writer_format | B |
| 15 | ONNX CI fixture ou mock InferenceSession | B |
| 16 | Vitest PII decoder exerce avec fixture | B |
| 17 | PROCESS_ARCHITECTURE.md 9 sections | C |
| 18 | IPC boundary JSON-RPC 2.0 spec | C |
| 19 | Cold-start budget < 5s documente | C |
| 20 | EXTERNAL_AUDIT_SCOPE.md scope in/out/vendor matrix | D |
| 21 | HARDENING_ROADMAP S28 line updated | D |
| 22 | HARDENING_ROADMAP S30 Nym line added | D |
| 23 | HARDENING_ROADMAP last_validated updated | D |
| 24 | Rust nextest 821+ all pass | all |
| 25 | Python SDK 195+ pass | all |
| 26 | Python coord 391+ pass (36f stale wheel) | all |
| 27 | Vitest 264+ pass | all |
| 28 | cargo fmt + clippy clean | all |

---

## 7. Dependances inter-phases

```
Phase A (P2 batch) — independant
Phase B (dette) — independant de A
Phase C (design doc) — independant de A/B
Phase D (audit scope) — depend C (PROCESS_ARCHITECTURE.md reference)
Phase E (wrap-up) — depend A/B/C/D
```

Phases A, B, C sont parallelisables en theorie. Phase D depend C.
Phase E est sequentielle finale.

---

## 8. Risk mitigations par phase

| Phase | Risk | Mitigation |
|---|---|---|
| A | R-S28-1 watermark+llguidance conflit | Test integration. Si conflit : bias OFF quand grammar active. |
| B | R-S28-2 platform writers non testables dev | Trait mock testing. cfg gate compile-only. |
| B | R-S28-3 ONNX fixture complexe | Scope-cut a mock + TODO si > 100 LOC. |
| C | R-S28-4 design incomplet sans benchmark | Doc identifie unknowns explicitement. Benchmark = prereq S29. |
| D | — | Low risk (docs only). |
