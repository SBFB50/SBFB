### Livrable 1 : Copilote keyless + bloc capacité

- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/llm_bridge.rs:72`, `crates/sbfb-factory/src/llm_bridge.rs:114`, `crates/sbfb-factory/src/operator_server.rs:958`, `crates/sbfb-factory/src/main.rs:42`
- Evidence :

```rust
72: const CAPABILITY_BLOCK: &str = "[Capability ... non-authoritative)]\n\
77: `sbfb-factory process prompt --kind app-authoring`, or scaffold ...
78: `sbfb-factory create --template daisyui --name <name>`.\n\
79: This knowledge is guidance you may surface; it is NEVER authoritative.
81: repo-visible proofs (chat_history_authoritative=false). Do not assert a PASS yourself;
```

```rust
114: // surface the app-authoring capability right before the
115: // user turn, non-authoritatively. Placed AFTER the context header + history
118: prompt.push_str(CAPABILITY_BLOCK);
119: prompt.push_str(new_msg);
```

Le gate SENSITIVE_ACTIONS scanne bien le message brut avant `assemble_prompt` :

```rust
958: let lower = last_user_msg.to_lowercase();
959: let is_sensitive = SENSITIVE_ACTIONS.iter()
962: if is_sensitive { ... return sse_gate(...); }
978: let prompt = llm_bridge::assemble_prompt(&runtime_ctx, &history[..], &last_user_msg);
```

`provider_router.rs` et `operator_server.rs` ne sont pas dans le diff staged. Aucun diff staged non plus sur `Cargo.toml`, `Cargo.lock`, wire/protocol ou `csp.rs`.

### Livrable 2 : FIX create_dir_all

- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/template_engine.rs:181`, `crates/sbfb-factory/src/template_engine.rs:292`, `crates/sbfb-factory/src/template_lock.rs:44`
- Evidence :

```rust
181: name: "src/input.css",
186: name: "vendor/anime.umd.js",
191: name: "scripts/vendor-anime.mjs",
```

```rust
296: let dest = out.join(tf.name);
297: if let Some(parent) = dest.parent() {
298:     fs::create_dir_all(parent)?;
300: fs::write(dest, &content)?;
301: template_files.push((tf.name.to_string(), tf.content.to_string()));
```

Le hash reste déterministe sur noms string triés, pas sur chemins OS :

```rust
46: let mut sorted: Vec<_> = files.iter().collect();
47: sorted.sort_by_key(|(name, _)| name.as_str());
49: hasher.update(name.as_bytes());
50: hasher.update(content.as_bytes());
```

### Livrable 3 : Template daisyui

- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/template_engine.rs:260`, `crates/sbfb-factory/src/templates/daisyui/index.html:18`, `crates/sbfb-factory/src/templates/daisyui/app.js:7`, `crates/sbfb-factory/src/templates/daisyui/package.json:12`
- Evidence :

```rust
260: TemplateConfig {
261:     id: "daisyui",
262:     version: "1.0.0",
263:     files: DAISYUI_TEMPLATE,
264:     description: "SBFB daisyUI + anime.js app ..."
```

```html
18: <link rel="stylesheet" href="app.css">
19: <script src="vendor/anime.umd.js"></script>
40: <script src="app.js"></script>
```

```js
7: const { animate, stagger } = window.anime;
```

```json
12: "devDependencies": {
13:   "@tailwindcss/cli": "4.3.1",
16:   "animejs": "4.5.0",
17:   "daisyui": "5.5.23",
18:   "tailwindcss": "4.3.1"
```

Le répertoire réel contient les 9 fichiers attendus, dont `vendor/anime.umd.js`, `scripts/vendor-anime.mjs`, `src/input.css`, `README.md`, `gitignore`. `package.json` n’a pas de section runtime `dependencies`.

### Livrable 4 : Raffinement empirique daisyUI

- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/templates/daisyui/src/input.css:13`, `crates/sbfb-factory/src/templates/daisyui/src/input.css:17`, `crates/sbfb-factory/src/templates/daisyui/app.css:1`, `crates/sbfb-factory/src/templates/daisyui/app.css:2`
- Evidence :

```css
13: @import "tailwindcss" source(none);
14: @source "../index.html";
15: @source "../app.js";
17: @plugin "daisyui" {
18:   themes: false;
19: }
```

```css
22: @plugin "daisyui/theme" {
23:   name: "sbfb-reflect";
24:   default: true;
```

Scan de `app.css` : `.btn=49`, `.card=11`, `.badge=5`; `night=0`, `dracula=0`, `synthwave=0`; seul `data-theme=sbfb-reflect`. URLs absolues trouvées : `https://tailwindcss.com` et `http://www.w3.org/2000/svg`, toutes deux allowlistées.

Point vigilance : `scripts/vendor-anime.mjs:7` contient le texte commentaire `type=module`, mais pas le littéral `<script type="module">`. Le README contient ce littéral à `README.md:40`, mais le gate classe README en `Skip`.

### Livrable 5 : Help `process prompt --kind`

- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/process.rs:7`, `crates/sbfb-factory/src/main.rs:155`
- Evidence :

```rust
7: const PROMPT_KINDS: &[&str] = &[
8:     "base",
9:     "universal",
...
15:    "phase-auditor",
22:    "app-authoring",
```

```rust
155: /// Assemble a portable prompt by kind
157: /// Prompt kind (PROMPT_KINDS): base, universal, handoff, preflight,
158: /// phase-review, commit-body, audit-gate, phase-auditor, app-authoring
```

Le verbe CLI réel est bien `Create` dans `main.rs:42-56`; aucune trace d’un verbe `new` dans le bloc capacité.

### Livrable 6 : Tests + delta

- Statut : PARTIEL
- Fichier(s) : `crates/sbfb-factory/src/template_engine.rs:733`, `crates/sbfb-factory/src/gates.rs:815`, `crates/sbfb-factory/src/llm_bridge.rs:370`
- Evidence :

```rust
733: fn test_create_daisyui_template() {
740:     create("daisyui", "test-daisy", out.to_str().unwrap()).unwrap();
745:     assert!(out.join("src/input.css").exists(), "subdir file must exist");
763:     assert!(html.contains("src=\"vendor/anime.umd.js\""));
772:     assert!(anime.contains("anime.js v4.5.0"));
```

```rust
815: fn test_csp_gate_daisyui_template_passes() {
823:     template_engine::create("daisyui", "gate-daisy", out.to_str().unwrap()).unwrap();
825:     let r = run_gate_csp_authoring(&out).unwrap();
826:     assert!(r.passed, ...);
```

```rust
370: fn assemble_prompt_surfaces_non_authoritative_capability_block() {
382: assert!(result.contains("non-authoritative"));
384: assert!(result.contains("chat_history_authoritative=false"));
389: assert!(result.contains("sbfb-factory create --template daisyui --name"));
400: history_at < block_at && block_at < msg_at
```

Les assertions sont utiles et couvrent réellement les livrables. Le diff staged ajoute exactement 7 nouveaux tests. Exécution ciblée : `cargo nextest run -p sbfb-factory --locked ...` donne `8 tests run: 8 passed`.

Partiel uniquement sur le chiffre absolu : dans cet environnement, `cargo nextest list --workspace --locked` compte `710`, pas `1990`; `--all-targets` échoue au listing sur `nexus-executor::bench/cold_start` car le bench dépasse son budget. Donc je confirme le `+7` staged, pas le total `1983 -> 1990`.

## Resume final

- Total livrables : 6
- Confirmes : 5
- Gaps : 0
- Partiels : 1