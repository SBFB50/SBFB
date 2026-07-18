# Sprint 82 — Phase C — Preflight G8 (restauration CI : GHA/GTK, Woodpecker E2E, integration-nightly)

## Verdict: PLAN-ADAPT

Le plan Phase C nomme « installer `libgtk-3-dev` dans les **2 jobs `ci.yml`** » mais
cette formulation est **factuellement incomplete ET imprecise**, verifie sur disque :

1. **Imprecise** : dans `ci.yml`, UN SEUL job (`test`) construit `--workspace` et a
   besoin de GTK. Le job `factory-operator` construit `cargo build -p sbfb-factory`
   (deps `sbfb-manifest` + `nexus-core-rs`, PAS `nexus-launcher`) → **0 GTK**.
2. **Incomplete** : `rust-ci.yml` (job `clippy` + leg ubuntu de la matrice `test`)
   ET `.woodpecker/ci-linux.yml` (`rust-clippy` / `rust-test` / `rust-doctest`)
   construisent AUSSI `--workspace` sur Linux et sont **tous les deux dans le gate
   push PO-4 « 3 verts »**. Le fix GTK doit y atterrir aussi, sinon PO-4 est intenable.

Cette extension est une **correction de completude adossee a une evidence OSS/on-disk
concrete** (`nexus-launcher` = membre workspace `Cargo.toml:9` + feature-gate Linux
`crates/nexus-launcher/Cargo.toml:58-60` + recette prouvee `docker/ci/Dockerfile:8`),
**PAS** une remise en cause d'une decision Day-0 figee (S2-F8 : la strategie
Woodpecker-first garde GHA en fallback, aucun gel ne contredit la reparation CI).
Etendre le fix aux fichiers reellement concernes est donc un **PLAN-ADAPT legitime**,
pas un DESIGN-CONFLICT.

Repartition des signaux : **S1a=clean**, **S1b=adapt**, **S2=adapt**, **S3=scope-note**,
**S4=wire-clean + frontier-scope-note**. Verification adversariale : les 5 scans
convergent sur la MEME image factuelle ; le seul point OSS a « clean » (S1a) confirme
que le paquet apt est correct et minimal — il ne contredit pas l'axe scope. Aucun scan
ne remonte a EXECUTE (le plan verbatim laisse rust-ci + Woodpecker rouges), aucun ne
descend a DESIGN-CONFLICT. **Agregat = PLAN-ADAPT.**

## S1 — OSS prior-art + deps/build (fusion S1a + S1b)

Signal fusionne : **S1a=clean / S1b=adapt**. Invariant **0-dep runtime TENU**.

- **Paquet apt correct ET minimal = `libgtk-3-dev` SEUL.** Le repo compile
  `tray-icon 0.24` + `muda 0.19` avec `default-features = false, features = ["gtk"]`
  sur la cible Linux uniquement (`crates/nexus-launcher/Cargo.toml:58-60`, verifie).
  C'est un jeu de features **plus etroit** que les defauts des crates :
  - `tray-icon` : `default = ["libxdo","gtk"]`, `gtk = ["muda/gtk","dep:libappindicator"]`,
    `libxdo = ["muda/libxdo"]`. `default-features=false` + `gtk` seul → **`libxdo` DESACTIVE**.
  - `muda` : `gtk = ["dep:gtk"]` ; tray-icon tire `muda/gtk` (pas `muda/libxdo`) ; rien
    dans le workspace ne re-active `libxdo` par unification de features (absent de `Cargo.lock`).
  - `libappindicator 0.9` depend de `gtk-sys 0.18` (GTK3 via pkg-config → `libgtk-3-dev`)
    mais **dlopen** l'appindicator au RUNTIME via `libloading` (tauri-apps PR #38) →
    `libappindicator3-dev` / `libayatana-appindicator3-dev` **PAS requis au build/CI**.
- **NE PAS copier la recette generique du README tray-icon** (`libgtk-3-dev libxdo-dev
  libappindicator3-dev`) : c'est la recette **default-features**. `libxdo-dev` et
  `libappindicator3-dev` seraient du poids mort + elargiraient la surface CI pour 0 benefice.
