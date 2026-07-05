<!--
SPDX-License-Identifier: AGPL-3.0-or-later
written: 2026-04-16  # Sprint 19 Phase E
last_validated: 2026-04-16  # initial write
triggers_revalidate:
  - "pkarr-relay upstream release > 0.11.x (potential breaking 0.12 or 1.0)"
  - "Hetzner pricing adjustment (CX22 EUR/mois change)"
  - "Caddy 3.x release (config syntax may shift)"
  - "Trivy supply-chain incident recurrence or action SHA-pin sweep"
  - "SBFB Gate 1 unlock / pre-launch federation outreach (S22+) → first real deploy"
audited_findings:
  - "2026-04-16 Phase E initial — 3 errata pre-draft caught in design doc §0 (pkarr-relay version 2.* → ^0.11, /_healthcheck → /, ports 6881/tcp+udp)"
-->

# PKARR_RELAY_OPS — ops runbook self-hosted pkarr relay

> **Portée (précision Sprint 81 Phase E2)** : ce runbook couvre l'image
> pubky `pkarr-relay` adossée au DHT **Mainline** (port 6881), qui
> alimente le **canari quorum anti-eclipse** (`SBFB_PKARR_RELAYS`) —
> jamais la discovery de l'endpoint iroh. Pour le mode **zéro-n0**
> (relais iroh + pkarr self-hosted face à l'EOL n0 2026-09-30, PLAN B
> C8), voir [`IROH_SELFHOST_OPS.md`](IROH_SELFHOST_OPS.md) : autre
> outil (`iroh-dns-server`), autre topologie, autres ports.

Cible : sysadmin Linux moyen, **pas de Rust requis**, ~30 min
pour un premier deploy fonctionnel sur Hetzner CX22 (ou
equivalent, ~8 EUR/mois avril 2026).

Ce document couvre l'image `ghcr.io/sbfb50/pkarr-relay:<version>`
livree en Sprint 19 Phase E. Le design + rationale est dans
[`.planning/research/S19_phase_E_pkarr_relay_design.md`](../../.planning/research/S19_phase_E_pkarr_relay_design.md).

> **Status 2026-04-16** : **pas** de relai SBFB self-hosted
> deploye. Ce document decrit la procedure pour quand une ONG
> partenaire ou un mainteneur SBFB decide de spin up un premier
> relai (trigger probable : tag v1.0 + flip MIRROR_FALLBACK §3).

---

## 1. Prerequis

| Ressource | Specification minimale | Recommande |
|---|---|---|
| VPS | 1 vCPU / 2 GB RAM / 20 GB SSD | Hetzner **CX22** (2 vCPU / 4 GB / 40 GB NVMe) |
| OS | Debian 12 ou Ubuntu 22.04+ | Ubuntu 24.04 LTS |
| Region | Peut-etre n'importe quelle | Europe (Helsinki FIN01 ou Falkenstein FSN1) pour juridiction UE robuste |
| DNS | Domaine ou sous-domaine dedie | `pkarr.<ong>.org` |
| Reseau | UDP 6881 sortant autorise (pour le DHT Mainline) | + TCP 80/443 entrant (Caddy ACME) |
| Cosign | CLI v2.4+ | [installation](https://docs.sigstore.dev/cosign/installation) |

**Cout indicatif** : ~8 EUR/mois Hetzner CX22 + 0 EUR domaine
(si sous-domaine existant). Snapshot backup Hetzner ~3 EUR/mois
(optionnel).

---

## 2. Verification chaine de confiance (obligatoire avant deploy)

L'image pkarr-relay est signee **keyless via Cosign** (Sigstore
Fulcio + Rekor transparency log) et inclut une attestation SLSA
in-toto. Avant de pull sur le VPS :

```bash
export IMAGE="ghcr.io/sbfb50/pkarr-relay:v1.0"  # bumper au tag voulu
export GH_IDENTITY="https://github.com/SBFB50/SBFB/.github/workflows/build-pkarr-image.yml@refs/heads/master"

# 1. Verifier la signature cosign keyless
cosign verify \
    --certificate-identity="${GH_IDENTITY}" \
    --certificate-oidc-issuer="https://token.actions.githubusercontent.com" \
    "${IMAGE}"

# 2. Verifier l'attestation SLSA build provenance native GitHub
gh attestation verify --type slsaprovenance \
    oci://${IMAGE}  # ou: cosign verify-attestation sur meme subject

# 3. (Optionnel) Inspecter le SBOM SPDX pour enumerer les
#    dependances transitivees
cosign download sbom "${IMAGE}" | jq '.packages[].name' | sort -u
```

