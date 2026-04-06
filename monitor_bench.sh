#!/bin/bash
CASE_ID="06d2da22-e6b2-4677-8aeb-8974c04bc0ed"
BASE="http://localhost:8000/api/cases/${CASE_ID}"
START=$(date +%s)
MAX_DURATION=600
INTERVAL=15
CYCLE=0
STOP_REASON=""
HYP_SHOWN=0

echo "=========================================="
echo " NEXUS Kulik Benchmark Monitor"
echo " Case: ${CASE_ID}"
echo " Started: $(date '+%Y-%m-%d %H:%M:%S')"
echo " Max duration: 10 minutes, poll: 15s"
echo "=========================================="
echo ""

parse_stats() {
    python -c "
import sys, json
try:
    d = json.load(sys.stdin)
    ev = d.get('evidence', d.get('evidence_count', d.get('total_evidence', '?')))
    en = d.get('entities', d.get('entity_count', d.get('total_entities', '?')))
    hy = d.get('hypotheses', d.get('hypothesis_count', d.get('total_hypotheses', '?')))
    al = d.get('alerts', d.get('alert_count', d.get('total_alerts', '?')))
    print(f'{ev}|{en}|{hy}|{al}')
except:
    print('?|?|?|?')
" 2>/dev/null
}

parse_status() {
    python -c "
import sys, json
try:
    d = json.load(sys.stdin)
    running = d.get('running', None)
    if running is True:
        phase = 'RUNNING'
    elif running is False:
        cc = d.get('cycle_count', 0)
        phase = 'SLEEPING' if cc > 0 else 'IDLE'
    else:
        phase = d.get('phase', d.get('current_phase', d.get('status', '?')))
    la = d.get('last_action', '')
    if la:
        phase = f'{phase}({la})'
    cyc = d.get('cycle_count', d.get('cycles_completed', d.get('cycle', '?')))
    print(f'{phase}|{cyc}')
except:
    print('?|?')
" 2>/dev/null
}

parse_audit() {
    python -c "
import sys, json
try:
    d = json.load(sys.stdin)
    entries = d if isinstance(d, list) else d.get('entries', d.get('audits', d.get('items', [])))
    if entries and len(entries) > 0:
        e = entries[0]
        action = e.get('action', e.get('event', e.get('type', '?')))
        print(action)
    else:
        print('none')
except:
    print('?')
" 2>/dev/null
}

print_hypotheses() {
    python -c "
import sys, json
try:
    d = json.load(sys.stdin)
    hyps = d if isinstance(d, list) else d.get('hypotheses', d.get('items', []))
    for h in hyps[:10]:
        title = h.get('title', h.get('text', '?'))[:70]
        score = h.get('score', h.get('confidence', '?'))
        status = h.get('status', '')
        print(f'    [{score}] {title} ({status})')
    if not hyps:
        print('    (none)')
except:
    print('    (parse error)')
" 2>/dev/null
}

print_suspects() {
    python -c "
import sys, json
try:
    d = json.load(sys.stdin)
    suspects = d if isinstance(d, list) else d.get('suspects', d.get('items', []))
    for s in suspects[:10]:
        name = s.get('name', s.get('entity_name', '?'))
        score = s.get('score', s.get('total_score', '?'))
        print(f'    {name}: {score}')
    if not suspects:
        print('    (none yet)')
except:
    print('    (parse error)')
" 2>/dev/null
}

