### Livrable 1 : `prompts/agent/app-authoring.md`
- Statut : CONFIRME
- Fichier(s) : `prompts/agent/app-authoring.md:1`, `prompts/agent/app-authoring.md:27`, `prompts/agent/app-authoring.md:46`, `prompts/agent/app-authoring.md:104`, `prompts/agent/app-authoring.md:141`, `prompts/agent/app-authoring.md:156`
- Evidence :
```md
27: ## Vendorization doctrine — `UMD classic-script jamais type=module`
29: Vendor `anime.umd.js` v4.5 under `vendor/` and load it with a **classic** `<script>` tag
31: `anime.svg.*`, `anime.utils.*`, `anime.eases`, `anime.stagger`). Never use `type=module`
```
```md
46: ## The 9 hard CSP pitfalls
53: 1. **`motion-path cx=0`** — an SVG element moved by `svg.createMotionPath` must keep
59: 2. **`box-shadow STATIQUE`** — never animate or transition `box-shadow`
69: 4. **`morphTo mono-trace`** — `svg.morphTo` requires the **same** target type
75: 5. **`prefers-reduced-motion → état-final`** — anime does **not** short-circuit anything
100: 9. **`UMD classic-script jamais type=module`** — recap of the vendorization doctrine
```
Les `source_ref` critiques sont cohérents : README confirme les pièges CSP aux lignes `64-68` et le vendor `anime.umd.js` aux lignes `81-82`; `PRIMITIVES.md` confirme notamment UMD/zéro réseau (`112`, `627`, `1014`, `1454`), `cx=0` (`1085`, `3168`, `3530`), `box-shadow STATIQUE` (`750`, `2105`, `3356`, `3572`), `morphTo` mono-trace (`1134`, `1171`, `1178`) et reduced-motion état-final (`664`, `998`, `1014`, `1109`, `1255`, `1380`). Les hashes 16-hex du prompt correspondent à `MANIFEST.json:35-44`.

### Livrable 2 : ajout `"app-authoring"` à `PROMPT_KINDS`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/process.rs:7`
- Evidence :
```rust
16:     // app-authoring (Sprint 79 Phase C, decision D2): surfaces the anime.js
17:     // CSP-safe authoring mastery to app-building agents. Resolves through the
18:     // generic `prompt_filename` arm to `prompts/agent/app-authoring.md` (no
22:     "app-authoring",
```
`prompt_filename` reste générique (`other => format!("{other}.md")`) aux lignes `79-83`. `KIND_ALIASES` reste sans entrée app-authoring aux lignes `25-29`. `PROVIDERS` reste inchangé aux lignes `31-41`. Je n’ai pas trouvé de promesse future dans le commentaire de provenance.

### Livrable 3 : test `app_authoring_prompt_surfaces_csp_markers`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/process.rs:915`
- Evidence :
```rust
925:         const MARKERS: &[&str] = &[
926:             "box-shadow STATIQUE",
927:             "motion-path cx=0",
928:             "morphTo mono-trace",
929:             "prefers-reduced-motion → état-final",
930:             "UMD classic-script jamais type=module",
```
Le test boucle bien sur `["claude", "local"]` aux lignes `932-940` et fait des assertions réelles avec `out.contains(marker)`. Le chemin `local` applique bien `strip_cloud_references` dans `prompt_data` aux lignes `837-839`. Test exécuté : `cargo test -p sbfb-factory process::tests::app_authoring_prompt_surfaces_csp_markers --locked` → OK.

### Livrable 4 : invariant `prompt_kinds_resolve_to_existing_files`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/process.rs:895`
- Evidence :
```rust
903:         let root = repo_root();
904:         for kind in PROMPT_KINDS {
905:             let path = root.join("prompts/agent").join(prompt_filename(kind));
906:             assert!(
907:                 path.exists(),
```
Comme `PROMPT_KINDS` contient maintenant `"app-authoring"` ligne `22`, cet invariant couvre le nouveau kind. Test exécuté : `cargo test -p sbfb-factory process::tests::prompt_kinds_resolve_to_existing_files --locked` → OK.

## Résumé final

- Total livrables : 4
- Confirmés : 4
- Gaps : 0
- Partiels : 0

Contrôles transverses : pas de modification de `Cargo.lock`. Caveat Git : l’audit ci-dessus porte sur la working tree actuelle; `prompts/agent/app-authoring.md` est présent mais encore non suivi Git (`??`), et `crates/sbfb-factory/src/process.rs` est modifié non commité.

