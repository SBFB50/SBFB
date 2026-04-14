# Sprint 6 — Plan détaillé (Shell Foundations + Schema Tabs)

**Écrit** : 2026-04-11 à partir de `.planning/sprint6_kickoff.md`
après validation user du split Sprint 6/7/8 et des décisions
Day 0 D1..D5. Ce document est la grille d'exécution Sprint 6 :
chaque commit cite la phase, chaque test est listé ici, chaque
fichier touché est nommé. **Aucun code n'est écrit avant que
cette grille soit commitée** (`docs(sprint6): kickoff + detailed
plan`).

**HEAD entrée** : `cdf4467`. Working tree clean modulo
`.planning/sprint6_kickoff.md` + ce fichier (commités ensemble
en ouverture).

**Goal Sprint 6 (une phrase)** : livrer un vocabulaire `TabView`
figé + renderer React correspondant, un Ctrl+K palette câblé,
des Vitest unit tests ciblés et un garde-fou CI de bundle size,
avec hello-world-app et `nexus-app-gov` portés comme premières
apps schema-driven.

---

## 1. État vérifié à l'entrée

### 1.1 Sprint 5 livré (source : `sprint5_verification.md` 22/22 verts)

- 193 tests Rust workspace (62 core-rs lib + 11 worker bin lib
  + 10 worker e2e + 105 worker-core lib + 5 doctests)
- 43 tests Python coordinator + 1 skipped
- 8 Playwright specs en 7.8 s contre un coordinator live
  (onboarding, add-coord, my-projects, project-detail,
  apps-tab, my-network, /browse stub, /curators stub)
- `cargo fmt --all --check` + `cargo clippy --workspace -- -D warnings` clean
- `ruff format --check` + `ruff check` clean
- `tsc --noEmit -p tsconfig.app.json` clean (strict +
  noUnusedLocals + noUnusedParameters)
- `npm run lint` clean (0 errors, 5 warnings T1 documentés)
- `npm run build` : main 425 KB + vendor-react 190 KB + CSS 90 KB, zéro warning
- `bash web/scripts/scan-en-strings.sh` clean

### 1.2 Consommé directement par Sprint 6

**Frontend** (`web/src/`) :

| Fichier | LOC | Rôle Sprint 6 |
|---|---|---|
| `api/coordinator.ts` | 498 | Étendu : ajout d'un schéma Zod `TabViewSchema` (D2) et d'un helper `getAppTabDescriptor(appName, tabName)` typé |
| `stores/projectStore.ts` | 152 | **Cible Vitest** (D4) — inchangé |
| `lib/format.ts` | 75 | **Cible Vitest** (D4) — inchangé |
| `lib/utils.ts` | 6 | Inchangé |
| `components/ui/command.tsx` | 195 | Re-exporté via `CommandPalette.tsx` (Phase C) |
| `components/project/AppsTab.tsx` | 335 | **Rewrite Phase B** : remplace le `JSON.stringify` par `<TabViewRenderer>` |
| `pages/ProjectDetail.tsx` | 227 | Inchangé (D3 — les 5 tabs natifs restent hard-coded) |
| `components/AppShell.tsx` | 269 | **Patch Phase C** : monte `<CommandPalette>` au niveau racine hors `<Outlet>` |

**Coordinator Python** (`packages/nexus-coordinator/`) :

| Fichier | Rôle Sprint 6 |
|---|---|
| `api/apps.py` | Étendu Phase A : GET `/app/{name}/tabs/{tab_name}/descriptor` valide le retour contre `TabViewModel` Pydantic et renvoie 500 typé si un tab retourne du legacy non-schema. Fallback legacy permis **une seule release** via flag `legacy_descriptor: true` dans la réponse. |

**SDK Python** (`packages/nexus-sdk/`) :

| Fichier | Rôle Sprint 6 |
|---|---|
| `app.py` | Inchangé — l'ABC NexusApp et les décorateurs ne bougent pas |
| `view.py` *(nouveau)* | Constructeurs helper `section`, `heading`, `text`, `kv`, `metric`, `table_`, `badge_list`, `button`, `chart_line`, `chart_bar`, `empty` + Pydantic `TabView` |
| `__init__.py` | Re-export `view` module |

**App gov** (`packages/nexus-app-gov/`) :

| Fichier | Rôle Sprint 6 |
|---|---|
| `src/nexus_app_gov/app.py` | Le tab "Contradictions" existant passe de dict legacy à `TabView` schema. Reste 1 tab. Le gros de la migration 19 tabs est **Sprint 8**, pas ici. |

**Hello-world-app** (`examples/hello-world-app/`) :

| Fichier | Rôle Sprint 6 |
|---|---|
| `hello.py` | Le tab "Hello" passe à `TabView` schema. Reste 45 LOC cible. |

### 1.3 Libs consultées via context7 (récap, détails §2)

| Lib | ID context7 | Version confirmée | Usage Sprint 6 |
|---|---|---|---|
| Vitest | `/vitest-dev/vitest` | 3.x stable (4.x disponible) | D4 — unit tests |
| @testing-library/react | (séparé) | latest | D4 — helpers render/screen |
| cmdk | `/dip/cmdk` | 1.1.1 (déjà dans package.json) | Phase C — palette |
| size-limit | crates.io / npm | à fixer dans plan | D4 — bundle CI guard |

