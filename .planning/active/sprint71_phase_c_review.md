# Phase Review — Sprint 71 Phase C (securite Factory)

## Verdict: PASS

Promu de PASS-PENDING apres reconciliation Codex (§Codex reconciliation
ci-dessous). Codex GPT 5.5 : 6 livrables, 5 CONFIRMES, 0 GAP, 1 PARTIEL
(correct-by-design, traite). Aucun GAP P0/P1.

(Rigor signal : 3 findings P2 documentes / >=1 requis pour PASS rigoureux.)

Date : 2026-05-30. HEAD : `0daff81`. Fallback skill `nexus-phase-review`
(l'agent `nexus-phase-review-deep` n'est pas enregistre dans cette
session — meme verdicts, profondeur reduite).

## Staging check (Step 1bis)
- Phase fichiers (10) : `crates/sbfb-factory/src/auth.rs` (NEW),
  `operator_server.rs`, `llm_bridge.rs`, `daemon_client.rs`, `main.rs`,
  `Cargo.toml`, `Cargo.lock`, `tests/operator_server.rs`,
  `tools/factory-operator/vite.config.ts`,
  `docs/agent/RRV_FACTORY_CONTRACT.md`.
- Artefacts planning de CETTE phase : `sprint71_phase_c_preflight.md` +
  ce `sprint71_phase_c_review.md` (+ codex_review a venir) — vont dans
  le commit phase (G8 artefact), pas un chore separe.
- Planning/docs split : N/A — `RRV_FACTORY_CONTRACT.md §4` est un
  livrable explicite du plan §7 C.2 (amendement PO-2), pas un doc
  hors-scope. Aucun fichier d'un sprint anterieur, aucun scope-cut leak.
- Untracked accidentels : 0.
- **Working tree coherent pour un commit phase atomique.**

## Memory consultation (Step 1.5)
| Memory | Contrainte | Statut |
|--------|-----------|--------|
| `feedback_approach.md` | pick deepest, no band-aid, research before code | RESPECTE — reutilise le pattern daemon S16 audite plutot qu'une lib externe ; preflight G8 fait avant code |
| `feedback_model_46.md` | toujours `claude-opus-4-8[1m]`, jamais alias | RESPECTE — `default_model()` = `claude-opus-4-8[1m]`, `"sonnet"` supprime (G9) |
| `feedback_context7_systematic.md` | context7 avant code lib/API | RESPECTE — preflight S1a a consulte tower-http (context7) + tokio + OWASP CSRF ; aucune nouvelle API crypto (OsRng = meme primitive que daemon) |
| `feedback_v1_prod_ready.md` | jamais differer UX avec excuse post-v1.0 | RESPECTE — token livre au front meme commit (proxy Vite), happy-path PO-2 preserve |

Aucune violation memory (pas de P1 memory).

## Suites (Step 2 / Step 3)
- cargo fmt --all --check : clean.
- cargo clippy --workspace --all-targets --locked -- -D warnings : **0 warning**.
- cargo nextest run --workspace --locked : **1512 passed, 0 skipped**
  (baseline Phase B 1498 → +14).
- cargo test --workspace --locked --doc : 0 fail.
- cargo build -p nexus-shell-daemon --release : OK.
- factory-operator : `eslint .` clean (rc 0), `npm run build`
  (tsc -b && vite build) OK.
- web/ shell (non-regression, front non touche) : lance — voir Delta tests.

| Suite | Avant | Apres | Delta |
|-------|-------|-------|-------|
| Rust workspace (nextest) | 1498 | 1512 | +14 |
| Rust doctests | 0 | 0 | +0 |
| Vitest unit (web/) | 279 | 279 | +0 (web/ non touche) |
| size-limit | 6/6 | 6/6 | inchange |

Decomposition +14 : `auth::tests` (5 : loopback_host_accepts /
loopback_host_rejects / loopback_origin / constant_time_eq /
env_token_takes_precedence) ; `llm_bridge::tests` (2 :
missing_claude_diagnostic / spawn_times_out) ; integration
`operator_server` (7 : server_rejects_missing_token /
server_rejects_foreign_host / cors_restricts_origin /
token_request_succeeds / sse_gates_sensitive_action /
sse_allows_nonsensitive / chat_stream_uses_opus_model).

## Modified-file branch coverage (Step 2bis, G9)
- `operator_server.rs` `handle_chat_stream` branche gate `is_sensitive`
  → testee par `sse_gates_sensitive_action` (gate) + `sse_allows_nonsensitive`
  (happy-path) ✅
