# Sprint 28 — Audit findings (S29 Phase 0)

**Date** : 2026-04-26
**Auditeur** : session fraîche S29 Phase 0
**Tip audité** : `e18985d` (S28 Phase E wrap-up)
**Verdict global** : **PASS**

---

## Résumé exécutif

6 agents d'audit indépendants ont vérifié les 8 tracks + pre-launch
protocol sur le code et les docs post-S28. Aucun P0, aucun P1.
5 findings P2 et 1 finding P3 documentés ci-dessous, satisfaisant
le signal de rigueur G4 (≥1 P2+ requis pour PASS, sinon CONCERN).

Pas de commit fix nécessaire — tous les findings sont du tech debt
informatif à logger dans PATTERNS.md ou à résoudre en S29.

---

## Track A — Watermark end-to-end wiring

### WIRE-1 compute_bias call site — PASS
`llama_cpp.rs:324` : `should_inject` gate correcte (vérifie
`config_enabled && !seed.is_empty()`). `compute_bias` appelé
uniquement quand `wm_active = true` (lignes 342-372). Quand
`enabled = false`, sampling non biaisé (ligne 374).

### WIRE-2 output_token_ids — PASS
`llama_cpp.rs:419` : `generated_ids` accumulé via
`push(token.0 as u32)` (ligne 396). Propagé dans
`GenerateResponse` (ligne 237) puis `ResultPayload`
(`runtime.rs:1068`). Test serde roundtrip `mod.rs:388-410`
couvre vec non-vide et `skip_serializing_if = "Vec::is_empty"`.

### CONFIG-1 watermark.toml.sample — PASS
`configs/watermark.toml.sample` : TOML valide, `enabled = false`,
`delta_logit = 2.0`, `window_size = 4`. Commentaires documentent
chaque valeur.

### SEED-1 trust_web_seeds.toml — PASS
`configs/trust_web_seeds.toml:9,11` : fingerprint
`PLACEHOLDER_ED25519_REPLACE_AT_GO_LIVE_...` avec commentaire
`# PLACEHOLDER`. Pas de zero-padding silencieux.

### P37-1 PATTERNS.md — PASS
`docs/rust/PATTERNS.md:2124-2132` : P37 mentionne `watermark.rs`
comme source primaire ET `llama_cpp.rs` comme call site
d'intégration.

### P2-REVIEW-1 generate_blocking params — **P2**
`llama_cpp.rs:258` : `#[allow(clippy::too_many_arguments)]`
présent sur 12 paramètres. **Commentaire justificatif absent.**
Le code est fonctionnellement correct — la closure
`spawn_blocking` nécessite le move de tous les params.
→ Tech debt : ajouter commentaire inline S29.

### P2-REVIEW-2 sampler rebuild per-step — **P2**
`llama_cpp.rs:342-372` : chaîne sampler reconstruite à chaque
step (HMAC-SHA256 sur vocab_size ~32k). Acceptable au rate
limit actuel (60 req/min ≪ 100 req/s). **Aucun commentaire
ne documente l'hypothèse de charge.**
→ Tech debt : documenter load assumption S29.

---

## Track B — Platform writers + ONNX CI fixture

### JOURNALD-1 cfg gate — PASS
`nexus-events-core/src/lib.rs:207` : `#[cfg(target_os = "linux")]`
gate production. `lib.rs:226` : stub `#[cfg(not(...))]` présent.

### OSLOG-1 cfg gate — PASS
`lib.rs:241` : `#[cfg(target_os = "macos")]` gate production.
`lib.rs:254` : stub `#[cfg(not(...))]` présent.

### FORMAT-1 structured fields — PASS (note P3)
`format_journal_fields()` (`lib.rs:190-196`) retourne 2 champs
(SBFB_EVENT_TYPE, SBFB_DETAILS). Le plan spécifiait 4 (MESSAGE,
PRIORITY inclus), mais `libsystemd::journal_send(Priority, &msg,
fields)` prend MESSAGE et PRIORITY comme paramètres séparés — la
déviation est correcte per API. `format_oslog_message()`
(`lib.rs:198-201`) produit `[sbfb:EventType] {json}` lisible.

