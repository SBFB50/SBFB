# Sprint 69 Phase C — preflight G8

Date : 2026-05-22 | HEAD : `1edaaa6` | Verdict : **EXECUTE plan-as-is**

---

## Memory consultation (Step 1.5)

- `feedback_approach.md` : pick deepest, research before code, no band-aids, challenger scope cuts — Phase C est un template statique HTML sans decision technique profonde, feedback note.
- `vision_model.md` : solo maintainer OpenBSD pattern — Phase C template est coherent (outil CLI, pas d'infra).
- `feedback_context7_systematic.md` : context7 obligatoire avant code touchant lib/API — Phase C ne touche aucune nouvelle lib, utilise `include_str!` embarquement compile-time existant. context7 non applicable (pas de lib tierce nouvelle).
- `feedback_kudos_non_monetary.md` : N/A (Phase C ne touche pas kudos).
- `fairness_vision.md` : N/A (Phase C ne touche pas scoring).
- Tensions plan vs memory : **aucune**.

---

## S1a — OSS prior art deep analysis

### Probleme fonctionnel exact

"How do mature OSS projects implement CLI scaffolding with multiple embedded templates and variable substitution for static web apps?"

### Projets analyses en profondeur

#### [1] project-scaffold (crates.io)
- URL : https://crates.io/crates/project-scaffold
- Description : CLI Rust scaffolding avec `{{variable}}` substitution dans fichiers et paths, types multiples (string, choice, boolean), hooks post-create, git integration.
- Pattern architectural : templates comme repertoires + fichier de config TOML, variable substitution `{{name}}` identique a SBFB Factory.
- Edge cases geres : conditional files, post-create hooks, git init.
- Verdict : ALIGNED — le pattern `include_str!` + substitution `{{name}}` de SBFB Factory est un sous-ensemble simplifie de ce pattern standard.

#### [2] cargo-scaffold (iomentum/cargo-scaffold)
- URL : https://github.com/iomentum/cargo-scaffold
- Description : scaffolding via handlebars templates, config TOML, language-agnostic.
- Pattern architectural : templates externes clones depuis git ou locaux, handlebars substitution.
- Difference : templates externes vs embarques. SBFB Factory embarque via `include_str!` pour distribution mono-binaire (decision D2 v4).
- Verdict : ALIGNED — approche mono-binaire justifiee par le contexte CLI standalone Factory.

#### [3] Vite create (npm create vite@latest)
- URL : https://vite.dev/guide/
- Description : scaffolding multi-template (react, vue, svelte, etc.) via un registre de templates embarques.
- Pattern architectural : map template_name -> fichiers, substitution package name. Selection template via flag CLI `--template`.
- Edge cases : validation template name, fallback si template inconnu, version pinning.
- Pattern multi-template : chaque template est un repertoire independant copie dans l'output. Le CLI route via un match/switch sur le nom.
- Verdict : ALIGNED — le pattern `match template_id { "static" => ..., "static-reader" => ... }` de SBFB Factory replique exactement ce modele.

#### [4] mdBook (rust-lang/mdBook)
- URL : https://github.com/rust-lang/mdBook
- Description : generateur de livres statiques HTML depuis Markdown. Navigation prev/next integree.
- Fichiers source lus : templates/index.hbs (structure HTML avec blocks prev_link/next_link).
- Pattern navigation : prev/next via objects `previous` et `next` contenant title + link. Genere depuis la table des matieres (SUMMARY.md).
- Relevance : Babel Reader est un reader de contenu statique avec navigation sections — pattern comparable. Difference : mdBook genere depuis Markdown, Babel Reader est un squelette HTML editable.
- Verdict : ALIGNED — la navigation prev/next par sections est le pattern standard pour les book readers.

#### [5] static-book-webpage-template (leo-aa88)
- URL : https://github.com/leo-aa88/static-book-webpage-template
- Description : template HTML/CSS/JS pur pour un site web de livre statique.
- Pattern navigation : `<div class="chapter">` caches/visibles, boutons prevChapter/nextChapter, toggle display via JS.
- Fichiers : index.html (structure), style.css, script.js (navigation logic).
- Pattern : sections `display: none/block` avec current index tracking en JS, font customization, theme toggle.
- Verdict : ALIGNED — le plan Phase C (sections de texte avec prev/next navigation, dark theme, responsive) replique exactement ce pattern standard pour les static book readers HTML purs.

### Tableau comparatif

| Aspect | Plan Phase C | project-scaffold | Vite create | mdBook | static-book-template |
|--------|-------------|-------------------|------------|--------|---------------------|
| Multi-template routing | match sur nom | config TOML | flag --template | N/A (1 template) | N/A (1 template) |
| Embedding | include_str! | repertoires externes | repertoires embarques | handlebars | fichiers bruts |
| Variable substitution | {{name}}/{{version}} | {{variable}} | package.json name | SUMMARY.md parse | N/A |
| Navigation sections | prev/next JS toggle | N/A | N/A | prev/next links | prev/next buttons |
| Dark theme | oui | N/A | N/A | oui (theme) | oui (toggle) |
| Provenance | factory.template.lock | N/A | N/A | N/A | N/A |
| Manifest | SBFB.json v2 | N/A | package.json | book.toml | N/A |

### Finding S1a

- Classification : **APPROACH-ALIGNED**
- Evidence : 5 projets OSS analyses (project-scaffold, cargo-scaffold, Vite create, mdBook, static-book-webpage-template). Le pattern multi-template avec include_str! + substitution + routing par nom est standard. La navigation prev/next sections est le pattern standard pour les book readers HTML.
- Impact sur le plan : aucun.
- Note APPROACH-NOVEL : le `factory.template.lock` BLAKE3 + `factory.provenance.json` sont specifiques a SBFB (tracabilite template), justifies par le contexte source verifiable (deploy verifie Sprint 14). Aucun des projets OSS n'a cet equivalent — c'est novel, pas naive.

---

## S1b — Deps/libs versions + CVE

### Deps dans le perimetre Phase C

Phase C ne modifie PAS `Cargo.toml`. Aucune dep ajoutee, aucune dep bumpee. Les fichiers template sont des fichiers statiques embarques via `include_str!` — pur code Rust + HTML/CSS/JS.

### Deps existantes verifiees (securite-pertinentes)

| Dep | Version lockee | CVE 2025-2026 | Status |
|-----|---------------|---------------|--------|
| serde_json | 1.0.149 | Fedora update 1.0.145 = routine (pas de CVE specifique) | clean |
| walkdir | 2.5.0 | TOCTOU inherent (issue #156 2021), pas de CVE formel | clean |
| blake3 | 1.8.5 | 0 advisory RustSec 2025-2026 | clean |
| dunce | 1.0 | stable, pas de CVE | clean |
| nexus-core-rs | local path | pas de dep tierce ajoutee Phase C | clean |
| sbfb-manifest | local path | pas de dep tierce ajoutee Phase C | clean |

### Specs

Aucune spec externe (RFC, SLSA, etc.) touchee par Phase C. Le template static-reader est du HTML/CSS/JS pur, pas de wire format protocolaire.

### Finding S1b

- 0 delta dep. 0 CVE critique. 0 breaking change.
- Classification : **clean**

---

## S2 — Decision chain reconstruction

### Fichiers scannes

- `crates/sbfb-factory/src/template_engine.rs` : 3 commits (49d6bcd, a201b3e, a4cc0aef)
- `crates/sbfb-factory/src/templates/` : 3 commits (49d6bcd, a201b3e, a4cc0aef)
- `crates/sbfb-factory/src/main.rs` : 6 commits (49d6bcd, a4cc0aef, a201b3e, 1d53f18, c92e656, aec036b)
- Bodies complets lus pour les 12 commits.

### Decisions historiques trouvees

#### Decision 1 : Template engine embarque via include_str!

- Sprint 67 Phase C, sha `49d6bcd` : Factory creee avec templates embarques via include_str!.
  Body extrait : "template_engine.rs : create() genere projet depuis template static (include_str!, substitution {{name}}/{{version}}, SBFB.json v2 via sbfb-manifest, factory.template.lock)"
  Scope cut explicit : "Template 'static' uniquement, pas de templates dynamiques"
- Sprint 69 kickoff D2 : "L'agent livre un template static-reader enrichi [...] meme structure (index.html + SBFB.json + sbfb-bridge.js), avec un squelette UI minimal"
- Reverse-commit check : N/A (decision actuelle, pas de reversion candidate)
- Status : **active** — Phase C l'etend avec un 2e template.
- Impact phase : **aucun** — Phase C suit la meme architecture (include_str! + routing par nom).

#### Decision 2 : Babel est cree par l'utilisateur, pas code par l'agent

- Sprint 69 kickoff D2, PO recadrage 2026-05-21 :
  Citation directe : "Babel est cree avec Factory par le dogfood utilisateur, pas code comme livrable agent"
- Implication : l'agent livre le template static-reader (squelette), l'utilisateur fait `sbfb-factory create --template static-reader --name babel-reader` puis ajoute son contenu.
- Status : **active**
- Impact phase : **aucun** — le plan Phase C est coherent (template + 3 tests, pas de contenu Babel).

#### Decision 3 : Bridge SDK copie depuis web/public/sbfb-bridge.js

- Sprint 67 Phase C, sha `49d6bcd` :
  Body extrait : "Bridge SDK = starter minimal, pas le SDK complet 413 lignes"
- Sprint 57 Phase C : Protocol Explorer a copie le SDK complet `web/public/sbfb-bridge.js`.
- Sprint 69 plan §6.2 : "sbfb-bridge.js — copie SDK bridge depuis web/public/sbfb-bridge.js"
- Status : **evolution** — Phase C prevoit de copier le SDK complet (comme Protocol Explorer), pas le starter minimal de S67.
- Impact phase : **CONCERN LOW** — le plan dit "copie SDK bridge", le template existant `static` a un starter minimal. La copie complete (422 lignes) est la bonne approche (coherence avec examples/). Pas bloquant — juste noter la divergence avec le template static existant.
- Reverse-commit check : `git log --all --oneline 49d6bcd..HEAD -- web/public/sbfb-bridge.js` — pas de reversion. Le SDK complet est a 422 lignes, mis a jour S68.

### Memory constraints

- `feedback_approach.md` : "pick deepest" — le template static-reader est l'option la plus poussee entre "pas de template enrichi" (rejete D2) et "template react-vite" (rejete, surdimensionne). Coherent.
- `vision_model.md` : solo maintainer — un template CLI est coherent. Pas d'infra.
- `feedback_context7_systematic.md` : Phase C ne touche pas de lib tierce. N/A.

### Finding S2

- 3 decisions historiques trouvees, toutes coherentes avec le plan Phase C.
- 1 CONCERN LOW : le bridge SDK dans le template doit etre la copie complete (web/public/sbfb-bridge.js, 422 lignes), pas le starter minimal du template `static` (5 lignes). Le plan §6.2 est explicite sur ce point.
- Classification : **clean** (0 bloquant)

---

## S3 — Threat model analysis

### Primitive analysee : Template static-reader (squelette HTML/CSS/JS + Factory scaffolding)

### Assets en jeu

- A1 **Integrite template** (low) : le template est embarque via `include_str!` au compile-time. Pas modifiable a runtime.
- A2 **Provenance template** (medium) : `factory.template.lock` (BLAKE3 hash) + `factory.provenance.json` lie la generation a un template specifique.
- A3 **Bridge SDK integrite** (medium) : `sbfb-bridge.js` est copie statiquement — pont iframe<->reseau.

### Threat actors

- TA1 **Utilisateur malveillant modifiant le template post-create** : full filesystem access. Motivation : publier une app piege.
- TA2 **Supply chain sur le bridge SDK** : copie divergente de sbfb-bridge.js pourrait creer une surface.

### Attack vectors identifies

1. **V1 Injection template post-create** (TA1) : L'utilisateur modifie le template genere pour injecter du JS malveillant.
   - Couverture : FG5 sandbox gate + FG6 secrets gate + FG8 provenance Ed25519 + iframe sandbox `sandbox="allow-scripts"` sans `allow-same-origin` + CSP `connect-src 'none'`.
   - Status : **couvert** par FG pipeline + iframe sandbox.

2. **V2 Bridge SDK desync** (TA2) : La copie de sbfb-bridge.js dans le template desynchronise.
   - Couverture : sync-bridge-sdk check fail-fast #14.
   - Status : **couvert**.

3. **V3 DoS via template excessif** : template > memoire compilateur.
   - Impact : negligeable — squelette HTML < 10KB, `include_str!` compile-time.
   - Status : **non-applicable**.

4. **V4 XSS dans le template HTML** : JS dans le template genere.
   - Couverture : contenu statique (pas de user input dynamique dans le template). JS = scaffolding (prev/next, bridge init). Pas de `innerHTML` avec input utilisateur.
   - Status : **couvert** par design.

5. **V5 Path traversal nom template** : `--template ../../../etc/passwd`.
   - Couverture : `template_engine.rs:71` match exact sur le nom ("static" ou "static-reader"), pas de path resolution. Retourne `TemplateNotFound` pour tout autre nom.
   - Status : **couvert** par match exhaustif.

### Mitigations existantes

- T0-T5 couvrent les vecteurs reseau/iframe (hors scope Phase C = locale CLI).
- T-PREVIEW-EXHAUSTION (S69 Phase A §13) couvre l'abus preview.
- FG pipeline (S69 Phase B) couvre le publish path.
- sync-bridge-sdk check couvre la desync SDK.

### Gaps identifies

- **Aucun gap identifie.** Phase C ajoute du contenu statique embarque dans un CLI local. Pas de nouvelle surface reseau/iframe/wire.

### Regression check

- Pas de regression T0-T5.
- Pas de nouveau vecteur non couvert.
- Aucun nouveau T necessaire.

### Verdict S3 : **clean** — 0 gap, 0 regression.

---

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui

canonical.rs (296 lignes) : 14 constantes DOMAIN_*_V1, fonction `canonical_bytes<T>()` avec JCS + domain separation. Aucune modification requise par Phase C.

### Structs verifiees

Phase C ne touche **aucune** struct dans canonical.rs ou dans les modules wire format. Les modifications sont :
- `template_engine.rs` : ajout d'un 2e template (routing `"static-reader"`)
- Nouveaux fichiers statiques dans `templates/static-reader/`
- `SbfbManifest` est genere dans `create()` (comme pour le template `static` existant) — pas de nouveau champ, pas de modification schema.

### Day 0 check

- D1 FG8 provenance Ed25519 : non touche (Phase B done).
- D2 Babel Reader template static-reader via Factory : **Phase C implemente D2** — coherent.
- D3 FG9 pipeline : non touche (Phase B done).
- D4 audit log + P2 : non touche (Phase A done).
- D5 Gate 1 test protocol : non touche (Phase D).
- **Aucune D1-D5 contredite.**

### Decisions actees pivot.md

- "Factory = outil client externe (crate sbfb-factory), hors daemon" : Phase C modifie sbfb-factory — coherent.
- "Feed raw-op extensible, pas de bump par op" : Phase C n'ajoute pas de feed op — coherent.
- "Gate 1 se valide sur @protocole + Proof Cards + publish + Babel dogfood" : Phase C = Babel template — coherent.
- **Aucune decision actee contredite.**

### Pre-launch policy

- `*_VERSION = 1` : Phase C ne touche aucune constante version.
  - TASK_FORMAT_VERSION = 1 : inchange
  - CURATOR_LIST_FORMAT_VERSION = 1 : inchange
  - KEY_ROTATION_FORMAT_VERSION = 1 : inchange
  - POW_FORMAT_VERSION = 1 : inchange
  - PIN_FILE_FORMAT_VERSION = 1 : inchange
  - FEED_FORMAT_VERSION = 1 : inchange
  - PROVENANCE_SCHEMA_VERSION = 1 : inchange
- `serde_json::to_string` : 10 occurrences dans sbfb-factory, toutes hors wire format protocolaire (manifest pretty-print, audit log, template lock, gates test, provenance local). Aucun usage sur struct signee — les structs signees passent par `canonical_bytes()` (JCS via serde_jcs).
- Pas de tolerant decoder multi-version. Pas de tests "legacy decode" zombie.

### SBFB.json bridge methods check

Plan §6.2 specifie `bridge.methods: ["storage_get", "storage_set", "identity_pubkey"]`.
Verification contre `BRIDGE_METHOD_ALLOWLIST` (sbfb-manifest/src/lib.rs:52-63) :
- `storage_get` : present — OK
- `storage_set` : present — OK
- `identity_pubkey` : present — OK
Les 3 methodes sont dans l'allowlist. Le manifest genere passera `validate()`.

### Finding S4 : **clean** — 0 VERSION bump, 0 Day 0 contredite, 0 wire format touche.

---

## Telemetrie preflight (agent deep)

- Duree totale : ~8min
- S1a : ~4min / 5 projets OSS analyses / 5 fichiers source lus (via WebFetch) / ~500 LOC reviewees / 0 context7 queries (pas de lib tierce nouvelle) / 6 WebSearch queries / finding : APPROACH-ALIGNED
- S1b : ~1min / 6 libs scannees / 3 CVE searches / finding : clean
- S2 : ~2min / 12 commits bodies lus / 2 archive files / 6 memory files / finding : clean (1 CONCERN LOW bridge SDK)
- S3 : FULL / ~0.5min / 5 vectors analyses / 0 gaps
- S4 : FULL / ~0.5min / 0 structs wire modifiees (Phase C ne touche pas de struct wire) / canonical.rs lu integralement : oui

---

## Action

Proceder code phase C.
