Audit effectué sur `master` live. Worktree modifié.

### Livrable 1 : Browse.tsx Provenance + FileCheck
- Statut : CONFIRME
- Fichier(s) : `web/src/pages/Browse.tsx:8`, `web/src/pages/Browse.tsx:253`
- Evidence :
```tsx
8: import { ExternalLink, FileCheck, Globe, Play, RefreshCw, Signal, SignalZero, Sparkles } from "lucide-react";
253: {entry.provenance_hash && (
258:   <FileCheck className="h-2.5 w-2.5" />
259:   Provenance
```

### Livrable 2 : BrowsedProject.tsx badges
- Statut : PARTIEL
- Fichier(s) : `web/src/pages/BrowsedProject.tsx:21`, `web/src/pages/BrowsedProject.tsx:258`, `web/src/pages/BrowsedProject.tsx:273`
- Evidence :
```tsx
21:   FileCheck,
258: {(entry.source ?? "curator") === "direct" && (
260:   Upload direct
273: {entry.provenance_hash && (
280:   <FileCheck className="h-3 w-3" />
281:   Provenance
```
- GAP : la note demandée `Provenance auto-attestee (SLSA L1)` sous le badge est absente (`rg` ne trouve aucune occurrence dans `BrowsedProject.tsx`).
- Estimation fix : 2 LOC.

### Livrable 3 : GpuConsentDialog.tsx L2
- Statut : PARTIEL
- Fichier(s) : `web/src/components/GpuConsentDialog.tsx:66`
- Evidence :
```tsx
66:   2: {
67:     title: "Apps depuis un depot public",
68:     hint: "Accepte les apps deployees depuis un depot Git public (provenance auto-attestee).",
70:       "Apps a source verifiable (SLSA L1). Exposition Sybil si contributeur malveillant.",
```
- GAP : l’ancien libellé n’est plus là, mais la chaîne exacte attendue `Apps deployees depuis un depot public (provenance auto-attestee)` n’est pas présente.
- Estimation fix : 2 LOC.

### Livrable 4 : Network.tsx L2 Depot public
- Statut : CONFIRME
- Fichier(s) : `web/src/pages/Network.tsx:353`
- Evidence :
```tsx
353: const LEVEL_LABELS: Record<ConsentLevel, string> = {
354:   1: "L1 — Mes projets",
355:   2: "L2 — Depot public",
356:   3: "L3 — Whitelist",
```

### Livrable 5 : Curators.tsx curator
- Statut : CONFIRME
- Fichier(s) : `web/src/pages/Curators.tsx:141`, `web/src/pages/Curators.tsx:235`
- Evidence :
```tsx
141: <h3 className="mb-1 font-bold">Ajouter un curator</h3>
144: hexadecimaux) d'un curator.
235: <h3 className="mb-1 font-bold">Aucun curator suivi</h3>
237: Ajoute un curator ci-dessus pour commencer a recevoir
```

### Livrable 6 : Protocol Explorer vocabulaire
- Statut : CONFIRME
- Fichier(s) : `examples/sbfb-explorer/index.html:144`, `examples/sbfb-explorer/index.html:340`, `examples/sbfb-explorer/index.html:414`, `examples/sbfb-explorer/index.html:456`
- Evidence :
```html
144: SLSA L1 signé Ed25519. L'archive réseau est construite
145: depuis le dépôt source par le noeud local. C'est une
146: auto-attestation.
340: <h3>Chaîne de provenance</h3>
414: <h3>Source verifiable par construction</h3>
420: Inspire par F-Droid — les apps publiques sont deployees
456: <a class="src" data-path="examples/sbfb-explorer/">a source verifiable</a>
```
- Note : `app.js` ne contient pas les anciennes chaînes ciblées.

### Livrable 7 : PUBLISH_MODEL.md
- Statut : GAP
- Fichier(s) : `docs/architecture/PUBLISH_MODEL.md:29`
- Evidence :
```md
29: | Etat | Source | Badge UI | Workers publics | Mutable | Preuve |
31: | **Local Draft** | disque dev, Vite/dev server, daemon local | aucun | non | oui (c'est du dev) | aucune |
32: | **Unverified Build** | zip uploade sans provenance | "non verifie" | seulement opt-in `is_open_source=false` | non (blob immutable, mais pas de preuve source) | hash artefact seulement |
33: | **Verified Release** | commit public + provenance SLSA L1 | "provenance auto-attestee" | oui, consent L2+ | non (blob + commit + hash lies) | repo_url + commit_sha + artifact_hash + provenance_hash |
```
- GAP : `Release avec provenance auto-attestee` est absent, et `Verified Release` reste utilisé aux lignes 33, 56, 111, 178, 192.
- Estimation fix : 8 LOC.

### Livrable 8 : test BrowsedProject Upload direct
- Statut : CONFIRME
- Fichier(s) : `web/src/pages/__tests__/BrowsedProject.test.tsx:283`
- Evidence :
```tsx
283: it("renders source badge 'Upload direct' for direct entries in top bar", async () => {
292:   await waitFor(() => {
293:     expect(screen.getByTestId("browsed-project")).toBeInTheDocument();
295:   expect(screen.getByText("Upload direct")).toBeInTheDocument();
```

## Resume final

- Total livrables : 8
- Confirmes : 5
- Gaps : 1
- Partiels : 2
- Estimation totale LOC fixes manquants : 12