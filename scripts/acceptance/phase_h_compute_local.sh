#!/usr/bin/env bash
# Sprint 76 Phase H — LOCAL compute acceptance.
# Replicates EXACTLY what the host bridge does for an iframe app:
#   1. resolve the local project doc id (GET /api/daemon/project-info)
#   2. submit a compute task carrying that project_id (POST /api/v1/tasks/submit)
#   3. poll the result (GET /api/v1/tasks/{id}/result) until 200
# Proves: app -> (bridge logic) -> daemon -> on-demand local worker
#         (Ollama) -> result -> /result. No mock.
set -uo pipefail

BASE="${BASE:-http://127.0.0.1:7654}"
PROMPT="${PROMPT:-Donne trois noms pour un chat roux. Reponds en une ligne.}"
MODEL="${MODEL:-llama3.1:8b}"

echo "== auth =="
TOKEN="$(curl -s "$BASE/auth/token" | sed -E 's/.*"token":"([^"]+)".*/\1/')"
if [ -z "$TOKEN" ] || [ "$TOKEN" = "$(curl -s "$BASE/auth/token")" ]; then
  echo "FAIL: could not parse token from $BASE/auth/token"; exit 1
fi
H=(-H "x-sbfb-token: $TOKEN")
echo "token ok (${#TOKEN} chars)"

echo "== project-info (the new route) =="
PI="$(curl -s "${H[@]}" "$BASE/api/daemon/project-info")"
echo "$PI"
PID="$(printf '%s' "$PI" | sed -E 's/.*"project_doc_id":"?([^",}]+)"?.*/\1/')"
if [ -z "$PID" ] || [ "$PID" = "null" ]; then
  echo "FAIL: no project_doc_id (daemon has no project doc mounted)"; exit 1
fi
echo "project_id = $PID"

echo "== submit (bridge injects project_id) =="
SUB="$(curl -s "${H[@]}" -H 'content-type: application/json' \
  -d "{\"project_id\":\"$PID\",\"task_type\":\"inference\",\"prompt\":\"$PROMPT\",\"model\":\"$MODEL\"}" \
  "$BASE/api/v1/tasks/submit")"
echo "$SUB" | head -c 400; echo
TID="$(printf '%s' "$SUB" | sed -E 's/.*"task_id":"([^"]+)".*/\1/')"
if [ -z "$TID" ]; then echo "FAIL: no task_id in submit response"; exit 1; fi
echo "task_id = $TID"

echo "== poll /result (max 120s) =="
START=$(date +%s)
while :; do
  R="$(curl -s -o /tmp/ct_result.json -w '%{http_code}' "${H[@]}" "$BASE/api/v1/tasks/$TID/result")"
  NOW=$(date +%s); EL=$((NOW-START))
  if [ "$R" = "200" ]; then
    echo "RESULT after ${EL}s:"
    cat /tmp/ct_result.json; echo
    echo "PASS"; exit 0
  fi
  if [ "$EL" -ge 120 ]; then
    echo "TIMEOUT after ${EL}s (last http=$R)"; cat /tmp/ct_result.json 2>/dev/null; echo
    echo "FAIL"; exit 1
  fi
  printf 'pending %ss (http=%s)\r' "$EL" "$R"
  sleep 2
done
