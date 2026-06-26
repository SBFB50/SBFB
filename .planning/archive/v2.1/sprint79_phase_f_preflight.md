# Sprint 79 — Phase F preflight (G8) : pack daisyUI knowledge + extension prompt-kind

**Méthode** : Workflow ultracode (run `wf_ae94856a-ac4`), fan-out 5 scans factuels
(S1a OSS/CSP daisyUI·Tailwind 4 · S1b deps/CVE · S2 décisions historiques + cadence
docs-contrat · S3 threat model CSP-par-classe · S4 structure miroir + MANIFEST blake3)
+ vérification adversariale (11 verdicts ; 1 vérificateur redondant a échoué au cap
StructuredOutput, sans impact couverture). 21 agents, ~1.48M tokens, modèle forcé
`claude-opus-4-8[1m]`.

## Verdict : PLAN-ADAPT

Aucune décision Day-0 figée (D1/D4/D7/D9) n'est contestée → **pas de DESIGN-CONFLICT,
pas d'arbitrage PO**. Le scope F (promotion + complétion + câblage du pack daisyUI, déjà
existant à 7 fichiers) tient. Mais le code doit suivre **l'approche corrigée** ci-dessous
(evidence repo concrète, citée fichier:ligne) plutôt que la lettre du plan, sur :
3 corrections factuelles + 3 livrables implicites obligatoires (modèle Phase A `9297f08`)
+ 1 contrainte de séquençage gate + 2 alignements d'hygiène.

Signaux scans : S1a PLAN-ADAPT · S1b EXECUTE · S2 PLAN-ADAPT · S3 PLAN-ADAPT · S4 PLAN-ADAPT.

---

## A. Corrections factuelles (un fait faux ancré dans une primitive de FRONTIÈRE = inacceptable)

### A1 — « 8 thèmes built-in » est FAUX → daisyUI 5.5.23 = **35 thèmes built-in** (verdict CONFIRMED)
- Le plan dit « template lean sans 8 thèmes built-in » (`sprint79_plan.md:339` livrable F,
  `:361` livrable G). daisyUI 5.5.23 expose **35** thèmes (triple preuve :
  `theming.json:37-73` array `builtin_themes` len==35 ; `README.md:19` « 35 thèmes built-in » ;
  `docs-llms.txt:99` liste upstream = 35). Le chiffre « 8 » n'apparaît nulle part dans le pack
  comme nombre de thèmes (grep `\b8\b` = couleurs sémantiques / zones hover-3d / color-mix 8%).
- **Formulation correcte à écrire dans le prompt + le knowledge** : « template *lean* qui
  n'active **aucun** des 35 thèmes built-in (`@plugin "daisyui" {}` sans liste `themes:`) ;
  seul le thème custom vendoré `sbfb-reflect` (oklch dark, `@plugin "daisyui/theme"`) est
  compilé. » Ne JAMAIS propager « 8 ».
- Concerne **F** (livrable 4 prompt) ; **G** (livrable template `src/input.css`) héritera la
  même correction — à signaler au preflight G.

### A2 — « fill-*/stroke-* ne compilent pas dans l'iframe » est IMPRÉCIS → REFUTÉ empiriquement (verdict REFUTED)
- Test empirique exécuté avec le binaire vendoré du repo (tailwindcss 4.3.1 + daisyUI 5.5.23) :
  `fill-primary stroke-secondary fill-current fill-none` **se compilent** en CSS statique
  (`.fill-primary{fill:var(--color-primary)}`, `.fill-current{fill:currentcolor}`, …),
  pur CSS sans fetch → 100% CSP-safe. Le gate Factory (`gates.rs:273-288`) ne flague JAMAIS
  `fill:`/`stroke:` — seulement les ressources distantes `url(http|//)`.
- **Vrai motif d'éviter de s'y fier** : (a) pas de build Tailwind **au runtime** dans l'iframe
  scellée (le qualificatif load-bearing déjà présent `prompts/agent/app-authoring.md:64-68`
  « no in-iframe Tailwind build ») ; (b) **purge** — une classe non vue par `@source`/composée
  en JS est éliminée. Remède = peindre via `fill: var(--color-*)` / `currentColor` /
  `stroke: color-mix(in oklch, …)` pour robustesse-à-la-purge + theme-awareness — **pas**
  parce qu'« elles ne compilent pas ».
