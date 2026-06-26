# Sprint 79 — Phase G preflight (G8) — Workflow `wf_845ce0fd-fb3`

**Verdict : PLAN-ADAPT** (0 DESIGN-CONFLICT). 5 scans Opus 4.8 1M (S1a OSS prior-art, S1b
deps/CVE, S2 décisions historiques, S3 threat model, S4 wire/invariants) + 9 vérifications
adversariales (0 réfutée). 33 findings, 15 agents, 960k tokens.

Signaux : S1a EXECUTE · S1b EXECUTE · S2 PLAN-ADAPT · S3 EXECUTE · S4 EXECUTE.

## Contexte

Phase G du Sprint 79 (« Copilote Ollama keyless + starter template daisyui vendoré ») matérialise la capacité d'authoring dans l'archive livrée et le copilote. Spec autoritaire : `.planning/active/sprint79_plan.md` lignes 349-376. Deux livrables :

1. **Copilote** : bloc capacité UI « prepend » non-autoritaire dans `assemble_prompt`, dispatch `ExecutionTarget::Ollama` KEYLESS après le gate SENSITIVE_ACTIONS, `provider_router` INCHANGÉ.
2. **Starter template** : 5e `TemplateConfig` `daisyui` → `crates/sbfb-factory/src/templates/daisyui/*` (index.html, src/input.css, app.css, vendor/anime.umd.js, package.json, README, gitignore), passant FG5/FG6 + `run_gate_csp_authoring` CLEAN.

Les 5 scans + 9 vérifications adversariales convergent : **0 DESIGN-CONFLICT**, les Day-0 #7/#9/#10 sont tous respectés par le code existant ou la vitrine. Verdict **PLAN-ADAPT** : le plan porte une imprécision factuelle (« 8 thèmes ») déjà corrigée FAUSSE par Phase F, et l'implémentation requiert un fix préalable (création des répertoires parents) + un package.json from-scratch.

## S1a — Prior-art OSS (EXECUTE)

Les deux livrables sont **dé-risqués** par la vitrine shippée `examples/daisyui-animejs-showcase/` qui prouve chaque maillon :
- `vendor/anime.umd.js` v4.5.0 (118 KB) : wrapper IIFE classic-script `(globalThis||self).anime={…}`, consommé via `<script src=vendor/anime.umd.js>` AVANT `app.js`. Aucun spike UMD nécessaire.
- Recette Tailwind 4 build-time `@import "tailwindcss" source(none); @source "../index.html"; @source "../app.js";` + `@plugin "daisyui/theme"{…oklch…}` compilé par `tailwindcss -i src/input.css -o app.css --minify`. Le `app.css` ne porte que deux URLs absolues, toutes deux allowlistées (`http://www.w3.org/2000/svg` + bannière `https://tailwindcss.com` MIT, `csp.rs:48-52`).
- Câblage copilote DÉJÀ en place : SENSITIVE_ACTIONS (`operator_server.rs:959-972`) → `assemble_prompt` (`:978`) → `target.run` (`:998-999`). `assemble_prompt` (`llm_bridge.rs:61-93`) ne contient AUCUN bloc capacité aujourd'hui — point d'injection unique.

**BUG BLOQUANT découvert** : `template_engine.rs:227` `fs::write(out.join(tf.name))` n'a aucun `create_dir_all` du parent ; tous les templates existants sont flat → le 5e template à sous-dossiers échoue.

## S1b — Deps / CVE / versions (EXECUTE)

Lot build-time figé **validé lockfile-resolved** (lu depuis les `node_modules` réellement installés du showcase, pas seulement le MANIFEST) : `daisyui 5.5.23`, `tailwindcss 4.3.1`, `@tailwindcss/cli|node|oxide 4.3.1`, `animejs 4.5.0`. **0 CVE connue** (les CVE historiques visent Tailwind 3.x, pas 4.x). Le template = `include_str!` d'assets statiques (miroir REACT_TEMPLATE) → **0 nouvelle crate Rust, 0 changement Cargo.lock, 0 bump wire**. Archive livrée = 0 dépendance runtime. **Attention Day-0 #10** : le `package.json` du showcase utilise des carets (`^4.1.0`) et n'inclut pas node/oxide explicitement → le nouveau template doit pinner les versions résolues SANS caret et lister node+oxide.

## S2 — Décisions historiques (PLAN-ADAPT)

