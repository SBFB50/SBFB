# Handoff — Process Evolution Commit 2 (Playwright E2E + harness multi-machine)

**Contexte.** Le PO veut : (1) phases illimitées [FAIT, Commit 1 `926f929`],
(2) E2E Playwright front solo + multi-machine bakés dans le process. Le
**Commit 1** a livré le volet process+tooling (regex phase `[A-Z]+[0-9]?`
partout + `test_agentctl.py` 20/20 + README §4 « Budget de phases » + « Gate
de testabilité par-sprint » T0/T1/T2 + §6 + CLAUDE.md). **Ce handoff = Commit 2**
(infra test, exige runtime : build + browser).

**4 arbitrages PO confirmés** : grammaire phase `[A-Z]+[0-9]?` (illimité) ;
E2E gate **par-sprint** (wrap-up bloquant + CI chaque push, PAS par-commit) ;
Playwright sur **vrai daemon** (pas mock — mocker recrée la vacuité) ; gate
multi-machine **split par tier** (hard-gate hermétique+compute-local, soft-gate
multi-machine via artefact JSON `PASS`/`BLOCK{diag}`/`RIG-ABSENT` + PROVISIONAL
+ carry P1 + prérequis convergence).

**Plan complet du Workflow** : script
`<session>/workflows/scripts/process-evolution-testability-wf_056f4424-28e.js` ;
sortie complète (24k) dans le transcript du run `wf_056f4424-28e`. Le manifeste
ci-dessous suffit.

## Ground truth vérifié (par le Workflow)
- Validateur canonique = **`scripts/agent/agentctl.py`** (les `.githooks/*` = shims).
- `scripts/verify.sh:80-81` lance déjà `npx playwright test` → **no-op** (ni config ni specs) → à câbler.
- `web/tests/global-setup.ts` + `global-teardown.ts` **survivent** (réutiliser) ; `web/e2e/` + `web/playwright.config.ts` **absents** ; `@playwright/test@^1.59.1` est devDep.
- `web/src/bridge/useBridge.ts:236-262` injecte `project_id` (compute) ; iframe = `data-testid="remote-iframe-element"`, opaque-origin (`frameLocator` obligatoire).
- `phase_h_compute_local.sh` fait déjà la réconciliation `/api/daemon/project-info`↔PROJECT_ID ; `b3_live_pc_vps.sh` NON (`die()` exit 1 uniforme, PROJECT_ID arbitraire) → à corriger.

## Manifeste Commit 2 — `feat(test): real Playwright E2E (solo) + multi-machine acceptance artifact`

