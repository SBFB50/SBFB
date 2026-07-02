# Review â€” Sprint 80 Phase H : VERIFY-plein front (front Factory Operator greenfield, `tools/factory-operator`)

**Date :** 2026-06-28
**PÃ©rimÃ¨tre :** working tree NON COMMITÃ‰ â€” front-pur (backends F `bb35d39` + G `ed00b4a` dÃ©jÃ  livrÃ©s). 13 fichiers modifiÃ©s/supprimÃ©s + 6 untracked (GatesPanel.tsx + .test, VerifyScene.test, plein/DiffViewer.tsx + .test, plein/wordDiff.ts + .test, useVerifyData.ts). 2 suppressions (`surfaces/DiffView.tsx` + `.test.tsx`).
**Orchestration :** review deep prÃ©-Codex â€” 6 dimensions â†’ 3 lentilles adversariales â†’ synthÃ¨se rÃ©conciliÃ©e sur le DIFF rÃ©el + grounding wire Rust (gates.rs:70-125, sprint_history.rs:1003-1021, process.rs:739-751).
**Fichiers revus (lus en ENTIER) :**
- Neufs : `state/useVerifyData.ts`, `components/verify/GatesPanel.tsx` (+ `.test.tsx`), `components/verify/plein/DiffViewer.tsx` (+ `.test.tsx`), `components/verify/plein/wordDiff.ts` (+ `.test.ts`), `components/verify/VerifyScene.test.tsx`
- ModifiÃ©s : `api/operator.ts`, `lib/verdict.ts` (+ `.test.ts`), `components/verify/VerifyScene.tsx` (rÃ©Ã©crit), `state/useOperator.ts` (verifyReady), `components/Rail.tsx` (ready-dot), `App.tsx`, `components/surfaces/ProcedeSurface.tsx` (migrÃ© DiffViewâ†’DiffViewer), `vite.config.ts`, `.size-limit.json`, `e2e/verify.spec.ts` (rÃ©Ã©crit)
- SupprimÃ©s : `components/surfaces/DiffView.tsx` + `.test.tsx`
- Grounding (non modifiÃ©s) : `state/useRailStatus.ts`, `lib/useTokenStream.ts`, `crates/sbfb-factory/src/gates.rs`, `sprint_history.rs`, `process.rs`
- Artefact preflight : `.planning/active/sprint80_phase_h_preflight.md` (verdict PLAN-ADAPT, 5 adaptations figÃ©es)

---

## Évaluation initiale (pré-fix, pré-Codex) : CONCERN

Correctness solide, invariant cardinal 0-verdict-UI TENU de bout en bout, 0 dep runtime nouvelle, 0 sink XSS, 5 adaptations PLAN-ADAPT honorÃ©es, dÃ©gradations V5/V6 visibles+honnÃªtes. **2 P1 bloquent le PASS-PENDING** : (1) un vrai dÃ©faut fonctionnel â€” la rÃ©fÃ©rence de fraÃ®cheur est gelÃ©e au mount â†’ l'indicateur Â« obsolÃ¨te Â» ment et son remÃ¨de est inopÃ©rant aprÃ¨s le 1er commit de session ; (2) une lacune de couverture â€” le hook net-neuf `useVerifyData.ts` n'a aucun test unitaire. ConformÃ©ment au critÃ¨re (CONCERN si 1-2 P1 non triviaux). Rien ne casse, rien ne fabrique un verdict ; les 2 P1 sont corrigeables rapidement avant le gate Codex.

**DÃ©compte findings (aprÃ¨s filtrage adversarial) :** P0 = 0 Â· P1 = 2 Â· P2 = 4 Â· P3 = 9 (+ 4 faux-positifs rÃ©futÃ©s, tracÃ©s). Aucun viol cardinal, aucune faille, aucune rÃ©gression cachÃ©e, aucun bump Day-0.

---

## RÃ©sumÃ© des 6 dimensions

