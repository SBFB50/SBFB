# Sprint 80 — Phase C — Review (avant commit)

**Phase** : C — STEER complet (atelier observable + composeur en dock) + rail d'orientation altitude-0 + primitive SSE `useTokenStream`.
**Date** : 2026-06-27.
**Agent** : `nexus-phase-review-deep` (6 dimensions + passe adversariale).
**Préflight** : `sprint80_phase_c_preflight.md` — verdict **PLAN-ADAPT** (9 plan-adaptations, 0 Day-0 touchée).
**Nature du diff** : **front-pur** — 0 fichier `.rs`, 0 route daemon, 0 wire, 0 dépendance front nouvelle.

---

## 1. Scope revu

Diff NON commité Phase C : modifiés `tools/factory-operator/{package.json,package-lock.json,.gitignore,src/App.tsx,src/lib/cn.ts}` ; neufs code `src/api/operator.ts`, `src/lib/{sseFrames,streamChunk,useTokenStream}.ts`, `src/data/intentions.ts`, `src/state/{useRailStatus,useOperator}.ts`, `src/components/{OrientationBar,Rail}.tsx`, `src/components/steer/{Composer,Atelier,Mur,TechDetails,SteerScene}.tsx`, `src/components/verify/VerifyPlaceholder.tsx` ; neufs tests `operator.test.ts`, `{sseFrames,streamChunk,useTokenStream}.test.ts`, `useOperator.test.ts`, `steer/{Composer,Mur,TechDetails}.test.tsx`, `e2e/steer.spec.ts`. Le client consomme uniquement des routes **existantes** de la crate `sbfb-factory` (status/context/providers/prompt/chat·session/send/stream). Frozen « Factory hors daemon » tenu.

---

## 2. Résumé des 6 dimensions (après filtrage adversarial)

| # | Dimension | Verdict | Findings retenus |
|---|---|---|---|
| 1 | Correction du transport SSE (`useTokenStream`/`sseFrames`/`streamChunk`) | **CONCERN** | 1 P1 (EOF-sans-terminal, requalifié depuis P2), 2 P3 (flush TextDecoder, branche catch-AbortError non assertée) |
| 2 | Fidélité aux 9 plan-adaptations du preflight | **PASS** | 0 finding — 9/9 implémentées en code (pas en commentaire), corroborées backend + tests |
| 3 | Invariants & doctrine Day-0 | **PASS** | 1 P3 hors-doctrine (getStatus/review_verdict shape non rendue) |
| 4 | Qualité des tests (sémantique) | **CONCERN** | 3 P2 (relaunch/interrupt/launchError ; error-chunk 1er terminal ; newSession code mort), 2 P3 (Atelier sans test ; coverage.all absent) |
| 5 | Sécurité | **PASS** | 2 P3 (sse_gate format! brut backend hors-diff ; nit libellé « token ✓ ») |
| 6 | Scope cuts honnêteté + qualité React/TS | **PASS** | 1 P1 (commentaire GATE_MESSAGE surclame), 2 P3 (EOF cf. dim.1 ; champs wire non consommés) |

