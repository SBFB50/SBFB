### Livrable 1 : module `phase.rs`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/main.rs:17`, `crates/sbfb-factory/src/phase.rs:37-168`, `crates/sbfb-factory/src/phase.rs:176-240`
- Evidence :
```rust
63:fn label_from_filename(name: &str, sprint: u32, kind: &str) -> Option<String> {
64:    let lower = name.to_ascii_lowercase();
67:    let mid = lower.strip_prefix(&prefix)?.strip_suffix(&suffix)?;
68:    if is_phase_label(mid) {
```
`discover_phase_artifacts` scanne `read_dir`, garde le vrai `path`, trie par `phase_order_key` (`phase.rs:79-92`). `discover_phase_labels` unionne `preflight/review/codex_review` (`99-109`). `find_phase_artifact` fait le lookup case-insensitive (`115-129`). `next_phase_label` gère le rollover bijectif (`139-162`) et les tests vérifient `z->aa`, `az->ba`, `zz->aaa`, ordre `aa` après `z`, découverte lowercase + uppercase (`176-240`).

### Livrable 2 : `detect_current_phase`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/process.rs:160-185`, `crates/sbfb-factory/src/process.rs:974-993`
- Evidence :
```rust
165:    if active_dir
166:        .join(format!("sprint{sprint}_verification.md"))
167:        .exists()
169:        return "done".to_string();
```
La phase courante vient ensuite de `discover_phase_artifacts(..., "review")`, puis `next_phase_label` + `display_label` (`174-184`). Le test lowercase au-delà de G (`a,b,h,i`) attend `J`, puis `verification.md` attend `done` (`981-993`).

### Livrable 3 : `status_sprint_data`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/process.rs:296-334`
- Evidence :
```rust
307:    let phases: Vec<PhaseStatusEntry> = crate::phase::discover_phase_labels(&active_dir, s)
310:            let review_path = crate::phase::find_phase_artifact(&active_dir, s, &label, "review");
316:                letter: crate::phase::display_label(&label),
326:                has_codex: crate::phase::find_phase_artifact(
```
Une entrée est produite par label découvert sur disque. Les champs publics restent en majuscule via `display_label`.

### Livrable 4 : regex `[A-Z]+[0-9]?`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/process.rs:545-549`, `crates/sbfb-factory/src/sprint_history.rs:151-158`
- Evidence :
```rust
548:const PHASE_TITLE_RE: &str =
549:    r"^(feat|fix|docs|chore|test|refactor)\([^)]+\):\s*Sprint\s+(\d+)\s+Phase\s+([A-Z]+[0-9]?)";
```
`PHASE_RE` utilise aussi `([A-Z]+[0-9]?)` à `sprint_history.rs:157`. Les tests couvrent `N`, `AA`, `F1` (`process.rs:997-1011`, `sprint_history.rs:1167-1179`).

### Livrable 5 : `audit_commit_data`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/process.rs:602-668`, `crates/sbfb-factory/tests/process_cli.rs:562-641`
- Evidence :
```rust
610:        let phase_label = caps[3].to_ascii_lowercase();
617:            let review_path =
618:                crate::phase::find_phase_artifact(&active_dir, sprint_num, &phase_label, "review");
```
Les lookups review/codex actifs et archives passent par `find_phase_artifact` (`634-640`, `649-668`). Le test CLI crée des artefacts actifs minuscules et vérifie qu’un titre `Phase A` passe (`586-641`).

### Livrable 6 : `sprint_history`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/sprint_history.rs:252-287`, `441-523`, `670-697`
- Evidence :
```rust
258:    for label in crate::phase::discover_phase_labels(dir, sprint) {
259:        if let Some(review) = crate::phase::find_phase_artifact(dir, sprint, &label, "review") {
266:        } else if crate::phase::find_phase_artifact(dir, sprint, &label, "preflight").is_some() {
```
`build_phase_histories` unionne labels disque + commits, trie via `phase_order_key`, lookup case-insensitive pour preflight/review/codex, affichage majuscule (`450-519`). `build_preflight_bilan` itère les artefacts découverts et émet le vrai nom disque via `file_name()` (`679-696`).

### Livrable 7 : tests de non-régression
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/phase.rs:176-240`, `crates/sbfb-factory/src/process.rs:974-1012`, `crates/sbfb-factory/src/sprint_history.rs:1167-1204`
- Evidence :
```rust
981:        for label in ["a", "b", "h", "i"] {
989:        assert_eq!(detect_current_phase(active, 79), "J");
992:        std::fs::write(active.join("sprint79_verification.md"), "done").unwrap();
993:        assert_eq!(detect_current_phase(active, 79), "done");
```
Tests exécutés : `cargo test -p sbfb-factory --locked phase` passe, et `cargo test -p sbfb-factory --locked sprint_summary_is_unbounded_and_case_insensitive` passe.

## Résumé final
- Total livrables : 7
- Confirmés : 7
- Gaps : 0
- Partiels : 0

Passe adversariale : les anciens caps trouvés par `rg` sont uniquement dans commentaires/tests de régression. Aucun changement détecté dans `Cargo.lock`, `Cargo.toml` ou `crates/sbfb-factory/Cargo.toml`; pas de dépendance ajoutée.