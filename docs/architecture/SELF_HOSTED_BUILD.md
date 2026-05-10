# Self-Hosted Build — le reseau compile le reseau

**Sprint 52 Phase B design doc.**
**LT-7 pre-v1.0 obligatoire** (cf. `docs/release/ROADMAP_COMMITMENTS.md`).

## 1. Pourquoi

Un reseau de compute qui depend de GitHub Actions pour se compiler
n'est pas un reseau de compute — c'est un wrapper. SBFB doit
pouvoir se builder via ses propres workers avant le tag v1.0.
Validation pre-tag = quorum controle sur 3 machines reelles
(Win+VPS+Mac). Diversite publique = post-launch progressif.

## 1.1 Strategie 3 etages

| Etage | Quoi | Sprint | Dependance GHA |
|---|---|---|---|
| **1. CI Woodpecker** | `.woodpecker/ci-linux.yml` + agent self-hosted VPS. CI Linux hors GHA. | S52 (config) + S54 (images pin + VPS prep) + **S55 (server deploy)** | GHA reste pour release multi-OS |
| **2. Build worker SBFB** | `task_type: "build"` protocol + sandbox hermetique + quorum SHA256. Le VPS bootstrap devient le premier build worker. | S54-S55 | GHA fallback seulement |
| **3. Reseau autonome** | N builders independants, attestation signee, distribution binaires via iroh-blobs. Validation controlee S60 pre-tag (Win+VPS+Mac, redundancy=3). Diversite publique post-launch. | S60 (validation) + post-launch (diversite) | GHA optionnel (second opinion) |

GHA descend progressivement : CI principale (aujourd'hui) →
release seulement (etage 1) → fallback (etage 2) → optionnel
(etage 3).

## 1.2 Ce que task_type "build" n'est PAS

Le worker actuel est un **LLM executor** : il lit des TaskEntry,
appelle `LlmBackend::generate`, signe un resultat. Ce n'est PAS
un runner CI arbitraire.

Ajouter `task_type: "build"` n'est **pas une extension triviale**
du worker LLM. C'est un **runtime separe** qui necessite :
- Sandbox hermetique (Nix, container, VM — pas un tmpdir)
- Cache reproductible (crates.io, npm registry — lockfiles)
- Logs streames vers le coordinateur (pas juste un resultat final)
- Zero secret dans l'environnement de build
- Artefacts signes + attestation SLSA
- Quorum multi-builder (N builders independants comparent SHA256)

La complexite est comparable a un Woodpecker/Forgejo runner
complet, pas a un "if task_type == build { cargo build }".

## 2. Modele

```
Coordinateur                      Workers (redundancy_factor = 3)
     |                                  |
     |  TaskEntry {                     |
     |    task_type: "build",           |
     |    metadata: {                   |
     |      repo: "SBFB50/SBFB",       |
     |      commit: "<sha>",           |
     |      binary: "nexus-worker",    |
     |      toolchain: "<hash>",       |
     |    }                             |
     |  }                               |
     | ---- iroh-docs dispatch -------> |
     |                                  | cargo build --release --locked
     |                                  | SOURCE_DATE_EPOCH=<commit_ts>
     |                                  | --remap-path-prefix
     |                                  |
     | <--- ResultEntry {               |
     |        output_sha256: "<hex>",   |
     |        artifact_blob: <hash>,    |
     |      } -------------------------+
     |
     |  Quorum : 2/3 SHA256 identiques → binaire accepte
     |  Divergence : reject + alerte (worker compromis ou
     |               toolchain mismatch)
```

## 3. Wire format — extension TaskEntry existant

Le `task_type: String` accepte deja n'importe quelle valeur.
Un build task utilise `task_type: "build"` et encode les parametres
dans `metadata: BTreeMap<String, String>` :

| Cle metadata | Valeur | Requis |
|---|---|---|
| `build.repo` | URL clone (HTTPS) | oui |
| `build.commit` | SHA complet 40 hex | oui |
| `build.binary` | nom du crate cible | oui |
| `build.toolchain_sha256` | hash du toolchain bundle | oui |
| `build.target` | triple Rust (ex: `x86_64-unknown-linux-gnu`) | oui |
| `build.source_date_epoch` | timestamp commit (determinisme) | oui |
| `build.cargo_flags` | flags supplementaires | non |

Les champs LLM (`prompt`, `model`, `system_prompt`) restent vides
("") pour un build task. Pas de nouveau champ struct — tout passe
par `metadata`. Pre-launch policy respectee : pas de bump
`TASK_FORMAT_VERSION`.

Le `ResultEntry` existant encode le resultat dans ses champs :
- `result` : SHA256 du binaire produit
- `metadata` : `{"artifact_blob": "<iroh-blobs hash>"}` pour
  recuperer le binaire via le reseau

## 4. Worker build executor (etage 2 — runtime separe)

**Ce module n'est PAS le worker LLM existant.** C'est un runtime
distinct, potentiellement un binaire separe ou un mode du worker,
avec des contraintes d'isolation plus strictes que l'inference.

### 4.1 Sandbox hermetique

Un tmpdir ne suffit PAS — un build malveillant peut lire le
filesystem host, exfiltrer des secrets, ou modifier d'autres
fichiers. Le sandbox doit etre hermetique :

**Options (a evaluer au sprint dedie)** :
- **Container OCI** (podman rootless) : isolation filesystem +
  network, image reproductible, overhead faible
- **Nix build sandbox** : hermetique par construction, cache
  content-addressable, mais courbe d'apprentissage
- **VM legere** (Firecracker/Cloud Hypervisor) : isolation
  maximale, overhead plus lourd

**MVP pragmatique** : podman rootless avec `--network=none` post-
clone (clone avec reseau, build sans). Le sprint dedie tranchera
apres benchmark des 3 options.

Invariants :
- `git clone --depth 1` dans le sandbox
- `cargo build --release --locked -p <binary>`
- Zero acces reseau pendant la compilation
- Zero secret dans l'environnement (pas de GITHUB_TOKEN, pas de
  COSIGN_KEY, pas de SSH keys)
