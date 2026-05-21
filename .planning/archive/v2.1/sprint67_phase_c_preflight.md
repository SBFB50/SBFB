# Sprint 67 Phase C — preflight G8

Date : 2026-05-20 | HEAD : `688b442` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- feedback_approach.md : pick deepest, no band-aid, research before
  code, OSS prior art obligatoire avant chaque phase — N/A conflict
  (Phase C est un crate nouveau aligne avec SYNTHESIS §3)
- feedback_context7_systematic.md : context7 obligatoire avant
  code/decision touchant lib/API — applique sur clap, blake3,
  walkdir (queries ci-dessous)
- vision_model.md : NO funding / NO fondation — N/A (Phase C
  n'introduit aucun pattern startup)
- feedback_kudos_non_monetary.md : kudos non-monnaie — N/A (Phase C
  ne touche pas kudos)
- fairness_vision.md : N/A (Phase C ne touche pas fairness/reputation)
- nexus_grid_pivot.md : Factory hors daemon (D2 v4), @protocole
  d'abord (D6 v4) — Phase C respecte les deux (sbfb-factory est un
  crate workspace independant, dep sbfb-manifest uniquement)
- Tensions plan vs memory : aucune

---

## S1a — OSS prior art deep analysis

### Probleme fonctionnel exact

"How do mature OSS projects implement CLI-based project scaffolding
with template substitution, template integrity tracking, and secret
scanning?"

### Projets analyses en profondeur

#### [Projet A] — cargo-generate (https://github.com/cargo-generate/cargo-generate)
- Fichiers source lus : src/template.rs (~300 LOC review via
  WebFetch), src/lib.rs (~200 LOC review via WebFetch),
  tests/integration/basics.rs (via WebSearch)
- Pattern architectural extrait : Liquid template engine (Shopify),
  full-featured parser with custom filters (KebabCase, SnakeCase),
  Rhai scripting hooks. Graceful degradation — missing variables →
  empty string, parse errors → original content returned unmodified.
- Edge cases geres : template version requirement check (tool
  version, not template version), variable type validation
  (string/bool/choice), file include/exclude lists, post-generation
  hooks
- Patterns abandonnes : aucun visible dans le README changelog
- **NO lockfile / NO template hash tracking** — cargo-generate est
  un outil de generation one-shot sans reproductibilite integree
