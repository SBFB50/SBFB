#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# SC2329: cleanup (trap) and pred_* (via wait_until) are invoked indirectly.
# SC2029: client-side expansion in ssh commands is intended — every value
#         interpolated there is validated (aliases, loopback URLs, ids).
# shellcheck disable=SC2329,SC2029
#
# Chat v1 Phase G — T2 live acceptance PC + VPS anchor (ADR_CHAT_V1,
# routes /app/{name}/chat/* in crates/nexus-shell-daemon/src/chat_api.rs,
# offline mailbox ALPN `sbfb/mailbox/0` in mailbox_protocol.rs, config
# `[mailbox]`). MANUAL acceptance: it needs the operator's PC, the real VPS
# anchor over the WAN and, for the group check, a third host. Never run in CI.
#
# === Safety (read this first) ===
#   - DRY-RUN IS THE DEFAULT. Without `--live` the script touches nothing:
#     no ssh, no process spawn, no network. It prints the exact plan,
#     checks the local toolchain, runs the analyzers' self-tests on
#     synthetic fixtures, and writes a NOT-RUN artefact to a SEPARATE file
#     (`.chat_t2_dry_run.json`) so it never clobbers a live result.
#   - `--live` asks for an explicit confirmation BEFORE the first remote
#     action: type the comma-joined ssh aliases it will reach (e.g. `vps`
#     or `vps,mac`). Non-interactive runs must set CHAT_T2_CONFIRM to that
#     exact string; anything else aborts with NOT-RUN (exit 2).
#   - No secret is printed or written: loopback tokens live in shell
#     variables only and reach the remote curl through ssh STDIN (never a
#     remote command line, so never a sudo/journal log line). Known
#     plaintexts are also passed through stdin to the remote grep, so the
#     leak scan cannot plant its own needle in the anchor's journal. The
#     artefact carries node ids (public) and COUNTS of IPs checked, never
#     the IPs themselves.
#   - Remote writes are limited to the chat API of an app-scoped name
#     (CHAT_APP, default `t2-chat-acceptance`, never a real user app) and a
#     short-lived mktemp control dir that is removed in the same call. The
#     script never restarts, reconfigures or deletes anything on the
#     anchor. Leftovers: the anchor keeps the test conversations in its
#     chat dir for that app (`$VPS_ROOT/shell-daemon/chat/<blake3 prefix>`)
#     and the mailbox registration until its TTL — remove by hand if wanted.
#
# === Topology ===
#   A  local sender     hermetic nexus-shell-daemon started by this script
#                       (fresh NEXUS_GRID_ROOT under .chat_t2/run-*/a,
#                       [mailbox] servers = [anchor node id]).
#   R  local recipient  second hermetic daemon on the PC (.../r) — same
#                       mailbox config; stopped and restarted by the
#                       offline check.
#   V  VPS anchor       the operator's systemd daemon, reached over ssh
#                       (VPS_SSH). Must run with `[mailbox] serve = true`.
#   T  third host       optional (THIRD_SSH + THIRD_DAEMON), group check.
#
# === Checks (each gets its own verdict) ===
#   one_to_one       A <-> V: create, invite, accept, both see 2 members,
#                    A->V and V->A messages decrypted on the other side.
#   group_of_3       A + V + T: each member sees the 3 per-member markers.
#                    No third host => RIG-ABSENT for THIS check only.
#   offline_mailbox  A -> R through V's mailbox: both opt in, deposit kept;
#                    R stopped; A sends a canary; A stopped; R restarted
#                    and must read the canary (A provably dead, V not a
#                    member => the anchor's ciphertext replica is the only
#                    possible source).
#   leak_plaintext   no known plaintext (every marker + canary) anywhere in
#                    the anchor data dir (`grep -rlaF`, binary as text) nor
#                    in its unit journal since the run started. A positive
#                    control (needle planted in a remote mktemp dir) proves
#                    the scan pipeline itself detects.
#   leak_registry    the anchor's `mailbox.json` holds only
#                    {docs: {namespace: {64-hex node id: int expiry}}}:
#                    our conversation is registered with depositors among
#                    {A, R}, no IP-like string, no PC IP (the one the anchor
#                    sees in $SSH_CLIENT + the PC's interface addresses).
#
# === Machine-readable contract (T2 gate, closed vocabulary) ===
# Every exit writes a JSON artefact (default
# `scripts/acceptance/.chat_t2_last_result.json`, override CHAT_T2_ARTIFACT)
# AND exits with a status-specific code:
#   PASS       exit 0 — every check PASS.
#   BLOCK      exit 1 — at least one check BLOCK{diagnosis} (product gap).
#   NOT-RUN    exit 2 — live mode not confirmed (dry-run itself exits 0).
#   RIG-ABSENT exit 3 — no BLOCK, but at least one check could not run
#                       (ssh down, binary missing, anchor not serving the
#                       mailbox, no third host, unreadable journal...).
# Artefact shape:
#   {"suite","kind","date","mode","run_id","status","diagnosis","contract",
#    "rig":{...},"checks":{"<name>":{"status","stage","delay_s","diagnosis"}}}
#
# Usage (from the PC, repo root):
#   bash scripts/acceptance/chat_live_pc_vps.sh             # dry-run (default)
#   bash scripts/acceptance/chat_live_pc_vps.sh --live      # asks to confirm
#   THIRD_SSH=mac THIRD_DAEMON=http://127.0.0.1:7654 \
#     bash scripts/acceptance/chat_live_pc_vps.sh --live
#
# Env (or scripts/acceptance/rig.local.env, gitignored; explicit env wins):
#   VPS_SSH        ssh alias of the anchor (default `vps`, cf. ~/.ssh/config)
#   VPS_DAEMON     anchor loopback URL seen over ssh (default http://127.0.0.1:7000)
#   VPS_ROOT       anchor NEXUS_GRID_ROOT (default /var/lib/nexus-grid)
#   VPS_UNIT       anchor systemd unit (default nexus-shell-daemon)
#   VPS_SUDO       prefix for privileged reads on the anchor (default empty,
#                  the alias logs in as root; e.g. `sudo -n` otherwise)
#   THIRD_SSH      ssh alias of the third host (default empty => group RIG-ABSENT)
#   THIRD_DAEMON   third host's daemon loopback URL (required with THIRD_SSH)
#   DAEMON_BIN     local nexus-shell-daemon binary (default target/release/...)
#   CHAT_APP       app name the chat state is scoped to (default t2-chat-acceptance)
#   LOCAL_PORT_A / LOCAL_PORT_R   loopback ports of A / R (default 7791 / 7792)
#   BOOT_TIMEOUT_SECS (90)  JOIN_TIMEOUT_SECS (120)  MSG_TIMEOUT_SECS (90)
#   MAILBOX_KEPT_TIMEOUT_SECS (180)  OFFLINE_SETTLE_SECS (90)
#   OFFLINE_DELIVERY_TIMEOUT_SECS (240)  POLL_SECS (3)
#   CHAT_T2_ARTIFACT   artefact path        CHAT_T2_RIG_DIR  local run dir
#   CHAT_T2_CONFIRM    non-interactive confirmation string
#   RIG_ENV            rig config path (default scripts/acceptance/rig.local.env)

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

