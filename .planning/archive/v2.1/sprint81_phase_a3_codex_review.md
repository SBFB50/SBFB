Audit fait sur le working tree courant, sans rejouer les runs live.

### Livrable 1 : Baseline curated `sprint81_t2_baseline_098.json`
- Statut : PARTIEL
- Fichier(s) : `.planning/active/sprint81_t2_baseline_098.json:10`, `:11`, `:21-26`; `crates/nexus-shell-daemon/src/feed_sync.rs:596`; `crates/nexus-shell-daemon/src/http.rs:3153`; `crates/nexus-worker-core/src/consent.rs:183`; `crates/nexus-shell-daemon/src/local_worker.rs:183`, `:306`; `crates/nexus-shell-daemon/src/runtime.rs:643`; `crates/nexus-core-rs/src/docs.rs:106`
- Evidence :
```rust
596:     let internal = headers
597:         .get("x-sbfb-feed-internal")
598:         .and_then(|v| v.to_str().ok())
599:         == Some("1");
600:     if !internal {
```
```rust
3153:         if let Err(e) = state.blob_serve_cache.load(
3154:             &hash,
3155:             &zip_bytes,
3156:             blob_serve::DEFAULT_MAX_DECOMPRESSED_BYTES,
3159:             return (StatusCode::BAD_REQUEST, format!("invalid archive: {e}")).into_response();
```
```rust
183:     pub fn default_for(own_node_id: impl Into<String>) -> Self {
184:         Self {
185:             level: ConsentLevel::OwnProjects,
397:         ConsentLevel::OwnProjects => {
398:             if task.project_id != consent.own_node_id {
399:                 return AllowOutcome::Reject(RejectReason::NotOwnProject);
```
```rust
183:         let (paths, sbfb_home) = provision(project_doc, user_sbfb_home).await?;
191:         let child = cmd.spawn()?;
306:     let ticket = project_doc
307:         .share_write()
309:         .map_err(|e| anyhow::anyhow!("share_write project doc: {e}"))?
```
- Si GAP : les claims code (a-e) sont confirmés. Le gap est de schéma/vocabulaire : le JSON a `"verdict": "BLOCK"` nu à `:11` et des valeurs `FAIL{...}` dans `per_test` à `:21-26`, alors que le livrable demande un vocabulaire fermé `PASS/BLOCK{...}/NOT-RUN`.

### Livrable 2 : Archive run `SBFB_INTEGRATION=1`
- Statut : CONFIRME
- Fichier(s) : `.planning/active/sprint81_a3a_integration_run_098.txt:4`, `:5`, `:21-24`, `:128-134`
- Evidence :
```text
4:  Nextest run ID 4e9bc33c-d15b-4139-9515-497de6ccced1
5:     Starting 10 tests across 2 binaries
21:     thread 'test_cross_daemon_blob_transfer' ... panicked
23:       left: 400
128:     Summary [  33.244s] 10 tests run: 4 passed, 6 failed, 0 skipped
```
- Scrub vérifié : aucune occurrence de `C:\Users\FlowUP`, aucune IPv4, aucun token/bearer/password/clé détecté.

### Livrable 3 : `.gitignore` ajoute `*.redb`
- Statut : CONFIRME
- Fichier(s) : `.gitignore:8-11`
- Evidence :
```gitignore
8: # iroh-docs stores carry the NamespaceSecret (write capability) — `*.db`
9: # does NOT match `.redb` (S81 Phase A3a, belt-and-suspenders for VPS
10: # store copies pulled next to the repo).
11: *.redb
```
- Vérification : `git check-ignore -v docs.redb` pointe bien sur `.gitignore:11:*.redb`.

### Livrable 4 : Préflight `sprint81_phase_a3_preflight.md`
- Statut : CONFIRME
- Fichier(s) : `.planning/active/sprint81_phase_a3_preflight.md:1`, `:3`, `:17`, `:128-160`, `:197-205`, `:303-322`
- Evidence :
```markdown
1: # Sprint 81 Phase A3 — Préflight G8
3: > **Verdict : PLAN-ADAPT + SPLIT A3a / A3b.**
17: > Workflow G8 : 5 scans ...
128: ### A3a — Baseline transport LIVE 0.98 + ressources rig (0 fix code)
160: ### A3b — Fix convergence coordinateur
```
- Structure G8 confirmée : table des 5 scans à `:197-205`, contre-vérification/prémisses à `:211-225`, carries à `:303-322`.

### Livrable 5 : Review `sprint81_phase_a3a_review.md`
- Statut : CONFIRME
- Fichier(s) : `.planning/active/sprint81_phase_a3a_review.md:1`, `:9`
- Evidence :
```markdown
1: # Sprint 81 Phase A3a — Review
3: > Phase A3a ...
9: ## Verdict: PASS-PENDING
```
- Vérification : `rg "^## Verdict"` retourne 1 occurrence.

### Livrable 6 : Invariant 0-fix-code
- Statut : CONFIRME
- Fichier(s) : `.gitignore:8-11`, `.planning/active/sprint81_t2_baseline_098.json:1`, `.planning/active/sprint81_a3a_integration_run_098.txt:1`
- Evidence :
```text
git status --short:
 M .gitignore
?? .planning/active/sprint81_a3a_integration_run_098.txt
?? .planning/active/sprint81_phase_a3_preflight.md
?? .planning/active/sprint81_phase_a3a_review.md
?? .planning/active/sprint81_t2_baseline_098.json
```
- Vérification : `git diff --name-only -- '*.rs' '*.ts' '*.tsx' '*.sh' '*.toml'` ne retourne rien. `git status --short crates web tests Cargo.toml Cargo.lock package.json web/package.json` ne retourne rien.

### Livrable 7 : Invariant 0-bump
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/task.rs:61`, `crates/nexus-core-rs/src/seed.rs:51`, `crates/nexus-core-rs/src/shard_plan.rs:77`
- Evidence :
```rust
crates/nexus-core-rs/src/task.rs:61: pub const TASK_FORMAT_VERSION: u16 = 1;
crates/nexus-core-rs/src/seed.rs:51: pub const SEED_FORMAT_VERSION: u16 = 1;
crates/nexus-core-rs/src/shard_plan.rs:77: pub const SHARD_PLAN_FORMAT_VERSION: u16 = 1;
```
- Vérification : `git diff --name-only -- crates` et `git diff --name-only -- crates/nexus-core-rs/src` ne retournent rien. Aucun fichier `crates/` n’est modifié.

## Résumé final
- Total livrables : 7
- Confirmés : 6
- Gaps : 0
- Partiels : 1