Aucun DESIGN-CONFLICT. UN ajustement obligatoire : `sprint79_plan.md:361` dit littéralement « template lean, 8 thèmes built-in retirés » — affirmation EXPLICITEMENT corrigée FAUSSE par Phase F (commit `8d7ee81` §A1 : daisyUI 5.5.23 = 35 thèmes). La formulation canonique « aucun des 35 thèmes built-in » est déjà gravée en code (`process.rs:978/989`, `app-authoring.md:177`). Le gate SENSITIVE_ACTIONS (« PASS » inclus, `:35`) tourne déjà avant le dispatch provider-indépendant ; le rejet de pi-ai/SDK est gelé Day-0 #7. La doctrine `chat_history_authoritative=false` vit dans le context-pack (`operator_server.rs:437-438/696-697`) et le prompt-kind.

## S3 — Threat model (EXECUTE)

Surface faible. Anti-PASS **mécaniquement garanti** : le gate scanne le message utilisateur BRUT (`req.message`/`last_user_msg`) AVANT `assemble_prompt`, donc le bloc capacité prepend n'est ni gaté ni un chemin de contournement. Le copilote Ollama keyless n'a AUCUNE capacité d'action (texte seul) ; seul l'agent Claude spawn écrit, et il reste derrière le même gate. Exigence de rédaction : bloc DESCRIPTIF/CONSULTATIF, jamais impératif (« réponds PASS » interdit), pour ne pas biaiser la génération Ollama vers un faux verdict affiché (sans effet de bord de toute façon). Le template daisyui passe le gate CSP : daisyUI n'émet que des `data:` URI inline (0 `url(http)`), `oklch()` est une fonction couleur (pas une `url()`), `backdrop-filter` est composite GPU non-réseau. **Précision supply-chain** : le zip `atelier.rs:146-187` n'exclut PAS `node_modules/` mécaniquement — l'exclusion dépend du `.gitignore` ; tout résidu serait attrapé par le gate (FAIL), pas exclu silencieusement. À documenter ainsi dans le README.

## S4 — Test (EXECUTE)

Couverture réelle dérivée des autres scans : +4 à +6 tests Rust (création template à sous-dossiers, gate CSP/FG5/FG6 clean, marqueur non-autoritaire dans assemble_prompt, anti-« 8 thèmes », pin versions).

## Vérifications adversariales

8 claims load-bearing vérifiées (le 9e est un placeholder de test). **Aucune réfutée** :
- daisyUI 5.5.23 = 35 thèmes built-in (CONFIRMÉ : `theming.json` len=35, `README.md:20`) ; lean = 0/35 ; plan « 8 » FAUX. Formulation canonique déjà en code.
- Bug `create_dir_all` parent manquant CONFIRMÉ par lecture directe (`template_engine.rs:215-229`, `fs::write` ne crée pas les parents ⇒ `NotFound` Win+Linux ; `.unwrap()` en test panique). Hash déterministe exige des littéraux forward-slash dans `TemplateFile.name` (`template_lock.rs:44-52` consomme `name.as_bytes()` sans normalisation).
- Carets du showcase package.json CONFIRMÉS ⇒ créer le package.json from-scratch.
- Gate SENSITIVE_ACTIONS avant dispatch, « PASS » dans la liste, `chat_history_authoritative=false` : tous CONFIRMÉS verbatim.
- Le gate réel = `run_gate_csp_authoring` (`gates.rs:386`), PAS `run_gate_authoring_csp` (spec erronée).
- Résidu gate keyword-based (pas capability-based) inchangé ; Phase G ne l'aggrave pas.

## Verdict

**PLAN-ADAPT.** L'approche tient, le code suit l'approche CORRIGÉE : formulation « aucun des 35 thèmes built-in (0/35) » jamais « 8 » ; fix `create_dir_all` du parent avant chaque write ; package.json from-scratch versions résolues sans caret ; gate `run_gate_csp_authoring`. Aucune Day-0 gelée contredite.

## Raffinement empirique (post-preflight, vérifié main-thread par build)

