# Sprint 79 — Vérification

**Thème** : Capacité Factory « app-authoring » — maîtrise anime.js 4.5.0 + daisyUI
5.5.23 dans le process de fabrication d'apps SBFB (module de connaissance
versionné + prompt-kind `app-authoring` + gate CSP déterministe Rust + couche
docs-contrat).

**Statut** : DONE — 8 phases feature (A→H) + Phase I (docs-contract closure +
wrap-up). Tip de clôture = commit Phase I.

---

## 1. Phases livrées

| Phase | Commit | Livrable |
|---|---|---|
| A | `9297f08` | promotion pack anime.js + `MANIFEST.json` blake3 (test recompute hermétique) |
| B | `b27079c` | canon cadence docs-contrat (README §6.12 + PATTERNS §P70 + AGENT_SYSTEM §7) + `check-frontier-contracts.sh` générique câblé 3 surfaces + fix faux-vert `phase-review-cross-check` régex |
| C | `95aba5b` | prompt-kind `app-authoring` (9e entrée `PROMPT_KINDS`, fiche CSP-annotée) |
| D | `070c7a9` | injection context-pack `authoring_knowledge` (hashed path refs, jamais inliné) + routing zone UI |
| E | `97f7720` | gate CSP déterministe Rust `run_gate_csp_authoring` BLOQUANT + source CSP unique `BLOB_SERVE_CSP` déplacé en `csp.rs` |
| F | `8d7ee81` | pack daisyUI 5.5.23 knowledge + extension prompt-kind |
| G | `38e50b2` | copilote Ollama keyless (`ExecutionTarget::Ollama`) + starter template daisyui vendoré |
| H | `545c78e` | self-check runtime CSP (filet runtime du gate statique) + confirmation CSP daemon + testabilité T1/T2 |
| I | [ce commit] | couche GUIDE docs-contract Factory (Diataxis FR + `llms.txt` + `WIRING_SPEC.md` + exemple `include!` runnable + `check-factory-docs.sh` câblé 3 surfaces + volet sémantique source-ref P2) + wrap-up final |

Commits hors-phase absorbés : `17ead31` fix unbounded phase discovery,
`eaafe2e` vérification doc-artefact Phase C + extension scope Phase I,
`2447f30` check-frontier-contracts volet 4 (prompt-kind provenance edge),
`16d84c2` SessionStart matcher `*` + read-proof.

## 2. Phase I — détail

Couche GUIDE (nœud 4 doctrine contrat-pour-LLM §2/§3), miroir S77 Phase N :

- **Diataxis FR** : `docs/factory/{README,EXPLANATION,HOW_TO_WIRE}.md` (corps FR) +
  `REFERENCE.md` (corps EN, agent-facing, exempté french-body).
- **`docs/factory/llms.txt`** (format llmstxt.org : H1 + blockquote Truth-Stack +
  sections de liens rank-1) + section `## Factory (app-authoring)` au `llms.txt`
  racine (banner re-scopé sharding + factory).
- **`docs/factory/WIRING_SPEC.md`** (contract-dense, source_refs `path:Symbol`
  vérifiés : `BLOB_SERVE_CSP`, `run_gate_csp_authoring`, `PROMPT_KINDS`,
  `app-authoring`, `authoring_knowledge`, `handle_context_pack`, `TemplateConfig`,
  `none_directives`, `CSS_URL_ALLOW`).
- **Exemple runnable** `docs/factory/examples/csp_contract.rs` lifté VERBATIM via
  `include!` dans `crates/nexus-core-rs/tests/factory_csp_contract.rs` (3 tests,
  drift policy → build rouge). **Option B (preflight PLAN-ADAPT R1)** : cible
  l'API lib `nexus_core_rs::csp` (re-export `lib.rs:104`), PAS le gate bin-privé
  `run_gate_csp_authoring` (`sbfb-factory` = crate binaire-pure sans `[lib]`).
- **`scripts/check-factory-docs.sh`** : clone factory-scopé de
  `check-sharding-docs.sh` (link-check + anchors + honesty-gate caveat cardinal +
  french-body + source-ref-check rank-1 + required-anchor allowlist + Truth-Stack
  + Not evidenced) **+ volet (5) line-semantic** (preflight R2) : refs nom-nu
  `PRIMITIVES.md:N` / `README.md:N` de `prompts/agent/app-authoring.md` résolues
  contre le pack `docs/factory/knowledge/animejs/` + bornes de ligne. **Câblé
  BLOQUANT 3 surfaces** (`ci.yml [16]`, `.woodpecker/ci-linux.yml` factory-docs-check
  même digest `bash:5`, `verify.sh step 21`).