- **Preuve empirique** : `docker/ci/Dockerfile:8` installe **UNIQUEMENT** `libgtk-3-dev`
  et l'image `sbfb-ci` build `clippy --workspace --all-targets` vert. `libgtk-3-dev` est
  l'umbrella apt dont les `Depends` tirent transitivement `libglib2.0-dev`, `libatk1.0-dev`,
  `libcairo2-dev`, `libgdk-pixbuf-2.0-dev`, `libpango1.0-dev` → tous les `.pc` que
  `glib-sys`/`gdk-sys`/`gtk-sys`/`atk-sys`/etc. lient au build.
- **`sudo` requis sur GHA** (runner `ubuntu-latest` = user `runner` non-root, sudo
  passwordless) ; **PAS de `sudo` sur Woodpecker** (image `rust:1.94` = root in-container,
  miroir `docker/ci/Dockerfile:8` qui omet sudo).
- **0 dep Rust ajoutee** : working tree propre au tip `1670251`, Phase C = YAML/TOML/doc
  only → `Cargo.lock` inchange, 0 surface CVE Rust nouvelle. `libgtk-3-dev` non-pinne est
  correct (dev-tool build-only, distro GPG-authentifiee, miroir Dockerfile ; pinner une
  version distro est fragile pour un gain nul).
- **actionlint** = validateur statique canonique des workflows GHA. Install sans Go :
  `curl -sL https://raw.githubusercontent.com/rhysd/actionlint/main/scripts/download-actionlint.bash | bash`
  (depose un binaire dans cwd) ; ou `go install github.com/rhysd/actionlint/cmd/actionlint@latest` ;
  ou `brew install actionlint`. Invocation : `actionlint .github/workflows/integration-nightly.yml`
  (exit != 0 sur findings). A lancer sur TOUS les workflows edites en verification Phase C.

## S2 — Decisions historiques traversees + contradiction Woodpecker

Signal scan : **adapt**. Aucune decision gelee rouverte.

- **Cause racine GHA-rouge = commit `b6a93a8`** « fix(sprint60): Linux .deb installer
  validated — GTK features » (2026-05-12) : ajoute le bloc
  `[target.'cfg(target_os="linux")'.dependencies]` tray-icon/muda `features=["gtk"]`.
  Colle exactement a « GHA CI ROUGE depuis 2026-05 » (CLAUDE.md). `nexus-launcher` est
  **le SEUL** crate du workspace tirant tray-icon/muda ; membre workspace → tout
  `--workspace`/`--all-targets` Linux lie GTK.
- **La recette `libgtk-3-dev` vit UNIQUEMENT dans `docker/ci/Dockerfile:8`** (ajoutee
  S75 Phase G) — jamais portee sur aucun workflow GHA ni Woodpecker. Seul `apt-get` dans
  `.github/workflows/` = `shellcheck.yml:18` (shellcheck), aucun libgtk.
