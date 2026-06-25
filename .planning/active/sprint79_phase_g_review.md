# Sprint 79 Phase G — Review (copilote Ollama + template daisyUI)

Workflow review `wf_f454c668-c76` — 6 dimensions (fan-out) + vérification adversariale + synthèse.

Diff de phase reviewé : 3 fichiers modifiés (`template_engine.rs`, `llm_bridge.rs`, `gates.rs`)
+ 9 fichiers template daisyui nouveaux + préflight + `main.rs` (help `--kind`). Suite sbfb-factory
200 tests verts, fmt clean, frontier-contracts clean, nextest workspace 1990 (+7 vs 1983), release build OK.

**Verdict : PASS-PENDING** (review OK ; Codex pas encore lancé). Aucun P0/P1 confirmé. 6 dimensions
saines (4 PASS + 2 CONCERN entièrement portées par un unique défaut documentaire ramené à P2 par la
vérification adversariale, **corrigé avant Codex**).

## Verdict: PASS

> Promu après reconciliation Codex (section `## Codex reconciliation` en bas).

## CORRECTNESS / BUGS — CONCERN → résolu (P2 corrigé)

- **P2 (CORRIGÉ)** — `CAPABILITY_BLOCK` (llm_bridge.rs) annonçait une commande CLI inexistante
  `sbfb-factory new --template daisyui`. Le vrai verbe est `create` (main.rs:44, clap kebab-case),
  `--name` requis (main.rs:50-51, pas de default), 0 alias. Corrigé en
  `sbfb-factory create --template daisyui --name <name>`. Un test de régression
  (`assemble_prompt_surfaces_non_authoritative_capability_block`) asserte désormais le verbe valide et
  l'absence de `sbfb-factory new`.

Le reste du chemin de données est sain : fix `create_dir_all` parent idempotent ; 5e TemplateConfig
daisyui (9 `include_str!` valides, flags substitute corrects) ; ordonnancement assemble_prompt correct ;
hash template déterministe cross-plateforme (forward-slash littéral) ; classification CSP exacte. **Aucun P0/P1.**

## SCOPE / PLAN-FIDELITY — PASS

