Je n’ai pas lancé Playwright ni cargo. J’ai lancé le gate léger autorisé : `bash scripts/scan-front-discipline.sh` → `clean`.

### Livrable 1 : T1 3a local SSE
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/e2e/steer.spec.ts:63`, `:92`, `:119`, `:123`, `:134`
- Evidence :
```ts
63: async function wireTranscript(
68:   const session = await request.post('/api/chat/session', {
74:   const sent = await request.post(`/api/chat/${id}/send`, {
79:   const stream = await request.get(`/api/chat/${id}/stream`, { headers: AUTH })
92:   const frames = await wireTranscript(request, 'local', 'écris une salutation brève')
93:   expect(frames.match(/"type":"delta"/g) ?? [], 'two streamed delta frames').toHaveLength(2)
94:   expect(frames.match(/"type":"done"/g) ?? [], 'exactly ONE Done frame (PO-14)').toHaveLength(1)
```
UI/log/counter aussi couverts : rendu `FIXTURE_OLLAMA_TEXT` ligne 119, `/log` assistant unique lignes 123-131, delta `generate +1` lignes 134-137.

### Livrable 2 : T1 3b Network
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/e2e/steer.spec.ts:147`, `:172`, `:175`, `:185`
- Evidence :
```ts
147:   const frames = await wireTranscript(request, 'network', 'demande la réponse du réseau fixture')
148:   expect(frames.match(/"type":"delta"/g) ?? [], 'zero delta frames').toHaveLength(0)
149:   expect(frames.match(/"type":"done"/g) ?? [], 'exactly ONE Done frame (PO-14)').toHaveLength(1)
150:   expect(frames.includes('"label":"network-poll"'), 'the poll path was exercised').toBe(true)
```
Le tour UI vérifie `FIXTURE_NETWORK_RESULT`, statut terminé, `/log` avec un assistant, puis `submit +1` aux lignes 172-188.

### Livrable 3 : Fixture daemon déterministe
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/e2e/serve-fixture-daemon.mjs:37`, `:64`, `:92`, `:116`
- Evidence :
```js
64:   if (req.method === 'POST' && path.endsWith('/tasks/submit')) {
69:     req.on('end', () => json(res, 200, { task: { task_id: 'e2e-task-1' } }))
81:   if (req.method === 'GET' && path.includes('/tasks/e2e-task-1')) {
87:       status: seen <= 1 ? 'dispatched' : 'completed',
```
Ollama NDJSON complet : lignes 93-108, avec `model/created_at/response/done`, write par objet, final `response: ''`. `Connection: close` est dans `json()` lignes 39-42 et l’arm NDJSON lignes 97-99. Bind loopback : ligne 116.

### Livrable 4 : Workspace git hermétique per-run
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/e2e/fixture-workspace.mjs:58`, `tools/factory-operator/e2e/serve-operator.mjs:41`, `crates/sbfb-factory/src/process.rs:56`
- Evidence :
```js
58: export function seedFixtureWorkspace(bundleSrc) {
59:   const ws = fs.mkdtempSync(path.join(os.tmpdir(), 'sbfb-op-e2e-ws-'))
61:   git(ws, 'init', '-q', '-b', 'main')
66:   git(ws, 'config', 'core.hooksPath', path.join(ws, '.no-hooks'))
```
Le seed crée l’ancre Sprint 0 et le commit Sprint 1 lignes 101-114, l’edit non-stage ligne 117, et copie le bundle lignes 119-121. `serve-operator` build depuis repo root lignes 51-54 puis spawn avec `cwd: workspace` et `SBFB_HOME` mkdtemp lignes 65-70.

### Livrable 5 : Câblage Playwright
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/e2e/fixtures.ts:8`, `tools/factory-operator/playwright.config.ts:34`
- Evidence :
```ts
34:   webServer: [
36:       command: 'node e2e/serve-operator.mjs',
50:         SBFB_DAEMON_ENDPOINT: `http://127.0.0.1:${FIXTURE_DAEMON_PORT}`,
51:         SBFB_NETWORK_POLL_INTERVAL_MS: '20',
52:         SBFB_NETWORK_TIMEOUT_SECS: '30',
53:         SBFB_OLLAMA_ENDPOINT: `http://127.0.0.1:${FIXTURE_DAEMON_PORT}`,
```
La deuxième entrée lance le daemon fixture et injecte `FIXTURE_NETWORK_RESULT` / `FIXTURE_OLLAMA_DELTAS` lignes 56-67. Les constantes partagées sont exportées dans `fixtures.ts:15-20`.

### Livrable 6 : Hermétisme du sous-test 2
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/e2e/steer.spec.ts:36`, `crates/sbfb-factory/src/provider_router.rs:80`
- Evidence :
```ts
36:   // Hermeticity (Phase I): route this benign turn at the fixture too —
40:   await page.getByTestId('provider-select').selectOption('local')
41:   await page.getByTestId('composer-input').fill('prépare le plan de la phase')
42:   await page.getByTestId('composer-launch').click()
```
Et côté Rust, `local` route vers Ollama, pas Claude : `provider_router.rs:82-83`. Claude/unknown reste seulement le fallback lignes 89-92.

### Livrable 7 : Scan anti-score/jauge
- Statut : PARTIEL
- Fichier(s) : `tools/factory-operator/scripts/scan-front-discipline.sh:41`, `:72`, `:91`, `:124`
- Evidence :
```bash
41: FORBIDDEN_SCORE='trust[- ]?score|score de ...
72: scan_scores() {
75:     grep -rniE --include='*.tsx' --include='*.ts' \
83:     grep -rniE --include='*.po' "^msgstr.*($FORBIDDEN_SCORE)" "$base" 2>/dev/null || true
91: self_test() {
```
Confirmé : axe score présent, self-test séparé `dir_tsx`/`dir_po` lignes 91-109, gate lancé et vert, 51 catalogues `.po` présents.  
Gap : le scan `.po` ne couvre que la ligne `msgstr ...`; une continuation PO multi-ligne (`msgstr ""` puis `"score..."`) n’est pas détectée. C’est une couverture partielle des `msgstr`, pas un parseur PO complet.

### Livrable 8 : Artefact T2 + harness
- Statut : PARTIEL
- Fichier(s) : `tools/factory-operator/scripts/t2-acceptance.mjs:39`, `:66`, `:118`, `:150`, `.planning/active/sprint80_t2_acceptance.json:1`
- Evidence :
```js
118:       const statuses = (spec.tests ?? []).map((t) => t.status)
120:         spec.ok && statuses.length > 0 && statuses.every((s) => s === 'expected')
121:       scenarios[TITLE_TO_ID.get(spec.title) ?? slug(spec.title)] = pass ? 'PASS' : 'BLOCK'
150: const artifact = {
152:   status: blocked ? 'BLOCK' : 'PASS',
```
Confirmé : artefact allowlist sans timestamp/durée/chemin/port/secret, `status: PASS`, 9 gates et 10 scénarios. `shell:true` win32 est présent lignes 31-42, exit 1 sur BLOCK ligne 161.  
Gap : le harness ne vérifie pas que les 10 ids attendus sont tous présents. Si un spec disparaît complètement, `scenarios` peut rester non vide et PASS sans BLOCK.

### Livrable 9 : CI + retrait `--passWithNoTests`
- Statut : CONFIRME
- Fichier(s) : `.github/workflows/ci.yml:175`, `.woodpecker/ci-linux.yml:108`, `tools/factory-operator/package.json:11`
- Evidence :
```yaml
175:       - name: "[3] vitest (unit — gates the PO-14 re-coverage at every push)"
176:         run: cd tools/factory-operator && npm run test:unit
178:       - name: "[4] discipline gates ...
190:       - name: "[8] hermetic T1 ...
```
Woodpecker ajoute `factory-operator-vitest` lignes 108-113. `package.json` retire bien `--passWithNoTests` sur `test:unit` et `test:coverage` lignes 11-13.

### Livrable 10 : Test unitaire 0-auto-reconnect
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/lib/useTokenStream.test.ts:152`
- Evidence :
```ts
152:   it('never re-opens the transport after a terminal — one fetch total (0 auto-reconnect)', async () => {
158:     mockFetch(closedBody([frame({ type: 'done', result: 'once' })]))
160:     act(() => result.current.start('/api/chat/x/stream'))
161:     await waitFor(() => expect(result.current.status).toBe('done'))
163:     expect(globalThis.fetch).toHaveBeenCalledTimes(1)
```

### Livrable 11 : Docs / commentaires
- Statut : CONFIRME
- Fichier(s) : `docs/rust/PATTERNS.md:4029`, `tools/factory-operator/src/lib/streamChunk.ts:3`, `tools/factory-operator/e2e/boot.spec.ts:10`
- Evidence :
```md
4029: ## §P72 — Sprint 80 Phase I: hermetic Operator E2E ...
4041: 1. **cargo config discovery is CWD-based.**
4048: 2. **win32 `.cmd` shims need a shell.**
4054: Related latent gap (carried, not patched from the harness):
4055: `collect_sprint_commits` falls back to the range `HEAD~50..HEAD`
```
`streamChunk.ts` ancre bien `sse_gate` par nom lignes 3-8. `boot.spec.ts` ne promet plus une phase future ; il pointe vers `steer.spec.ts` et `verify.spec.ts` lignes 10-11.

### Livrable 12 : Artefacts process
- Statut : PARTIEL
- Fichier(s) : `.planning/active/sprint80_phase_i_preflight.md:6`, `.planning/active/sprint80_phase_i_review.md:4`, `:124`, `:169`
- Evidence :
```md
6: **Verdict** : **PLAN-ADAPT**
136: ## 6. Adaptations PLAN-ADAPT (numérotées, evidence)
193: ## 10. Addendum 2026-07-02 — comblement S1a/S4
169: ## Verdict: PASS-PENDING
```
Confirmé : preflight contient PLAN-ADAPT, 9 adaptations et addendum §10 ; review contient PASS-PENDING et fixes post-review cohérents avec le code.  
Gap : la review n’est pas parfaitement cohérente sur le périmètre : elle annonce `16 fichiers` ligne 4, mais liste 12 modifiés + 5 nouveaux lignes 5-6, et le working tree contient aussi le fichier review non suivi. C’est un défaut de comptabilité process, pas de code.

## Défauts transverses
- `t2-acceptance.mjs` ne bloque pas la disparition complète d’un scénario attendu : voir livrable 8.
- `scan-front-discipline.sh` ne parse pas les continuations PO multi-lignes : voir livrable 7.
- Commentaire stale dans `tools/factory-operator/e2e/verify.spec.ts:5-8` : il parle encore des hunks du “THIS repo”, alors que le serveur est maintenant lancé avec `cwd=<fixture>`.
- Aucun fichier `.rs` modifié ou untracked. `package-lock.json` n’est pas modifié ; `package.json` ne change que les scripts.

## Résumé final
- Total livrables : 12
- Confirmés : 9
- Gaps : 0
- Partiels : 3