- **Action promotion** : `theming.json:79` est déjà correct (« souvent purgés/non émis »).
  À corriger lors du move : `components.json:233,574,696` et `README.md:35` qui portent encore
  « ne compilent pas » → ajouter le qualificatif (no-runtime-build + purge).

### A3 — Taxonomie CSP par classe : `@apply`/`backdrop-filter` ne sont PAS des risques réseau (verdict PARTIAL, sur le fond CONFIRMED)
- Le plan liste `url()/@apply/backdrop-filter/mask/SVG-fill` en slash-list « cas à risque »
  (`sprint79_plan.md:333-335`). Sous `connect-src 'none'` :
  - **`@apply`** = directive compile-time, résolue au build, absente du runtime → **SÛR**.
  - **`backdrop-filter`** = composite GPU, non soumis à default-src, ne traverse pas l'origine
    opaque → **SÛR (perf-only)**. (`synthesis.json:10,31` le dit déjà « confusion possible
    avec un risque de fuite … coût perf, pas un blocage CSP ».)
  - **`mask`** = `mask-image:data:` inline → **SÛR** (autorisé par `data:` dans default-src).
  - **Catégorie réseau-exfil RÉELLE** = `url(remote)` sous TOUTE propriété CSS :
    `background-image`/`mask-image`/`border-image`/`cursor`/`list-style-image`/`content` +
    `@font-face src:url(remote)` + `@import(remote)` + SVG `fill="url(https://…#id)"` /
    `<use href="https://…">`. La règle générique `gates.rs:288` les couvre toutes, mais le
    **verdict-par-classe** de `classes-bank.json` doit les **énumérer explicitement**.
- La matière correcte EXISTE déjà dans `synthesis.json` (csp_ruleset/risk_classes/tailwind_gotchas
  bien séparés) → le travail F = **transcrire** cette taxonomie correcte dans `classes-bank.json`,
  pas la re-concevoir ; et **ne pas** recopier la slash-list indifférenciée du plan.

---

## B. Livrables implicites OBLIGATOIRES (modèle Phase A `9297f08`, cadence docs-contrat §6.12)

La couche « knowledge » est une primitive de FRONTIÈRE : son **étiquette générée** = le
`MANIFEST.json` à hash-par-couche (`PATTERNS §P70:3854` « knowledge-pack MANIFEST … S79 A; F to
come »). Phase A a livré 5 artefacts pour animejs ; F doit miroiter **à l'identique** :

### B1 — MANIFEST.json recalculé couvrant TOUTES les couches promues (verdict CONFIRMED) — BLOQUANT test
- Le MANIFEST source daisyui (`examples/.../knowledge/daisyui/MANIFEST.json:29-33`) ne hashe que
  **3 fichiers** (components.json/theming.json/synthesis.json) sur **6** présents (manquent
  COMPONENTS.md, docs-llms.txt, README.md) et **n'a pas** de champ `freshness`. C'est le carry
  explicite « couverture partielle 3/4 → Phase E » du body `9297f08`, **non clos** au tip.
- Le test miroir exige `assert_eq!(computed_keys, expected_keys)` (couverture EXACTE,
  `animejs_manifest.rs:70-77`) + `freshness` présent (`:91-94`). → recalculer le MANIFEST sur
  **tous** les fichiers non-dot/non-MANIFEST sous `docs/factory/knowledge/daisyui/` (les 6 +
  `classes-bank.json` si créé), ajouter `freshness{snapshot_date,source_ref,refresh}` +
  `hash_convention`, convention `blake3(file_bytes).to_hex()[..16]`. Corriger aussi le drift
  doc `README.md:22` (prétend « hashes par couche » alors qu'il n'y en a que 3).

### B2 — Test hermétique Rust `crates/sbfb-factory/tests/daisyui_manifest.rs` (verdict PARTIAL→ substance confirmée)
- N'existe pas (grep `daisyui_manifest` = vide). Miroir de `animejs_manifest.rs` :
  recompute blake3[..16] par couche + coverage exacte + **garde anti-CR** (`!bytes.contains(b'\r')`)
  + sanity version `daisyui 5.5.23` + `freshness` présent. = **test #1 du delta +2**.
