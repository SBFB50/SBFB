Audit fait sur `git status`, `git diff`, `git diff --cached`, fichiers untracked, lecture complète des fichiers touchés, plus grounding Rust pour les wires. Vérifications exécutées : `npm run test:unit` = 137/137, `npm run build` OK, `npm run size` OK, `npm run gate:scan-front` clean, `npx playwright test e2e/verify.spec.ts` = 2/2.

### Livrable 1 : DiffViewer bespoke
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/components/verify/plein/DiffViewer.tsx:57`, `:197`, `:223`, `:290`, `:347`, `:386`
- Evidence :
```tsx
223: function onKeyDown(e: React.KeyboardEvent<HTMLDivElement>) {
224:   if (e.key === 'ArrowDown' || e.key === 'j') {
225:     e.preventDefault()
226:     moveHunk(1)
```
Bi-mode, word-diff apparié, navigation clavier, `aria-current`, minimap cliquable, change-set repliable et hunk-intent callback sont présents. Aucun re-diff fichier JS.

### Livrable 2 : wordDiff
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/components/verify/plein/wordDiff.ts:24`, `:47`, `:51`, `:60`, `:85`
- Evidence :
```ts
51: if (a.length > MAX_TOKENS || b.length > MAX_TOKENS) {
52:   return {
53:     old: oldText ? [{ text: oldText, changed: true }] : [],
54:     new: newText ? [{ text: newText, changed: true }] : [],
```
LCS token maison, borne `MAX_TOKENS`, et reconstruction exacte couverte par `wordDiff.test.ts:13-15`.

### Livrable 3 : GatesPanel
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/components/verify/GatesPanel.tsx:25`, `:66`, `:77`, `:82`, `:103`
- Evidence :
```tsx
66: ) : gates && gates.length > 0 ? (
67:   <span className="flex flex-wrap items-center gap-x-3 gap-y-0.5">
68:     {gates.map((entry) => (
69:       <GateGlyph key={entryKey(entry)} entry={entry} />
```
Restitution 1:1, key `(gate,status)`, pas d’agrégat racine, `run@rev`, bouton `relancer`, tray de détails.

### Livrable 4 : VerifyScene réécrit
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/components/verify/VerifyScene.tsx:90`, `:100`, `:123`, `:126`, `:140`, `:163`
- Evidence :
```tsx
126: <Tab label="Diff" active />
127: <Tab label="Aperçu scellé" disabled />
128: <Tab label="Preuve" disabled />
```
Diff actif, onglets scellé/preuve disabled sans fetch, bande gates+état permanente, terminal en outil secondaire, hunk intent via `op.launch`.

### Livrable 5 : useVerifyData
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/state/useVerifyData.ts:40`, `:55`, `:59`, `:61`, `:76`
- Evidence :
```ts
55: const [diffRes, gatesRes] = await Promise.allSettled([
56:   getWorkingTreeDiff(signal),
57:   getGates(signal),
58: ])
59: if (signal.aborted) return
```
Dégradation indépendante, erreurs séparées, abort non traité comme erreur, `reload` refetch les deux, `loading` dérivé.

### Livrable 6 : verdict.ts
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/lib/verdict.ts:32`, `:45`, `:59`, `:69`, `:100`, `:122`
- Evidence :
```ts
59: export const GATE_STATUS = {
60:   notRun: 'not_run',
61:   notApplicable: 'not_applicable',
62:   passed: 'passed',
63:   informational: 'informational',
```
Miroir unique des 5 `GateStatus`, glyph/tone/label sans mots interdits, `VERIFY_ETAT` ne contient pas `PASS`, `pickVerifyEtat` lit seulement loading/hasChanges.

### Livrable 7 : operator.ts wire API
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/api/operator.ts:357`, `:368`, `:375`, `:385`, `:394`, `:399`; Rust `crates/sbfb-factory/src/sprint_history.rs:1004`, `crates/sbfb-factory/src/gates.rs:75`
- Evidence :
```ts
357: export interface WorkingTreeDiff {
358:   head: string
359:   unstaged: FileDiff[]
360:   staged: FileDiff[]
361:   truncated: boolean
```
Le miroir TS correspond aux structs Rust : `WorkingTreeDiff`, `GateStatus`, `GateIssueView`, `GateEntryView`, `GatesView`. Fetch via `getJson<T>` avec `credentials: same-origin`.

### Livrable 8 : Bascule bi-focal
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/state/useOperator.ts:73`, `:196`; `tools/factory-operator/src/components/Rail.tsx:42`, `:83`; `tools/factory-operator/src/App.tsx:50`
- Evidence :
```ts
196: verifyReady: message !== null && (stream.status === 'done' || stream.status === 'ended'),
```
`verifyReady` allume seulement l’indice `ready && !active`; aucun auto-switch `setMode` déclenché par le stream.

### Livrable 9 : Migration DiffView -> DiffViewer
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/components/surfaces/ProcedeSurface.tsx:18`, `:155`; suppressions staged `DiffView.tsx`, `DiffView.test.tsx`
- Evidence :
```tsx
155: <DiffViewer
156:   files={diff.files}
157:   caption={`commit ${diff.sha.slice(0, 10)} — ${diff.title}`}
158:   emptyLabel="aucun fichier dans ce diff"
159:   testid="diff-view"
```
Ancien `DiffView` supprimé. Aucun import mort trouvé. Build TypeScript/Vite OK.

### Livrable 10 : Budget / manualChunk
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/vite.config.ts:68`, `:77`; `tools/factory-operator/.size-limit.json:17`, `:24`, `:31`
- Evidence :
```ts
68: manualChunks(id) {
69:   const nid = id.replace(/\\/g, '/')
77:   if (nid.includes('/src/components/verify/plein/')) return 'diff-viewer'
```
`diff-viewer` extrait. `size-limit` passe : `verify-surface 94.98/96 KB`, `diff-viewer 20.29/22 KB`, `css 24.45/25 KB`.

### Livrable 11 : Tests Vitest + E2E
- Statut : PARTIEL
- Fichier(s) : tests présents aux chemins demandés ; `tools/factory-operator/e2e/verify.spec.ts:17`, `:45`
- Evidence :
```ts
17: test('VERIFY-plein shows the bespoke diff-viewer + the live gates band; ÉTAT never says a verdict', async ({
45: test('the Procédé inspector restitutes ≥1 phase verdict (never a score) and the diff bi-usage renders a past commit', async ({
```
Vitest : CONFIRME, `137 passed`. Tests non tautologiques, pas de `expect(true)`, pas de `.only/.skip`. Gap : l’acceptance annonce `verify.spec.ts` en `7/7`, mais Playwright liste et exécute seulement 2 tests (`2 passed`).

## Transverse
- XSS : PARTIEL strict. 0 `dangerouslySetInnerHTML`/`innerHTML`/`eval`; diff hostile couvert par `DiffViewer.test.tsx:111-129`; contenu rendu en noeuds React texte. Mais la règle “un seul style inline = flexGrow minimap” est fausse globalement : `GateFlip.tsx:41` a aussi un `style={{ display, transformOrigin, transformStyle }}` statique, en plus de `DiffViewer.tsx:400-401`.
- Cardinal 0 verdict UI : CONFIRME. Pas d’`overall/all_passed/score`; `GatesPanel` restitue chaque entrée, `VERIFY_ETAT` est un état observable.
- Anti-PASS : CONFIRME. `npm run gate:scan-front` clean. Le seul code non-commentaire sensible est `verdict.ts:122` `verdict === 'PASS'`, comparateur autorisé.
- 0 dépendance runtime : CONFIRME. `package.json` hors diff; 0 import `zod/jsdiff/@tanstack`.
- Fraîcheur honnête : CONFIRME. Pas de badge runtime `obsolète`; `run@{diff.head}` via `VerifyScene.tsx:167` et `GatesPanel.tsx:77-80`.
- Bascule manuelle : CONFIRME. `verifyReady` n’appelle jamais `setMode`.
- Doc-comments : PARTIEL. `App.tsx:4` contient encore `Phase D adds`, formulation explicitement interdite si la règle s’applique à tous les commentaires de tête.
- DiffView migration : CONFIRME. `rg` ne trouve plus d’import `DiffView`; build OK.

## Résumé final
- Total livrables : 11
- Confirmés : 10
- Gaps : 0
- Partiels : 1

Findings :
- P2 `tools/factory-operator/e2e/verify.spec.ts:17` et `:45` : acceptance “7/7” non tenue, seulement 2 tests Playwright.
- P3 `tools/factory-operator/src/components/motion/GateFlip.tsx:41` : invariant “seul style inline = flexGrow minimap” non strictement vrai.
- P3 `tools/factory-operator/src/App.tsx:4` : commentaire `Phase D adds` contre la règle de commentaires non promissoires/formulations interdites.