### Playwright solo (vrai daemon, Decision 3)
- **`web/playwright.config.ts`** NEW : `testDir:'./e2e'` (PAS `./tests` ni sous `src/` — Vitest include `src/**`, scan-en-strings scanne `web/src` ; `e2e/` exclu des deux) ; `projects:[{name:'chromium'}]` ; `fullyParallel:false`, `workers:1` (daemon singleton, décision gelée) ; `globalSetup/teardown:'./tests/global-*.ts'` ; `use:{ baseURL, extraHTTPHeaders:{'x-sbfb-token':TEST_AUTH_TOKEN}, locale:'fr-FR', trace:'retain-on-failure', screenshot:'only-on-failure' }`.
- **`web/tests/global-setup.ts`** EXTEND : ajouter `--web-root <repoRoot>/web/dist` (g#if dist existe) ; pour le spec compute, `init` + monter un project doc (que `/api/daemon/project-info` rende non-null, `http.rs:841`).
- **`web/e2e/fixtures.ts`** : `addInitScript` seed `TEST_AUTH_TOKEN` (réutiliser `auth.ts primeAuthToken`) + helper `computeFrame(page)` → `page.frameLocator('[data-testid="remote-iframe-element"]')`.
- **4 specs `web/e2e/*.spec.ts`** :
  1. `browse-offline.spec.ts` (hermétique) : daemon up, 0 entry → `browse-grid` empty state rendu, pas de `daemon-offline-banner` ; search interactable (la version HONNÊTE de l'ancien spec vacuous).
  2. `onboarding-empty.spec.ts` (hermétique) : 1er load → `OnboardingEmpty` + CTA ; exerce `bootstrap.ts:46-66` auto-register.
  3. `curators-add-validation.spec.ts` (hermétique) : `/curators` clé invalide → erreur validation FR.
  4. **`compute-tester.spec.ts`** (FLAGSHIP, tag `@compute`, **env-gated** `test.skip(!process.env.SBFB_E2E_COMPUTE || !ollamaReachable)`) : `/browse/:projectId` → `computeFrame` → type prompt → submit → **assert texte généré rendu** dans l'iframe (~30s ; Ollama ~12s). Miroir de `phase_h_compute_local.sh`.
- **`web/package.json`** scripts : `"test:e2e":"playwright test --grep-invert @compute"` (hermétique, CI-safe), `"test:e2e:compute":"SBFB_E2E_COMPUTE=1 playwright test compute-tester"`, `"test:e2e:ui":"playwright test --ui"`. NON dans la chaîne verify par défaut (besoin du binaire).
- **`scripts/verify.sh:80-81`** : remplacer le no-op par `npm run test:e2e` (hermétique) ; `--quick` le skip.
- **`.github/workflows/ci.yml:81-86`** : remplacer le bloc no-op par un vrai job Playwright hermétique **sur le runner node-20 GHA** (PAS l'image `rust:1.94` sans libs browser) : build dist + daemon, `playwright install --with-deps chromium`, `npm run test:e2e`. Le compute spec reste local.
- **`.gitignore`** : `web/test-results/`, `web/tests/.tmp/`, `web/.playwright-state.json`.

### Harness multi-machine (Decision 4)
- **`scripts/acceptance/b3_live_pc_vps.sh`** EDIT :
  1. Rig config = données : sourcer `scripts/acceptance/rig.local.env` (gitignored) {VPS_SSH, WORKER_SSH, PROJECT_ID, MODEL, WORKER_BIN, ports}.
  2. **Préflight → `RIG-ABSENT` exit 3 distinct** quand : SSH échoue / Ollama+MODEL absent / binaires S76+ absents / **`GET /api/daemon/project-info` ≠ PROJECT_ID** (réconciliation, comme `phase_h_compute_local.sh:23-28`).
  3. **Artefact JSON** sur chaque sortie : `{status:PASS|BLOCK|RIG-ABSENT, stage:claim|inference|result-replication, delay_s, task_id, diagnosis, last_response}`. `die()` (exit 1 actuel) → writer status+exit (BLOCK=1, RIG-ABSENT=3).
  4. BLOCK auto-diagnostic : sur timeout, `ssh`-grep la réplique worker pour `task:{id}` → `"task never reached worker replica"` vs `"reached but no result"`.
  5. Timers sous-stage claim/inference/result-replication.

### Vérification Commit 2 (RUNTIME, avant de déclarer done)
- `(cd web && npm run build)` ; build daemon ; `(cd web && npm run test:e2e)` hermétique → **GREEN** ; `npm run test:e2e:compute` (Ollama up + project doc monté) → flagship **PASS** ; consigner dans une trace.
- Bloc front §7.4 (tsc/lint/test:unit/build/size/scan) VERT (l'ajout `e2e/` ne doit casser ni Vitest ni scan-en-strings car hors `src/`).
- Titre `feat(test): ...` SANS « Sprint N Phase X » (off-sprint → pas d'apparatus de phase). Body libre.

## Séparé (S77 Phase A — PRÉREQUIS cross-machine, PAS dans Commit 2)
- **Nouveau test d'intégration Rust** (`nexus-shell-daemon`, à côté de `dispatch_loop.rs`/`result_sync.rs`) : 2 nœuds iroh se découvrant via le **vrai chemin discovery** (pas un handshake in-process pré-partagé) + assert qu'une entrée `task:` **incrémentale écrite APRÈS subscribe** se propage au réplica distant. C'est le miroir unit du BLOCK live diagnostiqué (dispatch_loop écrit, mais `recv:0` / gossip neighborhood non formé). iroh 0.98, 0 bump wire. À inscrire `sprint77_audit_plan.md` comme phase GATING bloquant toute claim cross-machine.

## Pièges connus
- `git add` SÉPARÉ du `git commit` (hook PreToolUse voit le staging AVANT le add de la même commande).
- Playwright `e2e/` doit rester hors `web/src/` (sinon Vitest le double-claim + scan-en-strings le scanne).
- WARN lightcheck « missing file » si le body cite un fichier non encore créé → bénin.
