Audit basé sur le working tree courant (`git status`, `git diff`, fichiers non suivis inclus).

### Livrable 1 : Backend D1
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:520`
- Evidence :
```rust
520:/// pack). Sprint 80 Phase D (fold D1): the daisyui pack
524:const AUTHORING_KNOWLEDGE_MANIFESTS: &[&str] = &[
525:    "docs/factory/knowledge/animejs/MANIFEST.json",
526:    "docs/factory/knowledge/daisyui/MANIFEST.json",
527:];
```
- Diff backend limité à ce commentaire + constante. Pas de nouveau champ struct ni bump wire constaté.

### Livrable 2 : Backend D1 tests
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/tests/operator_server.rs:952`, `:989`
- Evidence :
```rust
952:    let daisyui = ak
957:                .is_some_and(|p| p.ends_with("daisyui/MANIFEST.json"))
959:        .expect("authoring_knowledge should reference the daisyui MANIFEST (S80 Phase D)");
960:    assert_eq!(daisyui["exists"], true, "daisyui MANIFEST should exist");
```
```rust
989:    let daisyui = ak
994:                .is_some_and(|p| p.ends_with("daisyui/MANIFEST.json"))
1000:        daisyui["exists"], true,
1001:        "chat session daisyui MANIFEST should exist"
```

### Livrable 3 : Front API
- Statut : PARTIEL
- Fichier(s) : `tools/factory-operator/src/api/operator.ts:290`, `crates/sbfb-factory/src/operator_server.rs:171`, `crates/sbfb-factory/src/sprint_history.rs:10`
- Evidence :
```ts
290:/** `GET /api/sprint-history` */
296:export function getCommitDiff(...)
300:export function getActionLog(...)
321:export function getTerminalSessions(...)
330:export function getChatLog(...)
338:export function terminalWsUrl(...)
```
```rust
171:        .route("/api/status", get(handle_status))
172:        .route("/api/lint", get(handle_lint))
173:        .route("/api/audit/{rev}", get(handle_audit))
176:        .route("/api/context-pack", post(handle_context_pack))
192:        .route("/api/sprint-history/diff/{sha}", get(handle_commit_diff))
198:        .route("/api/terminal/ws", get(handle_terminal_ws))
```
- Ce qui manque : les 11 appels existent et les chemins correspondent, mais les shapes TS ne mirrorent pas entièrement Serde. `SprintHistoryResult` Rust expose `roadmap`, `commits`, `verification` (`sprint_history.rs:17`, `:22`, `:27`) absents de `SprintHistory` TS (`operator.ts:174-201`). `TestSummary` Rust expose aussi `rust_delta`, `vitest_delta`, `size_limit` (`sprint_history.rs:77-84`) absents côté TS. `ContextPack` TS omet `operator_intent`, sérialisé par Rust (`operator_server.rs:596-600`).

