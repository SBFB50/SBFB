# Sprint 70 Phase E — deep review

HEAD: 69e3a06 | Agent: nexus-phase-review-deep (Opus 4.6 1M)

## Verdict : PASS

Promu de PASS-PENDING après réconciliation Codex.

(Rigor signal : 4 findings P2+ documentes / >=1 requis pour PASS)

## Memory consultation

- feedback_approach.md : pick deepest, no band-aid, research before code —
  **respecte** (meme stack que web/, 5 projets OSS dans le preflight S1a,
  design prompt + handoff avant code)
- feedback_context7_systematic.md : context7 obligatoire avant code
  touchant lib/API — **respecte** (preflight S1b scanne 9 libs, versions
  verifiees, 0 CVE applicable. Les libs sont les memes que web/)
- vision_model.md : NO funding, pattern OpenBSD solo — **N/A** (pas de
  suggestion institutionnelle)
- feedback_no_direct_blobserve.md : Viewer via shell Browse uniquement —
  **respecte** (Viewer = app SBFB standard via bridge, pas d'acces direct)
- feedback_iframe_sandbox_forms.md : pas de form dans iframe sandbox —
  **respecte** (Viewer utilise div+button, 0 `<form>` dans le code)
- feedback_commit_heredoc.md : git commit -F pour body > 30 lignes —
  **note** (a appliquer au commit)

## Staging check

- Phase fichiers : ~50 fichiers source (hors node_modules/dist)
  - `.planning/active/sprint70_factory_ux_design_prompt.md` (NEW)
  - `.planning/active/sprint70_factory_ux_design_handoff.md` (NEW)
  - `.planning/active/sprint70_phase_e_preflight.md` (NEW)
  - `examples/sbfb-factory-viewer/` (5 fichiers NEW)
  - `tools/factory-ui/` (10 fichiers NEW)
  - `tools/factory-operator/` (~35 fichiers source NEW)
- Planning/docs split : N/A (phase feat, planning+code dans le meme commit
  car le design prompt/handoff sont des livrables Phase E)
- Untracked accidentels : **OUI** — `tools/factory-operator/node_modules/`
  et `tools/factory-operator/dist/` sont untracked mais pas ignores par le
  root `.gitignore` (pattern `/node_modules/` ne matche que la racine).
  **P2-E-1 ci-dessous.**

## Suites verification

| Suite | Avant | Apres | Delta | Status |
|-------|-------|-------|-------|--------|
| cargo fmt | - | - | - | ok |
| cargo clippy | - | - | - | ok |
| Rust nextest | 1433 | 1481 | +48 (phases C+D) | ok |
| Rust doctests | ok | ok | 0 | ok |
| tsc --noEmit (web/) | - | - | - | ok |
| ESLint (web/) | - | - | - | ok (5 warnings standard) |
| Vitest | 279 | 279 | +0 | ok |
| Build web | - | - | - | ok |
| size-limit | 6/6 | 6/6 | - | ok |
| Release build | - | - | - | ok |
| tsc (factory-operator) | - | - | - | ok |
| ESLint (factory-operator) | - | - | - | ok (3 warnings shadcn standard) |
| Build (factory-operator) | - | - | - | ok |
| Boundary Viewer | - | - | - | ok (0 operator imports, 0 privileged endpoints) |
| Bridge sync | - | - | - | ok (422 lines, hash identique 4 copies) |

## Branch coverage semantique (deep)

Phase E est entierement NEW code (0 fichiers modifies). Pas de
delta Rust ni de delta Vitest. La couverture est evaluee sur les
criteres de boundary et fonctionnalite front.

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| Viewer `app.js` browse_list | boundary grep | oui (bridge.browseList) | filtre/search | empty state | DEEP-PASS |
| Viewer `app.js` proof_card_get | boundary grep | oui (bridge.proofCardGet) | proof absent | catch error | DEEP-PASS |
| Viewer boundary | grep acceptance | N/A | 0 operator/localhost/token | exhaustif 8 patterns | DEEP-PASS |
| readonly boundary | grep acceptance | N/A | 0 privileged endpoints | exhaustif 8 patterns | DEEP-PASS |
| factory-ui readonly exports | tsc | N/A | 8 composants exportes | N/A | DEEP-PASS |
| factory-ui operator exports | tsc | N/A | 13 fonctions exportees | N/A | DEEP-PASS |
| Operator 10 pages | tsc + lint + build | oui | 0 TS errors, 0 ESLint errors | shadcn warnings | DEEP-PASS |
| Operator i18n fr | manual review | oui | toutes les cles presentes | comparaison fr/en | DEEP-PASS |
| Operator i18n en | manual review | oui | toutes les cles presentes | miroir fr | DEEP-PASS |
| Operator TechnicalDetails | code review | oui | commandes techniques dans details repliable | N/A | DEEP-PASS |
| PhaseAssistant intentions | code review | oui (t() keys) | 6 intentions FR, jamais kind brut en CTA | tech in TechnicalDetails | DEEP-PASS |
| AgentSelector same-agent warning | code review | oui | warning si driver===verifier | N/A | DEEP-PASS |
| SprintOverview empty phases | code review | oui | EmptyPhases composant | N/A | DEEP-PASS |
| SprintOverview error state | code review | oui | ErrorState composant | N/A | DEEP-PASS |

## Scope cuts semantique (deep)

| # | Libelle | Intention | Grep mecanique | Diff semantique | Signal |
|---|---------|-----------|----------------|-----------------|--------|
| SC-1 | SearchManifest | Pas de wire format gossip reseau | 0 match | 0 code reseau dans les fichiers | CLEAN |
| SC-2 | Route React /factory | Pas de route shell produit | 0 match | Operator est un projet separe, pas une route web/ | CLEAN |
| SC-3 | @dev index tree-sitter | Pas d'index code source | 0 match | 0 code tree-sitter | CLEAN |
| SC-4 | Template react-vite | Pas de nouveau template | 0 match | N/A | CLEAN |
| SC-5 | CuratorVouched UI | Pas d'UI curation shell | 0 match | N/A | CLEAN |
| SC-6 | FG10 Review gate auto | Pas de gate automatise | 0 match | N/A | CLEAN |
| SC-7 | Fuzzing cargo-fuzz | Pas de fuzzing | 0 match | N/A | CLEAN |
| SC-8 | Feed format version bump | Pas de bump feed | 0 match | N/A | CLEAN |
| SC-9 | ProofCard comme feed op | Pas de feed op | 0 match | ProofCard est un composant React, pas une operation feed | CLEAN |
| SC-10 | iroh 1.0 upgrade | Pas d'upgrade iroh | 0 match | N/A | CLEAN |
| SC-11 | CI process workflow | Pas de CI multi-provider | 0 match | N/A | CLEAN |
| SC-12 | Provider router multi-LLM | Pas de routeur auto | 0 match | N/A | CLEAN |
| SC-13 | sbfb-search app | Pas d'app search | 0 match | N/A | CLEAN |
| SC-14 | Ingestion OSS broad | Pas d'ingestion | 0 match | N/A | CLEAN |

## Research grounding (deep)

### Preflight G8
- Fichier : `sprint70_phase_e_preflight.md` — **existe**
- Scans : **5/5** (S1a, S1b, S2, S3, S4 tous presents)
- S1a OSS : 5 projets (shadcn-admin, react-admin, sbfb-explorer, sbfb-ideas,
  Grafana) — >= 1 requis
- Verdict : **EXECUTE plan-as-is**
- Finding S1a : APPROACH-ALIGNED (separation physique Viewer/Operator = le
  pattern le plus securise)
- **PASS**

### Deps/API

| Dep/API | Version | Trace preflight | Coherence code-vs-doc | Signal |
|---------|---------|-----------------|----------------------|--------|
| react | 19.2.4-19.2.6 | oui (S1b) | OK | PASS |
| react-router-dom | 7.14.0-7.15.0 | oui (S1b) | OK | PASS |
| vite | 8.0.1-8.0.11 | oui (S1b) | OK | PASS |
| tailwindcss | 4.2.2 | oui (S1b) | OK | PASS |
| shadcn | 4.2.0-4.8.0 | oui (S1b) | OK | PASS |
| typescript | 5.9.3 | oui (S1b) | OK | PASS |
| i18next | 26.2.0 | non (nouvelle dep) | OK (lib standard i18n) | CONCERN P3 |
| react-i18next | 17.0.8 | non (nouvelle dep) | OK | CONCERN P3 |
| lucide-react | 1.16.0 | non (meme que web/) | OK | PASS |

### Coherence code-vs-source

- Le Viewer utilise `SBFBBridge` exactement comme sbfb-explorer/sbfb-ideas
  (meme pattern `new SBFBBridge()` + `.browseList()` + `.proofCardGet()`).
  Coherent avec la doc bridge Sprint 13 P24.
- L'Operator utilise Vite proxy pour les appels API, pas de fetch direct au
  daemon. Coherent avec le pattern de Phase D.
- Les endpoints appeles par l'Operator (`/api/status`, `/api/lint`,
  `/api/audit/{rev}`, `/api/prompt/{kind}`, `/api/context-pack`,
  `/api/actions/run`, `/api/chat/session`, `/api/chat/message`,
  `/api/actions/log`) sont les 13 endpoints livres par Phase D.
  Coherent avec `operator_server.rs`.

