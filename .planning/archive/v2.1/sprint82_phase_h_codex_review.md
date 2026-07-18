Verdict global : l’implémentation F-D5-01 est correcte, mais la clôture documentaire est partielle. Audit statique uniquement ; aucune suite de tests lancée.

### Livrable 1 : constante partagée du suffixe

- Statut : CONFIRME
- Fichier(s) : [docs.rs:50](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/docs.rs:50), [lib.rs:124](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/lib.rs:124)
- Evidence :

```rust
/// migration test). The upstream-drift tripwire lives in
/// `tests/store_migration.rs`: it runs the real migration [...]
pub const MIGRATION_BACKUP_SUFFIX: &str = ".backup-redb-v2-tuples";

pub use docs::{DocHandle, DocsClient, MIGRATION_BACKUP_SUFFIX};
```

Le commentaire documente bien `migrate_redb_v2_tuples.rs`, le littéral upstream non exporté et le tripwire. La constante n’est ni dans `canonical.rs`, ni dans `schemas/`, et son nom ne finit pas par `_VERSION`.

### Livrable 2 : dérivation du chemin daemon

- Statut : CONFIRME
- Fichier(s) : [runtime.rs:2741](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:2741), [runtime.rs:2773](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:2773), [runtime.rs:4818](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:4818)
- Evidence :

```rust
pub(crate) fn docs_migration_backup_path(iroh_data_dir: &Path) -> PathBuf {
    let mut p = iroh_data_dir.join("docs.redb").into_os_string();
    p.push(nexus_core_rs::MIGRATION_BACKUP_SUFFIX);
    p.into()
}
```

Le diff remplace exactement l’ancien `join("docs.redb.backup-redb-v2-tuples")`. `OsString::push` concatène sans séparateur : le résultat reste `iroh_data_dir/docs.redb.backup-redb-v2-tuples`. Le guard et ses deux tests existants ne comportent aucun autre changement.

### Livrable 3 : tripwire de migration upstream

- Statut : CONFIRME
- Fichier(s) : [store_migration.rs:34](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/tests/store_migration.rs:34), [store_migration.rs:107](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/tests/store_migration.rs:107), [store_migration.rs:265](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/tests/store_migration.rs:265)
- Evidence :

```rust
fn upstream_migration_backup_suffix_matches_shared_const() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("docs.redb");
    forge_legacy_store(&path);
    let store = DocsStore::persistent(&path).expect("... migrates ...");
```

```rust
assert_eq!(
    entries,
    vec!["docs.redb".to_owned(),
         format!("docs.redb{MIGRATION_BACKUP_SUFFIX}")],
);
```

La migration exécutée est bien celle d’iroh-docs : `DocsStore::persistent` appelle upstream `migrate_redb_v2_tuples::run` sur le `TableTypeMismatch`. L’assertion est utile et exacte. Le comptage statique passe de 3 à 4 annotations de test. Les constantes `FX_*` conservent les anciennes valeurs `[1]`, `[2]`, `kv:phase-f`, `[3]`, `[4]`, `[5]`, `[9]`, `1`, `42`, `7`, et les assertions byte-exactes demeurent.

### Livrable 4 : re-ancrage T20

- Statut : CONFIRME
- Fichier(s) : [PATTERNS.md:984](/C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:984), [PATTERNS.md:1040](/C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:1040), [node.rs:324](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/node.rs:324), [pkarr_resolver.rs:41](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/pkarr_resolver.rs:41)
- Evidence :

```markdown
the single `Endpoint::builder` chokepoint, whose test-only path
already demonstrates `.ca_tls_config(...)`; `iroh_runtime.rs` no
longer builds the endpoint. The status-update blockquote above is
the authoritative current pointer. Doc re-anchor only — the T20
security carry itself stays OPEN [...]
```

`node.rs:349/354` porte les builders, `node.rs:510` le relay mode et `node.rs:858` la démonstration test-only. `iroh_runtime.rs` ne contient aucun builder/TLS/relay correspondant. Le namespace `iroh::tls::CaTlsConfig` est celui réellement importé. Aucun fichier de câblage TLS n’est modifié.

