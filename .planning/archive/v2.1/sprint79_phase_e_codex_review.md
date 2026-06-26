J’ai audité le working tree actuel non committé. Contrôles exécutés avec succès : `cargo test -p nexus-core-rs csp --locked` (4 tests), `cargo test -p sbfb-factory test_csp_gate --locked` (7 tests), `cargo test -p sbfb-factory test_pipeline_csp_gate_blocks_even_with_skip_gates --locked`, `bash scripts/check-frontier-contracts.sh`, `npm run check:csp`.

### Livrable 1 : Source unique CSP Rust
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/csp.rs:33`, `:36`, `:39`, `:48`, `:63`; `crates/nexus-core-rs/src/lib.rs:44`, `:104`
- Evidence :
```rust
pub const BLOB_SERVE_CSP: &str = "... sandbox allow-scripts";
pub const BLOB_SERVE_COOP: &str = "same-origin";
pub const BLOB_SERVE_COEP: &str = "require-corp";
pub const CSS_URL_ALLOW: &[&str] = &[...];
pub fn none_directives(csp: &str) -> Vec<&str> { ... }
```
Tests inline confirmés à `csp.rs:81-158` : 3 tests `none_directives` + miroir JSON.

### Livrable 2 : Mirror `csp-contract.json`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/csp-contract.json:2-16`, `crates/nexus-core-rs/src/csp.rs:123-157`
- Evidence :
```json
"csp": "default-src 'self' ... sandbox allow-scripts",
"none_directives": ["connect-src", "worker-src", "frame-src", "object-src", "base-uri", "form-action"],
"css_url_allow": ["http://www.w3.org/2000/svg", ...]
```
Le test `csp_contract_json_mirrors_the_rust_consts` compare `csp`, `none_directives` et `css_url_allow` contre les consts Rust.

### Livrable 3 : Re-export daemon + call-site inchangé
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon-core/src/blob_serve.rs:277-284`, `crates/nexus-shell-daemon/src/http.rs:554-566`
- Evidence :
```rust
pub use nexus_core_rs::csp::{BLOB_SERVE_COEP, BLOB_SERVE_COOP, BLOB_SERVE_CSP};

headers.insert(
    "content-security-policy",
    blob_serve::BLOB_SERVE_CSP.parse().unwrap(),
);
```
Comparaison HEAD avant factorisation vs nouvelle constante : `old_length=215`, `new_length=215`, `byte_identical=True`.

### Livrable 4 : Gate Rust CSP authoring
- Statut : PARTIEL
- Fichier(s) : `crates/sbfb-factory/src/gates.rs:7`, `:194-290`, `:318-363`, `:386-470`; `crates/sbfb-factory/Cargo.toml:11-13`
- Evidence :
```rust
use nexus_core_rs::csp::CSS_URL_ALLOW;
const CSP_RULES: &[CspRule] = &[ ... ];
pub fn run_gate_csp_authoring(workspace: &Path) -> Result<GateResult, FactoryError> {
    for entry in WalkDir::new(&canonical).follow_links(false) { ... }
}
```
Le scanner, `WalkDir`, les tiers `Scanned/Vendored/Skip`, `type=module`, `CSS_URL_ALLOW`, et les 6 directives `'none'` + `default-src` existent.  
Partiel : la fonction de production n’importe pas `BLOB_SERVE_CSP`; la seule référence à `BLOB_SERVE_CSP` dans `gates.rs` est le test anti-drift à `gates.rs:1137`. Donc l’exigence “importe `BLOB_SERVE_CSP` / `CSS_URL_ALLOW`” n’est réalisée qu’à moitié côté code de production. Aucun edge vers `nexus-shell-daemon-core` : `Cargo.toml` dépend de `nexus-core-rs`, et `cargo tree -p sbfb-factory --locked | rg "nexus-shell-daemon-core|nexus-core-rs"` ne montre que `nexus-core-rs`.

### Livrable 5 : Test anti-drift cross-crate
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/gates.rs:1132-1143`
- Evidence :
```rust
use nexus_core_rs::csp::{BLOB_SERVE_CSP, none_directives};
for d in none_directives(BLOB_SERVE_CSP) {
    assert!(CSP_RULES.iter().any(|r| r.directive == d), ...);
}
```
Oui, si une directive `'none'` perd entièrement son entrée `CSP_RULES`, ce test échoue. Nuance : il vérifie la présence d’une règle par nom de directive, pas que les patterns soient exhaustifs ou non vides.

