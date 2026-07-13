Le diff suivi est bien de **5 fichiers, +55/-0**, sur la branche `master`. Deux documents `.planning/active/` non suivis existent en plus, mais ne font pas partie de `git diff`.

La cause GTK est confirmée par le code. En revanche, l’affirmation historique « GHA rouge depuis mai » n’est pas démontrable par le seul dépôt : `git blame` confirme l’introduction des dépendances GTK le 12 mai 2026, mais pas l’état des runs GitHub.

### Livrable 1 : `ci.yml` — permissions et GTK

- Statut : CONFIRME
- Fichier(s) : [.github/workflows/ci.yml:22](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/ci.yml:22), [.github/workflows/ci.yml:57](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/ci.yml:57), [.github/workflows/ci.yml:149](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/ci.yml:149)
- Evidence :

```yaml
22: permissions:
23:   contents: read
25: jobs:
26:   test:
```

```yaml
57: - name: Install GTK (nexus-launcher links gtk on Linux)
58:   run: sudo apt-get update && sudo apt-get install -y --no-install-recommends libgtk-3-dev
60: - name: "[2] cargo clippy"
61:   run: cargo clippy --workspace --all-targets --locked -- -D warnings
64:   run: cargo test --workspace --locked
```

Le second job reste étroit et sans installation GTK :

```yaml
149: factory-operator:
151:   runs-on: ubuntu-latest
170:   - name: Build Operator server (hermetic T1 host)
171:     run: cargo build -p sbfb-factory
```

### Livrable 2 : `rust-ci.yml` — clippy et matrice test

- Statut : CONFIRME
- Fichier(s) : [.github/workflows/rust-ci.yml:80](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/rust-ci.yml:80), [.github/workflows/rust-ci.yml:151](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/rust-ci.yml:151)
- Evidence :

```yaml
80: runs-on: ubuntu-latest
103: - name: Install GTK (nexus-launcher links gtk on Linux)
104:   run: sudo apt-get update && sudo apt-get install -y --no-install-recommends libgtk-3-dev
106: - name: cargo clippy --workspace --all-targets
107:   run: cargo clippy --workspace --all-targets --locked -- -D warnings
```

```yaml
151: - name: Install GTK (nexus-launcher links gtk on Linux)
152:   if: runner.os == 'Linux'
153:   run: sudo apt-get update && sudo apt-get install -y --no-install-recommends libgtk-3-dev
155: - name: cargo nextest run (workspace)
156:   run: cargo nextest run --workspace --profile ci --locked
```

La matrice reste `ubuntu-latest`, `windows-latest`, `macos-14`; GTK ne s’exécute donc que sur Ubuntu.

### Livrable 3 : Woodpecker — GTK dans les trois compilations

- Statut : CONFIRME
- Fichier(s) : [.woodpecker/ci-linux.yml:16](/C:/Users/FlowUP/Documents/Code/nexus/.woodpecker/ci-linux.yml:16), [.woodpecker/ci-linux.yml:22](/C:/Users/FlowUP/Documents/Code/nexus/.woodpecker/ci-linux.yml:22), [.woodpecker/ci-linux.yml:32](/C:/Users/FlowUP/Documents/Code/nexus/.woodpecker/ci-linux.yml:32), [.woodpecker/ci-linux.yml:40](/C:/Users/FlowUP/Documents/Code/nexus/.woodpecker/ci-linux.yml:40)
- Evidence :

```yaml
16: - name: rust-fmt
20:   - cargo fmt --all --check
22: - name: rust-clippy
28:   - apt-get update && apt-get install -y --no-install-recommends libgtk-3-dev
30:   - cargo clippy --workspace --all-targets --locked -- -D warnings
```

```yaml
37: - apt-get update && apt-get install -y --no-install-recommends libgtk-3-dev
38: - cargo test --workspace --locked
45: - apt-get update && apt-get install -y --no-install-recommends libgtk-3-dev
46: - cargo test --workspace --locked --doc
```

Les commandes sont dans les mêmes steps/conteneurs que les compilations, sans `sudo`. `rust-fmt` ne reçoit pas GTK.

### Livrable 4 : override nextest `multi_daemon`

- Statut : CONFIRME
- Fichier(s) : [.config/nextest.toml:85](/C:/Users/FlowUP/Documents/Code/nexus/.config/nextest.toml:85)
- Evidence :

```toml
85: [[profile.ci.overrides]]
86: filter = 'binary(multi_daemon)'
87: slow-timeout = { period = "60s", terminate-after = 3 }
89: # Emit a JUnit XML report...
92: [profile.ci.junit]
```

Le hard-kill effectif est `60 × 3 = 180 s`, donc supérieur aux 120 s exigées.

### Livrable 5 : `save-if` du cache nightly

