#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 12 Phase E (T33) — HTTPS/TLS provisioning via certbot.
#
# Run after provision.sh has set up nginx + HTTP. This script:
#   1. Installs certbot + the nginx plugin
#   2. Obtains a Let's Encrypt certificate for the given domain
#   3. Certbot auto-configures nginx for HTTPS + redirect
#
# Usage:
#   sudo bash deploy/provision-tls.sh <domain> [<email>]
#
# Example:
#   sudo bash deploy/provision-tls.sh sbfb.example.com admin@example.com

set -euo pipefail

DOMAIN="${1:?Usage: provision-tls.sh <domain> [<email>]}"
EMAIL="${2:-}"

echo "=== SBFB TLS provisioning ==="
echo "  domain: $DOMAIN"

# 1. Install certbot
echo "  [1/3] Installing certbot..."
apt-get update -qq
apt-get install -y -qq certbot python3-certbot-nginx

# 2. Obtain certificate
echo "  [2/3] Obtaining Let's Encrypt certificate..."
CERTBOT_ARGS=(
    --nginx
    -d "$DOMAIN"
    --non-interactive
    --agree-tos
    --redirect
)
if [ -n "$EMAIL" ]; then
    CERTBOT_ARGS+=(--email "$EMAIL")
else
    CERTBOT_ARGS+=(--register-unsafely-without-email)
fi
certbot "${CERTBOT_ARGS[@]}"

# 3. Verify
echo "  [3/3] Verifying..."
nginx -t
systemctl reload nginx

echo ""
echo "=== TLS provisioning complete ==="
echo "  https://$DOMAIN should now be live."
echo "  Certbot auto-renewal is enabled via systemd timer."