**Si une des commandes echoue** : ne pas pull en prod. Escalade
sur `github.com/SBFB50/SBFB/issues` avec tag `supply-chain`.

---

## 3. Installation pas-a-pas Hetzner CX22 + Ubuntu 24.04

### 3.1 Provisioning console Hetzner

1. Creer instance CX22 (2 vCPU x86, 4 GB RAM, 40 GB SSD NVMe),
   Ubuntu 24.04 LTS, region Helsinki (recommande) ou Falkenstein.
2. Upload cle SSH publique via console Hetzner.
3. Assigner un DNS A record `pkarr.<votre-domaine>.org` vers
   l'IP publique avant de lancer Caddy (sinon ACME challenge
   loop).

### 3.2 Setup user non-root + firewall

```bash
# SSH initial avec la cle Hetzner
ssh root@<ip>

# Creer user ops non-root avec sudo
adduser --disabled-password --gecos "" pkarr-ops
usermod -aG sudo pkarr-ops
mkdir /home/pkarr-ops/.ssh
cp /root/.ssh/authorized_keys /home/pkarr-ops/.ssh/
chown -R pkarr-ops: /home/pkarr-ops/.ssh
chmod 700 /home/pkarr-ops/.ssh
chmod 600 /home/pkarr-ops/.ssh/authorized_keys

# Disable root SSH
sed -i 's/^#*PermitRootLogin.*/PermitRootLogin no/' /etc/ssh/sshd_config
systemctl reload sshd

# UFW firewall : SSH + HTTP + HTTPS. Port pkarr 6881 reste
# derriere Caddy, pas expose direct.
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw enable
ufw status verbose

# Se reconnecter comme pkarr-ops
exit
ssh pkarr-ops@<ip>
```

### 3.3 Installer Docker Engine + Caddy

```bash
# Docker via script officiel
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker pkarr-ops
# LOGOUT + login again pour activer le groupe docker
exit
ssh pkarr-ops@<ip>

# Caddy
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' \
    | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/caddy-stable-archive-keyring.gpg] https://dl.cloudsmith.io/public/caddy/stable/deb/debian any-version main" \
    | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update && sudo apt install -y caddy
```

### 3.4 Configurer Caddy reverse-proxy + auto-HTTPS

```bash
sudo tee /etc/caddy/Caddyfile > /dev/null <<EOF
pkarr.<votre-domaine>.org {
    reverse_proxy 127.0.0.1:6881

    header {
        Strict-Transport-Security "max-age=31536000; includeSubDomains"
        X-Content-Type-Options "nosniff"
        Referrer-Policy "no-referrer"
    }
}
EOF

sudo systemctl reload caddy
sudo systemctl status caddy
```

