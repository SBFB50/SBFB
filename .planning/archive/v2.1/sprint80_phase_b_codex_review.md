### Livrable 1 : Jettison réel
- Statut : PARTIEL
- Fichier(s) : `tools/factory-operator/src/App.tsx:10`, `docs/agent/RRV_FACTORY_CONTRACT.md:109`, `docs/agent/RRV_FACTORY_CONTRACT.md:142`, `CLAUDE.md:495`
- Evidence :
```tsx
export function App() {
  return (
    <div className="min-h-screen bg-s0 text-tx font-sans">
```
```md
- **Lecture seule** : socle partage `tools/factory-ui/src/readonly`
...
- **Socle partage en lecture** : reutilise `tools/factory-ui/src/readonly`
```
- Vérifié : `tools/factory-ui` n’existe plus, l’ancien `src/pages`, `src/components/ui`, `i18n`, `hooks`, `executionChat` est supprimé du staging.
- Manque : P1, des docs contractuelles mentionnent encore le chemin supprimé `tools/factory-ui`.

### Livrable 2 : Stack scaffold
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/package.json:21`, `tools/factory-operator/vite.config.ts:48`
- Evidence :
```json
"@base-ui/react": "1.6.0",
"@fontsource-variable/geist": "^5.2.9",
"@fontsource-variable/geist-mono": "^5.2.8",
"motion": "^12.42.0",
```
```ts
plugins: [
  react(),
  babel({ presets: [reactCompilerPreset()] }),
  tailwindcss(),
],
```
- Vérifié : pas de Radix/shadcn/CVA/i18next/react-router dans `package.json` ou `package-lock.json`; `outDir: 'bundle'`, `modulePreload.polyfill:false`, `assetsInlineLimit:0`.

### Livrable 3 : Tokens oklch Tailwind v4
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/index.css:26`, `tools/factory-operator/src/App.tsx:12`
- Evidence :
```css
@theme {
  --color-s0: oklch(0.165 0.004 260);
  --color-s1: oklch(0.205 0.005 260);
  --color-bd: oklch(0.330 0.006 260);
```
```tsx
<div className="min-h-screen bg-s0 text-tx font-sans">
<header className="flex items-center gap-3 border-b border-bd bg-s1 px-4 py-2">
```
- Vérifié : `npm run build` passe. Le CSS bâti `bundle/assets/index-Bsd4bNvB.css` contient `.bg-s0`, `.text-tx`, `.border-bd`, `oklch(...)`, et pas de fuite `@theme/@utility`.

### Livrable 4 : 5 gates de discipline
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/scripts/check-no-radix-runtime.sh:21`, `tools/factory-operator/eslint.config.js:39`, `.github/workflows/ci.yml:169`, `.woodpecker/ci-linux.yml:103`
- Evidence :
```sh
if ! node -e 'const d=require("./package.json").dependencies||{};const r=Object.keys(d).filter(k=>k.startsWith("@radix-ui/"));...
if grep -rnE "['\"]@radix-ui/" src 2>/dev/null; then
```
```js
'no-restricted-imports': [
  'error',
  { paths: [{ name: 'motion/react', importNames: ['motion'] }],
```
- Vérifié : `npm run gates`, `npm run lint`, `npm run size` passent. Probes négatives : `@radix-ui`, `<motion.div>`, `tailwind.config.js`, texte `PASS` échouent réellement. CI GHA et Woodpecker câblent lint/gates/build/size.

### Livrable 5 : Squelette T1 hermétique
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/playwright.config.ts:26`, `tools/factory-operator/e2e/serve-operator.mjs:23`, `tools/factory-operator/e2e/boot.spec.ts:27`
- Evidence :
```ts
webServer: {
  command: 'node e2e/serve-operator.mjs',
  url: `http://127.0.0.1:${OPERATOR_TEST_PORT}/`,
```
```ts
expect(resp?.headers()['content-security-policy']).toContain("default-src 'self'")
await expect(page.getByTestId('operator-rail')).toBeVisible()
expect(violations, 'CSP violations on the built bundle').toEqual([])
```
- Vérifié : `npx playwright test --list` découvre 2 tests. `npm run test:e2e` avec `SBFB_FACTORY_BIN=target/debug/sbfb-factory.exe` passe : 2/2, vrai Operator Rust, bundle rebuild, cookie HttpOnly/303/reload/CSP assertés.

### Livrable 6 : Invariants
- Statut : PARTIEL
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:47`, `tools/factory-operator/package-lock.json:1298`, `tools/factory-operator/package-lock.json:3407`
- Evidence :
```rust
const OPERATOR_BUNDLE_SUBDIR: &str = "tools/factory-operator/bundle";
```
```json
"node_modules/@tailwindcss/node": {
  "dependencies": {
    "lightningcss": "1.32.0",
```
```json
"node_modules/lightningcss": {
  "license": "MPL-2.0",
```
- Vérifié : aucun `.rs` dans le diff stage, aucun fichier wire/canonical/route backend stage. Geist est vendored en `.woff2`, pas de CDN fonts. 
- Manque : P1 si la règle est “deps permissives only” stricte : `lightningcss` est `MPL-2.0` dans l’arbre npm de production via Tailwind.

## Résumé final
- Total livrables : 6
- Confirmés : 4
- Gaps : 0
- Partiels : 2
- P0 : aucun
- P1 : docs contractuelles mentionnent encore `tools/factory-ui`; invariant licence permissive-only non confirmé à cause de `lightningcss MPL-2.0`.

