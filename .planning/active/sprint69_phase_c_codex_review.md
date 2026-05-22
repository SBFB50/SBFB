Audit basé uniquement sur l’état courant du dépôt.

### Livrable 1 : `static-reader/index.html`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/templates/static-reader/index.html:10`
- Evidence :
```html
10:    :root {
11:      --bg: #1a1a2e; --surface: #16213e; --text: #e0e0e0;
12:      --accent: #0f3460; --highlight: #e94560; --muted: #888;
```
```html
64:  <nav>
65:    <div class="btn" role="button" tabindex="0" id="prev-btn" onclick="navigate(-1)">Prev</div>
66:    <span id="page-indicator">1 / 1</span>
67:    <div class="btn" role="button" tabindex="0" id="next-btn" onclick="navigate(1)">Next</div>
```
```js
117:    function saveCursor() {
118:      var bridge = window._bridge;
119:      if (bridge) {
120:        bridge.setStorage("reader_cursor", { value: currentSection }).catch(function() {});
```
```js
136:    document.addEventListener("keydown", function(e) {
137:      if (e.key === "ArrowLeft") navigate(-1);
138:      if (e.key === "ArrowRight") navigate(1);
```
```js
145:    bridge.getNodeStatus().then(function(status) {
146:      document.getElementById("status").textContent =
147:        (status.peers || 0) + " peers connected";
```

### Livrable 2 : `static-reader/sbfb-bridge.js`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/templates/static-reader/sbfb-bridge.js:28`, `crates/sbfb-factory/src/template_engine.rs:112`
- Evidence :
```js
28:  submitTask(payload) { return this._call("task_submit", payload || {}); }
29:  getStorage(key) { return this._call("storage_get", { key: key }); }
30:  setStorage(key, value) { return this._call("storage_set", Object.assign({ key: key }, value || {})); }
31:  getIdentityPubkey() { return this._call("identity_pubkey", {}); }
32:  getNodeStatus() { return this._call("node_status", {}); }
```
```rust
112:        id: "static-reader",
113:        version: "1.0.0",
114:        files: STATIC_READER_TEMPLATE,
115:        description: "SBFB reader app created with sbfb-factory",
116:        category: "content",
117:        bridge_methods: &["storage_get", "storage_set", "identity_pubkey"],
```
Le diff avec `templates/static/sbfb-bridge.js` montre une seule différence : l’ajout de `getIdentityPubkey()`.

### Livrable 3 : `static-reader/README.md`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/templates/static-reader/README.md:1`, `crates/sbfb-factory/src/template_engine.rs:75`
- Evidence :
```md
1:# {{name}}
2:
3:SBFB Reader app (v{{version}}) created with sbfb-factory.
```
```md
12:## Adding content
13:
14:Edit the `sections` array in `index.html`. Each section has a `title`
15:and `content` (HTML string). Add as many sections as needed.
```
```md
23:## Validate
24:
25:```bash
26:sbfb-factory validate .
```
```rust
75:    TemplateFile {
76:        name: "README.md",
77:        content: include_str!("templates/static-reader/README.md"),
78:        substitute: true,
```

### Livrable 4 : `static-reader/gitignore`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/templates/static-reader/gitignore:1`
- Evidence :
```gitignore
1:node_modules/
2:dist/
3:*.log
4:.DS_Store
5:Thumbs.db
```

### Livrable 5 : `template_engine.rs`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/template_engine.rs:64`
- Evidence :
```rust
64:const STATIC_READER_TEMPLATE: &[TemplateFile] = &[
65:    TemplateFile {
66:        name: "index.html",
67:        content: include_str!("templates/static-reader/index.html"),
```
```rust
93:struct TemplateConfig {
94:    id: &'static str,
95:    version: &'static str,
96:    files: &'static [TemplateFile],
97:    description: &'static str,
98:    category: &'static str,
99:    bridge_methods: &'static [&'static str],
```
```rust
121:fn find_template(id: &str) -> Result<&'static TemplateConfig, FactoryError> {
122:    TEMPLATES
123:        .iter()
124:        .find(|t| t.id == id)
125:        .ok_or_else(|| FactoryError::TemplateNotFound(id.to_string()))
```
```rust
147:    let bridge = if config.bridge_methods.is_empty() {
148:        None
149:    } else {
150:        Some(sbfb_manifest::BridgeConfig {
151:            methods: config
```
```rust
430:    fn test_validate_static_reader_passes() {
435:        assert!(validate(out.to_str().unwrap()).is_ok());
439:        assert_eq!(m.effective_schema_version(), 2);
440:        assert_eq!(m.category.as_deref(), Some("content"));
443:        assert!(methods.contains(&"storage_get".to_string()));
```

Vérification exécutée : `cargo test -p sbfb-factory --locked static_reader` -> 3 tests passés.

## Résumé final
- Total livrables : 5
- Confirmés : 5
- Gaps : 0
- Partiels : 0