- **CONTRADICTION WOODPECKER — RESOLUE (voir section dediee plus bas)** : `.woodpecker/
  ci-linux.yml` run `clippy/test/doctest --workspace` sur `rust:1.94@sha256` **nu**, sans
  libgtk → ces 3 steps DOIVENT etre rouges pour la meme cause. Le claim CLAUDE.md
  « Woodpecker operationnel (fmt/clippy/cargo test workspace + doctest) » est **STALE**
  (Woodpecker deploye vert S55 2026-05-05, AVANT que le gate GTK n'atterrisse le 2026-05-12).
- **Scope reel > « 2 jobs ci.yml »** : `rust-ci.yml` (clippy ubuntu + leg ubuntu de la
  matrice test) ET Woodpecker (rust-clippy/rust-test/rust-doctest) partagent le meme gap.
  PO-4 « 3 verts » = Woodpecker ci-linux + rust-ci 3-OS + integration-nightly → **les deux
  doivent recevoir libgtk**, quelle que soit la branche ARB-CI-1.
- **Aucun gel Day-0 ne s'oppose a la reparation GHA** : `docs/architecture/
  SELF_HOSTED_BUILD.md` cadre Woodpecker-first avec GHA « descendant » en fallback/second-avis,
  jamais « abandonner GHA ». `git log` DEVIATION/rejected/scope-cut sur `.github`/`.woodpecker`/
  `docker/ci` = 0 gel de design CI.
- **K-R-13 / K-R-14 / S81-J-2** = fixes concrets d'une-a-quelques lignes (details S3/S4).

## S3 — Threat model / supply-chain CI

Signal scan : **scope-note**. Le threat model §5.8 couvre les deps de paquets, PAS le
SHA-pinning des actions GHA → pinner les actions est **hors-scope** (gold-plating).

- **`ci.yml` n'a AUCUN bloc `permissions`** (verifie) → ses 2 jobs heritent du scope
  `GITHUB_TOKEN` par defaut du repo (potentiellement write). `rust-ci.yml:43-44` et
  `integration-nightly.yml:29-30` declarent deja `permissions: contents: read`. `ci.yml`
  tourne sur `pull_request` → une action `@v4` re-taggee/compromise sur un run de PR
  s'executerait avec un token write.
  **Durcissement in-scope, cheap** : ajouter un `permissions:\n  contents: read` au niveau
  workflow de `ci.yml`. **Verifie sur** : `upload-artifact@v4` utilise
  `ACTIONS_RUNTIME_TOKEN` (pas le scope `contents` du `GITHUB_TOKEN`) → restreindre a
  `contents: read` **ne casse PAS** les steps `[10d]`/`[9]` d'upload d'artefact. **Retenu**
  (P2) : on edite deja `ci.yml`, et cela aligne les 3 workflows + borne le blast-radius des
  actions non-pinnees.
- **SHA-pinning des actions `@v4` → SHA** : hardening legitime mais NON requis par le threat
  model SBFB (le seul precedent de pinning est les IMAGES conteneur Woodpecker, S54). Le
  bundler ici serait du gold-plating. **DEFER** vers un carry dette dedie (`CI-ACTION-SHA-PINNING`).
- **`libgtk-3-dev` non-pinne** = posture supply-chain acceptable (build-only, distro
  GPG-authentifiee, miroir `docker/ci/Dockerfile:8`). Ne PAS pinner la version.
- **K-R-13** (securite-adjacent, dans COVERS) : `profile.ci` hard-kill a 90s, pas d'override
  `binary(multi_daemon)` → un test lent-mais-correct est tue. Fix = override profile.ci (voir S4).
- **K-R-14** : `integration-nightly` ne tourne que sur cron + workflow_dispatch (jamais
  `pull_request`) → le vecteur de cache-poisoning que `save-if` defend est quasi-vacant ; le
  one-liner ferme le carry a cout nul, coherent avec `rust-ci.yml:97`.

## S4 — Wire format / canonical + frontiere process

Signal scan : **wire CLEAN + frontier scope-note**. **0 bump wire, 0 canonical, 0 version.**

- Phase C touche **uniquement** : `.github/workflows/{ci,rust-ci,integration-nightly}.yml`,
  `.woodpecker/ci-linux.yml`, `.config/nextest.toml` et (differe Phase T) la prose CLAUDE.md.
  **0 source crate, 0 `web/src`, 0 `tools/factory-operator/src`, 0 `*_VERSION`, 0 octet
  canonical.** Pre-launch protocol policy (FeedEntry/ProjectAnnouncement/CuratorList/…)
  **NON engagee**.
- **T1 = N-A-no-frontend-change** TIENT : cabler une suite existante (`npm run test:e2e`,
  `test:coverage`, `scan-en-strings.sh`) dans un step CI invoque des scripts existants =
  plomberie CI, PAS un changement de code front. La lecon S81-J-1 (Playwright vert obligatoire)
  se declenche sur une edition `web/src`/`factory-operator/src`, que Phase C ne fait pas.
  **Garde-fou** : si un step fallback edite `web/src`, T1 escalade — verifier qu'aucune telle
  edition ne se glisse.
- **T2 = N-A** (run reel differe Phase T) coherent : l'edit `.config/nextest.toml` est de la
  config test-runner, pas un wire format. Il declenche `rust-ci` via le path-filter
  (`rust-ci.yml:24`), attendu.
- **Frontiere process (CLAUDE.md CI claim)** : split canonique code-YAML-en-C /
  doc-reconcile-en-T (README §6.12 + §4). **Obligation in-phase** : le body de commit Phase C
  doit decrire honnetement la nouvelle realite CI (surfaces reparees + statut attendu) et ne
  PAS re-affirmer le claim Woodpecker stale ; la reconciliation de la prose « Etat actuel »
  CLAUDE.md est **intentionnellement differee Phase T**.

## Resolution de la contradiction Woodpecker

**Verdict : claim STALE (hypothese a), confiance HAUTE.**

- **Fait** : `.woodpecker/ci-linux.yml` run `cargo clippy --workspace --all-targets` (step
  `rust-clippy`, ligne 26), `cargo test --workspace` (`rust-test`, 31), `cargo test
  --workspace --doc` (`rust-doctest`, 36) sur `image: rust:1.94@sha256:b644...` — l'image
  **stock** Debian bookworm, sans aucun step `apt-get libgtk`.
- **Deduction** : `nexus-launcher` (membre workspace `Cargo.toml:9`, feature-gate GTK Linux
  `crates/nexus-launcher/Cargo.toml:58-60`) exige `libgtk-3-dev` pour tout build `--workspace`
  Linux → ces 3 steps **ne peuvent pas** compiler `glib-sys`/`gtk-sys` → **rouges**.
- **Refutation des hypotheses alternatives** : agent docker-backend (`SELF_HOSTED_BUILD.md`)
  → les commandes tournent DANS l'image SHA-pinnee → le GTK de l'hote **ne peut pas fuiter**.
  Le SHA force le digest upstream → pas d'image custom. `default-features=false` mais `gtk`
  explicitement active → pas d'exclusion par feature-flag. → hypotheses (b) host-leak /
  (c) custom-image / (d) feature-exclusion **rejetees**. Un seul fichier `.woodpecker` existe.
- **Datation** : Woodpecker deploye vert S55 (2026-05-05), gate GTK atterri `b6a93a8`
  (2026-05-12) → le claim precede le gate → **stale**, pas faux-a-l'origine.
- **Resolution** :
  1. **Ajouter `libgtk-3-dev` aux steps rust-clippy / rust-test / rust-doctest** de
     `.woodpecker/ci-linux.yml` (**PAS** `sudo` — root in-container). Requis **quelle que soit
     la branche ARB-CI-1** : Woodpecker est un des 3 verts PO-4 ET actuellement rouge.
     `rust-fmt` **n'a PAS besoin** de GTK (`cargo fmt` ne compile pas).
  2. **Check empirique live** : inspecter l'historique de build `ci.sbfb.world` (ou declencher
     un run manuel) pour observer le statut reel `rust-clippy`/`rust-test`. **NON bloquant** —
     le fix est deterministe independamment du resultat ; ce check n'est qu'une confirmation.
     *Caveat env* : peut etre inaccessible en session (reseau) → la conclusion « rouge » est
     HAUTE-confiance-inferee, pas byte-observee ; le fix reste correct.
  3. **Reconciliation prose CLAUDE.md finalisee Phase T** (le claim « Woodpecker operationnel »
     devient exact APRES le fix).

## Fichiers a editer + steps exacts

### 1. `.github/workflows/ci.yml`

**(a) Bloc `permissions` au niveau workflow** — inserer apres le bloc `env:` (ligne 14-16),
avant `jobs:` (ligne 18) :

```yaml
permissions:
  contents: read
```

**(b) Step libgtk dans le job `test`** — inserer entre « Setup Rust toolchain » (28-31) et le
step de cache (33), ou juste avant `[2] cargo clippy` (46). Runner `ubuntu-latest` fixe → pas
de guard :

```yaml
      - name: Install GTK (nexus-launcher links gtk on Linux)
        run: sudo apt-get update && sudo apt-get install -y --no-install-recommends libgtk-3-dev
```

**NE PAS** ajouter de step libgtk au job `factory-operator` (build `-p sbfb-factory` seul, 0 GTK).

### 2. `.github/workflows/rust-ci.yml`

**(a) Job `clippy`** — inserer entre « Install Rust toolchain » (84-87) et le cache (89), ou
juste avant `cargo clippy` (100). Job `ubuntu-latest` fixe → pas de guard :

```yaml
      - name: Install GTK (nexus-launcher links gtk on Linux)
        run: sudo apt-get update && sudo apt-get install -y --no-install-recommends libgtk-3-dev
```

**(b) Job `test` (matrice 3-OS)** — inserer avant `cargo nextest run` (141), **GUARDE Linux**
(la matrice inclut windows-latest/macos-14) :

```yaml
      - name: Install GTK (nexus-launcher links gtk on Linux)
        if: runner.os == 'Linux'
        run: sudo apt-get update && sudo apt-get install -y --no-install-recommends libgtk-3-dev
```

### 3. `.woodpecker/ci-linux.yml`

Chaque step tourne dans une image fraiche → l'`apt-get` doit etre ajoute **en tete des
`commands:` de CHAQUE step** compilant `--workspace`. **PAS de `sudo`** (root). NE PAS ajouter
a `rust-fmt` (ne compile pas).

`rust-clippy` (22-26) :
```yaml
    commands:
      - apt-get update && apt-get install -y --no-install-recommends libgtk-3-dev
      - rustup component add clippy
      - cargo clippy --workspace --all-targets --locked -- -D warnings
```

`rust-test` (28-31) :
```yaml
    commands:
      - apt-get update && apt-get install -y --no-install-recommends libgtk-3-dev
      - cargo test --workspace --locked
```

`rust-doctest` (33-36) :
```yaml
    commands:
      - apt-get update && apt-get install -y --no-install-recommends libgtk-3-dev
      - cargo test --workspace --locked --doc
```

### 4. `.config/nextest.toml`

Ajouter un override `profile.ci` pour `binary(multi_daemon)` (K-R-13). Miroir exact du schema
`profile.default.overrides` existant (lignes 37-45, `60s × 3 = 180s`). Inserer apres le bloc
`[profile.ci]` (avant `[profile.ci.junit]` ligne 80) :

```toml
# K-R-13: the relay-gated multi_daemon class (integration-nightly.yml,
# SBFB_INTEGRATION=1) boots real iroh nodes over the network; a
# slow-but-correct run must not be killed by the global 90s ci cap.
# 60s × 3 = 180s hard-kill (warns at 60s/120s), matching the
# two-node-convergence override in profile.default.
[[profile.ci.overrides]]
filter = 'binary(multi_daemon)'
slow-timeout = { period = "60s", terminate-after = 3 }
```

`binary(multi_daemon)` matche `crates/nexus-test-harness/tests/multi_daemon.rs` ET
`crates/nexus-coordinator-rs/tests/multi_daemon.rs` (coherent avec le `-E` de la nightly).
180s >= 120s (K-R-13) avec marge.

### 5. `.github/workflows/integration-nightly.yml`

Ajouter `save-if` master-only au cache Swatinem (K-R-14), miroir `rust-ci.yml:97`. Inserer
dans le bloc `with:` du step « Cache cargo registry + target » (43-48) :

```yaml
          save-if: ${{ github.ref == 'refs/heads/master' || github.ref == 'refs/heads/main' }}
```

*(Near-vacant : le workflow ne tourne que cron+dispatch, tous deux depuis master ; le one-liner
ferme mecaniquement le carry a cout nul.)*

### NON edite en Phase C

- **`CLAUDE.md`** : reconciliation du claim CI **differee Phase T** (plan:119,126). Le body de
  commit Phase C doit neanmoins decrire la realite corrigee (ne pas re-affirmer le claim stale).
- **`.github/workflows/release.yml`** : build `nexus-launcher` linux-x86_64 (`:30`, via
  `scripts/release-attest.sh`) **sans** step libgtk → **echec GTK latent au tag release**.
  HORS scope Phase C (pas un des 3 verts PO-4, tag-triggered). **Flag Phase T** : ajouter
  libgtk au leg Linux de release.yml avant le prochain tag, meme classe de correctness.
- **`docs/claude/README.md:2758`** (bloc Docker pre-push sur `rust:1.94` nu, meme gap latent) :
  reconciliation Phase T (process-doc).

## Adaptations de plan (PLAN-ADAPT)

1. **Etendre le fix libgtk a `rust-ci.yml`** (job `clippy` + leg ubuntu de la matrice `test`,
   guarde `if: runner.os == 'Linux'`) — sinon « rust-ci 3-OS vert » (PO-4) est intenable.
2. **Etendre le fix libgtk a `.woodpecker/ci-linux.yml`** (rust-clippy + rust-test +
   rust-doctest, **sans sudo**) — Woodpecker est un des 3 verts PO-4 ET actuellement rouge
   (claim stale). Requis quelle que soit la branche ARB-CI-1.
3. **Preciser « 2 jobs ci.yml »** : UN SEUL job (`test`) de `ci.yml` a besoin de GTK ; le job
   `factory-operator` (build `-p sbfb-factory`) et `integration-nightly` (build
   `-p nexus-shell-daemon`) n'en ont PAS besoin.
4. **Ajouter `permissions: contents: read`** au niveau workflow de `ci.yml` (aligne les 3
   workflows, borne le blast-radius des actions `@v4` non-pinnees ; sur — upload-artifact
   utilise `ACTIONS_RUNTIME_TOKEN`).
5. **`nextest.toml`** : override `[[profile.ci.overrides]]` `binary(multi_daemon)` slow-timeout
   `60s × 3 = 180s` (K-R-13), miroir de l'override profile.default existant.
6. **`integration-nightly.yml`** : `save-if` master-only au cache Swatinem (K-R-14, one-liner).
7. **Verification Phase C** : `actionlint` sur les workflows edites + check empirique live
   `ci.sbfb.world` (statut rust-clippy/rust-test) — NON bloquant, prose CLAUDE.md finalisee T.
8. **Fallback ARB-CI-1 (documente, NON primaire)** : si le runner GHA echoue ENCORE apres le
   fix GTK, cabler les 2 suites Playwright E2E + coverage web + `scan-en-strings` dans
   `.woodpecker/ci-linux.yml`. Defaut PO = GHA-GTK-first (E2E reste dans `ci.yml`).
9. **Paquet apt = `libgtk-3-dev` SEUL** — NE PAS copier la recette README generique
   (`libxdo-dev` + `libappindicator3-dev` inutiles pour ce jeu de features).

## Scope confirme (livrable Phase C, 0-wire / 0-dep / 0-frontend)

1. `ci.yml` : `permissions: contents: read` + step libgtk (job `test` seul).
2. `rust-ci.yml` : step libgtk (clippy + leg ubuntu matrice test guarde Linux).
3. `.woodpecker/ci-linux.yml` : apt libgtk (rust-clippy/rust-test/rust-doctest, sans sudo).
4. `.config/nextest.toml` : override profile.ci `binary(multi_daemon)` 180s.
5. `integration-nightly.yml` : `save-if` master-only.
6. Verification : `actionlint` + check live Woodpecker (non bloquant) ; body de commit honnete.

**Differe Phase T** (per plan) : reconciliation prose CLAUDE.md CI claim ; run reel des 3 verts
(PO-4 push gate) ; note release.yml/README.md:2758 GTK latent.

## Verdict: PLAN-ADAPT

**recommended apt = `libgtk-3-dev` SEUL** (jamais libxdo-dev / libappindicator3-dev).
**Woodpecker = STALE claim (hyp. a), confiance HAUTE → libgtk aux 3 steps rust + check live
+ reconciliation prose Phase T.** Aucun DESIGN-CONFLICT (aucun gel Day-0 en conflit). Aucun
blocage : coder les 5 fichiers + verification.

## Evidence (file:line)

- **GTK root cause** : `crates/nexus-launcher/Cargo.toml:58-60`
  (`[target.'cfg(target_os = "linux")'.dependencies]` tray-icon/muda `features=["gtk"]`,
  `default-features=false` lignes 19-20) ; `Cargo.toml:9` (membre workspace) ;
  `docker/ci/Dockerfile:5-8` (recette libgtk-3-dev seule, prouvee).
- **ci.yml** : `test` job `--workspace` clippy `:47` + test `:50` ; **aucun** bloc
  `permissions` (verifie) ; `factory-operator` build `-p sbfb-factory` `:157` (0 GTK) ;
  upload-artifact `@v4` `:97,:197`.
- **rust-ci.yml** : `permissions: contents: read` `:43-44` ; clippy `--workspace`
  `:100-101` ; test matrice ubuntu/windows/macos `:117-121`, nextest `--workspace` `:141-142`,
  doctest `:144-145` ; save-if master-only `:97,:133`.
- **integration-nightly.yml** : cron+dispatch `:23-27` ; `permissions: contents: read`
  `:29-30` ; build `-p nexus-shell-daemon` `:57` + nextest `-E 'binary(multi_daemon)'` `:59-65`
  (0 launcher, 0 GTK) ; Swatinem cache SANS save-if `:43-48`.
- **.woodpecker/ci-linux.yml** : `rust-fmt` `:16-20` (0 compile) ; `rust-clippy --workspace`
  `:22-26` ; `rust-test --workspace` `:28-31` ; `rust-doctest` `:33-36` ; image
  `rust:1.94@sha256:b644...` (stock, sans libgtk) ; factory-operator steps node-only, PAS
  d'E2E Playwright/coverage/scan-en-strings.
- **nextest.toml** : `[profile.ci]` slow-timeout `period="30s" terminate-after=3` = 90s `:69` ;
  PAS de `[[profile.ci.overrides]]` ; override two-node-convergence `60s×3` sous
  `[[profile.default.overrides]]` `:37-45`.
- **Seul apt-get GHA** = `shellcheck.yml:18` (shellcheck) ; release.yml `nexus-launcher`
  linux `:30`.
- **Plan** : `.planning/active/sprint82_plan.md:108-126` (Goal/Covers/Livrables/Testabilite
  Phase C).
