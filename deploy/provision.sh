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
echo "  [1/8] System update..."
apt-get update -qq
apt-get upgrade -y -qq

# ── Create nexus user ───────────────────────────────────────
echo "  [2/8] Creating nexus user..."
if ! id nexus &>/dev/null; then
  useradd --system --create-home --shell /bin/bash nexus
fi

# ── Directory structure ─────────────────────────────────────
echo "  [3/8] Creating directories..."
mkdir -p /opt/nexus-grid/{bin,identity,data,logs}
chown -R nexus:nexus /opt/nexus-grid

# ── Python 3.13 + uv (for coordinator VPS) ─────────────────
echo "  [4/8] Installing Python + uv..."
apt-get install -y -qq python3.13 python3.13-venv python3-pip curl
if ! command -v uv &>/dev/null; then
  curl -LsSf https://astral.sh/uv/install.sh | sh
fi

# ── nginx (web shell) ──────────────────────────────────────
echo "  [5/8] Installing nginx..."
apt-get install -y -qq nginx
cp /dev/stdin /etc/nginx/sites-available/nexus << 'NGINX_CONF'
# Managed by provision.sh — edit deploy/nginx-nexus.conf upstream.
server {
    listen 80;
    server_name _;
    root /opt/nexus-grid/web;
    index index.html;
    location / {
        try_files $uri $uri/ /index.html;
    }
    location /api/ {
        proxy_pass http://127.0.0.1:8000/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 30s;
    }
    location /daemon/ {
        proxy_pass http://127.0.0.1:7000/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_read_timeout 30s;
    }
    location ~ /\. {
        deny all;
    }
}
NGINX_CONF
ln -sf /etc/nginx/sites-available/nexus /etc/nginx/sites-enabled/nexus
rm -f /etc/nginx/sites-enabled/default
mkdir -p /opt/nexus-grid/web
chown nexus:nexus /opt/nexus-grid/web
systemctl enable nginx
systemctl reload nginx || systemctl start nginx

# ── Firewall ────────────────────────────────────────────────
echo "  [6/8] Configuring firewall..."
apt-get install -y -qq ufw
ufw --force reset
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
# HTTP for nginx (web shell).
ufw allow 80/tcp
# iroh QUIC — UDP on all ports (iroh uses ephemeral ports for QUIC)
ufw allow proto udp from any to any
ufw --force enable

# ── Systemd services ───────────────────────────────────────
echo "  [7/8] Installing systemd services..."

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

# ── Summary ────────────────────────────────────────────────
echo "  [8/8] Final checks..."
echo ""
echo "==> Provisioning complete."
echo "    nginx:  $(systemctl is-active nginx)"
echo "    ufw:    $(ufw status | head -1)"
echo ""
echo "    Next steps:"
echo "    1. deploy/gen-identity.sh        (generate node identity)"
echo "    2. deploy/deploy.sh --role coordinator  (upload binaries)"
echo "    3. deploy/deploy-web.sh          (build + upload web shell)"
echo "    4. systemctl enable --now nexus-daemon nexus-coordinator"
