# React 19 vs Solid 2.0 — retour objectif pour le Factory Operator (2026-06-27)

## 0. TL;DR

**GARDER React 19. Ne pas parier la beta Solid 2.0 maintenant** — malgré un coût de bascule réellement nul (0 ligne de front écrite). Confiance : **élevée** sur la direction (les 5 axes + les 2 lentilles adversariales convergent), **moyenne** sur la *marge* affichée par le score blueprint (459 vs 381 partiellement gonflé en greenfield, cf. §3). Le seul terrain où Solid pouvait gagner — le stream SSE async-first (PO-14) — ne tient pas comme justification : ~80-90 % du confort SSE vient de la **réactivité fine-grained déjà présente en Solid 1.9 stable**, pas de la nouveauté 2.0 ; `createAsync` ne « stream » pas les tokens ; et le footgun central (auto-reconnect `EventSource`) est **agnostique au framework**. Parier une beta « may still break » à GA non datée, hors-corpus-LLM, pour un control-center solo en lignée OpenBSD, serait du **churn auto-infligé**.

---

## 1. Fait charnière : statut Solid 2.0 (FAIT VÉRIFIÉ, ≥3 sources)

- **Solid 2.0 = BETA.** `v2.0.0-beta.0` sortie le **3 mars 2026**, publiée sous tag npm `next` (`solid-js@next`, `@solidjs/web@next`). Stable courant `latest` = **1.9.x**. [github releases #2596, InfoQ]
- **Fondation réactive réécrite from scratch** = `@solidjs/signals`, auto-décrite : « API stabilizing but **MAY STILL HAVE BREAKING CHANGES before final release** ». [github/solidjs/signals, npm]
- **Aucune date GA annoncée.** Pipeline *feedback-driven* (Carniato : « depends on community feedback »), pas calendaire ; l'Alpha a été sautée, il reste **Beta → RC → GA**. [listiak, discussion #2425]
- **Symptômes d'immaturité actuels** : pas de devtools sur 2.0 (dev hooks absents, signals réécrits) ; bugs cœur ouverts #2679 (« stores don't batch like signals do ») et #2604 (async signal n'affiche pas `initialValue`) ; débats de **design** encore ouverts sur le split-phase `createEffect` et la lecture post-`flush()`. [github issues]
- **SolidStart 2.0 = alpha, « not yet production-ready »** (non bloquant : SBFB = SPA/SSG servie par ServeDir Rust, pas de Node ; mais symptôme que le full-stack 2.0 traîne derrière le core). [listiak]
- **Messaging incohérent** entre sources (release tag « beta » vs articles décrivant encore « experimental, no dates ») = signal en soi que la trajectoire GA n'est pas verrouillée. (inférence)

Côté React, pour cadrer la comparaison (FAIT) : **React 19.0 stable depuis le 5 déc. 2024** (18 mois de prod), branches 19.1 et 19.2 patchées en parallèle (19.2.7 + 19.1.8 le 1er juin 2026) ; **React Compiler v1.0 GA le 7 oct. 2025** (Vite supporté) ; **Base UI 1.0 GA le 11 déc. 2025** (latest ~1.6, juin 2026).

---

## 2. Comparaison par axe qui COMPTE pour CE tool