MODE="dry-run"
for arg in "$@"; do
  case "$arg" in
    --live) MODE="live" ;;
    --dry-run) MODE="dry-run" ;;
    -h|--help) sed -n '/^# Chat v1 Phase G/,/^set -uo pipefail$/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) printf '[chat-t2] unknown argument: %s (use --live, --dry-run, --help)\n' "$arg" >&2; exit 2 ;;
  esac
done

# --- Rig config as data ---------------------------------------------------
RIG_ENV="${RIG_ENV:-$SCRIPT_DIR/rig.local.env}"
if [ -f "$RIG_ENV" ]; then
  # shellcheck disable=SC1090
  . "$RIG_ENV"
fi

VPS_SSH="${VPS_SSH:-vps}"
VPS_DAEMON="${VPS_DAEMON:-http://127.0.0.1:7000}"
VPS_ROOT="${VPS_ROOT:-/var/lib/nexus-grid}"
VPS_UNIT="${VPS_UNIT:-nexus-shell-daemon}"
VPS_SUDO="${VPS_SUDO:-}"
THIRD_SSH="${THIRD_SSH:-}"
THIRD_DAEMON="${THIRD_DAEMON:-}"
CHAT_APP="${CHAT_APP:-t2-chat-acceptance}"
LOCAL_PORT_A="${LOCAL_PORT_A:-7791}"
LOCAL_PORT_R="${LOCAL_PORT_R:-7792}"
BOOT_TIMEOUT_SECS="${BOOT_TIMEOUT_SECS:-90}"
JOIN_TIMEOUT_SECS="${JOIN_TIMEOUT_SECS:-120}"
MSG_TIMEOUT_SECS="${MSG_TIMEOUT_SECS:-90}"
MAILBOX_KEPT_TIMEOUT_SECS="${MAILBOX_KEPT_TIMEOUT_SECS:-180}"
OFFLINE_SETTLE_SECS="${OFFLINE_SETTLE_SECS:-90}"
OFFLINE_DELIVERY_TIMEOUT_SECS="${OFFLINE_DELIVERY_TIMEOUT_SECS:-240}"
POLL_SECS="${POLL_SECS:-3}"
CHAT_T2_RIG_DIR="${CHAT_T2_RIG_DIR:-$SCRIPT_DIR/.chat_t2}"
if [ "$MODE" = "live" ]; then
  CHAT_T2_ARTIFACT="${CHAT_T2_ARTIFACT:-$SCRIPT_DIR/.chat_t2_last_result.json}"
else
  CHAT_T2_ARTIFACT="${CHAT_T2_DRY_ARTIFACT:-$SCRIPT_DIR/.chat_t2_dry_run.json}"
fi
if [ -z "${DAEMON_BIN:-}" ]; then
  DAEMON_BIN="$REPO_ROOT/target/release/nexus-shell-daemon"
  [ -x "$DAEMON_BIN.exe" ] && DAEMON_BIN="$DAEMON_BIN.exe"
fi

# Mailbox resync period on the client (chat_api.rs MAILBOX_RESYNC_SECS):
# a settle window shorter than this cannot tell a live-sync gap from a
# delivery failure, and the BLOCK diagnosis says so.
MAILBOX_RESYNC_SECS=600
CHECK_NAMES="one_to_one group_of_3 offline_mailbox leak_plaintext leak_registry"
SSH_OPTS=(-o BatchMode=yes -o ConnectTimeout=15)
START_EPOCH="$(date +%s)"
RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)-$(od -An -N4 -tx1 /dev/urandom 2>/dev/null | tr -d ' \n')"

log() { printf '[chat-t2] %s\n' "$*"; }
warn() { printf '[chat-t2][warn] %s\n' "$*" >&2; }

# --- Temp state + cleanup -------------------------------------------------
WORK="$(mktemp -d 2>/dev/null || mktemp -d -t chat_t2)"
PYLIB="$WORK/chat_t2_lib.py"
CHECKS_JSONL="$WORK/checks.jsonl"
: >"$CHECKS_JSONL"
LOCAL_PIDS=""
cleanup() {
  local p
  for p in $LOCAL_PIDS; do
    kill -INT "$p" 2>/dev/null || true
  done
  sleep 1
  for p in $LOCAL_PIDS; do
    kill -0 "$p" 2>/dev/null && kill -KILL "$p" 2>/dev/null
  done
  rm -rf "$WORK"
}
trap cleanup EXIT

# Native path for Windows binaries (python, the daemon) under Git Bash.
native_path() {
  if command -v cygpath >/dev/null 2>&1; then cygpath -m "$1"; else printf '%s' "$1"; fi
}

# --- Python helper (JSON in/out, analyzers, artefact) ----------------------
cat >"$PYLIB" <<'PY'
import json, os, re, sys, datetime
try:
    sys.stdout.reconfigure(newline="\n")
except Exception:
    pass

HEX64 = re.compile(r"^[0-9a-f]{64}$")
IPV4 = re.compile(r"(?<![0-9.])(?:\d{1,3}\.){3}\d{1,3}(?![0-9.])")

def out(s):
    sys.stdout.write(s)

def load_stdin():
    try:
        return json.loads(sys.stdin.read() or "null")
    except Exception:
        return None

def contains_ip(text, ip):
    if ":" in ip:
        return ip.lower() in text.lower()
    return re.search(r"(?<![0-9.])" + re.escape(ip) + r"(?![0-9.])", text) is not None

def analyze_registry(raw, conv, nodes, ips):
    """Return (status, diagnosis) for the anchor's mailbox.json text."""
    try:
        reg = json.loads(raw)
    except Exception as e:
        return "BLOCK", "mailbox.json is not valid JSON (%s)" % e
    if not isinstance(reg, dict) or set(reg.keys()) != {"docs"}:
        keys = sorted(reg.keys()) if isinstance(reg, dict) else type(reg).__name__
        return "BLOCK", "unexpected top-level shape %s (expected exactly {docs})" % keys
    docs = reg["docs"]
    if not isinstance(docs, dict):
        return "BLOCK", "docs is not an object"
    for ns, deps in docs.items():
        if not isinstance(deps, dict):
            return "BLOCK", "namespace entry is not an object (%s...)" % ns[:12]
        for node, exp in deps.items():
            if not HEX64.match(node):
                return "BLOCK", "depositor key is not a 64-hex node id (%r...)" % node[:16]
            if not isinstance(exp, int) or isinstance(exp, bool):
                return "BLOCK", "depositor value is not an integer expiry"
    m = IPV4.search(raw)
    if m:
        return "BLOCK", "an IPv4-like string is present in the registry"
    hits = [i for i in ips if i and contains_ip(raw, i)]
    if hits:
        return "BLOCK", "%d PC IP address(es) found in the registry" % len(hits)
    if conv not in docs:
        return "BLOCK", "the offline conversation is not registered (deposit reported kept but absent from mailbox.json)"
    deps = set(docs[conv].keys())
    known = set(n for n in nodes if n)
    if not deps:
        return "BLOCK", "the offline conversation has no depositor"
    if not deps <= known:
        return "BLOCK", "foreign depositor(s) on our conversation: %d not among the test nodes" % len(deps - known)
    return "PASS", "registry = {docs:{ns:{node_id:expiry}}} only; our conversation held for %d test node(s); 0 IP-like string; %d PC IP(s) checked absent; %d namespace(s) total" % (len(deps), len(ips), len(docs))