while true; do
    NOW=$(date +%s)
    ELAPSED=$(( NOW - START ))

    if [ $ELAPSED -ge $MAX_DURATION ]; then
        STOP_REASON="10-minute timeout reached"
        break
    fi

    CYCLE=$((CYCLE + 1))
    TS=$(date '+%H:%M:%S')

    STATS_RAW=$(curl -s --max-time 5 "${BASE}/stats" 2>/dev/null)
    STATUS_RAW=$(curl -s --max-time 5 "${BASE}/investigation/status" 2>/dev/null)
    AUDIT_RAW=$(curl -s --max-time 5 "${BASE}/audit?limit=5" 2>/dev/null)

    STATS_PARSED=$(echo "$STATS_RAW" | parse_stats)
    STATUS_PARSED=$(echo "$STATUS_RAW" | parse_status)
    AUDIT_LAST=$(echo "$AUDIT_RAW" | parse_audit)

    EV=$(echo "$STATS_PARSED" | cut -d'|' -f1)
    EN=$(echo "$STATS_PARSED" | cut -d'|' -f2)
    HY=$(echo "$STATS_PARSED" | cut -d'|' -f3)
    AL=$(echo "$STATS_PARSED" | cut -d'|' -f4)

    PH=$(echo "$STATUS_PARSED" | cut -d'|' -f1)
    CY=$(echo "$STATUS_PARSED" | cut -d'|' -f2)

    printf "[%s] Evidence: %s/14 | Entities: %s | Hyps: %s | Alerts: %s | Phase: %s | Cycle: %s | Last: %s\n" \
        "$TS" "$EV" "$EN" "$HY" "$AL" "$PH" "$CY" "$AUDIT_LAST"

    # Show hypotheses/suspects when evidence is 14+ and hyps exist
    EV_NUM=$(echo "$EV" | grep -oE '^[0-9]+$')
    HY_NUM=$(echo "$HY" | grep -oE '^[0-9]+$')
    if [ -n "$EV_NUM" ] && [ -n "$HY_NUM" ] && [ "$EV_NUM" -ge 14 ] && [ "$HY_NUM" -gt 0 ]; then
        if [ $HYP_SHOWN -eq 0 ] || [ $((CYCLE % 4)) -eq 0 ]; then
            HYP_SHOWN=1
            echo "  >> Hypotheses:"
            curl -s --max-time 5 "${BASE}/hypotheses" 2>/dev/null | print_hypotheses
            echo "  >> Suspects:"
            curl -s --max-time 5 "${BASE}/suspects" 2>/dev/null | print_suspects
        fi
    fi

    # Check for sleeping/completed (phase may contain parens like SLEEPING(action))
    PH_BASE=$(echo "$PH" | cut -d'(' -f1 | tr '[:lower:]' '[:upper:]' | tr -d ' ')
    if [ "$PH_BASE" = "SLEEPING" ] || [ "$PH_BASE" = "COMPLETED" ]; then
        # Only stop if cycle_count > 0 (actual work was done)
        CY_NUM=$(echo "$CY" | grep -oE '^[0-9]+$')
        if [ -n "$CY_NUM" ] && [ "$CY_NUM" -gt 0 ]; then
            STOP_REASON="Investigation reached ${PH_BASE} state after ${CY} cycles"
            break
        fi
    fi

    sleep $INTERVAL
done

echo ""
echo "=========================================="
echo " FINAL SUMMARY"
echo " Stop reason: ${STOP_REASON}"
echo " Duration: $(( $(date +%s) - START )) seconds"
echo " Polls: ${CYCLE}"
echo "=========================================="
echo ""
echo "--- Case Stats ---"
curl -s --max-time 5 "${BASE}/stats" 2>/dev/null | python -c "
import sys, json
try:
    d = json.load(sys.stdin)
    for k, v in sorted(d.items()):
        print(f'  {k}: {v}')
except:
    print('  (unavailable)')
"
echo ""
echo "--- Investigation Status ---"
curl -s --max-time 5 "${BASE}/investigation/status" 2>/dev/null | python -c "
import sys, json
try:
    d = json.load(sys.stdin)
    for k, v in sorted(d.items()):
        if not isinstance(v, (dict, list)):
            print(f'  {k}: {v}')
except:
    print('  (unavailable)')
"
echo ""
echo "--- Hypotheses ---"
curl -s --max-time 5 "${BASE}/hypotheses" 2>/dev/null | print_hypotheses
echo ""
echo "--- Suspects ---"
curl -s --max-time 5 "${BASE}/suspects" 2>/dev/null | print_suspects
echo ""
echo "--- Last 5 Audit Entries ---"
curl -s --max-time 5 "${BASE}/audit?limit=5" 2>/dev/null | python -c "
import sys, json
try:
    d = json.load(sys.stdin)
    entries = d if isinstance(d, list) else d.get('entries', d.get('audits', d.get('items', [])))
    for e in entries[:5]:
        action = e.get('action', e.get('event', e.get('type', '?')))
        ts = e.get('timestamp', e.get('created_at', ''))
        details = e.get('details', e.get('message', ''))
        if isinstance(details, dict):
            details = str(details)[:100]
        elif isinstance(details, str):
            details = details[:100]
        print(f'  [{ts}] {action}: {details}')
    if not entries:
        print('  (none)')
except:
    print('  (unavailable)')
"
echo ""
echo "=========================================="
echo " Monitor ended: $(date '+%Y-%m-%d %H:%M:%S')"
echo "=========================================="
