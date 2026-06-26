# Sprint 79 Phase A Preflight (G8 deep, Workflow)

Date: 2026-06-24
HEAD: `477e147`
Verdict: **PLAN-ADAPT**
Méthode: Workflow ultracode `wf_5e058304-06c` — 5 scans canoniques en parallèle
(S1a/S1b/S2/S3/S4, Opus 4.8 1M chacun) + synthèse adversariale. 6 agents, 549K tokens.

## Evidence Rules
- Toute claim ci-dessous cite un chemin repo + ligne, une commande + sa sortie, ou une URL + date.
- Local sources lues : `prompts/agent/preflight.md`, `.planning/active/sprint79_plan.md` (§Phase A),
  `.planning/active/sprint79_kickoff.md`, `crates/sbfb-factory/src/provenance.rs`,
  `crates/sbfb-factory/src/gates.rs`, `crates/sbfb-factory/src/template_engine.rs`,
  `crates/sbfb-factory/src/pipeline.rs`, `Cargo.lock`,
  `examples/daisyui-animejs-showcase/knowledge/daisyui/MANIFEST.json`, `…/knowledge/README.md`.

## Scope
- Plan source: `.planning/active/sprint79_plan.md` §Phase A (l.169-187).
- Target files: `docs/factory/knowledge/animejs/*` (git mv des 5 couches anime.js + MANIFEST.json new) ;
  `provenance.rs` INCHANGÉ.
- Deps/APIs/specs: aucune nouvelle (anime/daisyUI/tailwind = devDeps build-time du template Phase F,
  PAS des deps Rust ; `blake3` déjà workspace dep).
- Security/protocol surfaces: aucune (0 canonical.rs/schemas/`*_VERSION`/DOMAIN_ touché).
- Tests attendus: +1 Rust hermétique (recompute blake3 par-couche == MANIFEST).

## S1a OSS Prior Art
- Domaine: corpus de connaissance versionné + content-addressé consommé par LLM/outils.
- Finding: **APPROACH-ALIGNED** (non-bloquant). Un MANIFEST.json content-addressé (version + date
  snapshot + hash blake3 par couche + table verdict CSP, sans auto-fetch, re-extraction manuelle au
  bump) mirrore le pattern mûr « pin version + per-file hash + recompute » (SLSA provenance,
  npm integrity, Subresource Integrity, convention llms.txt/llmstxt.org). Pas de `LIB-EXISTS` qui
  remplacerait un MANIFEST bespoke. Corroboré in-repo par le précédent daisyui
  (`knowledge/daisyui/MANIFEST.json`) et l'index agent `docs/sharding/llms.txt` (S77).
- Confiance: l'agent n'a pas cité d'URL précise dans son evidence array (confiance abaissée sur les
  URL spécifiques), mais la conclusion non-bloquante est corroborée par le précédent in-repo → ne
  change pas le verdict.

## S1b Dependencies, CVEs, Release Notes
- Finding: **clean**. 0 nouvelle crate, 0 nouvelle dep. `blake3` est déjà dep workspace, résolue à une
  version **unique 1.8.5** (`Cargo.lock:701-702`), aucun doublon / second-major (`cargo tree -d` propre).
  `anime`/`daisyui`/`tailwind` n'apparaissent dans AUCUN manifeste Rust (devDeps npm du template Phase F).
- API pour le test +1: `blake3::hash(&[u8]) -> Hash` ; `.to_hex() -> ArrayString` (mirror exact du
  hashing que le MANIFEST doit matcher).

## S2 Historical Decisions
- Finding: **SCOPE-CUT-CONSISTENT** (non-bloquant). `docs/factory/knowledge/` = décision Day-0
  resolu-preuve (kickoff D1) ; `prompts/agent/` est un répertoire plat à invariant testé
  (`process.rs:888-905`) — d'où le choix docs/. `docs/factory/knowledge` a **0 historique git**
  (rien de reverté → aucun conflit S2). Précédent direct: S77 Phase N (`a795700`) a hébergé des assets
  de connaissance agent-consommables sous `docs/` (`docs/sharding/llms.txt`, WIRING_SPEC).
- Schéma MANIFEST à mirrorer (clé pour Phase A): `knowledge/daisyui/MANIFEST.json` =
  `{pack, method, date, versions, default_theme, sources[], layers{}, counts{}, hashes{}}` ;
  hash = blake3 **tronqué 16 hex** (ex. `components.json`:`679193901618114c`, `MANIFEST.json:30`).
  NB: daisyui ne hash que **3** de ses 4 couches (.json seuls, pas COMPONENTS.md ni docs-llms.txt) —
  sous-couverture ; voir Plan adaptation (animejs hashera TOUTES les couches promues).

## S3 Local Patterns And Threat Model
- Finding: **SCOPE-CUT-CONSISTENT** (non-bloquant). FG6 (`gates.rs:127-165`) ne hash QUE le workspace
  d'app publié, **jamais `docs/`** → la claim D1 « hors workspace d'app ⇒ 0 impact FG6 » est **VRAIE**.
  Aucune régression T0-T5 (corpus repo-visible, human-reviewed, hashé, jamais dans l'archive d'app —
  `knowledge/README.md` « dev-only jamais publié »). Aucun pré-requis HARDENING pour S79.

## S4 Protocol And Wire Invariants
- Wire: **clean**. Aucun `canonical.rs`/`schemas/`/`DOMAIN_` touché ; les 14 `*_VERSION` restent =1 ;
  Day-0 D9 (0 bump wire / 0 dep) préservé. MANIFEST.json = fichier de données, pas un wire format.