def cmd_get(args):
    v = load_stdin()
    for k in args[0].split("."):
        if isinstance(v, dict):
            v = v.get(k)
        else:
            v = None
    if v is None:
        return out("")
    out(v if isinstance(v, str) else json.dumps(v))

def cmd_json(args):
    d = {}
    for a in args:
        k, _, val = a.partition("=")
        if k.endswith(":b"):
            d[k[:-2]] = val == "true"
        else:
            d[k] = val
    out(json.dumps(d))

def cmd_has_text(args):
    msgs = load_stdin()
    ok = isinstance(msgs, list) and any(isinstance(m, dict) and m.get("body") == args[0] for m in msgs)
    out("1" if ok else "0")

def cmd_members(args):
    convs = load_stdin()
    n = 0
    if isinstance(convs, list):
        for c in convs:
            if isinstance(c, dict) and c.get("id") == args[0]:
                n = len(c.get("members") or [])
    out(str(n))

def cmd_mbx_status(args):
    v = load_stdin()
    st = ""
    if isinstance(v, dict):
        for c in v.get("conversations") or []:
            if isinstance(c, dict) and c.get("id") == args[0]:
                st = str(c.get("status", ""))
    out(st)

def cmd_mailbox_serve(args):
    import tomllib
    text = sys.stdin.read()
    if not text.strip():
        # cat failed (missing file, no permission): never read as "false".
        return out("unreadable")
    try:
        cfg = tomllib.loads(text)
    except Exception:
        return out("unparseable")
    out("true" if (cfg.get("mailbox") or {}).get("serve") is True else "false")

def cmd_registry(args):
    path, conv, nodes, ips = args
    with open(path, "r", encoding="utf-8", errors="replace") as f:
        raw = f.read()
    st, diag = analyze_registry(raw, conv, nodes.split(","), [i for i in ips.split(",") if i])
    out(st + "\t" + diag)

def cmd_record(args):
    name, status, stage, delay, diag = args
    try:
        d = int(delay)
    except Exception:
        d = None
    out(json.dumps({"name": name, "status": status, "stage": stage, "delay_s": d, "diagnosis": diag}) + "\n")

def cmd_aggregate(args):
    jsonl, artefact, mode, names = args[0], args[1], args[2], args[3].split()
    checks = {}
    with open(jsonl, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                r = json.loads(line)
                checks[r.pop("name")] = r
    for n in names:
        checks.setdefault(n, {"status": "NOT-RUN", "stage": "not-reached", "delay_s": None, "diagnosis": "check not reached in this run"})
    ordered = {n: checks[n] for n in names}
    sts = [c["status"] for c in ordered.values()]
    env = os.environ
    if mode != "live":
        status = "NOT-RUN"
        diag = "dry-run: no remote action, no process spawned; plan printed and analyzers self-tested (%s)" % env.get("CT_DRY_NOTE", "")
    elif env.get("CT_ABORT"):
        status = "NOT-RUN"
        diag = env["CT_ABORT"]
    elif "BLOCK" in sts:
        status = "BLOCK"
        diag = "; ".join("%s BLOCK at %s: %s" % (n, c["stage"], c["diagnosis"]) for n, c in ordered.items() if c["status"] == "BLOCK")
    elif all(s == "PASS" for s in sts):
        status = "PASS"
        diag = "all %d checks PASS" % len(sts)
    else:
        status = "RIG-ABSENT"
        diag = "; ".join("%s %s: %s" % (n, c["status"], c["diagnosis"]) for n, c in ordered.items() if c["status"] != "PASS")
    doc = {
        "suite": "chat-v1-t2-pc-vps",
        "kind": "aggregate",
        "date": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "mode": mode,
        "run_id": env.get("CT_RUN_ID", ""),
        "status": status,
        "diagnosis": diag,
        "contract": "closed vocabulary PASS / BLOCK{diagnosis} / RIG-ABSENT / NOT-RUN per check; aggregate PASS only if every check PASS, BLOCK if any BLOCK, else RIG-ABSENT. No token, no IP value is ever written (counts only).",
        "rig": {
            "anchor_ssh_alias": env.get("CT_VPS_SSH", ""),
            "anchor_daemon": env.get("CT_VPS_DAEMON", ""),
            "anchor_node_id": env.get("CT_ANCHOR_ID", "") or None,
            "third_ssh_alias": env.get("CT_THIRD_SSH", "") or None,
            "chat_app": env.get("CT_CHAT_APP", ""),
            "local_nodes": {"sender": env.get("CT_NODE_A", "") or None, "recipient": env.get("CT_NODE_R", "") or None},
            "offline_settle_s": int(env.get("CT_SETTLE", "0") or 0),
        },
        "checks": ordered,
    }
    with open(artefact, "w", encoding="utf-8", newline="\n") as f:
        f.write(json.dumps(doc, indent=2) + "\n")
    out(status)

def cmd_selftest(args):
    a, r = "a" * 64, "b" * 64
    conv = "c0ffee" * 4
    good = json.dumps({"docs": {conv: {a: 1900000000, r: 1900000001}, "other": {"e" * 64: 5}}})
    cases = [
        ("good registry", good, [], "PASS"),
        ("PC IP present", good.replace("other", "203.0.113.7"), [], "BLOCK"),
        ("PC IP absent", json.dumps({"docs": {conv: {a: 1}}}), ["198.51.100.4", "2001:db8::1"], "PASS"),
        ("PC IPv4 in key", json.dumps({"docs": {conv: {a: 1}, "x198.51.100.4": {}}}), ["198.51.100.4"], "BLOCK"),
        # IPv6 is NOT covered by the generic IPv4 pattern: this case alone
        # proves the PC-IP list path is live (not shadowed by the regex).
        ("PC IPv6 in key", json.dumps({"docs": {conv: {a: 1}, "2001:db8::1": {}}}), ["2001:db8::1"], "BLOCK"),
        ("extra top field", json.dumps({"docs": {conv: {a: 1}}, "addrs": []}), [], "BLOCK"),
        ("non-hex node", json.dumps({"docs": {conv: {"node-1": 1}}}), [], "BLOCK"),
        ("string expiry", json.dumps({"docs": {conv: {a: "1"}}}), [], "BLOCK"),
        ("conv missing", json.dumps({"docs": {"other": {a: 1}}}), [], "BLOCK"),
        ("foreign depositor", json.dumps({"docs": {conv: {"f" * 64: 1}}}), [], "BLOCK"),
        ("not json", "{docs:", [], "BLOCK"),
    ]
    bad = 0
    for label, raw, ips, want in cases:
        got, diag = analyze_registry(raw, conv, [a, r], ips)
        flag = "ok " if got == want else "BAD"
        if got != want:
            bad += 1
        out("[selftest] %s %-18s want=%-5s got=%-5s %s\n" % (flag, label, want, got, diag))
    out("[selftest] %d failure(s)\n" % bad)
    sys.exit(1 if bad else 0)

CMDS = {k[4:]: v for k, v in globals().items() if k.startswith("cmd_")}
if __name__ == "__main__":
    CMDS[sys.argv[1]](sys.argv[2:])
PY

py() { python3 "$(native_path "$PYLIB")" "$@"; }

record_check() {
  # $1=name $2=status $3=stage $4=delay(int or empty) $5=diagnosis
  py record "$1" "$2" "$3" "${4:-}" "$5" >>"$CHECKS_JSONL"
  case "$2" in
    PASS) log "check $1: PASS ($5)" ;;
    *) warn "check $1: $2 at $3 — $5" ;;
  esac
}