| # | Dimension | RÃ©sultat |
|---|---|---|
| 1 | Correctness (diff-viewer V1/V2/V3, word-diff LCS, hook async co-fetch) | OK Ã  la lecture â€” word-diff inlineâ†”split alignÃ© sur le MÃŠME index (min/max), invariant concat=ligne tenu ; abort double-gardÃ© (0 setState-after-unmount) ; **1 dÃ©faut de logique : rÃ©fÃ©rence de fraÃ®cheur gelÃ©e (P1-1)** |
| 2 | Cardinal 0-verdict-UI + anti-PASS + miroir wire exact | OK â€” GATE_STATUS `satisfies` les 5 GateStatus (snake_case), GatesView restituÃ© 1:1 sans agrÃ©gat racine, 0 nouveau rendu de PASS (seul `=== 'PASS'` verdict.ts:116 strippÃ©), 'passed'â†’'tenue'/âœ“, scan-front non matchable |
| 3 | SÃ©curitÃ© (XSS / CSP / injection) | OK â€” 0 dangerouslySetInnerHTML/innerHTML/eval dans tout `src` (confirmÃ©), diff hostile en nÅ“uds texte React Ã©chappÃ©s, classes Tailwind littÃ©rales (pas de style-src) ; **surface d'injection de PROMPT (pas XSS) via onHunkIntent, attÃ©nuÃ©e (P3)** |
| 4 | Tests / couverture / gate de testabilitÃ© | CONCERN â€” Vitest 119/119 vert, mais **useVerifyData 0 test unitaire (P1-2)** + cluster P2 (V3 nav/minimap/split-word-diff, verifyReady+Rail-dot, getters, Promise.all) |
| 5 | ConformitÃ© PLAN-ADAPT (5 adaptations) + scope/dÃ©gradations V5/V6 | OK â€” 5 adaptations tenues, V5/V6/marqueur-par-fichier dÃ©gradÃ©s+visibles (dot neutre + label Â« dÃ©gradÃ© S81 Â»), onglets scellÃ©/Preuve disabled Â« Ã  venir S81 Â» 0-fetch |
| 6 | Patterns / budget size-limit / rÃ©gression capacitÃ© | OK â€” manualChunk diff-viewer HOISTÃ‰ (1 chunk partagÃ© VerifyScene+ProcedeSurface), bump per-entrÃ©e PLAN-ADAPT-consistant (â‰  Day-0), bi-usage V2/U7 prÃ©serve testid 'diff-view', terminal PTY toujours atteignable (e2e-prouvÃ©) |

---

## ConformitÃ© au preflight (PLAN-ADAPT) â€” 5 adaptations toutes tenues

| Adaptation figÃ©e | Ã‰tat | Ã‰vidence |
|---|---|---|
| 1. V4 restitue STRICTEMENT les 5 GateStatus (jamais PROVISIONAL/Not-evidenced/RIG-ABSENT) | CONFORME | `GATE_STATUS` `as const satisfies Record<string,GateStatus>` (verdict.ts:53-59) ; glyphes âœ“/âœ•/â€”/â€¢ ; labels tenue/bloquant/informatif/non exÃ©cutÃ©e/hors pÃ©rimÃ¨tre â€” aucun marqueur T2 fabriquÃ© |
| 2. 0 dep runtime nouvelle (contrat nullable via interface TS, pas `npm i zod`) | CONFORME | `getJson<T>` + interfaces `\| null` (operator.ts:357-396) ; package.json hors diff ; 0 import zod/jsdiff/@tanstack |
| 3. FraÃ®cheur `run@<rev>`/â—¦ obsolÃ¨te DÃ‰RIVÃ‰E front | **PARTIEL â€” rÃ©f. gelÃ©e (P1-1)** | `runRev=diff.head` (VerifyScene.tsx:163) restituÃ©, MAIS comparÃ©e Ã  `head` gelÃ© au mount (status.head) au lieu du head COURANT â†’ indicateur menteur aprÃ¨s commit |
| 4. Bascule MANUELLE, seule la DISPONIBILITÃ‰ Ã©tat-driven | CONFORME | `setMode` reste manuel (useOperator.ts:171-176) ; `verifyReady` (196) n'allume que le dot Rail (`ready && !active`, Rail.tsx:42-50) ; jamais d'auto-switch |
| 5. V5+V6+marqueur-gate-par-fichier DÃ‰GRADÃ‰S/carry S81 | CONFORME | dot neutre `bg-tx4` + label Â« marqueur de gate par fichier Â· dÃ©gradÃ© S81 Â» (DiffViewer.tsx:309-320) ; `line:null` en dur cÃ´tÃ© wire (gates.rs:101) |