- **CLAIM ARCHITECTURALE D1/kickoff FAUSSE (verdict: FALSE)** : « hashé GRATUITEMENT par
  `provenance::compute_output_hash` (tree-walk blake3) + FG8 dès qu'il est dans le source » est faux
  comme mécanisme runtime. Vérifié par lecture directe :
  - `compute_output_hash` (`provenance.rs:49`) est une fn **privée**, appelée seulement par
    `Provenance::generate` (`provenance.rs:29`), câblée uniquement à `template_engine.rs:264` sur un
    **workspace d'APP neuf**, produite côté serveur sur l'app clonée au publish (`pipeline.rs:48`).
  - FG8 (`gates.rs:208`) signe/vérifie l'`artifact_hash` de l'app, **jamais `docs/`**.
  - `provenance` émet un agrégat **64-hex** (`provenance.rs:79`) ; le MANIFEST daisyui stocke des
    **16-hex par fichier** (`MANIFEST.json:30-33`) → le champ T2 du plan « provenance blake3 == MANIFEST »
    (`plan:142-144`) est littéralement incohérent.
  - Le corpus est content-addressé **par le commit git**, point.
- Day-0 status: **location/architecture D1 PRÉSERVÉES** (docs/factory/knowledge/, process-asset,
  0-FG6). Seule la *description du mécanisme de hash* (claim factuelle incidente) est corrigée → pas un
  DESIGN-CONFLICT.

## Plan Adaptation
- Original plan/kickoff: « hashé gratuitement par `provenance::compute_output_hash` + FG8 » ; T2 field
  `manifest_hash_recompute_ok (provenance blake3 re-calcule == MANIFEST)` (`plan:142-144`).
- Evidence requérant l'adaptation: `provenance.rs:49/29/79`, `template_engine.rs:264`, `pipeline.rs:48`,
  `gates.rs:208` (FG8 sur app artifact), `daisyui/MANIFEST.json:30-33` (16-hex par-fichier).
- Approche corrigée:
  1. Le corpus est **content-addressé par le commit git** ; le MANIFEST **self-record** un blake3
     **16-hex par couche** (mirror daisyui).
  2. Test +1 = **hermétique, standalone** : recompute `blake3::hash(read(file)).to_hex()[..16]` par
     couche et `assert_eq!` == MANIFEST. **NE PAS** appeler `Provenance::generate` sur `docs/`, **NE PAS**
     prétendre que FG8 couvre `docs/`, **NE PAS** comparer à l'agrégat 64-hex de `compute_output_hash`.
  3. Le MANIFEST animejs hash **TOUTES** les couches promues (pas le sous-ensemble daisyui) → test
     non-vacuous. Le back-fix daisyui vers la même couverture = concern Phase E.
  4. Champ T2 reformulé: `manifest_hash_recompute_ok` = « per-layer blake3 16-hex recompute == MANIFEST ».
- File/test delta vs plan: identique en livrables (couches + MANIFEST + 1 test), mais le test ne dépend
  ni du daemon ni de provenance ni de FG8.

### Recommended test (à coder)
Test Rust hermétique dans `crates/sbfb-factory` (module `#[cfg(test)]` ou
`tests/animejs_manifest.rs`) : (1) résout le dir via `env!("CARGO_MANIFEST_DIR")` + `../../docs/factory/
knowledge/animejs/` ; (2) lit MANIFEST.json, parse `hashes` (filename→expected) ; (3) pour chaque,
recompute `blake3::hash(&fs::read(dir.join(name))?).to_hex()[..16]`, `assert_eq!` (message nommant le
fichier en drift) ; (4) assert l'ensemble des clés `hashes` == les fichiers de couche présents (dir
moins MANIFEST.json et dotfiles) → non-vacuous ; (5) assert `versions.animejs == "4.5.0"` + champ
freshness présent. 0-réseau, déterministe.

### Recommended MANIFEST schema (mirror daisyui, animejs)
`{pack:"animejs", method, date:"2026-06-23", versions{animejs:"4.5.0"}, sources[], layers{primitives,
examples,docs,synthesis,types}, counts{primitives:93,csp_usable:93,risk:0,demos:52,doc_pages:419,
types:70}, freshness{snapshot_date,source_ref,refresh:"manual re-extraction at bump"}, hashes{...16-hex
sur TOUTES les couches promues}}`. OMETTRE `default_theme` (daisyui-only). AJOUTER `freshness`.
Convention hash = `blake3::hash(bytes).to_hex()[..16]` (16 chars, mirror daisyui). Documenter le choix
de convention dans le commit body (3 conventions coexistent: file_hash `[..8]`, daisyui 16-hex,
provenance 64-hex — standardiser animejs sur 16-hex, back-fix daisyui = Phase E).

## Risks And Scope Cuts
- Blocking risks: aucun.
- Non-blocking / watch-items (→ portés au commit body + carry si besoin):
  1. Corriger la formulation D1/kickoff:39 dans le commit body (mécanisme = git commit + MANIFEST
     self-hash + test recompute, PAS provenance/FG8 sur docs/).
  2. Champ T2 `manifest_hash_recompute_ok` reformulé (per-layer 16-hex, pas provenance).
  3. Divergence de convention de hash (8/16/64-hex) notée ; back-fix daisyui = Phase E.
  4. `hashes` couvre TOUTES les couches promues présentes (pas le sous-ensemble daisyui).
- Scope cuts honorés (kickoff): asset hors archive d'app (FG6 0-impact, vérifié VRAI) ; 0 bump wire ;
  0 dep nouvelle ; connaissance consommée jamais autoritaire (Phase A n'émet aucun verdict PASS).

## Action
PLAN-ADAPT: coder Phase A avec l'approche corrigée ci-dessus. Le commit body DOIT citer ce preflight et
documenter la déviation (claim provenance fausse → content-addressing git + MANIFEST self-hash + test
recompute standalone).
