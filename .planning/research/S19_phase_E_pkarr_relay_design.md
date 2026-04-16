# Sprint 19 Phase E — pkarr relay self-hosted docker : design doc

**Ecrit** : 2026-04-16 (session fraiche post-S18 audit gate leve,
pre-Phase E implementation).
**Tip master d'entree** : `1a606a3` (chore(sprint18): audit-P3
batch).
**Scope** : decision design pour la Phase E S19 — image docker
pkarr-relay self-hostable + workflow CI build + ops doc deploy.
**Pas de code livre** : doc preparatoire avant `feat(sprint19):
Phase E`.

---

## 1. Probleme adresse

### 1.1 Le single-point-of-trust pkarr

iroh 0.97 utilise `presets::N0` qui code en dur **3 relais
pkarr operes par n0/iroh** :

- `https://relay.iroh.network` (Amerique du Nord)
- `https://eu-relay.iroh.network` (Europe)
- `https://ap-relay.iroh.network` (Asie-Pacifique)

Toute resolution `NodeId → SignedPacket` (DHT-like discovery
sur HTTP) passe par ces trois relais. Sprint 18 Phase C a deja
livre la **primitive** multi-relai (`RelayMode::Custom` +
`relay_config.rs`) et S19 Phase A cable la **primitive**
quorum 2/3 au runtime. Mais **les 3 relais par defaut restent
operes par la meme entite** : si n0 :

- subit une **subpoena** (USA — secret order, gag rule
  possible),
- est **DDoS** sustained (cas hypothetique mais documente
  dans threat model T2 §5.2),
- decide de **deplatform** SBFB pour raison legale ou ToS,
- **stoppe le service** (pivot strategique, faillite, etc.),

alors **toute la couche discovery SBFB tombe en simultane**.
Le quorum 2/3 livre S18-S19 ne protege pas contre une
**defaillance correlee** des 3 relais sous controle commun.

### 1.2 La promesse federation ONG

`HARDENING_ROADMAP §3 S19` + `VALIDATED_BLUEPRINT couche 4
"overlay DHT"` decrivent la trajectoire long-terme :

> Federation ONG = chaque ONG partenaire (Amnesty
> International, Human Rights Watch, FIDH, RSF, etc.) opere
> **un relai pkarr self-hosted** dans sa juridiction.
> Le client SBFB pondere le quorum entre N relais issus de
> juridictions differentes (USA + UE + Suisse + Norvege +
> ...). Sybil-resistance par diversite institutionnelle.

Pour que cette federation existe un jour, il faut :

1. **Une image docker** prete a deployer (Phase E S19 livre
   ca),
2. **Une doc deploy** assez courte pour qu'un sysadmin ONG
   non-Rust puisse spinup en ~30 minutes,
3. **Au moins un relai SBFB-self-hosted** comme proof-of-
   concept (deploy reel reporte S20+ : decision ops separee,
   cf. §4.5).

### 1.3 Ce que Phase E **ne resout pas**

Phase E livre **l'image et la doc**. Elle ne :

- ne deploie **pas** un relai reel SBFB (decision ops),
- ne **federe pas** de relais ONG (necessite partnership
  outreach, S22+ scope cut),
- ne contribue pas upstream pkarr (potentiel S20+ si patches
  decouverts au build),
- ne migre pas de Mainline DHT vers une autre couche (cf.
  §3.6 alternative rejetee).

---

## 2. Decision retenue

**Image Docker `ghcr.io/sbfb50/pkarr-relay:<version>`** buildee
**from upstream `pubky/pkarr` source** (pas un fork, pas un
re-implement), distribuee via **GitHub Container Registry
(ghcr.io)** sous l'organisation GitHub `SBFB50` (parite avec
le mirror Codeberg `SBFB/SBFB` pattern S18 Phase E3).

Build via **Docker BuildKit** (multi-stage, non-Kaniko archived
2025), workflow GHA `build-pkarr-image.yml` avec :

- **Trivy scan** inline (severity HIGH+CRITICAL fail-build),
- **Cosign signature keyless** (GitHub OIDC + Fulcio +
  Rekor),
- **SLSA L2 in-toto provenance attestation** generee par le
  workflow (parite S18 Phase B `provenance.json` worker/
  daemon/launcher/wheel),
- **SBOM SPDX via syft** attache en attestation,
- **Pin SHA** action `aquasecurity/trivy-action` (lecon
  attaque mars 2026, cf. §5.2.7).

Ops doc `docs/release/PKARR_RELAY_OPS.md` couvre **Hetzner CX22**
(2 vCPU / 4 GB / 40 GB SSD, ~7.99 EUR/mois HT post-ajustement
Hetzner avril 2026 — cf. §3.5 cost trade-off) avec :

- provisioning ssh + ufw firewall,
- **systemd unit hardene** (PrivateTmp, ProtectSystem=strict,
  NoNewPrivileges, ReadOnlyPaths, SystemCallFilter=@system-
  service),
- **Caddy** reverse proxy + auto-HTTPS (vs nginx+certbot :
  zero-config TLS = vrai gain pour ONG sysadmin non-expert,
  cf. §4.5.3),
- smoke test `pkarr-cli publish/resolve` end-to-end,
- monitoring baseline `journalctl` + `df -h` (pas Prometheus,
  reporte S22+),
- procedure **rotation SPKI cert** cross-ref Phase C TLS
  pinning (le SPKI hash du nouveau relai self-hosted doit
  etre ajoute au `~/.sbfb/relay-pins.json` bootstrap S19
  Phase C),
- federation onboarding : annonce relay public + procedure
  pour qu'un autre maintainer SBFB ajoute le relai a
  `relays.json` defaults futurs.

**Pas de deploy reel ce sprint.** L'image existe, la doc
existe, le smoke CI green. Le premier deploy reel est
decision ops separee post-S19 (pricing, ops energy, qui paie
le 8 EUR/mois — cf. §6 limites).

---

## 3. Alternatives strategiques considerees

### 3.1 Image Docker forkee depuis upstream pkarr (RETENU)

**Approche** : `FROM rust:1.94-slim-bookworm AS builder`,
`cargo install pkarr-relay --version 2.x.*` depuis crates.io
(crate publie par `pubky/pkarr` upstream), runtime image
`debian:bookworm-slim` avec `ca-certificates` + le binaire
copie. Pin minor version (`2.x.*`) pour stabilite + auto-pickup
des security patches.

**Avantages** :

- **Zero re-invention** : on suit le upstream, on beneficie
  des patches CVE automatiquement au prochain rebuild.
- **Audit upstream possible** : code Rust publie sur GitHub
  `pubky/pkarr`, lisible, sous MIT.
- **Crate publie sur crates.io** : passe par le filtre
  Rust security advisory db (`rustsec/advisory-db`), pas
  une source obscure.
- **Compatible Pubky network** : par construction, notre
  relai parle exactement le meme protocole que les autres
  relais Pubky existants (homeservers, indexers, autres
  relais ONG futurs).

**Inconvenients** :

- **Couplage version upstream** : si Pubky pivote ou release
  un breaking dans `pkarr-relay` 3.x, on est expose. Pin
  minor (`2.x.*`) limite mais ne supprime pas.
