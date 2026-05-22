### Livrable 1 : ProofCard.tsx
- Statut : CONFIRME
- Fichier(s) : `web/src/components/ProofCard.tsx:3`, `web/src/components/ProofCard.tsx:88`, `web/src/components/ProofCard.tsx:138`, `web/src/components/ProofCard.tsx:147`, `web/src/components/ProofCard.tsx:169`, `web/src/components/ProofCard.tsx:193`, `web/src/components/ProofCard.tsx:224`, `web/src/components/ProofCard.tsx:245`
- Evidence :
```tsx
169:       <button
170:         type="button"
171:         onClick={() => setExpanded((prev) => !prev)}
172:         className={`flex items-center gap-1.5 rounded-full px-3 py-1.5 text-[11px] font-medium transition-colors ${scoreBgColor(card.confidence)} ${scoreColor(card.confidence)} hover:opacity-80`}
177:         <span data-testid="proof-card-score">{card.confidence}/100</span>
```
CONFIRME : composant réel, expandable, score `0-100`, labels français, icônes `lucide-react`, couches de preuve via `buildLayers`, badge de risque et facteurs de risque.

### Livrable 2 : Integration BrowsedProject.tsx
- Statut : CONFIRME
- Fichier(s) : `web/src/pages/BrowsedProject.tsx:25`, `web/src/pages/BrowsedProject.tsx:184`, `web/src/pages/BrowsedProject.tsx:188`, `web/src/pages/BrowsedProject.tsx:190`, `web/src/pages/BrowsedProject.tsx:270`, `web/src/pages/BrowsedProject.tsx:360`
- Evidence :
```tsx
184:   const proofCardQuery = useQuery({
185:     queryKey: ["proof-card", coordUrl, entry.project_id],
187:       const resp = await authFetch(
188:         `${coordUrl}/api/daemon/proof-card/${encodeURIComponent(entry.project_id)}`,
190:       if (resp.status === 404) return null;
```
CONFIRME : `useQuery` appelle bien `GET /api/daemon/proof-card/{project_id}` via `authFetch`, avec `encodeURIComponent`, et `404 -> null`. Le rendu est dans la top bar auto-hide : `web/src/pages/BrowsedProject.tsx:270` puis `<ProofCard ... />` aux lignes `360-363`.

### Livrable 3 : THREAT_MODEL.md §12 T-PROOFCARD-FORMULA-GAME
- Statut : CONFIRME
- Fichier(s) : `docs/security/THREAT_MODEL.md:606`, `docs/security/THREAT_MODEL.md:615`, `docs/security/THREAT_MODEL.md:620`, `docs/security/THREAT_MODEL.md:632`, `docs/security/THREAT_MODEL.md:649`, `docs/security/THREAT_MODEL.md:658`, `docs/security/THREAT_MODEL.md:684`
- Evidence :
```md
615: ### T-PROOFCARD-FORMULA-GAME — Score gaming sans substance
620: 1. **Provenance factice** : generer une provenance auto-attestee
623: 2. **Curator collusion** : creer plusieurs curator keypairs et
626: 3. **License tag gaming** : declarer une licence SPDX dans le
629: 4. **Freshness gaming** : re-deployer periodiquement sans
```
CONFIRME : les 4 vecteurs sont documentés, les mitigations sont présentes lignes `632-647`, la table `Dimension/Valeur` lignes `649-654`, la renumérotation `## 13. Revue et evolution` ligne `658`, et l’historique v5 lignes `684-685`.

### Livrable 4 : Tests Vitest ProofCard
- Statut : CONFIRME
- Fichier(s) : `web/src/components/__tests__/ProofCard.test.tsx:36`, `web/src/components/__tests__/ProofCard.test.tsx:37`, `web/src/components/__tests__/ProofCard.test.tsx:42`, `web/src/components/__tests__/ProofCard.test.tsx:56`, `web/src/components/__tests__/ProofCard.test.tsx:69`, `web/src/components/__tests__/ProofCard.test.tsx:95`, `web/src/components/__tests__/ProofCard.test.tsx:103`, `web/src/components/__tests__/ProofCard.test.tsx:108`, `web/src/components/__tests__/ProofCard.test.tsx:113`
- Evidence :
```tsx
36: describe("ProofCard", () => {
37:   it("renders the confidence score", () => {
39:     expect(screen.getByTestId("proof-card-score")).toHaveTextContent("100/100");
42:   it("renders evidence layers when expanded", async () => {
48:     expect(screen.getByTestId("proof-card-layers")).toBeInTheDocument();
```
CONFIRME : 8 tests existent avec assertions utiles. Vérification exécutée : `npm run test:unit -- ProofCard.test.tsx` -> `1 passed`, `8 passed`.

## Resume final
- Total livrables : 4
- Confirmes : 4
- Gaps : 0
- Partiels : 0