### Livrable 5 : entrée H-3 wheel drift

- Statut : PARTIEL
- Fichier(s) : [PATTERNS.md:849](/C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:849), [verify.sh:3](/C:/Users/FlowUP/Documents/Code/nexus/scripts/verify.sh:3), [sprint82_plan.md:223](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_plan.md:223)
- Evidence :

```markdown
`setup.sh` and `.githooks/post-merge`
survived as Python-era zombies until S82 Phase E purged them
(`f727f8c`); `scripts/verify.sh` lives on, rebuilt 0-Python in
the same phase.
```

Les suppressions sont réelles dans `f727f8c`; `scripts/verify.sh` existe et son en-tête confirme la suppression des étapes Python.

- Écart restant : `PATTERNS.md:852-853` affirme que S81-H-3 est « handled in S82 Phase I ». Aucune Phase I S82 n’existe encore ; `sprint82_plan.md:223-225` ne fait que la router. La formulation exigée était « routed to S82 Phase I », au présent-vrai.

### Livrable 6 : passe de fidélité des 21 corrections

- Statut : PARTIEL
- Fichier(s) : [rust/PATTERNS.md:2114](/C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:2114), [shell/PATTERNS.md:13](/C:/Users/FlowUP/Documents/Code/nexus/docs/shell/PATTERNS.md:13)

Contrôles confirmés :

- P35 : `max_tasks`, `vram_wipe`, `LifecycleState` et `EphemeralLifecycle` concordent avec `ephemeral.rs:22-121`.
- P37 : le port Rust existe dans `watermark_detector.rs:1-38`.
- P40/P41 : Python est bien historique ; les seuils Rust sont `30/45` dans `canary_registry.rs:15-16`.
- P68 : `sampling_key`/`select_candidates` utilisent BLAKE3 dans `placement.rs:309-350`, tandis que `seeders_recent` appelle toujours `ids.sort()` dans `seed_registry.rs:315-332`.
- P69 : le daemon passe `PerfMap::new()` à `assign_fallback_nodes` dans `shard_session.rs:652-653`; `PERF_MAP_REPUBLISH_INTERVAL` n’a aucun consommateur hors `routing.rs`.
- P9, cœur demandé : `daemon.ts:231-298` construit bien `DaemonResult<T>` côté client et appelle directement `authFetch`.
- P21 : `Cargo.lock:11527-11528` résout bien `zip 8.6.0`; contrainte `8.5` à `Cargo.toml:204`.
- P39 : le lookup lit `ShardSessionRegistry` dans `http.rs:2158-2168`, alimenté par `shard_session.rs:457-465`.
- FROST : les deux mentions disent v3.x et `Cargo.lock:2535-2536` contient `3.0.0`.
- Les ancres symboliques `FeedEntry`, `try_parse_op`, `op_type`, `BlobStore` et `test_feed_republish_at_boot` existent.

Écarts factuels :

1. P36 reste contradictoire et décrit mal le quorum :

```rust
if best_count > majority_threshold {
    for r in &results {
        if r.sha256 != best_hash {
            tracing::warn!("quorum outlier detected");
        }
    }
```

Le chemin retourne ensuite `ValidationOutcome::Accepted` (`validator.rs:325-329`). Pourtant `PATTERNS.md:2160-2162` dit que des valeurs divergentes sont journalisées puis que la tâche est rejetée. De plus, `PATTERNS.md:2152` parle encore d’un hash des bytes canoniques, en contradiction avec `result_text` exact à `:2166`. Enfin, l’attribut réel est `#[serde(default = "default_redundancy_factor")]` (`task.rs:279`), pas `#[serde(default)]` comme écrit à `PATTERNS.md:2171`.

2. L’affirmation « No plain-`fetch` exception remains » est fausse :

