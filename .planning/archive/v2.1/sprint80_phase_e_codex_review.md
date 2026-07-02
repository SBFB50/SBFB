Verdict audit : **18/18 confirmés**, aucun `.rs` touché dans `git status` / `git diff`.

### Livrable 1 : `motion.ts`
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/lib/motion.ts:25`, `:28`, `:47`, `:57`
- Evidence : allowlist des 5 signatures, `REDUCED_MOTION_QUERY`, transitions nommées, `MOTION_TRAVEL_PX`. `rg SETTLE_MS|ALTITUDE_MS|GRAVITY_MS` : aucun hit. Test : `motion.test.ts:8-24`.

### Livrable 2 : `altitudeShift`
- Statut : CONFIRME
- Fichier(s) : `src/lib/altitudeShift.ts:21-49`
- Evidence : utilise `flushSync`, `document.startViewTransition`, fallback si API absente ou reduced-motion. Pas d’appel `animateView(` ; seul hit est un commentaire explicatif. Test : `altitudeShift.test.ts:35-64`.

### Livrable 3 : `usePrefersReducedMotion`
- Statut : CONFIRME
- Fichier(s) : `src/lib/usePrefersReducedMotion.ts:16-32`
- Evidence : `useState` initialise synchroniquement via `matchMedia`, abonnement `change`, cleanup `removeEventListener`. Test : `usePrefersReducedMotion.test.ts:22-58`.

### Livrable 4 : `MotionProvider`
- Statut : CONFIRME
- Fichier(s) : `src/components/motion/MotionProvider.tsx:25-35`, `src/components/verify/VerifyScene.tsx:21-27`
- Evidence : `<MotionConfig reducedMotion="user"><LazyMotion strict features={domAnimation}>`. Importé par la surface async VERIFY, pas par `App.tsx`.

### Livrable 5 : `GateFlip`
- Statut : CONFIRME
- Fichier(s) : `src/components/motion/GateFlip.tsx:18-47`
- Evidence : import nommé `span as MSpan` depuis `motion/react-m`; reduced-motion rend un `<span>` plain ; la valeur vient de la prop `value`. Test : `GateFlip.test.tsx:30-45`. Gate : `scan-front-discipline: clean`.

### Livrable 6 : `Reveal`
- Statut : CONFIRME
- Fichier(s) : `src/components/motion/Reveal.tsx:16-55`
- Evidence : import nommé `div as MDiv` depuis `motion/react-m`; reduced-motion rend des `<div>` plain ; aucun prop `layout`. Test : `Reveal.test.tsx:23-47`.

### Livrable 7 : `TokenCount`
- Statut : CONFIRME
- Fichier(s) : `src/components/motion/TokenCount.tsx:14-21`
- Evidence : CSS-only `motion-settle tabular-nums`, `key={String(value)}`, aucun import Motion. Test : `TokenCount.test.tsx:7-20`.

### Livrable 8 : `App.tsx`
- Statut : CONFIRME
- Fichier(s) : `src/App.tsx:10-15`, `:33-34`, `:56-64`
- Evidence : aucun provider Motion au hero ; `VerifyScene` est lazy ; wrapper `.motion-focal` autour de la zone focale.

### Livrable 9 : `useOperator`
- Statut : CONFIRME
- Fichier(s) : `src/state/useOperator.ts:161-171`
- Evidence : `setMode` enveloppe `setModeState` + `setSurface(null)` dans `altitudeShift`.

### Livrable 10 : `OrientationBar`
- Statut : CONFIRME
- Fichier(s) : `src/components/OrientationBar.tsx:13`, `:59-68`
- Evidence : compteurs dirty/staged rendus via `<TokenCount>`.

### Livrable 11 : `Mur`
- Statut : CONFIRME
- Fichier(s) : `src/components/steer/Mur.tsx:30-35`, `:53-55`
- Evidence : classe `motion-gravity`, invariant “aucun Forcer / Override / Bypass”. Test : `Mur.test.tsx:16-23`, `:33-39`.

### Livrable 12 : `VerifyScene`
- Statut : CONFIRME
- Fichier(s) : `src/components/verify/VerifyScene.tsx:21-54`
- Evidence : wrapper `<MotionProvider>`, bande gates en `<Reveal>/<RevealItem>`, état via `<GateFlip value={VERIFY_ETAT.bootstrap}>`. `VERIFY_ETAT.bootstrap` est défini sans verdict dans `src/lib/verdict.ts:21-24`.

### Livrable 13 : `index.css`
- Statut : CONFIRME
- Fichier(s) : `src/index.css:86-109`
- Evidence : `.motion-settle`, `.motion-gravity`, `::view-transition-old/new(focal)`, `.motion-focal`, rail + orientation exclus, reset `prefers-reduced-motion`.

### Livrable 14 : `vite.config.ts`
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/vite.config.ts:68-85`
- Evidence : `manualChunks` ne force que `vendor-react` et `vendor-xterm`; aucun `vendor-motion`.

### Livrable 15 : `.size-limit.json`
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/.size-limit.json:3-26`
- Evidence : `app <= 40 KB`, `verify-surface` sur `bundle/assets/VerifyScene-*.js <= 92 KB`, `css <= 21 KB`. `npm run size` : app `37.16 kB`, verify `86.07 kB`, css `20.64 kB`.

### Livrable 16 : `eslint.config.js`
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/eslint.config.js:47-93`
- Evidence : bannit `motion` nommé depuis `motion/react`, `<motion.*>`, `import('motion/react')`, et `ImportNamespaceSpecifier` depuis `motion/react-m`. Aucun commentaire `vendor-motion`.

### Livrable 17 : E2E motion
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/e2e/motion.spec.ts:14-51`
- Evidence : `emulateMedia({ reducedMotion: 'reduce' })`, espion `__vtCalls`, vérifie `verify-scene` visible, `steer-scene` absent, `__vtCalls === 0`, `getAnimations().running === 0`. Exécution : `1 passed`.

### Livrable 18 : setup jsdom
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/src/test/setup.ts:5-19`
- Evidence : mock `window.matchMedia`, défaut `matches: false`.

## Résumé final
- Total livrables : 18
- Confirmés : 18
- Gaps : 0
- Partiels : 0

Vérifications exécutées : `npm run lint`, `npm run gate:scan-front`, tests unitaires ciblés `7 passed / 20 passed`, `npm run size`, `npx playwright test e2e/motion.spec.ts`.