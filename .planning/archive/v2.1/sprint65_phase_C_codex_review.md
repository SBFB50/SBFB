### Livrable 1 : Badge dynamique BrowsedProject
- Statut : PARTIEL
- Fichier(s) : `web/src/pages/BrowsedProject.tsx:31`, `web/src/pages/BrowsedProject.tsx:183`, `web/src/pages/BrowsedProject.tsx:291`
- Evidence :
```tsx
183:   const verifyQuery = useQuery({
184:     queryKey: ["provenance-verify", coordUrl, entry.project_id],
185:     queryFn: async () => {
186:       const resp = await authFetch(
187:         `${coordUrl}/api/v1/project/${encodeURIComponent(entry.project_id)}/provenance`,
```
```tsx
307:                 {verifyQuery.isLoading ? (
308:                   <>
309:                     <Loader2 className="h-3 w-3 animate-spin" />
310:                     Verification...
```
```tsx
312:                 ) : verifyQuery.isSuccess && verifyQuery.data.verified ? (
313:                   <>
314:                     <FileCheck className="h-3 w-3" />
315:                     Signature verifiee
```
```tsx
317:                 ) : verifyQuery.isError || (verifyQuery.isSuccess && !verifyQuery.data.verified) ? (
318:                   <>
319:                     <AlertTriangle className="h-3 w-3" />
320:                     Verification echouee
```
- GAP : le badge initial `Provenance` existe, mais sa classe fallback est verte (`bg-emerald... text-emerald...`) et non neutre aux lignes `299-301` / `323-326`. Estimation fix : 3 LOC.

### Livrable 2 : scan-trust-wording.sh
- Statut : PARTIEL
- Fichier(s) : `scripts/scan-trust-wording.sh:13`, `scripts/scan-trust-wording.sh:26`, `scripts/scan-trust-wording.sh:47`, `scripts/scan-trust-wording.sh:63`, `scripts/scan-trust-wording.sh:78`, `scripts/scan-trust-wording.sh:91`
- Evidence :
```bash
13: filter_noise() {
14:   grep -vE '(__tests__|\.test\.|\.spec\.)' \
15:   | grep -vE '(SPRINT_LOG|\.planning/|archive/)' \
16:   | grep -vE '^\s*//' \
```
```bash
26: BARE_VERIFIE=$(grep -rnEi '\bverifi(e|ee|es|er)\b' \
27:   web/src/ examples/ \
28:   --include='*.tsx' --include='*.ts' --include='*.js' --include='*.html' \
```
```bash
91: if [ "$VIOLATIONS" -gt 0 ]; then
92:   echo "scan-trust-wording: $VIOLATIONS violation(s) found"
93:   exit 1
94: fi
```
- GAP : les 4 patterns et `exit 1` sont présents, et `bash scripts/scan-trust-wording.sh` retourne `scan-trust-wording: clean`. Mais le script scanne `web/src/ examples/`, pas les docs publics (`docs/` absent des cibles). Estimation fix : 8 LOC.

### Livrable 3 : 3 tests Vitest BrowsedProject
- Statut : CONFIRME
- Fichier(s) : `web/src/pages/__tests__/BrowsedProject.test.tsx:363`, `web/src/pages/__tests__/BrowsedProject.test.tsx:382`, `web/src/pages/__tests__/BrowsedProject.test.tsx:401`
- Evidence :
```tsx
363:   it("badge shows 'Signature verifiee' after successful verification", async () => {
370:       "/provenance": {
372:         verified: true,
378:       expect(screen.getByText("Signature verifiee")).toBeInTheDocument();
```
```tsx
382:   it("badge shows 'Verification echouee' when verification fails", async () => {
389:       "/provenance": {
391:         verified: false,
397:       expect(screen.getByText("Verification echouee")).toBeInTheDocument();
```
```tsx
401:   it("badge shows 'Verification...' while loading provenance", async () => {
406:         if (path.includes("/provenance")) {
407:           return new Promise<Response>(() => {});
432:       expect(screen.getByText("Verification...")).toBeInTheDocument();
```
- Verification : `npm run test:unit -- BrowsedProject` passe, `1` fichier, `18` tests.

## Resume final
- Total livrables : 3
- Confirmes : 1
- Gaps : 0
- Partiels : 2
- Estimation totale LOC fixes manquants : 11