**Folds livrÃ©s vÃ©rifiÃ©s :** V1 diff bi-mode inline/side-by-side + word-diff intra-ligne (mini-LCS de 2 lignes appariÃ©es, jamais un re-diff fichier â€” wordDiff.ts) ; V2/U7 bi-usage (LE MÃŠME `DiffViewer` rend working-tree ET un commit passÃ© via ProcedeSurface.tsx:153) ; V3 nav clavier j/kÂ·â†‘/â†“ + minimap densitÃ© (DiffViewer.tsx:214-231,384-402, **non testÃ©s â†’ P2**).

---

## Invariant cardinal 0-verdict-UI â€” TENU de bout en bout

- **Miroir wire EXACT** : `GateStatus` 5 variantes snake_case (gates.rs:74-88) = type TS (operator.ts:368) ; `GateIssueView{message,file:Option<String>,line:Option<u32>}` = `{message,file:string\|null,line:number\|null}` ; `GatesView{gates}` sans champ racine (gates.rs:111-121) = `{gates:GateEntryView[]}`.
- **0 agrÃ©gat / 0 % / 0 PASS calculÃ©** : GatesPanel mappe chaque entrÃ©e, keyÃ©e `(gate,status)` (entryKey, GatesPanel.tsx:21-23), un mÃªme gate (lint-planning) apparaÃ®t 2Ã— sans aplatissement ; testÃ© Â« never renders a verdict word or an aggregated score Â» (GatesPanel.test.tsx:36-41).
- **Slot Ã‰TAT = machine nommÃ©e** : `VERIFY_ETAT` 6 Ã©tats (verdict.ts:25-32), `pickVerifyEtat` lit loading/stale/hasChanges (38-43) â€” jamais Â« does it pass Â» ; test fige l'absence de PASS/VÃ©rifiÃ©/ApprouvÃ© (verdict.test.ts:46-62).
- **0 nouveau rendu de PASS** : seul littÃ©ral non-commentaire = `verdict.ts:116 === 'PASS'` (comparateur Phase D, strippÃ© par scan-front) ; la pill ProcedeSurface rend `{verdict}` (variable backend Phase D prÃ©-existante, restitution autorisÃ©e).
- **Scan anti-PASS** : dÃ©jÃ  vÃ©rifiÃ© vert (scan-front-discipline clean) ; grep manuel confirme 0 match `\b(PASS\|VÃ©rifiÃ©\|ApprouvÃ©)\b` en texte UI hors tests/commentaires.

---

## Findings (P0:0 Â· P1:2 Â· P2:4 Â· P3:9)

### P1 â€” Ã  corriger (ou acter explicitement) AVANT commit

