# nexus-grid VPS Deployment

Scripts for provisioning and deploying nexus-grid bootstrap nodes.

## VPS Setup

### 1. Provision (run once per VPS)

```bash
ssh root@<ip> 'bash -s' < deploy/provision.sh
```

This creates the `nexus` user, directory structure at `/opt/nexus-grid/`,
nginx with SPA routing + API proxy, firewall rules (SSH + HTTP + UDP for
iroh QUIC), and installs systemd service templates.

### 2. Generate identity (run once per VPS)

```bash
ssh root@<ip> 'bash -s' < deploy/gen-identity.sh
```

Creates a persistent Ed25519 keypair at `/opt/nexus-grid/identity/`.
Back up the secret key — losing it means losing the node's DHT identity.

### 3. Deploy binaries

```bash
# Build release binaries first
./scripts/build-release.sh

# Deploy to each VPS
./deploy/deploy.sh --host <eu-ip> --key ~/.ssh/vps_key --role coordinator
./deploy/deploy.sh --host <us-ip> --key ~/.ssh/vps_key --role daemon
./deploy/deploy.sh --host <asia-ip> --key ~/.ssh/vps_key --role daemon
```

### 4. Deploy web shell

```bash
# Builds locally (npm ci + npm run build) then uploads to nginx root.
./deploy/deploy.sh --host <eu-ip> --key ~/.ssh/vps_key --role web

# Or use the standalone script:
./deploy/deploy-web.sh --host <eu-ip> --key ~/.ssh/vps_key
```

### 5. Enable services

```bash
ssh root@<ip> 'systemctl enable --now nexus-daemon'
# On the EU VPS only:
ssh root@<ip> 'systemctl enable --now nexus-coordinator'
```

### 6. Create curator list (EU VPS only)

```bash
ssh nexus@<eu-ip> 'bash -s' < deploy/create-curator-list.sh
```

## Configuration Templates

| File | Purpose | Deploy to |
|---|---|---|
| `config.toml.example` | Shell daemon config | `/opt/nexus-grid/shell-daemon/config.toml` |
| `coordinator.toml.example` | Coordinator config | `/opt/nexus-grid/data/coordinator.toml` |
| `nginx-nexus.conf` | nginx site config | `/etc/nginx/sites-available/nexus` |

## VPS Fleet

| Region | Provider | Plan | Role |
|---|---|---|---|
| EU (Falkenstein) | Hetzner CX32 | 3 vCPU, 8 GB, NVMe | DHT + coordinator + apps + web |
| US (Chicago) | Vultr HF | 2 vCPU, 4 GB, NVMe | DHT bootstrap |
| Asia (Tokyo) | Vultr HF | 2 vCPU, 4 GB, NVMe | DHT bootstrap |

## GitHub Actions Deploy

The `.github/workflows/deploy.yml` workflow automates deployment via SSH.
Required secrets: `VPS_EU_HOST`, `VPS_EU_SSH_KEY`, `VPS_US_HOST`,
`VPS_US_SSH_KEY`, `VPS_ASIA_HOST`, `VPS_ASIA_SSH_KEY`.