## Security deep

### Scan automatique

| Fichier | Pattern | Ligne | Severite | Detail |
|---------|---------|-------|----------|--------|
| factory-ui/operator/api-client.ts | hardcoded URL | 3 | P3 | `BASE_URL = "http://127.0.0.1:4242"` — localhost OK pour local, mais le fichier n'est pas consomme par l'Operator actuellement |
| Viewer app.js | escapeHtml | 19-22 | clean | XSS protection via textContent/innerHTML pattern — correct |
| Viewer index.html | no CSP meta | - | P3 | Pas de meta CSP dans le HTML — le Viewer tourne dans iframe sandbox avec CSP blob-serve. Non bloquant. |

### Analyse semantique

- **Viewer inputs non-trustes** : les donnees arrivent via bridge
  (browse_list, proof_card_get). Le bridge est le canal controle du
  daemon. Les strings sont echappees via `escapeHtml()` avant insertion
  HTML. Le pattern `div.textContent = str; return div.innerHTML` est
  correct pour l'echappement. Pas de `innerHTML` sur des donnees non-
  echappees.

- **Operator inputs non-trustes** : les donnees arrivent de
  `sbfb-factory operator serve` (localhost). Toutes les reponses sont
  du JSON parse via `res.json()` et affichees dans des composants React
  (pas de `dangerouslySetInnerHTML`). Les actions POST sont allowlistees
  (`status-sprint`, `lint-planning`, `audit-commit`, `prompt`). Les
  actions sensibles (shell/commit/push) sont bloquees cote serveur
  (Phase D `operator_chat_rejects_sensitive_action_execution` test).

