# Sprint 31 — Audit findings (S32 Phase 0)

**Date** : 2026-04-27
**Auditeur** : session fraiche S32 Phase 0 (Opus 4.6 1M)
**Tip audite** : `b1570f2` (chore(sprint31): Phase E — wrap-up)
**Methode** : audit_plan 7 tracks A-G + 3 meta-tracks, code-first opinion avant PATTERNS.md

---

## Verdict : **PASS**

0 P0, 0 P1, 2 P2, 3 P3.
Rigor G4 satisfait (2 P2 issus d'angles non couverts par les
phase reviews intra-sprint — pas de findings importes).

---

## Dimensions auditees

| Track | Items | Verdict track | Findings |
|---|---|---|---|
| A — task_runner reel (Phase A `e85623a`) | TR-1..TR-6 | 5/6 PASS, 1 angle G4 | P2-AUDIT-1 |
| B — output filter E2E (Phase B `0771dc8`) | OF-1..OF-6 | 6/6 PASS | — |
| C — Tor transport phase 1 (Phase C `687f6db`) | TT-1..TT-7 | 6/7 PASS, 1 carry confirme | P3-AUDIT-1, P3-AUDIT-3 |
| D — P2 batch + G2 (Phase D `ab09b5d`) | BD-1..BD-6 | 4/6 PASS, 2 angles | P2-AUDIT-2, P3-AUDIT-2 |
| E — G1 Design Review Board | sprint31_design_review.md | PASS (exists) | — |
| F — Phase review completeness | 8/8 artefacts (4 preflights + 4 reviews) | PASS | — |
| G — HARDENING drift | S31 entry disclosed scope | PASS | — |
| Meta G8 traceability | 4 EXECUTE, 0 DESIGN-CONFLICT, 4 feat commits | PASS | — |
| Meta Roadmap Alexandria deviation | kickoff §1.2 + D5 explicite | PASS | — |
| Pre-launch protocol | `*_VERSION = 1` partout, 0 wire S31 | PASS | — |

---

## Findings

### P2-AUDIT-1 — Executor silent param drops (Track A, angle G4)

**Non couvert par** : Phase A review (P2 LOC estimees + P3 schemars dep — aucun finding sur les params IPC).

**Constat** : `task_runner.rs:16` construit `GenerationRequest::new(params.model.clone(), params.prompt.clone())` — seuls `model` + `prompt` consommes. Trois champs de `TaskExecuteParams` (definis dans `ipc.rs:98-108`) sont silently dropped :

| Champ | Impact | Severite |
|---|---|---|
| `max_tokens: u32` | Ollama genere sans cap `num_predict` — worker produit au-dela de la limite demandee par le coordinator | Fonctionnel |
| `grammar: Option<String>` | Coordinator peut demander JSON structure — executor retourne du raw text | Fonctionnel |
| `watermark_config: Option<WatermarkConfig>` | Couche 6 SynthID watermark doctrine doc-vs-runtime divergence | Defense-in-depth |

**Evidence** :
- `crates/nexus-executor/src/task_runner.rs:16` : `GenerationRequest::new(params.model.clone(), params.prompt.clone())` — pas d'`.options()` appele.
- `crates/nexus-executor/src/ipc.rs:102-106` : champs declares, deserialises, jamais lus.
- Le mock TCP test (`task_runner.rs:78`) fournit `eval_count: 10, prompt_eval_count: 5` mais les champs `prompt_tokens` / `completion_tokens` ne sont pas mappes dans `TaskExecuteResult` (absents du struct `ipc.rs:118-125`).

**Ce n'est PAS un gap securite** (l'executor est worker-side, le worker peut tricher de toute facon), mais un **gap de fidelite wire contract** : le coordinator-side pense controler ces params, l'executor les ignore.

**Carry S32** : wire `max_tokens` via `GenerationOptions::default().num_predict(params.max_tokens)`. `grammar` + `watermark_config` peuvent rester P3 (Ollama ne supporte pas GBNF natif, watermark est defense-in-depth).

### P2-AUDIT-2 — HARDENING_ROADMAP compteurs frontmatter stale (Track D, BD-6)

**Non couvert par** : Phase D review (P2-REVIEW-D-1 identifie le probleme mais le delegue a Phase E). Phase E n'a PAS corrige le fichier.

**Constat** : `docs/security/HARDENING_ROADMAP.md` frontmatter `last_validated` line contient :
```
Compteurs ~878 Rust / ~195 SDK / ~401+36f+6s coord / ~46 gov / ~267 Vitest / ~1870 total.
```

Valeurs reelles post-S31 (mesurees Phase E commit body `b1570f2`) :
```
~878 Rust / ~195 SDK / ~406+36f+6s coord / ~46 gov / ~267 Vitest / ~1877 total.
```

Delta : coord 401 → 406 (+5 output filter E2E tests Phase B + +7 Tor tests Phase C = +12 reels vs baseline 394), total 1870 → 1877 (+7).

**Evidence** : `git show --stat b1570f2 | grep HARDENING` = pas de modification. Phase E commit body mentionne "Reconciliation P2-REVIEW-D-1" mais n'a PAS touche le fichier.