- **Audit upstream requis** : le crate `pkarr-relay` n'a
  **pas** d'audit public 2026 (cf. §5.2.4 R-pkarr-audit). On
  herite du risque code upstream sans piste papier.
- **Single-vendor risk** : si Pubky disparait (faillite
  startup, pivot, etc.), on doit forker proprement nous-
  memes (transition couteuse).
- **Build-time dependency Rust toolchain** : le workflow GHA
  doit installer Rust 1.94, ce qui rallonge le build vs une
  image distroless prebuilt.

**Verdict** : RETENU pour MVP S19. Re-evaluer S22+ si Pubky
montre signes faiblesse ou si SBFB a besoin de patches qui
ne passent pas upstream.

### 3.2 Build from source — minimal binary SBFB-controlled

**Approche** : forker `pubky/pkarr` dans `crates/sbfb-pkarr-
relay/` du workspace SBFB, supprimer code mort (pubky-
homeserver hooks, indexer protocol, etc.) qu'on n'utilise
pas, livrer un binaire minimal `sbfb-pkarr-relay`.

**Avantages** :

- **Supply-chain controlee** : audit complet possible, pas
  de dependance externe pour le code critique.
- **Surface attaque minimale** : on supprime tout ce qu'on
  n'utilise pas (homeserver protocol, indexer endpoints,
  etc.).
- **Pin total** : version SHA-pinnee dans Cargo.lock SBFB,
  zero drift possible.

**Inconvenients** :

- **Cout maintenance prohibitif pour MVP** : suivre les
  patches upstream manuellement = backport CVE par CVE.
  Pour 1 mainteneur (FlowUP), ingerable.
- **Casse compatibilite Pubky** : si Pubky bumpe leur wire
  format DHT, notre fork tombe out-of-sync. Federation ONG
  devient impossible si chaque relai parle un dialect.
- **Re-invente la roue** : pkarr-relay upstream est ~2000
  LOC Rust, le cout au compromis n'est pas rationnel pour
  S19.

**Verdict** : REJETE S19. **A reconsiderer si** (a) Pubky
upstream stagne ou pivote (signal observable : pas de release
sur 6 mois + issues critiques non-reponses), (b) SBFB a un
besoin patch qui ne passe pas upstream apres 2 PR, (c) un
audit revele que >30% du code upstream est dead-code pour
notre usage (forker = simplifier).

### 3.3 Integrer pkarr-relay AS LIBRARY dans nexus-shell-daemon

**Approche** : `cargo add pkarr-relay --features library`
dans `crates/nexus-shell-daemon-core`, expose un endpoint
`POST /pkarr/{node_id}` cote daemon SBFB qui sert aussi de
relai pour d'autres clients SBFB.

**Avantages** :

- **1 seul binary** : pas de container separe, pas de
  Dockerfile, pas de doc deploy.
- **Reutilise l'auth loopback S16** : bearer token X-SBFB-
  Token rotated S18 protege deja le port HTTP.
- **Cout deploy zero** : si tu lances le daemon SBFB, tu es
  un relai pkarr de fait.

**Inconvenients** :