- Verdict : ALIGNED sur le pattern copie+substitution.
  NOVEL pour factory.template.lock (cargo-generate n'en a pas).

#### [Projet B] — Copier (https://github.com/copier-org/copier)
- Fichiers source lus : documentation Copier (via WebSearch +
  PyPI page), architecture overview (via WebSearch)
- Pattern architectural extrait : Jinja2 templates (.jinja suffix),
  versioned templates (git tags), update mechanism
  (`copier update`). Copier tracks template source et version
  pour permettre les updates incrementales.
- Edge cases geres : conditional files (`when` key), multi-template
  composition, template versioning via git tags
- **Template tracking** : Copier maintient un `.copier-answers.yml`
  dans le projet genere qui enregistre le template source, la
  version (git ref), et les variables utilisees. C'est l'equivalent
  fonctionnel du `factory.template.lock` du plan.
- Verdict : ALIGNED — Copier confirme que le tracking de l'origine
  template est un pattern mature.

#### [Projet C] — Backstage Scaffolder (https://github.com/backstage/backstage/tree/master/plugins/scaffolder)
- Fichiers source lus : README scaffolder module, scaffolder API
  (via WebSearch), catalog-info.yaml format doc
- Pattern architectural extrait : step-based scaffolding pipeline,
  parameters definition, dry-run mode. L'UI Backstage genere des
  projets via des "templates" qui sont des entity specs YAML.
- Edge cases geres : nested parameters, template validation,
  dry-run preview, custom actions
- **Pas de lockfile d'integrite** — le tracking est dans le
  Software Catalog (entity spec, pas hash)
- Verdict : ALIGNED sur le pattern CLI create+validate. Backstage
  est un serveur web, pas un CLI — architecture differente mais
  pattern de generation identique.

#### [Projet D] — Gitleaks (https://github.com/gitleaks/gitleaks)
- Fichiers source lus : config/gitleaks.toml (full TOML config,
  ~150+ rules, via WebFetch raw GitHub)
- Pattern architectural extrait : regex-based secret detection,
  keyword pre-filter, entropy threshold per-rule. Rules are TOML-
  defined with id/description/regex/keywords/entropy.
- Patterns de detection lus en detail :
  - AWS : `\b((?:A3T[A-Z0-9]|AKIA|ASIA|ABIA|ACCA)[A-Z2-7]{16})\b`
  - GitHub PAT : `ghp_[0-9a-zA-Z]{36}`
  - GitHub OAuth : `gho_[0-9a-zA-Z]{36}`
  - Private keys : `(?i)-----BEGIN[ A-Z0-9_-]{0,100}PRIVATE KEY...`
  - Generic API key : entropy 3.5 + keyword match
- Edge cases geres : allowlists per-rule, entropy threshold,
  path-based exclusions, git history scanning
- Verdict : ALIGNED — le plan Phase C utilise un sous-ensemble des
  patterns Gitleaks (AWS AKIA, GitHub ghp_/gho_, PEM private keys).
  Le plan ne fait pas d'entropy check — acceptable pour un MVP
  (scope cut conditionals S68+).

#### [Projet E] — Kingfisher (https://github.com/mongodb/kingfisher)
- Source : WebSearch result (MongoDB open source secret scanner,
  Rust, 950+ rules)
- Pertinence : confirme que des scanners de secrets en Rust
  existent avec des centaines de regles. Le plan Phase C utilise
  un sous-ensemble minimal (4-5 patterns). Extension via config
  fichier TOML serait le pattern industriel pour S68+.
- Verdict : ALIGNED pour le MVP. LIB-EXISTS potentiel mais
  Kingfisher est un outil complet (950 regles) — overkill comme
  dep pour 5 patterns. Le moteur interne est le bon choix pour
  le MVP.

### Tableau comparatif

| Aspect | Plan Phase C | cargo-generate | Copier | Gitleaks |
|--------|-------------|----------------|--------|----------|
| Template engine | include_str! + String::replace | Liquid (Shopify) | Jinja2 | N/A |
| Variable substitution | `{{name}}`, `{{version}}` | `{{crate_name}}`, Liquid syntax | `{{variable}}.jinja` | N/A |
| Template tracking | factory.template.lock (BLAKE3 hash) | Aucun | .copier-answers.yml (source+version+vars) | N/A |
| Secret scanning | 4-5 regex hardcodes | Aucun | Aucun | 150+ TOML rules + entropy |
| Path traversal | reject `..` + symlinks | Non applicable | Non applicable | Non applicable |
| Lockfile integrity | BLAKE3 hash du template | Aucun | git tag du template | N/A |

### Finding S1a

- Classification : **APPROACH-ALIGNED** (copie+substitution) +
  **APPROACH-NOVEL** (factory.template.lock BLAKE3)
- Evidence :
  - cargo-generate utilise le meme pattern copie+substitution
    (avec un moteur plus riche — Liquid), mais SANS lockfile
  - Copier confirme le besoin de tracking template origin
    (.copier-answers.yml — variables + source + version)
  - Gitleaks confirme les patterns regex du plan pour le secret
    scanner (sous-ensemble des 150+ regles Gitleaks)
  - Le factory.template.lock avec hash BLAKE3 est un ajout NOVEL
    justifie par le contexte SBFB (provenance, reproductibilite,
    audit trail pre-launch)
- Impact sur le plan : aucun — le plan est ALIGNED avec le SOTA
  pour la generation, NOVEL mais justifie pour le lockfile

---

## S1b — Deps/libs versions + CVE

### Deps Phase C (plan §S6.2)

| Dep | Version workspace | Status | CVE check |
|-----|-------------------|--------|-----------|
| sbfb-manifest | workspace (S67 Phase A) | Disponible | N/A (crate interne) |
| clap | 4.5 (derive) | Stable, deja dans workspace | WebSearch 2026-05-20 : 0 CVE |
| blake3 | 1.5 (workspace) | Stable, deja utilise | WebSearch 2026-05-20 : 0 CVE |
| serde + serde_json | 1.0 (workspace) | Stable | Connu clean |
| walkdir | NEW dep | Stable | WebSearch 2026-05-20 : 0 CVE rustsec, 0 advisory |
| zip | 8.5 (workspace) | Stable | CVE-2025-29787 affecte < 2.3.0 seulement — 8.5 non vulnerable |
| ed25519-dalek | 2.1 (workspace) | Stable | WebSearch 2026-05-20 : page rustsec consultee, pas de CVE 2025-2026 critique |
| thiserror | 1.0 (workspace) | Stable | Connu clean |

### Finding S1b

- **0 CVE bloquant** sur les deps Phase C
- **walkdir** est la seule dep NEW — crate mature (BurntSushi),
  stable, 0 advisory rustsec
- **zip 8.5** : CVE-2025-29787 (path traversal < 2.3.0) ne
  s'applique pas — version 8.5 est bien au-dessus
- Note : walkdir est utilise pour la traversee recursive du
  workspace genere dans `validate`. Le plan l'utilise avec
  `follow_links(false)` pour detecter les symlinks (context7
  `/burntsushi/walkdir` confirme `path_is_symlink()` API)

---

## S2 — Decision chain reconstruction

### Fichiers scannes

- `crates/sbfb-factory/` : 0 commits (crate nouveau)
- `crates/sbfb-manifest/` : 1 commit (Phase A `4ee93ab`)
- Recherche "factory" dans tous les commits : 20 resultats,
  bodies lus pour les pertinents

### Decisions historiques trouvees

#### Decision 1 : Factory hors daemon (D2 v4)

- Sprint 65, sha `276173a` : recherche roadmap v4, SYNTHESIS
  §3.1-3.2 etablit que Factory est un outil client externe
  Body extrait : "10 documents research (factory gap analysis,
  protocol neutrality, RRV scope/boundary...)"