- (Rectif rationale : la garde CR n'a PAS « fait échouer la review Phase A » — Phase A review =
  PASS, CR-findings P3 faux-positifs Git Bash. C'est une garde **préventive**.)

### B3 — `.gitattributes` du pack (verdict PARTIAL : parité, pas nécessité hash)
- Le pack daisyui source n'a pas de `.gitattributes` ; animejs en a un (`* text eol=lf -whitespace`).
  **Nuance prouvée** : le `.gitattributes` racine couvre DÉJÀ `eol=lf` pour `.json/.md/.txt`
  (`git check-attr` aux chemins de destination → `eol: lf` sans fichier par-pack) → la portabilité
  du hash n'est PAS à risque. L'unique delta d'un fichier dédié = `whitespace` unset.
  **Décision** : créer `docs/factory/knowledge/daisyui/.gitattributes` pour **parité structurelle
  exacte** avec animejs (ceinture+bretelles, coût nul), pas pour corriger un risque CI.

### Test #2 du delta +2 — marqueurs daisyUI dans le prompt
- Le test existant `process.rs:944-971 app_authoring_prompt_surfaces_csp_markers` n'a que 5
  marqueurs **anime-only**. → ajouter un **nouveau** test dédié
  `app_authoring_prompt_surfaces_daisyui_markers` (marqueurs : `source(none)`/`@source`,
  `sbfb-reflect`, `35` thèmes / aucun built-in, `fill: var(--color`/`currentColor`,
  `--minify`). = **test #2**, garde le test anime intact, honore le delta +2.

---

## C. Contrainte de séquençage (gate volet 4 `check-frontier-contracts.sh`, BLOQUANT commit+CI) (verdict CONFIRMED)
- Le volet 4 est **fiche-driven** : `app-authoring.md` est déjà en scope (référence
  `docs/factory/knowledge/animejs`). 4a (`:178-184`) : tout token 16-hex de la fiche doit être
  dans l'**union des MANIFEST sous `docs/factory/knowledge/`** (le MANIFEST sous `examples/` est
  **invisible**). 4b (`:188-194`) : tout chemin `docs/factory/knowledge/…{json,md,ts}` cité doit
  exister sur disque (regex EXCLUT `.txt` → `docs-llms.txt` non gate-vérifié comme path).
- **Ordre impératif dans le commit F** : (1) git mv + recalcul MANIFEST sous
  `docs/factory/knowledge/daisyui/` → (2) PUIS écrire les hashes/chemins daisyui dans
  `app-authoring.md`. Citer un hash daisyui avant promotion = gate ROUGE.
- (Démenti d'une prémisse : le gate n'EXIGE pas un MANIFEST « du seul fait de la promotion » —
  il n'existait même pas à Phase A `9297f08`, né en Phase B `b27079c` + volet 4 `2447f30`. Mais
  les livrables 3+4 déjà planifiés doivent être cohérents pour ne pas casser 4a/4b.)

## D. Hygiène de promotion (alignement source unique D4) (verdict CONFIRMED, décoratif)
- `synthesis.json:4` re-cite la policy CSP contiguë (cohérente : les 6 `'none'` == `none_directives()`)
  mais omet `frame-ancestors *` et la présente comme « la policy ». Le pack animejs promu ne cite
  jamais la chaîne complète (fragments seulement). → lors du move, faire pointer la prose CSP vers
  `BLOB_SERVE_CSP`/`csp-contract.json` (source unique D4) plutôt que ré-citer une copie divergeable.
- **Drift de chemin** : la source CSP n'est PLUS `blob_serve.rs:286` (cité plan D3/D4) mais
  `crates/nexus-core-rs/src/csp.rs:33`, re-exportée par `blob_serve.rs:284` (factorisation Phase E).
  Toute prose knowledge référençant la source doit pointer le bon chemin.

## E. classes-bank.json — décision (verdict PARTIAL)
- **Absent** partout (find = 0). Le plan le demande « si absente » → branche vraie. L'équivalent
  **fonctionnel** existe déjà dans `components.json` (68 entrées `html_example` + `sbfb_csp{usable,
  reason,risks}`). **Décision** : **créer** `classes-bank.json` dédié sur le gabarit
  `docs/factory/knowledge/animejs/examples-bank.json` (`name/source_path/primitives_used/idea/
  technique_tags/novelty_fingerprint/sbfb_reusable{ok,why}/snippet`) — banque **curatée** de blocs
  daisyUI(+anime) CSP-safe avec verdict-par-classe **explicite** (porte A3 : énumère les vecteurs
  url() ; marque @apply/backdrop-filter SÛRS). S'il est créé, il **doit** entrer dans le MANIFEST
  hashes (B1) + être couvert par le test (B2). Harmoniser/documenter `sbfb_csp` (components) vs
  `sbfb_reusable` (classes-bank) — clés distinctes assumées.