| Axe | React 19 | Solid 2.0 | Gagnant | Pourquoi (fait / inférence) |
|---|---|---|---|---|
| **Stream SSE / async (le + décisif, PO-14)** | `use()`/Suspense = async one-shot (RSC), **ne couvre pas** le token-streaming ; la danse `useRef`/`useEffect`/`accumRef`/anti-reconnect **reste** (vérifiée dans `ExecutionChat.tsx` l.96-250, ~8 mécanismes) | fine-grained supprime ~8 mécanismes (signaux stables → 0 `accumRef`, `onCleanup` déterministe → 0 effet StrictMode) ; `createAsync` aide l'**orchestration** (send/gate/abort), **pas la boucle token** | **Solid (modéré)** | FAIT : la réduction de plomberie est réelle. **MAIS** elle vient de la réactivité fine-grained **dispo en Solid 1.9 stable**, pas de la 2.0. `createAsync` résout *une* Promise, pas N deltas (inférence ≥2 sources). Footgun auto-reconnect `EventSource` = **agnostique** (MDN). |
| **Headless a11y** | **Base UI 1.x GA**, 25-35+ composants (Combobox/listbox/popover/focus-trap/positionnement) en **1 dépendance mûre**, pedigree Radix+MUI, cadence mensuelle additive | Kobalte (~1700★, **épinglé Solid 1.x**, compat 2.0 non confirmée) + corvu (quasi-mono-mainteneur) + Ark/Zag (marche aussi en React → égalisateur) ; **composer 2-3 libs** | **React (net)** | FAIT : couverture + gouvernance + une seule dep. C'est l'axe où le **pari beta est le plus défavorable** : l'écosystème headless le plus mûr est ancré 1.x. |
| **Interop CM6 (`@codemirror/merge`) + xterm.js** | escape-hatch `ref`+`useEffect`, footgun StrictMode double-mount ; wrappers mûrs (`@uiw/react-codemirror`) mais **inutiles pour `MergeView`** | `onMount` une fois, refs stables, pas de double-invoke ; glue `MergeView` à la main aussi | **Neutre** (micro-edge Solid) | FAIT : libs JS impératives → glue maison des deux côtés pour le diff-viewer. **Ne fournit aucun motif** de bascule. |
| **Fluence-agent + corpus** | Domine le corpus LLM (cycle auto-renforçant React+Tailwind+shadcn) ; React 19 émis fiablement 1-3 tentatives | Solid niche (~35k★, ~1,5M dl/sem, ~10-20× sous React) ; **API 2.0 réécrite ≈ absente du corpus** | **React (fort)** | FAIT : benchmark dur « SolidJS+Actix > 5 tentatives / échecs » vs React 1-3. INFÉRENCE étayée : agent hallucinera `createAsync`/split-phase/batching (analogie runes Svelte 5). Aggravant : tool écrit par agent + migration future **sans devtools 2.0 ni corpus**. |
| **Churn / risque / longévité (lignée OpenBSD)** | Meta-backed, gouvernance distribuée, 10 ans d'API, deux mineures patchées en // | Bus-factor élevé (figure unique Carniato/Netlify), beta « may break », 0 date GA | **React (net)** | FAIT : le critère « stabilité > churn, lire-un-diff-dans-5-ans » exclut structurellement une beta à GA non datée pour un solo. |
| **Coût-de-bascule maintenant** | — (incumbent) | Nul à T0 (0 ligne front) | **N'inverse pas** | FAIT : seul terme abaissé. N'affecte ni la fluence-agent récurrente, ni le churn beta sur des années, ni le bus-factor. Argument **ponctuel** vs coûts **intégrés**. Symétrie : **attendre la GA est aussi gratuit**. |

---

## 3. Le vrai arbitrage (honnête des deux côtés)

**Le cas pro-Solid, dans sa version la plus forte :**
- Le coût de bascule est **réellement nul** à T0 — c'est l'argument du PO et il est exact.
- La danse SSE React est un **coût récurrent réel et code-vérifié** (pas une caricature) : `handleSend` = ~130 lignes, 3 refs, flag de closure, helper `close()` répété sur 4 branches terminales, effet StrictMode dédié, gardes `prev ? … : prev`. Le stale-closure est « an entire family of bugs ». Même React 19.2 (`useEffectEvent`, oct. 2025) **ne résout pas** l'accumulateur mutant d'un handler `EventSource` long-vécu.
- Le confort **lecture long-terme** du fine-grained (pas de Rules-of-Hooks, pas de stale-closure, pas de tableau de deps) est réel pour le job #1 (lire un diff dans 5 ans).
- **Faille méthodo réelle** : le score blueprint 459-vs-381 est **partiellement gonflé en greenfield**. Le kickoff S80 **jette** `tools/factory-operator` ET `tools/factory-ui` → les « 411 tests React déjà là » sont jetés, « Base UI + xterm déjà câblables » = 0 ligne écrite (intention, pas actif). Une bonne partie du poids « incumbence » s'évapore. La *direction* tient sur les actifs durables (connaissance mainteneur + corpus + écosystème mûr), mais la **marge de confiance est à revoir à la baisse**.

**Pourquoi ça ne renverse PAS la balance :**
1. Le gain SSE convoité est ~entièrement capturable en **Solid 1.9 stable** (signals + `onCleanup`) — il **n'exige pas la beta 2.0**. La 2.0 n'est requise que pour l'orchestration async-first + Suspense retravaillé, et ce n'est *pas* ce qui rend la boucle token douloureuse.
2. Le gain 2.0 convoité tombe **pile sur la zone la moins stable** (`createEffect` 2-4 args, batching + `flush()`, piège « track-after-`await` » — littéralement le terrain du stream token→token) + bugs ouverts + 0 devtools.
3. Le footgun PO-14 (auto-reconnect = re-run du dernier tour) est **agnostique** : ni `use()` ni `createAsync` ne le règlent. Le vrai correctif est **orthogonal au framework** (cf. §5).
4. La fluence-agent (tool écrit par agent) et le headless mûr (Base UI GA vs Kobalte 1.x) tranchent net pour React, et le **pari beta dégrade spécifiquement** ces deux axes.
5. Côté React, le différenciateur perf/boilerplate de Solid est **largement absorbé** par React Compiler v1.0 (mémoïsation auto, y compris conditionnelle post-early-return). Contrepartie honnête : le Compiler est une **couche opaque** (bail-out silencieux) dont le risque tombe pile sur le code SSE/effets — mais **mitigeable** (lint, pin de version, gate T1 E2E déjà BLOQUANT).