### ROUTING-1 init_platform_emitter — PASS
`lib.rs:283-292` : Linux→Journald, macOS→OsLog,
other→TracingWriter. 3 branches cfg mutuellement exclusives.

### ONNX-1 mock InferenceSession — PASS
`web/src/sdk/pii/__tests__/wrapper.test.ts:56-196` :
`DecoderExercisingLoader` exerce `decodeSpans` + `greedyDedup` +
`toFinding`. 4 tests couvrent e2e, overlapping spans, threshold,
multi-width. Pas de TODO S29 en commentaire code (documenté dans
plan S28 comme Option B retenue).

### EVENT-1 event_type_name — PASS
`lib.rs:173-188` : 12/12 variantes SecurityEvent matchées
explicitement. Aucun `_ =>` catch-all. Test `lib.rs:469-479`
vérifie consistency serde tag sur `all_variants()`.

### P2-B-1 native impls non testées — **P2**
Sur Windows dev, seuls les stubs JournaldWriter/OsLogWriter sont
compilés et testés. Les format helpers compensent
(`format_journal_fields` + `format_oslog_message` testés
cross-platform). Carry S29 : CI Linux/macOS.

### P2-B-2 init_platform_emitter sans test direct — **P2**
Fonction 10 LOC, 3 branches cfg triviales. Chaque writer testé
individuellement. Carry S29 : test direct (mineur).

---

## Track C — Process isolation design doc

### DOC-1 complétude — PASS
`docs/security/PROCESS_ARCHITECTURE.md` : 11 sections (intro,
archi, IPC, lifecycle, state, fault, security, migration,
questions, pointeurs, revue). Dépasse le minimum de 9.

### IPC-1 JSON-RPC vs gRPC — PASS
§3.2 : table comparative 6 critères avec chiffres (JSON-RPC
2-5 µs vs gRPC 1-2 µs, +500 KB binaire). Rationale : delta
latence négligeable vs inference 100 ms+/token.

### COLD-1 cold-start budget — PASS
§4.3 : cible < 5s documentée (spawn + connect IPC < 100 ms +
Ollama load 1-3 s + premier token 100-500 ms). RTX 5080
benchmark explicitement listé comme prérequis S29.

### SECURITY-1 privilege reduction — PASS
§2.2, §5, §7.1-7.2 : executor n'a PAS accès au keypair
Ed25519 ni au bearer master. Token éphémère per-task
HMAC-SHA256 dérivé. Documenté en 4 endroits.

### FAULT-1 crash isolation — PASS
§6.1 : crash executor sans crash broker, backoff exponentiel
1s→2s→4s→8s→16s→30s cap. Alerte après 5 crashes en 5 min.
Broker continue de répondre aux requêtes shell.

### P2-C-1 blob-serve dans broker — PASS
§7.1 table privilege + §9 Q4 : gap documenté explicitement.
Option A (statu quo S29), Option B (executor dédié S30+).

### P2-C-2 benchmark cold-start — PASS
§9 Q1 + §4.3 : "benchmark réel avant implémentation S29 Phase
D2" comme prérequis explicite.

---

## Track D — External audit scope + HARDENING_ROADMAP

### SCOPE-1 in/out — PASS
`EXTERNAL_AUDIT_SCOPE.md` §2 : 7 sous-sections scope-in (crypto
7 crates, wire 6 structures, auth, transport, sandbox, process
isolation conditionnel). §3 : 6 exclusions (UI React, docs, CI,
tests, coord Python, apps SDK).

### VENDOR-1 matrix — PASS
§4 : Cure53 vs Trail of Bits, 6 critères (spécialité, track
record, engagement type, cost $20-50k vs $50-100k, durée 2-4w
vs 4-8w, fit surface SBFB). Recommandation Trail of Bits.