**Fix** : corriger les compteurs dans HARDENING_ROADMAP.md frontmatter au kickoff S32 ou en fix(sprint31) si juge bloquant. **P2 carry S32** (doc stale, pas de regression code).

### P3-AUDIT-1 — Feature gate `tor = []` compile trap (Track C, TT-6 angle)

**Constat** : `crates/nexus-core-rs/Cargo.toml:132` declare `tor = []` (feature vide, aucune dep activee). Or `tor_transport.rs:108-138` contient un bloc `#[cfg(feature = "tor")]` qui reference `arti_client::TorClient` — crate absente des deps (commentee `Cargo.toml:129`).

Si un utilisateur ou CI lance `cargo build --features tor`, le build **echoue** sur `unresolved import arti_client`. Aucun test ne compile avec `--features tor` actuellement.

**Couvert partiellement** : Phase C review P2-REVIEW-C-1 documente le carry arti-client dep activation. Mais le risque **compile-failure trap** n'est pas explicitement flag.

**Carry S32** : sera resolu quand rusqlite upgrade + arti-client dep activation atterrissent. Pas d'action immediate requise.

### P3-AUDIT-2 — HTTP FROST tests happy-path only (Track D, BD-5 angle)

**Constat** : les 4 tests HTTP FROST (`http.rs:2396-2693`) exercent le full flow happy-path (dealer → round1 → round2 → aggregate → Ed25519 verify). Le test `frost_http_aggregate_returns_valid_signature` verifie la signature end-to-end via `nexus_core_rs::crypto::verify` — excellent couverture du golden path.

Aucun test d'erreur path :
- `k > n` (invalid threshold)
- Malformed JSON body
- Wrong participant ID in round2
- Invalid nonces → aggregate failure

**Impact** : faible — les endpoints FROST sont loopback-only + peercreds. Mais les error paths non testes pourraient panic au lieu de retourner une erreur HTTP propre.

**Carry S32** : P3 informatif. Pas bloquant.

### P3-AUDIT-3 — Boot log Tor misleading quand disabled (Track C angle)

**Constat** : `coordinator.py:377-380` log `"Tor transport not available, using direct connections"` meme quand `config.enabled = false` (l'utilisateur n'a PAS demande Tor). Le log est misleading — il suggere une panne alors que c'est juste disabled.

**Fix sugere** : differencier `"Tor transport disabled by configuration"` vs `"Tor transport enabled but not available, using direct connections"`.

**Carry S32** : P3 nit. Pas bloquant.

---

## Angles G4 audites sans finding

| Angle (audit_plan §3) | Methode | Resultat |
|---|---|---|
| Tor enabled=false effet de bord | Read `tor_transport.rs:101-103` + `tor_client.py:64-66` | Noop clean (return early if !enabled) |
| Output filter context threading worker-influence | Read `validator.py:264-270` | Etanche (system_prompt + user_prompt from signed task_state, pas worker) |
| main.rs `#[allow(dead_code)]` mod ipc | Read `main.rs:3` | Pre-existant S20 (ipc module pas entierement utilise par main.rs, certains types sont test-only). Pas introduit S31. |

---

## Phase review findings — reconciliation

| Phase review finding | Audit verdict |
|---|---|
| P2-REVIEW-A-1 LOC estimees plan | Confirme — meta-process, carry S32 |
| P3-REVIEW-A-1 schemars dep inutile | Confirme — plan over-spec, resolu inline |
| P2-REVIEW-B-1 plan stale (result_guardrails.py path) | Confirme — observation meta-process |
| P3-REVIEW-B-1 output_filter_policy_path test | Confirme — exercee par 5 tests E2E |
| P2-REVIEW-C-1 rusqlite + arti dep activation | Confirme — carry S32 1/3 |
| P3-REVIEW-C-1 LOC estimees plan | Confirme — nit informatif |
| P2-REVIEW-D-1 compteurs HARDENING | **NON RESOLU** — P2-AUDIT-2 ci-dessus |
| P3-REVIEW-D-1 confidence_score cosmetique | Confirme — resolu Phase D |

---

## Carry-overs S32 (input pour kickoff)

| ID | Description | Reports | Priority |
|---|---|---|---|
| P2-REVIEW-B-1-S30 | Playwright COEP iframe test | **2/3 MANDATORY S33** | Phase dette S32 ou exemption |
| P2-REVIEW-C-1 | rusqlite 0.32→0.36 + arti-client dep activation | 1/3 | Phase dette S32 (couple iroh 0.98) |
| P2-REVIEW-A-1 | LOC plan meta-process | 1/3 | Discipline plan-writing |
| P2-AUDIT-1 | Executor silent param drops (max_tokens) | NEW | Wire max_tokens dans GenerationRequest |
| P2-AUDIT-2 | HARDENING compteurs frontmatter stale | NEW | Fix doc au kickoff S32 |
| LT-6 | iroh 0.98 upgrade (trigger met) | scheduled S32 | Phase dette dedie |

---

## Recommendation

- **Verdict PASS** : 0 P0, 0 P1 → S32 Phase A (ou phase dette upgrade iroh 0.98) peut demarrer directement.
- Les 2 P2 audit ne sont PAS bloquants (fidelite wire + doc stale).
- S32 kickoff devra integrer les 2 P2-AUDIT dans ses carries.