**Résumé de l'arbitrage** : le coût-de-bascule-nul est un argument *ponctuel (T0)* opposé à des coûts *intégrés sur des années* (churn beta, fluence-agent, bus-factor). Pour un invariant de **longévité solo**, l'argument T0 n'inverse pas. Et comme l'inverse est vrai — attendre la GA est gratuit tant que rien n'est écrit — **rien n'oblige à parier la beta maintenant.**

---

## 4. Verdict + recommandation (framé **G8-PROPOSE** — le PO tranche)

> Rappel cadre : la décision **React 19 est figée au kickoff S80 (Day-0 D1)**. Ceci est le **re-check documenté** prévu à la charnière D1 (« re-vérifier Solid 2.0 GA au preflight de la 1re phase front »). C'est une **proposition de confirmation**, pas une autorité de décision.

**PROPOSITION : CONFIRMER React 19.** Sur les 5 axes et les 2 lentilles adversariales, ce n'est pas un match serré : fluence-agent, headless mûr, churn/longévité pointent tous vers React ; l'interop est neutre ; le seul axe pro-Solid (SSE) ne justifie **pas la beta** car son bénéfice est déjà en Solid 1.9 stable et n'efface pas le footgun PO-14.

**Honnêteté affichée dans la proposition :** la *marge* du score blueprint est gonflée en greenfield (factory-operator + factory-ui jetés) ; la décision repose donc sur **premiers principes** (corpus-agent + écosystème mûr + lignée OpenBSD), comme le kickoff l'admet déjà (« PAS une victoire de premiers principes » → à reformuler : c'est une victoire de **risque-minimisé**, l'alternative étant en beta).

**Clause de réouverture falsifiable (3 conditions, toutes requises ensemble) :**
1. Solid **2.0 GA** taggé `latest` (pas `next`/beta), clause « may still have breaking changes » de `@solidjs/signals` **retirée** ;
2. **Kobalte OU Ark** publie une version **stable estampillée Solid 2.0 GA** (à re-vérifier sur le `package.json` réel — cf. discrepancy honnête ci-dessous) ;
3. interop **CM6 / `@codemirror/merge` / xterm** confirmée propre sur 2.0 GA.

En juin 2026, **aucune des trois** n'est réunie. Le re-check du preflight Phase B doit donc **confirmer React 19**.

*Discrepancy à porter au preflight Phase B :* la doc SSR Kobalte renvoie une chaîne de version **ambiguë** au résumé de recherche (impossible de trancher 1.x-only vs début de 2.x). Conclusion inchangée (tracker une beta reste tracker une beta ; aucun headless **GA-2.0 stable** n'existe), mais ce point précis se vérifie sur le `package.json` réel, pas sur les axes.

---

## 5. Plans d'action

### Si on garde React 19 (recommandé) — neutraliser la faiblesse SSE
1. **Quitter `EventSource` pour `fetch()` + `ReadableStream` + `AbortController`** (le vrai correctif, **orthogonal au framework**) : supprime l'auto-reconnect implicite (neutralise PO-14 mieux que toute feature framework), permet le header `X-SBFB-Token` natif (élimine la raison du proxy Vite documentée l.13-16 d'`executionChat.ts`), abort déterministe. À adopter **quel que soit le framework**.
2. **Encapsuler une fois dans une primitive testée** `useEventStream`/`useTokenStream` (accumulateur + Done-unique PO-14 + abort/reconnect) → la danse `useRef`/`useEffect` devient un détail interne audité une fois, pas un pattern répété.
3. **Gate E2E hermétique T1 BLOQUANT** (déjà en place) pour couvrir l'interaction React Compiler ↔ effets ; **pin de version exacte** du Compiler tant que la couverture n'est pas dense (discipline `--locked` / dual-platform déjà naturelle).
4. **Base UI 1.x** pour listbox/Combobox/popover/focus-trap/positionnement (1 dep mûre, 0 réimplémentation a11y) ; **CM6/xterm** via `ref`+`useEffect(mount/unmount)` avec garde anti-double-création StrictMode (glue `MergeView` à la main, attendue des deux côtés).