**Rate-limiting** : ce Caddyfile n'inclut **pas** de rate-
limiter a la couche proxy. Caddy community OSS n'expose pas
`rate_limit` nativement (module `caddyserver/rate-limit` =
build custom via xcaddy). La protection par defaut vient du
`rate_limiter` interne du binaire pkarr-relay (cf. `config.
toml` livre avec l'image : `burst_size = 50`, `per_second =
10`, `behind_proxy = true`). Si une charge adversarielle
justifie un rate-limit a la couche proxy, rebuild Caddy avec
xcaddy + module et ajouter un bloc `rate_limit` au Caddyfile.

Caddy obtient le certificat Let's Encrypt automatiquement au
premier hit HTTPS (ACME HTTP-01 challenge). Attendre ~30
secondes puis verifier :

```bash
curl -fsS https://pkarr.<votre-domaine>.org/
# doit retourner du HTML avec "pkarr" dans le body
```

### 3.5 systemd unit pour le container pkarr-relay

```bash
sudo mkdir -p /etc/pkarr /var/lib/pkarr-relay/cache
sudo chown -R 10001:10001 /var/lib/pkarr-relay

# Le config.toml par defaut est dans l'image a /etc/pkarr/config.toml.
# On peut le monter en override si besoin — ici on garde le default.

sudo tee /etc/systemd/system/pkarr-relay.service > /dev/null <<'EOF'
[Unit]
Description=pkarr-relay (SBFB self-hosted)
After=docker.service network-online.target
Wants=network-online.target
Requires=docker.service

[Service]
Type=simple
User=pkarr-ops
ExecStartPre=-/usr/bin/docker pull ghcr.io/sbfb50/pkarr-relay:v1.0
ExecStartPre=-/usr/bin/docker rm -f pkarr-relay
ExecStart=/usr/bin/docker run --rm --name pkarr-relay \
    -p 127.0.0.1:6881:6881/tcp \
    -p 6881:6881/udp \
    -v /var/lib/pkarr-relay/cache:/var/lib/pkarr/cache \
    --read-only \
    --tmpfs /tmp \
    --cap-drop=ALL \
    --security-opt=no-new-privileges:true \
    ghcr.io/sbfb50/pkarr-relay:v1.0
ExecStop=/usr/bin/docker stop pkarr-relay
Restart=on-failure
RestartSec=10s

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
CapabilityBoundingSet=
AmbientCapabilities=

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable --now pkarr-relay.service
sudo journalctl -u pkarr-relay -f --since "1 minute ago"
```

**Notes de durcissement** :

- `127.0.0.1:6881:6881/tcp` = le HTTP du relai **n'est pas
  expose direct internet**, Caddy est le seul ingress TLS.
- `6881:6881/udp` = le DHT Mainline expose directement (pas
  de TLS over UDP standard, et le protocole Mainline parle en
  clair).
- `--read-only + --tmpfs + --cap-drop=ALL` = container immutable
  minimal-privilege (defense en profondeur en cas de RCE).

### 3.6 Smoke test deploy

```bash
# Test 1 : dashboard via Caddy TLS
curl -fsS https://pkarr.<votre-domaine>.org/ | head -5

# Test 2 : healthcheck local container
sudo docker exec pkarr-relay curl -fsS http://127.0.0.1:6881/ \
    > /dev/null && echo "OK: container healthy"

# Test 3 : le DHT Mainline est atteignable (packet egress UDP)
sudo docker logs pkarr-relay 2>&1 | grep -i "listening.*6881" \
    && echo "OK: DHT listener up"
```

3/3 = deploy reussi.

---

## 4. Rotation SPKI cert (cross-ref S19 Phase C TLS pinning)

Caddy rotate auto le cert TLS Let's Encrypt tous les ~60 jours
et **reutilise par defaut la meme cle privee**, donc le SPKI
hash **reste stable** entre renewals. Pas de synchronisation
client necessaire dans le cas general.

Si tu **roll la cle** intentionnellement (apres compromission,
rotation planifiee) :

```bash
# 1. Forcer Caddy a regenerer la cle
sudo rm /var/lib/caddy/.local/share/caddy/certificates/acme-v02.api.letsencrypt.org-directory/pkarr.<votre-domaine>.org/*.key
sudo systemctl reload caddy
# Attendre ~30s ACME challenge

# 2. Extraire le nouveau SPKI SHA-256 base64url
echo | openssl s_client -connect pkarr.<votre-domaine>.org:443 \
        -servername pkarr.<votre-domaine>.org 2>/dev/null \
    | openssl x509 -pubkey -noout \
    | openssl pkey -pubin -outform DER \
    | openssl dgst -sha256 -binary \
    | openssl base64
# Copier la chaine 44 chars

# 3. Publier le nouveau pin
#    - Si deploy prive : push dans le ~/.sbfb/relay-pins.json
#      de chaque client SBFB concerne (file-watcher Phase C
#      reload sans restart).
#    - Si federe : ouvrir PR sur SBFB50/SBFB qui bumpe
#      relays.json defaults avec nouveau SPKI + release minor.
```

Template JSON entry :

```json
{
  "relay_url": "https://pkarr.<votre-domaine>.org",
  "spki_sha256": "<NEW-HASH-from-step-2>",
  "added_at": "2026-MM-DD",
  "source": "rotation"
}
```

Reference complete : [`RELAY_PIN_BOOTSTRAP.md`](RELAY_PIN_BOOTSTRAP.md).

---

## 5. Monitoring baseline (sans Prometheus)

Pour **1** relai single-instance pre-federation, les 4 commandes
suivantes suffisent :

```bash
# Logs live
journalctl -u pkarr-relay -f --since "1 hour ago"

# Erreurs passees
journalctl -u pkarr-relay --since today | grep -E "ERROR|WARN|5[0-9][0-9]"

# Disk cache
df -h /var/lib/pkarr-relay
du -sh /var/lib/pkarr-relay/cache

# Connections actives
sudo ss -tnH state established '( sport = :6881 )' | wc -l
```

Stack Prometheus/Grafana = **reportee S22+** (scope creep si
1 seul relai).

---

## 6. Mise a jour de l'image

### 6.1 Procedure manuelle (recommandee)

1. Verifier le changelog SBFB (release notes du tag v1.x.y).
2. Re-run `cosign verify` (§2) avec le nouveau tag.
3. Edit `/etc/systemd/system/pkarr-relay.service` → bumper la
   ligne `ghcr.io/sbfb50/pkarr-relay:v1.0` vers `v1.1`.
4. `sudo systemctl daemon-reload && sudo systemctl restart pkarr-relay`.
5. Smoke test §3.6.

### 6.2 Procedure semi-automatique (timer hebdomadaire)

Voir [`.planning/research/S19_phase_E_pkarr_relay_design.md §6.3`](../../.planning/research/S19_phase_E_pkarr_relay_design.md)
pour le script `pkarr-relay-update.sh` + `systemd.timer`
qui :

- `docker pull` au planning defini (ex Sun 03:00)
- `cosign verify` signature avant restart
- `systemctl restart` si image changee

Recommandation S19 : **manual**, au moins jusqu'a la premiere
release minor apres go-live v1.0.

---

## 7. Onboarding federation

Apres uptime stable ~7 jours et smoke tests green :

1. Ouvrir issue `github.com/SBFB50/SBFB` avec template :

   ```
   [federation] Nouveau relai pkarr public
   - ONG: <nom + lien>
   - URL relai: https://pkarr.<domaine>.org
   - Region/juridiction: <pays>
   - SPKI SHA256: <44 chars base64url>
   - Maintainer: <nom + cle GPG>
   - Uptime observe: <jours> depuis <date>
   ```

2. Review du mainteneur SBFB :
   - DNS resolves + cert valide ok
   - Smoke curl `GET /` ok
   - SPKI hash matche
   - Juridiction compatible mission anti-subpoena
3. Si review ok : PR mainteneur SBFB ajoute le relai dans
   `relays.json` defaults SBFB → landed dans la release minor
   suivante.
4. Les clients qui upgrade pickent automatiquement le relai
   dans leur quorum 2/3 (Phase A S19 wire).

---

## 8. Tear-down / migration

Si arret du relai :

```bash
sudo systemctl disable --now pkarr-relay.service
sudo docker rm -f pkarr-relay
sudo rm /etc/systemd/system/pkarr-relay.service
# Conserver /var/lib/pkarr-relay/cache pendant 7j (rebond possible)
```

Si le relai etait federe dans `relays.json`, ouvrir issue
`[federation] decommissionning pkarr.<domaine>.org` pour que
le mainteneur retire l'entry dans la release minor suivante.

---

## 9. References

- [Dockerfile + README image](../../docker/pkarr-relay/)
- [Workflow CI build-pkarr-image.yml](../../.github/workflows/build-pkarr-image.yml)
- [Smoke test pkarr-relay-healthcheck.sh](../../tests/ci-smoke/pkarr-relay-healthcheck.sh)
- [Design doc Phase E](../../.planning/research/S19_phase_E_pkarr_relay_design.md)
- [RELAY_PIN_BOOTSTRAP.md — SPKI Phase C](RELAY_PIN_BOOTSTRAP.md)
- [MIRROR_FALLBACK.md — go-live flip sequence](MIRROR_FALLBACK.md)
- [pkarr upstream github.com/pubky/pkarr](https://github.com/pubky/pkarr)
- [pkarr-relay crate lib.rs](https://lib.rs/crates/pkarr-relay)
- [Hetzner Cloud](https://www.hetzner.com/cloud)
- [Caddy automatic HTTPS](https://caddyserver.com/docs/automatic-https)