- Statut : PARTIEL
- Fichier(s) : [.github/workflows/integration-nightly.yml:43](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/integration-nightly.yml:43)
- Evidence :

```yaml
43: - name: Cache cargo registry + target
44:   uses: Swatinem/rust-cache@v2
47:   shared-key: "integration-ubuntu-latest"
52:   save-if: ${{ github.ref == 'refs/heads/master' || github.ref == 'refs/heads/main' }}
53:   cache-on-failure: "true"
```

- Si GAP/PARTIEL : le cache est bien protégé contre les branches ordinaires, PR, tags et dispatchs de branches feature. Mais la condition n’est pas littéralement « master-only » : elle autorise également `refs/heads/main`. Le dépôt actuel n’a pas de branche `main` et `origin/HEAD` pointe sur `origin/master`, donc l’effet présent est bien master-only. Pour satisfaire le contrat strict, il faudrait retirer l’alternative `main`, ou renommer l’exigence en « mainline-only ».

### Propriété A : complétude GTK des surfaces CI Rust/Linux

- Statut : PARTIEL
- Fichier(s) : [Cargo.toml:3](/C:/Users/FlowUP/Documents/Code/nexus/Cargo.toml:3), [crates/nexus-launcher/Cargo.toml:58](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-launcher/Cargo.toml:58), [.github/workflows/release.yml:29](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/release.yml:29), [scripts/release-attest.sh:74](/C:/Users/FlowUP/Documents/Code/nexus/scripts/release-attest.sh:74)
- Evidence :

```toml
Cargo.toml:3:  members = [
Cargo.toml:9:      "crates/nexus-launcher",
launcher:58: [target.'cfg(target_os = "linux")'.dependencies]
launcher:59: tray-icon = { version = "0.24", default-features = false, features = ["gtk"] }
launcher:60: muda = { version = "0.19", default-features = false, features = ["gtk"] }
```

Inventaire exhaustif des 12 workflows GHA et de Woodpecker :

| Surface | Compilation Rust/Linux | Verdict GTK |
|---|---|---|
| [`build-pkarr-image.yml:76`](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/build-pkarr-image.yml:76) | Dockerfile : `cargo install pkarr-relay`, crate externe | Pas de workspace Nexus, GTK non requis |
| [`build-worker.yml:139`](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/build-worker.yml:139) | `cargo`/`cross build -p nexus-worker` | Graphe étroit, GTK absent |
| [`canary-monthly.yml:83`](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/canary-monthly.yml:83) | `cargo build -p nexus-shell-daemon` | Graphe étroit, GTK absent |
| [`ci.yml:57`](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/ci.yml:57) | `test` : clippy/test workspace ; daemon étroit | Workspace couvert ; daemon bénéficie du step existant |
| [`ci.yml:149`](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/ci.yml:149) | `factory-operator` : `-p sbfb-factory` | Graphe étroit, GTK non requis |
| [`deploy.yml:53`](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/deploy.yml:53) | Télécharge et déploie des binaires | Aucune compilation |
| [`integration-nightly.yml:61`](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/integration-nightly.yml:61) | daemon + `-p nexus-test-harness -p nexus-coordinator-rs` | Graphes étroits, GTK absent |
| [`mirror-codeberg.yml:41`](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/mirror-codeberg.yml:41) | Miroir Git | Aucune compilation |
| [`phase-review-cross-check.yml:37`](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/phase-review-cross-check.yml:37) | Contrôle Git/Markdown | Aucune compilation |
| [`release.yml:29`](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/release.yml:29) | Matrice Ubuntu incluant directement `-p nexus-launcher` | **GTK absent** |
| [`rust-ci.yml:78`](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/rust-ci.yml:78) | clippy/test/doctest workspace | Toutes les jambes Linux sont couvertes |
| [`shellcheck.yml:14`](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/shellcheck.yml:14) | Shellcheck uniquement | Aucune compilation |
| [`supply-chain.yml:48`](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/supply-chain.yml:48) | `cargo-deny` analyse le workspace | Pas de compilation/link GTK |
| [`.woodpecker/ci-linux.yml:16`](/C:/Users/FlowUP/Documents/Code/nexus/.woodpecker/ci-linux.yml:16) | fmt + trois commandes workspace | Les trois compilations sont couvertes |

Les `cargo tree --target x86_64-unknown-linux-gnu` confirment que `nexus-worker`, `nexus-shell-daemon`, `sbfb-factory`, `nexus-test-harness` et `nexus-coordinator-rs` n’incluent ni `nexus-launcher`, ni `tray-icon`, ni `muda`, ni `gtk`. `nexus-launcher` est un crate feuille.

Le seul script CI câblé qui compile directement du Rust est [`scripts/release-attest.sh:76`](/C:/Users/FlowUP/Documents/Code/nexus/scripts/release-attest.sh:76) :