- Timeout configurable (default 30min)

### 4.2 Toolchain pinning

Le task descriptor inclut `build.toolchain_sha256` — le hash du
bundle toolchain attendu. Le worker :
1. Verifie que son toolchain local matche le hash
2. Si mismatch : refuse le task (status `rejected_toolchain`)
3. Le coordinateur re-dispatch a un worker compatible

Bundle toolchain = archive tar contenant :
- `rustc` + `cargo` (version exacte)
- `rustup` target installee
- Linker attendu (gcc/lld version)

Le hash est calcule par le coordinateur au moment du dispatch :
`sha256(rustc --version --verbose | sort)` ou equivalent.

**MVP simplification** : au lieu de bundler, le task specifie
`build.rustc_version: "1.94.0"` et le worker verifie
`rustc --version`. Le hash complet vient apres le MVP.

### 4.3 Determinisme

Flags obligatoires pour la reproductibilite :
- `SOURCE_DATE_EPOCH=<commit_timestamp>` (deja dans release-attest.sh)
- `--remap-path-prefix=$PWD=/build` (elimine les chemins locaux)
- `CARGO_INCREMENTAL=0` (pas de cache incremental)
- `codegen-units=1` + `lto=fat` (deja dans release profile)

### 4.4 Output

Le worker :
1. Calcule SHA256 du binaire produit
2. Upload le binaire dans iroh-blobs (hash content-addressable)
3. Soumet un `ResultEntry` avec `result: <sha256>` et
   `metadata.artifact_blob: <blobs_hash>`

## 5. Quorum verification

Le coordinateur recoit N `ResultEntry` (N = `redundancy_factor`).

- **Majorite SHA256 identique** (ex: 2/3 match) → binaire accepte.
  Le blob hash de la majorite est le binaire officiel.
- **0 match** → tous les workers ont diverge. Alerte + fallback
  GHA bootstrap. Investigation : toolchain mismatch probable.
- **1 outlier** → 1 worker a diverge. Log + reputation impact
  (kudos penalty ou quarantine selon politique).

Le mecanisme de quorum existe deja dans `redundancy_factor` +
`ResultValidator`. L'extension pour les builds est : la validation
compare `result` (SHA256) au lieu de faire une verification
semantique LLM.

## 6. Bootstrap sequence

```
Phase 0 — Stage 0 (GHA)
  Le premier binaire est compile par GitHub Actions.
  Il est signe via release-attest.sh + SLSA provenance.
  C'est le "genesis build".

Phase 1 — Premier noeud
  Le genesis binaire demarre un noeud. Il n'a pas de workers.
  GHA reste le seul CI/CD.

Phase 2 — Premier worker
  Un deuxieme noeud rejoint le reseau comme worker.
  Le coordinateur peut maintenant dispatcher des build tasks.
  Mais redundancy_factor = 1 (un seul worker) → pas de quorum.

Phase 3 — Quorum operationnel
  >= 3 workers avec le meme toolchain.
  Le coordinateur dispatch build tasks avec redundancy_factor = 3.
  Le reseau se compile lui-meme.
  GHA devient fallback uniquement.

Phase 4 — Validation controlee S60 pre-tag
  Win dev + VPS Helsinki + Mac : 3 builders, 1 build task reel,
  redundancy_factor=3, consensus SHA256 2/3 → AwaitingQuorum → Completed.
  Prouve le chemin E2E sur machines reelles.
  GHA sert de "second opinion" (cross-validation externe).
  Le tag v1.0 peut etre pose apres ce quorum valide.
  Diversite publique (N builders non-controles) = post-launch progressif.
```

