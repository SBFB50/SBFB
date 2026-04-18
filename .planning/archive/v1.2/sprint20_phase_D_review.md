# Sprint 20 Phase D — nexus-phase-auditor review

HEAD pre-commit: 2e045f1 (chore(planning): Sprint 20 Phase C — audit review archive)
Draft commit body: "feat(sprint20): Phase D — structured output dual-backend LlmBackend (Ollama format + llama.cpp llguidance)"
Timebox: 55m

## Verdict : PASS-with-carry

(0 P0, 0 P1, 2 P2 documentés — rigor signal G4 satisfait)

Commit autorisé. Les 2 P2 sont portés comme `sprint20_audit_findings.md` entrée Phase D et doivent être résolus en Phase F ou reportés `sprint21_audit_findings.md` selon la politique carry.

---

## Dimensions

### Security

- [x] **unsafe blocks** : 0 nouveau `unsafe` block dans le diff. Aucun `SAFETY` comment requis.
- [x] **unwrap/panic en prod** : `task_response_schema()` (prod, non-test) utilise `.expect("schemars RootSchema serializes cleanly...")` à la ligne 146. Infallible en pratique (serde_json::to_value d'un RootSchema est purement structurel), mais sans commentaire `// INFALLIBLE: ...` explicitant la garantie. Log P3 ci-dessous.
- [x] **unwrap/expect en tests** : nombreux `.unwrap()` et `.expect()` dans les blocs `#[cfg(test)]` — acceptable par convention.
- [x] **loopback / peer creds** : N/A Phase D (aucun endpoint HTTP loopback modifié).
- [x] **wire format JCS** : N/A Phase D (le schéma JSON est une structure locale, pas un wire format P2P canonique). Les chemins de signing restent dans `canonical.rs` inchangé.
- [x] **path traversal (expand_tilde)** : `LlamaCppBackend::from_config` appelle `expand_tilde(&cfg.model_path)` puis `PathBuf::from(...)`. La fonction `expand_tilde` fait `strip_prefix("~/")` + join HOME. Pas de validation `Path::components()` contre `..`. Vecteur d'impact : un opérateur qui met `~/../../etc/sensitive` dans `worker.toml` peut pointer vers un chemin arbitraire. **Risque faible** (config locale trust-root, pas user input), mais absent du threat model documenté. Log P3.
- [x] **secrets** : aucun secret, token, ou credential dans le diff. Pattern `(AKIA|ghp_|pat_|sbfb_[a-z]+_...)` : 0 match.
- [x] **schema enforcement** : `deny_unknown_fields` présent sur `TaskResponse` et `ToolCall` — conforme au pattern G-3 PATTERNS.md.

### Patterns

- [x] **SPDX headers** : présents sur les 7 nouveaux fichiers Rust (`mod.rs`, `ollama.rs`, `llama_cpp.rs`, `factory.rs`, `schema_bridge.rs`, `task_response.rs`, `schemas/mod.rs`). Conforme.
- [x] **`TASK_RESPONSE_VERSION = 1`** : pré-launch policy respectée. `version: u8 = 1`, pas de tolerant decoder multi-version.
- [x] **`#[serde(default)]` légitimité** : `reasoning` et `tool_calls` portent `#[serde(default)]` pour runtime tolerance (client Python minimal → pas de 422). Conforme à la politique CLAUDE.md §Pre-launch. Doc inline dans les champs.
- [x] **Sprint 19.1 primitive/wire/enforcement separation** : Phase D livre la primitive (`TaskResponse` struct + schema) + wire Ollama complet (format param + defensive validator) + wire llama.cpp partiel (matcher state machine + defensive validator, logit-bias S21+ carry). Structure respectée — les couches sont explicitement séparées.
- [x] **dual-backend `UnsupportedBackend` fail-loud** : `build_backend` refuse de silent-fallback vers Ollama quand `backend = "llama_cpp"` mais feature off. Conforme au principe fail-fast.
- [P2] **PATTERNS.md §P30 affirmation inexacte** : voir Finding P2-1 ci-dessous.

### Working tree audit (G5)

- [x] **PHASE** : 22 fichiers stagés (A/M/D) — tous dans le scope Phase D (dual-backend LlmBackend, schemas, config, engine wiring, PATTERNS). La suppression de `src/ollama.rs` est légitime (migré vers `src/llm/ollama.rs`).
- [x] **CRAFT** : 0 fichier planning/kickoff/SPRINT_LOG dans le diff. Le design doc `.planning/research/S20_phase_D_structured_output_design.md` est un artifact Phase D (non-CRAFT).
- [x] **DEBT** : 0 fichier tech-debt non autorisé.
- [x] **NOISE** : 0 fichier non-tracké (`.pdb`, `.exe`, `node_modules`, etc.).
- [x] **Section "Working tree audit (G5)"** : présente dans le draft commit body (listée dans l'entrée "PHASE 22 fichiers, CRAFT 0, DEBT 0, NOISE 0").

### Scope-cuts

Scan des items §8 kickoff contre le diff :

- `hardware keystore / TPM / StrongBox` : 0 implémentation. Mentions doctrinales dans PATTERNS.md et design doc.
- `HPKE` : 0 implémentation. Mention de rejet dans design doc (correct).
- `rate-limit` : occurrence dans `Cargo.toml` = commentaire `tower-http`. 0 implémentation.
- `redact` : occurrences dans PATTERNS.md et design doc = références futures Sprint 21. 0 implémentation.
- `kudos-weighted` : 1 occurrence dans PATTERNS.md = ref future. 0 implémentation.
- `sandbox / tool-calling` : `ToolCall` struct présente dans `TaskResponse` — **pre-déclaration explicite du wire format** pour éviter un version bump à S22. Le design doc §7.1 documente le rationale. Le coordinateur ignore `tool_calls` jusqu'à S22. Pas de scope creep.
- `DoH / DoT / dns.*fallback` : occurrence dans `crates/nexus-worker/src/cli.rs` — hors diff visible ici, à vérifier.
- `Arti / arti.*tor` : occurrence grep = le mot "artifacts" dans un commentaire `Cargo.toml`. 0 implémentation Tor.
- `PQC / ML-DSA / ML-KEM` : occurrences = références horizon long-terme dans design doc §7.3 + PATTERNS.md. 0 implémentation.

**Verdict scope-cuts : PASS.** La `ToolCall` pre-declaration est justifiée architecturalement et ne constitue pas du scope creep (format-only, zero behavior).

### Tests-delta

- [x] **Rust** : mesuré par `cargo nextest run -p nexus-worker-core -p nexus-core-rs --locked`. Baseline Phase C : 314 tests. Phase D : 341 tests. **Delta mesuré : +27**. Draft annonce +27 nets (598→625 workspace). **Cohérent.**
- [x] **Plan target vs livré** : le plan §7 Phase D annonçait +12 tests. Livré +27. Over-delivery positif (+15 supplémentaires) dû à la couverture étendue du refactoring config + engine + factory + stub. Aucun test skipped sans reason= détecté. Les 8 tests feature-gated `llm_llama_cpp` (dans `llama_cpp.rs`) ne s'exécutent pas sur machine Windows sans LLVM — correctement gérés par `#[cfg(feature = "llm_llama_cpp")]` sur le module.
- [x] **Python / Vitest / Playwright** : non touchés par Phase D. Compteurs inchangés (185 / 239 / 38).
- [x] **Workspace full** : `cargo nextest run --workspace --locked` non exécuté (PDB linker error `LNK1318` sur `nexus-worker` binary cible Windows — erreur pre-existante non liée à Phase D, confirmé par isolation sur les 2 crates touchées). Tests exhaustifs sur les crates modifiées : 341/341 PASS.

### Research-grounding

- [x] **`llguidance = "1.7"`** : tracé dans design doc §3 + Cargo.toml inline comment + kickoff §Sources. Context7 `/guidance-ai/llguidance` consulté 2026-04-18. RustSec : 0 advisory actif.
- [x] **`llama-cpp-2 = "0.1.143"`** : tracé dans design doc §3 + Cargo.toml inline comment. Context7 `/utilityai/llama-cpp-rs` consulté 2026-04-18. RustSec : 0 advisory actif.
- [x] **`schemars = "0.8.21"`** : tracé dans design doc §3 (pin justifié par contrainte transitive `ollama-rs 0.2.6`). RustSec : 0 advisory actif.
- [x] **`ollama-rs = "0.2.6"`** : workspace existant, inchangé en version.
- [x] **APIs crypto / specs standardisées** : aucune API crypto nouvelle dans ce diff. `serde_json::to_value` + `schemars::schema_for!` sont des utilitaires de sérialisation standard.
- [P2] **Version kickoff vs livrée** : kickoff §D4 spécifie `llguidance = "0.7"` mais la version livrée est `1.7` (bump majeur). Le design doc confirme que `1.7` est la version courante au 2026-04-18 et mentionne que la recherche a révélé le bump. Cependant le kickoff n'a pas été mis à jour (P2-D1 cité comme carry mais le kickoff reste incorrect). Voir Finding P2-2.

### Horizon long-terme + documentation amont

- [x] **Design doc présent** : `.planning/research/S20_phase_D_structured_output_design.md` (674 lignes, 12 sections). Présent avant le code (session 2026-04-18).
- [x] **Alternatives rejetées documentées** : D4 kickoff + design doc §2.2 : XGrammar (pas llama.cpp support), Outlines (Python IPC), GBNF natif (slower + pas Rust native), JSON Mode OpenAI (compat custom). Toutes avec rationale explicite.
- [x] **Solution la plus poussée** : `llguidance` est la solution Microsoft Reference pour JSON Schema constrained decoding sur llama.cpp. `aws-lc-rs` non requis pour ce cas (AEAD non utilisé en Phase D). Choix techniquement justifié.
- [x] **Aucune estimation LOC dans plan/kickoff** : grep `LOC estimee|~\s*\d+\s*LOC` → 0 match dans plan.md ou kickoff.md. Conforme §6.7.
- [P2] **§P30 documentation inexacte pour l'état Sprint 20** : voir Finding P2-1.

---

## Findings

### P2-1 — PATTERNS.md §P30 : affirmation inexacte sur l'enforcement LlamaCppBackend au Sprint 20

**Fichiers** : `docs/rust/PATTERNS.md:1558-1561` et `docs/rust/PATTERNS.md:1583-1597`, ainsi que `crates/nexus-worker-core/src/llm/llama_cpp.rs:307-308`

**Description** : PATTERNS.md §P30 section "Defense-in-depth" dit "Even with the grammar enforcing the format at sample time" et la section "Why Rust-side llguidance" dit "Matcher drives the `llama_cpp_2::sampling::LlamaSampler` chain". Ces deux affirmations décrivent l'état **cible** (Sprint 21+), pas l'état Sprint 20.

À Sprint 20, `apply_matcher_mask()` dans `llama_cpp.rs` :
- Calcule le mask (`compute_mask`) pour faire avancer l'état interne du matcher
- Extrait les `ff_tokens` (forced-forward tokens) et les consomme
- **Ne pousse PAS de logit-bias sampler frame** (les params `_sampler` et `_ctx` sont ignorés)

Conséquence : le `LlamaCppBackend` avec schéma activé fonctionne comme suit au Sprint 20 :
1. Matcher state machine avance (via ff_tokens bookkeeping)
2. Sampler sélectionne librement parmi tous les tokens (pas de contrainte logit)
3. `consume_token()` est appelé POST-sélection — si le matcher rejette → `SchemaViolation`
4. La validation finale (`validate_task_response`) est le vrai garde-fou

Ce comportement est **honnêtement documenté** dans le docstring de `apply_matcher_mask()` (lignes 388-408) et dans le design doc §4.3. Mais §P30 dans PATTERNS.md ne reflète pas cette nuance et crée une fausse impression chez un futur contributeur.

Par ailleurs, le commentaire à `llama_cpp.rs:307-308` dit "We apply the llguidance mask manually against the candidates array below (before sampler.sample) so the matcher's token-level enforcement beats the temperature sampler" — ce qui est factuellement inexact au Sprint 20.

**Fix requis** : Ajouter dans §P30 "Defense-in-depth" une note explicite :

```markdown
> **Sprint 20 état** : pour `LlamaCppBackend`, le logit-bias wire
> n'est pas encore actif (P3-D3 carry S21+). Le Matcher state
> machine avance correctement via `ff_tokens` + `consume_token`
> post-sélection ; la validation post-décode (`validate_task_response`)
> reste le garde-fou effectif. Wire-level enforcement (token rejeté
> avant sampling) arrive S21+.
```

Et corriger le commentaire trompeur dans `generate_blocking` à `llama_cpp.rs:307-308`.

### P2-2 — Kickoff §D4 version llguidance incorrecte non mise à jour (carry P2-D1)

**Fichier** : `.planning/active/sprint20_kickoff.md` §D4

**Description** : Le kickoff §D4 spécifie `llguidance = "0.7"` :
```
- `Cargo.toml` workspace : `llguidance = "0.7"` optional dep dans `nexus-worker-core`.
```

La version livrée est `1.7` (bump majeur, constaté via context7 2026-04-18). Le design doc et le Cargo.toml inline comment documentent correctement `1.7`. Le kickoff reste incorrect.

Ce finding est annoncé comme P2-D1 carry dans le draft commit body mais le kickoff lui-même n'a pas été mis à jour. Pour la traçabilité de l'audit gate S21, le kickoff doit refléter la version réellement utilisée.

**Fix requis** : Mettre à jour `sprint20_kickoff.md §D4` : `llguidance = "0.7"` → `llguidance = "1.7"` avec note de contexte "bumped à la session Phase D 2026-04-18 (context7 confirm)".

---

### P3-1 — expect() en prod sans commentaire INFALLIBLE

**Fichier** : `crates/nexus-core-rs/src/schemas/task_response.rs:146`

```rust
serde_json::to_value(root).expect("schemars RootSchema serializes cleanly to serde_json::Value")
```

`serde_json::to_value(RootSchema)` est effectivement infaillible (type purement structurel). L'expect est justifié mais la convention du projet (S4 audit note : "No .unwrap() in production code") suggère d'ajouter un commentaire `// INFALLIBLE:` avant la ligne, ou de restructurer en `unwrap_or_else(|e| unreachable!("RootSchema is always JSON-serializable: {e}"))` pour clarifier l'invariant.

### P3-2 — expand_tilde sans validation composantes path (path traversal local)

**Fichier** : `crates/nexus-worker-core/src/llm/llama_cpp.rs:442-448`

`expand_tilde("~/../../etc/passwd")` → join HOME → chemin hors home directory. Impact limité (worker.toml = config locale, trust-root opérateur), mais non documenté dans le threat model. Ajouter une validation `path.components().any(|c| c == std::path::Component::ParentDir)` avec reject `InvalidConfig` ou documenter explicitement pourquoi ce n'est pas un vecteur (config trust boundary).

### P3-3 — Plan Phase D delta tests sous-estimé non réconcilié

**Fichier** : `.planning/active/sprint20_plan.md §7 Phase D`

Plan cible `+12 tests`, livré `+27 tests`. Écart significatif (+15) non expliqué dans le commit body. L'over-delivery est positive mais la divergence plan/réel devrait être notée dans le commit body (ex: "plan ciblait +12, livré +27 dû au refactoring config+engine+stub étendu") pour la traçabilité des projections §9.

---

## Recommendation

**Commit autorisé.** 0 P0, 0 P1.

Actions avant Phase E :

1. **P2-1** (obligatoire) : corriger le commentaire `llama_cpp.rs:307-308` et ajouter la note d'état Sprint 20 dans `docs/rust/PATTERNS.md §P30` section "Defense-in-depth". Peut être fait en chore inline pre-Phase E ou groupé en chore de résolution Phase F.
2. **P2-2** (obligatoire) : mettre à jour `sprint20_kickoff.md §D4` : `llguidance = "0.7"` → `"1.7"`. Idem timing.
3. **P3-1/P3-2/P3-3** : nits à documenter dans `sprint20_audit_findings.md` pour Phase F ou S21.

Les P2 résolus AVANT le commit Phase E ou groupés dans un `chore(sprint20): audit-P2 Phase-D batch` distinct du commit Phase E (discipline commit atomique).