- **Préflight PLAN-ADAPT R3** : `BLOB_SERVE_CSP` ancré `csp.rs:33` (pas l'ancre
  morte `blob_serve.rs:286`) ; orthographe `run_gate_csp_authoring`.
- **Cross-gate (preflight watch-item)** : reword banner racine `llms.txt` → marqueur
  `check-sharding-docs.sh:232` migré de `sharding subsystem only` à
  `whole-repo agent index is` (survit au reword, gate sharding reste vert).

## 3. Delta tests

- **Rust nextest** : +3 (binaire `factory_csp_contract` : 3 tests
  `blob_serve_csp_blocks_the_six_exfil_directives` /
  `css_url_allow_is_the_only_absolute_url_exception` /
  `authoring_rules_stay_in_lockstep_with_the_policy`). 1991 → **1994** (Windows
  confirmé : nextest 1994/1994 0-skip ; Docker canonique = re-run pré-push).
- **Doc-lint** : +1 script net-new `check-factory-docs.sh` (vert) ; `check-sharding-docs.sh`
  + `check-frontier-contracts.sh` restent verts (non-régression).
- **0 bump wire, 0 dep nouvelle** : Phase I = docs + 1 script + 1 test `include!`
  d'une API existante ; aucun `*_FORMAT_VERSION` touché, `SBFB.json schema_version=2`
  inchangé.
- **Gate de testabilité par-sprint (§4)** : T1 `web/e2e/app-authoring.spec.ts`
  (Phase H) BLOQUANT-vert + T2 `scripts/acceptance/app_authoring_capability.sh`
  artefact JSON PASS confirmés verts à la clôture.

## 4. Scope cuts respectés (Day-0 figées, kickoff 177–234)

- 0 nouvelle primitive de code en Phase I (seul artefact code = 1 exemple lifté
  verbatim, drift-gardé).
- Wrapper `.claude/skills/app-authoring` : NON (Day-0 #6) — différé.
- Alliance PR-plugin Open CoDesign : DIFFÉRÉE (post-1.0, mono-auteur).
- Auto-fetch de fraîcheur : interdit (`connect-src 'none'`) — re-extraction manuelle
  au bump, gouvernée par `MANIFEST.json`.
- Nouvelle route daemon / autorité de verdict : aucune (scellage 100 % Factory).
- Connaissance CONSOMMÉE jamais autoritaire : 0 verdict PASS,
  `chat_history_authoritative=false` préservé.

## 5. Carry-over memory (à fusionner au prochain kickoff)

- **Capacité app-authoring = PROVISIONAL là où ça compte** : gate CSP + packs +
  template + self-check viewer LIVE-testés ; **parcours in-vivo de bout en bout**
  (auteur réel → publish → rendu cross-pair) et **efficacité générative**
  (prompt-kind / copilote Ollama) = `Not evidenced` → carry `sprint80_audit_plan.md`.
- **Cadence docs-contrat canonisée (Phase B)** : règle process README §6.12 +
  PATTERNS §P70 + AGENT_SYSTEM §7 + gate générique `check-frontier-contracts.sh`
  câblé CI. S79 = 1re instance de référence (sharding S77 = 1er pilote).
- **Carry P2 ouvert (Phase B)** : couverture étiquette des **~21 familles wire**
  non-schématisées (registre `// FRONTIER:` incrémental) → `sprint80_audit_plan.md`.
- **Sharding S77 PROVISIONAL** (carry P1) : orchestrateur de session in-vivo +
  benchmark live 2-machines = RIG-ABSENT, toujours ouvert (Factory-first a différé
  S78 ; cf. `sprint78_audit_plan.md`).
- **`sbfb-factory` est une crate binaire-pure** (pas de `[lib]`) — fait porteur
  pour tout futur exemple runnable / test cross-crate (cibler `nexus_core_rs`).

## 6. Verdict

S79 **DONE**. 8 phases feature + closure docs-contract. G8 par phase (PLAN-ADAPT
récurrents evidence-based, 0 DESIGN-CONFLICT) ; reviews Workflow multi-agents Opus
4.8 1M ; Codex GPT 5.5 par phase. Invariant **héberger ≠ publier, seeder ≠ auteur**
tenu. 0 bump wire, 0 dep. Audit gate S79 = Phase 0 de S80, cf.
`sprint80_audit_plan.md`.