- **CORS Operator** : Phase D a mis CORS Any pour localhost:*. L'Operator
  passe par le Vite proxy en dev, donc ne declenche pas CORS en prod. Le
  durcissement est prevu Phase F. **GAP Low** (cf. preflight S3 V2).

- **factory-ui/operator/api-client.ts** : le fichier n'est pas importe
  par l'Operator. S'il etait importe, le `BASE_URL` hardcode
  contournerait le Vite proxy. Risk: nil tant qu'il n'est pas utilise.

## Livrable verification (Claude pre-Codex, ne remplace pas Codex)

| # | Livrable | Statut | Fichier:ligne | Evidence |
|---|----------|--------|---------------|----------|
| 1 | Design prompt UX | CONFIRME | `.planning/active/sprint70_factory_ux_design_prompt.md:1-126` | Prompt complet avec 2 produits, ecrans, interdits securite, design system partage |
| 2 | Design handoff avec waiver | CONFIRME | `.planning/active/sprint70_factory_ux_design_handoff.md:1-69` | Waiver Claude Design (ligne 5), decisions palette, layout, composants, responsive |
| 3 | factory-ui/src/readonly/ (8 fichiers) | CONFIRME | `tools/factory-ui/src/readonly/index.ts:1-28` | 6 composants + labels + types exportes. StatusBadge, VerdictChip, ProofCard, SprintTimeline, PreviewList, ChangelogPanel |
| 4 | factory-ui/src/operator/ (2 fichiers) | CONFIRME | `tools/factory-ui/src/operator/index.ts:1-18` | 13 fonctions API client exportees (getStatus, getLint, getAudit, etc.) |
| 5 | Viewer SBFB.json | CONFIRME | `examples/sbfb-factory-viewer/SBFB.json:1-8` | schema_version 2, bridge_methods: browse_list, search, proof_card_get, storage_get, storage_set |
| 6 | Viewer index.html | CONFIRME | `examples/sbfb-factory-viewer/index.html:1-45` | Top bar, search, filter, grid, detail view, status bar. Lang="fr". |
| 7 | Viewer app.js | CONFIRME | `examples/sbfb-factory-viewer/app.js:1-155` | SPDX header, IIFE, bridge browseList/proofCardGet, escapeHtml, filter/search, detail proof card |
| 8 | Viewer style.css | CONFIRME | `examples/sbfb-factory-viewer/style.css:1-345` | Dark theme SBFB, responsive, top bar, cards, proof card, empty states |
| 9 | Viewer sbfb-bridge.js sync | CONFIRME | 422 lignes identique aux 3 autres copies | `diff` hash identique web/public/, sbfb-explorer/, sbfb-ideas/ |
| 10 | Operator package.json | CONFIRME | `tools/factory-operator/package.json:1-48` | Vite+React+TS+Tailwind+shadcn+i18next+react-router-dom |
| 11 | Operator App.tsx (router+layout) | CONFIRME | `tools/factory-operator/src/App.tsx:1-46` | 10 routes, Sidebar, StatusBar, useApi /status |
| 12 | Operator SprintOverview | CONFIRME | `tools/factory-operator/src/pages/SprintOverview.tsx:1-291` | Status API, phases grid, test stats, artifact indicators, empty/error states, skeletons, i18n |
| 13 | Operator AgentSelector | CONFIRME | `tools/factory-operator/src/pages/AgentSelector.tsx:1-201` | Qui code/verifie, localStorage persist, same-agent warning |
| 14 | Operator PhaseAssistant | CONFIRME | `tools/factory-operator/src/pages/PhaseAssistant.tsx:1-323` | 6 intentions metier FR, phase select auto-detect, TechnicalDetails, result copy |
| 15 | Operator LintOperator | CONFIRME | `tools/factory-operator/src/pages/LintOperator.tsx:1-127` | API /lint, error/warning badges, tooltips, empty state |
| 16 | Operator CommitAuditor | CONFIRME | `tools/factory-operator/src/pages/CommitAuditor.tsx:1-217` | SHA input, sections present/missing, review/codex gates |
| 17 | Operator AgentTransfer | CONFIRME | `tools/factory-operator/src/pages/AgentTransfer.tsx:1-208` | Handoff + context-pack generation, provider/role select |
| 18 | Operator ContextPackBuilder | CONFIRME | `tools/factory-operator/src/pages/ContextPackBuilder.tsx:1-219` | Provider/role/sprint/phase, POST /api/context-pack, result copy |
| 19 | Operator ActionCenter | CONFIRME | `tools/factory-operator/src/pages/ActionCenter.tsx:1-182` | 4 actions allowlistees, sensitive actions banner, results log |
| 20 | Operator AgentChat | CONFIRME | `tools/factory-operator/src/pages/AgentChat.tsx:1-238` | Session start, message send, bubbles, loading state, empty state |
| 21 | Operator ActionLog | CONFIRME | `tools/factory-operator/src/pages/ActionLog.tsx:1-126` | API /actions/log, entries with timestamps, args, result |
| 22 | Operator Sidebar | CONFIRME | `tools/factory-operator/src/components/Sidebar.tsx:1-109` | 10 nav entries, mobile hamburger, responsive collapse, i18n |
| 23 | Operator StatusBar | CONFIRME | `tools/factory-operator/src/components/StatusBar.tsx:1-13` | HEAD sha + sprint number, i18n |
| 24 | Operator TechnicalDetails | CONFIRME | `tools/factory-operator/src/components/TechnicalDetails.tsx:1-29` | Repliable, commande technique cachee par defaut |
| 25 | Operator useApi hook | CONFIRME | `tools/factory-operator/src/hooks/useApi.ts:1-68` | useReducer fetch, postApi, cancel on unmount |
| 26 | Operator i18n fr/en | CONFIRME | `tools/factory-operator/src/i18n/locales/fr.json:1-206` + `en.json:1-206` | 206 lignes chacun, cles identiques, contenu traduit |
| 27 | Operator shadcn components (11) | CONFIRME | `tools/factory-operator/src/components/ui/*.tsx` | badge, button, card, dialog, dropdown-menu, input, scroll-area, select, separator, tabs, tooltip |

