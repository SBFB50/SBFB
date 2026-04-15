# Reproducible builds + SLSA provenance

Livre en Sprint 18 Phase B. Chaque binary publie dans une Release
GitHub (nexus-launcher, nexus-worker, nexus-shell-daemon) est
accompagne de deux fichiers annexes :

- `<artifact>.sha256` : somme SHA256 du binaire (format compatible
  `sha256sum -c`).
- `<artifact>.intoto.jsonl` : **attestation SLSA Provenance v1.0**
  au format in-toto Statement v1. Signee Ed25519 via cosign keyless
  OIDC quand le workflow GitHub Actions tourne (`.sig` annexe).

Le pipeline de build garantit que deux invocations du meme commit,
avec le meme `SOURCE_DATE_EPOCH`, produisent des binaires
**byte-identical**. N'importe qui peut donc rebattre la chaine et
comparer le SHA256 avec celui publie pour attester la provenance.

Source de verite : `scripts/release-attest.sh` + `.cargo/config.toml`
+ `[profile.release]` dans `Cargo.toml` racine.

---

## 1. Verifier un artefact telecharge (SHA256)

Telecharger les trois fichiers d'une release (binaire + .sha256
+ .intoto.jsonl), puis :

```bash
# Linux / macOS
sha256sum -c nexus-launcher-linux-x86_64.sha256

# Windows (PowerShell)
Get-FileHash nexus-launcher-windows-x86_64.exe -Algorithm SHA256
# comparer avec la valeur dans le .sha256
```

Sortie attendue : `nexus-launcher-linux-x86_64: OK`.

Si le check echoue, **ne pas executer le binaire**. Ouvrir une
issue sur le repo en joignant les trois fichiers et le OS + arch.

## 2. Verifier l'attestation SLSA

L'attestation est un fichier JSON sur une ligne qui respecte le
schema [SLSA Provenance v1.0](https://slsa.dev/spec/v1.0/provenance).
Le champ `subject[0].digest.sha256` doit egaler celui du binaire.

```bash
# Sanity check : le hash dans l'attestation colle avec le binaire
jq -r '.subject[0].digest.sha256' nexus-launcher-linux-x86_64.intoto.jsonl
sha256sum nexus-launcher-linux-x86_64 | awk '{print $1}'
# les deux valeurs doivent etre identiques
```

Pour verifier la **signature** (si `.sig` present) :

```bash
cosign verify-blob \
  --certificate-identity-regexp "https://github.com/SBFB50/SBFB/.*" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  --signature nexus-launcher-linux-x86_64.intoto.jsonl.sig \
  nexus-launcher-linux-x86_64.intoto.jsonl
```

Un outil plus haut-niveau, [`slsa-verifier`](https://github.com/slsa-framework/slsa-verifier),
consomme directement l'in-toto et valide la provenance builder :

```bash
slsa-verifier verify-artifact \
  --provenance-path nexus-launcher-linux-x86_64.intoto.jsonl \
  --source-uri github.com/SBFB50/SBFB \
  nexus-launcher-linux-x86_64
```

## 3. Rebuild local deterministe

Pour reproduire le binaire `nexus-launcher` tagge `v1.2.3` :

```bash
git clone https://github.com/SBFB50/SBFB
cd SBFB
git checkout v1.2.3

# SOURCE_DATE_EPOCH est le timestamp du commit tagge.
# release-attest.sh le calcule automatiquement via git log -1 --format=%ct.
bash scripts/release-attest.sh nexus-launcher

sha256sum dist/nexus-launcher-linux-x86_64
# comparer avec le .sha256 publie dans la release
```

Contraintes :

- **Meme toolchain Rust** : utiliser la version stable pinne par
  `rust-toolchain.toml` si present, sinon la plus recente au
  moment du tag (GitHub Actions `dtolnay/rust-toolchain@stable`).
  Les versions majeures du compilateur peuvent casser le
  byte-for-byte (codegen upgrades, libstd layout).
- **Meme OS + arch** : un binaire Linux x86_64 ne reproduira pas
  un binaire Windows x86_64. Chaque tuple `(os, arch)` a son
  propre artefact + SHA256 + attestation.
- **`--locked`** : `Cargo.lock` doit etre celui du commit, non
  regenere. `release-attest.sh` passe explicitement `--locked` a
  cargo pour empecher toute resolution implicite de deps.
- **`CARGO_INCREMENTAL=0`** : deja impose via
  `.cargo/config.toml` (`[build] incremental = false`). Si vous
  override cette valeur via env, le build ne sera plus
  reproducible.

## 4. Profile release deterministe

Les valeurs dans `Cargo.toml` racine (`[profile.release]`) :

| Cle | Valeur | Raison |
|---|---|---|
| `codegen-units` | `1` | Single-thread codegen LLVM elimine le non-determinisme de l'ordonnancement parallel. |
| `lto` | `"fat"` | LTO global du workspace ; stable entre deux runs meme toolchain. |
| `strip` | `"symbols"` | Symboles + debuginfo varient avec la toolchain, retires. |
| `debug` | `false` | Explicite, meme si c'est le defaut release. |
| `opt-level` | `3` | Optim max ; deterministe si `codegen-units = 1`. |
| `panic` | `"abort"` | Choix legacy sizing, sans impact reproductibilite. |

Plus `CARGO_INCREMENTAL=0` (via `.cargo/config.toml`) et
`SOURCE_DATE_EPOCH=<commit_ct>` (via script) pour neutraliser les
timestamps embarques dans les metadata d'archive.

**Specifique Windows MSVC** : `.cargo/config.toml` applique
`rustflags = ["-C", "link-arg=/Brepro"]` pour les cibles
`x86_64-pc-windows-msvc` et `aarch64-pc-windows-msvc`. Sans ce
flag, le PE produit contient un `IMAGE_FILE_HEADER.TimeDateStamp`
et un `Debug Directory GUID` qui varient a chaque invocation du
linker ; meme SOURCE_DATE_EPOCH + meme toolchain produisent alors
~19 bytes divergents. `/Brepro` substitue des valeurs deterministes
aux deux champs.

## 5. Limitations connues

- **Cross-platform** : reproductibilite garantie **dans** un tuple
  `(os, arch)`, pas entre. Les attestations sont par plateforme.
- **Cranelift / LLVM codegen units** : en theorie `codegen-units = 1`
  supprime les reordonnancements ; en pratique un bug rare peut
  subsister. Si un user rapporte un mismatch SHA256 avec meme
  toolchain + meme commit, ouvrir une issue avec les deux binaires.
- **Wheel `nexus-core-py`** : maturin honore `SOURCE_DATE_EPOCH`
  pour le zip mais la reproductibilite interpreter-wise depend de
  la version exacte de Python + maturin. Best-effort S18, durci S19+.
- **Signature cosign keyless** : requiert que l'utilisateur fasse
  confiance au GitHub Actions OIDC issuer
  (`token.actions.githubusercontent.com`). Pas de cle persistante
  a la release — le transparency log Rekor sert de preuve.

## 6. Historique

| Sprint | Commit | Changement |
|---|---|---|
| 18 Phase B | cf. commit `feat(sprint18): Phase B` | Profile release deterministe, script release-attest.sh, workflow release.yml avec cosign + in-toto, doc user-facing. |
