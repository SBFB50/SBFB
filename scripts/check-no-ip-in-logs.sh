#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Chat plan Phase I — "discreet by default": count the IP addresses a
# daemon wrote to its logs.
#
# Usage: check-no-ip-in-logs.sh [--json OUT] PATH...
#   PATH  a log file, a directory searched for daemon.log* files, or `-`
#         for stdin (journald: `journalctl -u nexus-shell-daemon | ... -`).
#
# Every candidate is validated by Python's `ipaddress` (a timestamp such as
# 14:10:54 or a version string is not an address). Loopback and unspecified
# addresses (127.0.0.0/8, ::1, 0.0.0.0, ::) are ignored: a daemon names its
# own loopback API. Everything else is counted, split into
#   global   — anything else (Internet, CGNAT 100.64/10, documentation
#              ranges): BLOCK
#   private  — RFC 1918, IPv4 link-local, IPv6 unique-local: reported, not
#              blocking (a LAN topology hint, documented in THREAT_MODEL)
#
# The addresses themselves are NEVER printed — this output may be pasted
# or committed; only counts and file:line locations are.
#
# Exit 0 = PASS (no global address), 1 = BLOCK, 2 = usage / no log found.
# --json OUT writes {"status","files","global","private","locations"}.
set -euo pipefail

out=""
if [ "${1:-}" = "--json" ]; then
  out="${2:?--json needs a path}"
  shift 2
fi
[ "$#" -ge 1 ] || { echo "usage: $0 [--json OUT] PATH..." >&2; exit 2; }

files=()
for p in "$@"; do
  if [ "$p" = "-" ]; then
    # stdin, e.g. `journalctl -u nexus-shell-daemon | ... -` (journald is
    # where the terminal layer of a systemd unit ends up).
    stdin_copy="$(mktemp)"
    trap 'rm -f "$stdin_copy"' EXIT
    cat > "$stdin_copy"
    files+=("$stdin_copy")
  elif [ -d "$p" ]; then
    while IFS= read -r f; do files+=("$f"); done < <(find "$p" -type f -name 'daemon.log*' | sort)
  elif [ -f "$p" ]; then
    files+=("$p")
  fi
done
[ "${#files[@]}" -ge 1 ] || { echo "no log file found under: $*" >&2; exit 2; }

PY="$(command -v python3 || command -v python || true)"
[ -n "$PY" ] || { echo "python3 required" >&2; exit 2; }

"$PY" -I - "$out" "${files[@]}" <<'PYEOF'
import ipaddress, json, re, sys

out, files = sys.argv[1], sys.argv[2:]
# Candidates only; ipaddress decides. IPv6: hex groups with at least two
# colons, optionally bracketed / with a zone; IPv4: dotted quad.
V4 = re.compile(r'(?<![\d.])(\d{1,3}(?:\.\d{1,3}){3})(?!\d|\.\d)')
V6 = re.compile(r'(?<![0-9A-Za-z:.])([0-9A-Fa-f]{0,4}(?::[0-9A-Fa-f]{0,4}){2,7})(?:%\w+)?(?![0-9A-Za-z:.])')
# Same allow-list as nexus_core_rs::discovery::discreet_addr: anything else
# that is not loopback / unspecified counts as global.
PRIVATE = [ipaddress.ip_network(n) for n in (
    '10.0.0.0/8', '172.16.0.0/12', '192.168.0.0/16', '169.254.0.0/16',
    'fc00::/7')]

def classify(text):
    try:
        ip = ipaddress.ip_address(text)
    except ValueError:
        return None
    if ip.version == 6 and ip.ipv4_mapped:
        ip = ip.ipv4_mapped
    if ip.is_loopback or ip.is_unspecified:
        return None
    if any(ip in net for net in PRIVATE if net.version == ip.version):
        return 'private'
    return 'global'

counts = {'global': 0, 'private': 0}
locations = []
for path in files:
    with open(path, encoding='utf-8', errors='replace') as fh:
        for n, line in enumerate(fh, 1):
            for m in list(V4.finditer(line)) + list(V6.finditer(line)):
                kind = classify(m.group(1))
                if kind:
                    counts[kind] += 1
                    locations.append({'file': path, 'line': n, 'kind': kind})

status = 'BLOCK' if counts['global'] else 'PASS'
report = {'status': status, 'files': files, 'global': counts['global'],
          'private': counts['private'], 'locations': locations[:200]}
if out:
    with open(out, 'w', encoding='utf-8') as fh:
        json.dump(report, fh, indent=2)
print(f"{status}: {counts['global']} global, {counts['private']} private address(es) "
      f"in {len(files)} file(s)")
for loc in locations[:20]:
    print(f"  {loc['kind']:7} {loc['file']}:{loc['line']}")
sys.exit(1 if counts['global'] else 0)
PYEOF