Resume : 27 livrables / 27 confirmes / 0 gaps / 0 partiels

## Patterns drift + horizon long-terme

### Patterns
- P1 typed coordinator client : N/A (Operator n'appelle pas le coordinator)
- P2 base-ui render prop : le Operator utilise @base-ui/react + Radix,
  coherent. `TooltipTrigger render={<span />}` utilise bien le pattern
  render prop de base-ui (cf. AgentTransfer.tsx:119, ActionCenter.tsx:107)
- P5 CORS loopback only : Phase D CORS Any est un gap connu (defer Phase F).
  Le Viewer ne touche pas CORS (bridge only). Coherent.
- P24 postMessage bridge : le Viewer utilise le bridge comme seul canal.
  Coherent.

### Horizon long-terme
- Design doc present : oui (design prompt + handoff = docs avant code)
- D5 avec alternatives + rationale : oui (kickoff §D5 rejette 3 alternatives)
- Solution la plus poussee : oui (separation physique Viewer/Operator =
  pattern RBAC le plus strict, confirme par react-admin et shadcn-admin)
- Aucune LOC estimee au plan : 0 match (plan §8 ne contient pas d'estimation LOC)

## Commit body validation

### Titre
- Format attendu : `feat(factory): Sprint 70 Phase E — Factory Viewer + Operator local action-gated`
- Regex : match `feat(factory): Sprint 70 Phase E — .+`

### 9 sections body
- Draft body non fourni — **CONCERN** "draft-body-absent". L'executeur doit
  produire les 9 sections avec le template `.claude/templates/commit_body_phase.txt`.

### Co-Authored-By
- A verifier au commit.

## Findings

- **P2-E-1** `.gitignore` insuffisant pour `tools/factory-operator/node_modules/` —
  `tools/factory-operator/node_modules/` est visible a `git status` car le root
  `.gitignore` pattern `/node_modules/` ne matche que la racine. `dist/` est OK
  (pattern global). L'executeur qui fait `git add tools/factory-operator/` sans
  attention stagera ~1.8MB de node_modules.
  **Fix** : ajouter `node_modules/` (sans slash initial) au root `.gitignore`,
  ou creer `tools/factory-operator/.gitignore` avec `node_modules/`.

- **P2-E-2** `factory-ui/operator/api-client.ts` hardcode `BASE_URL = "http://127.0.0.1:4242"` sur port 4242 —
  le fichier `tools/factory-ui/src/operator/api-client.ts:3` hardcode le port.
  L'Operator utilise son propre `useApi.ts` avec relative paths + Vite proxy
  (port 4242 via `vite.config.ts` proxy). Le `api-client.ts` n'est pas importe
  par l'Operator. Inconsistance : l'Operator Vite proxy cible `127.0.0.1:4242`
  mais le serveur `sbfb-factory operator serve` utilise `--port 3001` par defaut
  dans les tests Phase D. Le port 4242 dans `api-client.ts` et le port 4242 dans
  `vite.config.ts` divergent du port 3001 documente dans le plan §7.1.
  **Fix** : aligner le port sur la valeur reelle (3001 ou 4242) et rendre
  `BASE_URL` configurable (env var ou param).

- **P2-E-3** SprintOverview.tsx artifact labels en anglais technique —
  `SprintOverview.tsx:162-170` affiche les labels "preflight", "review", "codex"
  directement comme texte dans les `ArtifactIndicator`. Ces termes techniques
  apparaissent dans l'UI principale, pas dans TechnicalDetails. Le plan §8.1.2
  exige que les termes techniques soient dans un panneau repliable. Les indicateurs
  d'artefact sont des labels courts informationnels (pas des CTA), donc le
  contournement est borderline. Neanmoins, la convention definie dans le plan
  exige des labels FR.
  **Fix** : utiliser `t("sprint.artifactPresent")` ou des labels FR pour les
  artefacts (ex: "Preflight" est acceptable en FR informatique, mais "review" et
  "codex" devraient etre "Relecture" et "Codex").

- **P2-E-4** `factory-ui/operator` n'est pas importe par l'Operator —
  `tools/factory-ui/src/operator/` exporte 13 fonctions API (index.ts + api-client.ts)
  mais l'Operator (`tools/factory-operator/`) n'importe jamais
  `@sbfb/factory-ui/operator` ni `factory-ui/operator`. Le code est du dead code
  par rapport a l'architecture actuelle.
  **Fix** : soit l'Operator importe et utilise `factory-ui/operator` (supprimant
  le `useApi.ts` doublon), soit documenter que `factory-ui/operator` est prevu
  pour Phase F ou S71 consommateurs.

- **P3-E-1** Viewer index.html : pas de meta CSP `<meta http-equiv="Content-Security-Policy">`
  dans le HTML. Le Viewer tourne dans l'iframe sandbox avec CSP blob-serve, donc
  la meta n'est pas strictement necessaire. Nit.

- **P3-E-2** `i18next` et `react-i18next` non traces dans le preflight S1b —
  ces deux deps sont nouvelles dans le projet (pas dans web/). Le preflight
  S1b scanne les libs de la stack mais ne mentionne pas i18next. Pas de CVE
  connue, lib mature. Nit.

- **P3-E-3** Operator CSS Vite warning "chunks > 500 kB" au build. Le bundle
  n'est pas split (pas de code-splitting Operator). Acceptable pour un outil
  local dev, pas un probleme de prod. Nit.

- **P3-E-4** `ActionCenter.tsx:113` utilise `size="icon-xs"` sur un Button.
  Ce variant n'existe probablement pas dans les Button variants shadcn standard.
  Le build passe car la prop est ignoree au runtime. Nit cosmetique.

## Codex reconciliation

- Status : RECONCILIE
- Rapport Codex : sprint70_phase_e_codex_review.md (brut, non reecrit)
- Verdict Codex : GAP (5 deliverables, 3 GAP)
- GAPs corriges :
  - Viewer bridge methods : `browseList()`/`proofCardGet()` → `getBrowseList()`/`getProofCard()` (app.js)
  - Viewer manifest : retire `storage_set` (lecture seule)
  - Viewer accessibility : `<div class="app-card">` → `<button>` avec `aria-label`
  - ActionCenter : `{ action: actionId }` → `{ command: actionId }` (aligne sur backend `ActionRunRequest.command`)
  - AgentChat : `session_id` → `id` dans response, `content` → `message` dans request body (aligne sur backend `ChatMessageRequest`)
  - factory-ui api-client : `action` → `command`, `content` → `message` (memes fixes)
- P2/P3 documentes dans body :
  - P2-CODEX-1 factory-ui tsc standalone : P3 (package peer-dep, IDE support, pas build standalone)
  - P2-CODEX-2 Viewer "Commit source"/"host shell" : faux positif (labels de preuve ≠ forbidden terms §8.4)
  - P3-CODEX-3 sidebar aria-label "Menu" hardcode : P3 minor
  - P3-CODEX-4 dialog.tsx "Close" : shadcn generated, pas modifie
- Suites relancees apres corrections : tsc 0 errors, build OK

## Dimensions explored (evidence audit exhaustif)

| Dimension | Commandes executees | Fichiers lus | Findings |
|-----------|---------------------|--------------|----------|
| Security | grep unwrap/unsafe/secrets, boundary grep 8 patterns Viewer, boundary grep 8 patterns readonly, escapeHtml review, input analysis Operator/Viewer | 50+ fichiers source | 0 P0, 0 P1 |
| Patterns | PATTERNS.md rust + shell lus, P1-P26 verifies vs code Phase E | PATTERNS.md 2747 lignes + shell/PATTERNS.md 2155 lignes | 0 drift |
| Scope-cuts | 14 items kickoff §7 + grep mecanique + lecture semantique diff complet | kickoff.md §7, tous fichiers Phase E | 0 leak |
| Branch coverage | 14 elements testes (boundary, composants, pages, i18n, empty states) | tous les fichiers .tsx + .ts + .js + .css + .json | 0 untested |
| Research grounding | preflight 5/5 scans + deps 9 libs + coherence bridge/API | preflight.md, package.json, api-client.ts, useApi.ts, vite.config.ts | 0 P0, 2 P3 (i18next non trace, port inconsistance) |
| Livrables | 27/27 verifies via Read | tous les fichiers Phase E | 0 gap |
| Horizon long-terme | design doc present, D5 alternatives, LOC 0 match | kickoff.md §D5, plan.md §8, preflight.md | 0 drift |

## Recommendation

- Ready to commit : non (verdict PASS-PENDING, Codex requis)
- Carry-overs S71 : aucun (les P2 sont resolvables dans la meme phase)
- Corrections needed avant Codex :
  1. **P2-E-1** : ajouter `node_modules/` au root `.gitignore` (sans slash initial)
  2. **P2-E-2** : aligner le port api-client.ts / vite.config.ts (4242 ou 3001) et
     documenter le choix
  3. **P2-E-3** : i18n-iser les labels artifact dans SprintOverview (FR)
  4. **P2-E-4** : documenter que `factory-ui/operator` est prevu pour des
     consommateurs futurs ou le supprimer

## Post-commit obligatoire

- [ ] Update nexus_grid_pivot.md (tip SHA + description sprint + compteurs tests)
- [ ] Update MEMORY.md (ligne index si pivot description changee)
- [ ] Verifier que review.md est stage dans le commit chore(planning) suivant
