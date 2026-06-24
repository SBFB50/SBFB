# Sprint 79 — Plan : Capacité Factory app-authoring (maîtrise anime.js + daisyUI dans le process de fabrication d'apps SBFB)

**Écrit** : 2026-06-23.
**Tip master** : `0f597cf` (S77 Phase K wrap-up, poussé origin/master).
**Roadmap** : capacité Factory **orthogonale** à l'arc compute (S71-S77) et au carry sharding S78. v2.1 Arc Factory.
**Design durci (la référence — exécution, pas conception)** :
`examples/daisyui-animejs-showcase/knowledge/factory-integration-design.md` (architecture hybride-phasée, 5 couches, plan A-G, 1er livrable) +
`examples/daisyui-animejs-showcase/knowledge/factory-integration-hardened.md` (8 questions tranchées, contrat CSP prouvé-code, snapshot versions, ajustements plan).

> **Scope ULTRA-COMPLET (directive PO `feedback_ultra_complete_sprints`)** : A→G **d'un bloc**,
> 0 defer du cœur. La capacité livre TOUT son objectif — module de connaissance versionné +
> prompt-kind `app-authoring` + injection context-pack/routing + **gate CSP déterministe Rust
> BLOQUANT dès son introduction** + pack daisyUI + copilote Ollama keyless + starter template +
> self-check runtime + factorisation source CSP unique + corrections check-csp. La dette / les gates
> sont des **phases du sprint**, jamais des defers.
> Phases ILLIMITÉES (regex `Phase [A-Z]+[0-9]?`, README §4). `Phase 0` = audit gate du sprint
> **réellement clos** au démarrage.

---

## §0 Arbitrages PO à trancher au boot (AVANT Phase A)