### Si on bascule Solid (NON recommandé maintenant) — risques à accepter explicitement
- **Churn non borné** : `@solidjs/signals` peut casser après 5k lignes écrites ; migration = re-révision manuelle **sans devtools 2.0 ni corpus-LLM** d'assistance.
- **Fluence-agent dégradée au 1er jet** sur l'API 2.0 réécrite (hallucinations `createAsync`/split-phase/batching) — coût aggravé car le tool est écrit par agent.
- **Pari écosystème headless** : suivre Kobalte/Ark/corvu sur 2.0 (aucun GA-2.0 stable aujourd'hui) → composer 2-3 libs, surface de maintenance accrue pour un solo.
- **Mitigation minimale si décision PO inverse** : exiger d'abord **Solid 2.0 RC** (pas beta) ; n'utiliser que l'API stabilisée ; isoler le stream dans une primitive ; ne PAS dépendre de `createAsync` pour la boucle token (de toute façon impératif). À noter : si le seul motif est le confort SSE, **Solid 1.9 stable** en livre l'essentiel **sans** le risque beta — option intermédiaire rarement évoquée mais cohérente.

---

## 6. Sources (URLs datées, recoupées ≥2)

**Solid 2.0 / statut / signals**
- github.com/solidjs/solid/releases — v2.0.0-beta.0 (3 mars 2026)
- github.com/solidjs/solid/discussions/2596 — v2.0.0 Beta ; /2425 — The Road to 2.0
- github.com/solidjs/signals — clause « may still have breaking changes » ; npmjs.com/package/@solidjs/signals
- github.com/solidjs/solid/issues/2679 (stores batch) ; /2604 (async signal initialValue)
- infoq.com/news/2026/05/solidjs-2-async/ (async first-class, signals réécrits)
- listiak.dev/blog/the-state-of-solid-js-in-2026-… (GA non datée, niche, SolidStart alpha)
- brenelz.com/posts/migrating-to-solid-2/ (createEffect 2-4 args, batching/flush, track-after-await)
- docs.solidjs.com/solid-router/reference/data-apis/create-async ; …/reference/lifecycle/on-cleanup ; …/basic-reactivity/create-resource (Context7 /websites/solidjs)
- tanstack.com/blog/tanstack-start-solid-v2 (support écosystème 2.0 en cours)

**React 19 / Compiler / Suspense**
- react.dev/versions ; react.dev/blog/2024/12/05/react-19 ; …/2025/10/01/react-19-2 ; …/2025/10/07/react-compiler-1
- react.dev/reference/react/use ; react.dev/reference/react/Suspense ; react.dev/learn/react-compiler/debugging
- blog.logrocket.com/react-compiler-rc/ ; blog.logrocket.com/react-useeffectevent/ ; github.com/reactwg/react-compiler/discussions/1
- saschb2b.com/blog/react-compiler-year-in-review ; infoq.com/news/2025/12/react-compiler-meta/
- edge-cases.com/react/react-19-use-hook-promises ; dev.to/a1guy/react-19-suspense-deep-dive-… ; syncfusion.com/blogs/post/react-19-suspense-for-data-fetching

**Headless a11y**
- base-ui.com/react/overview/releases (1.x GA) ; github.com/mui/base-ui/releases ; infoq.com/news/2026/02/baseui-v1-accessible/ ; news.ycombinator.com/item?id=46245401
- github.com/kobaltedev/kobalte ; kobalte.dev/docs/core/overview/ssr/ (chaîne version ambiguë — re-check Phase B)
- github.com/chakra-ui/ark ; ark-ui.com/docs/overview/about ; npmjs.com/package/@ark-ui/solid ; corvu.dev / github.com/corvudev/corvu
- preblocks.com/blog/radix-ui-vs-base-ui ; greatfrontend.com/blog/top-headless-ui-libraries-for-react-in-2026

**Interop CM6 / xterm**
- github.com/uiwjs/react-codemirror ; npmjs.com/package/@uiw/react-codemirror
- github.com/riccardoperra/solid-codemirror ; npmjs.com/package/solid-codemirror

**Fluence-agent / corpus / SSE API**
- saschb2b.com/blog/llm-default-react-stack ; dev.to/adioof/react-wont-die-because-llms-wont-let-it-8o (benchmark SolidJS+Actix échecs >5) ; makersden.io/blog/solidjs-vs-react-pros-and-cons
- developer.mozilla.org/.../EventSource/close ; .../Server-sent_events/Using_server-sent_events (auto-reconnect = propriété API)
- dmitripavlutin.com/react-hooks-stale-closures/ ; oneuptime.com/blog/post/2026-01-24-fix-stale-closure-issues-react-hooks/view

**Code réel SBFB lu en première main**
- `C:\Users\FlowUP\Documents\Code\nexus\tools\factory-operator\src\pages\ExecutionChat.tsx` (l.96-250) + `…\src\lib\executionChat.ts` (l.13-16, 81-87) — *à jeter au scaffold S80 (greenfield) : leur incumbence ≈ nulle.*