### Livrable 4 : Front lib cast
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/lib/cast.ts:30`, `tools/factory-operator/src/lib/cast.test.ts:19`
- Evidence :
```ts
30:export function parseCast(raw: string): Cast {
40:      value = JSON.parse(trimmed)
42:      continue // a partial/garbled line — skip, never throw
58:    // An output event: [time, "o", data].
60:    if (Array.isArray(value) && value.length >= 3 && value[1] === 'o')
```
```ts
19:  it('skips malformed / partial lines without throwing...', () => {
21:    const cast = parseCast(raw)
22:    expect(cast.events).toEqual([{ time: 0.1, data: 'ok' }])
```

### Livrable 5 : Front lib verdict
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/lib/verdict.ts:21`, `tools/factory-operator/src/lib/verdict.test.ts:35`
- Evidence :
```ts
21:export const VERIFY_ETAT = {
22:  awaiting: 'En attente de session agent · 0 verdict auto-clos',
23:  bootstrap: 'Inspection bootstrap · terminal + procédé',
24:} as const
```
```ts
39:    for (const text of Object.values(VERIFY_ETAT)) {
40:      expect(text).not.toMatch(/\bPASS\b/)
41:      expect(text).not.toMatch(/Vérifié|Approuvé/)
```
- `reviewTone`/`preflightTone` mappent des verdicts restitués (`verdict.ts:32-48`) vers des tons, sans score.

### Livrable 6 : Front state
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/state/useOperator.ts:160`, `tools/factory-operator/src/state/useCommitDiff.ts:27`, `tools/factory-operator/src/state/useOperator.test.ts:121`
- Evidence :
```ts
160:  // Selecting a focal MODE returns to that scene, closing any open inspector.
161:  const setMode = useCallback((m: FocalMode) => {
162:    setModeState(m)
163:    setSurface(null)
165:  const openSurface = useCallback((s: SecondarySurface) => setSurface(s), [])
169:  const preparePack = useCallback(() => setSurface('knowledge'), [])
```
```ts
27:  useEffect(() => {
30:    getCommitDiff(sha, controller.signal)
32:        if (!controller.signal.aborted) setResolved({ sha, diff, error: null })
45:  const ready = resolved !== null && resolved.sha === sha
49:    loading: sha !== null && !ready,
```

### Livrable 7 : VERIFY bootstrap
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/components/verify/VerifyScene.tsx:24`, `Terminal.tsx:29`, `TerminalXterm.tsx:73`
- Evidence :
```tsx
24:      <Terminal />
26:      {/* permanent gates + état band — honest, never a verdict */}
30:          <span title="câblage Phase G">non câblées — Phase G</span>
35:            {VERIFY_ETAT.bootstrap}
```
```tsx
29:  if (!started) {
42:            onClick={() => setStarted(true)}
45:            Démarrer la session terminal
63:        <Suspense ...>
66:          <TerminalXterm onStatus={setStatus} />
```
- `TerminalXterm` ouvre le WS same-origin via `new WebSocket(terminalWsUrl(resume))` (`TerminalXterm.tsx:73`).

### Livrable 8 : Front MUR
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/components/steer/Mur.tsx:49`, `SteerScene.tsx:41`
- Evidence :
```tsx
53:            {onPrepare ? (
54:              <button
57:                onClick={onPrepare}
61:                Préparer le pack
64:            <button ...>Retour à la composition</button>
```
- Invariant vérifié : pas de bouton Forcer/Override/Bypass. Les mots apparaissent seulement en texte négatif à `Mur.tsx:50` (“aucun « Forcer » ...”), pas comme affordance.

### Livrable 9 : ContextPackInspector
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/components/verify/ContextPackInspector.tsx:30`, `:69`, `:107`, test `:57`
- Evidence :
```tsx
30:function groups(pack: Partial<ContextPack>): PackGroup[] {
39:    { label: 'prompts', refs: prompts },
40:    { label: 'docs de procédé', refs: pack.process_docs ?? [] },
41:    { label: 'knowledge consultatif', refs: pack.authoring_knowledge ?? [] },
```
```tsx
69:      {entry.exists && entry.hash ? (
71:          {entry.hash}
77:        <span ... title="le fichier a changé depuis le scellé de la session">
78:          ◦ dérive — relu
```
- Tests utiles : hash rendu + brouillon non autoritaire (`ContextPackInspector.test.tsx:57-66`) et dérive D2 (`:69-89`).

### Livrable 10 : Inspecteurs secondaires
- Statut : CONFIRME
- Fichier(s) : `SurfaceHost.tsx:18`, `ProcedeSurface.tsx:193`, `DiffView.tsx:38`, `ConformiteCard.tsx:69`, `SessionsSurface.tsx:54`, `KnowledgeSurface.tsx:16`
- Evidence :
```tsx
18:const ProcedeSurface = lazy(...)
19:const SessionsSurface = lazy(...)
20:const KnowledgeSurface = lazy(...)
47:<Suspense ...>
48:{surface === 'procede' ? <ProcedeSurface /> : ...}
```
```tsx
193:{/* preflight bilan + verdict frise */}
201:{history.phases.map((p) => (
205:  className={`h-2 w-2 rounded-sm ${toneBg(reviewTone(p.review_verdict))}`}
213:{history.phases.map((p) => (
220:  preflightFile={fileByPhase.get(p.letter) ?? null}
```
```tsx
38:{file.hunks.map((hunk, hi) => (
40:  <div ...>{hunk.header}</div>
41:  {hunk.lines.map((line, li) => (
52:    <span className="whitespace-pre">{line.content}</span>
```
- `ConformiteCard` liste les “manques” (`ConformiteCard.tsx:69-78`) et pas une coche d’approbation. `SessionsSurface` lit journal + sessions (`:54-63`), session STEER (`:99-108`) et rejeu `.cast` (`:139-141`). `KnowledgeSurface` marque l’advisory non autoritaire (`:16-23`). Tests présents et avec assertions : Procede, DiffView, ConformiteCard, SessionsSurface.

### Livrable 11 : Câblage + config
- Statut : CONFIRME
- Fichier(s) : `App.tsx:39`, `Rail.tsx:75`, `.size-limit.json:24`, `vite.config.ts:68`, `vitest.config.ts:33`, `scan-front-discipline.sh:31`
- Evidence :
```tsx
39:<Suspense fallback={<div className="flex-1 bg-s0" />}>
40:  {op.surface !== null ? (
41:    <SurfaceHost op={op} />
45:    <VerifyScene />
```
```json
24:    "name": "vendor-xterm",
26:    "limit": "360 KB",
31:    "name": "vendor-xterm-css",
33:    "limit": "6 KB"
```
- `vite.config.ts:82-83` chunk `@xterm` en `vendor-xterm`; `vitest.config.ts:38-39` exclut `TerminalXterm`/`CastXterm`; `scan-front-discipline.sh:31-33` exclut les `*.test`.

### Livrable 12 : E2E VERIFY
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/e2e/verify.spec.ts:16`, `:29`
- Evidence :
```ts
16:test('VERIFY mode shows the bootstrap scene...', async ({ page }) => {
22:  await expect(page.getByTestId('terminal-start')).toBeVisible()
25:  const etat = await page.getByTestId('verify-etat').textContent()
26:  expect(etat ?? '').not.toMatch(/\bPASS\b/)
```
```ts
29:test('the Procédé inspector restitutes ≥1 phase...', async ({ page }) => {
34:  const phases = page.getByTestId('procede-phase')
42:    .getByTestId('verdict-pill')
47:  await expect(...getByText(/\d+\s*%/)).toHaveCount(0)
```

## Invariants et exécutions
- `PASS` UI : `bash scripts/scan-front-discipline.sh` => clean.
- Score/jauge/% : scan `rg` ne trouve pas de scoring de production ; seulement tests/commentaires.
- Forcer/Override/Bypass : pas d’affordance ; occurrence UI uniquement comme négation dans le MUR.
- Diff : `DiffView` rend `file.hunks` / `hunk.lines` fournis par Rust ; pas de re-diff JS constaté.
- Tests ciblés lancés : Rust `authoring_knowledge` 2 passed ; Vitest ciblé 8 files / 33 tests passed ; Playwright `e2e/verify.spec.ts` 2 passed ; `npm run size` OK (`app` 35.97 KB, `vendor-xterm` 341.54 KB, CSS xterm 3.94 KB).

## Résumé final
- Total livrables : 12
- Confirmés : 11
- Gaps : 0
- Partiels : 1