### ROADMAP-1 S28 line — PASS
`HARDENING_ROADMAP.md` §3 S28 (lignes 659-707) : réécrit
post-delivery. Reflète les livrables réels (watermark wiring +
dette + design docs), pas l'aspirationnel initial.

### ROADMAP-2 Nym S30 — PASS
§3 S30 (lignes 755-785) : Nym carry S28 avec "deferred 2026-04-25
post-G9 factual" + trigger `nym-sdk beta stable` + fallback
re-defer S32+.

### ROADMAP-3 last_validated — PASS
Header ligne 3 : `last_validated: 2026-04-26` avec contexte G2.

### GATE3-1 Gate 3 items S28 — PASS
§7 (lignes 919-941) : 3 items S28 documentés dans la checklist
Gate 3 (watermark wiring, PROCESS_ARCHITECTURE, EXTERNAL_AUDIT_SCOPE).
Items restants (audit externe, THREAT_MODEL §9, Tor) assignés S29+.

### P2-D-1 Note réalisme S29-S30 — PASS
Pattern établi S25-S26. S29-S30 suivront au kickoff (pratique
standard — la note est écrite au kickoff, pas en avance).

### P2-D-2 versions crates "verify at engagement" — **P2**
`EXTERNAL_AUDIT_SCOPE.md` §2 liste les versions crates (ed25519-dalek
2.1, aes-gcm 0.10, etc.) mais **aucune note explicite "verify at
engagement"**. Les versions peuvent évoluer entre S28 et l'envoi
RFP S29.
→ Recommandation : ajouter §2.7 "Version verification at RFP
time" dans S29 kickoff.

---

## Track E — G1 Design Review Board

### Design review existence — PASS
`sprint28_design_review.md` dans archive/v1.2/, daté 2026-04-25.

### Scoring D1-D5 — PASS
D1 ✅, D2 ⚠️, D3 ⚠️, D4 ✅, D5 ⚠️. 5/5 scores présents.

### Kickoff §4 acknowledged findings — PASS
Chaque ⚠️ reçoit une réponse explicite dans le kickoff :
- D2 ⚠️ : comparaison `tracing-journald` → trait EventWriter
- D3 ⚠️ : analyse déférée → PROCESS_ARCHITECTURE.md Phase C
- D5 ⚠️ : sources externes vérifiées (VALIDATED_BLUEPRINT,
  NVIDIA MIG User Guide, Nym SDK status)

### G4 rigor signal — PASS
3 ⚠️ sur 5 → pas de rubber-stamp. Ligne kickoff 351 confirme
"Rigor signal G4 satisfait (3 ⚠️ sur 5)".

---

## Track F — G8 traceability

### Preflight files 4/4 — PASS
A, B, C, D : tous présents dans archive/v1.2/. Verdict : 4×
EXECUTE plan-as-is (0 PLAN-ADAPT, 0 DESIGN-CONFLICT).

### Review files 4/4 — PASS
A, B, C, D : tous présents. Verdict : 4× PASS. 8 P2 documentés
au total (2 par phase en moyenne).

### Cohérence verdict × commit — PASS
4 EXECUTE → 4 commits phase livrés (`c5f35f7`, `a43a1a1`,
`ccbb6ca`, `727a780`). 0 DESIGN-CONFLICT → 0 pivot_proposal.

---

## Track G — Sprint pair phase dette (§6.2.1 Règle 1)

### S28 pair → dette obligatoire — PASS
Phase B étiquetée "Phase dette (sprint pair obligatoire §6.2.1
Règle 1)" dans kickoff et plan.

### SC-9 platform writers (2/3) — PASS résolu
Phase B commit `a43a1a1` : JournaldWriter + OsLogWriter impls +
5 tests format helpers + stubs. Compteur 2/3 → résolu.

### SC-10 ONNX CI fixture (5+/3 escalade) — PASS résolu
Phase B commit `a43a1a1` : mock InferenceSession + Vitest
wrapper.test.ts. Compteur 5+/3 (escalade obligatoire) → résolu.