**Cœur load-bearing CORRECT et bien testé** : latch du 1er terminal (`useTokenStream.ts:147-155,89-91`), supersede par runId anti double-open StrictMode (`:109-118,143,169,201-208`), abort honnête `reader.cancel()` depuis le listener + discrimination AbortError → `'aborted'` jamais `'error'` avec sortie partielle préservée (`:122-126,167,170-174`), parseur SSE robuste (carry ligne+frame, CRLF, lignes-commentaire `:`, flush end(), retrait d'un espace après `data:` — `sseFrames.ts:20-71`, 7 tests), `parseChunk` null-safe (`streamChunk.ts:35-68`), union 6-valeurs codée en dur avec `requires_gate` forgé (`streamChunk.ts:12-18`). MUR = barrière à contrôle unique « Retour », ZÉRO Forcer/Override/Bypass (`Mur.tsx`, `Mur.test.tsx:16-24`). Le front ne pré-filtre jamais la sensibilité : il restitue `requires_gate` du backend (`useOperator.ts:102-107`), preuve E2E plein-stack du 0-spawn (`e2e/steer.spec.ts:35-57`).

---

## 3. Passe adversariale — 2 risques structurels RÉFUTÉS

- **Race supersede/runId (double-open) — RÉFUTÉ.** `start()` bumpe `runIdRef` AVANT d'abort la boucle précédente (`useTokenStream.ts:112-115`) ; les dispatches d'une boucle périmée sont gardés par `isCurrent()` (`:143,167,169`) et le `finally` ne nullifie le ref que sous garde d'égalité `controllerRef.current===controller` (`:182`). Modèle mono-thread JS garantit l'installation synchrone du nouveau ref avant toute reprise post-await. Test `supersedes an in-flight stream` (`useTokenStream.test.ts:108-128`) : un seul Done='second', `call===2`.
- **Bypass MUR côté front / violation 0-spawn — RÉFUTÉ.** `launch()` POST toujours `/send` et n'ouvre le stream que si `!requires_gate` (`useOperator.ts:102-108`) ; `relaunch()` (seul open sans /send) est injoignable en état gated (SteerScene rend la branche MUR sans Atelier, `SteerScene.tsx:29-49`) ; le handler SSE re-applique le gate SENSITIVE_ACTIONS en défense-en-profondeur (`operator_server.rs:1116-1130`). E2E prouve `streamOpened===false` + atelier `count 0` sur intention sensible.

---

## 4. Vérification (suites) — re-confirmée par lecture des artefacts, non relancée

| Contrôle | Statut |
|---|---|
| `npm run lint` | **0 erreur** |
| `tsc --noEmit` (strict, verbatimModuleSyntax, erasableSyntaxOnly) | **0 erreur** |
| `vitest` unit | **45/45 vert** (non flaky) |
| `npm run build` | **OK** |
| `size-limit` | app **33,73/40 KB** · vendor **189,78/210 KB** · css **17,69/20 KB** (drop tailwind-merge = root-cause vérifié : 0 composant n'accepte de `className` externe, 0 `cn()` conflictuel → clsx seul suffit ; −27 KB) |
| 3 gates discipline (dont `scan-front-discipline.sh` : interdit `\b(PASS|Vérifié|Verifie|Approuvé|Approuve)\b`) | **3/3 vert** |
| E2E Playwright hermétique (`boot` CSP-clean + `steer` composeur→session + MUR sans spawn) | **4/4 vert** |

Note honnêteté couverture : `vitest.config.ts` n'a pas `coverage.all:true` → 6 fichiers source sans test (Atelier, SteerScene, Rail, OrientationBar, VerifyPlaceholder, `useRailStatus`) sont absents du rapport ; le « All files 88,5% » surévalue la couverture réelle (cf. P3).

---

## 5. Les 9 plan-adaptations du preflight — TOUTES confirmées en code

1. **S5 relance = nouveau tour plein-coût, jamais idempotent** — `relaunch()` fait `streamReset()`+`streamStart()` (re-spawn `target.run` + append assistant, backend `operator_server.rs:1156-1170`) ; UI honnête (`Atelier.tsx:101` title « nouveau tour assistant à coût d'inférence plein ») ; commentaire `useOperator.ts:14-17`.
2. **S1 intentions = asset BUILD-TIME** importé statiquement (`data/intentions.ts` ↔ `Composer.tsx:12`) ; 0 fetch `.planning`/`artifacts/draft`/`intentions.json` (grep vide).
3. **Union 6-valeurs codée en dur avec `requires_gate`** (hors enum 5-variantes) — `streamChunk.ts:12-18`, commentaire citant `llm_bridge.rs:42-59` + forge `operator_server.rs:1063`.
4. **Abort honnête S6a** — `reader?.cancel()` depuis le listener `abort` (`useTokenStream.ts:122-126,136`) + discrimination AbortError (`:170-174`) ; reducer `'aborted'` seulement depuis `'streaming'` (`:84-86`).
5. **Open idempotent + supersede via runId** — anti double-open StrictMode (`:109-118,143,169,193-208`).
6. **Auth cookie HttpOnly automatique** — `credentials:'same-origin'` aux 3 sites fetch (`operator.ts:27,38`, `useTokenStream.ts:130`), 0 header token JS, jamais `'omit'` (test adversarial `operator.test.ts:33-41`).
7. **Rail pouls-gates = placeholder non câblé** — `OrientationBar.tsx:66-69` « gates — » title « câblage Phase G » ; `useRailStatus.ts:6-9` ne fetch PAS `/api/gates` ; 0 verdict UI.
8. **Bras Network : result sur Done + post-terminal ignoré** — `Atelier.tsx:43` `body = status==='done' ? result ?? text : text` ; double latch (`useTokenStream.ts:147-155,89-91`).
9. **Refs de ligne MUR à jour** — `Mur.tsx:6` cite `operator_server.rs:37` (SENSITIVE_ACTIONS=[shell,commit,push,PASS], vérifié exact) ; `streamChunk.ts:7` cite `:1063` (sse_gate) ; 0 réf périmée `:35`/`:766-779` (grep vide).

---

## 6. Scope cuts — honnêtes et exacts (vérifiés contre `sprint80_plan.md`)

- MUR plein-largeur « Préparer le pack » → **Phase D** (`Mur.tsx:10-11` ↔ plan:123-124).
- Surfaces secondaires Terminal/Sessions/Historique/Knowledge → **Phase D** (`Rail.tsx:6-8` ↔ plan:144-145).
- VERIFY plein (diff-viewer/gates/preview) → **Phase H** (`VerifyPlaceholder.tsx:3-8` ↔ plan:202).
- Pouls gates réel → **Phase G** (`OrientationBar.tsx:66-69`, `VerifyPlaceholder.tsx:38` ↔ plan:182).
- Intentions plus riches → **S81** (`intentions.ts:10`).

Aucun commentaire ne promet une phase de façon mensongère. 0 verdict calculé UI. MUR zéro-bypass. Front qui ne pré-filtre jamais.

---

## 7. Findings retenus (sévérité jugée par la review)

### P0 — aucun.

### P1 — à corriger AVANT le commit atomique (non bloquant pour le verdict ; triviaux)

- **EOF-sans-terminal laisse `status='streaming'` à vie + bouton « Interrompre » mort.** `useTokenStream.ts:147-167` : si `reader.read()→done:true` avec `latched=false` ET `signal.aborted=false`, le bloc `:167` (gardé par `signal.aborted`) ne tire pas, le catch ne tire pas, AUCUN dispatch → le reducer reste `'streaming'` indéfiniment (aucun timeout). Aggravant : le `finally` (`:182`) nullifie `controllerRef` → `interrupt()→stop()→controllerRef.current?.abort()` (`:190`) devient un no-op, et `Atelier.tsx:41` (`streaming = status==='streaming'`) garde le bouton « Interrompre l'écoute » rendu et cliquable mais inerte. Voie **reachable** : bras Claude sortant `exit 0` sans ligne ndjson `result` → seulement `Debug{exit:0}`, aucun `Done`/`Error` (`llm_bridge.rs:317-328`). Violation de « ne ment jamais » (spinner vivant sur flux mort, contrôle d'arrêt mensonger). Non couvert par test. **Fix ~2 lignes** : après les deux passes de drain, `if (!latched && !signal.aborted && isCurrent()) dispatch({ kind terminal honnête })` (ex. statut dédié `'closed'` ou `'error'`/message « flux terminé sans résultat ») + ajouter un test `closedBody`-sans-terminal. *(Requalifié P1 depuis le P2/P3 des dimensions : invariant nommé « ne ment jamais » + contrôle mort + fix trivial → fix-avant-commit attendu sous la doctrine rigueur PO.)*
- **Commentaire GATE_MESSAGE surclame « reads identically ».** `useOperator.ts:26-30` prétend que le message du gate « reads identically whether it surfaced from /send or the SSE stream » — faux : `GATE_MESSAGE` est une réécriture **FR**, tandis que le chemin SSE remonte `stream.gate` = texte backend **EN** brut (`gate = sendGate ?? stream.gate`, `:146` ; backend `operator_server.rs:923,1127`). Le chemin /send ne porte d'ailleurs aucun corps de message. **Reword** en « restitution FR de la barrière backend ; le chemin SSE remonte le texte backend brut ». Commentaire seul, chemin divergent quasi-inatteignable (le /send gate d'abord), mais doctrine « un commentaire ne ment pas ».

### P2 — fermer ou carry EXPLICITEMENT avant commit (compléments de test / dead code)

- **`relaunch()` [plan-adaptation #1 NOMMÉE], `interrupt()` et le branch `launchError` sans aucun test** alors que câblés UI (`SteerScene.tsx:76` → Atelier). `useOperator` à 52,94% branch / 57,14% funcs (lignes 119-124 relaunch, 127 interrupt, 109-110 launchError non couvertes). `useOperator.ts:119-124,126-128,109-110`.
- **Branche `applyChunk` case `'error'` (1er terminal en plein flux) jamais exercée** : le seul frame error des tests est placé APRÈS un Done (ignoré par le latch) ; `useTokenStream.ts:71-72` non couvert. `useTokenStream.test.ts:71-85`.
- **`newSession` exporté mais câblé dans aucun composant (code mort) + non testé** : wire-or-remove. Idem `getStatus()`/`PhaseStatus.review_verdict` défini+testé mais rendu nulle part (surface API en avance). `useOperator.ts:137-144` ; `operator.ts:48-67,107-114`.

### P3 — documenter dans le carry

- **Error en plein flux APRÈS deltas partiels masque le message d'erreur dans l'Atelier** : `Atelier.tsx:43` donne priorité à `body` (texte partiel) sur le span d'erreur (`:73-81`) ; seul le libellé de statut « interrompu — erreur de flux » signale l'erreur (path `provider_router.rs:192-200`). Constat genuinement manqué par les 6 dimensions, minor.
- **Flush final TextDecoder absent** (`decode()` no-arg avant `decoder.end()`) — `useTokenStream.ts:150` ; impact nul (terminateurs ASCII), footgun nommé par le preflight non codé.
- **Branche catch-AbortError non assertée directement** — `useTokenStream.ts:170` ; comportement vérifié via la seule voie post-loop.
- **`coverage.all:true` absent** (`vitest.config.ts`) masque 6 fichiers sans test (Atelier/SteerScene/Rail/OrientationBar/VerifyPlaceholder/`useRailStatus`) derrière un 88,5% optimiste ; activer pour rendre la dette visible.
- **Champs wire typés/parsés mais non consommés** (donnée morte orientée-futur) : `SendResult.provider` (`operator.ts:94`), `done.cost_usd`/`done.duration_ms` (`streamChunk.ts:15,54-55`).
- **(Backend, hors-diff, traçabilité)** `sse_gate` forge le frame `requires_gate` par `format!` brut sans échappement de `msg` — non exploitable (msg constant) + front robuste (`parseChunk` try/catch → null) ; à durcir (`serde_json::to_string`) si le message gate devient dynamique. `operator_server.rs:1063-1064`.

---

## 8. Verdict + justification

**PASS-PENDING.** Aucun P0 ; aucun P1 « non corrigé bloquant ». Le cœur du transport SSE, les 9 plan-adaptations du preflight, les 6 invariants Day-0 (MUR sans bypass, intentions-pas-jargon, 0 verdict UI, CSP self-origin, 0 `.rs`/route/wire, fetch+ReadableStream+AbortController jamais EventSource) et la sécurité (cookie HttpOnly seul vecteur, 0 token JS, 0 XSS, parseur défensif) sont tenus et corroborés ligne par ligne contre le backend réel et la suite de tests. Les 2 risques structurels (race supersede, bypass MUR front) sont réfutés. Vérification déjà passée (lint 0, tsc 0, vitest 45/45, build, 3 gates discipline, E2E 4/4) cohérente avec le code lu — aucun gap identifié ne tombe sur un chemin testé.

Restent **2 P1 triviaux à appliquer avant le commit atomique** (garde terminale sur EOF-sans-terminal ~2 lignes + son test ; reword du commentaire GATE_MESSAGE), **3 P2** de couverture/dead-code à fermer ou carry explicitement (relaunch/interrupt/launchError ; error-chunk 1er terminal ; newSession wire-or-remove), et **des P3 documentés**. Codex (gate BLOQUANTE review→commit) n'a pas encore tourné → ce verdict n'est PAS un feu vert committable, mais la review est satisfaisante sous réserve des 2 P1.

**Recommandation review→commit** : appliquer les 2 P1, fermer/carry les 3 P2, acter les P3 dans le carry, puis enchaîner la boucle Codex (arrêt à « CLEAN ou P2/P3 documentés », jugement de sévérité maison + batch des fixes).

---

## 9. Corrections appliquées (review → pré-Codex)

Toutes appliquées et re-vérifiées (lint 0 · tsc 0 · **Vitest 52/52** [+7] · build · size app **34.52/40 KB** · 3 gates · **E2E 4/4**) :

**P1 (2/2 corrigés)**
- **EOF-sans-terminal** — `useTokenStream.ts` : nouveau statut terminal honnête `ended` ; après les deux passes de drain, `dispatch({kind: signal.aborted ? 'aborted' : 'ended'})` sur `!latched && isCurrent()`. Le flux clos sans Done/Error/gate ne reste plus bloqué à `streaming` ; l'Atelier rend « flux clos — aucun résultat final » et propose Relancer (plus de bouton mort). Test ajouté (`useTokenStream.test.ts` : deltas → debug exit → close ⇒ `ended`, texte préservé, `error=null`).
- **Commentaire GATE_MESSAGE surclamé** — `useOperator.ts:26-31` : reword honnête (FR restitution de la barrière ; le chemin SSE remonte `stream.gate` = texte backend brut ; near-mutually-exclusive car /send court-circuite avant l'ouverture du flux).

**P2 (3/3 fermés)**
- `relaunch` / `interrupt` / `newSession` désormais testés (`useOperator.test.ts` +3) ; `relaunch` ne re-`send` pas (re-stream plein), `interrupt`→`aborted` jamais `error`, `newSession`→composeur vide.
- Branche `error` 1er-terminal exercée (`useTokenStream.test.ts` : delta → error ⇒ `status=error`, texte partiel gardé).
- `newSession` **câblé** (plus de code mort) : bouton « ＋ Nouvelle session » dans l'Atelier hors-streaming (`Atelier.tsx`).

**P3 (traités)**
- `getStatus`/`SprintStatus`/`PhaseStatus` **retirés** (code mort orienté-futur ; le rail consomme `getContext`) ; test d'erreur `OperatorError` re-couvert via `getContext`. Phase D les ré-ajoutera pour l'arbre de procédé.
- **Flush final `TextDecoder`** ajouté avant `decoder.end()` (`useTokenStream.ts`).
- **Atelier** : l'erreur s'affiche désormais même avec sortie partielle (`div text-bad`), plus masquée par `body`.
- **`coverage.all:true`** activé (`vitest.config.ts`) — dette de couverture des composants présentationnels (exercés par l'E2E) rendue visible ; `coverage` ajouté aux ignores eslint + `.gitignore`.
- **`useRailStatus` testé** (invariant #7 : rail = `/api/context`, jamais `/api/gates`, placeholder pour le pouls).

**P3 carry (documentés, hors-scope Phase C)**
- Champs wire typés mais non consommés (`SendResult.provider`, `done.cost_usd/duration_ms`) = surface orientée-futur assumée.
- `sse_gate` backend forge `requires_gate` par `format!` brut (`operator_server.rs:1063`) — non exploitable (msg constant) + front défensif (`parseChunk` try/catch) ; à durcir (`serde_json::to_string`) si le message devient dynamique → **carry sprint dette Rust**.
- Branche catch-AbortError directe (`useTokenStream.ts`) vérifiée via la voie post-loop ; le double-chemin reste correct.

---

## 10. Codex reconciliation

Gate croisée **Codex GPT 5.5** lancée sur le diff stagé (`codex exec`, artefact brut `sprint80_phase_c_codex_review.md`, non réécrit). Codex a relancé lui-même `lint` / `tsc` / `test:unit` / `test:e2e`.

**Verdict Codex : 8/8 livrables CONFIRMÉ, 0 GAP, 0 PARTIEL.** Chaque livrable confronté au contrat backend Rust réel (StreamChunk 5 variantes vs union front 6 valeurs, `sse_gate` forgé hors serde, `from_provider`, gate sans spawn). 2 observations P3 :

- **P3-1 (corrigé)** — `useTokenStream.ts` finally faisait `releaseLock()` sans `cancel()` sur terminal : un serveur qui streamerait au-delà du terminal laisserait la connexion ouverte. **Fix** : `reader.cancel().catch(()=>{})` dans le finally (ferme déterministement le body sur terminal/EOF ; idempotent avec l'annulation du listener d'abort ; no-op sur flux déjà clos). Suites re-lancées vertes (tsc 0 · lint 0 · Vitest 52/52 · build · E2E 4/4).
- **P3-2 (non-issue, documenté)** — Codex note que le CTA d'intention « **Vérifier** avant validation » (`catalog/intentions.ts`) contient « Vérifier ». Ce n'est PAS un verdict : `scan-front-discipline.sh` interdit `\b(PASS|Vérifié|Verifie|Approuvé|Approuve)\b` — l'infinitif « Vérifier » (intention de l'utilisateur) ≠ le participe « Vérifié » (verdict). Le gate est précis et passe vert ; aucune copie à renommer.

Séquence respectée : preflight (PLAN-ADAPT) → review (PASS-PENDING) → Codex (8/8 CONFIRMÉ) → corrections P1/P2/P3 + P3-1 Codex → suites re-vertes → **PASS**.

## Verdict: PASS