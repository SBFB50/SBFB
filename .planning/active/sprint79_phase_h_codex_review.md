Constat préalable : `git status` montre que T1/T2/fixtures sont présents mais non suivis. Verdict ci-dessous = état local du workspace.

### Livrable 1 : Test Rust byte-exact CSP daemon
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:7563`, `crates/nexus-shell-daemon/src/http.rs:7585`, `crates/nexus-shell-daemon/src/http.rs:7608`
- Evidence :
```rust
#[tokio::test]
async fn blob_serve_csp_header_byte_exact_matches_contract() {
    let state = mk_state().await;
```
```rust
assert_eq!(resp_200.status(), StatusCode::OK);
assert_eq!(
    resp_200.headers().get("content-security-policy")
```
```rust
assert_eq!(resp_404.status(), StatusCode::NOT_FOUND);
assert_eq!(
    resp_404.headers().get("content-security-policy")
```
Les deux assertions comparent bien à `nexus_core_rs::csp::BLOB_SERVE_CSP` aux lignes `7593` et `7616`.

### Livrable 2 : T1 E2E Playwright hermétique
- Statut : CONFIRME
- Fichier(s) : `web/e2e/app-authoring.spec.ts:121`, `web/e2e/app-authoring.spec.ts:133`, `web/e2e/app-authoring.spec.ts:157`
- Evidence :
```ts
test("served CSP header is byte-equal to the single-source contract", async ({
  request,
}) => {
```
```ts
test("CLEAN app replays under the real CSP with zero violation", async ({
  page,
  request,
```
```ts
test("DIRTY app's runtime-assembled fetch is caught by the CSP at runtime", async ({
  page,
  request,
```
Le seed passe bien par `publish-blob` puis `publish` (`web/e2e/app-authoring.spec.ts:74-101`), le header est comparé avec `toBe(CSP_CONTRACT)` (`:130`), clean exige `toHaveLength(0)` (`:151-154`), dirty exige `toBeGreaterThan(0)` (`:174-179`). `npx playwright test app-authoring.spec.ts --list` liste 3 tests.

### Livrable 3 : Fixtures de test
- Statut : CONFIRME
- Fichier(s) : `web/e2e/fixtures/app-authoring/build-fixtures.mjs:24`, `web/e2e/fixtures/app-authoring/src/dirty/index.html:21`, `web/e2e/fixtures/app-authoring/README.md:11`
- Evidence :
```js
import { deflateRawSync } from "node:zlib";
import { readFileSync, writeFileSync } from "node:fs";
```
```html
var target = atob("aHR0cHM6Ly9leGFtcGxlLmNvbS9zYmZiLXBpbmc=");
fetch(target).catch(function () {});
```
```md
| `clean.zip` | positive control | inline DOM mutation only — **zero** CSP violation |
| `dirty.zip` | negative control | `fetch(atob("…"))` to an external host — violates `connect-src 'none'` |
```
`clean.zip` et `dirty.zip` existent et contiennent `index.html`. Le builder asserte aussi la cible dirty externe via base64 (`build-fixtures.mjs:119-125`). `node --check` passe.

### Livrable 4 : T2 harnais acceptance anti-faux-vert
- Statut : CONFIRME
- Fichier(s) : `scripts/acceptance/app_authoring_capability.sh:162`, `scripts/acceptance/app_authoring_capability.sh:169`, `scripts/acceptance/app_authoring_capability.sh:253`
- Evidence :
```bash
: > "$PW_JSON" 2>/dev/null || rig_absent "cannot truncate the report path $PW_JSON ..."
```
```bash
cd "$WEB_DIR" \
  && unset SBFB_E2E_BASE_URL SBFB_E2E_COMPUTE SBFB_E2E_PROJECT_ID SBFB_E2E_MODEL \
```
```bash
[ "$TESTS_TOTAL" -ge 3 ] || block "run" ...
[ "$SKIPPED" -eq 0 ] || block "run" ...
[ "$TESTS_PASSED" -eq "$TESTS_TOTAL" ] || block "run" ...
```
Je ne trouve pas de chemin `pass()/exit 0` sans les 3 contrôles réellement passés : `pass` n’est appelé qu’après le run gate, le mapping titre, et les champs CSP/clean/dirty (`scripts/acceptance/app_authoring_capability.sh:261-271`). `python3` est requis avant parsing (`:199`). `bash -n` passe.

### Livrable 5 : Docs gate-spécifiques
- Statut : CONFIRME
- Fichier(s) : `docs/rust/PATTERNS.md:3966`, `docs/factory/FACTORY_GATES.md:160`
- Evidence :
```md
## §P71 — Sprint 79 Phase H: runtime CSP self-check = browser-level console capture
```
```md
The served CSP is byte-compared to
`nexus_core_rs::csp::BLOB_SERVE_CSP`.
```
```md
**Le filet runtime (S79 Phase H, LIVRE) — complement du lint statique.**
```
Les docs décrivent bien le vrai iframe host, `page.on('console')`, CLEAN/DIRTY, T2 JSON, et le statut non autoritaire (`docs/factory/FACTORY_GATES.md:160-183`). Je n’ai pas vu de nouvelle promesse future dans `crates/` ou `web/src/` liée à cette phase.

### Livrable 6 : gitignore
- Statut : CONFIRME
- Fichier(s) : `.gitignore:147`
- Evidence :
```gitignore
scripts/acceptance/.app_authoring_last_result.json
scripts/acceptance/.app_authoring_pw.json
scripts/acceptance/.app_authoring_pw.log
```

## Resume final
- Total livrables : 6
- Confirmes : 6
- Gaps : 0
- Partiels : 0