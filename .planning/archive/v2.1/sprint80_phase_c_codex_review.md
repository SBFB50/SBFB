Audit du diff staged effectué. Vérifications lancées : `npm run lint`, `npx tsc --noEmit -p tsconfig.app.json`, `npm run test:unit`, `npm run test:e2e`.

### Livrable 1 : useTokenStream SSE
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/lib/useTokenStream.ts:117`, `:134`, `:152`, `:178`; tests `useTokenStream.test.ts:71`, `:95`, `:108`, `:152`
- Evidence :
```ts
117 const myRun = ++runIdRef.current
118 controllerRef.current?.abort()
134 const res = await fetch(url, { credentials: 'same-origin', ... })
156 if (handle(payload)) { latched = true; break read }
178 dispatch({ kind: signal.aborted ? 'aborted' : 'ended' })
```
Fetch + reader + TextDecoder + AbortController, pas d’`EventSource`, latch terminal, `ended`, abort `aborted`, supersede par `runId`, credentials same-origin.

### Livrable 2 : parseur SSE pur
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/lib/sseFrames.ts:20`, tests `sseFrames.test.ts:13`, `:19`, `:31`, `:41`
- Evidence :
```ts
21 let line = ''
22 let dataLines: string[] = []
28 const l = raw.endsWith('\r') ? raw.slice(0, -1) : raw
36 if (l.startsWith(':')) return
56 end(): string[] {
```
Carry ligne/frame, CRLF, commentaires, flush `end()` couverts par assertions utiles.

### Livrable 3 : union StreamChunk 6 valeurs
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/llm_bridge.rs:42`, `operator_server.rs:1063`, `tools/factory-operator/src/lib/streamChunk.ts:12`, tests `streamChunk.test.ts:23`
- Evidence :
```ts
12 export type StreamChunk =
13   | { type: 'delta'; text: string }
15   | { type: 'done'; cost_usd?: number; duration_ms?: number; result: string }
17   | { type: 'debug'; label: string; content: string }
18   | { type: 'requires_gate'; message: string }
```
Confronté au Rust : `StreamChunk` Rust a 5 variantes (`llm_bridge.rs:44-58`), `requires_gate` est bien JSON forgé hors serde par `sse_gate()`.

### Livrable 4 : focale STEER variante B
- Statut : CONFIRME
- Fichier(s) : `Composer.tsx:53`, `SteerScene.tsx:52`, `:82`, `Atelier.tsx:46`, `TechDetails.tsx:43`, `useOperator.ts:122`
- Evidence :
```tsx
52 if (!op.hasTurn) ... <Composer variant="grand" ... />
82 <Composer variant="dock" ... />
46 const body = turn.status === 'done' ? turn.result ?? turn.text : turn.text
43 Promise.allSettled([getPrompt(kind, provider, ...), getProviders(...)])
126 streamStart(streamUrl(sessionId))
```
Grand/dock réel, atelier accumule les sorties, Network affiche `Done.result`, détails techniques lisent `/api/prompt/{kind}`, relance = nouveau GET stream sans re-send.

### Livrable 5 : MUR requires_gate
- Statut : CONFIRME
- Fichier(s) : `useOperator.ts:105`, `:106`, `:111`; `Mur.tsx:42`, tests `Mur.test.tsx:16`, `useOperator.test.ts:44`, E2E `steer.spec.ts:35`
- Evidence :
```ts
105 const result = await sendMessage(id, { message: trimmed, provider })
106 if (result.requires_gate) {
108   setSendGate(GATE_MESSAGE)
109   return
111 streamStart(streamUrl(id))
```
Pas de préfiltre client, `/send` décide. MUR inline, seul bouton “Retour”, tests unitaires et E2E vérifient absence d’ouverture de stream.

### Livrable 6 : rail orientation
- Statut : CONFIRME
- Fichier(s) : `useRailStatus.ts:42`, `OrientationBar.tsx:42`, `:66`, `Rail.tsx:78`, `VerifyPlaceholder.tsx:38`
- Evidence :
```ts
42 getContext(controller.signal)
45 sprint: typeof ctx.sprint === 'number' ? ctx.sprint : null
49 dirty: Array.isArray(ctx.dirty_files) ? ctx.dirty_files.length : null
66 {/* Gates pulse — not wired before Phase G; never a verdict. */}
78 <ModeButton active={mode === 'steer'} label="STEER" ... />
```
`/api/context` seul, mode STEER/VERIFY manuel, gates placeholder non câblé, surfaces secondaires inertes. Cuts Phase D/G/H respectés.

### Livrable 7 : invariants front pur
- Statut : CONFIRME
- Fichier(s) : `api/operator.ts:25`, `:35`, `catalog/intentions.ts:25`, `boot.spec.ts:40`, commandes git/rg
- Evidence :
```ts
25 async function getJson<T>(path: string, signal?: AbortSignal)
27   credentials: 'same-origin'
35 async function postJson<T>(path: string, body: unknown)
38   credentials: 'same-origin'
25 export const INTENTIONS: readonly IntentionPreset[] = [
```
`git diff --cached --name-only | rg "\.rs$|daemon"` : aucun résultat. Pas de route daemon, pas de token JS, intentions statiques, pas de `/api/artifacts/draft` réel, pas de `/api/gates` réel, CSP testé.

### Livrable 8 : tests
- Statut : CONFIRME
- Fichier(s) : `useTokenStream.test.ts:71`, `useOperator.test.ts:44`, `useRailStatus.test.ts:48`, `boot.spec.ts:27`, `steer.spec.ts:35`
- Evidence :
```txt
Vitest: Test Files 9 passed (9), Tests 52 passed (52)
Playwright: 4 passed
steer.spec.ts:56 expect(streamOpened).toBe(false)
boot.spec.ts:55 expect(violations).toEqual([])
```
Les tests ont des assertions réelles : terminal latch, abort, ended, supersede, Network result, MUR sans spawn, cookie/CSP.

## Resume final
- Total livrables : 8 / Confirmes : 8 / Gaps : 0 / Partiels : 0

## Bugs / risques
- P3 : `useTokenStream.ts:156-159` latche puis sort du read loop, mais le `finally` fait `releaseLock()` sans `reader.cancel()` sur terminal. Avec le backend actuel les flux ferment, donc pas gap Phase C ; avec un flux post-terminal infini, la connexion pourrait rester ouverte.
- P3 : `catalog/intentions.ts:34` rend “Vérifier avant validation” comme CTA d’intention. Ce n’est pas un verdict UI, mais si l’invariant “aucun mot Vérifie” est interprété littéralement, cette copie devra être renommée.