---

## 2. Décisions Day 0 (D1..D5 — gelées après validation §4 kickoff)

### D1 — Vocabulaire `TabView` custom minimal

**Retenu** : 11 block types discriminés (kind string-literal),
pas de `@rjsf/shadcn`.

**Raisons documentées** :
- View-centric, pas form-centric. Les tabs gov sont
  majoritairement des dashboards lecture-seule ; RJSF est taillé
  pour des formulaires d'édition.
- Zéro nouvelle dep npm (minzip ~0 pour le renderer, le bundle
  grows only via le code qu'on écrit nous-mêmes).
- Compatibilité Tailwind 4 immédiate (RJSF shadcn ship une
  config `content:` Tailwind 3 qui casse avec Tailwind 4 CSS-first).
- Contrôle total des deux côtés (producer Python, consumer React)
  donc pas de risque de drift avec un upstream.
- Bundle impact mesurable : ~300-400 LOC React renderer, ~15-20 KB
  brut avant gzip. Target : main ≤ 475 KB (budget D5).

**Schéma TypeScript exact (figé)** :

```ts
type BlockTone = "neutral" | "ok" | "warn" | "danger";

type TabBlock =
  | { kind: "section"; title?: string; blocks: TabBlock[] }
  | { kind: "heading"; level: 1 | 2 | 3; text: string }
  | { kind: "text"; text: string; muted?: boolean }
  | { kind: "kv"; items: { label: string; value: string | number; hint?: string }[] }
  | { kind: "metric"; label: string; value: string | number; delta?: number; unit?: string; tone?: BlockTone }
  | { kind: "table"; columns: { key: string; label: string; align?: "left" | "right" | "center" }[]; rows: Record<string, string | number | null>[]; empty_text?: string }
  | { kind: "badge_list"; items: { label: string; tone?: BlockTone }[] }
  | { kind: "button"; label: string; action: { kind: "route"; path: string } | { kind: "task_submit"; worker: string; payload: unknown } }
  | { kind: "chart_line"; label: string; points: { x: string; y: number }[]; y_unit?: string }
  | { kind: "chart_bar"; label: string; bars: { label: string; value: number; tone?: BlockTone }[] }
  | { kind: "empty"; text: string };

type TabView = {
  schema_version: 1;
  tab_name: string;
  title?: string;
  blocks: TabBlock[];
};
```

**Pydantic mirror (figé côté Python)** : `packages/nexus-sdk/src/nexus_sdk/view.py`
expose le même schéma via Pydantic 2 discriminated unions
(`Annotated[Union[...], Field(discriminator="kind")]`). Les
helpers `section(...)`, `metric(...)` etc. retournent des
instances validées puis `.model_dump()` pour le JSON.

**Charts** : `chart_line` et `chart_bar` ne tirent **aucune**
lib de charting (pas de recharts, d3, victory). Rendu = SVG
inline maison, ~100 LOC pour les deux types. Axes et labels
minimalistes, dark-theme natif via classes Tailwind. Si un
tab gov a besoin d'un chart plus élaboré, c'est v1.2.

### D2 — Versioning & contrat cross-langue

- `schema_version: 1` littéral obligatoire sur chaque `TabView`.
  Le renderer React **rejette** tout payload sans ce champ ou
  avec une valeur ≠ 1 (affichage : bloc `empty` "schéma
  incompatible").
- Source de vérité côté Python : `nexus_sdk.view.TabView` Pydantic.
- Source de vérité côté TypeScript : `web/src/api/coordinator.ts`
  (nouveau schéma Zod `TabViewSchema`).
- Les deux schémas sont frozen dans le même commit Phase A. Toute
  bump cross-langue = commit séparé qui touche les deux fichiers
  atomiquement.
- Un test existe côté Python (`test_view_schema_matches_zod`) qui
  lit le schéma Zod via un snapshot commit (dumpé dans
  `packages/nexus-sdk/tests/snapshots/tabview_schema.json`) et
  échoue si divergence — snapshot regénéré manuellement à
  chaque bump volontaire.

### D3 — Portée renderer (scope strict)

- Les 5 tabs natifs `ProjectDetail.tsx` (Overview, Tasks, Kudos,
  Invites, Apps) **restent hard-coded**. Ils consomment des
  APIs coordinator natives, pas des descriptors d'apps.
- Le renderer `<TabViewRenderer>` s'applique **uniquement** au
  contenu de `AppsTab.tsx` quand un descriptor est fetché via
  `GET /app/{name}/tabs/{tab_name}/descriptor`.
- Fallback legacy : si un descriptor ne valide pas le Zod
  `TabViewSchema`, afficher un bloc `empty` "Descriptor legacy
  non porté" + le `<pre>JSON.stringify</pre>` en toggle
  repliable. Cette fallback existe pour une release seulement
  — retirée au Sprint 7 ou Sprint 8 selon l'état de
  `nexus-app-gov`.

### D4 — Portée Vitest Sprint 6

- Fichiers testés : **3 uniquement**.
  - `src/lib/format.ts` (75 LOC, cibles : les 4 helpers purs avec
    cas null/undefined + boundaries)
  - `src/stores/projectStore.ts` (152 LOC, cibles : add/remove/
    setActive/updateCoordinator/clear, persist localStorage mock)
  - `src/components/app/tabview/TabViewRenderer.tsx` (nouveau
    Phase B, cibles : chaque kind de block + fallback + schéma
    invalid)
- Objectif : **≥95% line coverage** sur ces 3 fichiers. Mesuré
  avec `vitest --coverage` (v8 provider).
- Run : `<2 s` end-to-end en CI (pas de subprocess, pas de
  jsdom page load).
- Pas de tests composants pour les 5 tabs natifs (Playwright
  les couvre déjà).
- Pas de tests pour `AppsTab.tsx` en dehors du renderer (Playwright
  apps-tab-render.spec.ts couvre le happy path).

### D5 — Bundle budget + Ctrl+K trigger

**Budget CI** (échec si dépassé) :

| Asset | Limite | Headroom vs Sprint 5 |
|---|---|---|
| `index-*.js` (main) | ≤ **475 KB** | +50 KB (Sprint 5 à 425 KB) |
| `vendor-react-*.js` | ≤ **210 KB** | +20 KB (190 KB) |
| `index-*.css` | ≤ **100 KB** | +10 KB (90 KB) |
| vendor-ui (Radix / base-ui) | ≤ **110 KB** | nouveau split |

**Outil** : `size-limit` avec preset `@size-limit/file` (pas
`preset-app` qui simule un runtime). Config dans `web/.size-limit.json`.
Run : `npm run size` wrapping `size-limit`, invoqué en CI juste
après `npm run build`.

**Trigger Ctrl+K** :
- Keybind : `Ctrl+K` (Win/Linux) ou `Cmd+K` (macOS) — détecté
  via `e.ctrlKey || e.metaKey`.
- Listener global : installé dans `AppShell.tsx` avec
  `useEffect` + `document.addEventListener("keydown", ...)`.
- Close : `Escape` ou clic hors dialog.
- Groupes initiaux : **Navigation** (Mes projets, Mes workers,
  Browse, Curators), **Projets** (un item par coordinator
  enregistré, navigue vers /project/:name), **Actions** (Ajouter
  un coordinator, Actualiser). Sprint 7 ajoutera "Subscribe to
  curator list", Sprint 8 ajoutera des actions app-driven.

---

## 3. Research consulté (détails)

### 3.1 Vitest 3.x / 4.x pour Vite 6 + React 19 + TS strict

Source : context7 `/vitest-dev/vitest` (benchmark 10.0).

- `defineConfig` importé depuis `vitest/config`, **pas** depuis
  `vite`. Fusion avec `vite.config.ts` existant via `mergeConfig`
  de vite si besoin.
- Environment `jsdom` nécessaire pour `projectStore.ts` (qui
  utilise `localStorage` via Zustand `persist`). Environment
  `node` suffit pour `format.ts`.
- `@testing-library/react` + `@testing-library/jest-dom` +
  `@testing-library/user-event` doivent être installés
  séparément — non fournis par Vitest.
- Pattern localStorage mock via `Object.defineProperty(window,
  "localStorage", { value: ... })` en `beforeEach` + reset store
  via `useStore.persist.clearStorage()` en `afterEach`.
- `clearMocks: true` + `restoreMocks: true` dans la config pour
  isolation par défaut.

**Pas de blocker**. Installation : `npm i -D vitest @vitest/coverage-v8
@testing-library/react @testing-library/jest-dom
@testing-library/user-event jsdom`.

### 3.2 cmdk + shadcn Command

Source : context7 `/dip/cmdk`, reconnaissance
`web/src/components/ui/command.tsx:1-195` (existe déjà, non utilisé).

- shadcn v4 `<CommandDialog>` wrap déjà cmdk 1.1.1 (présent
  `web/package.json:30`). Rien à installer.
- Pattern keyboard handler global documenté, utiliser `useEffect`
  dans `AppShell.tsx` ou dans un hook `useCommandPalette()`
  (préférence : hook, pour faciliter le test Vitest si besoin —
  note : Vitest D4 n'inclut pas ce hook, Playwright le couvrira).
- Navigation : `useNavigate()` de React Router 7, identique à
  Sprint 5.
- **Attention pattern P2 PATTERNS.md** : cmdk ne traverse pas base-ui
  `render` vs Radix `asChild` — c'est une primitive à part.
  Pas d'impact ici.

### 3.3 size-limit pour Vite

Source : docs GitHub `ai/size-limit` (via WebFetch si context7
manque). Preset `@size-limit/file` mesure les fichiers tels
quels en sortie de `vite build`, sans simuler de runtime. C'est
ce qu'on veut — on compare des bytes, pas une simulation.

Config cible `web/.size-limit.json` (format JSON, pas JS, pour
éviter un `require()` dans un projet ESM-only) :

```json
[
  { "name": "main",       "path": "dist/assets/index-*.js",        "limit": "475 KB" },
  { "name": "vendor-react","path": "dist/assets/vendor-react-*.js","limit": "210 KB" },
  { "name": "vendor-ui",  "path": "dist/assets/vendor-ui-*.js",    "limit": "110 KB" },
  { "name": "css",        "path": "dist/assets/index-*.css",       "limit": "100 KB" }
]
```

Script `web/package.json` : `"size": "size-limit"`.

**Pas de blocker**. Installation : `npm i -D size-limit @size-limit/file`.

### 3.4 Charts SVG maison (pas de dep)

Pas de source externe — la règle est de ne **pas** réintroduire
recharts / d3 / victory (tous retirés Sprint 5 Day 0). Implémentation :

- `chart_line` : SVG inline, normalisation min/max auto, path
  `<polyline>` + axe X (labels espacés), axe Y (3 ticks auto),
  ~60 LOC.
- `chart_bar` : SVG inline, `<rect>` par barre, couleur par
  `tone`, ~40 LOC.
- Responsive : `viewBox` fixe + `width="100%"`, style flex via
  Tailwind sur le wrapper.

Test Vitest : snapshot du SVG rendu pour 3 datasets fixes
(empty, 1 point, 10 points).

---

## 4. Phase A — SDK view module + coordinator wiring

**Commit cible** : `feat(sdk,coordinator): Sprint 6 Phase A — TabView
schema + coordinator validation`

**Branche de travail** : master direct (pas de branche sprint —
cohérent avec Sprint 4 et 5).

### 4.1 Fichiers ajoutés

- `packages/nexus-sdk/src/nexus_sdk/view.py` (≈ 250 LOC)
  - Enum `BlockKind`, `BlockTone`
  - `TabBlockSection`, `TabBlockHeading`, `TabBlockText`,
    `TabBlockKV`, `TabBlockMetric`, `TabBlockTable`,
    `TabBlockBadgeList`, `TabBlockButton`, `TabBlockChartLine`,
    `TabBlockChartBar`, `TabBlockEmpty` — Pydantic 2 BaseModels
    avec `kind: Literal["..."]`
  - `TabBlock = Annotated[Union[...], Field(discriminator="kind")]`
  - `TabView` model avec `schema_version: Literal[1] = 1`,
    `tab_name: str`, `title: Optional[str]`, `blocks: list[TabBlock]`
  - Helpers constructeurs : `section()`, `heading()`, `text()`,
    `kv()`, `metric()`, `table_()`, `badge_list()`, `button_route()`,
    `button_task()`, `chart_line()`, `chart_bar()`, `empty()`
  - Docstring 1 ligne par helper (WHY uniquement, PATTERNS convention)

- `packages/nexus-sdk/tests/test_view.py` (≈ 150 LOC)
  - Construction de chaque block type + `.model_dump()` matches
    le JSON attendu
  - `TabView` valide schema_version=1, rejette schema_version=2
  - Helpers : 1 test par constructeur

- `packages/nexus-sdk/tests/snapshots/tabview_schema.json` (figé)
  - Dump JSON-schema généré par Pydantic 2 `model_json_schema()`
  - Test `test_view_schema_stable` compare au snapshot et échoue
    si le schéma drifte involontairement

### 4.2 Fichiers modifiés

- `packages/nexus-sdk/src/nexus_sdk/__init__.py`
  - Ajout `from . import view`
  - Export de `TabView`, `TabBlock` et des helpers

- `packages/nexus-coordinator/src/nexus_coordinator/api/apps.py`
  - La route `GET /app/{name}/tabs/{tab_name}/descriptor` valide
    désormais le retour contre `TabView.model_validate(...)`.
  - En cas d'échec de validation : renvoyer 200 avec
    `{"descriptor": <dict original>, "legacy_descriptor": true}`
    (D3 fallback). L'erreur de validation est loggée en WARNING.
  - En cas de succès : renvoyer `{"descriptor": <validated>.model_dump(),
    "legacy_descriptor": false}`.

- `packages/nexus-coordinator/tests/test_app_tab_descriptor.py`
  - Test existant (`test_async_descriptor_returns_result`) mis à
    jour pour refléter la nouvelle shape `{descriptor, legacy_descriptor}`.
  - Nouveau test `test_schema_driven_descriptor_validates` : une
    app factice retourne un `TabView` valide, la réponse contient
    `legacy_descriptor: false` et un `descriptor` qui passe le
    schéma Zod côté frontend (vérification cross-langue via
    snapshot JSON).
  - Nouveau test `test_legacy_descriptor_falls_back` : une app
    factice retourne un dict non-schema, la réponse contient
    `legacy_descriptor: true` et préserve le dict tel quel.

### 4.3 Fichiers inchangés (rappel)

- `packages/nexus-sdk/src/nexus_sdk/app.py` — ABC NexusApp
  inchangé
- `packages/nexus-sdk/src/nexus_sdk/decorators.py` — `@nexus_tab`
  inchangé (retourne toujours dict, mais les apps écrivent
  maintenant des `.model_dump()` de `TabView`)

### 4.4 Critères d'acceptation Phase A

- [ ] `uv run --package nexus-sdk pytest packages/nexus-sdk/tests/ -q` → vert
  (≥ 6 tests sdk total, dont nouveaux `test_view` + snapshot)
- [ ] `uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/ -q` → ≥ 45 passed + 1 skipped
  (+ 2 tests nouveaux par rapport à Sprint 5)
- [ ] `ruff check packages/nexus-sdk/ packages/nexus-coordinator/` → clean
- [ ] Schéma Pydantic JSON dumpé dans snapshot, test snapshot passe

---

## 5. Phase B — Web renderer + port hello-world + gov

**Commit cible** : `feat(web,sdk,app-gov,examples): Sprint 6 Phase B — TabView
renderer + hello-world + gov schema port`

### 5.1 Fichiers ajoutés (web)

- `web/src/components/app/tabview/types.ts` (≈ 100 LOC)
  - Type TS exact `TabView` + `TabBlock` + sous-types (cf §2 D1)

- `web/src/components/app/tabview/schema.ts` (≈ 150 LOC)
  - Schéma Zod `TabViewSchema` qui miroir `types.ts`
  - Discriminated union via `z.discriminatedUnion("kind", [...])`
  - Export `parseTabView(raw: unknown): { ok: true; value: TabView } | { ok: false; error: string }`

- `web/src/components/app/tabview/TabViewRenderer.tsx` (≈ 50 LOC)
  - Composant principal : reçoit `tabView: TabView`, render
    `<TabBlockRenderer>` pour chaque bloc
  - Gère le wrapper titre + padding

- `web/src/components/app/tabview/TabBlockRenderer.tsx` (≈ 200 LOC)
  - Switch exhaustif sur `block.kind` (TypeScript never-check
    à la fin pour garantir que tous les kinds sont couverts)
  - Un sous-composant par kind (inline ou extrait si >30 LOC)

- `web/src/components/app/tabview/blocks/Section.tsx` (≈ 20 LOC)
  - Carte shadcn avec titre optionnel + récursion sur sous-blocs

- `web/src/components/app/tabview/blocks/Metric.tsx` (≈ 30 LOC)
  - Grand chiffre + label + delta coloré (tone)

- `web/src/components/app/tabview/blocks/Table.tsx` (≈ 40 LOC)
  - `<table>` HTML natif + classes Tailwind
  - `empty_text` affiché si `rows.length === 0`

- `web/src/components/app/tabview/blocks/ChartLine.tsx` (≈ 60 LOC)
  - SVG inline avec normalisation min/max
  - 3 ticks Y + labels X espacés

- `web/src/components/app/tabview/blocks/ChartBar.tsx` (≈ 40 LOC)
  - SVG inline avec barres colorées par tone

- `web/src/components/app/tabview/blocks/Button.tsx` (≈ 30 LOC)
  - action.kind === "route" → `useNavigate`
  - action.kind === "task_submit" → `useMutation` via client coordinator

- `web/src/components/app/tabview/blocks/KV.tsx`, `Text.tsx`,
  `Heading.tsx`, `BadgeList.tsx`, `Empty.tsx` — ≤20 LOC chacun

- `web/src/components/app/tabview/__tests__/TabViewRenderer.test.tsx`
  (≈ 200 LOC, **D4 cible Vitest**)
  - 1 test par kind de block (11 tests)
  - 1 test fallback schéma invalid
  - 1 test schéma `schema_version !== 1`
  - 1 test recursive section

### 5.2 Fichiers modifiés (web)

- `web/src/api/coordinator.ts`
  - Ajout `import { TabViewSchema } from "../components/app/tabview/schema"`
  - Nouveau helper
    ```ts
    export async function getAppTabDescriptor(
      baseUrl: string,
      appName: string,
      tabName: string,
    ): Promise<{ descriptor: TabView | null; legacy: boolean; raw: unknown }>
    ```
  - Parse via `TabViewSchema.safeParse`, si `!success` → legacy=true, descriptor=null, raw=original

- `web/src/components/project/AppsTab.tsx`
  - Remplace le bouton "Invoquer" + `JSON.stringify` :
    - Un tab async → bouton "Invoquer" → `getAppTabDescriptor`
    - Si `!legacy` → render `<TabViewRenderer tabView={descriptor} />`
    - Si `legacy` → render bloc `<Empty text="Descriptor legacy" />`
      + toggle `<details>` qui montre le JSON brut

### 5.3 Fichiers modifiés (Python apps)

- `packages/nexus-app-gov/src/nexus_app_gov/app.py`
  - Le tab `contradiction_tab` (≈ 15 LOC aujourd'hui) réécrit
    pour retourner un `TabView` via les helpers :
    ```python
    from nexus_sdk.view import TabView, section, heading, text, metric, empty

    @nexus_tab(name="Contradictions", icon="alert-octagon")
    def contradiction_tab(self):
        return TabView(
            schema_version=1,
            tab_name="contradictions",
            title="Détection de contradictions",
            blocks=[
                heading(level=1, text="Analyse de cohérence politique"),
                text(text=POLITICAL_CONTRADICTION_PROMPT.splitlines()[0]),
                metric(label="Déclarations analysées", value=0),
                metric(label="Contradictions détectées", value=0, tone="warn"),
                empty(text="Aucune analyse en cours — soumettre un lot via /statements"),
            ],
        ).model_dump()
    ```

- `examples/hello-world-app/hello.py`
  - Tab `hello_tab` réécrit idem :
    ```python
    @nexus_tab(name="Hello", icon="sparkles")
    def hello_tab(self):
        return TabView(
            schema_version=1,
            tab_name="hello",
            title="Hello World",
            blocks=[
                heading(level=1, text="Bienvenue sur hello-world-app"),
                text(text="Première app nexus-grid portée sur le schéma TabView v1."),
                metric(label="Tâches soumises", value=0),
            ],
        ).model_dump()
    ```

### 5.4 Critères d'acceptation Phase B

- [ ] `cd web && npx tsc --noEmit -p tsconfig.app.json` → exit 0
- [ ] `cd web && npm run lint` → 0 errors (warnings T1 inchangés)
- [ ] `cd web && npm run build` → exit 0, budgets respectés (cf D5)
- [ ] `uv run --package nexus-sdk pytest packages/nexus-sdk/tests/` → vert
- [ ] `uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/` → vert
- [ ] Port manuel : `nexus-coordinator init test-gov && start test-gov` +
  `curl /app/gov/tabs/Contradictions/descriptor` renvoie `legacy=false`
  avec un TabView valide

---

## 6. Phase C — Ctrl+K command palette

**Commit cible** : `feat(web): Sprint 6 Phase C — Ctrl+K command palette`

### 6.1 Fichiers ajoutés

- `web/src/components/command-palette/CommandPalette.tsx` (≈ 150 LOC)
  - Wrap `<CommandDialog>` de `components/ui/command.tsx`
  - Groupes Navigation / Projets / Actions
  - Listener global `Ctrl+K` / `Cmd+K` via `useEffect`
  - Utilise `useProjectStore` pour peupler le groupe Projets
  - Chaque `CommandItem` → `useNavigate` ou action locale +
    `setOpen(false)`

- `web/src/components/command-palette/useCommandPalette.ts` (≈ 30 LOC)
  - Hook qui expose `{ open, setOpen, toggle }` + installe le
    listener global

### 6.2 Fichiers modifiés

- `web/src/components/AppShell.tsx`
  - Import `<CommandPalette>` et mount au niveau racine
    (hors `<Outlet>`, sibling de `<Sidebar>` / `<Header>`)
  - Ajout petit indicateur `kbd` dans le header : `⌘K` ou
    `Ctrl+K` selon l'OS détecté (navigator.platform simple check)

### 6.3 Tests Phase C

- Playwright spec `web/tests/command-palette.spec.ts` :
  - Charger `/`, presser `Control+K`, vérifier que le dialog
    s'ouvre, taper "Mes projets", presser Enter, vérifier que
    `/projects` est atteint.
  - Charger `/projects`, presser `Escape` après ouverture,
    vérifier que le dialog se ferme.

### 6.4 Critères d'acceptation Phase C

- [ ] `tsc --noEmit` clean
- [ ] `npm run lint` clean
- [ ] `npm run build` clean, budgets respectés
- [ ] Nouveau Playwright spec vert (total 9 → 10 specs)
- [ ] Test manuel : Ctrl+K ouvre, Escape ferme, navigation OK

---

## 7. Phase D — Vitest unit tests + bundle CI guard

**Commit cible** : `feat(web): Sprint 6 Phase D — Vitest unit tests + size-limit CI`

### 7.1 Fichiers ajoutés

- `web/vitest.config.ts` (≈ 30 LOC)
  ```ts
  import { defineConfig, mergeConfig } from "vitest/config";
  import viteConfig from "./vite.config";

  export default mergeConfig(
    viteConfig,
    defineConfig({
      test: {
        globals: true,
        environment: "jsdom",
        setupFiles: ["./src/test/setup.ts"],
        include: ["src/**/*.{test,spec}.{ts,tsx}"],
        exclude: ["**/node_modules/**", "**/dist/**", "tests/**"],
        clearMocks: true,
        restoreMocks: true,
        coverage: {
          provider: "v8",
          reporter: ["text", "json-summary"],
          include: [
            "src/lib/format.ts",
            "src/stores/projectStore.ts",
            "src/components/app/tabview/**",
          ],
          thresholds: { lines: 95, functions: 95, branches: 90, statements: 95 },
        },
      },
    }),
  );
  ```

- `web/src/test/setup.ts` (≈ 20 LOC)
  - `import "@testing-library/jest-dom"`
  - Helper mock localStorage

- `web/src/lib/__tests__/format.test.ts` (≈ 100 LOC)
  - 4 helpers × plusieurs cas chacun
  - Couvre null/undefined/boundary/happy path

- `web/src/stores/__tests__/projectStore.test.ts` (≈ 150 LOC)
  - add/remove/setActive/update/clear
  - persist → mock localStorage vérifie la clé `nexus-grid:shell:v1`
  - `selectActiveCoordinator` returns undefined / correct entry

- (Le test Vitest pour TabViewRenderer est déjà listé Phase B §5.1)

- `web/.size-limit.json` (cf §3.3)

### 7.2 Fichiers modifiés

- `web/package.json`
  - `"scripts"` ajout :
    - `"test:unit": "vitest run"`
    - `"test:unit:watch": "vitest"`
    - `"test:coverage": "vitest run --coverage"`
    - `"size": "size-limit"`
  - `"devDependencies"` ajout : `vitest`, `@vitest/coverage-v8`,
    `@testing-library/react`, `@testing-library/jest-dom`,
    `@testing-library/user-event`, `jsdom`, `size-limit`,
    `@size-limit/file`

- `web/eslint.config.js`
  - Ajout globals Vitest (`describe`, `it`, `test`, `expect`,
    `vi`, `beforeEach`, `afterEach`) dans la section `files:
    ["**/*.test.ts", "**/*.test.tsx"]`

- `web/.gitignore`
  - Ajout `coverage/`

### 7.3 Critères d'acceptation Phase D

- [ ] `cd web && npm run test:unit` → tous verts, <2 s
- [ ] `cd web && npm run test:coverage` → thresholds respectés
  (≥95% lines sur les 3 fichiers cible)
- [ ] `cd web && npm run build && npm run size` → exit 0, aucun
  asset ne dépasse son budget D5
- [ ] `cd web && npm run lint` → clean (les globals Vitest sont
  reconnus dans les fichiers de test)
- [ ] Playwright inchangé, toujours vert

---

## 8. Phase E — Polish, Playwright, verification doc

**Commit cible** : `feat(web): Sprint 6 Phase E — polish, Playwright,
verification`

### 8.1 Playwright additions

- `web/tests/tabview-schema-driven.spec.ts` (nouveau) :
  - Démarrer le coordinator (déjà fait par globalSetup)
  - Naviguer vers `/project/test-gov`
  - Cliquer le tab "Apps", puis "Invoquer" sur le tab Contradictions
  - Vérifier que le renderer TabView affiche un `<h1>` "Analyse
    de cohérence politique", 2 `<Metric>`, 1 bloc empty
  - Pas de JSON `<pre>` visible

- `web/tests/command-palette.spec.ts` (déjà listé Phase C)

### 8.2 Docs

- `docs/shell/PATTERNS.md` :
  - **Nouveau P8 — TabView est le seul contrat renderer pour les
    tabs d'apps**. Règle : tout tab retourné par une app via
    `@nexus_tab` DOIT retourner un `TabView` schema_version=1.
    Les 5 tabs natifs de ProjectDetail sont hors périmètre.
  - **T1** : inchangé (accepté)
  - **T2** : **FERMÉ Sprint 6 Phase D** — annoter avec commit SHA
  - **T3** : **FERMÉ Sprint 6 Phase D** — annoter avec commit SHA

### 8.3 Verification document

- `.planning/sprint6_verification.md` — même format Sprint 5 :
  - How to re-run (commandes)
  - Checklist fail-fast (table, cf §9)
  - Summary par phase avec commits
  - Scope cuts respectés

### 8.4 Scan EN strings

- Run `bash web/scripts/scan-en-strings.sh` avant commit.
  Tous les nouveaux strings Phase C (groupes palette) en
  français.

### 8.5 Commit final

```
feat(web): Sprint 6 Phase E — stubs, polish, verification

- Playwright tabview-schema-driven spec
- P8 + T2/T3 closed in docs/shell/PATTERNS.md
- .planning/sprint6_verification.md checklist 24/24
- French-only scan clean
```

---

## 9. Fail-fast checklist (cible Sprint 6)

Mirror de `.planning/sprint5_verification.md` mais avec **24
rows** (ajout de 2 Vitest + 1 size + 1 TabView schema cross-langue).

| # | Check | Commande | Critère |
|---|---|---|---|
| 1 | SDK `view` module importable | `uv run python -c "from nexus_sdk.view import TabView, section, metric"` | exit 0 |
| 2 | Pydantic TabView validates schema_version=1 | `pytest packages/nexus-sdk/tests/test_view.py::test_tabview_requires_schema_version_1` | pass |
| 3 | Pydantic rejects schema_version=2 | `pytest packages/nexus-sdk/tests/test_view.py::test_tabview_rejects_unknown_schema_version` | pass |
| 4 | SDK snapshot stable | `pytest packages/nexus-sdk/tests/test_view.py::test_view_schema_stable` | pass |
| 5 | Coordinator validates TabView | `pytest packages/nexus-coordinator/tests/test_app_tab_descriptor.py::test_schema_driven_descriptor_validates` | pass |
| 6 | Coordinator falls back legacy | `pytest packages/nexus-coordinator/tests/test_app_tab_descriptor.py::test_legacy_descriptor_falls_back` | pass |
| 7 | All SDK tests | `uv run --package nexus-sdk pytest packages/nexus-sdk/tests/ -q` | ≥ 10 passed |
| 8 | All coordinator tests | `uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/ -q` | ≥ 45 passed + 1 skip |
| 9 | All app-gov tests | `uv run pytest packages/nexus-app-gov/tests/ -q` | all pass |
| 10 | All Rust tests unchanged | `cargo test --workspace --exclude nexus-core-py --locked` | ≥ 193 |
| 11 | cargo fmt + clippy | `cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| 12 | ruff format + check | `uv run ruff format --check packages/ examples/ && uv run ruff check packages/ examples/` | exit 0 |
| 13 | tsc strict | `cd web && npx tsc --noEmit -p tsconfig.app.json` | exit 0 |
| 14 | ESLint | `cd web && npm run lint` | 0 err, 5 T1 warnings |
| 15 | Vite build | `cd web && npm run build` | exit 0, no warnings |
| 16 | size-limit budgets | `cd web && npm run size` | all assets within budget D5 |
| 17 | Vitest unit tests run | `cd web && npm run test:unit` | all pass, <2 s |
| 18 | Vitest coverage thresholds | `cd web && npm run test:coverage` | lines ≥95%, funcs ≥95%, branches ≥90% |
| 19 | TabView renderer covers 11 kinds | (inside test) | 11 tests pass |
| 20 | Command palette Ctrl+K | Playwright `command-palette.spec.ts` | pass |
| 21 | TabView schema-driven e2e | Playwright `tabview-schema-driven.spec.ts` | pass |
| 22 | All Playwright | `cd web && npx playwright test` | **≥ 10 passed** |
| 23 | French-only | `bash web/scripts/scan-en-strings.sh` | exit 0 |
| 24 | PATTERNS.md P8 + T2/T3 closed | `grep -q "P8 — TabView" docs/shell/PATTERNS.md && grep -q "FERMÉ Sprint 6" docs/shell/PATTERNS.md` | exit 0 |

---

## 10. Git plan

9 commits cibles (même cadence Sprint 5), tous sur master :

1. `docs(sprint6): kickoff + detailed plan` — kickoff.md + plan.md
2. `feat(sdk,coordinator): Sprint 6 Phase A — TabView schema + coordinator validation`
3. `feat(web,sdk,app-gov,examples): Sprint 6 Phase B — TabView renderer + hello-world + gov schema port`
4. `feat(web): Sprint 6 Phase C — Ctrl+K command palette`
5. `feat(web): Sprint 6 Phase D — Vitest unit tests + size-limit CI`
6. `feat(web): Sprint 6 Phase E — polish, Playwright, verification`

*(Si une phase nécessite un fix post-coup, commit séparé `fix(...)` entre la phase et la suivante — comme Sprint 2 `de9589d` + `ed2ea76`.)*

## 11. Scope cuts (à respecter strictement)

- **Pas de port des 5 tabs natifs** (Overview/Tasks/Kudos/Invites/
  Apps) sur TabView — D3 figé
- **Pas de charts D3 / recharts / victory** — SVG inline ou rien (D1)
- **Pas de `@rjsf/shadcn`** — D1 figé
- **Pas de tests Vitest pour composants hors des 3 cibles D4**
- **Pas de `nexus-shell-daemon`** — c'est Sprint 7
- **Pas de curator list / DHT / gossip** — c'est Sprint 7
- **Pas d'extension SDK `AppContext.storage` / `AppContext.events`
  / file upload** — c'est Sprint 8 Phase A
- **Pas de port d'un tab gov réel autre que Contradictions** — Sprint 8
- **Pas de mobile responsive < 1280px**
- **Pas de dark/light theme switch** — Sprint 5 a figé dark-only

## 12. Risks

- **R1 — Tailwind 4 + @rjsf/shadcn** : mitigé D1 (on n'utilise pas RJSF).
- **R2 — TabView vocabulary insuffisant pour gov** : mitigé §3 Sprint 8
  (scope cut Reseau graph + Carte Leaflet). Sprint 6 valide sur
  Contradictions, si un autre tab gov force une extension ce sera
  v1.1 du schéma, commit séparé Sprint 7 ou 8.
- **R3 — Budget bundle dépassé après ajout TabView + cmdk runtime** :
  mitigé D5 (budget +50 KB explicite). Si dépassé, analyse
  `size-limit --why` et scope cut (inline SVG charts en particulier).
- **R4 — Vitest coverage thresholds trop stricts** : mitigé D4
  (portée réduite à 3 fichiers). Si un edge case bloque, abaisser
  le threshold documenté dans ce plan plutôt que sauter le test.
- **R5 — Command palette navigate conflicts avec React Query cache** :
  Sprint 5 P4 oblige React Query pour tous les fetchs ; la
  palette ne fait que `useNavigate`, zéro fetch direct. Pas de risque.

## 13. Checkpoint de clôture Sprint 6

Sprint 6 est **fermé** quand :
1. Checklist §9 24/24 verte
2. `git log --oneline master ^cdf4467` affiche 6-9 commits
3. `.planning/sprint6_verification.md` commité et lisible
4. `docs/shell/PATTERNS.md` contient P8 et marque T2 + T3 fermés
5. Aucun TODO(Sprint6) dans le code (`grep -r 'TODO(Sprint6)'
   | wc -l` = 0)

Après fermeture : kickoff + plan Sprint 7 (P2P Discovery Layer)
rédigés dans `.planning/sprint7_kickoff.md` avant tout nouveau
commit code.