finish() {
  # Writes the aggregate artefact and exits with the status code.
  local status
  status="$(CT_RUN_ID="$RUN_ID" CT_VPS_SSH="$VPS_SSH" CT_VPS_DAEMON="$VPS_DAEMON" \
    CT_ANCHOR_ID="${ANCHOR_ID:-}" CT_THIRD_SSH="$THIRD_SSH" CT_CHAT_APP="$CHAT_APP" \
    CT_NODE_A="${NODE_A:-}" CT_NODE_R="${NODE_R:-}" CT_SETTLE="$OFFLINE_SETTLE_SECS" \
    CT_ABORT="${ABORT_REASON:-}" CT_DRY_NOTE="${DRY_NOTE:-}" \
    py aggregate "$(native_path "$CHECKS_JSONL")" "$(native_path "$CHAT_T2_ARTIFACT")" "$MODE" "$CHECK_NAMES")"
  cat "$CHAT_T2_ARTIFACT" 2>/dev/null || true
  log "artefact: $CHAT_T2_ARTIFACT — status $status"
  case "$status" in
    PASS) exit 0 ;;
    BLOCK) exit 1 ;;
    RIG-ABSENT) exit 3 ;;
    NOT-RUN) if [ "$MODE" = "live" ]; then exit 2; else exit "${DRY_EXIT:-0}"; fi ;;
    *) exit 1 ;;
  esac
}

# Every check that could not run because the rig is missing.
all_rig_absent() {
  local n
  for n in $CHECK_NAMES; do record_check "$n" "RIG-ABSENT" "preflight" "" "$1"; done
  finish
}

# --- Input validation (both modes) ----------------------------------------
INPUT_ERR=""
case "$CHAT_APP" in ''|*[!a-z0-9-]*) INPUT_ERR="CHAT_APP must match [a-z0-9-]+, got '$CHAT_APP'" ;; esac
for v in LOCAL_PORT_A LOCAL_PORT_R BOOT_TIMEOUT_SECS JOIN_TIMEOUT_SECS MSG_TIMEOUT_SECS \
         MAILBOX_KEPT_TIMEOUT_SECS OFFLINE_SETTLE_SECS OFFLINE_DELIVERY_TIMEOUT_SECS POLL_SECS; do
  case "${!v}" in ''|*[!0-9]*) INPUT_ERR="$v must be a non-negative integer, got '${!v}'" ;; esac
done
[ "$LOCAL_PORT_A" != "$LOCAL_PORT_R" ] || INPUT_ERR="LOCAL_PORT_A and LOCAL_PORT_R must differ"
case "$VPS_SSH" in ''|-*|*[!A-Za-z0-9@._-]*) INPUT_ERR="VPS_SSH must be a plain ssh alias/user@host, got '$VPS_SSH'" ;; esac
if [ -n "$THIRD_SSH" ]; then
  case "$THIRD_SSH" in -*|*[!A-Za-z0-9@._-]*) INPUT_ERR="THIRD_SSH must be a plain ssh alias/user@host" ;; esac
fi
for v in VPS_DAEMON THIRD_DAEMON; do
  case "${!v}" in ''|http://127.0.0.1:[0-9]*|http://localhost:[0-9]*) ;; *) INPUT_ERR="$v must be a loopback http URL (http://127.0.0.1:PORT)" ;; esac
  case "${!v}" in *[!A-Za-z0-9:/._-]*) INPUT_ERR="$v contains unexpected characters" ;; esac