## 7. Trust model

### 7.1 Menaces

- **Worker malveillant** insere du code dans le binaire compile
  → le quorum SHA256 detecte la divergence (1 outlier sur 3)
- **Collusion N/2+** workers produisent le meme binaire malveillant
  → mitigation : workers selectionnes aleatoirement, pas auto-
  inscrits (le coordinateur choisit qui build)
- **Toolchain supply chain** : le compilateur Rust lui-meme est
  compromis → hors scope SBFB (meme probleme pour GHA). Mitigation
  future : reproducible builds du compilateur via bootstrap chain.

### 7.2 Avantage vs GHA

GHA = un seul runner construit le binaire. Si le runner est
compromis (ou GitHub lui-meme), le binaire est compromis sans
detection possible. Avec le quorum SBFB : 3 machines independantes
doivent produire le meme binaire. La probabilite de compromettre
3 machines aleatoires simultanement est significativement plus
faible qu'une seule machine centralisee.

## 8. Scope MVP (sprint dedie)

Le sprint dedie (~S54-S55) livre :
1. Worker build executor (sandbox tmpdir + cargo build)
2. `task_type: "build"` handling dans dispatch_loop
3. Quorum SHA256 dans ResultValidator
4. CLI `sbfb build <repo> <commit> <binary>` pour trigger
5. Integration test : 3 workers locaux buildent le meme commit,
   quorum valide

Hors scope MVP :
- Cross-platform builds (MVP = x86_64-linux seulement)
- Toolchain bundle distribution via iroh-blobs
- Auto-update (le reseau build + deploy automatiquement)
- Test suite execution distribuee (futur LT-8)

## 9. VPS status — sbfb-eu (135.181.42.188)

**Sprint 55 Phase A** : Woodpecker CI operationnel.

| Composant | Status | Details |
|---|---|---|
| Docker | 29.4.3 | Installe S54 Phase D |
| CI images | 3 pullees SHA256 | rust:1.94, node:20, bash:5 |
| Deploy key GitHub | Active | Acces repo read-only |
| woodpecker-cli | 3.14.0 | Installe S54 Phase D |
| Woodpecker server | Docker Compose | `configs/woodpecker/docker-compose.yml` |
| Woodpecker agent | Docker Compose | Side-by-side avec server |
| Caddy reverse proxy | Auto-TLS Let's Encrypt | `configs/woodpecker/Caddyfile` |
| systemd service | `woodpecker.service` | `configs/systemd/woodpecker.service` |
| Pipeline | `.woodpecker/ci-linux.yml` | 12 steps, images SHA256 |

### 9.1 Deploy checklist

```bash
# 1. Copier les configs sur le VPS
scp -r configs/woodpecker/ user@135.181.42.188:/opt/woodpecker/

# 2. SSH sur le VPS
ssh user@135.181.42.188

# 3. Creer .env depuis le template
cd /opt/woodpecker
cp .env.example .env
# Editer .env avec les vrais secrets

# 4. Installer Caddy (si pas deja fait)
sudo apt install -y caddy
sudo cp Caddyfile /etc/caddy/Caddyfile
sudo systemctl reload caddy

# 5. Demarrer Woodpecker
docker compose up -d

# 6. Installer le service systemd
sudo cp /path/to/woodpecker.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable woodpecker

# 7. Verifier
curl -s https://<domain>/healthz  # → 200
docker compose logs -f             # → healthy
```

### 9.2 GitHub OAuth App setup

1. Aller sur https://github.com/settings/applications/new
2. Application name: `SBFB Woodpecker CI`
3. Homepage URL: `https://<domain>`
4. Authorization callback URL: `https://<domain>/authorize`
5. Copier Client ID et Client Secret dans `.env`

### 9.3 Webhook setup

Woodpecker cree automatiquement les webhooks GitHub lors de
l'activation d'un repo dans l'UI. Prerequis : le GitHub OAuth
App doit avoir les permissions `repo` + `admin:repo_hook`.

## 10. Blockers connus

| Blocker | Severite | Mitigation |
|---|---|---|
| Rust reproducible builds (#129080) | Medium | SOURCE_DATE_EPOCH + remap-path-prefix couvrent 95% des cas. MVP accepte 5% false negative |
| Toolchain heterogeneite | Medium | MVP pin version string, hash complet post-MVP |
| Binaires 50MB+ × 3 workers | Low | iroh-blobs content-addressable, deduplique naturellement |
| Build time 5-15min | Low | Timeout 30min, async task model deja en place |