```yaml
release.yml:29: os: [ubuntu-latest, macos-latest, windows-latest]
release.yml:30: binary: [nexus-worker, nexus-shell-daemon, nexus-launcher]
release.yml:41: runs-on: ${{ matrix.os }}
release.yml:64: run: bash scripts/release-attest.sh ${{ matrix.binary }}
release-attest.sh:76: cargo build --release --locked -p "$BINARY"
```

- Si GAP/PARTIEL : **aucune surface Linux `--workspace` n’est oubliée**. En revanche, `release.yml` construit directement `nexus-launcher` sur Ubuntu sans GTK. C’est un vrai carry fonctionnel, mais pas un gap de la Phase C telle que bornée aux push-gates : le workflow est tag/dispatch, préexistant et absent du diff. Il doit être fermé avant le prochain tag ou lancement manuel de release.

### Propriété B : application effective de l’override nextest

- Statut : CONFIRME
- Fichier(s) : [.config/nextest.toml:47](/C:/Users/FlowUP/Documents/Code/nexus/.config/nextest.toml:47), [.config/nextest.toml:85](/C:/Users/FlowUP/Documents/Code/nexus/.config/nextest.toml:85), [.github/workflows/integration-nightly.yml:68](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/integration-nightly.yml:68)
- Evidence :

```toml
47: [profile.ci]
63: retries = 1
69: slow-timeout = { period = "30s", terminate-after = 3 }
74: failure-output = "immediate-final"
75: status-level = "retry"
```

```toml
85: [[profile.ci.overrides]]
86: filter = 'binary(multi_daemon)'
87: slow-timeout = { period = "60s", terminate-after = 3 }
92: [profile.ci.junit]
94: report-name = "nexus-core-rs"
```

Le parseur TOML conserve séparément :

- les cinq scalaires du profil `ci` ;
- l’array `profile.ci.overrides` ;
- la table `profile.ci.junit`.

Une comparaison TOML avec `HEAD` confirme que tous les scalaires et `junit` sont inchangés. `cargo nextest list --profile ci ... -E 'binary(multi_daemon)'` a sélectionné exactement **10 tests**, tous dans `nexus-coordinator-rs::multi_daemon` ou `nexus-test-harness::multi_daemon`. Le nightly utilise bien `--profile ci` aux lignes 68-70.

### Propriété C : impact de `permissions: contents: read`

- Statut : CONFIRME
- Fichier(s) : [.github/workflows/ci.yml:22](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/ci.yml:22), [.github/workflows/ci.yml:32](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/ci.yml:32)
- Evidence :

```yaml
22: permissions:
23:   contents: read
32: - uses: actions/checkout@v4
41:   uses: actions/cache@v4
111:  uses: actions/upload-artifact@v4
```

L’inventaire exhaustif des deux jobs ne trouve aucun `git push`, création de release, commentaire PR/issue ou appel API nécessitant une permission d’écriture. Les seules actions sont checkout, toolchains, cache, setup-node et upload d’artefacts. Aucune étape n’est cassée par `contents: read`.

### Propriété D : absence de régression hors CI/config

- Statut : CONFIRME
- Fichier(s) : [.config/nextest.toml:85](/C:/Users/FlowUP/Documents/Code/nexus/.config/nextest.toml:85), [.github/workflows/ci.yml:22](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/ci.yml:22), [.github/workflows/integration-nightly.yml:52](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/integration-nightly.yml:52), [.github/workflows/rust-ci.yml:103](/C:/Users/FlowUP/Documents/Code/nexus/.github/workflows/rust-ci.yml:103), [.woodpecker/ci-linux.yml:28](/C:/Users/FlowUP/Documents/Code/nexus/.woodpecker/ci-linux.yml:28)
- Evidence :

```text
M  .config/nextest.toml
M  .github/workflows/ci.yml
M  .github/workflows/integration-nightly.yml
M  .github/workflows/rust-ci.yml
M  .woodpecker/ci-linux.yml
```

`git diff` donne exactement `5 files changed, 55 insertions(+), 0 deletions`. Aucun `.rs`, `web/src`, `tools/factory-operator/src`, wire format, `*_VERSION`, `Cargo.toml` ou `Cargo.lock` n’est touché. `git diff --check` est propre.

Les deux fichiers non suivis sont uniquement :

- `.planning/active/sprint82_phase_c_preflight.md`
- `.planning/active/sprint82_phase_c_review.md`

## Résumé final

- Total items : 9
- Confirmés : 7 / Gaps : 0 / Partiels : 2
- Verdict global : **GAPS-P2/P3 (documentables)**

La restauration des push-gates GTK et l’override nextest sont correctement implémentés. Les deux réserves sont le whitelist `master || main` plus large que le libellé strict « master-only », et le carry GTK préexistant de `release.yml`, à fermer avant le prochain release.