### Livrable 6 : Wiring pipeline hors `skip_gates`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/pipeline.rs:27-57`, `:195-209`
- Evidence :
```rust
if !skip_gates { ... FG5 ... FG6 ... }

let fg_csp = gates::run_gate_csp_authoring(workspace)?;
if !fg_csp.passed {
    return Err(format!("FG-CSP-authoring FAIL: {}", issues.join("; ")).into());
}
```
Le test `test_pipeline_csp_gate_blocks_even_with_skip_gates` injecte `fetch(...)`, appelle `run_publish_pipeline(..., true)`, et asserte l’erreur `FG-CSP-authoring FAIL`.

### Livrable 7 : `check-csp.mjs`
- Statut : CONFIRME
- Fichier(s) : `examples/daisyui-animejs-showcase/scripts/check-csp.mjs:3-22`, `:35-38`, `:48-82`, `:94-109`
- Evidence :
```js
// base-uri 'none'; form-action 'none'; ...
const contract = JSON.parse(readFileSync(join(repoRoot, "crates", "nexus-core-rs", "csp-contract.json"), "utf8"));
const CSS_URL_ALLOW = contract.css_url_allow;
[/<form[\s/][^>]*action\s*=\s*["']?(?:https?:|\/\/)/i, ...]
const MODULE_SCRIPT = /<script[\s/][^>]*type\s*=\s*["']module["']/i;
```
Le commentaire mentionne bien `vendor/anime.umd.js` à `:21`. `npm run check:csp` sort `CSP conformance: OK`.

### Livrable 8 : `check-frontier-contracts.sh`
- Statut : CONFIRME
- Fichier(s) : `scripts/check-frontier-contracts.sh:130-156`
- Evidence :
```sh
CSP_FILE="crates/nexus-core-rs/src/csp.rs"
for directive in \
  "connect-src 'none'" \
  ...
  "form-action 'none'"; do
```
Commande exécutée : `bash scripts/check-frontier-contracts.sh` -> exit 0.

### Livrable 9 : Docs factory + threat model
- Statut : CONFIRME
- Fichier(s) : `docs/factory/FACTORY_GATES.md:105-158`, `:232-255`; `docs/security/THREAT_MODEL.md:741-773`
- Evidence :
```md
### FG-CSP-authoring — Conformite CSP au publish (Sprint 79 Phase E)
... bloque la distribution ...
**Non-delegable.** ... hors du bloc `skip_gates`
### 13.1 Gate CSP authoring au publish (Sprint 79 Phase E)
```

Points de risque vérifiés :
- Gate hors `skip_gates` : confirmé.
- CSP byte-identique : confirmé.
- Pas de nouvel edge `sbfb-factory -> nexus-shell-daemon-core` : confirmé.
- Faux-négatif évident : `<form action="/local">` n’est pas bloqué par le gate car la regex `form-action` ne cible que `https?:` ou `//` (`gates.rs:251-260`), alors que `form-action 'none'` bloque toute soumission.
- Faux-négatif évident : une URL protocol-relative hors `link/script/@import/url()`, par exemple `<img src="//evil/x.png">`, n’est pas couverte par le catch-all `ABSOLUTE_URL_PATTERN` qui ne matche que `https?://` (`gates.rs:270-288`, `:303`, `:454-460`).
- Faux-positif assumé : toute URL absolue `http(s)` non allowlistée dans un asset scanné est rejetée même si elle est seulement affichée comme texte (`gates.rs:454-460`; documenté dans `FACTORY_GATES.md:141-145`).

## Resume final
- Total livrables : 9
- Confirmes : 8
- Gaps : 0
- Partiels : 1