---

## Track H — HARDENING_ROADMAP drift

| Item prescrit | Livraison S28 | Justification deferral | Verdict |
|---|---|---|---|
| Nym mixnet | Déféré S30+ | G9 2026-04-25 : SDK beta, 0 code fondation, pas de SOCKS iroh 0.97 | PASS |
| MIG partitioning | Déféré post-v1.0 | G9 : RTX 5080 consumer, MIG = A100/H100 enterprise | PASS |
| D2 broker/executor code | Design-only → S29 | Prérequis PROCESS_ARCHITECTURE.md + benchmark cold-start | PASS |
| D3 Windows RPC | Déféré S29 | Co-landing D2, windows-rs maturity | PASS |
| C4 task-scoped sandbox | Déféré S29 | Co-landing D2 | PASS |
| External audit prep | **Livré** Phase D | EXTERNAL_AUDIT_SCOPE.md + vendor matrix | PASS |

Tous les deferrals justifiés dans kickoff §7 et HARDENING_ROADMAP
§3 post-delivery. Aucun drift non justifié.

---

## §4 Pre-launch protocol

### VERSION constants = 1 — PASS
7 constantes vérifiées (CURATOR_LIST_FORMAT_VERSION,
KEY_ROTATION_FORMAT_VERSION, BLOB_VERSION 0x01,
POW_FORMAT_VERSION, TASK_RESPONSE_VERSION,
TASK_FORMAT_VERSION, PIN_FILE_FORMAT_VERSION). Toutes = 1.

### Aucun tolerant decoder — PASS
Toutes les validations utilisent l'égalité stricte (`!=`).
Zéro pattern `1..=`, range match, ou multi-version. Tests de
rejet (version=99) couvrent curator, key_rotation, pow.

### Aucun zombie legacy test — PASS
Zéro test simulant un décodage "legacy" ou "previous version".
Tous les tests version sont des tests de rejet.

### Watermark interne worker — PASS
`watermark_seed` et `output_token_ids` dans `Task`/`ResultPayload`
(iroh-docs). Absents de `GossipEvent` et tous types gossip.

### Platform writers locaux — PASS
Zéro référence journald/oslog dans nexus-core-rs. Writers
confinés à nexus-events-core, émission locale uniquement.

---

## Meta-track — Findings phase reviews routés

Les 8 P2 documentés par les phase reviews (2 × A, 2 × B, 2 × C,
2 × D) ont été vérifiés indépendamment dans les tracks
correspondantes ci-dessus. Tous confirmés par l'audit.

---

## Tableau récapitulatif findings

| ID | Track | Sévérité | Description | Action |
|---|---|---|---|---|
| F-A-1 | A | P2 | `generate_blocking` 12 params : `#[allow(too_many_arguments)]` sans commentaire justificatif | Tech debt S29 |
| F-A-2 | A | P2 | Sampler chain rebuild per-step sans documentation load assumption | Tech debt S29 |
| F-B-1 | B | P2 | Impls natives JournaldWriter/OsLogWriter non testées fonctionnellement (Windows dev) | CI Linux/macOS S29 |
| F-B-2 | B | P2 | `init_platform_emitter()` sans test direct (trivial 3 branches cfg) | Test S29 |
| F-D-1 | D | P2 | Versions crates EXTERNAL_AUDIT_SCOPE.md sans note "verify at engagement" | §2.7 S29 kickoff |
| F-B-3 | B | P3 | `format_journal_fields` 2 champs vs 4 spécifiés (correct per API libsystemd) | Informatif |

---

## Verdict

**PASS** — 0 P0, 0 P1, 5 P2, 1 P3.

G4 rigor signal satisfait (5 P2 + 1 P3 ≥ 1 P2+ requis).

Aucun commit fix nécessaire. Les 5 P2 sont du tech debt
informatif à adresser en S29 (carry items).