- `operator_server.rs` wiring modele session → testee par
  `chat_stream_uses_opus_model` ✅
- `operator_server.rs` `build_router` middleware auth → testee par
  `server_rejects_missing_token` / `server_rejects_foreign_host` /
  `cors_restricts_origin` / `token_request_succeeds` ✅
- `operator_server.rs` `handle_chat_send` persistance `req.model` →
  exercee indirectement (session.model relu au stream) ✅
- `llm_bridge.rs` `spawn_agent_stream` NotFound + idle-timeout →
  `missing_claude_diagnostic` / `spawn_times_out` ✅
- `daemon_client.rs` `crate::auth::auth_token_path` → couvert par
  `discover_fails_without_running_json` (chemin token) ✅
- Les 24 tests operator existants → verts avec le harness token-aware
  (smoke `operator_once_smoke` inclus). ✅

## Commit body validation (Step 4 / 4bis)
9/9 headers presents dans le draft (`## Contexte`, `## Fichiers`,
`## Delta tests`, `## Verification §7.4`, `## Scope cuts`,
`## G8 traceability`, `## Pre-launch protocol`, `## Codex verification`,
`## Carry closure / Unblock`). Titre `fix(factory): Sprint 71 Phase C —
gate SSE + opus-4-8 + token auth + spawn timeout`. Delta +14 coherent.
Co-Authored-By Opus 4.8 (1M) present.

## Research grounding (Step 4ter)
- **4ter-A preflight G8** : `sprint71_phase_c_preflight.md` existe, 5
  scans presents (S1a/S1b/S2/S3/S4), verdict SCOPE-CUT-CONSISTENT. S1a
  nomme les references OSS (tower-http `AllowOrigin::predicate` via
  context7, tokio process timeout/kill docs.rs, OWASP CSRF, CVE-2025-49596 /
  CVE-2025-66414, microsoft/sudo) ET la reference interne forte (daemon
  S16 `http.rs:513` + `auth.rs`). PASS.
- **4ter-B deps** : aucune nouvelle crate au lock — `rand` (OsRng) etait
  deja au workspace (`Cargo.toml:60`), +1 ligne edge dans Cargo.lock,
  zero bump. `rand::rngs::OsRng` = exactement la primitive du daemon
  `generate_token` (auth.rs:159-164). axum/tower-http/tokio deja au lock.
  PASS.

## Horizon long-terme + documentation amont (Step 4quater)
- Nouveau module `auth.rs` : doc-comment complet + reference au pattern
  canonique daemon S16 + duplication tracee comme tech debt. ✅
- D3/D4/D5/D6 Day-0 ont alternatives rejetees (kickoff §5 : tolerance
  floue rejetee, dep `which` rejetee, bypassPermissions delibere). ✅
- Solution la plus poussee : alignement sur le standard loopback durci
  S16 (audite, en vigueur), pas une primitive ad-hoc. ✅
- Aucune estimation LOC au plan. ✅

## Scope cuts verification (Step 5)
Aucune ligne du diff ne touche un scope cut (kickoff §8 / plan §12) :
- #1 ProviderRouter multi-LLM → S72 : D4 cable seulement le DEFAUT
  `claude-opus-4-8[1m]` + le passthrough de `req.model`, **pas** un
  router multi-provider. ✅
- #2 Chat Factory route reseau → S72 : aucune logique de routage ajoutee. ✅
- #16 Packaging produit Factory → S74 : le token bootstrap proxy est de
  la securite dev, pas du packaging onboarding. ✅
- Diff limite a sbfb-factory + factory-operator + docs/agent. 0 fichier
  d'un scope cut. ✅

## Findings (rigor signal — 3 P2 + 3 P3)
- **P2** : Duplication securite — `crates/sbfb-factory/src/auth.rs`
  reimplemente `is_loopback_host`/`is_loopback_origin` + token gen, qui
  vivent canoniquement dans `nexus-shell-daemon-core::auth` (S16).
  Deliberee (evite de tirer iroh/gossip dans un outil de scaffolding),
  tracee en doc-comment ; les 2 copies sont unit-testees. **Carry** :
  unifier dans un module loopback-auth partage, consolider PATTERNS au
  wrap-up S71 Phase E. Code de securite duplique = risque de drift.
