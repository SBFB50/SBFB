# Sprint 37 — Plan

**Kickoff ref** : `sprint37_kickoff.md` (D1..D5 gelees).
**Phases** : A (MANDATORY + P2 batch), B (hash-chain), C (wrap-up).

---

## Phase A — MANDATORY log convergence + .icns + P2 batch audit/review

### §A.1 Log convergence (D1)

1. Modifier `crates/nexus-shell-daemon-core/src/paths.rs` :
   `log_dir()` retourne `<root>/logs/` au lieu de
   `<root>/shell-daemon/logs/`.
2. Verifier que le daemon boot (`runtime.rs`) utilise bien
   `paths::log_dir()` pour `init_logging()`.
3. Modifier `crates/nexus-launcher/Cargo.toml` : ajouter
   `tracing-appender = { workspace = true }` + `tracing` +
   `tracing-subscriber` deps.
4. Refactorer `crates/nexus-launcher/src/main.rs` :
   - Remplacer `LOG_FILE` OnceLock + `lprint!` macro par
     `tracing_appender::rolling::daily(log_dir, "launcher.log")`
     + `tracing_subscriber` init (pattern identique au daemon).
   - `launcher_log_path()` → `launcher_log_dir()` retournant
     `~/.sbfb/logs/`.
   - Garder le panic hook ecrivant dans le meme directory.
5. Test : verifier que `~/.sbfb/logs/` contient `launcher.log`
   ET `daemon.log` apres boot (integration-level, pas automated
   test — les 2 binaires tournent dans des process separes).

### §A.2 .icns macOS (D2)

1. Creer `tools/png-to-icns/Cargo.toml` + `src/main.rs` :
   mini binaire qui prend un PNG en arg et ecrit un .icns
   (tailles 16/32/64/128/256/512/1024 pixels). Dep : `icns 0.3`
   + `image` (pour resize le PNG source).
2. Modifier `scripts/bundle-macos.sh` : appeler
   `cargo run -p png-to-icns -- assets/nexus-launcher.png
   assets/nexus-launcher.icns` avant la copie dans le .app
   bundle.
3. Modifier `configs/macos/Info.plist` : `CFBundleIconFile` →
   `nexus-launcher.icns`.
4. Ajouter `tools/png-to-icns` au workspace members dans
   `Cargo.toml` racine.

### §A.3 P2 batch fixes (D4)

1. **HARDENING compteurs** : modifier `docs/security/HARDENING_ROADMAP.md`
   last_validated → compteurs corrects (936 Rust / ~1939 total +
   mention S37 update).
2. **unwrap_or_default()** : dans `http.rs`, remplacer les 2
   occurrences (L1290, L1402) par :
   ```rust
   match serde_json::to_value(&entry) {
       Ok(body) => (StatusCode::OK, Json(body)).into_response(),
       Err(e) => {
           tracing::error!("serialization failed: {e}");
           (StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "internal"}))).into_response()
       }
   }
   ```
3. **Mutex poisoned tests** : ajouter 3 tests dans `http.rs` qui
   empoisonnent le Mutex (panic dans un thread avec le guard tenu)
   puis verifient que les handlers retournent 500.
4. **Double query project_id** : modifier `validate_result()` dans
   `validator.rs` pour retourner `(ValidationOutcome, Option<TaskRecord>)`
   ou un enum riche qui inclut le `TaskRecord` quand Accepted.
   Le handler `coordinator_submit_result` utilise le `project_id` du
   record retourne au lieu de refaire `db.get_task()`.

### §A.4 Delta tests attendu

- +3 mutex poisoned tests (http.rs)
- +1 test log path convergence (paths.rs ou integration)
- Existants inchanges

### §A.5 Verification Phase A

- `cargo nextest run -p nexus-shell-daemon --locked`
- `cargo nextest run -p nexus-coordinator-rs --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo fmt --all --check`
- `cargo build -p nexus-shell-daemon --release`
- Full fail-fast 3 blocs

---

## Phase B — KudosLedger hash-chain (BLAKE3 + JCS canonical)

### §B.1 Dependencies