## F. Invariants confirmés (EXECUTE, ne pas sur-vérifier)
- **D7 snapshot** figé confirmé ground-truth (lockfile + node_modules) : daisyui 5.5.23,
  tailwindcss 4.3.1, @tailwindcss/cli·node·oxide 4.3.1, animejs 4.5.0.
- **D9 0-dép-runtime** tenu : daisyui/tailwind/anime en `devDependencies` build-time uniquement ;
  runtime archive = 0 dép. Scope F = knowledge + prompt + 1 test Rust ; **aucun** Cargo.toml /
  package.json runtime muté. `web/` n'utilise pas daisyui.
- **CVE** : `npm audit --omit=dev` = 0 ; 17 moderate devDeps = chaîne lighthouse/@sentry/
  @opentelemetry (build-only, jamais embarqué) ; daisyui/tailwind/anime absents. Aucune advisory
  daisyui 5.5.23 / tailwindcss 4.3.1 (la glob-injection est 3.x).
- **0 bump wire** : grep `FEED_FORMAT_VERSION|_ANNOUNCEMENT_VERSION|FeedEntry` sur les 2 packs = 0.
- **D1 provenance INCHANGÉ** : `compute_output_hash` est privée, tree-walk un workspace d'app au
  publish, jamais `docs/`. Le mot « vérifiable par provenance » (plan F:336) est trompeur — hériter
  le PLAN-ADAPT Phase A : mécanisme réel = MANIFEST self-hash + test recompute autonome (pas
  provenance/FG8). 0 impact FG6 (hors workspace d'app).

---

## Plan F adapté (ordre d'exécution dans le commit unique)
1. `git mv` byte-identique des 7 fichiers `examples/daisyui-animejs-showcase/knowledge/daisyui/*`
   → `docs/factory/knowledge/daisyui/` (R100, comme A).
2. Corrections in-pack lors du move : A1 (35 thèmes, jamais 8), A2 (`components.json`/`README.md`
   « ne compilent pas » → qualifié), D (prose CSP `synthesis.json:4` → pointe source unique +
   chemin `csp.rs:33`).
3. Créer `docs/factory/knowledge/daisyui/classes-bank.json` (E + A3, gabarit examples-bank, verdict
   par classe explicite).
4. Créer `docs/factory/knowledge/daisyui/.gitattributes` (B3, parité).
5. Recalculer `docs/factory/knowledge/daisyui/MANIFEST.json` : hash blake3[..16] de **toutes** les
   couches présentes (7 + classes-bank.json) + `freshness` + `hash_convention` (B1).
6. Étendre `prompts/agent/app-authoring.md` avec la maîtrise daisyUI (recette build
   `tailwindcss -i src/input.css -o app.css --minify`, `@import "tailwindcss" source(none)` +
   `@source`, thème `sbfb-reflect` oklch dark, template lean = 0/35 built-in), **citant uniquement**
   des chemins/hashes `docs/factory/knowledge/daisyui/` (C, après étapes 1-5).
7. `crates/sbfb-factory/tests/daisyui_manifest.rs` (B2, test #1) + nouveau test
   `app_authoring_prompt_surfaces_daisyui_markers` dans `process.rs` (test #2). **Delta +2.**
8. `docs/factory/FACTORY_GATES.md` / PATTERNS : doc gate-spécifique différée Phase I (clôture) —
   ne pas la traiter ici (cohérent plan §3 graphe).

## Conditions de réussite (gate de testabilité)
- `cargo nextest -p sbfb-factory` vert avec les +2 tests ; `cargo fmt`/`clippy` 0.
- `bash scripts/check-frontier-contracts.sh` vert (4a/4b satisfaits par l'ordre §1-6).
- T1/T2 = Phase H (pas ici). Phase F = data/docs/prompt/test, plateforme-agnostique.

**Conclusion : EXÉCUTER Phase F selon le plan adapté ci-dessus (PLAN-ADAPT, 0 Day-0 touché).**