La directive preflight #5 (« NE PAS émettre de bloc `@plugin "daisyui"{themes:…}` ; l'absence du
bloc = 0 thème built-in ») est **empiriquement FAUSSE** et a été corrigée à la compilation. Le
main-thread a compilé `app.css` avec `tailwindcss 4.3.1` + `daisyui 5.5.23` (recette `build:css`) :
**sans** le plugin `@plugin "daisyui"`, daisyUI ne génère AUCUN composant (`.btn`/`.card`
absents). La forme lean correcte est `@plugin "daisyui" { themes: false; }` — elle charge les
composants en activant **0 des 35 thèmes built-in** — suivie de `@plugin "daisyui/theme"` pour le
seul thème custom `sbfb-reflect`. Preuve de build : `app.css` = 18 750 octets, `.btn/.card/.badge/
progress` présents, `night/dracula/synthwave/cyberpunk` absents, seules URLs absolues =
`http://www.w3.org/2000/svg` + bannière `https://tailwindcss.com` (toutes deux allowlistées
`CSS_URL_ALLOW`). Le gate `run_gate_csp_authoring` passe CLEAN sur le template généré (test
`test_csp_gate_daisyui_template_passes`). Cet écart est un PLAN-ADAPT empirique, pas un
DESIGN-CONFLICT : aucun invariant Day-0 (#7/#9/#10, anti-PASS, 0 bump wire) n'est touché.

## Directives de codage (main-thread)

1. **Copilote** : ajouter un bloc capacité non-autoritaire DANS `llm_bridge.rs::assemble_prompt`, prepend AVANT `new_msg` (ligne 91), après header context + history. Bloc DESCRIPTIF/CONSULTATIF (jamais « réponds PASS »), miroir `app-authoring.md:12/227` (« guidance, never authoritative ; verdict/PASS = vraie session agent »), avec marqueur stable `chat_history_authoritative=false` / « non-authoritative ».
2. **NE PAS toucher `provider_router.rs`** (Day-0 #7) : bras Ollama keyless (`Ollama::default()` loopback) + dispatch déjà câblés. RIEN à câbler côté Ollama.
3. **NE RIEN insérer entre le gate (`operator_server.rs:959-972`) et le dispatch (`:998-999`)** ; garder « PASS » dans SENSITIVE_ACTIONS (`:35`).
4. **FIX PRÉALABLE** `template_engine.rs:221-229` : `if let Some(p)=out.join(tf.name).parent(){fs::create_dir_all(p)?}` avant chaque `fs::write`. Vérifier `expected_files` (`:271-289`) et `TemplateLock::generate` (`:257`) cohérents.
5. **5e TemplateConfig `daisyui`** dans TEMPLATES (`:170-203`), miroir REACT_TEMPLATE. `TemplateFile.name` en forward-slash littéral (`src/input.css`, `vendor/anime.umd.js`) pour hash déterministe cross-plateforme.
6. **Fichiers template** : `index.html` (`data-theme="sbfb-reflect"`, link app.css classic, script vendor/anime.umd.js classic AVANT app.js, JAMAIS type=module/CDN) ; `src/input.css` (`@import "tailwindcss" source(none)` + `@source` + `@plugin "daisyui/theme"` sbfb-reflect oklch inline, AUCUN bloc `@plugin "daisyui"{themes:…}`, pas de tailwind.config.js) ; `app.css` pré-compilé déterministe (data:/relatif + bannière tailwind allowlistée) ; `vendor/anime.umd.js` (copie v4.5.0 existante) ; `package.json` (devDeps figées sans caret + scripts build:css/vendor:anime, runtime 0 dép) ; `README.md` ; `gitignore`.
7. **README.md** : doctrine vendorisation (UMD same-origin, anti-ESM/CDN, Tailwind-CDN + Google Fonts interdits) + CSP (COEP require-corp ⇒ classic scripts only) + confiance build-time (devDeps figées + gate de sortie ; node_modules exclu par gitignore, résidu = gate FAIL — le zip n'exclut pas node_modules mécaniquement) + DÉCLINE le pattern react UMD no-build (daisyui A un build-step).
8. **Formulation thèmes** : partout « aucun des 35 thèmes built-in (0/35), seul le thème custom sbfb-reflect ». JAMAIS « 8 ».
9. **package.json** : versions résolues SANS caret — daisyui 5.5.23, tailwindcss 4.3.1, @tailwindcss/cli|node|oxide 4.3.1, animejs 4.5.0.
10. **Gate** : référencer `run_gate_csp_authoring` (`gates.rs:386`), pas `run_gate_authoring_csp`.

## Tests (+4 à +6 Rust)

1. `test_create_daisyui_template` : `create("daisyui",…)` crée src/ et vendor/ sans panic (prouve le fix parent).
2. `test_csp_gate_daisyui_template_passes` : `run_gate_csp_authoring(workspace)` → `passed==true` (fixture clean T1b, modèle `csp_workspace` `gates.rs:772`).
3. `test_daisyui_template_fg5_fg6_pass` : `run_gate_fg5_sandbox` + `run_gate_fg6_secrets` → `passed==true`.
4. `assemble_prompt_surfaces_non_authoritative_capability_block` : le prompt contient le marqueur non-autoritaire ET new_msg, bloc avant new_msg.
5. `test_daisyui_template_no_false_eight_themes` : aucun « 8 thèmes » ; présence « aucun des 35 thèmes built-in » + « sbfb-reflect ».
6. `test_daisyui_package_json_pins_resolved_versions` : 0 caret, pins exacts du lot.
