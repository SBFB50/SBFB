#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# nexus-grid VPS provisioning script.
# Run on a fresh Ubuntu 24.04 VPS via SSH:
#   ssh root@<ip> 'bash -s' < deploy/provision.sh
#
# Creates the nexus user, directories, firewall rules, and installs
# systemd service templates. Does NOT deploy binaries — use deploy.sh
# for that.

set -euo pipefail

echo "==> nexus-grid VPS provisioning"

# ── System update ───────────────────────────────────────────
echo "  [1/6] System update..."
apt-get update -qq
apt-get upgrade -y -qq

# ── Create nexus user ───────────────────────────────────────
echo "  [2/6] Creating nexus user..."
if ! id nexus &>/dev/null; then
  useradd --system --create-home --shell /bin/bash nexus
fi

# ── Directory structure ─────────────────────────────────────
echo "  [3/6] Creating directories..."
mkdir -p /opt/nexus-grid/{bin,identity,data,logs}
chown -R nexus:nexus /opt/nexus-grid

# ── Python 3.13 + uv (for coordinator VPS) ─────────────────
echo "  [4/6] Installing Python + uv..."
apt-get install -y -qq python3.13 python3.13-venv python3-pip curl
if ! command -v uv &>/dev/null; then
  curl -LsSf https://astral.sh/uv/install.sh | sh
fi

# ── Firewall ────────────────────────────────────────────────
echo "  [5/6] Configuring firewall..."
apt-get install -y -qq ufw
ufw --force reset
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
# iroh QUIC — UDP on all ports (iroh uses ephemeral ports for QUIC)
ufw allow proto udp from any to any
# Coordinator HTTP (only on EU VPS, loopback by default)
# ufw allow 8765/tcp  # uncomment if coordinator needs external access
ufw --force enable

# ── Systemd services ───────────────────────────────────────
echo "  [6/6] Installing systemd services..."

cat > /etc/systemd/system/nexus-daemon.service << 'UNIT'
[Unit]
Description=nexus-grid shell daemon (DHT bootstrap)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=nexus
ExecStart=/opt/nexus-grid/bin/nexus-shell-daemon start
WorkingDirectory=/opt/nexus-grid
Restart=always
RestartSec=5
WatchdogSec=30
Environment=NEXUS_GRID_ROOT=/opt/nexus-grid/data
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
UNIT

cat > /etc/systemd/system/nexus-coordinator.service << 'UNIT'
[Unit]
Description=nexus-grid coordinator (official apps)
After=network-online.target nexus-daemon.service
Wants=network-online.target

[Service]
Type=simple
User=nexus
ExecStart=/opt/nexus-grid/bin/nexus-coordinator-start.sh
WorkingDirectory=/opt/nexus-grid
Restart=always
RestartSec=5
Environment=NEXUS_GRID_ROOT=/opt/nexus-grid/data

[Install]
WantedBy=multi-user.target
UNIT

cat > /opt/nexus-grid/bin/nexus-coordinator-start.sh << 'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
cd /opt/nexus-grid/data

# Initialize if first boot
if [ ! -f coordinator.toml ]; then
  /opt/nexus-grid/bin/nexus-coordinator init
fi

exec /opt/nexus-grid/bin/nexus-coordinator start
SCRIPT
chmod +x /opt/nexus-grid/bin/nexus-coordinator-start.sh
chown nexus:nexus /opt/nexus-grid/bin/nexus-coordinator-start.sh

systemctl daemon-reload
echo ""
echo "==> Provisioning complete."
echo "    Next: run deploy/deploy.sh to upload binaries."
echo "    Then: systemctl enable --now nexus-daemon"
