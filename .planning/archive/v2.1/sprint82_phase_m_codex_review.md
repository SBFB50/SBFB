Verdict global : les 6 livrables sont confirmés sur le working tree non committé de `master`, à `HEAD 013b611c704e`. Les fichiers `.planning/research/*` ont été exclus comme demandé.

### Livrable 1 : Golden de caractérisation HTTP

- Statut : CONFIRME
- Fichier : [http.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:12783)
- Evidence : neuf tests `golden_http_*` aux lignes 12783, 12826, 12859, 12901, 12934, 12976, 13013, 13059 et 13100. Ils couvrent les six domaines, CSP sur erreur, CORS RAW et fallback SPA.

```rust
name: "kudos_verify_chain",
method: Method::GET,
uri: "/api/v1/kudos/proj-golden/verify",
json_body: None,
want_status: 200,
```

Le test publish couvre bien `publish`, `publish-blob` et `directory/publish` aux lignes 13017-13051. `publish_blob` est réellement le handler éloigné défini ligne 3261. CSP est vérifié sur l’erreur `blob_serve_bad_hash` lignes 12800-12819 ; CORS et SPA vérifient statut, en-têtes et corps lignes 13060-13128.

Les neuf tests ont passé deux fois consécutivement : `9/9`, puis `9/9`.

### Livrable 2 : Helpers golden réels

- Statut : CONFIRME
- Fichier : [http.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:12677)
- Evidence : allowlist et placeholder lignes 12677-12679 ; récursion objets et tableaux lignes 12684-12702 ; `GoldenBody`, `GoldenCase`, `golden_check` et `golden_run` lignes 12704-12778.

```rust
golden_redact(&mut got);
let mut want = want.clone();
golden_redact(&mut want);
assert_eq!(got, want, "[golden:{}] body drifted", case.name);
```

`golden_check` vérifie réellement le statut lignes 12740-12744, chaque en-tête littéral lignes 12745-12753, le JSON ou le texte exact lignes 12754-12767. Le champ non volatil `"catalog_len": 0` reste explicitement asserté ligne 13049 : aucune tautologie.

### Livrable 3 : Déduplication du harness

- Statut : CONFIRME
- Fichiers : [http.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:4619), [test feed-insert](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:7972)
- Evidence : `TestHeaders::{Inject, Raw}` lignes 4619-4627 et constructeur unique `build_test_router_ext` lignes 4641-4670.

```rust
if !h.contains_key(AUTH_HEADER_NAME) {
    h.insert(AUTH_HEADER_NAME, HeaderValue::from_static(TEST_TOKEN));
}
if !h.contains_key(HOST) {
    h.insert(HOST, HeaderValue::from_static("127.0.0.1:0"));
}
h.remove(ORIGIN);
```

À `HEAD`, ces opérations existaient dans deux couches ; le working tree n’en contient plus qu’une. Aucune insertion de `x-sbfb-feed-internal` n’existe dans le harness. Le test négatif `feed_insert_rejects_without_internal_header` passe avec le reste de la cible daemon.

### Livrable 4 : Wrappers de posture préservés

- Statut : CONFIRME
- Fichier : [http.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:4677), lignes 4677-4679, 7691-7695 et 8595-8610.
- Evidence :

```rust
fn build_test_router(state: Arc<DaemonHttpState>) -> Router {
    build_test_router_ext(state, &[], None, TestHeaders::Inject)
}
```

`build_cors_test_router` conserve sa signature et utilise `TestHeaders::Raw`. Ses cinq appels préexistants sont intacts ; le sixième est le nouveau golden CORS. `build_test_router_with_web_root` retourne toujours `(Router, tempfile::TempDir)` et transmet `Some(tmp.path())` en posture Inject.

Les 178 appels préexistants à `build_test_router` sont intacts. `build_test_router_with_cors` avait deux références à `HEAD` — définition et unique appel — et n’en a désormais aucune.

### Livrable 5 : Invariant production byte-identique

- Statut : CONFIRME
- Fichier : [http.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:246), module de tests ligne 4528.
- Evidence :

```rust
pub fn build_router(
    state: Arc<DaemonHttpState>,
    auth: AuthState,
    cors_origins: &[String],
    web_root: Option<&FsPath>,
```

Le premier hunk commence ligne 4619, après `#[cfg(test)] mod tests` ligne 4528. Le SHA-256 du segment normalisé `build_router`, lignes 246-543, est identique à `HEAD` et au working tree :

`922f73ebfc6967864aaf2e36ee69eca2d467de984b42b682e084678d4317c628`

Contrôles supplémentaires :

- Delta `.route(...)` : `0`.
- Delta DTO `Serialize`/`Deserialize`/`#[serde(...)]` : `0`.
- Delta constantes `*_VERSION` : `0`.
- Fichiers modifiés sous `crates/nexus-core-rs/src` : `0`.
- `Cargo.toml`/`Cargo.lock` modifiés : `0`.

### Livrable 6 : Oracle de count exact

- Statut : CONFIRME
- Fichiers : [http.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:12783), [auth.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-launcher/src/auth.rs:304)
- Evidence :

```text
#[tokio::test] dans http.rs : 177 → 186
Ajouts dans le diff          : +9
Suppressions                 :  0
#[test] synchrones           : 22 → 22
#[cfg(unix)] dans http.rs    :  0 → 0
```

Les dix hunks du diff concernent uniquement les trois zones de constructeurs du harness et l’ajout final des golden. Les 37 lignes retirées sont exclusivement du code/commentaire de harness ; aucun corps de test existant n’est modifié.

Oracles mesurés :

- Windows : `cargo nextest list` → `2108`.
- Linux hors launcher : `2079`.
- Launcher : `32` tests Windows + exactement un test Unix ligne 304 → `33`.
- Linux total : `2079 + 33 = 2112`.

Le delta étant exactement neuf tests inconditionnels, cela confirme `2099 → 2108` sous Windows et `2103 → 2112` sous Linux.

Vérifications exécutées : `cargo fmt --all --check` PASS ; golden deux fois `9/9` ; cible daemon complète `453/453`.

## Résumé final

- Total livrables : 6
- Confirmés : 6
- Gaps : 0
- Partiels : 0

Le diff source de phase reste non committé dans `http.rs`. Aucun fichier du working tree n’a été modifié par l’audit.