1. Ajouter `blake3 = { workspace = true }` a
   `crates/nexus-coordinator-rs/Cargo.toml`.
2. Pas de nouvelle dep externe (nexus-core-rs re-exporte
   `canonical_bytes` et `DOMAIN_KUDOS_V1`).

### §B.2 DB query

1. Ajouter `get_last_entry_hash(project_id: &str) -> Option<String>`
   dans `db.rs` :
   ```sql
   SELECT entry_hash FROM kudos
   WHERE project_id = ?1
   ORDER BY created_at DESC
   LIMIT 1
   ```

### §B.3 Hash computation dans credit()

1. Modifier `kudos_ledger::credit()` :
   - Appeler `db.get_last_entry_hash(project_id)` pour obtenir
     le `prev_hash` (ou `"genesis"` si None).
   - Construire un `KudosEntry` intermediaire avec
     `entry_hash: String::new()` (placeholder pour eviter
     circularite).
   - Calculer les canonical bytes :
     `nexus_core_rs::canonical::canonical_bytes(&hashable, DOMAIN_KUDOS_V1)`
   - Hasher : `let hash = blake3::hash(&canonical);`
   - `entry_hash = hex::encode(hash.as_bytes())`.
   - Inserer l'entree complete avec les 2 champs remplis.

### §B.4 Verification chain (read-only)

1. Ajouter `verify_chain(db: &CoordinatorDb, project_id: &str)
   -> Result<bool>` dans `kudos_ledger.rs` :
   - Lire toutes les entrees du projet ordonnees par created_at ASC
   - Pour chaque entree : re-calculer le hash attendu, verifier
     `entry_hash == expected`, verifier `prev_hash == previous.entry_hash`
   - Retourner `true` si toute la chaine est valide

2. Ajouter `get_project_entries(project_id: &str) -> Vec<KudosEntry>`
   dans `db.rs`.

### §B.5 Tests

- `credit_sets_entry_hash` : credit() produit un entry_hash non-vide
- `credit_chains_prev_hash` : 2eme entree a prev_hash = 1ere entry_hash
- `credit_genesis_hash` : 1ere entree a prev_hash = "genesis"
- `verify_chain_valid` : chain de 3 entrees → true
- `verify_chain_tampered` : modifier 1 entry_hash → false
- `cross_project_chains_independent` : 2 projets ont des chains separees

### §B.6 Delta tests attendu

- +6 tests kudos_ledger.rs / db.rs
- Existants inchanges (les tests existants credit() fonctionnent
  toujours — ils verifiaient que `entry_hash` etait vide, maintenant
  il sera rempli → adapter les assertions)

### §B.7 Verification Phase B

- `cargo nextest run -p nexus-coordinator-rs --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo fmt --all --check`
- Full fail-fast 3 blocs

---

## Phase C — Wrap-up

- verification.md fail-fast 28+ rows
- sprint38_audit_plan.md
- SPRINT_LOG.md row S37
- CLAUDE.md etat actuel (compteurs, carries, etat)
- HARDENING_ROADMAP.md compteurs finaux + last_validated S37
- Migration `.planning/active/sprint37_*.md` + `sprint36_audit_findings.md`
  → `.planning/archive/v1.2/`
- Commit : `chore(sprint37): Phase C — wrap-up + verification
  + audit plan S38 + migration`

---

## §5 Research consulte

- **tracing-appender 0.2** : deja dans le workspace, utilise par
  daemon (logging.rs) et worker (logging.rs). Pattern connu.
- **blake3 1.5** : deja dans le workspace, utilise par nexus-core-rs
  (canonical.rs, pow.rs, gossip topic). Pattern connu.
- **serde_jcs 0.2** : deja dans le workspace, utilise par
  canonical_bytes(). Pattern connu.
- **icns 0.3** (crates.io) : pure Rust, lit/ecrit ICNS. MIT.
  Maintenu depuis 2017 (7 ans). Derniere release 2024.
  Pas de CVE connue. Utilisee pour la generation .icns
  cross-platform dans un outil de build one-shot.
- **DOMAIN_KUDOS_V1** : deja defini dans canonical.rs L86. Reserve
  pour exactement ce cas d'usage.
