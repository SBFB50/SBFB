# Sprint 6 — Audit Plan (à jouer dans une session fraîche)

**Écrit** : 2026-04-11, en fin de Sprint 6, par l'agent qui
vient de livrer les 6 commits `02ab9bf` → `3d1c3d5`.

**Pourquoi ce document** : `.planning/sprint6_verification.md`
est une checklist fail-fast 24 rows **self-reportée** par
l'agent qui a écrit le code. Toutes les rows passent — mais
c'est l'agent qui les a écrites et qui confirme qu'elles
passent. Ce n'est pas une vérification, c'est une auto-attestation.

Ce document est le plan d'un **audit indépendant** à exécuter
dans une session Claude Code fraîche, sans historique Sprint 6,
pour challenger les choix et trouver les vraies faiblesses avant
Sprint 7/8.

**Principe** : le fail-fast dit "le code compile, les tests
passent". L'audit dit "le code fait ce qu'il prétend faire, la
surface testée correspond à la surface exécutée en prod, et
les décisions sont justifiées à la relecture".

---

## 0. Mode d'emploi pour la session fraîche

**Avant de commencer**, l'auditeur (agent ou humain) doit :

1. `git log --oneline master ^cdf4467` — lire les 6 commits Sprint 6
2. Lire `.planning/sprint6_kickoff.md` (kickoff) + `.planning/sprint6_plan.md` §2 (D1..D5) + `.planning/sprint6_verification.md` (self-report)
3. **Ne pas lire** `docs/shell/PATTERNS.md` §P8 avant d'avoir
   formé un avis sur la policy — pour challenger, pas ratifier
4. Tenir un journal `.planning/sprint6_audit_findings.md` au
   fur et à mesure. Chaque finding = `{track, severity, what, evidence, fix}`
5. Sévérités : **P0** (casse prod / data loss), **P1** (bloque
   Sprint 7 ou 8), **P2** (tech debt explicite), **P3** (nit)

**Timebox suggéré** : 2-3 h de session. Audit indépendant, pas
re-spec.

**Format du delivrable final** : une section par track ci-dessous,
chacune avec son verdict PASS / CONCERN / FAIL + la liste des
findings. Puis un **verdict global** (PASS / CONDITIONAL PASS /
FAIL) avec les conditions pour lever un CONDITIONAL.

---

## 1. Track A — Intégrité du contrat cross-langue

**Question centrale** : le `TabView` Pydantic (Python) et
`TabViewSchema` Zod (TypeScript) **valident-ils exactement les
mêmes payloads** ? La seule garantie actuelle est un snapshot
JSON-schema Python-only. Un développeur peut renommer un champ
côté Python + régénérer le snapshot + oublier de toucher Zod.

### A1 — Structural diff des deux schémas

**Méthode** :
1. Dans `packages/nexus-sdk/`, dumper le JSON schema Pydantic via
   `python -c "from nexus_sdk.view import TabView; import json; print(json.dumps(TabView.model_json_schema(), indent=2, sort_keys=True))"` et sauver dans `/tmp/pydantic.json`
2. Dans `web/`, écrire un petit script Node qui importe
   `TabViewSchema` depuis `src/components/app/tabview/schema.ts`
   et appelle `zodToJsonSchema` (dep temporaire) pour produire
   `/tmp/zod.json`
3. `diff /tmp/pydantic.json /tmp/zod.json` — diff structurel
4. Attendu : différences acceptables **uniquement** sur la
   forme interne (Pydantic produit `$ref`/`$defs`, Zod produit
   anyOf inline). **Toute divergence sur les noms de champs,
   les literals, les requireds, les types primitifs est un
   P1 finding.**

**Pass** : tous les kinds ont les mêmes champs, mêmes requireds,
mêmes literals. **Concern** : divergences acceptables mais
non documentées. **Fail** : au moins un champ diverge.

### A2 — Round-trip réel Python → HTTP → Zod

**Méthode** :
1. Écrire un petit script `scripts/cross_lang_roundtrip.py`
   qui construit un TabView maximal (au moins 1 bloc par kind,
   valeurs edge : unicode, négatifs, grandes listes, nulls,
   champs optionnels omis vs présents)
