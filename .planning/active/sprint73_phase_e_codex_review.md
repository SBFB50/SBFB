### Livrable 1 : `searchBrowse` via `callDaemon`
- Statut : CONFIRME
- Fichier(s) : `web/src/api/daemon.ts:203`, `web/src/api/daemon.ts:373`
- Evidence :
```ts
203: async function callDaemon<T>(
212:     res = await authFetch(url, {
226:   if (res.status === 503) {
249:   const parsed = bodySchema.safeParse(raw);
```
```ts
373: export function searchBrowse(
379:   const params = new URLSearchParams({
384:   return callDaemon(
386:     `/api/daemon/search?${params.toString()}`,
```
`q`, `limit`, `offset` passent par `URLSearchParams`, donc `q` ne peut pas s’échapper de la query string. `searchBrowse` retourne bien `DaemonResult<SearchResponse>` via `callDaemon`, pas via `fetch` brut.

### Livrable 2 : Schémas Zod alignés sur le JSON Rust
- Statut : CONFIRME
- Fichier(s) : `web/src/api/daemon.ts:330`, `crates/nexus-shell-daemon/src/http.rs:2010`, `crates/nexus-coordinator-rs/src/search.rs:8`
- Evidence :
```ts
330: export const SearchResultSchema = z
332:     project_id: z.string(),
338:     score: z.number(),
339:     repo_url: z.string().nullable(),
343:     is_open_source: z.boolean(),
```
```ts
354: export const SearchResponseSchema = z
356:     results: z.array(SearchResultSchema),
357:     total: z.number().int().min(0),
358:     took_ms: z.number().int().min(0),
```
```rust
2010:             serde_json::json!({
2011:                 "project_id": r.project_id,
2017:                 "score": r.score,
2022:                 "repo_url": r.repo_url,
2026:                 "is_open_source": r.is_open_source,
```
Correspondance clé par clé vérifiée : les 12 clés Rust sont présentes côté Zod, les 4 champs provenance sont `.nullable()` et non `.optional()`, `is_open_source` est `z.boolean()`, `score` est `z.number()` sans `.min()`. L’enveloppe `{ results, total, took_ms }` est aussi `.strict()`.

### Livrable 3 : Barre de recherche Browse et rendu sécurisé
- Statut : CONFIRME
- Fichier(s) : `web/src/pages/Browse.tsx:65`, `web/src/pages/Browse.tsx:153`, `web/src/pages/Browse.tsx:168`, `web/src/pages/Browse.tsx:237`
- Evidence :
```tsx
65:   const searchQuery = useQuery({
66:     queryKey: ["daemon-search", coordUrl, trimmed],
67:     queryFn: () => searchBrowse(coordUrl, trimmed),
68:     enabled: isSearching,
```
```tsx
153: function isHttpsUrl(url: string | null | undefined): url is string {
154:   return typeof url === "string" && url.startsWith("https://");
```
```tsx
168:         <input
172:           placeholder="Rechercher une app par nom, catégorie ou description"
174:           data-testid="browse-search-input"
```
```tsx
267:           {hit.archive_hash && (
269:               P2P
272:           {hit.provenance_hash && (
278:               Provenance
281:           {repoUrl && (
291:               Source
```
Le champ dédié est au-dessus du rendu contenu, la query React Query est bien `["daemon-search", coordUrl, trimmed]`, activée seulement si le terme est non vide. Le lien `Source` n’est rendu que si `repo_url` commence par `https://`.

### Livrable 4 : Tests Vitest avec assertions réelles
- Statut : CONFIRME
- Fichier(s) : `web/src/api/__tests__/daemon.test.ts:510`, `web/src/api/__tests__/daemon.test.ts:566`, `web/src/pages/__tests__/Browse.test.tsx:123`
- Evidence :
```ts
520:     const result = await searchBrowse(BASE, "react");
533:     expect(String(urlArg)).toBe(
534:       `${BASE}/api/daemon/search?q=react&limit=20&offset=0`,
```
```ts
547:     await searchBrowse(BASE, "a&b=c d", 5, 10);
549:     expect(String(calls[0][0])).toBe(
550:       `${BASE}/api/daemon/search?q=a%26b%3Dc+d&limit=5&offset=10`,
```
```ts
584:   it("parses a hit whose triplet is null (non-release op)", () => {
599:     expect(parsed.success).toBe(true);
605:   it("rejects a hit that omits a provenance key (strict, not optional)", async () => {
614:     await expect(searchBrowse(BASE, "x")).rejects.toThrow(/protocol error/);
```
```tsx
151:   it("does not render a non-https repo_url as a link (XSS guard)", async () => {
156:           makeSearchHit({ repo_url: "javascript:alert(1)" }),
170:     expect(screen.queryByTestId("search-repo-link")).not.toBeInTheDocument();
```
Tests ciblés exécutés : `npm run test:unit -- src/api/__tests__/daemon.test.ts src/pages/__tests__/Browse.test.tsx` depuis `web/` : 2 fichiers passés, 33 tests passés.

## Résumé final
- Total livrables : 4
- Confirmés : 4
- Gaps : 0
- Partiels : 0