- **P2 (documenté)** — `input.css` émet `@plugin "daisyui" { themes: false; }` alors que la directive
  préflight #5/#6 suggérait de NE PAS émettre de bloc `@plugin "daisyui"{themes:…}`. **PLAN-ADAPT
  empirique LÉGITIME** : build vérifié main-thread (tailwindcss 4.3.1 + daisyui 5.5.23) — sans
  `@plugin "daisyui"` les composants `.btn/.card` ne sont PAS générés ; `themes: false` charge les
  composants avec **0 thème built-in** (app.css = 18 750 octets, `.btn/.card/.badge/progress` présents,
  night/dracula/synthwave absents, seules URLs allowlistées). Aucun invariant Day-0 enfreint. Tracé dans
  le préflight (section ## Raffinement empirique) + le commit body.
- **P3** — `.gitignore` livré sous le nom source `gitignore`, restitué `.gitignore` à la génération
  (contournement cargo standard, miroir des autres templates).

Vérifications vertes : 0 occurrence « 8 thèmes » hors test-garde ; package.json 0 caret/tilde, versions
exactes ; `provider_router.rs` ET `operator_server.rs` NON modifiés (Day-0 #7) ; bloc capacité
non-autoritaire (anti-PASS tenu) ; aucun livrable Phase H/I n'a fui.

## Sécurité — PASS

Anti-PASS mécanique : le gate SENSITIVE_ACTIONS (operator_server.rs:35/:959-972) scanne `last_user_msg`
BRUT **avant** `assemble_prompt` (:978) → le bloc capacité prepend n'est jamais un chemin de
contournement, et le mot « PASS » qu'il contient ne déclenche rien. Template CSP clean
(`run_gate_csp_authoring` PASS sur tout le workspace y compris `scripts/*.mjs`, FG5/FG6 PASS),
vendor UMD same-origin (0 primitive réseau), copilote Ollama keyless sans capacité d'action.

- **P3** — Build-time `@tailwindcss/oxide` télécharge un binaire natif non-vendoré (chaîne build-time
  hors-runtime, hors-couverture du gate CSP runtime). Version exact-pin, identique au template react.
  Durcissement supply-chain build-time (lockfile npm commité) à considérer plus tard.

## RESEARCH-GROUNDING — PASS

API anime.js v4 correcte (named-export `window.anime`, param `ease` pas `easing`), bundle réellement
v4.5.0 UMD, `app.css` réel Tailwind 4.3.1 + daisyUI 5.5.23 compilé, 0 thème built-in fuité, recette
`@import "tailwindcss" source(none)` + `@source` + `@plugin "daisyui"{themes:false}` + `@plugin
"daisyui/theme"` correcte. Gate tests instancient le vrai template.

- **P3** — Animation `value` du progress (app.js) valide anime v4 mais diverge du pattern proxy-object
  `{v:0}`+onUpdate du showcase ; couverte par le garde REDUCE, pas de défaut correctness.

## TESTS — PASS

- **P2 (CORRIGÉ)** — l'ordonnancement bloc/historique n'était testé qu'avec historique vide ; le test
  asserte désormais `history < block < message` avec historique non-vide.
- **P3** — assertion anti-PASS : remplacée par garde positive sur le verbe CLI réel + marqueurs positifs.
- **P3** — pas de snapshot du hash template_lock forward-slash (déterminisme garanti par const littérale).

Delta +7 honnête et vérifié (1983→1990). Tests phares non-vacuous : `test_create_daisyui_template`
prouve le fix `create_dir_all` (3 sous-chemins qui auraient NotFound-paniqué) ;
`test_csp_gate_daisyui_template_passes` exécute le vrai gate sur le workspace généré.

## PATTERNS / CONVENTIONS — CONCERN → résolu

- **P2 (CORRIGÉ)** — même finding `new`/`create` (cf. CORRECTNESS).
- **P2 (CORRIGÉ)** — drift pré-existant ré-exposé : l'aide `--kind` (main.rs:157) ne listait pas
  `base`/`universal`/`app-authoring` (réel = `PROMPT_KINDS` 9 kinds, process.rs:7-23) alors que le bloc
  capacité dirige le copilote vers `--kind app-authoring`. Help string aligné sur `PROMPT_KINDS`.
- **P3** — versions build-time en littéraux dupliqués package.json/test : pattern correct (le test
  verrouille le pin), aucune action.

Provenance « Sprint 79 Phase G » uniforme ; aucune promesse future dans le diff (anti STALE-PHASE-K) ;
forward-slash littéral correct ; frontier-contracts §6.12 = asset additif scellé, clean.

## Constat environnement (dual-platform)

Le run Docker canonique `sbfb-ci` a vu **3 tests d'intégration `operator_context_pack_*` échouer en
timeout 30s** (`reqwest TimedOut` sur `/api/context-pack`). **Cause = environnement, PAS Phase G** :
`handle_context_pack` appelle `process::context_data` qui lance **git** ; sur le volume Windows monté
dans le conteneur, git refuse l'« dubious ownership » de `/work`. Diagnostic décisif : avec
`git config --global --add safe.directory /work`, les 5 tests `context_pack` **passent** (7-19s).
Phase G ne touche NI `operator_server.rs` NI `process::context_data`. Win natif (ownership correct)
passe les 1990 ; le CI Linux natif (Woodpecker/GHA, checkout natif) est non affecté. **À retenir pour le
workflow dual-platform** : le conteneur sbfb-ci a besoin de `safe.directory` pour les tests
git-touchants sur volume Windows monté.

## Vérifications adversariales

1 finding P0/P1 vérifié (le `new`/`create`) : CONFIRMÉ réel mais **ramené P1→P2** (const consultative
non-autoritaire, jamais exécutée, 0 impact build/gate/invariant/test/sécurité). Corrigé avant commit.
Aucun finding rejeté.

## Verdict

**PASS-PENDING → PASS** après reconciliation Codex. Aucun P0/P1. Le P2 CLI `new`/`create` + le P2
help `--kind` + le P2 couverture-ordre sont **corrigés** ; le PLAN-ADAPT empirique `themes: false` et
les P3 restants sont documentés (préflight + commit body).

## Codex reconciliation

Codex (GPT 5.5, `codex exec`) — output brut dans `sprint79_phase_g_codex_review.md` (jamais réécrit).
**Verdict : 6 livrables, 5 CONFIRME, 0 GAP, 1 PARTIEL.**

- Livrables 1-5 **CONFIRME** avec evidence fichier:ligne : copilote (bloc capacité non-autoritaire,
  verbe CLI réel `create`, gate SENSITIVE_ACTIONS scanne le message brut avant `assemble_prompt`,
  provider_router/operator_server/Cargo.lock/csp.rs hors-diff), fix `create_dir_all` (hash déterministe
  sur noms triés), template daisyui (9 fichiers, 0 dep runtime), raffinement `themes: false` (app.css :
  `.btn=49`/`.card=11`/`.badge=5`, `night/dracula/synthwave=0`, seules URLs allowlistées), help `--kind`.
- Livrable 6 **PARTIEL** — Codex confirme les **+7 tests staged** (assertions utiles, non-vacuous,
  `8 tests run: 8 passed`) mais ne peut pas vérifier le **total absolu** 1990 : dans SON environnement
  `cargo nextest list --workspace` compte 710 et `--all-targets` échoue au listing sur
  `nexus-executor::bench/cold_start` (budget bench dépassé). C'est une **limitation d'environnement
  Codex, pas un GAP** : le total 1990 est mesuré côté main-thread par `cargo nextest run --workspace`
  (1990 passed, 0 skipped). Aucune correction.
- Points de vigilance Codex levés : `README.md:40` contient le littéral `<script type="module">` mais
  README = tier **Skip** (non scanné) ; `scripts/vendor-anime.mjs:7` a le texte `type=module` mais PAS
  le littéral `<script ...>` → gate clean. Confirmé par `test_csp_gate_daisyui_template_passes`.

**0 GAP, 0 P0/P1 Codex → review promue PASS.** Suites non relancées (aucune correction de code post-Codex).