2. POST ce dict à une instance coordinator éphémère qui expose
   `/tabview/echo` (à créer temporairement comme outil d'audit
   — si on refuse d'ajouter une route, utiliser directement
   `/app/{name}/tabs/{tab}/descriptor` avec une app fixture)
3. Récupérer la réponse et la passer à `TabViewSchema.safeParse`
   côté Node via un autre script
4. **Attendu** : `parsed.success === true` pour CHAQUE TabView

**Pass** : 100% des payloads passent Zod. **Fail** : un seul
payload produit un Zod `safeParse.success === false` — c'est
une drift silencieuse, P0 ou P1 selon le champ.

### A3 — Anti-snapshot : le snapshot guard détecte-t-il vraiment
une drift ?

**Méthode** :
1. Introduire volontairement une modif dans `nexus_sdk/view.py`
   (par exemple renommer `muted` → `subdued` dans `TabBlockText`)
2. Relancer `pytest packages/nexus-sdk/tests/test_view.py::test_view_schema_stable_snapshot`
3. Attendu : **le test doit échouer** avec un diff lisible
4. Revert

**Pass** : l'échec est lisible + nomme le champ qui a bougé.
**Fail** : test passe (snapshot ne détecte rien) ou échec
cryptique.

---

## 2. Track B — Résilience du renderer avec des données réelles

**Question centrale** : les 11 block kinds sont-ils **réellement
utilisés par un test de rendu avec des données non-triviales**,
ou bien 10 d'entre eux ne sont prouvés que par les Vitest
isolés et le seul exercice live est le tab gov minimal (heading
+ text + 2 metrics + empty) ?

### B1 — Audit du coverage du renderer

**Méthode** :
1. Rerun `cd web && npm run test:coverage`
2. Ouvrir le rapport et lister **tous** les uncovered lines sur
   `src/components/app/tabview/**`
3. Pour chaque ligne non-couverte, ouvrir le fichier et juger :
   est-ce une branche morte ? un cas d'erreur légitime ?