- **P2** : `default_model()` = `claude-opus-4-8[1m]` honore la regle
  modele gelee, mais l'acceptation du suffixe `[1m]` par le flag
  `--model` du CLI `claude` n'est **pas verifiee a l'execution** — le
  chemin SSE `/chat/{id}/stream` est orphelin cote front reel
  (`AgentChat.tsx` utilise `/api/terminal/ws`). **Carry** : verifier la
  valeur `--model` acceptee par le CLI avant de re-cabler le SSE
  (`operator_server.rs:776` via `llm_bridge.rs`).
- **P2** : Le terminal PTY WebSocket `/api/terminal/ws` est protege par
  l'auth de connexion (Host+Origin+token, D5) mais **pas** par le gate
  de contenu SENSITIVE_ACTIONS (D3) — il n'a pas de "dernier message
  user" a inspecter (terminal brut pilote par l'utilisateur). Limite
  documentee (contrat §4 + ce review). **Carry** : T1 plein
  (CONFIRM_PROMPT spawn/write) post-S71 (`LOOPBACK_..._TRUST_TIERS §6`).
- **P3** : Le filtre SENSITIVE_ACTIONS est un `contains` substring sur
  `to_lowercase()` — `"PASS"`→`"pass"` matche `compass`/`passage` (faux
  positifs). **Fail-safe** (sur-gate = sur) et **identique** au gate
  existant `/chat/message` + `/chat/send` (aucune regression). Affiner
  (word-boundary) = amelioration future, hors scope.
- **P3** : Token livre au front via le proxy Vite **dev-only** ; le
  packaging produit de l'Operator front (livraison token en prod) est le
  scope cut #16 → S74. Acceptable pre-v1.0 (Operator = outil dev local).
- **P3** : `sse_gate`/`sse_error` construisent le JSON via `format!`
  avec interpolation brute — sur aujourd'hui (messages = litteraux
  statiques), mais un futur message dynamique devrait passer par
  `serde_json` pour eviter une rupture JSON.

Aucun P0/P1.

## Codex gate (§4.5) — zero exemption
- Status : FAIT. Prompt `.git/CODEX_SPRINT71_PHASE_C.txt` →
  `codex exec --dangerously-bypass-approvals-and-sandbox -o
  .planning/active/sprint71_phase_c_codex_review.md` (GPT 5.5, output
  brut non reecrit). Codex a re-execute les tests : `auth::tests` 5/5,
  `llm_bridge::tests` 5/5, `operator_server` 29/29 OK.
- Resultat : 6 livrables — **5 CONFIRMES, 0 GAP, 1 PARTIEL**.

## Codex reconciliation
- Status : FAIT.
- 0 GAP P0/P1 → aucune correction bloquante, pas de BOUCLE complete
  requise.
- **1 PARTIEL (Livrable 3 / G7)** : la preflight `OPTIONS` est repondue
  par CORS (couche externe) AVANT le token (couche interne). Codex le
  classe PARTIEL, PAS GAP ("ce n'est pas un spawn/write direct", deja
  documente l.141-142). **Correct-by-design** : une preflight ne porte
  aucune donnee et ne declenche aucun handler ; la requete REELLE passe
  par `auth_required` (Host/Origin/token). CORS DOIT rester externe
  (sinon l'auth 401 toute preflight — le browser n'envoie jamais le
  bearer sur une preflight — et casse le CORS legitime). Traitement :
  commentaire de code `operator_server.rs` enrichi pour expliciter ce
  rationale securite (amelioration doc, zero changement de comportement).
  Suites relancees apres l'edit commentaire : fmt clean, clippy 0
  warning, nextest sbfb-factory 112/112. Note de coherence : cette
  meme surface a ete sondee par un script OPTIONS injecte hors-bande
  (Host: evil.com) — NON execute (canal non fiable) ; l'analyse
  independante Codex + code confirme l'absence de faille.
- Review final : **PASS** (rapport Codex lu, 0 GAP P0/P1, PARTIEL
  correct-by-design documente).

## Recommendation
- Ready to commit : **oui** (PASS final, Codex reconcilie).
- Carry-overs S72 (P2 non resolus, entree obligatoire
  `sprint72_audit_findings.md`) : duplication auth (unify) ; `--model`
  CLI acceptance (verifier avant re-cablage SSE) ; PTY WS T1 plein.
- Corrections appliquees : commentaire rationale OPTIONS/CORS (PARTIEL
  Codex).

## Post-commit obligatoire
- [ ] Update `nexus_grid_pivot.md` (tip SHA + compteurs 1512 + carries P2).
- [ ] Update `MEMORY.md` (ligne index pivot).
- [ ] `review.md` + `preflight.md` + `codex_review.md` stages dans le
      commit phase.