| # | Localisation | Description | Correctif |
|---|---|---|---|
| P1-1 | `VerifyScene.tsx:95` + `App.tsx:64` + `useRailStatus.ts:40-61` | **RÃ©fÃ©rence de fraÃ®cheur GELÃ‰E (dÃ©faut de logique, pas couverture).** `stale = diff.head !== head` ; `head=status.head` issu de useRailStatus, `useEffect(...,[])` fetchÃ© UNE fois au mount, jamais re-fetchÃ©. `diff.head` se re-fetch au `reload`, `status.head` reste figÃ© â†’ aprÃ¨s le 1er commit de session (cas NORMAL de l'Operator) + un Â« relancer Â», `stale` colle Ã  true ; GatesPanel.tsx:75-79 affiche `â—¦ obsolÃ¨te Â· run@<head-courant>` (auto-contradictoire) + Ã‰TAT 'stale', et Â« relancer Â» NE PEUT PAS l'effacer (il ne re-fetch que le diff). DÃ©vie de l'adaptation prÃ©flight #3 (Â« obsolÃ¨te quand le head COURANT diverge Â»). Advisory/non-cardinal/non-crash mais ment dans le workflow normal. Format sain (diff.head et status.head = mÃªme `rev-parse --short HEAD`) â†’ le dÃ©faut est le GEL, pas le format. L'e2e ne l'attrape pas (aucun commit pendant le test). | DÃ©river la rÃ©fÃ©rence LIVE d'une source re-fetchÃ©e avec le diff : re-fetch `/api/context` (ou un head endpoint) DANS useVerifyData/au reload et comparer `diff.head` au head LIVE. Repli minimal : retirer le badge Â« obsolÃ¨te Â» + l'Ã©tat 'stale' tant qu'aucune source live n'existe (ne pas livrer un indicateur menteur). Ã€ dÃ©faut : DÃ‰SACTIVER l'indicateur + acter le dÃ©faut/correctif comme carry S81 dans le body. |
| P1-2 | `useVerifyData.ts:38-64` + `VerifyScene.test.tsx:8` | **Hook net-neuf SANS test unitaire (lacune de couverture).** Co-fetch Promise.all (40), mapping OperatorErrorâ†’`VERIFY indisponible (${status})` vs gÃ©nÃ©rique (49), abort double-gardÃ© (.then 42 / .catch 45 / cleanup 52), reload tick (60-63) : rien en unitÃ© (VerifyScene.test mocke le hook ; seul filet = happy-path T1 e2e). Logique correcte-Ã -la-lecture (0 setState-after-unmount, gardes confirmÃ©es) â†’ trou de couverture, pas bug. Contraire Ã  la rigueur per-phase uniforme (directive PO). | Ajouter `useVerifyData.test.ts` : error-mapping (OperatorError vs gÃ©nÃ©rique), abort-clean (aucun setResolved aprÃ¨s abort), reload(tick). OU documenter EXPLICITEMENT le report e2e-only + carry dans le body. |

### P2 â€” documentables (corrigeables rapidement)

- **P2-a** `DiffViewer.tsx:214-231,251-255,384-402` â€” Fold V3 (nav clavier j/kÂ·â†‘/â†“ + minimap densitÃ©), banniÃ¨re `â—¦ tronquÃ©`, et word-diff EN MODE SPLIT non testÃ©s (`DiffViewer.test.tsx`, 7 tests, 0 ArrowDown/j/k/minimap/truncated ; le test split 49-55 n'assert que la duplication du ctx). Fold V3 revendiquÃ© livrÃ© â†’ non vÃ©rifiÃ© par la gate de testabilitÃ©.
- **P2-b** `useOperator.ts:196` + `Rail.tsx:42-50` â€” `verifyReady` jamais assertÃ© (useOperator.test.ts, grep=0) ; `Rail.test.tsx` ABSENT â†’ le dot `ready && !active` (anti-auto-switch D6, cÅ“ur de l'adaptation #4) non testÃ©. Ã€ couvrir : transitions verifyReady + dot prÃ©sent/absent selon (verifyReady, mode actif).
- **P2-c** `useVerifyData.ts:40` â€” Couplage co-fetch fragile : un 500 sur `/api/gates` SEUL rejette TOUT le hook â†’ VerifyScene.tsx:129-130 masque le working-tree diff pourtant disponible (surface centrale/par dÃ©faut). `Promise.allSettled` / dÃ©gradation indÃ©pendante prÃ©fÃ©rable.
- **P2-d** `operator.ts:399-406` vs `operator.test.ts` â€” `getWorkingTreeDiff`/`getGates` non testÃ©s alors que les autres getters le sont individuellement (incohÃ©rence de pattern + assertion de forme `{head,unstaged,staged,truncated}` / dÃ©paquetage `gatesView.gates` absente).

### P3 â€” documentÃ©s

- **P3-a** `GatesPanel.test.tsx:11-20` â€” Glyphe âœ“/'passed' non assertÃ© au niveau PANNEAU (fixture sans entrÃ©e 'passed'). NUANCE dÃ©gradante : le mapping 'passed'â†’'âœ“' EST unit-testÃ© (verdict.test.ts:75-81) â†’ gap strictement panneau-intÃ©gration. Ajouter `{status:'passed',issues:[]}` + `getAllByText('âœ“')`.
- **P3-b** `VerifyScene.tsx:96` â€” Erreur mappÃ©e sur l'Ã©tat 'awaiting' (Â« En attente de session agent Â») ne dÃ©crit pas un Ã©chec de lecture ; cardinal tenu, clartÃ© seulement. Optionnel : Ã©tat `unavailable`.
- **P3-c** `useOperator.ts:196` + `useTokenStream.ts:29` â€” `verifyReady` exclut 'error'/'aborted' (terminaux) ; l'indice visuel manque sur un tour terminal en erreur, mais useVerifyData fetch indÃ©pendamment â†’ bascule manuelle reste possible. Choix dÃ©fendable Ã  acter.
- **P3-d** `e2e/verify.spec.ts:31-36` â€” Garde-fou de non-rÃ©gression XSS (prÃ©flight risk #6 : injecter `<script>`/onerror + assert textContent littÃ©ral) NON cÃ¢blÃ©. Protection structurelle rÃ©elle (0 dangerouslySetInnerHTML, nÅ“uds texte) â†’ 0 faille, mais un futur refactor rÃ©introduisant un sink ne serait pas attrapÃ©.
- **P3-e** (sÃ©curitÃ©) `VerifyScene.tsx:98` â€” Surface d'INJECTION DE PROMPT (pas XSS) via onHunkIntent (`Examiner et corriger ${file} â€” hunk ${hunkHeader}`, contenu diff hostile routÃ© Ã  une session LLM privilÃ©giÃ©e). AttÃ©nuÃ© : dÃ©clenchement MANUEL + gate SENSITIVE_ACTIONS + corps JSON. Carry S81.
- **P3-f** `VerifyScene.tsx:94` â€” staged/unstaged aplaties (`[...unstaged, ...staged]`) : fichier partiellement indexÃ© rendu 2Ã— sans marqueur indexÃ©/non-indexÃ© (compteurs prÃ©servÃ©s au caption, keys fidx distinctes â†’ 0 collision React).
- **P3-g** `.size-limit.json` â€” Bumps Ã  chiffrer au commit body (verify-surface 92â†’96, css 21â†’25, diff-viewer 22 neuf). Hoisting VÃ‰RIFIÃ‰ (vite.config.ts:76 manualChunk `/src/components/verify/plein/`, 1 chunk partagÃ©). PLAN-ADAPT-consistant, budgets per-entrÃ©e â‰  Day-0. Confirmer `npm run size` (pas de duplication).
- **P3-h** `VerifyScene.tsx:113` â€” ContinuitÃ© de session terminal : toggle diffâ†’terminalâ†’diff dÃ©monte `<Terminal/>` + reset `started` â†’ rÃ©affiche Â« DÃ©marrer la session Â» ; `resume` supportÃ© cÃ´tÃ© wire mais pas rÃ©-attachÃ© au remount. Non-bloquant.
- **P3-i** `DiffViewer.tsx:200,204-205` â€” refs current/file/hunk non purgÃ©s au reload, mais le reload passe par loading=true qui dÃ©monte DiffViewer (refs recrÃ©Ã©s) â†’ scÃ©nario quasi-inatteignable, impact ~nul.

### Faux-positifs RÃ‰FUTÃ‰S (tracÃ©s pour honnÃªtetÃ©)

- **DÃ©salignement word-diff inlineâ†”split sur runs inÃ©gaux** : applyWordDiff apparie del[p]â†”add[p] sur `min(dels,adds)` (DiffViewer.tsx:71-76), splitRows zippe sur `max()` avec le MÃŠME index (124) â†’ alignement cohÃ©rent, lignes excÃ©dentaires `.words=undefined` rendues plain, invariant concat=ligne tenu (wordDiff.test.ts:14-15). PAS de bug (seulement non testÃ© = P2-a).
- **setState-after-unmount dans useVerifyData** : cleanup `controller.abort()` (52) neutralise `.then` (`!signal.aborted` 42) ET `.catch` (`if aborted return` 45) avant tout setResolved. SÃ»r en StrictMode double-invoke. PAS de fuite.
- **Â« Phase H introduit un nouveau rendu de PASS Â»** : seul littÃ©ral non-commentaire = `verdict.ts:116 === 'PASS'` (comparateur strippÃ©) ; pill ProcedeSurface rend `{verdict}` (variable backend Phase D). 0 nouveau rendu de verdict.
- **Â« stale toujours vrai par format mismatch Â»** : diff.head (sprint_history.rs:1012) et status.head (process.rs:204) = mÃªme `rev-parse --short HEAD`. Format sain ; le vrai dÃ©faut est le GEL de la rÃ©fÃ©rence (P1-1), pas le format.

---

## Actions avant commit

1. **Corriger P1-1** (rÃ©fÃ©rence de fraÃ®cheur live, ou dÃ©sactiver+acter l'indicateur) et **P1-2** (useVerifyData.test.ts, ou report e2e documentÃ©). Re-lancer Vitest + e2e aprÃ¨s fix.
2. **BLOQUANT â€” Gate Codex** (`codex exec`, gpt-5.5, raw dans `sprint80_phase_h_codex_review.md`) : boucler jusqu'Ã  CLEAN ou P2/P3 documentÃ©s, puis promouvoir reviewâ†’PASS.
3. **Discipline commit** : 1 commit `feat(factory-operator): Sprint 80 Phase H â€” <titre>`, body 9 sections, delta tests cumulÃ© (+Vitest, suppression DiffView), **chiffrer les bumps size-limit (P3-g)**, scope cuts respectÃ©s (V5/V6/per-fichierâ†’carry S81, scellÃ©/Preuveâ†’S81).
4. **VÃ©rifications lourdes** : Vitest + build tsc/vite + lint + gates discipline + size-limit + T1 e2e BLOQUANT-vert au wrap-up (gate de testabilitÃ© par-sprint) ; dual-platform Docker AVANT push.

## Évaluation initiale (pré-fix, pré-Codex) : CONCERN

---

## RÃ©solution des findings (re-review)

Re-review adversariale du diff working-tree `tools/factory-operator` (lecture intÃ©grale, faits re-dÃ©rivÃ©s du code). Verdict rÃ©visÃ© : CONCERN -> **PASS-PENDING**. Tous les P1 RESOLVED, 0 nouvelle rÃ©gression P0/P1.

### P1-1 â€” RÃ©fÃ©rence de fraÃ®cheur GELÃ‰E (l'indicateur Â« obsolÃ¨te Â» mentait) -> RESOLVED par retrait honnÃªte
Le correctif supprime l'indicateur menteur plutÃ´t que de le rafraÃ®chir Ã  moitiÃ©.
- **Ã‰tat `stale` supprimÃ©** : `VERIFY_ETAT` n'a plus que `awaiting|bootstrap|reading|inspecting|empty|unavailable` â€” aucune clÃ© `stale` (`src/lib/verdict.ts:32-39`).
- **Signatures resserrÃ©es** : `pickVerifyEtat({ loading, hasChanges })` ne prend plus `stale` (`src/lib/verdict.ts:45`) ; `VerifyScene({ op })` ne prend plus de prop `head` (`src/components/verify/VerifyScene.tsx:90`) ; `App.tsx:64` rend `<VerifyScene op={op} />` sans `head` ; `GatesPanel` ne reÃ§oit plus `stale` ni badge `â—¦ obsolÃ¨te` (props = `gates/loading/error/runRev/onReload`, `src/components/verify/GatesPanel.tsx:38-50`).
- **`run@<rev>` prÃ©servÃ© ET corrigÃ©** : `runRev={diff?.head ?? null}` (`VerifyScene.tsx:167`) â€” la fraÃ®cheur restitue dÃ©sormais le `head` co-rÃ©cupÃ©rÃ© dans le MÃŠME cycle par `useVerifyData`/`getWorkingTreeDiff`, plus le `status.head` du rail figÃ© au mount. Rendu : `run@{runRev}` + bouton `relancer` uniquement (`GatesPanel.tsx:72-86`).
- **Carry S81 documentÃ©** : `src/lib/verdict.ts:24-30` explicite que le seul head live (`/api/context`) est fetchÃ© une fois au mount et mentirait aprÃ¨s le 1er commit -> divergence de fraÃ®cheur honnÃªte = carry S81 (head re-pollÃ© ou rev sur `/api/gates`).
- **Aucun prop/type cassÃ©** : `etat = diffError ? 'unavailable' : pickVerifyEtat(...)`, toutes les clÃ©s existent dans `VERIFY_ETAT` ; `GateFlip value={VERIFY_ETAT[etat]}` typÃ©.
- **VerrouillÃ© par test** : `GatesPanel.test.tsx:54-60` assert `run@d59ee32` prÃ©sent ET `not.toMatch(/obsolÃ¨te/)`.

### P1-2 â€” `useVerifyData.ts` net-neuf sans test -> RESOLVED (couverture sÃ©mantique)
`src/state/useVerifyData.test.ts` couvre rÃ©ellement, pas des stubs :
- happy-path co-fetch diff+gates (`:35-43`),
- dÃ©gradation INDÃ‰PENDANTE : gates 500 -> diff toujours prÃ©sent (`:45-53`),
- les DEUX branches de mapping : `OperatorError` -> Â« VERIFY indisponible (500) Â» (`:45-53`) et `Error` gÃ©nÃ©rique -> Â« VERIFY indisponible Â» (`:55-61`),
- reload -> re-fetch x2 des deux (`:63-71`).

### P2-c â€” `Promise.all` rejetait tout si `/api/gates` Ã©chouait -> RESOLVED
- `Promise.allSettled([getWorkingTreeDiff, getGates])` avec `diffError`/`gatesError` sÃ©parÃ©s (`src/state/useVerifyData.ts:55-65`).
- Garde abort **avant tout setState** : `if (signal.aborted) return` (`:59`). Re-dÃ©rivÃ© : `allSettled` ne rejette jamais ; sur unmount les deux fetchs rejettent en AbortError mais la garde court-circuite l'Ã©criture -> un abort ne produit JAMAIS d'error string, quelle que soit la raison de rejet.
- Routage vÃ©rifiÃ© : `VerifyScene` route `diffError` -> zone diff + Ã©tat `unavailable` (`VerifyScene.tsx:98,131-134`) et `gatesError` -> `GatesPanel` (`:166-167`). ProuvÃ© par Â« keeps the diff visible when only the gates fail Â» (`VerifyScene.test.tsx:90-97`).

### P2-a/b/d + P3-a/d â€” Couverture -> RESOLVED
- `DiffViewer.test.tsx` : mouvement rÃ©el d'`aria-current` hunk[0]->hunk[1] sur ArrowDown (`:94-101`, impl. `current`+`moveHunk` DiffViewer.tsx:214-231,351), minimap 1 bouton/fichier (`:87-92`), tronquÃ© `â—¦ tronquÃ©` (`:82-85`), word-diff conservÃ© en split (`:103-109`), **garde XSS rÃ©elle** : contenu en text-node React `{line.line.content}` (DiffViewer.tsx:145), `container.querySelector('script')` null (`:111-129`).
- `Rail.test.tsx` : ready-dot D6 visible (steer+ready), cachÃ© si VERIFY actif ou non-ready (`:17-37`).
- `operator.test.ts` : `getWorkingTreeDiff` (`:81-84`) + `getGates` + propagation `AbortSignal` (`:86-95`).
- `useOperator.test.ts` : `verifyReady` = false (no turn / mid-stream / abort) et true sur `done` (`:145-164`), alignÃ© sur `useOperator.ts:196` (`message!==null && (done||ended)`).
- `GatesPanel.test.tsx` : glyphe `âœ“` + keyed `(gate,status)` (lint-planning x2 jamais aplati) + `run@rev` sans `/obsolÃ¨te/` (`:26-60`).

### Cardinal Â« 0 verdict calculÃ© UI Â» â€” tenu
Scan anti-PASS/agrÃ©gation sur tout le front modifiÃ© : aucun `overall`/`all_passed`/`%`/`dangerouslySetInnerHTML` dans le rendu ; le seul `=== 'PASS'` (`verdict.ts:122`) est le littÃ©ral de restitution autorisÃ© et documentÃ© (gate scan-front le strip). `GatesPanel` restitue chaque entrÃ©e distincte sans agrÃ©gat (test Â« never renders a verdict word or an aggregated score Â»).

### RÃ©gressions introduites â€” aucune (P0/P1)
Migration `DiffView` -> `DiffViewer` propre : `ProcedeSurface.tsx:16,153-157` importe `DiffViewer` avec `testid="diff-view"`, couvert par le test bi-usage V2/U7 (`DiffViewer.test.tsx:71-75`) ; `DiffView.tsx`/`.test.tsx` supprimÃ©s sans import pendant (seul un commentaire `ProcedeSurface.tsx:11` y rÃ©fÃ¨re encore). Le manualChunk `vite.config.ts` (`/src/components/verify/plein/` -> chunk `diff-viewer`) isole correctement le viewer partagÃ© hors du hero `verify-surface` ; `.size-limit.json` ajoute la cible `diff-viewer` (22 KB) et bumpe verify-surface 92->96 KB et css 21->25 KB (verts en amont).

### RÃ©sidus (P3, non bloquants)
1. `GatesPanel.tsx:16` â€” commentaire d'en-tÃªte encore stale (Â« â—¦ obsolÃ¨te is a comparison of two restituted revs Â») alors que le badge a Ã©tÃ© retirÃ© : Ã  aligner sur le carry S81 (verdict.ts:24-30).
2. `useRailStatus.ts:48` â€” `RailStatus.head` dÃ©sormais set-only (peuplÃ©, plus aucun lecteur) : code mort, retirer ou annoter pour S81.
3. `ProcedeSurface.tsx:11` â€” commentaire Â« fold J11, DiffView Â» (composant renommÃ© DiffViewer). Trivial.
4. `vite.config.ts` â€” commentaire Â« under its 92 KB budget Â» vs limite bumpÃ©e 96 KB. Trivial.
5. `useVerifyData.test.ts` â€” chemin d'abort mid-fetch non exercÃ© (garde correcte par lecture ; coverage-only).

---

## Nettoyage des résidus P3 (post-re-review)

Les 4 premiers P3 résiduels listés ci-dessus ont été CORRIGÉS dans une passe de propreté (honnêteté de commentaire, doctrine projet) ; le 5e est documenté :
1. `GatesPanel.tsx:16` — commentaire d'en-tête réécrit (run@rev seul, pas de badge obsolète, carry S81). CORRIGÉ.
2. `useRailStatus.ts` — `RailStatus.head` (set-only) RETIRÉ de l'interface + EMPTY + setter (0 lecteur ; `VerifyScene` lit `diff.head`, `ProcedeSurface` lit `history.head`). CORRIGÉ.
3. `ProcedeSurface.tsx:11` — commentaire « DiffView » → « DiffViewer (Phase H, fold V2/U7) ». CORRIGÉ.
4. `vite.config.ts` — commentaire « 92 KB budget » → « bumped to 96 KB (review P3-g) ». CORRIGÉ.
5. `useVerifyData.test.ts` — chemin d'abort mid-fetch : DOCUMENTÉ (garde `if (signal.aborted) return` identique au pattern prouvé `useCommitDiff`/`useRailStatus` ; difficile à asserter en React 19 qui ne warn plus sur setState-after-unmount). Couverture par lecture + happy/error/reload.

Re-vérif après nettoyage : Vitest 137/137, lint clean, scan-front clean, build OK.

## Codex reconciliation

Gate croisée Codex GPT-5.5 (rapport brut : `sprint80_phase_h_codex_review.md`). **11 livrables : 10 CONFIRMÉ · 0 GAP · 1 PARTIEL · 0 P0/P1.** Codex a relancé `npm run test:unit` (137/137), `npm run build`, `npm run size`, `gate:scan-front` (clean). Cardinal, anti-PASS, 0-dep-runtime, XSS, fraîcheur-honnête, bascule-manuelle : tous **CONFIRMÉ** (codex_review:132-140). Disposition des 3 findings (tous P2/P3, aucun bloquant) :

- **PARTIEL L11 + P2 (« 7/7 » vs 2)** — NON-ISSUE : Codex a lancé `playwright test e2e/verify.spec.ts` seul (= 2 tests verify.spec). Le « 7/7 » désigne la suite e2e COMPLÈTE (`npm run test:e2e` : boot 2 + motion 1 + steer 2 + verify 2 = **7 tests**, verte) ; verify.spec en contribue 2. Clarté de doc, pas un gap code (codex_review:130).
- **P3 — GateFlip.tsx:41 style inline** — NON-ISSUE : `GateFlip` est **Phase E** (pré-existant, NON touché Phase H) ; son style statique (`display/transformOrigin/transformStyle`) est CSP-safe (Phase E shippe déjà sous `default-src 'self'`). Mon prompt sur-affirmait « seul style inline = minimap » ; en Phase H le seul style inline NEUF est le `flexGrow` numérique de la minimap. Aucun nouveau sink (codex_review:133).
- **P3 — App.tsx:4 commentaire** — **CORRIGÉ** : en-tête réécrit au passé immuable (« Phase D added … Phase H made VERIFY the full diff-viewer ») + mention `verifyReady` (App.tsx:3-11) (codex_review:139).

0 GAP P0/P1 → review promue **PASS**.

---

## Verdict: PASS
