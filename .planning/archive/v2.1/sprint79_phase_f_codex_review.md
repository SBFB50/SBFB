### Livrable 1 : Promotion du pack daisyUI
- Statut : CONFIRME
- Fichier(s) : `docs/factory/knowledge/daisyui/MANIFEST.json:18`, `docs/factory/knowledge/daisyui/README.md:4`
- Evidence :
```text
git diff --staged --name-status :
R100 examples/.../daisyui/COMPONENTS.md -> docs/factory/knowledge/daisyui/COMPONENTS.md
R100 examples/.../daisyui/MANIFEST.json -> docs/factory/knowledge/daisyui/MANIFEST.json
R100 examples/.../daisyui/README.md -> docs/factory/knowledge/daisyui/README.md
R100 examples/.../daisyui/components.json -> docs/factory/knowledge/daisyui/components.json
R100 examples/.../daisyui/docs-llms.txt -> docs/factory/knowledge/daisyui/docs-llms.txt
R100 examples/.../daisyui/synthesis.json -> docs/factory/knowledge/daisyui/synthesis.json
R100 examples/.../daisyui/theming.json -> docs/factory/knowledge/daisyui/theming.json
```
Le dossier source existe mais est vide : `exists=True`, `count=0`.

### Livrable 2 : `classes-bank.json`
- Statut : CONFIRME
- Fichier(s) : `docs/factory/knowledge/daisyui/classes-bank.json:3`, `docs/factory/knowledge/daisyui/classes-bank.json:140`, `docs/factory/knowledge/daisyui/classes-bank.json:162`
- Evidence :
```text
ConvertFrom-Json : count 16
missing : aucun
bad_csp : aucun
csp_class distribution : safe=11, safe-build=3, safe-composite=1, network-exfil-if-remote=1
```
Extraits :
```text
135: "name": "svg-icon-paint",
140: "csp_class": "safe-build",
141: "csp_note": "CORRECTION load-bearing : fill-*/stroke-* SE COMPILENT..."
157: "name": "media-local-only",
162: "csp_class": "network-exfil-if-remote",
163: "csp_note": "Une url() http(s)/protocol-relative distante est BLOQUEE..."
```
La seule entrée détectée par regex comme contenant `https?` hors `network-exfil-if-remote` est `svg-icon-paint`, mais elle cite explicitement `https://...` comme vecteur réseau distinct bloqué ; le snippet utilise `fill="var(--color-primary)"`.

### Livrable 3 : `.gitattributes`
- Statut : CONFIRME
- Fichier(s) : `docs/factory/knowledge/daisyui/.gitattributes:1`
- Evidence :
```text
1:# Extracted daisyUI 5.5.23 / Tailwind 4.3.1 knowledge corpus...
2:# hand-written code. Its bytes are hashed (16-hex blake3) in MANIFEST.json...
4:# rely on the repo-root .gitattributes) and disable git whitespace linting...
5:* text eol=lf -whitespace
```

### Livrable 4 : MANIFEST recalculé et testé
- Statut : CONFIRME
- Fichier(s) : `docs/factory/knowledge/daisyui/MANIFEST.json:31`, `crates/sbfb-factory/tests/daisyui_manifest.rs:26`
- Evidence :
```text
31: "freshness": {
36: "hash_convention": "blake3(file_bytes).to_hex()[..16]..."
38: "components.json": "01632e0b4a95dad4",
43: "README.md": "a01b86a763bcc2ea",
44: "classes-bank.json": "ccc1e9fae1649876"
```
Le test est non-vacuous :
```text
73: assert_eq!(computed_keys, expected_keys, ...)
81: assert_eq!(computed, expected, ...)
87: assert_eq!(manifest["versions"]["daisyui"].as_str(), Some("5.5.23"), ...)
92: assert_eq!(manifest["versions"]["tailwindcss"].as_str(), Some("4.3.1"), ...)
97: assert!(manifest.get("freshness").is_some(), ...)
```
Commande exécutée : `cargo test -p sbfb-factory --locked daisyui_manifest` -> `1 passed`.