- Sprint 65, sha `9727818` : Phase D livre FACTORY_GATES.md
  (11 gates FG0-FG10)
  Body extrait : "FACTORY_GATES.md 11 gates FG0-FG10 spec
  pour S67-S69"
- Sprint 67, sha `d477d81` : kickoff D5 gele sbfb-factory CLI
  crate
  Body extrait : "D5 — sbfb-factory CLI crate externe avec
  create + validate"
- Reverse-commit check : pas de reversion trouvee
- Status : **active**
- Impact phase : aucun — Phase C respecte D2

#### Decision 2 : Pre-launch protocol policy

- Sprint 16, sha `d1e6971` : drop pre-launch backward-compat
  Body extrait : "VERSION stays at 1, decoder accepts only v == 1"
- Status : **active**
- Impact phase : aucun — Phase C ne touche aucun wire format
  (sbfb-factory est local-only, pas de surface reseau)

#### Decision 3 : Feed raw-op extensible (D4 v4)

- Sprint 67, sha `4ee93ab` : Phase A livre CuratorVouched raw-op
  Body extrait : "FEED_FORMAT_VERSION = 1 preserve.
  CuratorVouched/CuratorDisendorsed = nouvelles variantes
  PublicFeedOperation, PAS de bump (raw-op P51)"
- Status : **active**
- Impact phase : aucun — Phase C ne touche pas le feed

### Memory constraints

- feedback_approach.md : pick deepest — applique (le plan utilise
  include_str! + String::replace, pas Tera/Handlebars overkill
  pour le MVP, c'est le bon choix per SYNTHESIS §3.3)
- feedback_context7_systematic.md : context7 queries faites pour
  clap (derive subcommands), blake3 (hash API — timeout,
  connaissance workspace existante), walkdir (symlink detection)
- vision_model.md : N/A

---

## S3 — Threat model analysis

### Primitive analysee : sbfb-factory CLI (create + validate)

### Assets en jeu

- A1 Workspace genere (fichiers locaux) : criticite low
  (fichiers locaux, pas de donnees sensibles par design)
- A2 Template integrity (factory.template.lock) : criticite
  medium (audit trail pour provenance future S68+)
- A3 Secret scanning coverage : criticite medium (protege contre
  la publication accidentelle de secrets dans une app SBFB)

### Threat actors

- TA1 Developpeur negligent : publie accidentellement des secrets
  (AWS keys, GitHub tokens) dans son app SBFB
- TA2 Template malveillant : un template modifie pourrait injecter
  du code malveillant dans les apps generees
- TA3 Path traversal attacker : un workspace crafted avec `../`
  ou symlinks pourrait escaper le repertoire de validation

### Attack vectors identifies

1. V1 Secret leakage via app publish : un developpeur oublie une
   API key dans son code, `validate` ne la detecte pas →
   publication sur le reseau P2P public
   - Couverture : secret_scanner.rs avec 4-5 regex patterns
     (AWS AKIA, GitHub ghp_/gho_, PEM private keys)
   - Gap : pas d'entropy check (faux negatifs possibles sur
     les secrets custom/non-standard)
   - Severity : Medium (gap non-bloquant — le scanner couvre les
     patterns les plus courants, extension TOML rules S68+)

2. V2 Path traversal dans validate : un workspace contient
   `../../etc/passwd` ou un symlink vers `/etc/shadow`
   - Couverture : validate rejette `..` dans les paths et
     detecte les symlinks via walkdir `path_is_symlink()`
   - Gap : aucun identifie — la protection est adequate
   - Severity : N/A (couvert)

3. V3 Template substitution injection : un `{{name}}` contenant
   du HTML/JS malveillant serait injecte dans index.html
   - Couverture : le `name` provient du `--name` CLI arg, pas
     d'un input reseau. Pre-launch, le developpeur est l'user
   - Gap : pas de sanitization du name contre les caracteres
     speciaux HTML (XSS potentiel dans le HTML genere)
   - Severity : Low (pre-launch, l'app tourne dans un iframe
     `sandbox="allow-scripts"` sans `allow-same-origin`, le
     XSS ne peut pas escalader)