done
case "$VPS_ROOT" in /*) ;; *) INPUT_ERR="VPS_ROOT must be an absolute path" ;; esac
case "$VPS_ROOT$VPS_UNIT$VPS_SUDO" in *[!A-Za-z0-9/._\ -]*) INPUT_ERR="VPS_ROOT/VPS_UNIT/VPS_SUDO contain unexpected characters" ;; esac

REMOTES="$VPS_SSH"
[ -n "$THIRD_SSH" ] && REMOTES="$VPS_SSH,$THIRD_SSH"

print_plan() {
  log "=== chat v1 T2 — PC + VPS anchor (mode: $MODE, run $RUN_ID) ==="
  log "anchor V      : ssh $VPS_SSH -> $VPS_DAEMON (root $VPS_ROOT, unit $VPS_UNIT, sudo '${VPS_SUDO:-none}')"
  if [ -n "$THIRD_SSH" ]; then
    log "third host T  : ssh $THIRD_SSH -> ${THIRD_DAEMON:-<THIRD_DAEMON unset>}"
  else
    log "third host T  : none (group_of_3 will be RIG-ABSENT)"
  fi
  log "local A / R   : $DAEMON_BIN on 127.0.0.1:$LOCAL_PORT_A / :$LOCAL_PORT_R, roots under $CHAT_T2_RIG_DIR/run-$RUN_ID/"
  log "chat app      : $CHAT_APP"
  log "artefact      : $CHAT_T2_ARTIFACT"
  log "plan (remote actions are marked [REMOTE]):"
  log "  P1 [REMOTE] ssh $VPS_SSH: read loopback token, GET /api/daemon/info (anchor node id)"
  log "  P2 [REMOTE] ssh $VPS_SSH: read $VPS_ROOT/shell-daemon/config.toml, require [mailbox] serve = true"
  log "  P3 [REMOTE] ssh $VPS_SSH: GET /app/$CHAT_APP/chat/identity (chat surface up, not duress)"
  [ -n "$THIRD_SSH" ] && log "  P4 [REMOTE] ssh $THIRD_SSH: token + chat identity on $THIRD_DAEMON"
  log "  P5 [local ] start A and R (fresh roots, [mailbox] servers = [anchor id]), wait /health"
  log "  C1 [REMOTE] A creates a 1:1, V accepts the invite; A->V and V->A messages"
  [ -n "$THIRD_SSH" ] && log "  C2 [REMOTE] A creates a group, V and T accept; each member sends a marker; all see all"
  log "  C3 [local ] A and R opt in to the mailbox; A creates a conversation, R accepts; wait deposit 'kept'"
  log "     [local ] stop R; A sends the canary; wait ${OFFLINE_SETTLE_SECS}s; stop A; restart R; R must read it"
  log "  C4 [REMOTE] ssh $VPS_SSH: positive control in a mktemp dir, then grep -rlaF every known plaintext"
  log "              in $VPS_ROOT and in journalctl -u $VPS_UNIT since the run start (patterns via stdin)"
  log "  C5 [REMOTE] ssh $VPS_SSH: read $VPS_ROOT/shell-daemon/mailbox.json + \$SSH_CLIENT; analyze locally"
  log "  F  [local ] stop A/R, write the artefact"
}

# ==========================================================================
# DRY-RUN — nothing remote, nothing spawned.
# ==========================================================================
if [ "$MODE" != "live" ]; then
  print_plan
  DRY_EXIT=0
  NOTES=""
  [ -z "$INPUT_ERR" ] || { warn "input: $INPUT_ERR"; DRY_EXIT=1; NOTES="input error: $INPUT_ERR"; }
  for tool in ssh curl python3 od; do
    command -v "$tool" >/dev/null 2>&1 || { warn "missing local tool: $tool (live mode would be RIG-ABSENT)"; NOTES="$NOTES missing $tool;"; }
  done
  [ -x "$DAEMON_BIN" ] || { warn "DAEMON_BIN '$DAEMON_BIN' not executable (cargo build -p nexus-shell-daemon --release); live mode would be RIG-ABSENT"; NOTES="$NOTES daemon binary absent;"; }
  python3 -c 'import tomllib' 2>/dev/null || { warn "python3 lacks tomllib (needs >= 3.11)"; NOTES="$NOTES python<3.11;"; }
  log "analyzer self-test (synthetic fixtures, local only):"
  if py selftest; then
    NOTES="${NOTES} selftest ok"; NOTES="${NOTES# }"
  else
    DRY_EXIT=1
    NOTES="${NOTES} SELFTEST FAILED"
  fi
  [ -n "$THIRD_SSH" ] && [ -z "$THIRD_DAEMON" ] && warn "THIRD_SSH set without THIRD_DAEMON: group_of_3 would be RIG-ABSENT"
  log "dry-run complete — re-run with --live to execute (you will be asked to type '$REMOTES')"
  DRY_NOTE="$NOTES"
  finish
fi

# ==========================================================================
# LIVE — explicit confirmation before ANY remote action.
# ==========================================================================
[ -z "$INPUT_ERR" ] || all_rig_absent "invalid input: $INPUT_ERR"
print_plan
if [ -n "${CHAT_T2_CONFIRM:-}" ]; then
  ANSWER="$CHAT_T2_CONFIRM"
elif [ -t 0 ]; then
  printf '[chat-t2] This will act on the remote host(s) listed above. Type "%s" to proceed: ' "$REMOTES"
  IFS= read -r ANSWER || ANSWER=""
else
  ANSWER=""
fi
if [ "$ANSWER" != "$REMOTES" ]; then
  ABORT_REASON="live run not confirmed (expected the exact string '$REMOTES' interactively or in CHAT_T2_CONFIRM); nothing was done"
  warn "$ABORT_REASON"
  finish
fi
log "confirmed — starting"

# --- HTTP plumbing ----------------------------------------------------------
# api_* set API_CODE (HTTP status, 000 on transport error) and API_BODY.
_split_response() {
  API_CODE="${1##*$'\n'}"
  API_BODY="${1%$'\n'*}"
  case "$API_CODE" in [0-9][0-9][0-9]) ;; *) API_CODE="000"; API_BODY="" ;; esac
}

local_api() {
  # $1=base $2=token $3=method $4=path [$5=json body]
  local out
  if [ -n "${5-}" ]; then
    out="$(printf '%s' "$5" | curl -sS -m 30 -X "$3" -H "x-sbfb-token: $2" \
      -H 'Content-Type: application/json' --data-binary @- -w '\n%{http_code}' "$1$4" 2>/dev/null)"
  else
    out="$(curl -sS -m 30 -H "x-sbfb-token: $2" -w '\n%{http_code}' "$1$4" 2>/dev/null </dev/null)"
  fi
  _split_response "$out"
}

remote_api() {
  # $1=ssh alias $2=base $3=token $4=method $5=path [$6=json body]
  # The token is the first stdin line, read remotely into a shell variable:
  # it never appears on any command line the remote host could log.
  local out
  if [ -n "${6-}" ]; then
    out="$(printf '%s\n%s' "$3" "$6" | ssh "${SSH_OPTS[@]}" "$1" \
      "IFS= read -r t; curl -sS -m 30 -X $4 -H \"x-sbfb-token: \$t\" -H 'Content-Type: application/json' --data-binary @- -w '\n%{http_code}' '$2$5'" 2>/dev/null)"
  else
    out="$(printf '%s\n' "$3" | ssh "${SSH_OPTS[@]}" "$1" \
      "IFS= read -r t; curl -sS -m 30 -H \"x-sbfb-token: \$t\" -w '\n%{http_code}' '$2$5'" 2>/dev/null)"
  fi
  _split_response "$out"
}

remote_sh() { ssh "${SSH_OPTS[@]}" "$1" "$2" </dev/null 2>/dev/null; }

# Node dispatch: A (local sender), R (local recipient), V (anchor), T (third).
napi() {
  case "$1" in
    A) local_api "$BASE_A" "$TOK_A" "$2" "$3" "${4-}" ;;
    R) local_api "$BASE_R" "$TOK_R" "$2" "$3" "${4-}" ;;
    V) remote_api "$VPS_SSH" "$VPS_DAEMON" "$TOK_V" "$2" "$3" "${4-}" ;;
    T) remote_api "$THIRD_SSH" "$THIRD_DAEMON" "$TOK_T" "$2" "$3" "${4-}" ;;
  esac
}

APP_PATH="/app/$CHAT_APP/chat"
valid_id() { case "$1" in ''|*[!A-Za-z0-9]*) return 1 ;; *) return 0 ;; esac; }

chat_create() { # node title -> CONV
  CONV=""
  napi "$1" POST "$APP_PATH/conversations" "$(py json "title=$2")"
  [ "$API_CODE" = 200 ] || return 1
  CONV="$(printf '%s' "$API_BODY" | py get id)"
  valid_id "$CONV"
}
chat_invite() { # node conv -> INVITE
  INVITE=""
  napi "$1" POST "$APP_PATH/conversations/$2/invite" '{}'
  [ "$API_CODE" = 200 ] || return 1
  INVITE="$(printf '%s' "$API_BODY" | py get invite)"
  [ -n "$INVITE" ]
}
chat_accept() { # node invite expected_conv
  napi "$1" POST "$APP_PATH/accept" "$(py json "invite=$2")"
  [ "$API_CODE" = 200 ] || return 1
  [ "$(printf '%s' "$API_BODY" | py get id)" = "$3" ]
}
chat_send() { # node conv text
  napi "$1" POST "$APP_PATH/conversations/$2/send" "$(py json "text=$3")"
  [ "$API_CODE" = 200 ] && SENT_TEXTS="${SENT_TEXTS}$3"$'\n'
  [ "$API_CODE" = 200 ]
}
pred_has() { # node conv text
  napi "$1" GET "$APP_PATH/conversations/$2/messages"
  [ "$API_CODE" = 200 ] && [ "$(printf '%s' "$API_BODY" | py has_text "$3")" = 1 ]
}
member_count() { # node conv -> stdout
  napi "$1" GET "$APP_PATH/conversations"
  if [ "$API_CODE" = 200 ]; then printf '%s' "$API_BODY" | py members "$2"; else printf '0'; fi
}
pred_members() { # node conv n
  local c; c="$(member_count "$1" "$2")"
  [ "${c:-0}" -ge "$3" ] 2>/dev/null
}
pred_kept() { # node conv
  napi "$1" GET "$APP_PATH/mailbox"
  [ "$API_CODE" = 200 ] && [ "$(printf '%s' "$API_BODY" | py mbx_status "$2")" = "kept" ]
}

WAIT_ELAPSED=0
wait_until() { # timeout cmd...
  local t="$1" start now
  shift
  start="$(date +%s)"
  while :; do
    if "$@"; then WAIT_ELAPSED=$(( $(date +%s) - start )); return 0; fi
    now="$(date +%s)"
    if [ $(( now - start )) -ge "$t" ]; then WAIT_ELAPSED=$(( now - start )); return 1; fi
    sleep "$POLL_SECS"
  done
}

marker() { printf 't2-chat-%s-%s' "$RUN_ID" "$1"; }
SENT_TEXTS=""

# --- Local hermetic daemons -----------------------------------------------
RUN_DIR="$CHAT_T2_RIG_DIR/run-$RUN_ID"

start_local() { # name port -> sets PID_<NAME> ; returns 1 on failure
  local name="$1" port="$2" root pid base
  root="$RUN_DIR/$name"
  base="http://127.0.0.1:$port"
  mkdir -p "$root/shell-daemon" "$root/.sbfb" || return 1
  if curl -fsS -m 2 "$base/health" >/dev/null 2>&1 </dev/null; then
    START_ERR="port $port already answers /health (another daemon?) — change LOCAL_PORT_A / LOCAL_PORT_R"
    return 1
  fi
  if [ ! -f "$root/shell-daemon/config.toml" ]; then
    printf '[network]\napi_host = "127.0.0.1"\napi_port = %s\n\n[mailbox]\nservers = ["%s"]\n' \
      "$port" "$ANCHOR_ID" >"$root/shell-daemon/config.toml"
  fi
  NEXUS_GRID_ROOT="$(native_path "$root")" SBFB_HOME="$(native_path "$root/.sbfb")" \
    SBFB_PIPE_SUFFIX="-chat-t2-$name-$$" \
    "$DAEMON_BIN" start --headless >>"$root/daemon.log" 2>&1 </dev/null &
  pid=$!
  LOCAL_PIDS="$LOCAL_PIDS $pid"
  case "$name" in a) PID_A="$pid" ;; r) PID_R="$pid" ;; esac
  if ! wait_until "$BOOT_TIMEOUT_SECS" curl -fsS -m 3 "$base/health" -o /dev/null; then
    START_ERR="local daemon $name did not answer /health within ${BOOT_TIMEOUT_SECS}s (log: $root/daemon.log)"
    return 1
  fi
  if ! kill -0 "$pid" 2>/dev/null; then
    START_ERR="local daemon $name exited after boot (another instance on this root?) — log: $root/daemon.log"
    return 1
  fi
  return 0
}

stop_local() { # pid -> 0 when provably dead
  local pid="$1"
  kill -INT "$pid" 2>/dev/null || true
  for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do
    kill -0 "$pid" 2>/dev/null || return 0
    sleep 1
  done
  kill -KILL "$pid" 2>/dev/null || true
  sleep 2
  ! kill -0 "$pid" 2>/dev/null
}

local_token() { curl -fsS -m 5 "$1/auth/token" 2>/dev/null </dev/null | py get token; }
local_node_id() { curl -fsS -m 5 -H "x-sbfb-token: $2" "$1/api/daemon/info" 2>/dev/null </dev/null | py get node_id; }

# ==========================================================================
# PREFLIGHT — failures are RIG-ABSENT, never a product BLOCK.
# ==========================================================================
log "=== preflight ==="
for tool in ssh curl python3 od; do
  command -v "$tool" >/dev/null 2>&1 || all_rig_absent "local tool '$tool' not on PATH"
done
python3 -c 'import tomllib' 2>/dev/null || all_rig_absent "python3 >= 3.11 (tomllib) required"
[ -x "$DAEMON_BIN" ] || all_rig_absent "DAEMON_BIN '$DAEMON_BIN' is not an executable (cargo build -p nexus-shell-daemon --release)"

remote_sh "$VPS_SSH" true || all_rig_absent "ssh to '$VPS_SSH' failed (host down, auth or network)"
TOK_V="$(remote_sh "$VPS_SSH" "curl -fsS -m 10 '$VPS_DAEMON/auth/token'" | py get token)"
[ -n "$TOK_V" ] || all_rig_absent "no loopback token from $VPS_DAEMON/auth/token on the anchor (daemon down or wrong VPS_DAEMON)"
napi V GET /api/daemon/info
ANCHOR_ID="$(printf '%s' "$API_BODY" | py get node_id)"
case "$ANCHOR_ID" in *[!0-9a-f]*|'') all_rig_absent "anchor /api/daemon/info gave no node id (HTTP $API_CODE)" ;; esac
[ "${#ANCHOR_ID}" -eq 64 ] || all_rig_absent "anchor node id is not 64-hex"
SERVE="$(remote_sh "$VPS_SSH" "$VPS_SUDO cat '$VPS_ROOT/shell-daemon/config.toml'" | py mailbox_serve)"
[ "$SERVE" = "true" ] || all_rig_absent "anchor config does not set [mailbox] serve = true (got '$SERVE'; docs/release/IROH_SELFHOST_OPS.md §9)"
napi V GET "$APP_PATH/identity"
[ "$API_CODE" = 200 ] || all_rig_absent "anchor chat surface unavailable (GET identity -> HTTP $API_CODE; 503 = duress or chat disabled)"
log "anchor OK — node ${ANCHOR_ID:0:12}…, mailbox serve on, chat up"

THIRD_OK=0
THIRD_ERR="no third host configured (set THIRD_SSH + THIRD_DAEMON)"
if [ -n "$THIRD_SSH" ]; then
  if [ -z "$THIRD_DAEMON" ]; then
    THIRD_ERR="THIRD_SSH set but THIRD_DAEMON unset"
  elif ! remote_sh "$THIRD_SSH" true; then
    THIRD_ERR="ssh to '$THIRD_SSH' failed"
  else
    TOK_T="$(remote_sh "$THIRD_SSH" "curl -fsS -m 10 '$THIRD_DAEMON/auth/token'" | py get token)"
    if [ -z "$TOK_T" ]; then
      THIRD_ERR="no loopback token from $THIRD_DAEMON on '$THIRD_SSH'"
    else
      napi T GET "$APP_PATH/identity"
      if [ "$API_CODE" = 200 ]; then THIRD_OK=1; else THIRD_ERR="third host chat surface unavailable (HTTP $API_CODE)"; fi
    fi
  fi
fi

START_ERR=""
start_local a "$LOCAL_PORT_A" || all_rig_absent "$START_ERR"
start_local r "$LOCAL_PORT_R" || all_rig_absent "$START_ERR"
BASE_A="http://127.0.0.1:$LOCAL_PORT_A"
BASE_R="http://127.0.0.1:$LOCAL_PORT_R"
TOK_A="$(local_token "$BASE_A")"
TOK_R="$(local_token "$BASE_R")"
[ -n "$TOK_A" ] && [ -n "$TOK_R" ] || all_rig_absent "no loopback token from a local daemon"
NODE_A="$(local_node_id "$BASE_A" "$TOK_A")"
NODE_R="$(local_node_id "$BASE_R" "$TOK_R")"
[ -n "$NODE_A" ] && [ -n "$NODE_R" ] || all_rig_absent "local daemon /api/daemon/info gave no node id"
napi A GET "$APP_PATH/identity"
[ "$API_CODE" = 200 ] || all_rig_absent "local chat surface unavailable on A (HTTP $API_CODE)"
log "local A ${NODE_A:0:12}… and R ${NODE_R:0:12}… up"

# ==========================================================================
# C1 — 1:1 exchange A <-> V
# ==========================================================================
check_one_to_one() {
  local c m1 m2 t0
  log "=== C1 one_to_one (A <-> anchor) ==="
  chat_create A "t2 1:1 $RUN_ID" || { record_check one_to_one BLOCK create "" "A could not create a conversation (HTTP $API_CODE)"; return; }
  c="$CONV"
  chat_invite A "$c" || { record_check one_to_one BLOCK invite "" "A could not mint an invite (HTTP $API_CODE)"; return; }
  chat_accept V "$INVITE" "$c" || { record_check one_to_one BLOCK accept "" "anchor could not accept the invite (HTTP $API_CODE)"; return; }
  t0="$(date +%s)"
  if ! { wait_until "$JOIN_TIMEOUT_SECS" pred_members A "$c" 2 && wait_until "$JOIN_TIMEOUT_SECS" pred_members V "$c" 2; }; then
    record_check one_to_one BLOCK join "$(( $(date +%s) - t0 ))" "membership did not converge to 2 on both sides within ${JOIN_TIMEOUT_SECS}s"
    return
  fi
  m1="$(marker one-a2v)"
  m2="$(marker one-v2a)"
  chat_send A "$c" "$m1" || { record_check one_to_one BLOCK send-a "" "A send failed (HTTP $API_CODE)"; return; }
  t0="$(date +%s)"
  wait_until "$MSG_TIMEOUT_SECS" pred_has V "$c" "$m1" \
    || { record_check one_to_one BLOCK deliver-a2v "$(( $(date +%s) - t0 ))" "anchor did not read A's message within ${MSG_TIMEOUT_SECS}s"; return; }
  chat_send V "$c" "$m2" || { record_check one_to_one BLOCK send-v "" "anchor send failed (HTTP $API_CODE)"; return; }
  wait_until "$MSG_TIMEOUT_SECS" pred_has A "$c" "$m2" \
    || { record_check one_to_one BLOCK deliver-v2a "$(( $(date +%s) - t0 ))" "A did not read the anchor's message within ${MSG_TIMEOUT_SECS}s"; return; }
  record_check one_to_one PASS deliver-v2a "$(( $(date +%s) - t0 ))" "2 members on both sides; A->V and V->A decrypted (round trip $(( $(date +%s) - t0 ))s)"
}

# ==========================================================================
# C2 — group of 3 (A admin, V, T)
# ==========================================================================
check_group() {
  local c n who t0
  log "=== C2 group_of_3 ==="
  if [ "$THIRD_OK" -ne 1 ]; then
    record_check group_of_3 RIG-ABSENT third-host "" "$THIRD_ERR"
    return
  fi
  chat_create A "t2 group $RUN_ID" || { record_check group_of_3 BLOCK create "" "A could not create the group (HTTP $API_CODE)"; return; }
  c="$CONV"
  for who in V T; do
    chat_invite A "$c" || { record_check group_of_3 BLOCK invite "" "A could not mint an invite for $who (HTTP $API_CODE)"; return; }
    chat_accept "$who" "$INVITE" "$c" || { record_check group_of_3 BLOCK accept "" "$who could not accept (HTTP $API_CODE)"; return; }
  done
  t0="$(date +%s)"
  for who in A V T; do
    wait_until "$JOIN_TIMEOUT_SECS" pred_members "$who" "$c" 3 \
      || { record_check group_of_3 BLOCK join "$(( $(date +%s) - t0 ))" "$who does not see 3 members within ${JOIN_TIMEOUT_SECS}s"; return; }
  done
  for who in A V T; do
    chat_send "$who" "$c" "$(marker "group-$who")" || { record_check group_of_3 BLOCK send "" "$who send failed (HTTP $API_CODE)"; return; }
  done
  t0="$(date +%s)"
  for who in A V T; do
    for n in A V T; do
      wait_until "$MSG_TIMEOUT_SECS" pred_has "$who" "$c" "$(marker "group-$n")" \
        || { record_check group_of_3 BLOCK deliver "$(( $(date +%s) - t0 ))" "$who did not read $n's marker within ${MSG_TIMEOUT_SECS}s"; return; }
    done
  done
  record_check group_of_3 PASS deliver "$(( $(date +%s) - t0 ))" "3 members everywhere; 9/9 (reader, sender) pairs decrypted"
}

# ==========================================================================
# C3 — offline delivery through the anchor mailbox (A -> R)
# ==========================================================================
OFFLINE_CONV=""
OFFLINE_KEPT=0
check_offline() {
  local c canary t0 who
  log "=== C3 offline_mailbox (A -> R via the anchor) ==="
  for who in A R; do
    napi "$who" POST "$APP_PATH/mailbox" '{"enabled":true}'
    [ "$API_CODE" = 200 ] || { record_check offline_mailbox RIG-ABSENT opt-in "" "$who could not enable the mailbox (HTTP $API_CODE; 400 = no [mailbox] servers)"; return; }
  done
  chat_create A "t2 offline $RUN_ID" || { record_check offline_mailbox BLOCK create "" "A could not create the conversation (HTTP $API_CODE)"; return; }
  c="$CONV"
  OFFLINE_CONV="$c"
  chat_invite A "$c" || { record_check offline_mailbox BLOCK invite "" "A could not mint an invite (HTTP $API_CODE)"; return; }
  chat_accept R "$INVITE" "$c" || { record_check offline_mailbox BLOCK accept "" "R could not accept (HTTP $API_CODE)"; return; }
  t0="$(date +%s)"
  if ! { wait_until "$JOIN_TIMEOUT_SECS" pred_members A "$c" 2 && wait_until "$JOIN_TIMEOUT_SECS" pred_members R "$c" 2; }; then
    record_check offline_mailbox BLOCK join "$(( $(date +%s) - t0 ))" "A/R membership did not converge to 2"
    return
  fi
  # The anchor must NOT be a member: it would otherwise be a legitimate source.
  [ "$(member_count A "$c")" -eq 2 ] || { record_check offline_mailbox BLOCK membership "" "unexpected member count on A (anchor must not be a member)"; return; }
  t0="$(date +%s)"
  for who in A R; do
    wait_until "$MAILBOX_KEPT_TIMEOUT_SECS" pred_kept "$who" "$c" \
      || { record_check offline_mailbox BLOCK deposit "$(( $(date +%s) - t0 ))" "$who's deposit never reached status 'kept' within ${MAILBOX_KEPT_TIMEOUT_SECS}s"; return; }
  done
  OFFLINE_KEPT=1
  log "deposits kept by the anchor (${WAIT_ELAPSED}s); stopping R"
  stop_local "$PID_R" || { record_check offline_mailbox RIG-ABSENT stop-r "" "could not stop R (pid $PID_R)"; return; }
  canary="$(marker "offline-canary-$(od -An -N8 -tx1 /dev/urandom | tr -d ' \n')")"
  chat_send A "$c" "$canary" || { record_check offline_mailbox BLOCK send "" "A send failed while R offline (HTTP $API_CODE)"; return; }
  log "canary sent by A while R is offline; settling ${OFFLINE_SETTLE_SECS}s for the anchor replica"
  sleep "$OFFLINE_SETTLE_SECS"
  stop_local "$PID_A" || { record_check offline_mailbox RIG-ABSENT stop-a "" "could not stop A (pid $PID_A)"; return; }
  if curl -fsS -m 2 "$BASE_A/health" >/dev/null 2>&1 </dev/null; then
    record_check offline_mailbox RIG-ABSENT stop-a "" "A still answers /health after stop — the sender is not provably offline"
    return
  fi
  log "A stopped (provably: pid gone, /health silent); restarting R"
  start_local r "$LOCAL_PORT_R" || { record_check offline_mailbox RIG-ABSENT restart-r "" "$START_ERR"; return; }
  TOK_R="$(local_token "$BASE_R")"
  [ -n "$TOK_R" ] || { record_check offline_mailbox RIG-ABSENT restart-r "" "no token from restarted R"; return; }
  t0="$(date +%s)"
  if wait_until "$OFFLINE_DELIVERY_TIMEOUT_SECS" pred_has R "$c" "$canary"; then
    record_check offline_mailbox PASS offline-delivery "$WAIT_ELAPSED" "R read the canary ${WAIT_ELAPSED}s after restart with A stopped and the anchor not a member — served from the anchor's ciphertext replica"
  else
    local hint=""
    if [ "$OFFLINE_SETTLE_SECS" -lt "$MAILBOX_RESYNC_SECS" ]; then
      hint=" INCONCLUSIVE between a live-sync gap on the anchor and a delivery failure: settle ${OFFLINE_SETTLE_SECS}s < client resync ${MAILBOX_RESYNC_SECS}s — re-run with OFFLINE_SETTLE_SECS=660 before diagnosing."
    fi
    record_check offline_mailbox BLOCK offline-delivery "$WAIT_ELAPSED" "restarted R did not read the canary within ${OFFLINE_DELIVERY_TIMEOUT_SECS}s (A stopped).${hint} Logs: $RUN_DIR/{a,r}/daemon.log"
  fi
}

# ==========================================================================
# C4 — no known plaintext in the anchor data dir / journal
# ==========================================================================
check_leak_plaintext() {
  local ctl n res files jhits jlines
  log "=== C4 leak_plaintext (anchor data dir + journal) ==="
  if [ -z "$SENT_TEXTS" ]; then
    record_check leak_plaintext RIG-ABSENT no-plaintext "" "no message was sent in this run — nothing to look for"
    return
  fi
  # Positive control: the SAME grep invocation must find a needle planted
  # in a throwaway dir (needle via stdin, dir removed in the same call).
  ctl="$(marker "control-$(od -An -N8 -tx1 /dev/urandom | tr -d ' \n')")"
  n="$(printf '%s\n' "$ctl" | ssh "${SSH_OPTS[@]}" "$VPS_SSH" \
    "umask 077; c=\$(mktemp -d) || exit 9; mkdir \"\$c/data\"; cat >\"\$c/pat\"; cp \"\$c/pat\" \"\$c/data/blob.bin\"; $VPS_SUDO grep -rlaF -f \"\$c/pat\" \"\$c/data\" 2>/dev/null | wc -l; rm -rf \"\$c\"" 2>/dev/null | tr -d ' \r')"
  if [ "$n" != "1" ]; then
    record_check leak_plaintext RIG-ABSENT control "" "positive control failed (grep found '$n' instead of 1 planted needle) — scan pipeline unusable"
    return
  fi
  res="$(printf '%s' "$SENT_TEXTS" | ssh "${SSH_OPTS[@]}" "$VPS_SSH" \
    "umask 077; p=\$(mktemp) || exit 9; cat >\"\$p\"; f=\$($VPS_SUDO grep -rlaF -f \"\$p\" '$VPS_ROOT' 2>/dev/null | wc -l); j=\$($VPS_SUDO journalctl -u '$VPS_UNIT' --since @$(( START_EPOCH - 300 )) -o cat --no-pager 2>/dev/null > \"\$p.j\"; grep -cF -f \"\$p\" \"\$p.j\"); l=\$(wc -l < \"\$p.j\"); rm -f \"\$p\" \"\$p.j\"; echo \"\$f \$j \$l\"" 2>/dev/null | tr -d '\r')"
  read -r files jhits jlines <<<"$res"
  case "$files$jhits$jlines" in ''|*[!0-9]*) record_check leak_plaintext RIG-ABSENT scan "" "remote scan returned no usable counts"; return ;; esac
  n="$(printf '%s' "$SENT_TEXTS" | grep -c .)"
  if [ "$files" -ne 0 ]; then
    record_check leak_plaintext BLOCK data-dir "" "$files file(s) under $VPS_ROOT contain a known plaintext (of $n sent) — inspect with grep -rlaF on the anchor"
  elif [ "$jhits" -ne 0 ]; then
    record_check leak_plaintext BLOCK journal "" "$jhits journal line(s) of $VPS_UNIT contain a known plaintext"
  elif [ "$jlines" -eq 0 ]; then
    record_check leak_plaintext RIG-ABSENT journal "" "data dir clean ($n plaintexts) but the $VPS_UNIT journal is empty/unreadable for the run window — cannot conclude on logs"
  else
    record_check leak_plaintext PASS scan "" "$n known plaintexts: 0 file under $VPS_ROOT, 0 of $jlines journal lines since run start; positive control detected"
  fi
}

# ==========================================================================
# C5 — mailbox registry holds node ids only (no PC IP)
# ==========================================================================
local_ips() {
  case "$(uname -s 2>/dev/null)" in
    MINGW*|MSYS*|CYGWIN*) ipconfig 2>/dev/null | tr -d '\r' | sed -n 's/.*IPv[46][^:]*: *\([0-9A-Fa-f:.]*\).*/\1/p' ;;
    Linux) hostname -I 2>/dev/null | tr ' ' '\n' ;;
    Darwin) ifconfig 2>/dev/null | awk '/inet6? /{print $2}' ;;
  esac | sed 's/%.*//' | grep -E '^[0-9A-Fa-f:.]+$' | grep -vE '^(127\.|::1$|fe80:)' | sort -u
}