- **Mixe roles client/serveur** : nexus-shell-daemon est un
  **client** P2P (consume le DHT pour decouvrir des peers).
  Pkarr-relay est un **serveur** HTTP (sert le DHT comme
  cache pour d'autres clients). Mixer les deux casse la
  separation des roles, complique le threat model
  (l'attaque sur le relai exposes le client et vice-versa),
  et viole le principe `nexus-shell-daemon = singleton
  strict` (decision Day-0 figee).
- **Casse le pattern federation** : la federation ONG vise
  des **operateurs distincts du daemon** (Amnesty heberge
  un relai sans avoir besoin d'etre un noeud SBFB
  fonctionnel). Si le relai = fork du daemon, l'ONG doit
  installer le daemon complet (Python coordinator, etc.) =
  prohibitif.
- **Pas la facon dont pkarr upstream est concu** :
  upstream Pubky publie `pkarr-relay` comme binaire
  autonome justement parce que les operateurs de relais
  ne sont pas necessairement des operateurs de homeservers.
- **Casse la doctrine port-7000-iframe-isolation S12** :
  ajouter un endpoint server-side au daemon augmente la
  surface attaque CSP cross-origin si pas extreme attention.

**Verdict** : REJETE. La separation roles client/serveur est
non-negociable pour la federation et pour la lisibilite du
threat model. Phase E reste un container separe.

### 3.4 Pas de self-hosted, attendre la federation Pubky naturelle

**Approche** : ne rien faire S19. Attendre que la federation
Pubky se densifie organiquement (homeservers + relais
operes par la communaute Pubky) et utiliser ces relais
publics dans `relays.json` defaults SBFB futur.

**Avantages** :

- **Zero ops cost** : pas de relai a maintenir, pas de doc
  a ecrire, pas de container a publier.
- **Beneficie de l'effet reseau Pubky** : si Pubky reussit
  son adoption 2026-2027, on a deja 50+ relais dispo
  gratuitement.

**Inconvenients** :

- **Timeline imprevisible** : Pubky a **lance** le concept
  pkarr en 2024, mais la federation reste embryonnaire en
  2026 (Pubky network = plusieurs entites mais
  majoritairement controlees par ou alignees avec le team
  Pubky core, pas une vraie diversite institutionnelle
  ONG).
- **Pas de controle sur la diversite juridictionnelle** :
  les relais Pubky existants sont majoritairement aux US/
  Canada. Un quorum 2/3 sur 3 relais US ne resiste pas a
  un subpoena coordonne.
- **Pas alignement avec mission SBFB** : SBFB cible
  explicitement les ONG droits humains. Attendre que
  Pubky fasse le travail outreach = abandonner l'identite
  produit.
- **Casse HARDENING_ROADMAP §3 S19** : item explicitement
  liste pour Sprint 19. Rejeter = decaler tout
  HARDENING_ROADMAP §3 S20+ qui depend de la primitive
  federation.

**Verdict** : REJETE — trop passif. SBFB doit contribuer
**activement** a la federation des j0, pas attendre. Phase E
livre la **brique technique** qui rend la federation possible
(image + doc), meme si le deploy reel viendra plus tard.

### 3.5 k3s / Kubernetes vs Docker Compose vs systemd-nspawn vs raw docker

**Comparaison ops complexity vs control, cible operateur ONG
sysadmin non-expert** :

| Option | Complexity | Control | Adapte ONG ? |
|---|---|---|---|
| **k3s / k8s** | Tres haut (cluster, ingress, secrets manager, etc.) | Tres haut (auto-heal, multi-replica, rolling deploy) | NON — overkill pour 1 relai single-instance |
| **Docker Compose** | Moyen (1 fichier YAML, `docker compose up -d`) | Bas (pas d'auto-heal robuste, restart policy basique) | OUI mais moyen — necessite Docker Engine + Compose plugin |
| **systemd-nspawn** | Moyen-haut (machinectl, systemd-networkd, etc.) | Tres haut (integration native systemd) | NON — peu connu hors devs Linux avances |
| **Raw docker run** + **systemd unit** (RETENU) | Bas (1 container, 1 unit file) | Moyen (restart=on-failure systemd, healthcheck docker) | OUI — pattern le plus connu sysadmin moyen |

**Verdict** : raw `docker run` lance via **unit file
systemd** (`After=docker.service`, `Restart=on-failure`,
hardening directives §4.5.2). Pattern documente dans
`PKARR_RELAY_OPS.md §3`. Compose serait acceptable mais
n'apporte rien pour 1 service single-replica + ajoute une
dependance plugin.

### 3.6 Migration Mainline DHT → IPFS Kademlia / libp2p Kademlia / Solana DNS

**Approche (rejetee)** : changer la couche DHT sous-jacente
pour reduire la dependance upstream Pubky.

| Cible | Pros | Cons |
|---|---|---|
| **IPFS Amino DHT** (libp2p Kademlia, /ipfs/kad/1.0.0) | Reseau public >100k noeuds, doc large, lib Rust mature (`rust-libp2p`) | Pas de signed packets natif (besoin couche app au-dessus), latence resolution >1s typique, reset hostile aux courts records DNS-like |
| **libp2p Kademlia generic** | Tres flexible | On reinvente pkarr from scratch — perd le benefice "10M+ noeuds Mainline DHT existants" |
| **Solana DNS** (SNS) | Resilient, bonne UX user-facing | Lien blockchain crypto (governance, fees, etc.), incompatible philosophie SBFB "no payment, no token" + violation `feedback_kudos_non_monetary` |
| **Mainline DHT via pkarr (status quo)** | 10M+ noeuds existants, BEP44 stable depuis 2014, pas de fees, pas de blockchain | Dependance Pubky upstream (mitigee §3.1) |

**Verdict** : REJETE. Mainline DHT via pkarr reste la
meilleure couche pour le scope SBFB (privacy-first, no-
monetary, P2P-pure). La diversite vient de **multi-relai
quorum** (S18-S19 livre) et **federation ops** (S19 Phase E
livre la brique), pas d'un changement de DHT.

### 3.7 Synthese alternatives

| # | Option | Verdict |
|---|---|---|
| 3.1 | Image fork upstream pkarr | **RETENU** |
| 3.2 | Build from source minimal | Reporte S22+ si signaux faiblesse upstream |
| 3.3 | Library embarquee daemon | Rejete (mixe roles) |
| 3.4 | Attendre federation Pubky | Rejete (passif, hors mission) |
| 3.5 | k8s vs compose vs systemd | systemd unit + raw docker (retenu) |
| 3.6 | Migration DHT (IPFS/libp2p/SNS) | Rejete (pas le bon levier) |

---

## 4. Choix d'implementation

### 4.1 Image base + lib version

**Decision** : multi-stage build :

- **Builder stage** : `FROM rust:1.94-slim-bookworm` (tag
  pinne sur la meme version que le workspace SBFB pour
  parite supply-chain), `cargo install pkarr-relay
  --version 2.x.* --root /build/dist`, version pin `2.x.*`
  (minor range, auto-pickup security patches sans bump
  major).
- **Runtime stage** : `FROM debian:bookworm-slim` (~80 MB
  vs `rust:1.94-slim` ~700 MB) avec uniquement `ca-
  certificates` + `curl` (pour healthcheck) + le binaire
  copie. Taille finale visee : **~95 MB**.

**Distroless rejete** : `gcr.io/distroless/cc-debian12`
fournirait ~25 MB plus petit, mais (a) pas de shell pour
debug d'urgence ops ONG (cf. §3 ArchWiki note "Distroless
images lack shell and thus are harder to debug"), (b) pas
de `curl` pour healthcheck, devrait etre remplace par un
TCP probe qui ne valide pas le response body. Le delta 25 MB
ne justifie pas la friction debug pour un sysadmin ONG non-
expert.

**Alpine rejete** : musl libc compat avec Rust statiquement
linke sur glibc Hetzner host = piege classique (cf. §3 search
"Alpine's musl libc implementation instead of glibc creates
compatibility issues"). Pkarr-relay n'a pas de raison
specifique de bouger sur musl, et Hetzner/Debian = parite
host-container glibc.

**Pin pkarr-relay** : minor range `2.x.*` (verifier au
build via `cargo install --version` que le crate publie sur
crates.io exists et n'a pas d'advisory). Si la 3.x sort en
breaking, **bump deliberes** S20+ apres review changelog (et
re-test smoke contre relais Pubky existants).

**CVE check builder Rust** : la base `rust:1.94-slim-
bookworm` est elle-meme scannee par Trivy (cf. §4.5).
Wasmtime CVE avril 2026 n'affecte **pas** pkarr-relay (pas
de wasm runtime dans le binaire), donc pas d'impact direct.
**A re-verifier session fraiche** Phase E que le snapshot
crates.io de `pkarr-relay 2.x.*` ne pull pas
transitivement un crate vulnerable (run `cargo audit`
manuellement avant build).

### 4.2 Multi-stage build

```dockerfile
# syntax=docker/dockerfile:1.7
FROM rust:1.94-slim-bookworm AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo install pkarr-relay --version "2.*" --root /build/dist --locked
# --locked pour reproductibilite (lecon S18 Phase B)

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl && rm -rf /var/lib/apt/lists/* && \
    useradd --system --no-create-home --uid 10001 --shell /usr/sbin/nologin pkarr
COPY --from=builder /build/dist/bin/pkarr-relay /usr/local/bin/pkarr-relay
USER pkarr
EXPOSE 6881/tcp
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -fsS http://127.0.0.1:6881/_healthcheck || exit 1
ENTRYPOINT ["/usr/local/bin/pkarr-relay"]
CMD ["--config", "/etc/pkarr/config.toml"]
```

**Notes** :

- `--locked` cargo install : bloque l'install si lockfile
  upstream divergent (lecon S18 reproducible builds).
- User `pkarr` UID 10001 non-root : execution non-priv,
  defense en profondeur meme si Docker daemon root.
- Healthcheck endpoint **`/_healthcheck`** : le pkarr-relay
  upstream expose un endpoint `_healthcheck` (verifier
  exact name session fraiche Phase E via `cargo install
  pkarr-relay && pkarr-relay --help` ; si absent, fallback
  TCP probe `nc -z 127.0.0.1 6881`).
- `EXPOSE 6881/tcp` : le port HTTP du relai (BEP44 over
  HTTP). **Pas** de port UDP : pkarr-relay est un cache
  HTTP en frontal du Mainline DHT, le relai lui-meme parle
  UDP au DHT en sortie mais expose seulement HTTP en
  entree (clients).

### 4.3 Healthcheck

**Decision** : `HEALTHCHECK` Docker natif (`curl -fsS http://
127.0.0.1:6881/_healthcheck`) avec timing 30s/5s/3 retries.

**Rationale endpoint interne pkarr** :
- Si pkarr-relay 2.x expose `/_healthcheck` → utiliser
  (verifier via `--help` Phase E session fraiche).
- Si non-expose → fallback TCP probe via `nc -z 127.0.0.1
  6881` (probe minimal mais valide pas le response body, ok
  pour un service bien-defini).

**Cote ops Hetzner** : le systemd unit ajoute aussi un
`ExecStartPre=/usr/bin/curl -fsS http://127.0.0.1:6881/
_healthcheck` apres `sleep 5` au boot pour fail-fast si le
binaire crash au demarrage (pattern `Type=notify` vs
`Type=simple` : `simple` retenu car pkarr-relay ne signale
pas systemd).

### 4.4 Registry choix : ghcr.io

**Decision** : **ghcr.io/sbfb50/pkarr-relay**.

| Critere | ghcr.io (RETENU) | Docker Hub | Quay.io | Self-hosted Harbor |
|---|---|---|---|---|
| **Free tier OSS** | Free unlimited public images, no pull rate limit | Free **mais** 200 pulls/6h sur le free tier (restrictif) | Free public repos + 100 GB storage / 1 TB egress | Cost ~10 EUR/mois VPS |
| **Cosign + SLSA support** | Native (OCI 1.1 referrers, OIDC GitHub) | Oui mais auth keyless GitHub OIDC moins natif | Native (Project Quay supporte cosign) | Manuel |
| **Trust chain** | Tied a GitHub org `SBFB50` (parite Codeberg mirror S18) | Vendor distinct (Docker Inc) | Red Hat (IBM) | SBFB-controlled (mais SPOF) |
| **Bandwidth ONG-pull** | Free unlimited | Limite 200/6h = ONG botcheck pull casse | Quotas mais permissif | Notre charge |
| **Deplacement futur** | Migrer = re-tag + push, OCI standard | Idem | Idem | Idem |

**Justification ghcr.io** :

- **Parite supply-chain S18** : le code source est sur
  `github.com/SBFB50/SBFB`, le mirror Codeberg `codeberg.
  org/SBFB/SBFB`. Heberger l'image docker sur **un 4eme
  vendor** (Docker Hub ou Quay) ajoute un point d'attaque
  supply-chain sans benefice. ghcr.io tient sur la **meme**
  identite GitHub (compromis GitHub Actions = compromis
  ghcr.io anyway).
- **Pas de pull rate limit** = critere bloquant pour des
  ONG potentiellement derriere un seul NAT avec 50+ techs
  qui pull en parallel apres une release.
- **Cosign keyless GitHub OIDC** = le workflow GHA peut
  signer avec son identite OIDC sans gerer une cle privee
  (zero secret a roll, zero risque leak).

**Trade-off accepte** : ghcr.io status "free" pour OSS est
**non-contractuel** (cf. recherche §3 GitHub Discussion 183054
"Currently Free Status"). Si GitHub flip le billing 2027+,
on migre ghcr → Quay (effort estime ~1 jour : push tag, update
doc PKARR_RELAY_OPS, update relays.json). Backup plan
documente dans §6.

**Self-hosted Harbor rejete** : SBFB n'a pas la capacite ops
2026 pour maintenir un registry. C'est une fausse
independance (le VPS qui heberge Harbor = SPOF de meme
nature).

### 4.5 Workflow GHA `build-pkarr-image.yml`

#### 4.5.1 Triggers + permissions

```yaml
name: build-pkarr-image
on:
  push:
    branches: [master]
    paths:
      - "docker/pkarr-relay/**"
      - ".github/workflows/build-pkarr-image.yml"
  push:
    tags: ["v*"]
  workflow_dispatch:

permissions:
  contents: read
  packages: write
  id-token: write   # cosign keyless OIDC
  attestations: write   # GitHub native attestations API
```

**Rationale path filter** : eviter de rebuild l'image a
chaque commit Rust qui ne touche pas au Dockerfile. Trigger
explicite via `workflow_dispatch` pour rebuild force (apres
upstream pkarr release par exemple).

#### 4.5.2 Multi-arch ?

**Decision** : **amd64 uniquement S19**, **arm64 reporte
S22+**.

**Rationale** :
- Hetzner CX22 = amd64. Les VPS budget Europe 2026 sont
  majoritairement amd64.
- arm64 multi-arch via `docker/setup-qemu-action` rallonge le
  build de ~5min a ~15min (emulation QEMU), cout CI sans
  benefice user immediat S19.
- Si une ONG veut deployer sur Raspberry Pi 5 (arm64), elle
  peut build localement depuis le Dockerfile (documente dans
  README docker/).
- Bumper a multi-arch viendra naturellement si on observe un
  vrai besoin user (issue GH "support arm64").

#### 4.5.3 Workflow steps cibles

```yaml
jobs:
  build-and-push:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout (pinned SHA — cf. lecon trivy mars 2026)
        uses: actions/checkout@<SHA-pinned-S20-policy>

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@<SHA-pinned>

      - name: Log in to GHCR
        uses: docker/login-action@<SHA-pinned>
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push (amd64 only S19)
        id: build
        uses: docker/build-push-action@<SHA-pinned>
        with:
          context: docker/pkarr-relay
          platforms: linux/amd64
          push: true
          tags: |
            ghcr.io/sbfb50/pkarr-relay:${{ github.ref_name }}
            ghcr.io/sbfb50/pkarr-relay:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max
          provenance: true   # SLSA L1 native build provenance
          sbom: true         # SPDX SBOM native

      - name: Trivy scan (HIGH+CRITICAL fail-build)
        uses: aquasecurity/trivy-action@<SHA-pinned-post-mars-2026>
        with:
          image-ref: ghcr.io/sbfb50/pkarr-relay:${{ github.ref_name }}
          severity: CRITICAL,HIGH
          exit-code: 1
          ignore-unfixed: true

      - name: Cosign install
        uses: sigstore/cosign-installer@<SHA-pinned>

      - name: Cosign sign keyless
        env:
          COSIGN_EXPERIMENTAL: "true"
        run: |
          cosign sign --yes \
            ghcr.io/sbfb50/pkarr-relay@${{ steps.build.outputs.digest }}

      - name: Generate SLSA L2 in-toto attestation (parite S18 Phase B)
        run: |
          cosign attest --yes \
            --predicate provenance.json \
            --type slsaprovenance \
            ghcr.io/sbfb50/pkarr-relay@${{ steps.build.outputs.digest }}

      - name: GitHub native build provenance attestation
        uses: actions/attest-build-provenance@<SHA-pinned>
        with:
          subject-name: ghcr.io/sbfb50/pkarr-relay
          subject-digest: ${{ steps.build.outputs.digest }}
          push-to-registry: true
```

**Notes critiques** :
- **Pin SHA toutes les actions** : lecon attaque trivy
  fevrier-mars 2026 (76/77 tags force-pushed vers du
  malware). HARDENING_ROADMAP §3 S18 E3-2 P3 carry-over
  reporte le pin SHA des 4 workflows GHA SBFB en une fois
  vers un sprint security ops dedie. **Phase E S19 doit
  rejoindre cette policy** (pas de tag `@v3` mutable, full
  SHA 40-char).
- **`provenance: true` + `sbom: true`** : Docker build-push-
  action v5+ genere natif l'attestation SLSA L1 + SBOM
  SPDX. Sans cosign sign, c'est deja une provenance
  attestee par l'identite OIDC du runner GHA.
- **Cosign sign separe** : L'attestation cosign attest
  --type slsaprovenance superpose une **2eme** provenance
  signee plus largement reconnue (Sigstore Rekor
  transparency log entry queryable post-build). Parite avec
  SLSA L2 (signature CI-side, non-isole du build job — pas
  encore L3).
- **Trivy `ignore-unfixed: true`** : bloque seulement les
  CVE pour lesquelles un patch existe. Une CVE upstream non
  patchee n'arrete pas le build (sinon SBFB serait bloque
  des qu'une CVE Debian apparait sans patch immediat). Cf.
  pattern S18 Phase A.

**Trivy supply-chain attack mars 2026 — leçon retenue** :
- Pin `aquasecurity/trivy-action` au **SHA exact du commit
  pre-incident** (verifier au moment Phase E que le SHA
  pinne pointe vers un commit pre-fevrier 2026 qui n'a pas
  ete force-pushed, ou explicitement post-recovery audite).
- Documenter dans le workflow YAML un commentaire `# pinned
  to <SHA> verified post-mars-2026-incident
  recovery, see https://github.com/aquasecurity/trivy/
  security/advisories/GHSA-69fq-xp46-6x23`.

### 4.6 Image signature & SBOM

**Pile retenue** : **Cosign keyless** (Sigstore Fulcio +
Rekor transparency log) + **GitHub Native Attestations API**
(redondance intentionnelle).

**Pourquoi 2 mecanismes** :

- **Cosign keyless** : standard de facto OSS 2026, signature
  attachee a l'identite OIDC GHA (`https://github.com/SBFB50/
  SBFB/.github/workflows/build-pkarr-image.yml@refs/heads/
  master`). Verifiable par n'importe qui avec `cosign verify`
  + identite GHA matching.
- **GitHub Native Attestations** (`actions/attest-build-
  provenance@v1`) : attestation aussi push sur le registry
  via OCI Referrers API, queryable via `gh attestation
  verify`. Format SLSA in-toto v1.0 standard.

**SBOM** : genere natif par `docker/build-push-action@v5+`
via `sbom: true` (utilise BuildKit + buildx attest backend,
format SPDX 2.3). Attache a l'image via OCI 1.1 referrers
(non-tag-pollution).

**Cohérence S18 SLSA setup** : S18 Phase B livre l'attestation
SLSA in-toto pour worker/daemon/launcher/wheel binaries via
`provenance.json` + verification offline. Phase E reprend la
**meme primitive predicate** (`type slsaprovenance`,
buildType URI standard `slsa.dev/build-type/...` corrige par
audit S18 P3 batch `1a606a3`). Verification cote operateur
ONG :

```bash
# Verify keyless signature
cosign verify \
  --certificate-identity=https://github.com/SBFB50/SBFB/.github/workflows/build-pkarr-image.yml@refs/heads/master \
  --certificate-oidc-issuer=https://token.actions.githubusercontent.com \
  ghcr.io/sbfb50/pkarr-relay@sha256:...

# Verify SLSA provenance attestation
cosign verify-attestation \
  --certificate-identity=https://github.com/SBFB50/SBFB/.github/workflows/build-pkarr-image.yml@refs/heads/master \
  --certificate-oidc-issuer=https://token.actions.githubusercontent.com \
  --type slsaprovenance \
  ghcr.io/sbfb50/pkarr-relay@sha256:...
```

Ces 2 commandes sont copy-paste dans `PKARR_RELAY_OPS.md §2`
(provisioning) **avant** de lancer `docker pull` en prod, pour
que le sysadmin ONG verifie la chain of trust.

---

## 5. Ops doc PKARR_RELAY_OPS.md — sections

Le doc est conçu pour **30 minutes deploy time** par un
sysadmin Linux moyen, **sans expertise Rust ni P2P**.

### 5.1 Provisioning Hetzner CX22

**Format** : **commands copy-paste** dans des blocs bash,
pas de longs paragraphes.

```bash
# 1. Creer instance Hetzner CX22 (web console : 2 vCPU, 4 GB RAM,
#    40 GB SSD NVMe, ~7.99 EUR/mois HT, region Helsinki/Falkenstein
#    pour latence Europe optimale + juridiction UE robuste)
#    Image OS : Ubuntu 24.04 LTS
#    SSH key upload via console Hetzner

# 2. SSH initial + setup user non-root
ssh root@<ip>
adduser --disabled-password --gecos "" pkarr-ops
usermod -aG sudo pkarr-ops
mkdir /home/pkarr-ops/.ssh && cp /root/.ssh/authorized_keys /home/pkarr-ops/.ssh/
chown -R pkarr-ops: /home/pkarr-ops/.ssh && chmod 700 /home/pkarr-ops/.ssh
sed -i 's/^PermitRootLogin .*/PermitRootLogin no/' /etc/ssh/sshd_config
systemctl reload sshd

# 3. UFW firewall (ssh 22, http 80, https 443 — pas de port
#    pkarr direct expose internet, tout passe par Caddy reverse-
#    proxy + cert)
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw enable
ufw status verbose

# 4. Install Docker Engine (script officiel Docker)
ssh pkarr-ops@<ip>
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker pkarr-ops
# logout + relogin pour activer groupe docker
```

### 5.2 systemd unit + hardening

```ini
# /etc/systemd/system/pkarr-relay.service
[Unit]
Description=pkarr-relay (SBFB-self-hosted)
After=docker.service network-online.target
Wants=network-online.target
Requires=docker.service

[Service]
Type=simple
User=pkarr-ops
ExecStartPre=-/usr/bin/docker pull ghcr.io/sbfb50/pkarr-relay:v1.0
ExecStartPre=-/usr/bin/docker rm -f pkarr-relay
ExecStart=/usr/bin/docker run --rm --name pkarr-relay \
    -p 127.0.0.1:6881:6881 \
    -v /var/lib/pkarr-relay/cache:/var/lib/pkarr/cache \
    -v /etc/pkarr/config.toml:/etc/pkarr/config.toml:ro \
    --read-only \
    --tmpfs /tmp \
    --cap-drop=ALL \
    --security-opt=no-new-privileges:true \
    ghcr.io/sbfb50/pkarr-relay:v1.0
ExecStop=/usr/bin/docker stop pkarr-relay
Restart=on-failure
RestartSec=10s

# systemd hardening (cf. recherche §3 redhat.com/blog/mastering-
# systemd, ArchWiki Sandboxing baseline)
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes
RestrictNamespaces=yes
RestrictRealtime=yes
RestrictSUIDSGID=yes
LockPersonality=yes
MemoryDenyWriteExecute=yes
SystemCallArchitectures=native
SystemCallFilter=@system-service
SystemCallFilter=~@privileged @resources
ReadOnlyPaths=/
ReadWritePaths=/var/lib/pkarr-relay
CapabilityBoundingSet=
AmbientCapabilities=

[Install]
WantedBy=multi-user.target
```

**Notes** :
- Le bind `127.0.0.1:6881:6881` impose le passage par Caddy
  (loopback only, pas d'expose direct internet du port pkarr).
  Si tu n'utilises pas Caddy, change a `0.0.0.0:6881:6881` +
  `ufw allow 6881/tcp` (mais alors pas de TLS = casse Phase
  C SBFB-side).
- `--read-only` + `--tmpfs /tmp` = container immutable au
  runtime (defense en profondeur en cas de RCE pkarr).
- `--cap-drop=ALL` = container sans capabilities Linux
  privilegies.
- `SystemCallFilter=~@privileged @resources` = bloque les
  syscall priv et resource-management (depasses pour un
  cache HTTP simple).
- `ReadOnlyPaths=/` + `ReadWritePaths=/var/lib/pkarr-relay`
  = systemd reapplique read-only meme si docker container
  bypass.

### 5.3 nginx reverse proxy + Let's Encrypt — **rejete au profit de Caddy**

**Decision** : **Caddy** comme reverse proxy + auto-HTTPS.

**Rationale comparatif** :

| Critere | Caddy (RETENU) | nginx + certbot | nginx + acme.sh |
|---|---|---|---|
| **Config simplicite** | 3 lignes Caddyfile | ~30 lignes nginx + cron | Idem nginx + acme.sh script |
| **Auto-HTTPS** | Native (ACME built-in, renew auto) | Certbot externe + cron + reload nginx | Idem |
| **HTTP/3** | Native, active si HTTPS active | Plugin compile-time | Plugin |
| **Throughput** | ~36k req/s HTTPS | ~38k req/s HTTPS | Idem nginx |
| **Adapte ONG sysadmin moyen** | Tres oui (zero ACME knowledge) | Moyen (debug certbot = pas trivial) | Faible (acme.sh peu connu) |
| **Throughput pkarr-relay** | ~10 req/s typique (cache HTTP) — gap throughput non-pertinent | Idem | Idem |

Le **gap throughput** nginx vs Caddy (~6%) est non-pertinent
pour un pkarr-relay (charge ~10 req/s typique d'apres
documentation Pubky relays.md "very light and cheap to
operate"). La **simplicite ops** est le critere bloquant pour
un operateur ONG sysadmin moyen.

**Caddyfile minimal** :

```caddyfile
# /etc/caddy/Caddyfile
pkarr.example-ong.org {
    reverse_proxy 127.0.0.1:6881

    # Headers securite minimaux
    header {
        Strict-Transport-Security "max-age=31536000; includeSubDomains"
        X-Content-Type-Options "nosniff"
        Referrer-Policy "no-referrer"
    }

    # Rate limit basique (anti-burst, pas anti-DDoS sustained)
    rate_limit {
        zone pkarr_zone {
            key {client_ip}
            events 100
            window 1m
        }
    }
}
```

**Install Caddy Hetzner** :

```bash
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/caddy-stable-archive-keyring.gpg] https://dl.cloudsmith.io/public/caddy/stable/deb/debian any-version main" | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update && sudo apt install -y caddy

# Configurer DNS A record pour pkarr.example-ong.org → IP Hetzner
# AVANT de start Caddy (sinon ACME challenge fail loop).
sudo cp Caddyfile /etc/caddy/Caddyfile
sudo systemctl reload caddy
sudo systemctl status caddy
```

### 5.4 Smoke test (`pkarr-cli publish/resolve`)

```bash
# Cote dev, install pkarr-cli (binaire upstream Rust)
cargo install pkarr-cli --version "2.*"

# Test publish vers le relai self-hosted
pkarr-cli put --relays https://pkarr.example-ong.org \
    "test_record_value_$(date +%s)"

# Test resolve via le meme relai
PUBKEY=<ed25519-pubkey-from-publish-output>
pkarr-cli get --relays https://pkarr.example-ong.org $PUBKEY

# Verify roundtrip (test cross-relai)
pkarr-cli get --relays https://relay.iroh.network $PUBKEY
# Doit retourner le meme record (federation ok)
```

**Critere acceptation deploy** : 3/3 tests passent. Si le
3eme echoue, verifier que `Caddy` propage bien le request body
(POST avec contenu binaire) et que la config TLS Caddy n'a
pas un timeout trop court pour les uploads pkarr.

### 5.5 Monitoring baseline

**Decision S19** : **journalctl + df-h + uptime** uniquement.
Pas de Prometheus/Grafana/Loki.

**Rationale** : Phase E livre **un** relai single-instance.
Stack monitoring complete = scope creep S22+ (federation
multi-relai justifie monitoring centralise). Pour 1 relai,
les 4 commandes suivantes suffisent :

```bash
# Logs container
journalctl -u pkarr-relay -f --since "1 hour ago"

# Filtrer 5xx errors
journalctl -u pkarr-relay --since today | grep -E "ERROR|5[0-9][0-9]"

# Disk usage cache pkarr
df -h /var/lib/pkarr-relay
du -sh /var/lib/pkarr-relay/cache

# Connections actives
ss -tnH state established '( sport = :6881 )' | wc -l
```

**Cron healthcheck cote dev** (optionnel) : un script
`scripts/check-relay.sh` qui POST + GET un test record
toutes les 5 min depuis le dev box, alerte par email si fail
2 fois consecutives. Documente comme **optionnel** dans §5
PKARR_RELAY_OPS, pas requis pour S19.

### 5.6 Rotation SPKI cert (cross-ref Phase C TLS pinning)

**Cas d'usage** : Caddy rotate auto le cert TLS Let's Encrypt
tous les ~60 jours. Le **SPKI hash** du nouveau cert peut
changer (selon si la **cle** privee est rotee aussi — par
defaut Caddy reutilise la cle, donc SPKI **stable** entre
renewals).

**Procedure rotation explicite (cle bumpee)** :

```bash
# 1. Cote relai self-hosted, force key roll Caddy
sudo rm /var/lib/caddy/.local/share/caddy/certificates/acme-v02.api.letsencrypt.org-directory/pkarr.example-ong.org/pkarr.example-ong.org.key
sudo systemctl reload caddy
# Caddy regenere une key + re-emet cert via ACME

# 2. Extraire nouveau SPKI hash
echo | openssl s_client -connect pkarr.example-ong.org:443 -servername pkarr.example-ong.org 2>/dev/null \
    | openssl x509 -pubkey -noout \
    | openssl pkey -pubin -outform DER \
    | openssl dgst -sha256 -binary \
    | base64
# Output : "Xxxxxxx..."

# 3. Cote chaque client SBFB, bumper relay-pins.json
# Editer ~/.sbfb/relay-pins.json :
{
  "pins": [
    {
      "relay_url": "https://pkarr.example-ong.org",
      "spki_sha256": "<NEW-HASH-from-step-2>",
      "added_at": "2026-MM-DD",
      "source": "rotation"
    }
  ]
}
# Le file-watcher Phase C reload sans restart daemon
```

**Communication aux clients** : si le relai est federe
(annonce dans `relays.json` defaults), le mainteneur SBFB
push une release minor `relays.json` avec le nouveau pin
SHA. Pre-launch S19, pas de federation reelle = procedure
manuelle out-of-band (email / forum / Codeberg issue).

### 5.7 Federation onboarding

**Apres deploy reussi** :

1. **Verifier uptime stable** ~7 jours (`uptime` + `journalctl
   -u pkarr-relay --since "7 days ago" | grep -c ERROR` <
   100).
2. **Annonce publique** : ouvrir une issue
   `github.com/SBFB50/SBFB` avec template `[federation]
   Nouveau relai pkarr public — <ONG> — <region>` + URL
   relai + SPKI pin + commit signe maintainer ONG.
3. **Mainteneur SBFB review** :
   - DNS resolves + cert valide,
   - smoke test `pkarr-cli` passe,
   - SPKI hash matche,
   - juridiction declaree compatible avec mission anti-
     subpoena (eviter US-only ou autoritarian states).
4. **Add to defaults** : si review ok, mainteneur push une
   PR sur `relays.json` defaults SBFB qui ajoute le relai
   au pool quorum.
5. **Bump version SBFB** : la PR est landed dans une release
   minor SBFB (ex `v1.0.5`). Les clients qui upgrade
   pickent automatiquement le nouveau relai dans leur
   quorum.
6. **Tracking** : entree dans `docs/release/FEDERATED_RELAYS.
   md` (a creer sprint outreach futur) avec liste publique
   relais federes + leur juridiction + leur operateur.

---

## 6. Limites connues + futures evolutions

### 6.1 Pas de deploy reel S19 — quand premier relai reel ?

Phase E livre **l'image + la doc**. **Aucun relai
self-hosted SBFB n'existe a la cloture S19**. Le premier
deploy reel est une **decision ops separee** :

- **Cout** : ~8 EUR/mois Hetzner CX22. Qui paie ? FlowUP
  perso ? Donor ? Crowdfunding ?
- **Ops energy** : ~2h initial setup + ~1h/mois maintenance
  (rolling update, monitoring sanity check). Maintainable
  par FlowUP solo S20+ ?
- **Trigger** : decision pourrait etre **liee au tag v1.0
  go-live** (cohérence avec MIRROR_FALLBACK §3 flip
  sequence : passer en public Codeberg + Radicle + activer
  relai self-hosted simultanement = signal "infrastructure
  diversifiee" maximal pour les premiers users externes).

**Recommandation S19** : laisser la decision ouverte. Phase
E livre le levier, pas la decision.

### 6.2 1 relai = 1 SPOF si pas de pair

Un relai SBFB self-hosted seul ne change **rien** au quorum
2/3 (qui necessite 3 relais). Les benefices reels arrivent
**quand >= 2 relais ONG-operes** rejoignent la federation.

**Roadmap S22+ "multi-relay-self-hosted"** :
- S22 : outreach Amnesty + RSF + HRW (3 cibles initiales).
- S23 : si 1 ONG s'engage, livre Phase Federation Onboard
  (template ONG + support ops dedie).
- S24+ : monitoring centralise (Prometheus federate), si
  >= 3 relais federes.

### 6.3 Update strategy (auto-update vs manual)

**Decision S19** : **manual via systemd timer hebdomadaire**
(pas auto-update au pull `latest`).

```ini
# /etc/systemd/system/pkarr-relay-update.timer
[Unit]
Description=Weekly check for pkarr-relay image update

[Timer]
OnCalendar=Sun 03:00
Persistent=true

[Install]
WantedBy=timers.target
```

```bash
# /usr/local/bin/pkarr-relay-update.sh (executed by .service unit)
#!/usr/bin/env bash
set -euo pipefail
CURRENT=$(docker inspect --format '{{.Image}}' pkarr-relay 2>/dev/null || echo none)
docker pull ghcr.io/sbfb50/pkarr-relay:v1.0
NEW=$(docker inspect --format '{{.Id}}' ghcr.io/sbfb50/pkarr-relay:v1.0)
if [[ "$CURRENT" != "$NEW" ]]; then
    # Verify cosign signature BEFORE restart (defense supply chain)
    cosign verify \
        --certificate-identity=https://github.com/SBFB50/SBFB/.github/workflows/build-pkarr-image.yml@refs/heads/master \
        --certificate-oidc-issuer=https://token.actions.githubusercontent.com \
        ghcr.io/sbfb50/pkarr-relay:v1.0 || exit 1
    systemctl restart pkarr-relay
    echo "$(date) restarted to image $NEW" >> /var/log/pkarr-relay-updates.log
fi
```

**Pourquoi pas auto** : un compromise registry (cf. Trivy
mars 2026) injectant une image malicieuse serait pull en
silence et restart applique. Le **cosign verify avant
restart** est le minimum vital. Manual `systemctl restart`
post-review du changelog SBFB release notes serait encore
plus safe — laisse au choix operateur ONG (documente les 2
modes dans `PKARR_RELAY_OPS.md §rotation`).

### 6.4 Cost — modele ONG donor

~8 EUR/mois = 96 EUR/an. Pour ~10 relais federes long terme,
cout total ~1000 EUR/an. **Modele donor** : ONG partenaire
paie son propre relai (cohérent avec leur mission +
juridiction). SBFB ne sponsorise **pas** les relais ONG.

**Pre-launch (S19-S22)** : SBFB peut sponsoriser **1** relai
showcase (~96 EUR/an) si decision ops + funding ok. Pas de
budget federation crowdfunded encore.

### 6.5 Backup config

**Decision S19** : `pkarr-relay` cache est **regenerable**
(c'est un cache Mainline DHT, pas une source of truth).
Perte = re-build via lookup DHT au prochain query.
**Pas de backup** S19.

**Configs critiques** : seulement `Caddyfile` + systemd unit
+ TLS state Caddy (`/var/lib/caddy/`). Operateur ONG est
responsable de son backup OS-level (snapshot Hetzner =
3 EUR/mois option built-in).

### 6.6 R-pkarr-audit P2 — pas d'audit upstream pkarr public

Aligne avec les zones rouges S17 (R-iroh-audit P0,
R-wasmtime-cve P0, R-libcrux-hax P2). pkarr s'ajoute :

- **Code review interne** : reading `pubky/pkarr` source
  avant integration S19 = **carry-over Sprint 22+ task**
  (audit niveau "manuel par mainteneur SBFB", pas un audit
  externe paye).
- **Rustsec advisory db monitoring** : `cargo audit` dans CI
  S18 deja active = pickup auto les advisories pkarr-relay
  futurs.

**Phase E livre l'image avant audit complet** car (a)
pkarr-relay est upstream-controlled (notre rebuild ne
change rien au code), (b) Mainline DHT est public-by-design
(pas de secret SBFB exfiltrable via le relai), (c) le pin
S19 Phase C TLS empeche un MITM relai attacker.

---

## 7. References

### 7.1 pkarr upstream

- [pkarr github.com/pubky/pkarr](https://github.com/pubky/pkarr)
  — repo source, README, exemples docker compose
- [pkarr design/relays.md](https://github.com/Pubky/pkarr/blob/main/design/relays.md)
  — design doc relais HTTP cache layer Mainline DHT
- [pkarr-relay crates.io](https://lib.rs/crates/pkarr-relay)
  — crate Rust publie, version 2.x.* avril 2026
- [pkarr docs site](https://pubky.github.io/pkarr/)
  — Quick Start docker + config.toml example

### 7.2 Pubky network 2026

- [Pubky network architecture (Pubky.org)](https://docs.pubky.org/Explore/PubkyCore/Introduction)
  — Pubky Core architecture, federation status, role pkarr
  + homeservers + indexers
- [Pubky: The Next Web (Medium John Carvalho)](https://medium.com/pubky/pubky-the-next-web-3287b35408f1)
  — vision Pubky 2026, BEP44 DHT, 10M+ noeuds Mainline

### 7.3 Hosting + infra cost 2026

- [Hetzner Cloud Review 2026 (BetterStack)](https://betterstack.com/community/guides/web-servers/hetzner-cloud-review/)
  — pricing CX22 7.99 EUR/mois post-ajustement avril 2026
- [Hetzner Price Adjustment Docs](https://docs.hetzner.com/general/infrastructure-and-availability/price-adjustment/)
  — adjustment officiel avril 2026
- [Cloudflare Workers vs Fly.io vs Railway latency (OpenStatus 2026)](https://www.openstatus.dev/blog/monitoring-latency-cf-workers-fly-koyeb-raylway-render)
  — comparatif latency 2026 (pkarr-relay non-supporte par CF
  Workers car pas JavaScript-only)

### 7.4 Container registry + supply chain 2026

- [GHCR free tier vs Docker Hub vs Quay (Razorops 2026)](https://razorops.com/blog/top-10-free-container-registery-services/)
  — comparatif free tier OSS
- [GHCR billing discussion (GitHub Community 183054)](https://github.com/orgs/community/discussions/183054)
  — clarification "currently free" non-contractuel, plan B necessaire
- [Liquibase SBOM + SLSA L3 + Cosign on GHCR (Security Boulevard fev 2026)](https://securityboulevard.com/2026/02/supply-chain-security-for-liquibase-secure-docker-images-sbom-provenance-signing/)
  — implementation SLSA L3 cosign keyless OIDC GHA → ghcr.io

### 7.5 Container scan + SBOM 2026

- [Trivy supply chain attack mars 2026 GHSA-69fq-xp46-6x23](https://github.com/aquasecurity/trivy/security/advisories/GHSA-69fq-xp46-6x23)
  — incident force-push tags trivy-action, lecon pin SHA
- [Snyk vs Trivy 2026 comparison (DEV Community)](https://dev.to/rahulxsingh/snyk-vs-trivy-commercial-security-platform-vs-open-source-scanner-2026-5e4b)
- [Trivy vs Grype 2026 (AppSec Santa)](https://appsecsanta.com/sca-tools/trivy-vs-grype)

### 7.6 systemd + reverse proxy 2026

- [Mastering systemd hardening (Red Hat)](https://www.redhat.com/en/blog/mastering-systemd)
- [systemd Sandboxing (ArchWiki)](https://wiki.archlinux.org/title/Systemd/Sandboxing)
- [Systemd hardening 2026 Ubuntu (OneUptime)](https://oneuptime.com/blog/post/2026-03-02-how-to-configure-systemd-service-hardening-on-ubuntu/view)
- [Caddy automatic HTTPS](https://caddyserver.com/docs/automatic-https)
- [Nginx vs Caddy 2026 (PrivateDevops)](https://privatedevops.com/articles/nginx-vs-caddy-2026-reverse-proxy-comparison)
- [Let's Encrypt ACME client options](https://letsencrypt.org/docs/client-options/)

### 7.7 SLSA + cosign + reproducible builds 2026

- [SLSA L3 Docker images (Fystack 2026)](https://fystack.io/blog/secure-crypto-infrastructure-slsa-l3-provenance-for-docker-images-how-we-made-our-builds-verifiable)
- [Sigstore in-toto attestations (sigstore docs)](https://docs.sigstore.dev/cosign/verifying/attestation/)
- [SLSA L3 Kubernetes container provenance (OneUptime fev 2026)](https://oneuptime.com/blog/post/2026-02-09-slsa-level3-build-provenance/view)
- [BuildKit vs Kaniko archived (CodeCentric 2025-2026)](https://www.codecentric.de/en/knowledge-hub/blog/7-ways-to-replace-kaniko-in-your-container-image-builds)
  — Kaniko archived juin 2025, BuildKit retenu

### 7.8 GitHub Actions security 2026

- [GitHub Actions SHA pinning policy (StepSecurity)](https://www.stepsecurity.io/blog/pinning-github-actions-for-enhanced-security-a-complete-guide)
- [tj-actions/changed-files mars 2025 + Trivy mars 2026 attacks (DEV ameer-pk)](https://dev.to/ameer-pk/the-trivy-attack-why-sha-pinning-fails-github-actions-14if)

### 7.9 Image base + distroless 2026

- [Distroless vs Alpine vs Debian-slim attack surface (DasRoot janv 2026)](https://dasroot.net/posts/2026/01/building-minimal-container-images-with/)
- [Alpine or Debian hardened images (Medium Medha Goel fev 2026)](https://medium.com/@goel_medha/alpine-or-debian-the-security-decisions-that-shape-your-hardened-images-069c5aed657a)

### 7.10 context7 traces (avril 2026)

- **`/sigstore/cosign`** (queried 2026-04-16) : "Sign docker
  container image with keyless GitHub OIDC and attach SLSA
  in-toto attestation, verify signature, attach SBOM". 259
  snippets, source reputation Low (mais cohérent avec
  recherches WebSearch 2026 sur cosign+attest+SLSA).
  Validation du flow `cosign sign --yes` + `cosign attest
  --type slsaprovenance` + `cosign verify-attestation
  --certificate-identity https://github.com/SBFB50/SBFB/...`
  retenu §4.5.3 + §4.6.
- **`/websites/rs_iroh`** (cf. sprint19_plan.md §3.1, queried
  meme session) : `PkarrRelayClient::new(pkarr_relay_url)`
  + `.resolve(NodeId) -> SignedPacket` confirmant que
  pkarr-relay upstream parle exactement le protocole
  consomme par iroh 0.97 — donc notre image self-hosted est
  drop-in remplacable d'un relai n0.

### 7.11 CVE 2026 ecosysteme adjacent

- **CVE-2026-33056** (Cargo `tar` crate, mars 2026) — non
  applicable pkarr-relay (pas de crate tar dans la chaine
  dependencies pkarr).
- **Bytecode Alliance Wasmtime advisories avril 2026** —
  non applicable (pas de wasm runtime dans pkarr-relay).
- **Trivy supply-chain attack mars 2026 GHSA-69fq-xp46-6x23**
  — applicable au workflow GHA (cf. §4.5 lecon pin SHA).
- **Pas d'advisory pkarr-specifique observe avril 2026**
  (recherche `pubky pkarr CVE 2026` retourne null + RustSec
  advisory db ne liste pas pkarr-relay au moment du design).

---

**Note de placement** : ce design doc est ecrit dans
`.planning/research/` (pattern S17 recherche pure +
S18 designs phases — cf. `.planning/research/STACK.md`,
`ARCHITECTURE.md`, etc.). Il sera reference par le commit
Phase E :

```
feat(sprint19): Phase E — pkarr relay self-hosted docker image + ops doc

[...]
Design doc : .planning/research/S19_phase_E_pkarr_relay_design.md
(6 alternatives strategiques considerees, 1 trace context7
cosign + 10 sources WebSearch sur upstream pkarr / Hetzner /
ghcr / Trivy / systemd / Caddy / SLSA / BuildKit / distroless /
GHA SHA pinning).
```
