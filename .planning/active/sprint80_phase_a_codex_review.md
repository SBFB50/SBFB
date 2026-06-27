### Livrable 1 : `AuthState` avec `session_secret`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/auth.rs:83`, `:89`, `:101`, `:108`, `:423`
- Evidence :
```rust
83: pub struct AuthState {
84:     token: Arc<String>,
85:     session_secret: Arc<String>,
...
94:             session_secret: Arc::new(generate_token()),
```
Accesseurs présents : `session_secret()` lignes 101-103, `token_matches()` lignes 108-109. Test utile : `session_secret_is_distinct_from_token` lignes 423-431.

### Livrable 2 : `auth_required` header puis cookie gated
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/auth.rs:330`, `:337`, `:343`, `:344`, `:346`
- Evidence :
```rust
330:     // First transport: the bearer header
331:     let header_ok = headers
332:         .get(AUTH_HEADER)
334:         .map(|t| constant_time_eq(t.as_bytes(), auth.token.as_bytes()))
337:     if !header_ok {
```
Le chemin cookie est uniquement dans `if !header_ok`, compare `auth.session_secret` lignes 343-345, puis exige `Sec-Fetch-Site: same-origin` lignes 346-351. Le chemin header n’exige donc pas `Sec-Fetch-Site`.

### Livrable 3 : `cookie_value` manuel
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/auth.rs:274`, `:282`, `:283`, `:286`, `:399`
- Evidence :
```rust
282: pub fn cookie_value<'a>(cookie_header: &'a str, name: &str) -> Option<&'a str> {
283:     cookie_header.split(';').find_map(|pair| {
284:         let (k, v) = pair.split_once('=')?;
285:         let v = v.trim();
286:         if k.trim() == name && !v.is_empty() {
```
Tests utiles multi-cookie, absent, valeur vide et nom préfixe : lignes 399-420. Aucune dépendance cookie ajoutée dans `crates/sbfb-factory/Cargo.toml:11-39`.

### Livrable 4 : bootstrap public `GET /`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:297`, `:298`, `:308`, `:313`, `:318`, `:332`
- Evidence :
```rust
308:     if let Some(token) = req.uri().query().and_then(|q| query_param(q, "token")) {
309:         if boot.auth.token_matches(token) {
313:             let cookie = format!(
314:                 "{}={}; HttpOnly; SameSite=Strict; Path=/",
```
Host loopback refait lignes 298-305. Cookie = `session_secret()` ligne 316, pas de `Secure`/`Max-Age`, host-only. Redirect 303 + `Location: /` + `Referrer-Policy: no-referrer` lignes 318-327. Token absent/invalide tombe sur `serve_bootstrap_index` lignes 329-332, neutre.

### Livrable 5 : routeur scindé + `ServeDir` dédié
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:47`, `:152`, `:163`, `:199`, `:203`, `:217`
- Evidence :
```rust
152:     let bootstrap = Router::new()
153:         .route("/", get(handle_bootstrap))
...
163:     let serve_assets =
164:         ServeDir::new(bundle.clone()).fallback(ServeFile::new(bundle.join("index.html")));
```
`authed` contient `/api/*` + `fallback_service(serve_assets)` lignes 170-199, puis `auth_required` lignes 203-206. Le routeur merge bootstrap public puis authed lignes 217-219. `ServeDir` reçoit `bundle`, jamais `repo_root_pub()` ; `bundle = root.join(OPERATOR_BUNDLE_SUBDIR)` lignes 225-229.

### Livrable 6 : CSP Operator + `nosniff`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:217`, `:220`, `:343`, `:347`, `:351`
- Evidence :
```rust
343: async fn operator_csp_middleware(req: Request, next: Next) -> Response {
344:     let mut response = next.run(req).await;
347:         header::CONTENT_SECURITY_POLICY,
348:         HeaderValue::from_static("default-src 'self'; connect-src 'self'"),
351:         header::X_CONTENT_TYPE_OPTIONS,
```
Pas de `unsafe-inline` / `unsafe-eval` dans la valeur CSP. Middleware appliqué après merge bootstrap/authed lignes 217-220, donc couvre les réponses de ces routes, dont 401/403/404.

### Livrable 7 : threat model amendé
- Statut : CONFIRME
- Fichier(s) : `docs/security/THREAT_MODEL.md:815`, `:822`, `:827`, `:835`, `:842`, `:852`
- Evidence :
```md
815: **Amendement Sprint 80 Phase A (cookie de transport + garde cross-port).**
822: cookie change le modele de menace et **invalide l'affirmation S71 « un
823: navigateur tiers ne connait pas le token »**
```
Gardes P1 documentées : `Sec-Fetch-Site` sur chemin cookie lignes 827-834, `session_secret` distinct lignes 835-840. Résidu token URL/history/Referer documenté ligne 852 ; cross-port documenté lignes 827-834.

### Livrable 8 : tests unitaires + intégration
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/auth.rs:399`, `:423`; `crates/sbfb-factory/tests/operator_server.rs:235`, `:270`, `:287`, `:312`, `:330`, `:347`, `:361`, `:379`
- Evidence :
```rust
235: fn bootstrap_valid_token_sets_cookie_and_303() {
260:     // P1-B: the cookie value must NOT be the bearer token.
262:     assert_ne!(
263:         cookie, TEST_TOKEN,
```
Tous les cas demandés ont des assertions utiles : cookie valide, invalide sans cookie, cookie+`Sec-Fetch-Site`, cookie sans `Sec-Fetch-Site`, mauvais cookie, header gagnant, CSP présent, Host non-loopback 403.

### Invariants transverses
- Cookie comparé à `session_secret`, jamais au bearer : confirmé `auth.rs:343-345`.
- `Sec-Fetch-Site` exigé seulement sur chemin cookie : confirmé `auth.rs:337-351`.
- `ServeDir` jamais rooté sur `repo_root_pub()` : confirmé `operator_server.rs:163-164`, `:225-229`.
- Bootstrap hors `auth_required` : confirmé `operator_server.rs:152-157`, `:203-206`, `:217-219`.
- Aucun `use nexus_shell_daemon*` réel trouvé dans `sbfb-factory`; occurrences restantes seulement en commentaires. Aucun `Cargo.toml` modifié dans le diff.

Tests lancés :
- `cargo test -p sbfb-factory auth::tests::` : OK, 7 passed.
- `cargo test -p sbfb-factory --test operator_server` : OK, 51 passed.
- Note : `--lib` n’est pas applicable, la crate n’a pas de target library.

## Resume final
- Total livrables : 8
- Confirmes : 8
- Gaps : 0
- Partiels : 0