# Sprint 82 — Phase C — Review (restauration surfaces CI : GHA/GTK, Woodpecker, integration-nightly)

## Verdict: PASS

Review Workflow avant-plan (fan-out 3 lentilles adversariales indépendantes, arbre sale
→ agents synchrones). Diff = **5 fichiers YAML/TOML, +55/-0, 0 code source, 0 wire, 0 dep,
0 frontend**. Les 3 lentilles convergent : **le diff code est CLEAN**. Le seul CONCERN
(Lens C) porte **entièrement sur des obligations de body de commit** (commit pas encore
écrit), toutes P2/P3 et satisfaites par la rédaction du body ci-dessous — **aucun défaut
de code, aucune boucle de re-code**.

- **Lens A (correctness & complétude)** : **CLEAN**. 0 P0/P1/P2.
- **Lens B (adversariale « casse-le »)** : **CLEAN**. Toutes les attaques réfutées par les
  graphes de deps réels + parse config ; aucun défaut fabriqué.
- **Lens C (scope / process / docs-contrat / honnêteté)** : **CONCERN** → résolu par le body
  (C-1/C-2/C-3/C-5/C-6 ci-dessous). Code CLEAN.

## Vérification (ground-truth, outils consommateurs des fichiers édités)

- `actionlint` v1.7.12 (téléchargé) → **exit 0** sur `ci.yml` + `rust-ci.yml` + `integration-nightly.yml`.
- Parse YAML (`safe_load_all`) des 4 workflows → **valides**.
- Parse TOML (`tomllib`) de `.config/nextest.toml` → override `[[profile.ci.overrides]]`
  `{filter:'binary(multi_daemon)', slow-timeout:60s×3=180s}` correctement parsé **ET** scalaires
  `[profile.ci]` (retries=1, slow-timeout 30s global, status-level, failure-output, fail-fast)
  **+ sous-table `[profile.ci.junit]` préservés** (l'array-of-tables ne cannibalise rien).
- `cargo nextest list --profile ci -E 'binary(multi_daemon)'` → **exit 0**, liste **10 tests**
  (`nexus-coordinator-rs::multi_daemon` ×1 + `nexus-test-harness::multi_daemon` ×9) = le set `-E`
  exact de la nightly → l'override PARSE sous nextest ET le filtre matche les bons binaires.

### Exemption des suites lourdes §7.4 (justification écrite — canon README §7.4)

Phase C ne touche **AUCUN code source Rust/web** (`git diff --name-only` = 5 fichiers CI/config,
0 `.rs`/`.tsx`/`.ts`/`web/src`/`tools/factory-operator/src`/`Cargo.*`/`*_VERSION`). Re-lancer
`clippy/nextest/build --workspace` re-validerait du code **byte-identique au tip `1670251`** (déjà
vert à ce commit) → 0 signal neuf sur CE diff. La mémoire `feedback_full_failfast` n'est **PAS
violée** : son intention est de ne pas sauter un langage parce qu'on a touché l'autre ; ici NI Rust
NI web source n'est touché. La vérification faite exerce les **outils consommateurs** de chaque
fichier édité (nextest charge/valide l'override TOML ; actionlint valide les GHA ; parse valide
Woodpecker). Exemption canon-conforme + suffisante ; re-lancer le full workspace = théâtre.

## Findings consolidés

### Correctness / complétude (Lens A + B) — CLEAN

- **Coverage exhaustive** : inventaire des 12 workflows GHA + Woodpecker + scripts CI. Seul
  `nexus-launcher` (binaire feuille, aucun crate n'en dépend) linke GTK, uniquement sous
  `cfg(target_os="linux")` + feature `gtk`. Les seuls builds `--workspace`/Linux =
  `ci.yml(test)` + `rust-ci(clippy + leg ubuntu matrice test)` + `.woodpecker(rust-clippy/
  rust-test/rust-doctest)` → **tous reçoivent libgtk**. Builds ciblés (`build-worker -p
  nexus-worker`, `canary-monthly` + `integration-nightly -p nexus-shell-daemon`,
  `ci.yml factory-operator -p sbfb-factory`) = graphes gtk-clean (re-vérifiés `cargo tree`) →
  correctement non touchés. `deploy/supply-chain/mirror/phase-review/pkarr/shellcheck` = 0
  compile Rust.
- **Ordre correct** : dans Woodpecker, `apt-get` précède `rustup component add`/`cargo` (index 0) ;
  `rust-fmt` sans GTK (ne compile pas). Guards OS : matrice rust-ci `if: runner.os == 'Linux'`
  (win/mac exclus, corrects car GTK linux-only) ; job clippy rust-ci ubuntu-only (guard inutile).
- **K-R-13** : override profil `ci` (celui qu'`integration-nightly` utilise via `--profile ci`),
  180s ≥ 120s, miroir exact de l'override `profile.default` two-node-convergence (`nextest.toml:45`).
- **K-R-14** : `save-if` master-only correct sur `integration-nightly` (cron+dispatch only), miroir
  `rust-ci.yml`.
- **Paquet minimal** : `libgtk-3-dev` SEUL (features=["gtk"], `default-features=false` → libxdo OFF,
  appindicator dlopen runtime) — recette prouvée `docker/ci/Dockerfile:8`. Pas de libxdo-dev/
  libappindicator3-dev.
- **`permissions: contents: read`** (ci.yml) non-cassant : `upload-artifact@v4` utilise
  `ACTIONS_RUNTIME_TOKEN`, pas le scope `contents` ; corroboré — `rust-ci.yml` tourne DÉJÀ vert avec
  ce bloc + le même set d'actions.
- **0 régression silencieuse** : `git diff --stat` = 5 fichiers CI/config, 0 source/wire/version.
- **sudo** : présent sur GHA (`ubuntu-latest` non-root, sudo passwordless), absent sur Woodpecker
  (root in-container). Corrects.

### Gap latent unique (INFO, hors scope Phase C — carry Phase T)

- **`release.yml`** (`push: tags: ["v*"]`) build `nexus-launcher` sur ubuntu via
  `scripts/release-attest.sh` **sans** step libgtk → échec GTK latent au **prochain tag**. Non-régression
  Phase C (tag-triggered, hors des 3 verts push-gate PO-4). **Explicitement flaggé Phase T** par le
  preflight (`sprint82_phase_c_preflight.md:284-287`). Recommandation : garantir que ce carry survit
  en item tracké pour que le prochain tag ne soit pas le mécanisme de découverte.

### Obligations de body (Lens C — CONCERN résolu par le commit)

- **C-1 [P2]** : `## Verification` du body écrit explicitement l'exemption §7.4 (0 source Rust/web ;
  code identique à `1670251` ; config validée par outils consommateurs) et ne prétend JAMAIS
  « 3 blocs verts ».
- **C-2 [P2]** : `## Carry closure` marque **S81-J-2 = PARTIAL** (calibré Phase C via save-if +
  slow-timeout / RUN réel différé Phase T) — PAS CLOSED (le workflow existe déjà + est déjà
  `workflow_dispatch`-able ; il n'a toujours jamais tourné).
- **C-3 [P2]** : `## Contexte` décrit honnêtement la réalité corrigée (surfaces réparées + statut
  ATTENDU non encore observé) SANS ré-affirmer le claim stale « Woodpecker opérationnel ».
  Réconciliation prose CLAUDE.md correctement différée Phase T (on réconcilie APRÈS le run vert).
- **C-5 [P3]** : `## Scope cuts` disclose l'addition `permissions: contents: read` (hors des 4
  carries nommés) ; SHA-pinning `@v4→SHA` correctement DÉFÉRÉ (carry `CI-ACTION-SHA-PINNING`).
- **C-6 [P3]** : `## Delta tests` = **+0** explicite sur toutes les suites (l'override change la
  POLITIQUE de timeout, pas le SET de tests → count nextest inchangé = invariant refacto).
- **C-4 / C-7 [CLEAN]** : provenance in-code = passé immuable (anti STALE-PHASE-K ; fichiers CI hors
  périmètre `check-frontier-contracts.sh` de toute façon) ; T1/T2 N-A légitimes (0 `web/src` touché).

### Note P3 (Lens A) — seuil d'alerte K-R-13

Le plan écrit « slow-timeout `binary(multi_daemon) >= 120s` ». L'impl met `period=60s`
(terminate-after=3 → hard-kill 180s, alertes « slow » cosmétiques à 60s/120s). **Intent respecté** :
aucun run correct n'est TUÉ avant 180s ≥ 120s ; miroir exact du précédent `profile.default`. Retenu
(plus de marge que `period=120s`).

## Type de commit recommandé

`fix(ci): Sprint 82 Phase C — restauration surfaces CI (GHA/GTK, Woodpecker, integration-nightly)`.
`fix` = type le plus exact (répare une CI cassée) + présent dans la liste canonique du gate
(`README.md` §4, `feat|fix|docs|test|refactor`). `chore` off-canon ; `feat` sur-vend.

## Codex reconciliation

Codex GPT-5.6 Sol (reasoning max, CLI 0.144.1) exécuté → output brut dans
`.planning/active/sprint82_phase_c_codex_review.md` (NON réécrit). **Verdict global Codex :
GAPS-P2/P3 (documentables)** — 9 items, **7 CONFIRME, 0 GAP, 2 PARTIEL**. Les deux PARTIELs
sont documentables, aucun changement de code requis (pas de boucle P0/P1) :

1. **Livrable 5 (save-if) — PARTIEL** : Codex note que `master || main` est plus large que
   « master-only » littéral. **Résolution : GARDÉ tel quel** — c'est le **miroir exact de
   `rust-ci.yml:97`** (convention établie du repo ; cohérence des 3 workflows). L'intent K-R-14
   (aucun save depuis branche feature/PR) est tenu ; Codex confirme lui-même « l'effet présent
   est bien master-only » (le repo n'a pas de branche `main`, `origin/HEAD`→master). Le `|| main`
   est un garde-fou future-proof (rename master→main éventuel), pas un élargissement du vecteur
   de poison. Documenté dans le body.
2. **Propriété A (release.yml) — PARTIEL** : `release.yml` build `nexus-launcher` sur Ubuntu sans
   GTK. Codex confirme « aucune surface Linux `--workspace` n'est oubliée » et qualifie release.yml
   de « vrai carry fonctionnel, mais pas un gap de la Phase C telle que bornée aux push-gates »
   (tag/dispatch, préexistant, absent du diff). **Différé Phase T** (déjà tracké preflight
   `:284-287` + review). À fermer avant le prochain tag.

Les 7 CONFIRME couvrent : ci.yml (permissions + GTK job test), rust-ci (clippy + matrice Linux),
Woodpecker (3 steps rust), override nextest (180s, scalaires + junit préservés, 10 tests matchés),
propriété B (override effectif profil ci), propriété C (`contents: read` ne casse aucun step),
propriété D (0 régression hors CI/config, `git diff --check` propre).

**Réserve honnête (Codex)** : Codex note que « GHA rouge depuis mai » n'est PAS démontrable par le
seul dépôt (`git blame` confirme l'intro des deps GTK le 2026-05-12, mais pas l'état des runs
GitHub). Cohérent avec la posture Phase C : le RUN vert réel des 3 verts est **différé Phase T**
(gate push PO-4) ; le body ne prétend PAS « CI verte observée », seulement « surfaces réparées,
statut attendu ».

Séquence respectée : review PASS-PENDING → Codex → réconciliation (0 fix, 2 P2/P3 documentés) →
promotion `## Verdict: PASS` → commit.