4. **Lignes connues au checkpoint Sprint 6** :
   - `TabBlockRenderer.tsx` 85.71% — lignes 45-46 (branche
     `_exhaustive: never`, acceptable mais flag qu'aucun kind
     inconnu n'a jamais frappé cette branche en test)
   - `ButtonBlock.tsx` 57.14% — lignes 18-24 (branche
     `task_submit` qui fait juste `console.warn`, dead code
     pour l'instant). **Demander : pourquoi shipper du code
     mort ?**
   - `schema.ts` 62.5% branches — lignes 298-299 (formatage du
     message d'erreur dans `parseTabView`)

**Pass** : chaque ligne non-couverte a une justification
écrite dans le journal. **Concern** : `ButtonBlock.tsx` 57%
n'est pas justifié — le code task_submit devrait être soit
retiré, soit testé. **P2**.

### B2 — Data fuzzing du renderer

**Méthode** :
1. Construire 10 TabViews "méchants" via Pydantic helpers :
   - Tableau de 500 lignes
   - Labels unicode + RTL (arabe, hébreu)
   - Chart avec 1 seul point / 0 point / 2 points identiques
   - Metric avec `delta=0` (branche neutre)
   - Metric avec valeur string très longue (`"1234567890".repeat(20)`)
   - Chart avec valeurs toutes négatives (yMin = yMax négatifs)
   - Chart_line avec 1 point (stepX = NaN ? toY ?)
   - Section récursive profondeur 10
   - Table avec colonne qui n'existe pas dans les rows
   - Button `task_submit` avec payload très imbriqué
2. Passer chacun via `TabViewSchema.safeParse` côté TS (via
   Vitest one-off) et assert success
3. Render chacun via `@testing-library/react` et vérifier qu'il
   ne crash pas

**Pass** : aucun crash, aucun NaN dans le SVG, aucun warning
React console. **Fail** : n'importe quel crash ou un SVG qui
produit `M NaN,NaN` ou équivalent.

### B3 — Observation manuelle du rendu live

**Méthode** :
1. Écrire une app fixture temporaire `examples/tabview-showcase/`
   qui contient **un tab par kind**, chaque tab valide mais
   stressé par des données edge
2. Boot coord + shell, naviguer manuellement (ou via Playwright
   headed) dans chaque tab et regarder le rendu
3. Screenshot each tab et les joindre au journal

**Pass** : tous les rendus lisibles, aucun bloc qui déborde
la carte, aucun label tronqué sans ellipsis. **Concern** :
design cassé sur au moins un kind.

---

## 3. Track C — Solidité des tests eux-mêmes

**Question centrale** : les 77 tests Vitest et les 10 Playwright
**testent-ils vraiment quelque chose**, ou bien sont-ils des
tautologies ("je construis X, puis j'asserte X") ?

### C1 — Mutation testing manuel sur 3 fichiers

**Méthode** : choisir 3 fonctions clés et introduire une
mutation subtile. Relancer les tests. Attendu : au moins un
test échoue pour chaque mutation.

Mutations à essayer :
1. `formatRelativeTime` : changer `past ? "il y a" : "dans"` en
   `past ? "dans" : "il y a"` (inversion)
2. `projectStore.addCoordinator` : changer `activeCoordinatorUrl:
   s.activeCoordinatorUrl ?? url` en `activeCoordinatorUrl:
   url` (casse la persistance de l'actif si déjà sélectionné)
3. `TabViewRenderer` : remplacer `tabView.blocks.length === 0`
   par `tabView.blocks.length === 1` (inversion de l'edge case)

**Pass** : les 3 mutations font échouer au moins un test chacune.
**Fail** : une mutation passe silencieusement — le test n'est
pas assez précis.

### C2 — Audit des Vitest `describe.skip` / `it.skip` / `test.only`

**Méthode** : `grep -rn 'skip\|\.only' web/src/**/*.test.*`.
Attendu : 0 résultat. Tout skip non-documenté est un P2.

### C3 — Playwright : est-ce que `tabview-schema-driven.spec.ts`
vérifie vraiment le rendu ou juste la présence du texte ?

**Méthode** :
1. Lire la spec
2. Pour chaque `expect`, se demander : "et si le texte était
   dans un bloc legacy-fallback `<details>` au lieu du
   renderer, est-ce que le test l'attraperait ?"
3. Le test DOIT avoir une assertion négative `expect(...).toHaveCount(0)`
   sur "Descripteur legacy" — vérifier que c'est le cas (oui,
   c'est le cas d'après le fichier, mais le confirmer).
4. Ajouter une assertion positive sur une classe CSS ou un
   `data-testid` du renderer (pas juste du texte) pour
   renforcer l'ancre.

**Pass** : la spec a au moins une assertion structurelle (pas
juste du texte) + une négative sur la fallback. **Concern** :
full text-based.

---

## 4. Track D — Legacy fallback : bombe à retardement ?

**Question centrale** : `legacy_descriptor: true` est un
fallback "une release seulement". Est-ce qu'il y a **quoi que
ce soit** qui garantit que la release suivante le retire ?

### D1 — Recherche de sentinelle

**Méthode** :
1. `grep -rn 'legacy_descriptor' packages/ web/`
2. `grep -rn 'TODO.*legacy\|FIXME.*legacy\|Sprint 8.*legacy' .`
3. Vérifier qu'il existe **au moins un** :
   - commentaire `# DEPRECATED Sprint 7/8` avec une date ou un
     commit SHA de référence
   - test qui échoue si legacy est utilisé en dehors de l'app
     gov porté (test de non-regression inversé)
   - issue GitHub ou TODO dans `.planning/sprint8_*` rappelant
     la suppression

**Pass** : au moins un mécanisme de rappel existe. **Fail** :
rien — c'est à la mémoire humaine et ça se perdra.

### D2 — Test du chemin fallback : warning log et fallback

**Méthode** :
1. Créer une app fixture qui retourne `{"truc": 42}`
2. Booter coord + requêter `/app/{name}/tabs/{tab}/descriptor`
3. Vérifier que la réponse a `legacy_descriptor: true` et que
   le WARNING est bien loggé (pas silent)
4. Côté shell, vérifier que le `<details>` fallback s'ouvre et
   montre le JSON raw

**Pass** : warning visible + details accessible. **Concern** :
silent fallback (difficile à déboguer en prod).

### D3 — Compteur de legacy descriptors au boot

**Méthode** : est-ce que le coord logge, au boot ou à chaque
request, un compteur `"N apps still returning legacy
descriptors"` ? Si non, un ops ne verra jamais qu'il faut
migrer.

**Pass** : log existe. **Concern** : ajouter comme P2 tech
debt pour Sprint 7/8.

---

## 5. Track E — Bundle budgets : utiles ou placebo ?

**Question centrale** : les budgets D5 (main 475 / vendor-react
210 / vendor-ui 110 / css 100 KB raw bytes) sont généreux —
Sprint 5 était à 425/190/0/90, Sprint 6 à 455/189/31/93. Ils
laissent une marge de +50 KB. Est-ce que les budgets attraperaient
une régression de +30 KB ?

### E1 — Où sont allés les +30 KB sur le main chunk ?

**Méthode** :
1. `cd web && npx vite build --mode=analyze` ou
   `npx vite-bundle-visualizer` en dep temporaire
2. Lister les 10 plus gros modules dans `dist/assets/index-*.js`
3. Identifier ce qui est nouveau vs Sprint 5 : le renderer
   TabView (~400 LOC), les 11 block components, le Zod schema,
   cmdk, le command palette, les 2 SVG charts

**Attendu** : le renderer + cmdk expliquent ~30 KB. Si autre
chose apparaît, c'est un leak (par exemple un import
transitif de `moment` via `zustand/middleware`).

**Pass** : la comptabilité boucle et explique tout. **Fail** :
quelques KB inexpliqués → leak à investiguer.

### E2 — Tightening des budgets

**Méthode** : proposer des nouveaux budgets serrés, par
exemple main = actuel + 20 KB (475 KB), vendor-ui = actuel +
10 KB (42 KB), css = actuel + 5 KB (99 KB). Vérifier que le
build actuel passe toujours et que ça laisse juste assez de
place pour Sprint 7 (ajout de fetch API + daemon client).

**Pass** : budgets actualisés pour Sprint 7, documentés dans
PATTERNS.md T2. **Concern** : les laisser larges cache les
drifts futurs.

### E3 — Mode brotli parallèle ?

**Méthode** : ajouter un deuxième run `size-limit` avec
`brotli: true` sans limites (juste reporting) pour tracer
l'évolution de la taille transférée réelle.

**Pass** : reporting ajouté (non-bloquant). **Concern** :
pas critique pour Sprint 6.

---

## 6. Track F — Ctrl+K : vraiment portable ?

**Question centrale** : le Playwright test ouvre la palette via
le bouton header + un `dispatchEvent(KeyboardEvent)` synthétique.
`page.keyboard.press("Control+K")` ne déclenchait rien en
headless Chromium. **Est-ce que ça marche vraiment dans un vrai
browser ?**

### F1 — Test manuel trois browsers

**Méthode** :
1. `cd web && npm run dev`
2. Tester Ctrl+K dans : Chromium/Chrome, Firefox, Safari (si
   dispo), Edge
3. Pour chaque : est-ce que le dialog s'ouvre ? Escape ferme ?
   Tab ordering correct ? Est-ce que Ctrl+K intercepte les
   raccourcis browser (par exemple Ctrl+K = search bar dans
   Chrome) ?

**Pass** : 4/4 browsers ouvrent la palette au premier Ctrl+K.
**Concern** : Chrome (par exemple) vole le raccourci parce que
le handler ne met pas assez vite `preventDefault`. **Fail** :
au moins un browser ne réagit pas.

### F2 — Conflit avec input text focus

**Méthode** : dans le dialog « Ajouter un coordinateur », focus
l'input URL, presser Ctrl+K. Est-ce que ça ouvre la palette par
dessus le dialog (multi-layer) ou est-ce qu'il mange Ctrl+K ?

**Pass** : soit la palette s'ouvre par-dessus proprement, soit
le focus dans input désactive le raccourci de façon prévisible.
**Concern** : comportement ambigu ou double-dialog sans issue.

### F3 — Command palette ne prévient pas les défauts browser

**Méthode** : relire `useCommandPalette.ts`. Confirmer que
`e.preventDefault()` est bien appelé **avant** `setOpen`.
Vérifier aussi si on doit `stopPropagation` pour éviter que
cmdk lui-même intercepte.

**Pass** : preventDefault présent, commenté. **Concern** : pas
de stopPropagation documenté.

---

## 7. Track G — Risques pour Sprint 7 + 8

**Question centrale** : quelles hypothèses de Sprint 6 vont se
briser au Sprint 7 ou 8 ?

### G1 — Button.task_submit est dead code

Déjà identifié Track B1. Sprint 7/8 devra implémenter. Action :
soit retirer le kind `button` avec action `task_submit` du
vocabulaire v1, soit décider maintenant la signature du contexte
passé au handler.

**Pass** : décision écrite dans `.planning/sprint7_kickoff.md`
avant Sprint 7. **Concern** : pas décidé → Sprint 7 hérite
du choix.

### G2 — Command palette n'accepte pas d'items contribués par
des apps

Sprint 8 voudra qu'une app `gov` ajoute "Nouveau fact-check" au
palette. Actuellement, `CommandPalette.tsx` hardcode les 3
groupes. Il n'y a pas de SDK hook `registerCommand`. Le design
doit être fait avant Sprint 8.

**Pass** : API esquissée dans `.planning/sprint8_kickoff.md`
(mais ce kickoff n'existe pas encore — OK, le noter comme
pré-requis).

### G3 — Le vocabulaire v1 est-il suffisant pour Dashboard / Reseau / Videos ?

Pour les tabs gov qui impliquent des charts complexes ou du
graph WebGL, **le plan Sprint 8 a déjà assumé un scope cut**
(reagraph et Leaflet ne reviennent pas). L'auditeur doit
confirmer : est-ce qu'un rendu tabulaire suffit pour "Reseau" ?

**Pass** : le plan Sprint 8 assume le scope cut → pas un
finding Sprint 6. **Concern** : si l'auditeur pense que c'est
inacceptable, remonter comme risk pour le roadmap.

### G4 — RouterProvider + Command palette + React Query + Zustand — ordre
des providers

Le plan D5 veut "Ctrl+K ouvre même sur une page d'erreur React".
Vérifier que `<CommandPalette>` est monté **en dessous** du
RouterProvider mais **au-dessus** de `<Outlet>` pour être
robuste à un crash de route. Lire `App.tsx` et `AppShell.tsx`.

**Pass** : palette toujours montée tant que l'outer shell
tient. **Concern** : montée dans `AppShell` qui est elle-même
un route element → crash de la route = palette disparaît.

---

## 8. Track H — Dépendances et sécurité

### H1 — Audit npm

**Méthode** : `cd web && npm audit --audit-level=moderate`
**Pass** : 0 vuln ≥ moderate. **Concern** : 1-2 vuln dans des
devDeps (vitest transitive). **Fail** : une vuln dans une dep
de runtime (react, zod, zustand, cmdk, react-router, @tanstack).

### H2 — Versions pinnées

**Méthode** : lister toutes les nouvelles deps Phase D
(`vitest`, `@testing-library/*`, `jsdom`, `size-limit`,
`@size-limit/file`, `@vitest/coverage-v8`). Vérifier les
versions majeures, chercher des breaking changes connus.

**Pass** : tout est en version stable. **Concern** : une dep
en 0.x ou en RC.

### H3 — cmdk + React 19 compat

**Méthode** : chercher l'issue tracker de cmdk pour "React 19"
ou "subscribe undefined". L'audit Sprint 6 a découvert un bug
subtil (shadcn CommandDialog sans `<Command>` wrapper). Vérifier
que ce bug n'a pas d'autre manifestation ailleurs dans shadcn
v4 / cmdk 1.1.1.

**Pass** : issue documentée, workaround correct. **Concern** :
bug ouvert côté upstream qui pourrait réapparaître.

---

## 9. Track I — Documentation et traçabilité

### I1 — Le kickoff/plan/verif sont-ils cohérents ?

**Méthode** : lire les 3 documents et confirmer que :
- les 5 décisions D1..D5 du kickoff apparaissent toutes dans
  le plan
- les 24 rows de la fail-fast du plan apparaissent toutes dans
  la verif
- la verif nomme les commits SHA réels (pas des placeholders)

**Pass** : triangulation cohérente. **Concern** : un item
manque dans un des 3 docs (par exemple un décision prise en
cours de route et non documentée).

### I2 — Les commit messages correspondent aux changements

**Méthode** : `git show` sur les 6 commits Sprint 6 et vérifier
que le message résume fidèlement le diff. Chercher les
commit messages qui prétendent avoir fait X mais dont le diff
ne contient pas X.

**Pass** : 6/6 fidèles. **Concern** : au moins un commit
messages gonfle son propre scope.

### I3 — MEMORY.md / nexus_grid_pivot.md à jour

**Méthode** : lire les deux fichiers de mémoire Claude.
Confirmer qu'ils reflètent bien "Sprint 0→6 closed, Sprint 7
= P2P Discovery next".

**Pass** : à jour. **Fail** : dit encore "Sprint 5 closed,
Sprint 6 = next".

---

## 10. Format du delivrable

L'auditeur produit un **seul fichier** : `.planning/sprint6_audit_findings.md`

Structure minimale :

```markdown
# Sprint 6 — Audit Findings

**Auditor**: {fresh context, date}
**Commits audited**: 02ab9bf..3d1c3d5

## Track A — Contract integrity — VERDICT: PASS | CONCERN | FAIL
- A1: ...
- A2: ...
- A3: ...

## Track B — Renderer resilience — VERDICT: ...
...

## Global verdict

**PASS | CONDITIONAL PASS | FAIL**

Conditions to lift CONDITIONAL (if any):
1. ...
2. ...

## Findings list (sorted by severity)

| # | Severity | Track | Summary | Fix effort |
|---|---|---|---|---|
| 1 | P0 | ... | ... | ... |
...
```

**Sévérités** :
- **P0** — casse prod / data loss. Bloque merge vers main (si
  on avait une PR flow).
- **P1** — bloque Sprint 7 ou 8. À corriger avant début Sprint 7.
- **P2** — tech debt explicite. À trackher dans
  `docs/shell/PATTERNS.md` ou le plan Sprint suivant.
- **P3** — nit. Optionnel.

---

## 11. Ce que l'audit ne doit PAS faire

- **Re-écrire le vocabulaire TabView**. Si l'auditeur pense que
  le vocabulaire est mal taillé, il le note comme finding P2 —
  il ne propose pas une v2.
- **Ré-implémenter le renderer**. Si un bloc component est mal
  écrit, finding P1/P2 et exemple de fix en 10 lignes, pas un
  rewrite.
- **Sortir du scope Sprint 6**. Ne pas auditer Sprint 5 (déjà
  fait) ni anticiper Sprint 7 (pas encore planifié).
- **Modifier le code sans demander**. L'audit produit un
  rapport + des findings. Les fixes viennent après, en commits
  séparés `fix(...)` si l'utilisateur les approuve.

---

## 12. Pré-requis techniques

- `cargo`, `uv`, `node 22`, `npm`, `python 3.13` sur le PATH
- `web/node_modules` installé (`cd web && npm install`)
- `target/debug/nexus-worker.exe` compilé (`cargo build -p nexus-worker`) pour le Playwright state roundtrip
- `docker compose up -d` (Neo4j / ChromaDB / Robin) **pas
  nécessaire** — Sprint 6 ne touche pas aux services Docker

---

**Fin du plan**. La session d'audit produit
`.planning/sprint6_audit_findings.md` + éventuellement une
pile de commits `fix(...)` si l'utilisateur valide les
findings P0/P1.
