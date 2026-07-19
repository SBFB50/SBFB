# Sprint 83 — Phase B — Preflight G8

**Phase** : B « Fondation accessibilité, i18n et préférences privées »
**Date** : 2026-07-18
**HEAD** : `ce99c97` (Phases A + A1 DONE) — working tree PROPRE (vérifié `git status --porcelain` = vide)
**Cadre** : `sprint83_plan.md` §B (l.350-404) · `sprint83_kickoff.md` §4.5 (AM-1..AM-12) · CLAUDE.md §Pre-launch · README §6.9/§6.12/§7.4
**Méthode** : 5 scans factuels (S1a OSS prior-art, S1b deps/CVE, S3 threat, S4 wire) + vérification adversariale de 13 claims load-bearing. Aucune écriture, lecture seule.

---

## VERDICT : PLAN-ADAPT

La fondation Phase B est **exécutable** : toutes les briques OSS sont matures aux versions attendues, le contrat `ResolvedAccessibilityProfile.v1` n'introduit **aucun bump wire** (census DOMAIN reste 25, gelé), et l'invariant cardinal (needs[] jamais exposé) a des fermetures propres et précédentées. **Aucun DESIGN-CONFLICT** : aucune Day-0 gelée n'est touchée, aucune impossibilité OSS/threat.

Le verdict est **PLAN-ADAPT** (pas EXECUTE pur) parce que **deux éléments du texte de plan doivent être corrigés AVANT le premier Edit**, sous peine que la phase cite un chemin inexistant ou omette un livrable de condition binaire :

1. **Chemin module commun caduc** — `plan:366` propose « `crates/sbfb-workflow-core` (ou module commun `tools/factory-ui/readonly` réutilisé) ». Vérifié : `ls tools/factory-ui` → *No such file or directory* (jeté S80, greenfield Operator). La parenthèse pointe un chemin mort. **Adaptation** : le profil vit dans `crates/sbfb-workflow-core` (identité de contrat Rust) + un **module web NEUF** ; le contrat Operator est réutilisé par **re-dérivation**, jamais par import.
2. **Version EN 301 549 non consignée** — `grep` sur `.planning/active/` : aucun numéro de version. Or B.4 (`plan:394-395`, « Condition binaire … EN 301 549 figée consignée ») l'exige, et WA-16/Q6 reste un item §11 pendant (`kickoff:885`). **Adaptation** : trancher et écrire un numéro dans `sprint83_at_matrix.md` (réserve G-A11Y-EN301549).

Ces deux points sont des **précisions d'implémentation / documentaires**, pas des Day-0 : ni le pivot P2P, ni Lingui OVERRIDE, ni le census gelé, ni la séparation Hub/Operator ne sont remis en cause.

---

## S1a — OSS prior-art (signal source : PLAN-ADAPT)

**Toutes les briques sont disponibles et matures ; deux écarts plan↔réalité corrigés ci-dessus.**