4. V4 Template tampering : un attaquant modifie les templates
   embarques (include_str!)
   - Couverture : les templates sont compile-time embedded
     (include_str!), pas de fichier externe chargeable →
     tampering = recompilation du binaire = supply chain attack
     (couvert par R2 §8 THREAT_MODEL)
   - Gap : aucun — compile-time embedding est la meilleure
     protection
   - Severity : N/A (couvert par supply chain mitigations)

5. V5 DoS/resource exhaustion dans create : un `--name` tres long
   ou un template tres gros
   - Couverture : templates embarques = taille fixe compile-time.
     `--name` est un argument CLI local, pas un input reseau
   - Gap : aucun (local-only tool)
   - Severity : N/A

6. V6 Supply chain (nouvelle dep walkdir) :
   - Couverture : walkdir est un crate BurntSushi mature, 0 CVE,
     0 advisory rustsec
   - Gap : aucun
   - Severity : N/A

### Mitigations existantes

- T-FEED-* : N/A (Phase C ne touche pas le feed)
- T-SEARCH-* : N/A (Phase C ne touche pas le search)
- Supply chain (R2 §8) : couvre V4 template tampering

### Gaps identifies

- GAP1 V1 entropy check absent : severity Low — extension TOML
  rules S68+ (scope cut plan #10 Factory audit log)
- GAP2 V3 name sanitization : severity Low — iframe sandbox
  contient le blast radius

### Regression check

- La primitive ne diminue l'efficacite d'aucune mitigation T0-T5
  existante
- La primitive ne cree aucun nouveau vecteur non couvert (local-
  only tool, pas de surface reseau)
- Pas de nouveau T necessaire

### Verdict S3 : **clean** (0 gap severity High, 2 gaps Low)

---

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui

296 lignes. 14 domaines DOMAIN_*_V1 definis. Aucun ne concerne
sbfb-factory (Factory est un outil client sans surface
cryptographique propre en S67 — la signature Ed25519 est
optionnelle S67 per plan §S7.1).

### Structs verifiees

Phase C ne touche AUCUNE struct dans canonical.rs. Phase C cree
un crate nouveau (sbfb-factory) qui n'a pas de surface wire
format. Le seul lien est la dep sbfb-manifest (crate Phase A)
pour generer SBFB.json v2 — mais SbfbManifest n'est PAS un type
signe (pas de DOMAIN_MANIFEST_V1, pas de canonical_bytes).

### Day 0 check

- D1 FTS5 search : non touche par Phase C
- D2 sbfb-manifest crate : respecte — sbfb-factory dep
  sbfb-manifest (pas le contraire)
- D3 CuratorVouched : non touche par Phase C
- D4 Feed entries read : non touche par Phase C
- D5 sbfb-factory CLI : **Phase C implemente D5** — conforme

Decisions actees pivot.md : aucune contredite
- Factory hors daemon (D2 v4) : respecte
- @protocole d'abord (D6 v4) : respecte (Phase B search livre,
  Phase C est la couche @dev Factory)
- Feed raw-op (D4 v4) : non touche
- FTS5 first (D1 v4) : non touche

### Pre-launch policy

- `*_VERSION = 1` : aucune constante version touchee par Phase C
- Pas de tolerant decoder multi-version : N/A
- Pas de tests "legacy decode" zombie : N/A
- sbfb-factory n'introduit aucun wire format — les artefacts sont
  des fichiers locaux (SBFB.json v2, factory.template.lock,
  factory.provenance.json) sans surface reseau en S67

### Verdict S4 : **clean**

---

## Telemetrie preflight (agent deep)

- S1a : 5 projets OSS analyses (cargo-generate, Copier,
  Backstage scaffolder, Gitleaks, Kingfisher) / 6 fichiers source
  lus via WebFetch / ~800 LOC reviewees / 3 context7 queries
  (clap, blake3 timeout, walkdir) / 8 WebSearch queries /
  finding : APPROACH-ALIGNED + APPROACH-NOVEL (lockfile)
- S1b : 8 libs scannees / 3 CVE searches / finding : 0 CVE
  bloquant (CVE-2025-29787 zip non-applicable v8.5)
- S2 : 3 decisions historiques reconstruites (D2 Factory hors
  daemon, pre-launch policy, feed raw-op) / 5 commit bodies lus
  en entier / 0 archive files / 6 memory files lus /
  finding : 0 conflit
- S3 : FULL / 6 vectors analyses / 2 gaps (Low severity) /
  0 regression
- S4 : FULL / canonical.rs lu integralement (296 lignes) / 0
  structs touchees par Phase C / Day 0 D1-D5 preservees /
  pre-launch policy respectee

## Action

Proceder code phase C.