```tsx
const resp = await fetch(
  `http://127.0.0.1:18765/app/${appName}/files/upload`,
  { method: "POST", body: form },
);
const data = (await resp.json()) as { /* ... */ };
```

Ce code existe à `FileUploadBlock.tsx:53-68`, en contradiction avec `shell/PATTERNS.md:13-16` et `:26-31`.

3. `shell/PATTERNS.md:190` dit « safeParse on every response », mais `daemon.ts:258-275` retourne les non-2xx avant le `safeParse`, exécuté seulement à `:290`.

4. P43 dit que `pow_keypair` sert le kudos ledger (`rust/PATTERNS.md:2352`). En réalité, `kudos_ledger::credit` (`kudos_ledger.rs:76-112`) ne reçoit aucune clé et calcule un BLAKE3 non signé. La clé sert bien à signer les tâches (`http.rs:3741-3742`), pas le ledger.

### Livrable 7 : note Track C

- Statut : PARTIEL
- Fichier(s) : [sprint81_audit_findings.md:103](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint81_audit_findings.md:103)
- Evidence :

```markdown
La prose de S81-C-1/C-2/C-4/C-5 n'a jamais ete persistee [...]
Disposition Phase H : passe de fidelite PATTERNS<->code RE-DERIVEE [...]
S81-C-4/C-5 subsumes par la meme passe [...]
S81-C-3 re-ancre [...]
```

La note datée contient bien les quatre éléments demandés.

- Écart restant : `:111-113` affirme « TOUS corrigés dans le commit Phase H » et « SOLDÉS ». La phase est toujours non commitée, et les écarts du livrable 6 montrent que la disposition n’est pas entièrement soldée.

### Livrable 8 : preflight PLAN-ADAPT et tally

- Statut : PARTIEL
- Fichier(s) : [sprint82_phase_h_preflight.md:15](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_phase_h_preflight.md:15), [sprint82_phase_h_preflight.md:176](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_phase_h_preflight.md:176)
- Evidence :

```markdown
Résultat : **21 findings vérifiés — 14 P2 + 7 P3 [...]
(4 UPGRADE-P2 : frost, sybil-tail, perf-map seam, zip [...]
Les 21 findings, TOUS corrigés in-phase (doc-only) :
```

Le verdict `PLAN-ADAPT`, la section et le tally sont présents. Le décompte est arithmétiquement cohérent : 14 P2, 7 P3, dont quatre promotions P2.

- Écart restant : la phrase « TOUS corrigés » à `:190` est contredite par les erreurs documentaires constatées au livrable 6.

### Livrable 9 : invariants transverses

- Statut : PARTIEL
- Fichier(s) : [shell/PATTERNS.md:1192](/C:/Users/FlowUP/Documents/Code/nexus/docs/shell/PATTERNS.md:1192), [rust/PATTERNS.md:850](/C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:850)

Invariants confirmés :

- Aucun delta dans `Cargo.toml`, `Cargo.lock`, `canonical.rs` ou `schemas/`.
- Les blobs de `Cargo.toml` et `Cargo.lock` sont byte-identiques à `HEAD`.
- Census statique : exactement 25 familles `const DOMAIN_*_Vn`.
- Aucun nouveau `*_VERSION`.
- `git diff --check` est propre.

Écarts :

```markdown
decompresse le zip en memoire (crate `zip`, 8.6.0 au lock,
contrainte declaree 8.5 — 2.6 a l'origine S12), cache les
fichiers dans un `BlobServeCache` LRU [...]
```

Cette prose ajoutée à `shell/PATTERNS.md:1194-1195` est en français, contrairement à l’invariant « prose PATTERNS en anglais ».

La formulation « handled in S82 Phase I » (`rust/PATTERNS.md:852-853`) et « TOUS corrigés dans le commit Phase H » (`sprint81_audit_findings.md:111`) déclarent aussi des états futurs/non réalisés au lieu d’un état présent-vrai.

## Résumé final

- Total livrables : 9
- Confirmés : 4
- Gaps : 0
- Partiels : 5

Note hors décompte : l’artefact supplémentaire non suivi `sprint82_phase_h_review.md:11` porte encore `## Verdict: PASS-PENDING`, et précise à `:16` que la réconciliation Codex n’est pas encore faite.