- **axe-core / @axe-core/playwright** : le mécanisme `AxeBuilder.withTags(['wcag22aa'])` (WCAG 2.2 AA, support axe-core ≥4.5) est correct en connaissance OSS. **⚠ correction adversariale** : la version « 4.12.1 dernière (juin 2026) » n'est PAS vérifiable depuis le repo (dépasse le cutoff + absente de l'arbre) — à traiter comme **version à PINNER/prouver au `npm install`** (`npm view axe-core version`), jamais comme fait « constaté le 2026-07-18 ». axe-core n'est **déclaré dans aucun `package.json` direct** (`web/` + `operator` = 0) → dev-dependency à ajouter. Nuance : axe-core@4.12.1 existe **transitivement** dans `examples/daisyui-animejs-showcase/package-lock.json` (via `lighthouse@13.4.0`) — mais cette copie ne sert pas le harness ; il faut bien l'ajouter en dev-dep directe.
- **Lingui 6.4.0** (déjà côté Operator) : compile les `.po` en **tableaux de tokens eval-free** (`compileMessage.mjs`, 0 `new Function`/`eval`) → compatible CSP Hub sans `unsafe-eval`. **Ne gère PAS le RTL** : `dir=rtl` + logical properties = chantier CSS/HTML **indépendant de Lingui** (l'Operator le fait à part : `i18n.ts:76 directionOf()`, `catalogs.ts:17 el.dir`, `index.css:120 [dir="rtl"]`). Ce socle existe **uniquement** côté Operator ; le Hub doit le reconstruire via module commun neuf.
- **WCAG 2.2 AA / ATAG 2.0 / COGA** : périmètre factuel stable (9 critères neufs vs 2.1, 4.1.1 supprimé ; ATAG pertinent car le Hub EST un outil d'authoring → mappe G-A11Y-AUTHORING ; COGA testé par panel AI1).
- **Preboot anti-flash + tokens oklch** : patrons OSS établis, déjà éprouvés côté Operator (`public/preboot.js` + `check-accessibility-system.sh`). oklch = Baseline 2023. Modèle d'app SBFB sandboxée à suivre : `examples/daisyui-animejs-showcase` (build-time tooling, runtime 0-dep vendored, check-csp) ; `explorer`/`ideas` = HTML pur.

---

## S1b — Dépendances / CVE / supply-chain (signal : EXECUTE)

**Périmètre dep étroit et à faible risque.**

- **Rust : 0 dep neuve.** `schemars` est déjà déclaré dans `crates/sbfb-workflow-core/Cargo.toml:14` (`{ workspace = true }`), workspace pinné `1.2` (`Cargo.toml:361`), résolu `1.2.1` (`Cargo.lock`). `ResolvedAccessibilityProfile.v1` = un type schemars de plus dans un crate qui a déjà `contracts.rs` → **aucun crate neuf**. *(Le lock contient aussi schemars 0.9.0 transitif d'un autre crate — sans rapport, workflow-core résout bien sur 1.2.1.)*
- **npm : axe-core + @axe-core/playwright** — MPL-2.0, **0 CVE au 2026-07-18** (vérifié via **OSV** api.osv.dev, Snyk indisponible dans l'env → source concordante). Dev-only → **hors archive Hub**, CSP `connect-src 'none'` inchangée.
- **npm : @lingui/* côté Hub** — MIT. Aligner sur **6.4.0** (= Operator) pour éviter deux arbres Lingui divergents.
- **Aucun gate supply-chain npm ne bloque** : `web/audit-ci.json` = `{critical:true, allowlist:[]}` (vulnérabilités **critical-only**, **0 license-gate**) ; `deny.toml` = **Rust workspace uniquement** (MPL-2.0 déjà autorisé l.114, MIT l.106, mais ne s'applique qu'aux crates). Le **rouge CI-1 supply-chain** (cargo-deny LICENSES, async_executors/borrow-or-share) est **Rust-only, sans rapport** avec les deps npm de B — à ne pas confondre.
- **Node ≥22.19.0 REQUIS** par @lingui 6.4.0 (`engines` de cli/core/react/vite-plugin/macro = `>=22.19.0` ; `globSync` de `node:fs` absent avant Node 22.0). **⚠ correction** : « Node 22 suffisant » n'est vrai en CI **que par coïncidence** — `setup-node@v4` résout le pin flottant `"22"` vers le dernier 22.x (≥22.19.0). `tools/factory-operator/package.json` `engines: ">=22"` est **plus lâche** que la dépendance réelle, et **aucun `.nvmrc`** n'existe → un env Node 22.0–22.18 passerait l'engines mais échouerait EBADENGINE contre @lingui. **Recommandation** : `engines: ">=22.19.0"` + `.nvmrc` pinné. CI-2 (bug globSync/Node20) reste **fermée** aux 3 sites (`ci.yml:70`, `:176`, `supply-chain.yml:93`).

---

## S2 — Décisions historiques traversées

- **§Pre-launch (CLAUDE.md)** : défaut additif-0-bump (AM-12) — respecté par ProfileV1 (host→app local, non-signé).
- **Lingui = OVERRIDE PO** (memory `factory_universal_ux_pivot`) — i18n imposé, NE PAS re-débattre. Faisable sous CSP Hub (confirmé S1b).
- **Séparation Hub/Operator** (kickoff §20.7 ; memory `idea_hub_factory_ultracode_redesign` confinement 3 tiers) : Hub = app SBFB sandboxée iframe (`allow-scripts` sans `allow-same-origin`, `connect-src 'none'`) ; Operator = outil hôte privilégié **hors CSP** (`po_directive_factory_front_redesign`). **JAMAIS importer l'Operator dans le Hub** — module commun re-dérivé seulement.
- **tools/factory-ui jeté S80** (décision S70 SUPERSEDE S80 Phase B, CLAUDE.md) — confirme la caducité du chemin plan:366.
- **AM-3 exception ratifiée PO Option A** (memory `nexus_grid_pivot` : « // FRONTIER au remplissage ») — s'applique à ProfileV1 (cf. S4).

---

## S3 — Threat model (signal : EXECUTE) — INVARIANT CARDINAL needs[]

`needs[]` = **PII santé/handicap** (catégorie LINDDUN sensible, plus forte que « PII light » de l'Asset A4 actuel). L'invariant : **needs[] ne quitte JAMAIS le host** (0 occurrence dans DOM/logs/prompts/trace/bridge/provider — test B.3).

**7 vecteurs énumérés → fermetures requises :**

| Vecteur | Constat (fichier:ligne) | Sévérité | Fermeture B |
|---|---|---|---|
| **DOM (data-* )** | Fuite **DÉJÀ matérialisée** dans le code de référence : `preboot.js:367` **ET** `accessibility.ts:486` posent verbatim `root.setAttribute('data-needs', resolved.needs.join(' '))` (vérifié Read). *(3e site statique `index.html:2 data-needs=""` = slot vide, pas une fuite runtime.)* | **P0** | Module commun factorise `applyResolvedAxes()` needs-free (15 axes-effet seulement) ; `data-needs` reste extension Operator-only. Preboot Hub n'écrit JAMAIS data-needs. |
| **bridge postMessage** | `web/src/bridge/protocol.ts` : canal push `sbfb-bridge-event` (payload `z.unknown()`) = seul moyen d'injecter dans iframe sandbox sans `allow-same-origin` | **P0** | Payload event a11y = ProfileV1 projeté (15 axes), validé **Zod `.strict()`**, needs[] ABSENT du schéma. |
| **trace** | `nexus-trace-core/src/lib.rs:40-41` `TraceEvent.attributes: HashMap<String,Value>` ouvert, sérialisé (JSONL/canary signé) | **P1** | Champ needs `#[serde(skip)]` OU type d'ENTRÉE distinct du type de SORTIE ; test `needs_never_serialized_outward` (B.3). |
| **prompts LLM** | `sbfb-factory/src/llm_bridge.rs:84 assemble_prompt()` alimente Claude/Ollama/Network = **seul canal réseau externe non bloqué par CSP** | **P1** | Résolution STRICTEMENT côté rendu (présentation), JAMAIS côté génération. Aucun axe a11y n'entre dans assemble_prompt. |
| **Operator vs Hub** | data-needs acceptable Operator (local, hors réseau, hors CSP) / inacceptable Hub (untrusted iframe) | INFO | Justifie de NE PAS réutiliser `applyAccessibilityPreferences` verbatim. |
| **THREAT_MODEL** | v19, **aucune surface a11y/prefs** (seule mention : Asset A4 « consent PII light » l.62) | **P1** | Entrée **§18** DUE (docs-contrat frontière, README §6.12) : needs[] asset haute sensibilité, 7 vecteurs, mitigation projection needs-free, résiduel linkability accepté L. Bump v20. |
| **census/golden** | 8e contrat casse assert len==7 (cf. S4) | **P2** | Module dédié (cf. décision (a)). |

Principe : **n'envoyer que l'EFFET (motion=reduced), jamais la CAUSE (vestibular)**. L'invariant vaut **par construction**, pas par confiance dans la sandbox.

---

## S4 — Wire format / contrat (signal : EXECUTE)

- **DOMAIN = 0 besoin.** Census `DOMAIN_*_V1` = **25** (vérifié `grep -hoE 'const DOMAIN_[A-Z0-9_]+_V[0-9]+' crates/ | sort -u | wc -l` = 25), gelé par `check-frontier-contracts.sh:190 DOMAIN_CENSUS_FROZEN=25`. Décomposition : 24 dans `nexus-core-rs/src` (23 canonical.rs + KEYSTORE) + 1 `DOMAIN_TRACE_EVENT_V1` dans `nexus-trace-core` = 25 (le brief « grep nexus-core-rs » donne 24 : le 25e est hors nexus-core-rs — les deux sont vrais). ProfileV1 = projection host→app **locale, non-signée, non-gossipée** → **0 DOMAIN, 0 *_FORMAT_VERSION**. ⚠ **Piège** : inventer un DOMAIN pour « honorer » AM-3 ferait passer le census à 26 et **tripperait le tripwire** = un wire bump par définition — À NE PAS FAIRE.
- **Placement = module séparé.** `contracts.rs:143 assert_eq!(obj.len(), CONTRACT_NAMES.len(), "seven reserved identities")` + `CONTRACT_NAMES: [&str;7]` (l.129) + golden `contracts.schema.json`. **⚠ correction adversariale importante** : le registre `all_contract_schemas()` est **manuel** (`map.insert()`, `CONTRACT_NAMES` codé main, **0 auto-découverte**) → **déclarer un struct `#[derive(JsonSchema)]` est INERTE** tant qu'aucun insert ne l'ajoute. Donc la rupture n'est PAS causée par l'ajout du profil, mais **uniquement** par le choix (évitable) de l'enregistrer dans `all_contract_schemas()`. **Décision (a)** : profil dans `crates/sbfb-workflow-core/src/accessibility_profile.rs` avec son **propre golden** `accessibility_profile.schema.json` → les 7 identités et `contracts.schema.json` restent **strictement intacts**. Corroboré par le plan B.3 qui nomme déjà `accessibility_profile::tests::needs_never_serialized_outward`, et sémantiquement correct (le profil n'est pas un contrat-pack signé D1-D4).
- **Précédent EXACT** : `nexus-core-rs/src/schemas/shard.rs:130-135` et `:227-232` (frontière loopback consommée par runtime distinct = schemars + golden drift-gate, **ZÉRO tag `// FRONTIER:` domain/version** car non-signée). ProfileV1 = même classe. Détail mécanique : le gate `check-frontier-contracts.sh` grep `// FRONTIER: ` (colon+**espace**) ; la prose backtickée `` `// FRONTIER:` `` (colon+backtick) n'est PAS matchée → n'exige aucun domain/version. **Reprendre cette formulation backtickée pour ProfileV1.**
- **AM-3** (`kickoff §4.5 l.536` nomme explicitement « ProfileV1 ») : l'« étiquette » est satisfaite par (a) golden drift-gate dédié + (b) commentaire de frontière source à la shard.rs. Exception PO Option A « // FRONTIER au remplissage » → **PAS de DOMAIN**.
- **docs-contrat §6.12** : ProfileV1 EST une frontière (host→app lu par l'app iframe = runtime distinct). **AM-3 DEFERE GUIDE + llms.txt à AJ1.** → **Phase B ne livre PAS d'entrée GUIDE** (déviation ratifiée, pas un gap) ; le preflight NE DOIT PAS bloquer sur « GUIDE manquant en B ». Consigner ProfileV1 dans l'index des frontières pour reprise AJ1.
- **Parité Rust↔TS** : `check-frontier-contracts.sh` ne fait **AUCUN** check Rust↔TS ; le miroir TS Hub est neuf et non-gaté auto. Recommandation (non exigée par un gate B) : committer le JSON Schema généré comme artefact partagé référencé par le golden Rust **et** le miroir TS.

---

## Décisions d'implémentation tranchées

| # | Question | Décision |
|---|---|---|
| **(a)** | 8e identité du registre OU type séparé ? | **Type schemars SÉPARÉ** — `accessibility_profile.rs` + golden propre `accessibility_profile.schema.json`, HORS `all_contract_schemas()`. Assert len==7 et `contracts.schema.json` intacts (registre manuel : le struct déclaré est inerte). |
| **(b)** | Placement TS miroir + politique de résolution | **Module web NEUF** (module commun a11y), re-dérivé de `accessibility.ts` **sans importer l'Operator**. needs[] résout les 15 axes LOCALEMENT côté host puis est RETIRE de la projection. Le miroir ne vit **PAS** dans `examples/workflow-hub/` (qui consomme la projection via bridge) mais dans une couche web partagée. |
| **(c)** | Canal preboot host→app needs-free | **OUI** — preboot Hub n'écrit jamais data-needs ; injection via `sbfb-bridge-event` (Zod `.strict()`, needs absent du schéma). Module commun factorise `applyResolvedAxes()` needs-free. |
| **(d)** | DOMAIN/wire bump | **0 bump CONFIRMÉ** (census 25 gelé, ProfileV1 local non-signé). |
| **(e)** | Étiquette `// FRONTIER` + entrée GUIDE cette phase ? | **Golden drift-gate + commentaire de frontière backtické = OUI en B. Entrée GUIDE + llms.txt = NON (déféré AJ1, AM-3).** Entrée THREAT_MODEL §18 = OUI en B. |

---

## Périmètre AM-8 (fondation ≠ conformité)

**DANS B** : ProfileV1 (schéma + golden + résolution needs-free) · `examples/workflow-hub/` amorcé (preboot, tokens oklch, contrat mode guidé **déclaratif** — 3 projections implémentées en **T**) · harness a11y exécutable (axe-core wcag22aa, Playwright clavier, reduced-motion/forced-colors/zoom/RTL, scan anti-flash, gate profil-sans-needs) via module commun · socle Lingui Hub + 3 gates i18n + français + RTL (CSS dir distinct de Lingui) · `sprint83_at_matrix.md` (5 OS/AT réservée, owner/dispo/fallback/exit + trace 1er contact panel round 1 = W1).

**REPORTÉ hors B** : conformité réelle (audit WCAG 2.2 AA / ATAG / COGA vert) = **surfaces R..V + panel AI1** · 3 projections mode guidé implémentées = **Phase T** · clôture docs-contrat GUIDE + llms.txt ProfileV1 = **AJ1**. B **ouvre** G-A11Y-PREFS + G-I18N + 9 G-A11Y-* mais ces gates **ferment aux surfaces + AI1**, pas en B.

---

## Wire / census — synthèse

- 0 `DOMAIN_*_V1` neuf, 0 `*_FORMAT_VERSION`, census **25 inchangé** (gelé).
- ProfileV1 = golden dédié (draft 2020-12) + commentaire frontière backtické à la `shard.rs`.
- `contracts.rs` (7 identités) **non touché**.

---

## Invariant needs[] — fermetures (récap actionnable)

1. **Type** : needs en type d'ENTRÉE distinct du type de SORTIE, OU `#[serde(skip)]` — le schéma exporté `.v1` n'expose **aucune** propriété needs.
2. **DOM** : `applyResolvedAxes()` needs-free ; preboot Hub sans data-needs (les 2 sites Operator restent Operator-only).
3. **Bridge** : payload event Zod `.strict()`, needs absent.
4. **Trace/prompt** : aucun axe a11y dans `assemble_prompt` ni dans `TraceEvent.attributes`.
5. **Test B.3** : 0 occurrence de « needs » dans DOM sérialisé / trace / réseau (couvre **preboot + apply**) + `needs_never_serialized_outward` Rust.

---

## Questions ouvertes résiduelles

1. **EN 301 549** : quel numéro consigner ? V3.2.1 (harmonisé légal actuel = WCAG 2.1, incohérent cible 2.2 AA) vs V4.x draft (WCAG 2.2 AA, statut OJEU **non vérifiable depuis le repo**). Item §11 pendant (WA-16/Q6, `kickoff:885`). Seul repère repo = V3.2.1 en research (`workflow_agents…ultradeep:3064`).
2. **Parité Rust↔TS** : artefact JSON Schema partagé (golden Rust + miroir TS) vs obligation manuelle documentée — non exigé par un gate B, à arbitrer.
3. **Résiduel linkability** (motion=reduced + contrast=high + colorVision=safe corrèle profil santé) : accepté **L**, à consigner §18 (mitigé connect-src 'none' + effet≠cause), pas fermé.
4. **Version Lingui Hub** : pinner `^6.4.0` (= Operator) — confirmer au 1er commit i18n.
5. **Q12** (Research Packs non stagés) reste non tranché depuis Phase A — hors B, à ne pas perdre.

---

## Checklist d'implémentation (main-thread)

- [ ] **Avant code** : corriger `plan:366` (retirer/marquer caduque « `tools/factory-ui/readonly` ») + trancher le numéro EN 301 549 (WA-16).
- [ ] **Rust** : `crates/sbfb-workflow-core/src/accessibility_profile.rs` — struct schemars ProfileV1 (15 axes, needs en type d'entrée séparé/skip) + commentaire frontière **backtické** à la `shard.rs:130` (PAS de DOMAIN) + golden `accessibility_profile.schema.json` + test `needs_never_serialized_outward`. Vérifier `contracts.rs:143` **toujours vert** (len==7).
- [ ] **Vérifier** : `check-frontier-contracts.sh` reste vert (census 25) ; NE PAS toucher `DOMAIN_CENSUS_FROZEN`.
- [ ] **Web** : module commun a11y NEUF (`applyResolvedAxes()` needs-free) + miroir TS ProfileV1 (re-dérivé, **0 import Operator**) + preboot Hub anti-flash sans data-needs.
- [ ] **Hub** : `examples/workflow-hub/` amorcé (SBFB.json schema_version 2 + index.html, modèle `daisyui-animejs-showcase` : build-time, runtime 0-dep vendored) + tokens oklch + contrat mode guidé déclaratif.
- [ ] **Harness** : ajouter `axe-core` + `@axe-core/playwright` en **dev-dep** (`web/` + `operator`), **pinner la version au install** (npm view) ; Playwright clavier-only, reduced-motion/forced-colors/zoom/RTL, scan anti-flash, gate profil-sans-needs. Étendre `check-accessibility-system.sh` via module commun.
- [ ] **i18n** : socle Lingui Hub `^6.4.0` (catalogues **bundlés statiques**, pas import lazy, pour CSP Hub réelle) + 3 gates (no-literal scopé Hub / anti-verdict cross-locale / parité clés) + français + RTL (CSS dir/logical-properties). Recommander `engines: ">=22.19.0"` + `.nvmrc`.
- [ ] **THREAT_MODEL** : entrée §18 (needs[] asset haute sensibilité, 7 vecteurs, mitigation projection needs-free, résiduel linkability L) → bump v20.
- [ ] **Matrice AT** : `sprint83_at_matrix.md` (5 OS/AT réservée, owner/dispo/fallback/exit par ligne + trace 1er contact panel round 1 = W1).
- [ ] **NE PAS livrer en B** : entrée GUIDE/llms.txt ProfileV1 (AJ1) ; 3 projections mode guidé implémentées (T) ; conformité WCAG/ATAG/COGA verte (surfaces + AI1).
- [ ] **Wrap-up** : consigner ProfileV1 dans l'index des frontières (reprise AJ1) ; T1 E2E hermétique BLOQUANT-vert + T2 artefact JSON.