Deux décisions de cadrage roadmap ne sont **pas** tranchées par le design durci (laissées-PO, hardened
#2/#3) et **doivent** être confirmées au kickoff avant tout code :

| Arbitrage | Recommandation design durci | À confirmer PO |
|---|---|---|
| **A0-1 — Ordre vs carry sharding S78** | Le sprint est numéroté **S79** mais le PO a demandé de le **DÉMARRER à la prochaine session**. Il PRÉCÈDE potentiellement le carry P1 sharding (S78 = orchestrateur session in-vivo + benchmark live + 4 carries 3/3, objectif phare PROVISIONAL distinct). La capacité anime/daisyUI est **techniquement orthogonale** (0 dépendance sharding↔authoring). **NE PAS trancher l'ordre dans ce plan** — le signaler comme arbitrage PO. | Démarrer S79 maintenant et différer S78 ? OU fermer S78 d'abord ? Décision PO pure. |
| **A0-2 — Phase 0 = quel audit gate ?** | `Phase 0` = audit gate du sprint **réellement fermé** au moment du démarrage : **S77** si S78 n'est pas joué, **S78** s'il l'est avant. Le plan écrit Phase 0 de façon agnostique ; le sprint-N audité est fixé au boot selon A0-1. | Quel sprint précède réellement S79 ? |

Aucune autre `prior_decision` n'est rouverte. Toutes les 8 questions du design durci sont tranchées
(emplacement `docs/factory/knowledge/`, snapshot `daisyUI 5.5.23 / Tailwind 4.3.1 / anime 4.5.0`,
contrat CSP = import `BLOB_SERVE_CSP`, kind = `app-authoring`, pas de wrapper `.claude/skills` au 1er jet,
gate **BLOQUANT** dès l'introduction).

---

## §1 État vérifié à l'entrée (à remplir au kickoff)

| Suite | Count attendu (S77 closed) | Commande | Observed |
|---|---|---|---|
| Rust nextest (Win) | 1949 | `cargo nextest run --workspace --locked` | |
| Rust nextest (Docker canonique) | 1953 (+4 `#[cfg(unix)]`) | `docker run sbfb-ci ... cargo nextest run --workspace --locked` | |
| Rust doctests | ok | `cargo test --workspace --locked --doc` | |
| cargo fmt | 0 diff | `cargo fmt --all --check` | |
| cargo clippy | 0 warnings | `cargo clippy --workspace --all-targets --locked -- -D warnings` | |
| release build | ok | `cargo build -p nexus-shell-daemon --release` | |
| sbfb-factory tests | inclus nextest | `cargo nextest run -p sbfb-factory --locked` | |
| Vitest | 411 | `(cd web && npm run test:unit)` | |
| Vitest coverage | ≥ seuils 85/85/78/85 | `(cd web && npm run test:coverage)` | |
| size-limit | 6/6 | `(cd web && npm run size)` | |
| E2E Playwright hermétique | 41 + 1 skip (`@shard`) | `(cd web && npm run test:e2e)` | |
| scan-en-strings | clean | `bash web/scripts/scan-en-strings.sh` | |
| check:csp vitrine | OK | `(cd examples/daisyui-animejs-showcase && npm run check:csp)` | |

---

## §2 Décisions Day 0 (gelées — du design durci, ne PAS re-débattre)

| D# | Décision | Implication code |
|---|---|---|
| D1 | Module de connaissance = asset de **PROCESS repo-visible** sous `docs/factory/knowledge/{animejs,daisyui}/`, JAMAIS dans `prompts/agent/` (plat) ni dans l'archive d'app. Hashé GRATUITEMENT par `provenance::compute_output_hash` (tree-walk blake3) + FG8 ; hors workspace d'app → **0 impact FG6** lock==provenance. | `docs/factory/knowledge/animejs/*` + `docs/factory/knowledge/daisyui/*` (move/create) ; `provenance.rs` INCHANGÉ. |
| D2 | Kind = **`app-authoring`** (rôle d'activité, cohérent avec PROMPT_KINDS orient-action). Ajout à `PROMPT_KINDS` + création `prompts/agent/app-authoring.md` SINON `prompt_kinds_resolve_to_existing_files` échoue. Aucun alias requis. | `process.rs:7-16` + `:888-905` ; `prompts/agent/app-authoring.md` (new). |
| D3 | Gate CSP = **re-incarnation Rust déterministe** (regex/statique, FAIL=blocage, jamais jugement LLM, FACTORY_GATES.md:205-207) qui **IMPORTE `BLOB_SERVE_CSP`** de `nexus-shell-daemon-core` comme source de vérité — JAMAIS re-hardcode, JAMAIS lit le commentaire périmé de check-csp. **BLOQUANT publish dès son introduction** (hardened #8). | `gates.rs` (new `run_gate_csp_authoring`) ; `pipeline.rs` (wiring à côté FG5/FG6) ; import `BLOB_SERVE_CSP`. |
| D4 | **Factorisation source CSP unique** : la vérité CSP est aujourd'hui dupliquée/désync à 3 endroits (`blob_serve.rs:286` canonique, `check-csp.mjs:3-12` commentaire périmé+incomplet, docstring `http.rs:234`). Source unique = constante Rust exportée + manifeste de règles ; check-csp.mjs ET gate Rust consomment ce manifeste ; test cross-crate prouvant que NETWORK couvre toutes les directives `'none'`. | `nexus-shell-daemon-core/src/blob_serve.rs` (export + ruleset) ; `gates.rs` ; `check-csp.mjs` ; test cross-crate. |
| D5 | **Corrections check-csp confirmées (GAP)** : ajouter `form-action 'none'` (anti-exfiltration `<form action=>`) + `base-uri 'none'` (`<base href>`) + `object-src 'none'` (`<object>/<embed>`) + `frame-src 'none'` (iframes imbriquées) ; corriger le drift commentaire `esm→umd` (l.12 dit `anime.esm.js`, le code l.75 lit `vendor/anime.umd.js` — le code est la vérité). | `check-csp.mjs:12,23-37` ; `gates.rs` (mêmes règles). |
| D6 | Copilote = `assemble_prompt` prepend bloc capacité UI **avant** dispatch `ExecutionTarget::Ollama` keyless (après `SENSITIVE_ACTIONS`). **JAMAIS pi-ai/SDK direct** ; `provider_router` INCHANGÉ. Connaissance CONSOMMÉE/AFFICHÉE, jamais autoritaire (artifact-draft anti-PASS préservé, `chat_history_authoritative=false`). | `llm_bridge.rs:61-93`. |
| D7 | Snapshot **figé** (pas les carets) : daisyUI `5.5.23`, Tailwind `4.3.1` (`tailwindcss`+`@tailwindcss/cli`+`@tailwindcss/node`+`@tailwindcss/oxide`), anime.js `4.5.0`. Theme défaut same-origin = `sbfb-reflect` (oklch dark custom). Template **lean** : retirer les 8 thèmes built-in. Build = `tailwindcss -i src/input.css -o app.css --minify` avec `@import "tailwindcss" source(none)` + `@source` explicites. Vendorisation UMD classic-script JAMAIS `type=module` (CORS impossible en origine opaque sous COEP require-corp). | `templates/daisyui/*` (new) ; `template_engine.rs:170-203,90-126`. |
| D8 | **Pas de wrapper `.claude/skills/`** au 1er jet (additif, différable ; hardened #7). Le prompt-kind portable suffit (déjà `strip_cloud_references` local). | — |
| D9 | Pré-launch : nouveau prompt-kind + champ context-pack additif + gate additif + raw-op inchangé = **0 bump wire, 0 dépendance nouvelle** (anime/daisyUI sont devDependencies build-time du template, runtime archive = 0 dép). | — |

---

## §3 Graphe de dépendances inter-phases

```
0 (audit gate sprint précédent — BLOQUANT)
   │
   ▼
A (promotion pack anime.js + MANIFEST, data) ──┐
   │                                            │
   ▼                                            ▼
B (prompt-kind app-authoring) ──────────► C (injection context-pack + routing zone UI)
   │                                            │
   │   D6 (factorisation CSP source unique + corrections check-csp) ◄── prérequis DUR de E
   ▼                                            │
D (gate CSP déterministe Rust BLOQUANT) ◄───────┘  [consomme la source unique D6]
   │
   ▼
E (pack daisyUI — extension knowledge + prompt-kind)
   │
   ▼
F (copilote Ollama + starter template daisyui vendoré)
   │
   ▼
G (self-check runtime viewer + confirmation BLOB_SERVE_CSP == contrat + T1/T2)
   │
   ▼
H (docs-contract closure — mirror S77 Phase N : Diataxis docs/factory + llms.txt + WIRING_SPEC
   + include! example + check-factory-docs.sh BLOQUANT 3 surfaces + canon PATTERNS §P / AGENT_SYSTEM
   + wrap-up final SPRINT_LOG/CLAUDE.md/sprint80_audit_plan/memory)
```

**Note** : la factorisation CSP (source unique D4/D5) est intégrée **dans la Phase D** (elle conditionne
le gate). Le pack daisyUI **existe déjà partiellement** (`knowledge/daisyui/` : MANIFEST + 4 couches,
68 composants, 0 risk) — Phase E le **promeut + complète + câble** dans le prompt-kind, ce n'est pas une
extraction from-scratch.

## §3bis Cadence docs-contrat (doctrine « contrat-pour-LLM », appliquée à S79)

Référence : `.planning/research/doctrine_contrat_pour_llm.md` §2 (5 couches CODE / ÉTIQUETTE /
COMMIT / GUIDE+llms.txt / arête provenance) + §3 (règle de cadence). **S79 est la 1re instance
concrète** de cette cadence. Deux régimes, additifs, sans alourdir les phases :

- **ÉTIQUETTE générée drift-gated → PAR PHASE**, livrée DANS LE COMMIT de la primitive de
  frontière (gratuite — le schéma/hash est généré ; la gate ne peut pas pourrir : drift → build
  rouge). Pour S79 :
  - **A / E** : `MANIFEST.json` + test de re-calcul blake3 par couche == MANIFEST (étiquette du
    knowledge pack). [A : **LIVRÉ** `9297f08` — `tests/animejs_manifest.rs`.]
  - **B** : prompt-kind `app-authoring` + invariant `prompt_kinds_resolve_to_existing_files`
    (contrat kind↔fichier : build cassé si kind sans `.md`).
  - **C** : champ context-pack `authoring_knowledge{path,hash}` + test hash-recompute.
  - **D** : **SOURCE CSP UNIQUE** = manifeste de règles dérivé de `BLOB_SERVE_CSP` + **test
    cross-crate anti-drift** (l'étiquette CSP générée — cœur « étiquette générée drift-gated »).
- **GUIDE + `llms.txt` (synthèse) → UNE phase de clôture = Phase H** (l'image complète n'est
  figeable qu'à la fin ; mirror EXACT de S77 Phase N). **PAS** « une phase de doc par phase ».

**Arête de provenance in-code (rang-1)** : chaque primitive de frontière porte un commentaire
`// Sprint 79 Phase X · décision D#` pointant **uniquement vers du passé immuable** (sprint /
phase / décision qui ont eu lieu), **JAMAIS une promesse future** (anti STALE-PHASE-K, cf.
doctrine §2 + l'incident réel `http.rs:2111` « lands in Phase K »). Gaté par le source-ref-check
de `check-factory-docs.sh` (Phase H).

---

## §4 Gate de testabilité par-sprint (README §4 — NON NÉGOCIABLE)

Le sprint n'est DONE que si T1 + T2 sont verts au wrap-up (Phase G) et T1 court en CI à chaque push.

### T1 — E2E hermétique BLOQUANT (Playwright + harness Rust)

**Ce qui est testé concrètement pour cette capacité** (deux volets, tous deux hermétiques, 0 réseau,
0 matériel — purement déterministe car le gate CSP est statique) :

- **T1a — La connaissance est imprimée** : `sbfb-factory process prompt --kind app-authoring --provider claude`
  imprime le bloc de maîtrise (synthesis distillé + 9 pièges CSP durs + doctrine vendorisation UMD +
  pointeurs hash vers `docs/factory/knowledge/{animejs,daisyui}/`). Variante `--provider local` imprime
  la même connaissance, profondeur réduite (`strip_cloud_references`). **Assert** : la sortie contient les
  marqueurs de pièges (`box-shadow STATIQUE`, `motion-path cx=0`, `morphTo mono-trace`,
  `prefers-reduced-motion → état-final`, `UMD classic-script jamais type=module`) ET le couplage mécanique
  `prompt_kinds_resolve_to_existing_files` est vert (FAIL build si kind sans fichier). Test Rust
  `cargo nextest -p sbfb-factory`.
- **T1b — Gate CSP : FAIL sur app dirty / PASS sur app clean** (le cœur du gate de testabilité). Fixtures
  workspace **clean** (template daisyui vendoré same-origin : `app.css` link, `vendor/anime.umd.js` classic
  script) → `run_gate_csp_authoring` **PASS**. Fixtures **dirty** (chacune une violation : `fetch(`,
  `<script src=https://...`, `@import url(`, `url(https://...` dans CSS hors allowlist, `new Worker`,
  `<form action=...`, `<base href=...`, `<object>`, `type=module`) → **FAIL déterministe**, publish bloqué,
  message d'erreur nomme la directive violée. Test Rust fixtures clean/dirty miroir + test cross-crate
  (NETWORK couvre toutes les directives `'none'` de `BLOB_SERVE_CSP`).
- **T1c — E2E front (si copilote/Operator touché Phase F)** : spec Playwright hermétique non-taguée
  (BLOQUANT, miroir du pattern `compute-shard.spec.ts`) vérifiant l'intention UX lisible de l'Operator
  (« Donner à l'agent la maîtrise UI/animation pour cette app ») sans jargon `kind/provider`, sentinelle
  DOM négative (pas de fuite de secret/token). Tagué hermétique, court en CI.

### T2 — Acceptance artefact JSON machine-lisible

**Harness** `scripts/acceptance/app_authoring_capability.sh` produisant un artefact JSON
`{ "status": "PASS" | "BLOCK" | "RIG-ABSENT", ... }` (jamais un `DIFFÉRÉ-matériel` en prose). Pour cette
capacité, **aucun matériel requis** (gate statique + prompt déterministe), donc le résultat attendu est
**PASS** (RIG-ABSENT n'est pas applicable). Champs : `prompt_app_authoring_emitted` (bool), `pieges_csp_count`
(9 attendus), `csp_gate_clean_pass` (bool), `csp_gate_dirty_fail` (liste des directives détectées),
`manifest_versions` (`{daisyui:5.5.23, tailwindcss:4.3.1, animejs:4.5.0}`), `manifest_hash_recompute_ok`
(provenance blake3 re-calculé == MANIFEST), `template_build_app_css_present` (bool),
`blob_serve_csp_equals_contract` (bool, Phase G). `BLOCK{diagnosis}` si une assertion échoue.

---

## Phase 0 — Audit gate du sprint précédent (BLOQUANT)

**Objectif** : exécuter l'audit gate du sprint **réellement clos** au démarrage (S77 par défaut, S78 si
joué avant — cf. §0 A0-2). Pattern Phase 0 permanent depuis Sprint 7 : gate BLOQUANT P0/P1.
**Livrables** :
- Lire `sprint{N}_audit_plan.md` (le `Track Testabilité standing` + les carries 3/3 routés).
- Agent `nexus-audit-gate` (Cas A) → `audit_findings.md` : verdict PASS / CONDITIONAL PASS / FAIL.
- Tout P0/P1 ouvert = **absorbé comme phase de S79** (jamais ignoré) ; P2/P3 routés ou notés carry.
- Si le sprint précédent est S77 : noter que le **carry P1 sharding (RIG-ABSENT T2)** reste ouvert et
  n'est PAS adressé par S79 (orthogonal) — il reste pour S78.

**Fichiers touchés** : `.planning/active/audit_findings.md` (artefact agent) ; aucun code sauf si
P0/P1 fix absorbé.
**Gates** : verdict audit gate explicite avant Phase A. Aucun G8/Codex (phase d'audit, pas de code par
défaut).
**Delta tests** : 0 (sauf fix P0/P1 absorbé → tests du fix).
**Commit** : pas de commit dédié si PASS sans fix (audit = artefact planning) ; si fix absorbé →
`fix(scope): Sprint 79 Phase 0 — <fix carry>`.

---

## Phase A — Promotion du pack anime.js + MANIFEST (data, 0 code Rust)

**Objectif** : faire du pack anime.js v4.5 **existant** un asset de PROCESS Factory partagé, hashé
gratuitement par provenance. Réutilisation SANS réécriture.
**Livrables** :
- Déplacer/copier `examples/daisyui-animejs-showcase/knowledge/{primitives,examples-bank,docs,synthesis}.json`
  + `*.md` + `anime-types.d.ts` (5 couches, 93 primitives toutes annotées `sbfb_csp.usable`, 52 démos,
  419 pages doc, 70 types) vers `docs/factory/knowledge/animejs/`.
- `docs/factory/knowledge/animejs/MANIFEST.json` : `version=4.5.0`, `date=2026-06-23`, hash blake3 des 5
  couches, table verdict CSP (déjà présente dans les couches), champ de fraîcheur.
- Asset repo-visible **hors workspace d'app** (statut « dev-only jamais publié dans l'archive »,
  README knowledge:3) → 0 impact FG6/FG8.

**Fichiers touchés** : `docs/factory/knowledge/animejs/*` (move + MANIFEST.json new) ; `provenance.rs`
INCHANGÉ (tree-walk récursif le hashe automatiquement).
**Gates** : G8 preflight Phase A (vérifier que `docs/factory/` existe via FACTORY_GATES.md, `knowledge/`
absent) ; review ; Codex.
**Delta tests** : +1 Rust (test re-calcul hash MANIFEST == provenance blake3) ou couvert par T2.
**Commit** : `feat(factory): Sprint 79 Phase A — promotion pack anime.js + MANIFEST knowledge`

---

## Phase B — Prompt-kind `app-authoring` (1er geste, le plus direct)

**Objectif** : surfacer la maîtrise anime.js CSP-annotée à TOUT agent fabriquant une app, vendor-neutre.
**Livrables** :
- `prompts/agent/app-authoring.md` : synthesis distillé (~64 KB max → condensé : 26 cross_products,
  11 novelty_levers + how_to_push, novelty_heuristic 5-dim) + **9 pièges CSP durs verbatim**
  (motion-path `cx=0`, glow box-shadow STATIQUE `::after` opacity-only, SVG `var(--color-*)`,
  morphTo mono-trace même type, `prefers-reduced-motion → branche état-final` revert/seek/utils.set +
  garde-fou CSS `0.001ms !important`) + **doctrine vendorisation UMD** (classic-script `window.anime`,
  JAMAIS `type=module`/ESM/CDN) + **pointeurs chemin+hash** vers les couches lourdes
  (`docs.json` 781 KB, `primitives.json` 314 KB) chargées en `depth=deep`.
- Ajout `"app-authoring"` à `PROMPT_KINDS` (`process.rs:7-16`).
- Le test de couplage `prompt_kinds_resolve_to_existing_files` (`process.rs:888-905`) garantit
  mécaniquement l'existence du `.md` (FAIL build sinon). Étendre l'assertion si besoin.
- 0 exécution, rollback = supprimer 1 `.md` + 1 entrée de tableau.

**Fichiers touchés** : `crates/sbfb-factory/src/process.rs:7-16` + `:888-905` ;
`prompts/agent/app-authoring.md` (new).
**Gates** : G8 preflight Phase B ; review ; Codex.
**Delta tests** : +1 (couplage kind ; ou réutilise l'invariant existant) ; **T1a** devient runnable ici.
**Commit** : `feat(factory): Sprint 79 Phase B — prompt-kind app-authoring (anime.js CSP-annoté)`

---

## Phase C — Injection context-pack + routing zone UI

**Objectif** : surfaçage AUTOMATIQUE au bootstrap de session et selon la zone fonctionnelle, 0 nouvelle
autorité.
**Livrables** :
- Champ additif `authoring_knowledge {path, hash blake3 via file_hash}` dans `handle_context_pack`
  (`operator_server.rs:355-427`, modèle exact = `process_docs` `:404-409`, ~1 ligne `file_hash` par doc)
  pointant `docs/factory/knowledge/{animejs,daisyui}/MANIFEST.json` → session fraîche reçoit la matrice
  CSP-annotée vérifiable-par-recalcul au même rang que base/universal/handoff.
- `handle_chat_session` (`:648-700`) hérite via le même pack ; `chat_history_authoritative=false` préservé.
- Zone `UI/animation/design app SBFB` ajoutée aux **Routing-tables** des 2 SKILL.md
  (`.claude/skills/nexus-phase-preflight/SKILL.md` + `nexus-phase-review/SKILL.md`) → toute phase/app
  touchant le front surface AUTOMATIQUEMENT la contrainte CSP + la maîtrise (comme la zone `lib externe`
  pointe vers context7).

**Fichiers touchés** : `crates/sbfb-factory/src/operator_server.rs:355-427,648-700` ;
`.claude/skills/nexus-phase-preflight/SKILL.md` + `.claude/skills/nexus-phase-review/SKILL.md`.
**Gates** : G8 preflight Phase C ; review ; Codex.
**Delta tests** : +2 (context-pack inclut `authoring_knowledge` + hash recalcul match).
**Commit** : `feat(factory): Sprint 79 Phase C — injection context-pack authoring_knowledge + routing zone UI`

---

## Phase D — Gate CSP déterministe Rust BLOQUANT + factorisation source unique

**Objectif** : PROUVER mécaniquement la conformité sandbox de l'app authored (pas la documenter).
**BLOQUANT publish dès son introduction** (hardened #8).
**Livrables** :
- **Factorisation source CSP unique (D4/D5)** : exporter `BLOB_SERVE_CSP` (déjà `blob_serve.rs:286`) +
  un **manifeste de règles machine-lisible** dérivé d'elle (les directives + leur traduction en regex de
  détection). `check-csp.mjs` ET le gate Rust consomment ce manifeste plutôt que de re-dériver. Corriger
  immédiatement le commentaire `check-csp.mjs:12` (`esm→umd`) et compléter NETWORK avec `form-action`/`base-uri`.
- `run_gate_csp_authoring(workspace) -> GateResult` en Rust (`gates.rs`, modèle `run_gate_fg5_sandbox`) :
  - **IMPORTE `BLOB_SERVE_CSP`** de `nexus-shell-daemon-core` (source de vérité, JAMAIS re-hardcode,
    JAMAIS lit le commentaire périmé).
  - Reprend les **13 regex NETWORK du CODE** `check-csp.mjs:23-37` (fetch/XHR/WebSocket/EventSource/
    sendBeacon = connect-src ; Worker/SharedWorker/importScripts/serviceWorker = worker-src ;
    remote link/script/`@import`/`url()` = default-src).
  - **3 tiers** : authored `{index.html,app.js}` = 0 http(s) absolu + 0 NETWORK ;
    compiled `{app.css}` = 0 NETWORK + chaque URL absolue dans `CSS_URL_ALLOW`
    `{http://www.w3.org/2000/svg, http://www.w3.org/1999/xlink, https://tailwindcss.com}` ;
    vendored `{vendor/anime.umd.js}` = 0 NETWORK live.
  - **AJOUTE les règles manquantes (GAP)** : `form-action 'none'` (`<form action=>` + `action=` dynamiques),
    `base-uri 'none'` (`<base href>`), `object-src 'none'` (`<object>/<embed>`), `frame-src 'none'`
    (iframes imbriquées) ; valider `app.css` chargé en `<link rel=stylesheet href=relatif>` et `vendor/*.js`
    en classic `<script src>` same-origin JAMAIS `type=module`.
  - Tree-walk `WalkDir` (comme FG5) pour détecter les assets runtime dynamiquement.
- **Wiring pipeline** : insérer `run_gate_csp_authoring` dans `run_publish_pipeline` (`pipeline.rs:15`)
  à côté de FG5/FG6, **FAIL = publish bloqué** (`return Err(...)` comme FG5/FG6).
- **Test cross-crate** : asserter que la liste NETWORK couvre toutes les directives `'none'` de
  `BLOB_SERVE_CSP` (anti-drift futur).
- Doc `docs/factory/FACTORY_GATES.md` : nouveau gate (FG-CSP-authoring) + rationale anti-exfiltration.

**Fichiers touchés** : `crates/sbfb-factory/src/gates.rs` (new `run_gate_csp_authoring`) ;
`crates/sbfb-factory/src/pipeline.rs:15` (wiring) ;
`crates/nexus-shell-daemon-core/src/blob_serve.rs:286` (export + ruleset) ;
`examples/daisyui-animejs-showcase/scripts/check-csp.mjs:12,23-37` (corrections + consomme manifeste) ;
`docs/factory/FACTORY_GATES.md`.
**Gates** : G8 preflight Phase D (vérifier `BLOB_SERVE_CSP` chaîne exacte `blob_serve.rs:286` + GAP
form-action/base-uri confirmé) ; review (CSP non-négociable, jamais ML) ; Codex.
**Delta tests** : +6 à +10 (fixtures clean PASS + fixtures dirty FAIL par directive : fetch, script src
https, @import, url() hors allowlist, Worker, form action, base href, object, type=module + test
cross-crate couverture directives). **T1b** runnable ici (cœur gate de testabilité).
**Commit** : `feat(factory): Sprint 79 Phase D — gate CSP déterministe Rust BLOQUANT + source CSP unique`

---

## Phase E — Pack daisyUI (promotion + complétion + câblage prompt-kind)

**Objectif** : intégrer l'équivalent daisyUI au module de connaissance, ancré code source, voie build-time
vendorée. **NB** : le pack existe déjà partiellement (`knowledge/daisyui/` MANIFEST + 4 couches,
68 composants tous CSP-usable, 0 risk) — Phase E le **promeut, complète et câble**, pas une extraction
from-scratch.
**Livrables** :
- Promouvoir `examples/daisyui-animejs-showcase/knowledge/daisyui/` (components.json/COMPONENTS.md,
  theming.json oklch tokens, synthesis.json compositions composant+anime CSP-safe, docs-llms.txt,
  MANIFEST.json `daisyui 5.5.23 / tailwindcss 4.3.1 / animejs 4.5.0`) vers
  `docs/factory/knowledge/daisyui/`.
- Compléter la couche `classes-bank.json` (blocs HTML verbatim + `sbfb_reusable{ok,why}`, gabarit
  examples-bank anime) si absente, et **valider explicitement le verdict CSP par classe** des cas à risque
  `url()`/`@apply`/`backdrop-filter`/`mask`/SVG-fill (`fill-*`/`stroke-*` Tailwind ne compilent pas dans
  l'iframe → peindre en `var(--color-*)`/`color-mix(in oklch)`).
- MANIFEST.json recalculé (hash blake3 des couches, vérifiable par provenance).
- Étendre `prompts/agent/app-authoring.md` avec la maîtrise daisyUI (recette build `tailwindcss -i
  src/input.css -o app.css --minify` avec `@import source(none)` + `@source` explicites ; thème défaut
  `sbfb-reflect` oklch dark ; template lean sans 8 thèmes built-in).

**Fichiers touchés** : `docs/factory/knowledge/daisyui/*` (move + complétion) ;
`prompts/agent/app-authoring.md` (extend).
**Gates** : G8 preflight Phase E (vérifier verdict CSP par classe sur les cas à risque) ; review ; Codex.
**Delta tests** : +2 (hash MANIFEST daisyui recalcul + prompt app-authoring contient marqueurs daisyUI).
**Commit** : `feat(factory): Sprint 79 Phase E — pack daisyUI knowledge + extension prompt-kind`

---

## Phase F — Copilote Ollama keyless + starter template daisyui vendoré

**Objectif** : matérialiser la capacité dans l'archive livrée + copilote d'authoring.
**Livrables** :
- **Copilote** : bloc capacité UI prepend dans `assemble_prompt` (`llm_bridge.rs:61-93`) AVANT le message
  utilisateur, dispatch `ExecutionTarget::Ollama` **keyless** APRÈS le gate `SENSITIVE_ACTIONS`. JAMAIS
  pi-ai/SDK direct ; `provider_router` INCHANGÉ. Connaissance affichée jamais autoritaire (anti-PASS).
- **Starter template** : 5e `TemplateConfig` `daisyui` (`template_engine.rs:170-203,90-126`) →
  `templates/daisyui/*` (new) :
  - `index.html` : `<html data-theme="sbfb-reflect">`, `<link rel=stylesheet href=app.css>` (classic),
    `<script src=vendor/anime.umd.js>` (classic, AVANT app.js), JAMAIS `type=module`/CDN.
  - `src/input.css` : `@import "tailwindcss" source(none); @source "../index.html"; @source "../app.js";`
    + thème `sbfb-reflect` inline oklch (template **lean**, 8 thèmes built-in retirés). Pas de
    `tailwind.config.js`.
  - `package.json` build-time devDeps figés (`daisyui 5.5.23`, `tailwindcss 4.3.1`, `@tailwindcss/cli/node/oxide 4.3.1`),
    script `build:css` + `vendor:anime` ; runtime archive = **0 dépendance**.
  - `README.md` CSP + doctrine vendorisation ; décline le pattern react UMD no-build.
- Le template `daisyui` passe **FG5/FG6 + gate CSP-authoring** (clean) → fixture clean de T1b.

**Fichiers touchés** : `crates/sbfb-factory/src/llm_bridge.rs:61-93` ;
`crates/sbfb-factory/src/template_engine.rs:170-203,90-126` ;
`crates/sbfb-factory/src/templates/daisyui/*` (new).
**Gates** : G8 preflight Phase F (vérifier arm Ollama keyless inchangé + SENSITIVE_ACTIONS ordre) ;
review ; Codex.
**Delta tests** : +4 à +6 Rust (template daisyui crée structure attendue + passe gate CSP clean +
assemble_prompt prepend bloc capacité) ; +front si Operator UX touché (T1c).
**Commit** : `feat(factory): Sprint 79 Phase F — copilote Ollama keyless + starter template daisyui vendoré`

---

## Phase G — Self-check runtime viewer + confirmation CSP daemon + testabilité (T1/T2)

**Objectif** : filet RUNTIME pour les violations CSP construites à l'exécution (échappent au lint statique :
`url()`/`@font-face` dynamiques) + clôture du sprint avec gate de testabilité vert.
**Livrables** :
- **Self-check runtime SANS Electron** : rejeu de l'app dans le viewer iframe `blob_serve` sous la CSP
  RÉELLE de prod (`connect-src 'none'`, origine opaque, COEP require-corp) via postMessage — complément
  runtime du lint statique. Socle `tools/factory-ui/src/readonly` (réutilisé Viewer + Operator).
- **Confirmation `BLOB_SERVE_CSP` daemon == contrat de gate** : vérifier que la chaîne servie par
  `nexus-shell-daemon` (`blob_serve.rs:286`, posée sur CHAQUE réponse y compris 404 par
  `blob_serve_csp_middleware`) est bien celle importée par le gate Phase D (sinon le lint protège une CSP
  fictive). Champ `blob_serve_csp_equals_contract` de T2.
- **T1** (E2E hermétique BLOQUANT) vert + tagué CI ; **T2** (artefact JSON acceptance) = **PASS**
  (`scripts/acceptance/app_authoring_capability.sh`).
- Docs gate-spécifiques (le reste de la doc = **Phase H** clôture) : `docs/rust/PATTERNS.md`
  (§ gate CSP-authoring + source CSP unique), `docs/factory/FACTORY_GATES.md` (finalisé, nouveau
  FG-CSP-authoring). [SPRINT_LOG row S79 / CLAUDE.md état / `nexus_grid_pivot.md` + `MEMORY.md` /
  `sprint80_audit_plan.md` → **Phase H** (clôture finale, après les docs-contract).]

**Fichiers touchés** : `tools/factory-ui/src/readonly/*` ;
`crates/nexus-shell-daemon-core/src/blob_serve.rs` (consommé, confirmé) ;
`docs/factory/FACTORY_GATES.md` ; `docs/rust/PATTERNS.md` (§ gate CSP) ;
`scripts/acceptance/app_authoring_capability.sh` (new) ;
`web/e2e/app-authoring.spec.ts` ou équivalent (T1). [SPRINT_LOG / CLAUDE.md / sprint80_audit_plan → Phase H.]
**Gates** : G8 preflight Phase G ; review Workflow multi-dimensions ; Codex ; gate de testabilité
par-sprint vert. [Le gate dual-platform AVANT push est porté par la **Phase H** = phase de clôture.]
**Delta tests** : +front (self-check viewer) + T1 E2E + T2 artefact JSON.
**Commit** : `feat(factory): Sprint 79 Phase G — self-check runtime + confirmation CSP daemon + testabilité T1/T2`

---

## Phase H — Docs-contract closure Factory (couche GUIDE + llms.txt, mirror EXACT S77 Phase N)

**Objectif** : le nœud GUIDE de la doctrine contrat-pour-LLM (`.planning/research/doctrine_contrat_pour_llm.md`
§2/§3) — synthèse navigable LLM + humaine de la capacité app-authoring, figée à la clôture car
l'image complète n'est figeable qu'à la fin. Mirror EXACT de S77 Phase L/M/N (que la cadence
§3bis a déjà dispersé en étiquettes-par-phase ; H ne fait QUE la synthèse + le doc-lint + la
canonisation + le wrap-up). **Aucune nouvelle primitive de code** ; additif, 0 bump wire.
**Livrables** :
- **HUMAIN — Diataxis FR** : `docs/factory/{README,EXPLANATION,HOW_TO_WIRE,REFERENCE}.md` pour la
  capacité app-authoring (anime.js + daisyUI + gate CSP `run_gate_csp_authoring` + vendorisation UMD
  same-origin). FR pour les 3 premiers ; `REFERENCE.md` **corps EN** (agent-facing, exempté
  french-body, comme S77).
- **LLM/agent** : `docs/factory/llms.txt` (format llmstxt.org, Truth-Stack
  `repo files > .planning/active/ > commits > prompts > chat` + règle « Not evidenced ») +
  **entrée racine `llms.txt`** indexant `docs/factory/llms.txt` (le root llms.txt existe déjà,
  scope sharding — ajouter la section factory).
- **`docs/factory/WIRING_SPEC.md`** (EN contract-dense, sections fixes) : source_refs `path:Symbol`
  vers les primitives de frontière S79 — au minimum `PROMPT_KINDS`, `app-authoring`,
  `run_gate_csp_authoring`, `BLOB_SERVE_CSP`, `authoring_knowledge`, `TemplateConfig` (daisyui),
  `handle_context_pack`, le `MANIFEST.json` + son test. Required-anchor allowlist sur ces symboles.
- **Exemple runnable lifté VERBATIM via `include!`** (modèle S77 `examples/sign_verify.rs` →
  `crates/nexus-core-rs/tests/shard_sign_verify.rs`) : un snippet d'authoring CSP-clean
  (`docs/factory/examples/*`) `include!`-é dans un test nextest → l'exemple ne peut pas mentir
  (build rouge s'il drift).
- **`scripts/check-factory-docs.sh`** — CLONE de `scripts/check-sharding-docs.sh` :
  (1) link-check repo-relatif ; (2) **source-ref-check rank-1** (chaque `path:Symbol` : fichier
  existe ET symbole grep-trouvé, sinon EXIT 1) ; (3) **required-anchor allowlist** ;
  (4) **honesty-gate** (marqueurs `PROVISIONAL` / `Not evidenced` + **caveat cardinal** Factory =
  « lint statique CSP ≠ garantie runtime complète [self-check viewer requis] ; connaissance
  CONSOMMÉE jamais autoritaire [0 verdict PASS] ») ; (5) **french-body** (REFERENCE.md exempté).
  **CÂBLÉ BLOQUANT sur 3 surfaces** (exactement comme check-sharding-docs) : `.github/workflows/ci.yml`
  + `.woodpecker/ci-linux.yml` + `scripts/verify.sh`.
- **Honnêteté (non négociable)** : aucun « shipped/LIVE » faux ; marquer `PROVISIONAL` ce qui ne
  tourne pas in-vivo ; Truth-Stack `repo > planning > commits > prompts > chat`.
- [Le **CANON dans le process Claude Code** (README + AGENT_SYSTEM + PATTERNS §P) + le **gate-map
  GÉNÉRIQUE** + le **wrap-up final** sont portés par la **Phase I** : H ne livre QUE l'instance
  Factory de la couche GUIDE + son doc-lint factory-scopé.]

**Fichiers touchés** : `docs/factory/{README,EXPLANATION,HOW_TO_WIRE,REFERENCE,llms.txt,WIRING_SPEC.md}` ;
`docs/factory/examples/*` ; `llms.txt` (racine, section factory) ; `scripts/check-factory-docs.sh` (new) ;
`.github/workflows/ci.yml` + `.woodpecker/ci-linux.yml` + `scripts/verify.sh` (câblage check-factory-docs) ;
`crates/sbfb-factory/tests/*` (test `include!` de l'exemple).
**Gates** : G8 preflight Phase H ; review Workflow ; Codex ; `check-factory-docs.sh` **vert**.
**Delta tests** : +1 Rust (example `include!` runnable) + 1 script doc-lint net-new (`check-factory-docs.sh`).
**Commit** : `docs(factory): Sprint 79 Phase H — couche agent llms.txt + WIRING_SPEC + Diataxis Factory`

---

## Phase I — Canon de la cadence docs-contrat DANS LE PROCESS Claude Code + gate-map générique + wrap-up

**Objectif** : la directive PO va **plus loin que Factory** — la cadence docs-contrat devient une
**règle du process Claude Code SBFB lui-même**, appliquée à TOUT sprint/phase futur (pas seulement
S79). S79 en est la **1re instance de référence** ; Phase I la **canonise** et l'**outille
génériquement**. Doctrine §6 (« où ça doit vivre ») + §7/§10 (gate-map + trous réels).
**Livrables** :
- **Process source-of-truth** : `docs/claude/README.md` — nouvelle **convention de cadence**
  docs-contrat (étiquette générée drift-gated PAR PHASE dans le commit de la primitive ; GUIDE +
  `llms.txt` en UNE phase de clôture ; arête de provenance rang-1 vers le passé immuable, jamais
  une promesse). Standing rule = chaque futur sprint la suit.
- **Pattern nommé** : `docs/rust/PATTERNS.md` nouveau §P « cadence docs-contrat » (les 5 couches +
  leçon L/M/N S77 + anti STALE-PHASE-K) ; miroir shell `docs/shell/PATTERNS.md` si pertinent.
- **Généralisation portable** : `docs/agent/AGENT_SYSTEM.md` (la doctrine portable, S79 = 1re
  instance ; consommée jamais autoritaire).
- **GATE-MAP GÉNÉRIQUE** `scripts/check-frontier-contracts.sh` (doctrine §7/§10), **câblé BLOQUANT
  CI 3 surfaces** (`.github/workflows/ci.yml` + `.woodpecker/ci-linux.yml` + `scripts/verify.sh`),
  `set -euo pipefail`, BusyBox-safe (modèle `check-sharding-docs.sh`). Cœur livrable S79 (le reste
  = carry honnête) :
  - **Anti STALE-PHASE-K source-ref GÉNÉRIQUE repo-wide** : grep `lands in Phase [A-Z]|will
    (populate|expose|add|read|land)` → **FAIL** si la phase promise est close (le trou EDGE réel :
    ~356 commentaires de provenance non gatés aujourd'hui ; `http.rs:2111` « lands in Phase K » LIVE).
  - **Couverture étiquette** sur un **registre explicite `// FRONTIER: <name> domain=… version=…`**
    (arbitrage doctrine §7 Q2 — registre explicite vs convention : à trancher au preflight Phase I) ;
    **FAIL** si un type `// FRONTIER:` n'a ni snapshot schéma ni exemption `// FRONTIER-NO-SCHEMA:`.
  - **Honnêteté** : marqueur de statut requis sur tout doc de frontière (modèle PROVISIONAL).
- **Fix du faux-vert CI réel** (doctrine §10) : `phase-review-cross-check.yml` regex obsolète
  `[A-F]` (0 match, plafond périmé alors qu'on est ≥ Phase H/I) → régex `Phase [A-Z]+[0-9]?`.
  + cross-check `BLOB_SERVE_CSP.contains("form-action")` ajouté (les 2 tests substring laissent
  passer un drift — fermé par la Phase D, re-asserté ici en méta).
- **Carry honnête** : la couverture étiquette des **~21 familles wire** non-schématisées (doctrine
  §10) n'est PAS faite en S79 → documentée `PROVISIONAL` + routée `sprint80_audit_plan.md`. Ne PAS
  prétendre « tout est gaté ».
- **Wrap-up FINAL du sprint** : `docs/claude/SPRINT_LOG.md` (row S79), `CLAUDE.md` (état + capacité +
  cadence canonisée), `nexus_grid_pivot.md` + `MEMORY.md` (post-commit), `sprint80_audit_plan.md`.

**Fichiers touchés** : `docs/claude/README.md` ; `docs/rust/PATTERNS.md` (§P) ; `docs/shell/PATTERNS.md` ;
`docs/agent/AGENT_SYSTEM.md` ; `scripts/check-frontier-contracts.sh` (new) ; `.github/workflows/ci.yml`
+ `.woodpecker/ci-linux.yml` + `scripts/verify.sh` (câblage) ; `.github/workflows/phase-review-cross-check.yml`
(fix régex) ; `docs/claude/SPRINT_LOG.md` ; `CLAUDE.md` ; `.planning/active/sprint80_audit_plan.md`.
**Gates** : G8 preflight Phase I (trancher registre `// FRONTIER:` explicite vs convention, doctrine
§7 Q2) ; review Workflow ; Codex ; `check-frontier-contracts.sh` **vert** ; **gate dual-platform
(Win nextest + Docker canonique `sbfb-ci` rust:1.94, fmt 0 sous les 2 toolchains) AVANT push** (phase
finale) ; gate de testabilité par-sprint (T1/T2) confirmé vert.
**Delta tests** : extension doc-lint générique + éventuels tests d'auto-vérif du script ; +tests de
fix régex CI (au besoin).
**Commit** : `chore(process): Sprint 79 Phase I — canon cadence docs-contrat + check-frontier-contracts générique`

---

## §5 Invariants tenus (rappel — vérifiés à chaque review)

- **Scellage 100% Factory non-délégable** : la connaissance n'accorde AUCUNE dispense.
  CSP/COEP/COOP/FG5/FG6/FG8/Ed25519 restent souverains. Le lint authoring est **ADDITIF** ;
  FG6 lock==provenance reste vrai (tout asset vendoré hashé par `compute_output_hash`).
- **Autorité descendante process > RRV > Factory** : le module est CONSOMMÉ/AFFICHÉ, jamais une autorité
  de verdict (artifact-draft anti-PASS préservé, `chat_history_authoritative=false`).
- **Gates déterministes, pas de ML/scoring opaque** (FACTORY_GATES.md:205-207) : le gate CSP = scan
  statique regex, FAIL=blocage, jamais jugement LLM.
- **Copilote = `ExecutionTarget::Ollama` keyless loopback, JAMAIS pi-ai/SDK direct** ; `provider_router`
  INCHANGÉ ; provider (qui lit le prompt) ≠ backend exécution worker (D8 process.rs).
- **Vendorisation same-origin UMD classic-script JAMAIS ESM/CDN** ; Tailwind compile build-time en
  `app.css` same-origin ; Tailwind-CDN + Google Fonts interdits.
- **Pré-launch** : 0 bump wire, 0 dépendance runtime nouvelle, contrat 8→9 prompt-kinds étendu proprement.
- **Pas de wrapper `.claude/skills/`** au 1er jet (additif, différable).
- **Sprint ultra-complet A→I d'un bloc, 0 defer du cœur** (G = testabilité/self-check, H = docs-contract
  Factory instance, I = canon process + gate-map générique ; la dette/gates = phases du sprint).
- **Cadence docs-contrat (doctrine §3 + §3bis)** : étiquette générée drift-gated PAR PHASE dans le
  commit de la primitive ; GUIDE + `llms.txt` en UNE phase de clôture ; arête de provenance in-code
  rang-1 vers le passé immuable seulement (anti STALE-PHASE-K). **S79 = 1re instance ; la règle est
  canonisée dans le PROCESS Claude Code lui-même (README + AGENT_SYSTEM + PATTERNS + check-frontier-contracts
  générique), pas seulement dans Factory.**

## §6 Risques (du design durci) + mitigations

| Risque | Mitigation (intégrée au plan) |
|---|---|
| Poids tokens (docs.json 781 KB + primitives.json 314 KB) | Phase B : `app-authoring.md` = synthesis distillé + pièges seuls par défaut ; couches lourdes RÉFÉRENCÉES par chemin+hash, chargées en `depth=deep`. |
| Dérive lint vitrine vs gate Factory | Phase D : source CSP unique (D4) + gate paramétrable par workspace + CODE comme source (pas le commentaire) + test cross-crate anti-drift. |
| Verdict CSP par classe daisyUI subtilement faux (`url()`/`@apply`/`backdrop-filter`/`mask`/SVG-fill) | Phase E : auditer ces cas explicitement ; le gate mécanique (Phase D) attrape ce que le verdict advisory rate. |
| Fraîcheur (snapshot 2026-06-23 périmé au bump) | MANIFEST `date+version` ; re-extraction MANUELLE (pas d'auto-fetch, conforme connect-src none). |
| Gate statique insuffisante seule (`url()`/`@font-face` runtime) | Phase G : self-check runtime viewer OBLIGATOIRE (filet pour les animations qui s'exécutent dans le temps). |
| CSP réelle blob-serve ≠ contrat | Phase D importe `BLOB_SERVE_CSP` (prouvé `blob_serve.rs:286`) ; Phase G confirme l'égalité (champ T2). |
| Sur-ingénierie si mal borné | Objectif borné à la capacité-cœur prouvée (A→D) + durcissement (E→G) dans UN sprint, jamais des defers. |
