# Sprint 76 — Plan d'exécution : GPU partagé volontaire, prouvé cross-machine

> Feuille de route ligne-par-ligne. Phases A-G, chacune = 1 commit atomique avec
> G8 preflight → code → suites vertes (dual-platform fail-fast) → review-deep →
> Codex → reconciliation PASS → commit body 9 sections → memory. **Sprint PAIR :
> Phase B est la phase dette RÉSERVÉE, NON convertible en feature (Règle 1 G7).**
> Phase 0 audit gate S75 = CONDITIONAL PASS (`73831c0`), P1 `DURESS-BOOT-LEAK`
> levé `23a08c9` — **condition de blocage Phase A LEVÉE**.

**Écrit** : 2026-06-15.
**Tip master** : `23a08c9` (audit findings S75 : 0 P0, 1 P1 FIXÉ `23a08c9`, 14 P2, 6 P3).
**Roadmap** : Sprint 6/6 de l'Arc 3.5 (clôture de l'arc), v2.1 — Protocole Neutre
+ Factory/RRV. Source : `.planning/roadmap_v5_factory_complete_vision.md` (section
« S75 — GPU partagé volontaire, prouvé cross-machine », lue comme S76 par
l'amendement S75 : GPU décalé S75→S76, sharding pipeline → S77).

---

## §1 État vérifié à l'entrée

Compteurs documentés au tip `23a08c9` (non re-runnés au plan — la fail-fast
Win+Docker sera re-jouée comme gate AVANT push, `feedback_wsl_before_push`). Le
P1 fix `23a08c9` a ajouté ~+2 tests duress (gate 2 chemins) au-dessus de la
baseline UX-ARRIVAL. Source of truth mesurable de sortie = §Fail-fast checklist
(colonne Observed remplie en `sprint76_verification.md`).

| Suite | Count | Commande | Observed |
|---|---|---|---|
| Rust nextest (Windows natif) | ~1761 | `cargo nextest run --workspace --locked` | |
| Rust nextest (Docker Linux canonique) | ~1765 | `rust:1.94` nextest workspace (image `sbfb-ci`) | |
| Rust doctests | ok | `cargo test --workspace --locked --doc` | |
| cargo fmt | 0 diff | `cargo fmt --all --check` | |
| cargo clippy | 0 warnings | `cargo clippy --workspace --all-targets --locked -- -D warnings` | |
| Vitest `web/` | 379 | `(cd web && npm run test:unit)` | |
| Vitest factory-operator | 7 | `(cd tools/factory-operator && npm run test)` | |
| coverage web | 87.17/79.01/85.92/88.5 (≥ 85/85/78/85) | `(cd web && npm run test:coverage)` | |
| size-limit | 6/6 | `(cd web && npm run size)` | |
| release build | ok | `cargo build -p nexus-shell-daemon --release` | |
| **Total** | **~2158** | (Rust nextest + Vitest 379 + factory 7 + size 6) | |

---

## §2 Décisions Day 0 (gelées — détail kickoff §4)

| D# | Décision | Implication code (fichiers vérifiés, file:line) |
|---|---|---|
| D1 | Surface « offrir ma puissance » : REUSE consent (`OwnProjects/OpenSource/Whitelist/All`) + caps + GPU monitor ; net-new MINCE (front + champ additif snapshot + bascule enrôlement worker co-localisé). Réconcilier préfixe `/api/v1`. | `consent.rs:84-435` REUSE intégral 0 logique ; `state_writer.rs:60-204` (+champ additif `consent`, 0 bump `SCHEMA_VERSION=1`) ; `engine/runtime.rs:929-982` (passer niveau+usage au flush) ; **point clé** `local_worker.rs:259-313` (worker co-localisé lit `consent.json` user si `OpenSource`/`All`, vs `Whitelist[own_doc]` hardcodé `:307-308`) ; `http.rs:423-430` routes daemon ; front `GpuConsentDialog.tsx`, `consent.ts`, `Network.tsx`, `coordinator.ts` ; `vite.config.ts` (préfixe). |
| D2 | E2E cross-machine task-routing compute (lève B-3) = acceptance LIVE sur iroh-docs/blobs (transport forcé par iroh 0.98 gelé + modèle S75 prouvé). | **ZÉRO changement mécanique** : `dispatch_loop.rs:23-60` (sole-writer), `engine/runtime.rs:847-919` (pompe pull), `result_sync.rs:142` (`spawn_result_subscribe`) câblé `runtime.rs:692`, `validator.rs:219-338` (quorum), `task.rs:319-435` (sign/verify). Livrable = acceptance LIVE scriptée + gate anti-régression `runtime.rs:3629`. |
| D3 | Quorum redundancy>1 sur sorties DÉTERMINISTES : cohorte homogène exact-match (étage 1, prouvé S76) + TOPLOC en réserve documentée (étage 2). Validator INCHANGÉ. | `validator.rs:202-338` quorum exact-match **INCHANGÉ** (verrou) ; `engine/runtime.rs:1260-1285` + `llm/mod.rs:253-263` `deterministic(seed)` déjà câblé, seed cross-worker-stable ; **`task.rs:374` `model_digest` = `blake3(nom)` (`runtime.rs:1082`) à durcir → hash GGUF [P1 ou doc-note]** + `task.rs:383` `logprobs_hash` = slot TOPLOC étage 2 ; `capability_store.rs` + `dispatcher.rs:37-133` = routing cohorte homogène (net-new). |
| D4 | Dashboard contributeur : vue d'agrégation contributeur sur le ledger kudos existant (per-task natif, EMA `alpha=0.97`). Kudos non-monétaire, zéro token crypto (gelé). | `kudos_ledger.rs:124-163` `get_contributor_kudos` miroir `get_project_kudos` ; `db.rs:1025-1068` query `WHERE worker_node_id=?1` + **index SQLite `worker_node_id`** ; `kudos_api.rs:44-144` handler `contributor_dashboard(Path(node_id))` ; `validator_loop.rs:108-120` + `http.rs:3342-3351` = point anti-gaming `tokens_generated` ; `consent.rs:229-328` `usage.json` = GPU-heures locales ; front page contributeur (réutilise `Network.tsx`). |
| D5 | Quantization 4-bit DOCUMENTÉE (GGUF doc-only ; runtime quant déjà présent inchangé ; cible single-GPU ≤14B modèle entier ; gros modèles = sharding cross-machine 2 machines × 1 GPU = S77 — arbitrage PO « personne n'a 2 GPU », mono-machine 2-GPU enterré). | `llm/llama_cpp.rs:143-164` câble UNIQUEMENT `with_n_gpu_layers` — **inchangé** (tensor-split = S77) ; `config.rs:331-356` `LlamaCppConfig` inchangé ; `gpu/mod.rs:147-151` `vram_budget_remaining_bytes` + `consent.rs:417-432` cap VRAM réutilisables ; doc cible `docs/operators/QUANTIZATION.md` + lien panneau D1. |

**Findings G1 (`sprint76_design_review.md`)** : 3 ⚠️ (D1, D2, D3), tous **adjust**,
corrections appliquées inline au kickoff §4. D4 ✅ / D5 ✅. **Arbitrages PO
Checkpoint §11 (TRANCHÉS)** : D1 = `OpenSource`+`All` ouvrent ; D2 = convergence
`result:` WAN = 1er critère falsifiable ; D3 = durcir `model_digest` (nom→GGUF) en
**P1 phase compute** ; D4 = **durcir maintenant** `log_utility(median(tokens))` du
groupe d'accord + sanity-bound (Phase E `credit()`) ; D5 = **doc-only, mono-machine
2-GPU enterré** (« personne n'a 2 GPU »), gros modèles = sharding cross-machine S77.

---

## §3 Graphe de dépendances inter-phases

```
Phase 0 (audit gate S75 DONE = CONDITIONAL PASS, P1 levé 23a08c9)
   │
   ▼
A (panneau « offrir ma puissance » + enrôlement worker co-localisé, D1)
   │   └─> dépendance aval : enrôlement at-large produit un worker public
   │       réellement servant, pré-requis du LIVE compute cross-machine.
   ▼
B (DETTE RÉSERVÉE non convertible — duress + 3 anti-escalade + tests + doc)
   │   └─> insérée tôt : ferme le lot duress local-only et les carries 2-reports
   │       AVANT que les phases compute ne touchent les mêmes fichiers (http.rs,
   │       runtime.rs, browse.rs). PULL-3 (B3) prépare le dial-set du quorum (C/D).
   ▼
C (E2E cross-machine B-3 palier 1 + cohorte homogène, D2 + D3 étage 1)
   │   └─> dépend de A (worker public servant) + B3 (PULL-3 dial-set failover).
   │       Corrige model_digest + advertit le tuple capability AVANT le quorum.
   ▼
D (quorum redundancy>1 prouvé déterministe palier 2, D3 étage 1 suite)
   │   └─> dépend de C : le routing cohorte-homogène (model_digest+tuple) DOIT
   │       exister avant de prouver redundancy=2 byte-identique. StubBackend
   │       hermétique + LIVE VPS+PC+Mac.
   ▼
E (dashboard contributeur, D4)
   │   └─> dépend de D : la vue contributeur agrège des lignes kudos créditées
   │       APRÈS quorum-accept (validator_loop.rs:70). Décision anti-gaming
   │       tokens_generated cohérente avec le groupe d'accord quorum prouvé en D.
   ▼
F (quantization 4-bit documentée, D5)
   │   └─> dépend de A (lien depuis le panneau) + D3 (pré-condition quorum
   │       même-GGUF documentée). Doc-only, pas de dépendance code dure.
   ▼
G (wrap-up + acceptance LIVE consolidée + clôture Arc 3.5 6/6)
       └─> dépend de A..F : verification.md fail-fast + sprint77_audit_plan +
           THREAT_MODEL + PATTERNS + memory + SPRINT_LOG + roadmap (Arc 3.5 clos).
```

**Justification de chaque dépendance** :
- **A → B** : B est insérée juste après A pour fermer le lot duress et les carries
  2-reports avant que C/D/E ne modifient `http.rs`/`runtime.rs`/`browse.rs` (mêmes
  fichiers). Éviter les conflits de touche et purger la dette d'abord.
- **B → C** : B3 (PULL-3 cross-tier failover) câble la chaîne de fallback du
  dial-set ; un dial-set vide affaiblit la redondance du quorum (C/D). PULL-3 est
  un pré-requis mécanique du compute multi-worker fiable.
- **A → C** : sans l'enrôlement at-large livré en A, aucun worker public ne sert
  les tâches du réseau — l'acceptance LIVE B-3 (palier 1) n'a pas de worker
  servant.
- **C → D** : le routing cohorte-homogène (`model_digest` durci + tuple capability)
  posé en C est la pré-condition du quorum déterministe : prouver redundancy=2
  byte-identique exige que les 2 réplicas soient routés vers des workers homogènes.
- **D → E** : la vue contributeur agrège des lignes kudos créditées uniquement
  après `Accepted` (`validator_loop.rs:70`) ; la décision anti-gaming
  `tokens_generated` (median du groupe d'accord) dépend du quorum prouvé en D.
- **A,D3 → F** : la doc quant lie depuis le panneau A (« ta carte 16GB → ≤14B »)
  et documente la pré-condition quorum même-GGUF posée en D3. Pas de dép code dure
  (doc-only) mais dépendance de contenu.
- **A..F → G** : G consolide l'acceptance LIVE, remplit la fail-fast, écrit les
  artefacts de clôture d'arc.

---

## §4 Phase A — Panneau « offrir ma puissance » + enrôlement worker co-localisé (D1)

### A.1 Scope

Exposer une surface front « offrir ma puissance » qui réutilise intégralement le
moteur consent (`OwnProjects/OpenSource/Whitelist/All`, caps W/VRAM/h,
UsageTracker, ConsentWatcher fail-closed — 25 tests `consent.rs`, 0 changement de
logique) et le rend **opérationnel at-large**. Le travail est de l'exposition +
wiring, pas de la primitive. Trois composants nets-new minces : (1) un champ
additif `consent` au `WorkerStateSnapshot` (niveau actif + caps consommés vs
plafonds) — **additif ⇒ PAS de bump `SCHEMA_VERSION=1`** (autorisé par le
doc-comment `state_writer.rs:23-29`) ; (2) l'enrôlement worker co-localisé : quand
l'utilisateur active le partage public (`OpenSource`/`All`), le worker
co-localisé **lit le `consent.json` utilisateur** au lieu du `Whitelist[own_doc]`
hardcodé (`local_worker.rs:307-308`), `All` restant un opt-in **double-confirmé** ;
(3) la page front (intention « Offrir ma puissance au réseau », pas
`consent/set`/`kind/provider`) avec une jauge caps-consommés.

**Pré-requis bloquant (D1 adjust)** : la première tâche est de réconcilier le
préfixe de route. Le client front POST `/consent/set`/`/consent/get`
(`consent.ts:80,122`) alors que le daemon monte `/api/v1/consent`/`/api/v1/
consent/set` (`http.rs:423-424`). Vérifier `web/vite.config.ts` (proxy dev) et le
comportement en build packagée ; si la page consent est inerte en prod, c'est un
`fix(sprint76)` légitime dans la phase, pas un report. Critère : POST consent
depuis le front packagé écrit réellement `consent.json`.

**Sémantique d'enrôlement TRANCHÉE (D1 adjust, noms d'enum réels
`consent.rs:391-432`)** : `OwnProjects`/`Whitelist`(least-priv) = OFF (le worker
co-localisé garde son `Whitelist[own_doc]` actuel) ; `OpenSource`/`All` = le
worker co-localisé lit le `consent.json` utilisateur ; `All` = opt-in
double-confirmé (cohérent `threatNote` « risque maximum » `GpuConsentDialog.tsx`).
Pause = retomber au niveau least-privilege (réversible instantané via
ConsentWatcher `notify`).

### A.2 Livrables

| Livrable | Description |
|---|---|
| Réconciliation préfixe route | `web/src/api/consent.ts` (ou `vite.config.ts` proxy) : POST/GET consent atteint le daemon (`/api/v1/consent*`) en dev ET en build packagée. Si trou prod confirmé, fix dans la phase. |
| Champ additif `ConsentSnapshot` | `state_writer.rs` : `pub struct ConsentSnapshot { level: u8, max_hours_day: Option<f64>, hours_used_today: f64, max_watts: Option<u32>, max_vram_mb: Option<u64> }` + `WorkerStateSnapshot { ..., #[serde(default)] pub consent: Option<ConsentSnapshot> }`. **0 bump `SCHEMA_VERSION`** (additif). Doc-comment du champ : rationale runtime-tolerance, pas legacy-compat. |
| Pompe → snapshot | `engine/runtime.rs:929-982` : passer `self.consent.current()` (niveau) + `self.usage` (`hours_used_today`) à `SnapshotInputs` au flush tick. |
| Enrôlement worker co-localisé | `local_worker.rs:259-313` : si le `consent.json` utilisateur est `OpenSource`/`All`, provision le worker co-localisé avec ce niveau (lecture du `consent.json` user) ; `OwnProjects`/`Whitelist` conservent `Whitelist[own_doc]` least-privilege ; `All` requiert un opt-in double-confirmé. |
| Type front `WorkerStateV1` étendu | `web/src/api/coordinator.ts` : `consent?` optionnel (Zod tolérant, le champ peut être absent d'un worker ancien). |
| Page front « offrir ma puissance » | Réutilise `GpuConsentDialog` (choix niveau + sliders existants) + `GpuCard` (`Network.tsx:197-253`) + nouvelle jauge « X h / Y h aujourd'hui » + « niveau actif » alimentée par `consent`. CTA principal = intention « Offrir ma puissance au réseau ». Strings FR (`scan-en-strings`). |

### A.3 Tests plan

1. `consent_snapshot_serializes_additively` (Rust) — un `WorkerStateSnapshot`
   sans `consent` désérialise à `None` (champ omis) ; `SCHEMA_VERSION` reste 1.
2. `consent_snapshot_carries_level_and_usage` (Rust) — le flush snapshot porte le
   niveau actif + `hours_used_today` de l'`UsageTracker`.
3. `colocated_worker_honors_user_consent_when_public` (Rust) — quand le
   `consent.json` user est `OpenSource`/`All`, le worker co-localisé adopte ce
   niveau (pas `Whitelist[own_doc]`).
4. `colocated_worker_least_privilege_when_off` (Rust) — `OwnProjects`/`Whitelist`
   → le worker co-localisé reste verrouillé sur son propre doc.
5. `consent_route_reaches_daemon_prefix` (Vitest ou Rust selon le fix) — un POST
   consent depuis le client front atteint le préfixe daemon `/api/v1/consent/set`.
6. `offer_power_page_renders_caps_gauge` (Vitest) — la page rend la jauge
   « h/h aujourd'hui » + niveau actif depuis le champ `consent` du snapshot.
7. `offer_power_cta_is_intention_not_jargon` (Vitest) — le CTA affiche
   « Offrir ma puissance au réseau » (FR), pas `consent/set`/`kind/provider`.

### A.4 Critère d'acceptation

```
cargo nextest run -p nexus-worker-core -p nexus-shell-daemon --locked
(cd web && npm run test:unit && npx tsc --noEmit -p tsconfig.app.json && npm run lint)
bash web/scripts/scan-en-strings.sh
grep -n "SCHEMA_VERSION: u32 = 1" crates/nexus-worker-core/src/engine/state_writer.rs   # inchangé
```
Test #3 PASSE (le worker co-localisé sert public quand l'utilisateur l'a activé).
Le POST consert depuis le front packagé écrit `consent.json` (critère #5).
`SCHEMA_VERSION` reste 1 (grep).

### A.5 Commit cible

`feat(daemon+shell): Sprint 76 Phase A — offer-my-power panel + co-located worker enrollment`
Body 9 sections : (1) `## Contexte` (D1, exposition+wiring du moteur consent mûr,
préfixe route réconcilié, sémantique enrôlement tranchée) ; (2) `## Fichiers`
(table) ; (3) `## Delta tests` (+5 Rust, +2 Vitest, décompo per-module) ; (4)
`## Verification §7.4` (CI manifest) ; (5) `## Scope cuts respectes (kickoff §7)`
(exhaustif : pas de scheduler idle BOINC, pas de self-test enrôlement, pas de flag
`enabled` séparé) ; (6) `## G8 traceability` (SHA preflight + verdict + SHA review
+ PASS Codex) ; (7) `## Pre-launch protocol` (0 bump `SCHEMA_VERSION`, champ
`consent` additif, `*_VERSION` à 1) ; (8) `## Codex verification` ; (9)
`## Carry closure / Unblock` (débloque l'acceptance LIVE C : worker public servant).

---

## §5 Phase B — DETTE RÉSERVÉE (non convertible en feature, Règle 1 G7)

### B.1 Scope

Sprint PAIR → cette phase est la **dette réservée, NON convertible en feature**.
Le P1 `DURESS-BOOT-LEAK` étant déjà fixé (`23a08c9`), Phase B est purement
dette/refacto/tests/doc. Aucun item 3/3 MANDATORY à l'entrée, mais **4 carries
sont à 2 reports** : 3 (CARRY-3, LOOPBACK-TIERS-STALE, PULL-3) sont traités ICI
pour casser l'escalade G7 (un 3e report = MANDATORY phase S77), le 4e
(SYBIL-SEEDER-TAIL) est reconduit S77 avec exemption « dépendance interne
sharding » nommée. La phase ferme aussi le **lot duress freres local-only**
(miroir du P1 wire-emit déjà fermé, mais côté mutation locale), les trous de
couverture test (5 pages front 0-test + CI Playwright no-op + shell T6/T7), et
plusieurs corrections doc THREAT_MODEL/LOOPBACK.

**Priorité si débordement (R5)** : B1/B2/B7 (le lot duress restant + les 2-reports
anti-escalade) ; B9 (tests) **non sacrifiable** (Règle 1 : la phase dette inclut
les tests manquants, non convertible). Les items B sont scopés Rust (B1-B5),
front/test (B6/B9/B10), doc (B7/B8/B11).

### B.2 Livrables

| Item | Livrable (exit condition binaire) |
|---|---|
| **B1. Lot duress freres local-only** (DURESS-FRERES-LOCAL + Publisher-binding observed) | `seed_voluntary` + `set_keep_online` (`http.rs:1529-1577`, `:2066/2206`) court-circuitent en `IdentityMode::Duress` (no-op + réponse leurre cohérente, miroir `run_boot_seed_driver`) ; capture observed (`iroh_runtime.rs:507-521`) liée à l'identité PoW publisher (borne la forge). |
| **B2. CARRY-3-AGGREGATOR-SANITIZE** (2 reports) | `trustworthy_open_source` (`is_open_source && provenance_hash.is_some() && repo_url.is_some()`) re-appliqué à l'INGRESS aggregator (`runtime.rs:2231`, chokepoint partagé index+/browse) ; THREAT_MODEL §15.1 doc « /browse is_open_source spoofable, verrou-4 ≠ attestation crypto ». |
| **B3. PULL-3 cross-tier failover + SeedAnnounced non-converge** (2 reports) | Chaîne fallback ordonnée ticket-mort→directory→multi-provider câblée au call-site driver E (`seed_voluntary` ~`http.rs:2099-2189`) ; note d'investigation root-cause `SeedAnnounced peer_count:0` (propagation feed cross-swarm) consignée. |
| **B4. T6-OUTBOX-DIRECT** | Test 2-nœuds `GossipCmd::Outbox` (`runtime.rs:1787`) `neighbor_count>0` (pattern hijack-guard S75-A). Test manquant pur. |
| **B5. WS-3/PD-5 hoisting** | `my_endpoint_addr()` hoisté once-per-pass au replay (`runtime.rs:1655-1850`). Refacto efficience, nextest inchangé. |
| **B6. DISCRIMINATEUR-CURATOR-ANCRE** | `listCurators().entries` distingue curator-pur vs ancre-non-annoncée (`Nodes.tsx:164,347`, 0 wire) ; copy UI honnête /nodes lignes « en attente ». |
| **B7. LOOPBACK-TIERS-STALE** (2 reports) | 7 routes S74+S75 inscrites `LOOPBACK_ENDPOINTS_TRUST_TIERS.md §3` (T0 ; T1 candidats `/directory/publish` + `/seed/request`) ; phrase fausse du plan d'audit corrigée. |
| **B8. THREAT-BLOBSERVE-BEARER** | Cellule mitigation blob-serve corrigée (`THREAT_MODEL §15.1` vs `http.rs:252-255`) : route PUBLIQUE par construction ; amplification bornée subscribed-only+cap+timeout. |
| **B9. Lot test front** (FRONTEND-COVERAGE-GAP + CI-PLAYWRIGHT-NOOP + shell T6/T7) | Smoke render 5 pages 0-test (Network/Curators/Projects/OnboardingEmpty/ProjectDetail) ; ≥1 spec Playwright réel OU étape CI [10] retirée+browsers ; renderer fuzz/anchors T6/T7. Coverage ≥ 85/85/78/85. |
| **B10. BRIDGE-ALLOWLIST-DRIFT** | Allowlist Rust(10)↔TS(15) alignée OU test parité (`protocol.ts:20-44`, `sbfb-manifest/lib.rs:52-63`) ; doc `manifest.methods` déclaratif (pas d'évasion sandbox). |
| **B11. UX-ARRIVAL-PLAN-INSCRIPTION** (doc) | Surface UX-ARRIVAL (registre observed + from_subscribed + split arrival) inscrite au `sprint76_audit_plan.md` comme track marqué couvert. |

### B.3 Tests plan

1. `seed_voluntary_noop_in_duress` — `seed_voluntary` en `IdentityMode::Duress`
   N'EFFECTUE AUCUNE mutation du data root réel (assert zéro écriture).
2. `set_keep_online_noop_in_duress` — `set_keep_online` en duress = no-op +
   réponse leurre cohérente (assert zéro mutation M18).
3. `observed_capture_bound_to_publisher_pow` — la capture observed lie l'identité
   à l'identité PoW publisher (une ID forgée non liée n'est pas enregistrée).
4. `aggregator_downgrades_open_source_without_provenance` (B2) — une annonce
   gossip `is_open_source:true` sans `provenance_hash`/`repo_url` est downgradée à
   `false` à l'ingress `runtime.rs:2231`.
5. `pull_falls_back_across_tiers_when_ticket_dead` (B3) — ticket-mort → bascule
   directory → multi-provider (fallback ordonné testé bout-en-bout).
6. `outbox_gossip_has_neighbors_two_nodes` (B4) — test 2-nœuds `GossipCmd::Outbox`
   `neighbor_count>0`.
7. `endpoint_addr_hoisted_once_per_pass` (B5) — `my_endpoint_addr()` appelé une
   seule fois par passe de replay (pas par entrée).
8. `nodes_distinguish_curator_from_anchor` (Vitest, B6) — `listCurators().entries`
   distingue curator-pur vs ancre-non-annoncée ; copy /nodes honnête.
9. `blobserve_mitigation_cell_matches_impl` (B8) — la cellule THREAT_MODEL
   blob-serve reflète `http.rs:252-255` (route publique, amplification bornée).
10. `network_page_smoke_renders` (Vitest, B9) — `Network` rend sans crash.
11. `curators_page_smoke_renders` (Vitest, B9) — `Curators` rend sans crash.
12. `projects_page_smoke_renders` (Vitest, B9) — `Projects` rend sans crash.
13. `onboarding_empty_page_smoke_renders` (Vitest, B9) — `OnboardingEmpty` rend.
14. `project_detail_page_smoke_renders` (Vitest, B9) — `ProjectDetail` rend.
15. `bridge_allowlist_parity` (Vitest ou Rust, B10) — l'allowlist TS et l'allowlist
   Rust sont alignées (ou la parité déclarative est testée).

### B.4 Critère d'acceptation

```
cargo nextest run -p nexus-shell-daemon -p nexus-shell-daemon-core --locked
(cd web && npm run test:unit && npm run test:coverage)
test -f docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md   # §3 mis à jour (7 routes)
grep -c "FORMAT_VERSION" crates/nexus-core-rs/src/*.rs    # inchangé (0 bump wire)
```
Tests #1/#2 PASSENT (duress no-op zéro mutation). Test #4 PASSE (downgrade
ingress). Test #5 PASSE (failover multi-tier). Coverage front ≥ 85/85/78/85. Les
3 carries 2-reports (CARRY-3 B2, LOOPBACK-TIERS B7, PULL-3 B3) fermés. 0 bump wire.

### B.5 Commit cible

`fix(daemon+shell): Sprint 76 Phase B — duress siblings + 2-report carries + test/doc debt`
Body 9 sections : (1) `## Contexte` (phase dette réservée non convertible, lot
duress local-only miroir P1 fermé, anti-escalade G7) ; (2) `## Fichiers` ; (3)
`## Delta tests` (+~10 Rust, +~7 Vitest) ; (4) `## Verification §7.4` ; (5)
`## Scope cuts respectes (kickoff §7)` ; (6) `## G8 traceability` ; (7)
`## Pre-launch protocol` (0 bump wire, tout local/doc/test) ; (8)
`## Codex verification` ; (9) `## Carry closure / Unblock` (CARRY-3, LOOPBACK-TIERS,
PULL-3, DURESS-FRERES, T6-OUTBOX, WS-3/PD-5, DISCRIMINATEUR, THREAT-BLOBSERVE,
FRONTEND-COVERAGE, BRIDGE-ALLOWLIST, UX-ARRIVAL CLOSED ; PULL-3 débloque le
dial-set du quorum C/D).

---

## §6 Phase C — E2E cross-machine compute B-3 (palier 1) + cohorte homogène (D2 + D3 étage 1)

### C.1 Scope

Lever **B-3** : démontrer la première exécution compute sur **deux processus OS
sur deux hôtes physiques** via une acceptance LIVE scriptée (palier 1 : VPS
coordinateur/ancre ↔ PC RTX 5080 worker réel Ollama, `redundancy=1`). Le PC mint
un invite worker-scope depuis le VPS (route prod `/api/v1/invite/create`
scope:worker → ticket du doc projet), enrôle le projet, démarre `nexus-worker`
(binaire OS, pas in-process) ; un submit HTTP au VPS → le PC claim+exécute sur GPU
réel → `result:` signé revient WAN → `GET /result` au VPS rend `result_text`.
**Zéro changement de mécanique** sur le chemin compute (dispatch/pompe/result-sync/
validator/sign-verify) — le livrable B-3 est l'acceptance LIVE + sa trace SSH.

En parallèle, poser le **routing cohorte-homogène** (D3 étage 1, pré-condition du
quorum déterministe de la Phase D) : (i) **décider/durcir `model_digest`** de
`blake3(model_name)` (`runtime.rs:1082`) → hash du fichier GGUF (le champ existe
déjà `task.rs:374`) [P1 dans la phase si tranché ainsi au preflight, OU doc-note
si hors-scope] ; (ii) advertir le tuple `(model_digest, quant, runtime_family)`
dans la capability worker (`capability_store.rs`). Le validator
`validate_quorum_pre_guardrail` reste **INCHANGÉ**.

**1er critère d'acceptance falsifiable (D2 adjust)** : mesurer le délai de
réplication `result:` PC→VPS sur WAN réel. **Si > timeout du gate (150×200ms=30s),
c'est un BLOCK à diagnostiquer, PAS un timeout à rallonger** — en référence
explicite au constat S75 `SeedAnnounced peer_count:0 ~10 min` (chemin DOC distinct
du gossip de feed, hypothèse à falsifier en premier).

### C.2 Livrables

| Livrable | Description |
|---|---|
| Acceptance LIVE palier 1 (B-3) | Script SSH PC↔VPS : invite worker-scope minté, `nexus-worker` binaire démarré sur le PC, submit HTTP au VPS, claim+exécute GPU réel (Ollama, sortie réelle), `result:` signé WAN, `GET /result` rend `result_text`. Trace consignée `sprint76_verification.md`. |
| Mesure convergence `result:` WAN | 1er critère falsifiable : délai réplication `result:` PC→VPS mesuré ; >30s = BLOCK diagnostiqué (root-cause), pas timeout rallongé. |
| `model_digest` durci (ou doc-note) | `runtime.rs:1082` : si tranché P1 au preflight, `model_digest` = hash du fichier GGUF (pas `blake3(nom)`), cohérent avec son doc-comment `task.rs:374` « exact model file » ; sinon doc-note la discordance et garder le name-hash pour S76 (durcissement S77). |
| Capability tuple advertise | `capability_store.rs` + `dispatcher.rs:37-133` : le worker advertit `(model_digest, quant, runtime_family)` ; le dispatcher n'assigne les réplicas `verifiable`+redundancy>1 qu'aux workers homogènes sur ce tuple. |
| Gate anti-régression conservé | `runtime.rs:3629 e2e_network_execute_gate_real_http_no_frontier_mock` reste le gate in-process 2-nœuds (StubBackend), non touché. |

### C.3 Tests plan

1. `capability_advertises_homogeneity_tuple` (Rust) — un worker advertit
   `(model_digest, quant, runtime_family)` dans sa capability.
2. `dispatcher_routes_replicas_to_homogeneous_cohort` (Rust) — pour
   `verifiable`+redundancy>1, le dispatcher n'assigne les réplicas qu'aux workers
   au même tuple ; un worker au tuple différent n'est pas assigné.
3. `model_digest_hashes_gguf_file_or_documented` (Rust) — si durci :
   `model_digest` ≠ `blake3(model_name)` (hash du fichier GGUF) ; si doc-note :
   le test documente la discordance et garde le name-hash (assert explicite).
4. `e2e_network_execute_gate_real_http_no_frontier_mock` (Rust, **gate
   anti-régression existant**) — reste vert (chemin compute in-process inchangé).
5. Acceptance LIVE `b3_live_pc_vps_result_rendered` (checklist manuelle SSH, non
   unit) — submit VPS → claim PC GPU réel → `result_text` rendu WAN ; délai
   `result:` < 30s mesuré (sinon BLOCK).

### C.4 Critère d'acceptation

```
cargo nextest run -p nexus-coordinator-rs -p nexus-shell-daemon --locked
# + acceptance LIVE scriptée (SSH PC↔VPS, trace dans verification.md)
```
Acceptance LIVE palier 1 démontrée (le PC exécute une tâche soumise au VPS, result
signé rendu WAN). Le délai de réplication `result:` PC→VPS est **mesuré et <30s**
(si >30s : BLOCK diagnostiqué, pas timeout rallongé). Test #2 PASSE (routing
cohorte homogène). Le gate `runtime.rs:3629` reste vert.

### C.5 Commit cible

`feat(coordinator+daemon): Sprint 76 Phase C — cross-machine compute B-3 (live) + homogeneous cohort routing`
Body 9 sections : (1) `## Contexte` (D2 lève B-3 par acceptance LIVE, transport
forcé iroh 0.98 + modèle S75 prouvé, D3 étage 1 cohorte homogène, 1er critère
falsifiable convergence WAN) ; (2) `## Fichiers` ; (3) `## Delta tests` (+4 Rust +
acceptance LIVE) ; (4) `## Verification §7.4` ; (5)
`## Scope cuts respectes (kickoff §7)` (pas de scheduler push, pas de RPC synchrone,
pas de DHT custom, cross-GPU hétérogène = post-S77) ; (6) `## G8 traceability` ;
(7) `## Pre-launch protocol` (0 bump wire ; `model_digest`/`logprobs_hash`
existent déjà v1 ; tout additif) ; (8) `## Codex verification` ; (9)
`## Carry closure / Unblock` (B-3 LEVÉ ; débloque le quorum déterministe Phase D).

---

## §7 Phase D — Quorum redundancy>1 prouvé déterministe (palier 2) (D3 étage 1 suite)

### D.1 Scope

Prouver le **quorum redundancy>1 sur sorties DÉTERMINISTES** : acceptance LIVE
palier 2 (VPS + PC + Mac comme deux workers indépendants, `redundancy_factor=2`,
tâche `verifiable=true`). Les deux workers homogènes produisent le même
`result_text` (le contrat `verifiable` force temp=0 + `seed=blake3(task_id)`,
`engine/runtime.rs:1260-1285`), le validator `validate_quorum_pre_guardrail`
(`validator.rs:219-338`, **INCHANGÉ**) voit deux `result_text` byte-identiques →
consensus accepté. Le test hermétique cross-process se fait via `StubBackend`
(déterministe par construction), l'acceptance LIVE avec le même modèle+quant
Ollama sur deux hôtes homogènes.

**Résultat attendu honnête écrit comme critère (D3 adjust, anti faux-vert T1)** :
l'exact-match tient en cohorte homogène, **diverge** sur GPU hétérogène (que le
validator rejette correctement — outlier logging déjà là). L'acceptance écrit
explicitement les deux issues (homogène → consensus, hétérogène → divergence
rejetée comme attendu) pour ne pas masquer le risque cross-hardware (Ingonyama :
même GGUF diverge cross-GPU).

**Étage 2 = design note seulement (NON codé S76)** : `logprobs_hash` (`task.rs:383`,
« layer 3 ») est le slot TOPLOC pour une vérification sémantique tolérante future ;
lié au backend `LlamaCppBackend` feature-gated `llm_llama_cpp` (Ollama n'expose pas
les hidden states). Aucun bump wire (champ déjà v1).

### D.2 Livrables

| Livrable | Description |
|---|---|
| Test cross-process redundancy>1 hermétique | `dispatch_loop.rs:155-303` (base, `multi_thread` MANDATORY P2-A-1) : deux workers StubBackend exécutent la même tâche `verifiable` redundancy=2/3 → `best_count > threshold`, `result_text` byte-identique cross-process. |
| Acceptance LIVE palier 2 (quorum) | VPS+PC+Mac, `redundancy_factor=2`, `verifiable=true`, même quant Ollama → deux `result_text` identiques → consensus accepté. Trace SSH `sprint76_verification.md`. |
| Résultat hétérogène-diverge documenté | L'acceptance écrit le résultat attendu : homogène → consensus ; hétérogène → divergence rejetée (outlier logging). Anti faux-vert. |
| Design note TOPLOC (étage 2) | `logprobs_hash` slot étage 2 documenté (PATTERNS rust + THREAT_MODEL row) : commitment top-k hidden state, feature `llm_llama_cpp`, post-S77. Aucun code. |
| Validator INCHANGÉ (verrou) | `validate_quorum_pre_guardrail` non touché — vérifié par grep diff. |

### D.3 Tests plan

1. `quorum_redundancy_two_stubworkers_byte_identical` (Rust) — deux workers
   StubBackend, redundancy=2, `verifiable=true` → `result_text` byte-identique →
   `best_count > threshold` → consensus accepté.
2. `quorum_diverging_outputs_rejected` (Rust) — deux `result_text` divergents
   (simulant cross-GPU hétérogène) → divergence rejetée, outlier loggué (résultat
   attendu honnête).
3. `verifiable_seed_is_cross_worker_stable` (Rust) — `seed=blake3(task_id)`
   identique pour les deux réplicas (prémisse « même seed » tenue).
4. `validator_quorum_unchanged` (Rust ou grep) — `validate_quorum_pre_guardrail`
   non modifié (signature + logique exact-match préservées).
5. Acceptance LIVE `quorum_live_vps_pc_mac_consensus` (checklist manuelle SSH) —
   redundancy=2 homogène → consensus LIVE ; hétérogène → divergence rejetée.

### D.4 Critère d'acceptation

```
cargo nextest run -p nexus-coordinator-rs -p nexus-shell-daemon --locked
# + acceptance LIVE quorum (SSH VPS+PC+Mac, trace dans verification.md)
git diff --stat crates/nexus-coordinator-rs/src/validator.rs   # 0 ligne quorum touchée
```
Test #1 PASSE (quorum=2 byte-identique cross-process). Test #2 PASSE (divergence
rejetée). Acceptance LIVE quorum=2 homogène → consensus démontrée ; résultat
hétérogène-diverge écrit (anti faux-vert). `validate_quorum_pre_guardrail`
INCHANGÉ (diff vide).

### D.5 Commit cible

`feat(coordinator+daemon): Sprint 76 Phase D — redundancy>1 deterministic quorum proven (live + cross-process)`
Body 9 sections : (1) `## Contexte` (D3 étage 1 suite, quorum prouvé déterministe
sur cohorte homogène, validator inchangé, résultat hétérogène-diverge honnête) ;
(2) `## Fichiers` ; (3) `## Delta tests` (+4 Rust + acceptance LIVE) ; (4)
`## Verification §7.4` ; (5) `## Scope cuts respectes (kickoff §7)` (TOPLOC étage
2 = design note non codé, cross-GPU hétérogène = post-S77, déterminisme
cross-hardware non garanti) ; (6) `## G8 traceability` ; (7)
`## Pre-launch protocol` (0 bump wire ; `logprobs_hash` slot déjà v1) ; (8)
`## Codex verification` ; (9) `## Carry closure / Unblock` (quorum déterministe
prouvé ; débloque le dashboard contributeur Phase E sur lignes quorum-accept).

---

## §8 Phase E — Dashboard contributeur (D4)

### E.1 Scope

Livrer le **dashboard contributeur** : une vue d'agrégation keyée sur
`worker_node_id` sur le ledger kudos existant (per-task natif, EMA `alpha=0.97`).
Le « per-project » actuel (`get_project_kudos`, `kudos_ledger.rs:134-163`) n'est
qu'une vue d'agrégation ; le dashboard contributeur = une **deuxième vue** sur les
mêmes lignes, réutilisant exactement `effective_score()` (même décroissance EMA).
Kudos **non-monétaire, zéro token crypto** (décision gelée). Trois métriques
honnêtes : kudos effectifs (EMA), tâches servies (= lignes validées par quorum),
**GPU-heures données LOCALES** (lues depuis `usage.json` du nœud — honnête
« heures que cette machine a données », non-attestées ; les GPU-heures ne sont PAS
dans le ledger).

**Décision anti-gaming `tokens_generated` (D4-Q, tranchée au preflight)** :
`amount = log_utility(tokens_generated)` et `tokens_generated` est self-déclaré
dans le payload signé (`task.rs:363`) mais **hors quorum** (le validator ne compare
que `result_text`) → un worker malhonnête peut gonfler `tokens_generated` sans
casser le quorum. Option (a) **durcir** : `amount =
log_utility(median(tokens_generated))` du groupe d'accord (BOINC discard-high/low)
+ sanity-bound `tokens ≤ f(generation_time_ms)` au `credit()` (modifie la signature
de `credit()` pour recevoir le groupe). Option (b) **documenter le trou en P2** (la
`log_utility` compresse déjà l'incitatif <10×). La cohérence avec le groupe
d'accord quorum prouvé en D rend (a) faisable proprement ; le défaut est arbitré au
preflight Phase E.

### E.2 Livrables

| Livrable | Description |
|---|---|
| `get_contributor_kudos` | `kudos_ledger.rs:124-163` : miroir exact de `get_project_kudos`, même `effective_score` EMA `alpha=0.97`. Agrège `effective_total`, `tasks_served` (COUNT lignes), `raw_total`, `per_project` breakdown. |
| Query + index SQLite | `db.rs:1025-1068` : `SELECT … WHERE worker_node_id = ?1` + **index SQLite sur `worker_node_id`** (aujourd'hui les requêtes scope `project_id`). |
| Route `contributor_dashboard` | `kudos_api.rs:44-144` : handler `contributor_dashboard(Path(node_id))` miroir de `leaderboard()`. |
| Décision anti-gaming `tokens_generated` | `validator_loop.rs:108-120` + `http.rs:3342-3351` : durcir `amount = log_utility(median(tokens_generated))` du groupe + sanity-bound, OU documenter P2 (selon preflight). |
| GPU-heures locales | `consent.rs:229-328` `usage.json` (`hours_today`) = source des GPU-heures **locales** pour le panneau (jamais répliquées, libellées « heures données par cette machine »). |
| Front page contributeur | Réutilise `Network.tsx` (GpuCard + ProjectsServedCard) + nouvelle route API ; 3 métriques (kudos effectifs / tâches servies / GPU-heures locales). Strings FR. |

### E.3 Tests plan

1. `get_contributor_kudos_aggregates_ema` (Rust) — la vue contributeur applique
   le même EMA `alpha=0.97` que `get_project_kudos` (kudos effectifs cohérents
   entre les deux vues).
2. `contributor_kudos_counts_tasks_served` (Rust) — `tasks_served` = nombre de
   lignes ledger validées (pas tentées).
3. `contributor_kudos_query_uses_worker_index` (Rust) — la query
   `WHERE worker_node_id=?1` utilise l'index SQLite ajouté.
4. `tokens_generated_hardened_or_documented` (Rust) — si durci :
   `amount = log_utility(median)` du groupe d'accord + sanity-bound rejette un
   `tokens_generated` aberrant ; si P2 : test documentant la compression <10× de
   `log_utility`.
5. `contributor_dashboard_route_mirrors_leaderboard` (Rust) — la route
   `contributor_dashboard(node_id)` renvoie l'agrégat contributeur.
6. `contributor_page_renders_three_metrics` (Vitest) — la page rend kudos
   effectifs + tâches servies + GPU-heures locales (libellées « cette machine »).

### E.4 Critère d'acceptation

```
cargo nextest run -p nexus-coordinator-rs -p nexus-shell-daemon --locked
(cd web && npm run test:unit && npx tsc --noEmit -p tsconfig.app.json)
bash web/scripts/scan-en-strings.sh
```
Test #1 PASSE (EMA cohérent entre vues projet et contributeur). La route +
la page rendent les 3 métriques honnêtes. La décision anti-gaming
`tokens_generated` est tranchée (durcie OU P2 documenté). Index `worker_node_id`
présent. Kudos non-monétaire (0 token crypto).

### E.5 Commit cible

`feat(coordinator+shell): Sprint 76 Phase E — contributor dashboard (per-task kudos aggregation)`
Body 9 sections : (1) `## Contexte` (D4, vue d'agrégation contributeur sur ledger
existant, EMA cohérent, GPU-heures locales honnêtes, anti-gaming tranché) ; (2)
`## Fichiers` ; (3) `## Delta tests` (+5 Rust +1 Vitest) ; (4)
`## Verification §7.4` ; (5) `## Scope cuts respectes (kickoff §7)` (pas de
conversion monétaire Gridcoin, pas d'EigenTrust ranking global, pas de champ wire
signé GPU-heures, reconnaissance contributeur publique = post-launch) ; (6)
`## G8 traceability` ; (7) `## Pre-launch protocol` (0 bump wire ; per-task natif ;
GPU-heures via `usage.json` local) ; (8) `## Codex verification` ; (9)
`## Carry closure / Unblock` (dashboard contributeur livré).

---

## §9 Phase F — Quantization 4-bit documentée (D5)

### F.1 Scope

Livrer de la **DOCUMENTATION**, pas un nouveau runtime de quantification (le
runtime quantifié existe déjà : `LlamaCppBackend` charge n'importe quel GGUF
pré-quantifié via `load_from_file`, offload GPU via `n_gpu_layers` ; le format
4-bit est baked dans le `.gguf`). Le livrable = (1) un doc opérateur
`docs/operators/QUANTIZATION.md` recommandant le format par taille de carte
(Q4_K_M par défaut, IQ4_XS quand on serre la VRAM, Q4_K_S si l'arXiv prime) ; (2)
la table d'empreintes VRAM mesurées ; (3) le branchement honnête des caps VRAM
existants (`gpu/mod.rs:147-151` + `consent.rs:417-432`, design note — gate
admission par budget). Lien depuis le panneau D1 (« ta carte 16GB → modèles ≤14B
Q4_K_M »).

**Cadrage produit (arbitrage PO Checkpoint §11) — « personne n'a 2 GPU »** : un
contributeur a **UNE** carte 16GB, pas deux ; le mono-machine 2-GPU n'est pas un
vrai déploiement. **Cible honnête single-GPU = ≤14B, modèle ENTIER** (Qwen2.5-14B
Q4_K_M ~8.5 GB tient 1×16GB). Les gros modèles (32B/70B) ne tiennent sur AUCUNE
carte 16GB (70B Q4_K_M = 42.5 GB, IQ4_XS = 37.9 GB, même Q2_K = 26.4 GB) ; le
chemin vers ces tailles n'est PAS « ajouter une 2e carte » mais **éclater le modèle
sur 2+ machines à 1 GPU chacune = sharding cross-machine = S77**. L'offload CPU
mono-machine (2-5 tok/s, batch/async) reste un palliatif documenté, pas la voie
principale. **Pré-condition quorum (lien D3)** : deux workers DOIVENT utiliser le
MÊME GGUF (même quant, même build) pour un exact-match — la doc l'impose.
**Doc-only** : le tensor-split mono-machine multi-GPU (`with_split_mode`+
`with_devices`) est **rejeté fermement** (PO : pas de cible, personne n'a 2 GPU) ;
le multi-GPU réaliste = cross-machine = S77.

### F.2 Livrables

| Livrable | Description |
|---|---|
| `docs/operators/QUANTIZATION.md` | Doc opérateur : reco format par taille de carte ; table d'empreintes VRAM (7B/8B Q4_K_M ~4.6 GB 1×16GB modèle entier ; **14B Q4_K_M ~8.5 GB cible honnête single-GPU 1×16GB** ; 32B Q4_K_M ~22 GB carte 24GB hors-cible ; 70B Q2_K ~26.4 GB / IQ4_XS ~37.9 GB / Q4_K_M ~42.5 GB = **ne tient sur AUCUNE carte 16GB → sharding cross-machine 2+ machines = S77** ; offload CPU mono-machine 2-5 tok/s = palliatif) ; pré-condition quorum même-GGUF ; cible single-GPU ≤14B + gros modèles = sharding cross-machine S77 explicite (mono-machine 2-GPU enterré). |
| Design note caps VRAM | Section doc : `vram_budget_remaining_bytes` (`gpu/mod.rs:147-151`) + cap VRAM (`consent.rs:417-432`) réutilisables tels quels pour gater l'admission d'un modèle dont l'empreinte dépasse le budget (cap actuel lit l'estimé déclaré du Task, pas la taille GGUF réelle — noté design note, hors scope bloquant). |
| Lien depuis panneau D1 | Le panneau « offrir ma puissance » (Phase A) pointe vers `QUANTIZATION.md` (« ta carte 16GB → modèles ≤14B Q4_K_M »). |

### F.3 Tests plan

Phase doc-only (pas de code runtime nouveau). Vérifications :
1. `quantization_doc_present` (test -f) — `docs/operators/QUANTIZATION.md` existe.
2. `quantization_doc_has_footprint_table` (grep) — la doc contient la table
   d'empreintes (Q4_K_M / IQ4_XS / Q2_K) et la cible ≤14B.
3. `quantization_doc_states_70b_is_s77` (grep) — la doc indique explicitement
   « gros modèles (70B) = sharding cross-machine 2+ machines × 1 GPU = S77 ; ne
   tient sur aucune carte 16GB ; mono-machine 2-GPU n'est pas une cible ».
4. `quantization_doc_states_quorum_precondition` (grep) — la doc impose le
   même-GGUF comme pré-condition du quorum redundancy>1.
5. `llama_cpp_unchanged_doc_only` (grep) — `llm/llama_cpp.rs:143-164` câble
   toujours UNIQUEMENT `with_n_gpu_layers` (pas de `split_mode`/`devices` ajouté).

### F.4 Critère d'acceptation

```
test -f docs/operators/QUANTIZATION.md
grep -q "≤14B\|<=14B\|14B" docs/operators/QUANTIZATION.md
grep -q "S77" docs/operators/QUANTIZATION.md
grep -n "with_split_mode\|with_devices" crates/nexus-worker-core/src/llm/llama_cpp.rs   # absent (S77)
```
La doc existe, contient la table d'empreintes, la cible ≤14B honnête, le renvoi
70B=S77, et la pré-condition quorum même-GGUF. Le backend `llama_cpp.rs` reste
inchangé (doc-only ; tensor-split = S77).

### F.5 Commit cible

`docs(operators): Sprint 76 Phase F — 4-bit quantization documentation (GGUF, single-GPU ≤14B, large models = cross-machine sharding S77)`
Body 9 sections : (1) `## Contexte` (D5 doc-only, runtime quant déjà présent
inchangé, cible single-GPU honnête ≤14B modèle entier, gros modèles = sharding
cross-machine S77 [arbitrage PO « personne n'a 2 GPU »], pré-condition quorum) ;
(2) `## Fichiers` ; (3) `## Delta tests` (0 test runtime ; vérifs grep/test -f) ;
(4) `## Verification §7.4` ; (5) `## Scope cuts respectes (kickoff §7)` (tensor-split
mono-machine rejeté [pas de cible], AWQ/GPTQ/EXL2/bitsandbytes rejetés, Q2_K-défaut
rejeté, câblage VRAM-live S77) ; (6) `## G8 traceability` ; (7) `## Pre-launch protocol`
(0 code wire ; doc-only) ; (8) `## Codex verification` ; (9)
`## Carry closure / Unblock` (quant documentée ; lien panneau D1).

---

## §10 Phase G — Wrap-up + acceptance consolidée + clôture Arc 3.5 6/6

### G.1 Scope

Clôturer le sprint et **l'Arc 3.5 (6/6)**. Produire `sprint76_verification.md`
(fail-fast §Observed rempli, Win + **Docker Linux canonique = gate AVANT push**,
`feedback_wsl_before_push`) + `sprint77_audit_plan.md` (plan Phase 0 S77 :
duress résolu, carries reconduits, surfaces compute/quorum/dashboard, candidats P1
S77). Consolider l'**acceptance LIVE cross-machine** (B-3 palier 1 + quorum palier
2, traces SSH PC/VPS/Mac dans verification.md). Mettre à jour `THREAT_MODEL.md`
(rows surface compute cross-machine + duress frères fermés), `PATTERNS.md` rust
(compute/quorum/TOPLOC étage 2) + shell, `SPRINT_LOG.md` row S76, `CLAUDE.md`
état, `roadmap_v5` (Arc 3.5 6/6 clos, S77 sharding ouvert). Carries reconduits S77
(SYBIL-SEEDER-TAIL avec exemption « dépendance sharding » nommée,
REVISION-HOME-DURABILITY, KNOWN-ENTRY-OVERCOUNT, seeder catalog_len:0,
RE-DRIVE-ON-INGEST, T-NN+3).

### G.2 Livrables

| Livrable | Description |
|---|---|
| `sprint76_verification.md` | Self-report fail-fast (colonne Observed remplie) + traces acceptance LIVE B-3 + quorum + §5 carry-over for memory. |
| `sprint77_audit_plan.md` | Plan Phase 0 S77 : tracks (duress résolu confirmé, carries reconduits, surfaces compute/quorum/dashboard/quant, candidats P1 S77). |
| `THREAT_MODEL.md` rows | Surface compute cross-machine (claim pull, result signé, quorum cohorte homogène) + duress frères fermés (B1). |
| `PATTERNS.md` rust + shell | Patterns compute/quorum/cohorte homogène + TOPLOC étage 2 design note + tech debt mise à jour. |
| `SPRINT_LOG.md` + `CLAUDE.md` + `roadmap_v5` | Row S76 ; état CLAUDE.md ; Arc 3.5 6/6 clos + S77 sharding ouvert (amendement). |

### G.3 Tests plan

Phase wrap-up : pas de code feature nouveau. Vérifications :
1. `verification_md_present` (test -f) — `sprint76_verification.md` existe avec
   Observed rempli.
2. `audit_plan_s77_present` (test -f) — `sprint77_audit_plan.md` existe.
3. Fail-fast dual-platform complet re-joué (Win nextest + Docker canonique + web)
   — toutes rows vertes (gate AVANT push).
4. Acceptance LIVE consolidée (B-3 palier 1 + quorum palier 2) tracée dans
   verification.md.

### G.4 Critère d'acceptation

```
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release
# Docker Linux canonique (image sbfb-ci rust:1.94) nextest workspace — gate AVANT push
(cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && npm run test:unit && npm run test:coverage && npm run build && npm run size)
bash web/scripts/scan-en-strings.sh
test -f .planning/active/sprint76_verification.md
test -f .planning/active/sprint77_audit_plan.md
test -f docs/operators/QUANTIZATION.md
```
Fail-fast dual-platform complet vert (Win + Docker canonique + web). Acceptance
LIVE B-3 + quorum tracée. Artefacts écrits. Arc 3.5 6/6 clos dans roadmap_v5.

### G.5 Commit cible

`feat(daemon): Sprint 76 Phase G — wrap-up + cross-machine compute acceptance + Arc 3.5 close`
Body 9 sections : (1) `## Contexte` (clôture Arc 3.5 6/6, acceptance LIVE
consolidée, carries reconduits S77) ; (2) `## Fichiers` ; (3) `## Delta tests`
(hygiène/doc, total cumulé) ; (4) `## Verification §7.4` (fail-fast dual-platform
complet) ; (5) `## Scope cuts respectes (kickoff §7)` (exhaustif) ; (6)
`## G8 traceability` ; (7) `## Pre-launch protocol` (0 bump wire sur tout le
sprint) ; (8) `## Codex verification` ; (9) `## Carry closure / Unblock` (carries
S77 reconduits avec compteurs incrémentés ; LT-2 ARMÉ hors-sprint, LT-5 supersedé,
LT-7 dormant décision actée).

---

## §11 Phase H — Pont compute iframe câblé + acceptance compute LOCAL (post-audit)

**Ajout post-audit** (hors plan A-G initial), demandé par le PO après la clôture
S76 : prouver EN VRAI le compute S76 **en LOCAL** via un projet SBFB dédié
(« Compute Tester ») qui soumet une tâche IA par le pont et affiche le résultat.
Plan détaillé : `sprint76_phase_h_compute_tester_plan.md`. Préflight :
`sprint76_phase_h_preflight.md` (verdict **PLAN-ADAPT**).

STEP 0 (trace code) a montré que le chemin compute depuis une iframe sandboxée
n'était câblé sur **aucun** segment (route submit app-scoped morte depuis S50,
payload mismatch, aucun canal de retour résultat). Adaptation livrée, additive,
0 bump wire :
- daemon : route read-only `GET /api/daemon/project-info` → `{project_doc_id}`
  (le worker local on-demand ne claime que `project_id == project_doc.id()`) ;
- parité allowlist cross-langage : `task_result` ajouté (Rust `sbfb-manifest` +
  TS `BridgeMethodSchema`, 16 méthodes) ;
- bridge : `task_submit` re-pointé vers le daemon-level prouvé (host injecte
  `project_id`) + `task_result` poll (404=pending) ; SDK `getTaskResult` (5 bundles
  + 4 templates Factory) ; app `examples/compute-tester/`.

**Décision PO** : poll (option A) maintenant ; le push live (SSE daemon adossé
iroh-docs `subscribe`, option B) est routé **S77** avec la convergence WAN.

Acceptance LIVE LOCAL = **PASS** (12 s, llama3.1:8b, `result_text` réel, app
déployée + browsable + render path blob-serve ; manifeste `task_result` validé
live) — trace `sprint76_verification.md §5.2`.

Commit (1 commit atomique) :
`feat(bridge+daemon): Sprint 76 Phase H — wire iframe compute path + LOCAL acceptance`
Body 9 sections. Review `sprint76_phase_h_review.md` PASS ; Codex
`sprint76_phase_h_codex_review.md` PARTIAL-not-reject (2 items corrigés, 3
documentés/carry). Carry neuf **TABVIEW-APP-SUBMIT-DEAD** S77.

---

## Delta tests estimé

| Phase | Rust | Vitest | Détail |
|---|---|---|---|
| A | +5 | +2 | snapshot additif (#1/#2), enrôlement worker co-localisé (#3/#4), route préfixe (#5) ; front jauge caps + CTA intention (#6/#7) |
| B | +~10 | +~7 | duress no-op (#1/#2/#3), aggregator downgrade (#4), failover multi-tier (#5), outbox 2-nœuds (#6), hoisting (#7), blob-serve doc (#9) ; front 5 pages smoke (#10-#14) + curator/ancre (#8) + bridge parité (#15) |
| C | +4 | 0 | capability tuple (#1), routing cohorte homogène (#2), model_digest durci/doc (#3), gate anti-régression (#4 existant) ; acceptance LIVE B-3 (#5, non unit) |
| D | +4 | 0 | quorum 2-stubworkers byte-identique (#1), divergence rejetée (#2), seed cross-worker-stable (#3), validator inchangé (#4) ; acceptance LIVE quorum (#5, non unit) |
| E | +5 | +1 | EMA agrégation contributeur (#1), tasks_served (#2), index worker (#3), anti-gaming durci/doc (#4), route dashboard (#5) ; front 3 métriques (#6) |
| F | 0 | 0 | doc-only ; vérifs grep/test -f (pas de test runtime nouveau) |
| G | 0 | 0 | wrap-up ; fail-fast re-joué (pas de test feature nouveau) |
| **Total** | **+~28** | **+~10** | |
| **Sortie estimée** | **~1789 (Win) / ~1793 (Docker)** | **~389** | **~2206 (Rust + Vitest 389 + factory 7 + size 6)** |

---

## Fail-fast checklist (Observed rempli en verification.md)

| # | Check | Commande | Critère | Observed |
|---|---|---|---|---|
| 1 | fmt | `cargo fmt --all --check` | exit 0 | |
| 2 | clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warn | |
| 3 | nextest workspace (Win) | `cargo nextest run --workspace --locked` | 0 fail | |
| 4 | doctests | `cargo test --workspace --locked --doc` | 0 fail | |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | OK | |
| 6 | Docker Linux canonique | `rust:1.94` (`sbfb-ci`) nextest workspace | 0 fail | |
| 7 | web tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 | |
| 8 | web lint | `(cd web && npm run lint)` | 0 err | |
| 9 | web Vitest | `(cd web && npm run test:unit)` | pass | |
| 10 | web coverage | `(cd web && npm run test:coverage)` | ≥ 85/85/78/85 | |
| 11 | web build+size | `(cd web && npm run build && npm run size)` | 6/6 | |
| 12 | scan FR | `bash web/scripts/scan-en-strings.sh` | clean | |
| 13 | A — snapshot additif 0-bump | test `consent_snapshot_serializes_additively` + grep `SCHEMA_VERSION: u32 = 1` | PASS + unchanged | |
| 14 | A — enrôlement worker public | test `colocated_worker_honors_user_consent_when_public` | PASS | |
| 15 | A — least-priv quand OFF | test `colocated_worker_least_privilege_when_off` | PASS | |
| 16 | A — route consent atteint daemon | test `consent_route_reaches_daemon_prefix` | PASS | |
| 17 | B — duress seed_voluntary no-op | test `seed_voluntary_noop_in_duress` | PASS (zéro mutation) | |
| 18 | B — duress set_keep_online no-op | test `set_keep_online_noop_in_duress` | PASS (zéro mutation) | |
| 19 | B — aggregator downgrade ingress | test `aggregator_downgrades_open_source_without_provenance` | PASS | |
| 20 | B — failover multi-tier | test `pull_falls_back_across_tiers_when_ticket_dead` | PASS | |
| 21 | B — outbox 2-nœuds | test `outbox_gossip_has_neighbors_two_nodes` | PASS | |
| 22 | B — 5 pages front smoke | tests `*_page_smoke_renders` (Network/Curators/Projects/OnboardingEmpty/ProjectDetail) | 5 PASS | |
| 23 | B — LOOPBACK §3 à jour | `test -f docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` + 7 routes S74+S75 inscrites | PASS | |
| 24 | C — routing cohorte homogène | test `dispatcher_routes_replicas_to_homogeneous_cohort` | PASS | |
| 25 | C — gate compute anti-régression | test `e2e_network_execute_gate_real_http_no_frontier_mock` | PASS | |
| 26 | C — acceptance LIVE B-3 + convergence WAN <30s | trace SSH PC↔VPS (verification.md) | démontré, délai mesuré | |
| 27 | D — quorum 2 byte-identique | test `quorum_redundancy_two_stubworkers_byte_identical` | PASS | |
| 28 | D — divergence rejetée (anti faux-vert) | test `quorum_diverging_outputs_rejected` | PASS | |
| 29 | D — validator inchangé | `git diff --stat crates/nexus-coordinator-rs/src/validator.rs` quorum | 0 ligne quorum | |
| 30 | D — acceptance LIVE quorum | trace SSH VPS+PC+Mac (verification.md) | consensus + hétérogène-diverge | |
| 31 | E — agrégation contributeur EMA | test `get_contributor_kudos_aggregates_ema` | PASS | |
| 32 | E — route dashboard | test `contributor_dashboard_route_mirrors_leaderboard` | PASS | |
| 33 | E — page 3 métriques | test `contributor_page_renders_three_metrics` | PASS | |
| 34 | F — doc quantization présente | `test -f docs/operators/QUANTIZATION.md` + table + ≤14B + S77 | PASS | |
| 35 | F — backend doc-only inchangé | grep `with_split_mode`/`with_devices` absent de `llama_cpp.rs` | unchanged | |
| 36 | 0 bump wire | grep `*_FORMAT_VERSION`/`*_ANNOUNCEMENT_VERSION` inchangés | PASS | |
| 37 | verification.md écrit | `test -f .planning/active/sprint76_verification.md` | PASS | |
| 38 | audit_plan S77 écrit | `test -f .planning/active/sprint77_audit_plan.md` | PASS | |

---

## Scope cuts (reprise exhaustive kickoff §7)

| # | Item | Sprint cible | Rationale (factuel) |
|---|---|---|---|
| 1 | **Sharding pipeline** (modèle 70B éclaté sur 2+ machines × 1 GPU) | S77 | Feature distincte (directive PO `feedback_ultra_complete_sprints`), pas un defer du GPU. 70B (42.5 GB) ne tient sur AUCUNE carte 16GB ; **arbitrage PO « personne n'a 2 GPU »** → seul chemin = sharding **cross-machine** (pas mono-machine 2-GPU). S76 prouve d'abord le task-routing cross-machine. |
| 2 | **Tensor-split mono-machine multi-GPU** (`with_split_mode`+`with_devices`) | rejeté (renvoie #1 S77) | **Arbitrage PO Checkpoint §11 : « personne n'a 2 GPU »** — le mono-machine 2-GPU n'est pas un vrai déploiement, rien à câbler. Multi-GPU réaliste = cross-machine (2 machines × 1 GPU) = sharding pipeline (#1, S77). NON ré-évaluable. |
| 3 | **Câblage VRAM-live à l'admission** (cap vs `gpu.snapshot()` réel) | S77 | Le cap vérifie l'`estimated_*` déclaré du Task (`engine/runtime.rs:929-935`, zéro=inerte). Net-new hors scope strict B-3 ; jauge heures (mesurée) suffit pour le panneau honnête. |
| 4 | **Durcir `tokens_generated` hors-quorum** (median du groupe d'accord) | S76-E ou P2 | Décision PO au preflight E : durcir (modifie signature `credit()`) OU documenter P2 (la `log_utility` compresse déjà <10×). Pas un défaut bloquant. |
| 5 | **TOPLOC étage 2 implémenté** (commitment top-k hidden state) | post-S77 | Ollama n'expose pas les hidden states ; lié `LlamaCppBackend` C-API feature-gated. Design note + slot `logprobs_hash` posé S76, implémentation future. |
| 6 | **Quorum cross-GPU hétérogène** (exact-match sur hardware différent) | post-S77 (TOPLOC) | Factuellement impossible en stock (Ingonyama : même GGUF échoue) sans réécriture kernels. S76 prouve la cohorte homogène ; l'hétérogène attend TOPLOC. |
| 7 | **`execute_build` câblé (LT-7)** | S77 (ré-éval C) | S76 déjà chargé (B-3 + quorum + dashboard + panneau) ; décision dormant-jusqu'à-S77 sauf trivialité au preflight. |
| 8 | **Reconnaissance contributeur publique** (modèle Petals `--public_name`) | post-launch | Opt-in non-MVP ; le dashboard contributeur D4 livre le cœur, l'attribution publique réseau-wide est cosmétique. |
| 9 | **Self-test/benchmark d'enrôlement** (modèle vast.ai « verified ») | jamais (rejeté) | Réseau non-monétaire (pas de SLA) ; friction contraire au budget pair frais < 1 min ; qualité gérée a posteriori par quorum/quarantine. |
| 10 | **Scheduler horaire/idle BOINC `global_prefs`** (day-of-week, idle-detect) | post-launch | Détection idle OS-spécifique (X11/Win32/Quartz) = surface multi-plateforme lourde ; le moteur niveaux + caps couvre le besoin volontaire MVP. |
| 11 | **AWQ/GPTQ/EXL2/bitsandbytes** (runtimes quant alternatifs) | jamais (rejeté) | Tous GPU-only Python hors-stack Rust in-process ; changer de format = changer de runtime = casse `LlmBackend`. SBFB = GGUF/llama.cpp. |
| 12 | **GPU-heures attestées réseau-wide** (champ wire signé) | post-launch | Self-déclaré = gameable (classe `tokens_generated`) ; ouvre surface wire pré-launch ; `usage.json` local honnête suffit pour le panneau. |
| 13 | **Upgrade iroh 1.0** (1.0.0-rc.1 disponible 2026-05-27) | Gate-1/PO | Décision gelée iroh 0.98 pinné ; upgrade = décision PO/Gate-1, pas un sprint feature. |
| 14 | **Bump `llama-cpp-2` 0.1.143→0.1.146/147** | opportun (preflight) | Hygiène + `fit_params` multi-carte ; CVE 2026 non applicables (in-process + llguidance) ; bump optionnel non bloquant, à faire si une phase touche le backend. |

---

## Risks (reprise kickoff §9)

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | **Convergence `result:` cross-machine WAN échoue** (constat S75 `SeedAnnounced peer_count:0 ~10 min` ; chemin DOC ≠ gossip feed mais non prouvé) | Medium | High | 1er critère falsifiable Phase C (D2 adjust) : mesurer délai PC→VPS ; >timeout=BLOCK à diagnostiquer, pas timeout rallongé. Chemin DOC est le S75-prouvé pour apps. |
| R2 | **Déterminisme cross-hardware casse l'exact-match** (Ingonyama : même GGUF diverge cross-GPU) | High | Medium | D3 cohorte homogène (même digest+quant+runtime) ; résultat hétérogène-diverge écrit comme attendu (anti faux-vert T1) ; StubBackend hermétique pour le test cross-process. |
| R3 | **Bug wiring `/api/v1` vs `/consent/set`** rend le panneau inerte en prod packagée | Medium | High | Pré-requis bloquant Phase A (D1 adjust) : vérifier `vite.config.ts` + réconcilier ; critère POST consent front-packagé écrit `consent.json` ; fix(sprint76) si trou prod. |
| R4 | **Trou anti-gaming `tokens_generated`** (self-déclaré hors-quorum → kudos gonflés) | Medium | Medium | Décision PO D4-Q (preflight E) : durcir `amount=log_utility(median)` du groupe d'accord + sanity-bound, OU P2 documenté (log_utility compresse <10×). |
| R5 | **Phase B dette débordée** (lot duress + 3 anti-escalade + tests) sacrifie une fermeture | Medium | Medium | Priorité B1/B2/B7 (les 2-reports + duress restant) ; B9 (tests) non sacrifiable (Règle 1 : phase dette inclut tests manquants, non convertible). |
| R6 | **Escalade G7** : CARRY-3 / LOOPBACK-TIERS / PULL-3 passent 3/3 si non traités | Low (si B exécutée) | Medium | Les 3 sont des livrables Phase B avec exit condition binaire ; SYBIL-SEEDER-TAIL reconduit avec exemption « dépendance sharding » nommée. |
| R7 | **`model_digest` durci (nom→GGUF) casse des tests existants** (couche 2 vérif) | Low | Medium | D3 adjust : durcir EST P1 OU doc-note ; si durci, miroir des tests existants + capability advert testée ; sinon doc la discordance et garder le name-hash pour S76. |

---

## Checkpoint de clôture

Sprint fermé quand **TOUTES** ces conditions binaires sont vraies :

1. **38/38 fail-fast verts** (dont Docker Linux canonique row #6 + acceptance LIVE
   B-3 row #26 + acceptance LIVE quorum row #30, gate AVANT push).
2. **7 commits feat/fix/docs** (A `feat(daemon+shell)`, B `fix(daemon+shell)`,
   C `feat(coordinator+daemon)`, D `feat(coordinator+daemon)`,
   E `feat(coordinator+shell)`, F `docs(operators)`, G `feat(daemon)`) — chacun
   avec body 9 sections + delta tests cumulé + scope cuts exhaustifs.
3. **`sprint76_verification.md` écrit** (colonne Observed remplie, traces
   acceptance LIVE B-3 + quorum, §5 carry-over for memory).
4. **`sprint77_audit_plan.md` écrit** (tracks Phase 0 S77).
5. **`docs/operators/QUANTIZATION.md` écrit** (table empreintes + cible ≤14B +
   70B=S77 + pré-condition quorum même-GGUF).
6. **PATTERNS rust + shell à jour** (compute/quorum/cohorte homogène + TOPLOC
   étage 2 design note + tech debt).
7. **THREAT_MODEL à jour** (rows surface compute cross-machine + duress frères
   fermés).
8. **memory `nexus_grid_pivot.md` tip + compteurs à jour** ; index MEMORY.md
   cohérent.
9. **SPRINT_LOG.md row S76 ajoutée** ; `CLAUDE.md` état ; `roadmap_v5` Arc 3.5 6/6
   clos + S77 sharding ouvert.
10. **0 bump wire sur tout le sprint** (`*_FORMAT_VERSION`/`*_ANNOUNCEMENT_VERSION`
    à 1 ; `SCHEMA_VERSION` à 1) ; **0 delta dépendance non justifié**.
11. **3 carries 2-reports fermés** (CARRY-3 B2, LOOPBACK-TIERS B7, PULL-3 B3) ;
    SYBIL-SEEDER-TAIL reconduit S77 avec exemption nommée.
12. **Arc 3.5 Factory Complete Vision 6/6 clos** ; S77 (sharding pipeline) ouvert
    comme feature distincte.