check_leak_registry() {
  local reg pcip ips verdict
  log "=== C5 leak_registry (anchor mailbox.json) ==="
  if [ -z "$OFFLINE_CONV" ] || [ "$OFFLINE_KEPT" -ne 1 ]; then
    record_check leak_registry RIG-ABSENT precondition "" "no kept deposit in this run (offline_mailbox did not reach the deposit stage)"
    return
  fi
  reg="$WORK/mailbox.json"
  if ! remote_sh "$VPS_SSH" "$VPS_SUDO cat '$VPS_ROOT/shell-daemon/mailbox.json'" >"$reg" || [ ! -s "$reg" ]; then
    record_check leak_registry BLOCK read "" "deposit reported kept but $VPS_ROOT/shell-daemon/mailbox.json is absent/empty on the anchor"
    return
  fi
  # shellcheck disable=SC2016  # $SSH_CLIENT must expand on the REMOTE side
  pcip="$(remote_sh "$VPS_SSH" 'printf %s "${SSH_CLIENT%% *}"' | tr -d '\r')"
  ips="$( { printf '%s\n' "$pcip"; local_ips; } | grep -v '^$' | sort -u | paste -sd, -)"
  verdict="$(py registry "$(native_path "$reg")" "$OFFLINE_CONV" "$NODE_A,$NODE_R" "$ips")"
  record_check leak_registry "${verdict%%$'\t'*}" registry "" "${verdict#*$'\t'}"
}

check_one_to_one
check_group
check_offline
check_leak_plaintext
check_leak_registry
log "=== done — stopping local daemons ==="
finish