### Livrable 5 : Corrections factuelles in-pack
- Statut : CONFIRME
- Fichier(s) : `docs/factory/knowledge/daisyui/README.md:36`, `docs/factory/knowledge/daisyui/components.json:229`, `docs/factory/knowledge/daisyui/synthesis.json:4`
- Evidence :
```text
README.md:36: fill-* / stroke-* se compilent en CSS statique...
README.md:36: vrai risque = absence de build Tailwind runtime + purge @source
components.json:233: ils se compilent en CSS statique mais ne sont pas fiables...
synthesis.json:4: SOURCE UNIQUE = BLOB_SERVE_CSP ... csp-contract.json ...
synthesis.json:4: ... frame-ancestors * ; sandbox allow-scripts ...
```
Autres corrections trouvées : `components.json:574`, `components.json:696`, `components.json:1762`, `components.json:2086`, `components.json:2442`. Recherche `8 themes|8 thèmes|huit thèmes` : aucun match. Les seuls matchs `fail to compile/ne compilent pas` sont des formulations correctives qui disent explicitement que l’ancienne affirmation était fausse.

### Livrable 6 : Extension `app-authoring.md`
- Statut : CONFIRME
- Fichier(s) : `prompts/agent/app-authoring.md:168`, `prompts/agent/app-authoring.md:182`, `crates/sbfb-factory/src/process.rs:985`
- Evidence :
```text
168: Build recipe ... tailwindcss -i src/input.css -o app.css --minify
170: @import "tailwindcss" source(none); ... @source "./index.html"
175: Theme ... sbfb-reflect
177: aucun des 35 thèmes built-in
182: Per-class CSP taxonomy ... classes-bank.json
```
Pointeurs path+hash :
```text
217: docs/factory/knowledge/daisyui/components.json — 01632e0b4a95dad4
219: COMPONENTS.md — 69306d7652712df8
220: classes-bank.json — ccc1e9fae1649876
222: theming.json — f44553ffe9ba2cfe
224: synthesis.json — fc084fcd88eb8f44
```
Test marqueurs :
```text
985: const MARKERS: &[&str] = &[
986: "daisyUI 5.5.23",
987: "source(none)",
988: "sbfb-reflect",
989: "aucun des 35 thèmes built-in",
993: for provider in ["claude", "local"] {
```
Commande exécutée : `cargo test -p sbfb-factory --locked app_authoring_prompt_surfaces_daisyui_markers` -> `1 passed`.

### Livrable 7 : Fix `check-frontier-contracts.sh`
- Statut : CONFIRME
- Fichier(s) : `scripts/check-frontier-contracts.sh:166`
- Evidence :
```text
166: # Union of every blake3 16-hex digest recorded across all pack MANIFESTs.
167: # `-h` is load-bearing: with >=2 packs `grep` over multiple files prefixes each
168: # match with `filename:`, which would defeat the whole-line `grep -qxF` below
170: manifest_hashes="$(find "$KNOW_DIR" -name MANIFEST.json -exec grep -hoE ...
182: if ! printf '%s\n' "$manifest_hashes" | grep -qxF "$h"; then
```
Root-cause vérifiée :
```text
grep -oE  -> docs/factory/knowledge/animejs/MANIFEST.json:8faa36021466192a
grep -hoE -> 8faa36021466192a
```
Commande exécutée : `bash scripts/check-frontier-contracts.sh` -> exit 0, `check-frontier-contracts: clean`.

### Livrable 8 : +2 tests et non-régression animejs
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/tests/daisyui_manifest.rs:26`, `crates/sbfb-factory/src/process.rs:974`, `crates/sbfb-factory/tests/animejs_manifest.rs:25`
- Evidence :
```text
daisyui_manifest.rs:26: fn daisyui_manifest_hashes_match_promoted_layers()
daisyui_manifest.rs:60: assert!(!bytes.contains(&b'\r'), ...)
daisyui_manifest.rs:73: assert_eq!(computed_keys, expected_keys, ...)
process.rs:974: fn app_authoring_prompt_surfaces_daisyui_markers()
animejs_manifest.rs:25: fn animejs_manifest_hashes_match_promoted_layers()
```
Commandes exécutées :
```text
cargo test -p sbfb-factory --locked daisyui_manifest -> 1 passed
cargo test -p sbfb-factory --locked app_authoring_prompt_surfaces_daisyui_markers -> 1 passed
cargo test -p sbfb-factory --locked animejs_manifest_hashes_match_promoted_layers -> 1 passed
git diff -- crates/sbfb-factory/tests/animejs_manifest.rs -> aucun diff
```

## Resume final
- Total livrables : 8
- Confirmes : 8
- Gaps : 0
- Partiels : 0