<!--
SPDX-License-Identifier: AGPL-3.0-or-later
-->

# pkarr-relay (SBFB self-hosted image)

Image Docker buildable depuis `pubky/pkarr` upstream
(`pkarr-relay` crate, ^0.11), distribuee sous
`ghcr.io/sbfb50/pkarr-relay:<tag>` pour permettre a un mainteneur
SBFB ou une ONG de spin up un relai pkarr en ~30 minutes dans sa
juridiction.

Cette image ne remplace pas les 3 relais n0 par defaut d'iroh
0.97. Elle s'y ajoute pour diversifier la base de la federation
pkarr (cf. HARDENING_ROADMAP §3 S19 + design doc
`.planning/research/S19_phase_E_pkarr_relay_design.md`).

## Build local

```bash
cd docker/pkarr-relay
docker build -t pkarr-relay:local .
```

Build time ~5-10 min (first time, cold cargo index). Le stage
builder (`rust:1.94-slim-bookworm`) est jetable ; l'image finale
(`debian:bookworm-slim`) pese ~100 MB.

## Run rapide (dev / smoke test)

```bash
docker run --rm --name pkarr-relay \
    -p 127.0.0.1:6881:6881/tcp \
    -p 6881:6881/udp \
    -v pkarr-cache:/var/lib/pkarr/cache \
    pkarr-relay:local

# Dans un autre terminal
curl -fsS http://127.0.0.1:6881/
```

Le dashboard HTML du relai s'affiche (stats cache + DHT). Les
routes `GET /:pubkey` et `PUT /:pubkey` servent le protocole
BEP44-over-HTTP consomme par iroh 0.97 via `PkarrRelayClient`.

## Run production (Hetzner ou autre VPS)

Cf. [`docs/release/PKARR_RELAY_OPS.md`](../../docs/release/PKARR_RELAY_OPS.md)
pour le provisioning complet (Hetzner CX22, systemd unit
hardene, Caddy auto-HTTPS, rotation SPKI cert, onboarding
federation).

Quick-start copy-paste :

```bash
# 1. Pull + verify cosign (chain of trust SLSA L2 S19)
cosign verify \
    --certificate-identity=https://github.com/SBFB50/SBFB/.github/workflows/build-pkarr-image.yml@refs/heads/master \
    --certificate-oidc-issuer=https://token.actions.githubusercontent.com \
    ghcr.io/sbfb50/pkarr-relay:latest

# 2. Run via systemd unit (voir PKARR_RELAY_OPS.md §3)
sudo systemctl enable --now pkarr-relay.service
sudo journalctl -u pkarr-relay -f
```

## Configuration

Le fichier [`config.toml`](./config.toml) livre avec l'image
reflete les defaults upstream pkarr + ajustements SBFB (rate
limiter actif derriere reverse-proxy, cache path sur volume
docker). Pour override, monter un fichier externe :

```bash
docker run -v ./my-config.toml:/etc/pkarr/config.toml:ro ...
```

Reference complete : [pkarr upstream docs](https://github.com/pubky/pkarr/blob/main/relay/src/config.example.toml).

## Ports exposes

| Port | Protocole | Usage |
|---|---|---|
| 6881 | TCP | HTTP — dashboard + BEP44-over-HTTP routes |
| 6881 | UDP | Mainline DHT (membership + record resolve) |

Meme numero pour 2 protocoles = convention upstream pkarr. Les
deux doivent passer le firewall.

## Supply-chain + provenance

L'image est signee **keyless via Cosign** (GitHub OIDC → Fulcio
→ Rekor transparency log) et accompagnee d'une attestation SLSA
in-toto (parite S18 Phase B) + un SBOM SPDX natif `buildx`. Voir
`.github/workflows/build-pkarr-image.yml` pour le workflow de
build et `PKARR_RELAY_OPS.md §2` pour la procedure de verification
cote ops.

## Trouble-shooting

- `docker logs pkarr-relay` → logs pkarr (tracing)
- `curl http://127.0.0.1:6881/` → doit retourner HTML 200, sinon
  le HTTP server n'est pas up (verif `journalctl` container)
- DHT stale : `docker volume rm pkarr-cache` + restart (le cache
  est regenerable depuis le DHT Mainline, pas une source of
  truth)

## License

AGPL-3.0-or-later (meme license que le reste du projet SBFB).
Le crate `pkarr-relay` upstream est MIT ; notre image le wrappe
sans modifications, donc compatible.
