# nexus-grid VPS Deployment

Scripts for provisioning and deploying nexus-grid bootstrap nodes.

## VPS Setup

### 1. Provision (run once per VPS)

```bash
ssh root@<ip> 'bash -s' < deploy/provision.sh
```

This creates the `nexus` user, directory structure at `/opt/nexus-grid/`,
firewall rules (SSH + UDP for iroh QUIC), and installs systemd service
templates.

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

### 4. Enable services

```bash
ssh root@<ip> 'systemctl enable --now nexus-daemon'
# On the EU VPS only:
ssh root@<ip> 'systemctl enable --now nexus-coordinator'
```

## VPS Fleet

| Region | Provider | Plan | Role |
|---|---|---|---|
| EU (Falkenstein) | Hetzner CX32 | 3 vCPU, 8 GB, NVMe | DHT + coordinator + apps |
| US (Chicago) | Vultr HF | 2 vCPU, 4 GB, NVMe | DHT bootstrap |
| Asia (Tokyo) | Vultr HF | 2 vCPU, 4 GB, NVMe | DHT bootstrap |

## GitHub Actions Deploy

The `.github/workflows/deploy.yml` workflow automates deployment via SSH.
Required secrets: `VPS_EU_HOST`, `VPS_EU_SSH_KEY`, `VPS_US_HOST`,
`VPS_US_SSH_KEY`, `VPS_ASIA_HOST`, `VPS_ASIA_SSH_KEY`.
