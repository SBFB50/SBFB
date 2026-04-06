import { useState, useEffect, useMemo } from 'react';
// @ts-ignore — react-calendar-timeline types are incomplete
import Timeline from 'react-calendar-timeline';
import 'react-calendar-timeline/lib/Timeline.css';
import moment from 'moment';
import { api } from '../api/client';

// Dark theme override
const TIMELINE_STYLES = `
.react-calendar-timeline .rct-header-root { background: var(--bg-card) !important; border-bottom: 1px solid var(--border) !important; }
.react-calendar-timeline .rct-calendar-header { background: var(--bg-card) !important; }
.react-calendar-timeline .rct-dateHeader { background: var(--bg-primary) !important; color: var(--text-muted) !important; border: 1px solid var(--border) !important; font-size: 10px !important; }
.react-calendar-timeline .rct-sidebar .rct-sidebar-row { background: var(--bg-card) !important; color: var(--text-primary) !important; border-bottom: 1px solid var(--border) !important; font-size: 11px !important; }
.react-calendar-timeline .rct-horizontal-lines .rct-hl-even, .react-calendar-timeline .rct-horizontal-lines .rct-hl-odd { background: var(--bg-primary) !important; border-bottom: 1px solid var(--border) !important; }
.react-calendar-timeline .rct-vertical-lines .rct-vl { border-left: 1px solid var(--border) !important; }
.react-calendar-timeline .rct-scroll { background: var(--bg-primary) !important; }
.react-calendar-timeline .rct-item { border-radius: 4px !important; border: none !important; font-size: 10px !important; }
.react-calendar-timeline .rct-item .rct-item-content { padding: 2px 6px !important; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
`;

const GROUP_COLORS: Record<string, string> = {
  evidence: '#3b82f6',
  entity: '#22c55e',
  hypothesis: '#a855f7',
  suspect: '#ef4444',
  monitoring: '#06b6d4',
  alert: '#eab308',
  analysis: '#f97316',
};

interface TimelineEvent {
  id: string;
  group: string;
  title: string;
  start: number;
  end: number;
  color: string;
}

export default function InvestigationTimeline({ caseId }: { caseId: string }) {
  const [events, setEvents] = useState<TimelineEvent[]>([]);
  const [collapsed, setCollapsed] = useState(true);

  useEffect(() => {
    if (!caseId || collapsed) return;

    const fetchData = async () => {
      try {
        const auditResp = await api.get(`/cases/${caseId}/audit?limit=100`);
        const auditData = auditResp.data || [];

        const items: TimelineEvent[] = [];

        for (const entry of auditData) {
          const ts = entry.timestamp ? new Date(entry.timestamp).getTime() : 0;
          if (!ts) continue;

          const action = entry.action || '';
          let group = 'analysis';
          let color = GROUP_COLORS.analysis;

          if (action.includes('evidence')) { group = 'evidence'; color = GROUP_COLORS.evidence; }
          else if (action.includes('entity') || action.includes('geocode')) { group = 'entity'; color = GROUP_COLORS.entity; }
          else if (action.includes('hypothesis')) { group = 'hypothesis'; color = GROUP_COLORS.hypothesis; }
          else if (action.includes('suspect')) { group = 'suspect'; color = GROUP_COLORS.suspect; }
          else if (action.includes('monitoring') || action.includes('osint')) { group = 'monitoring'; color = GROUP_COLORS.monitoring; }
          else if (action.includes('alert') || action.includes('contradiction')) { group = 'alert'; color = GROUP_COLORS.alert; }

          items.push({
            id: entry.id || `evt-${items.length}`,
            group,
            title: (entry.summary || action).slice(0, 60),
            start: ts,
            end: ts + 60000, // 1 min width
            color,
          });
        }

        setEvents(items);
      } catch {
        // ignore
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 10000);
    return () => clearInterval(interval);
  }, [caseId, collapsed]);

  const groups = useMemo(() => [
    { id: 'evidence', title: 'Preuves' },
    { id: 'entity', title: 'Entites' },
    { id: 'hypothesis', title: 'Hypotheses' },
    { id: 'suspect', title: 'Suspects' },
    { id: 'monitoring', title: 'Monitoring' },
    { id: 'alert', title: 'Alertes' },
    { id: 'analysis', title: 'Analyse' },
  ], []);

  const timelineItems = useMemo(() =>
    events.map(e => ({
      id: e.id,
      group: e.group,
      title: e.title,
      start_time: moment(e.start),
      end_time: moment(e.end),
      itemProps: {
        style: {
          background: e.color,
          color: '#fff',
          fontSize: '10px',
          borderRadius: '4px',
          border: 'none',
        },
      },
    })),
  [events]);

  // Default time range: from first event to last event, or last hour
  const defaultStart = events.length > 0
    ? moment(Math.min(...events.map(e => e.start))).subtract(5, 'minutes')
    : moment().subtract(1, 'hour');
  const defaultEnd = events.length > 0
    ? moment(Math.max(...events.map(e => e.start))).add(5, 'minutes')
    : moment();

  return (
    <div className="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl overflow-hidden">
      <style>{TIMELINE_STYLES}</style>
      <button
        onClick={() => setCollapsed(!collapsed)}
        className="w-full flex items-center justify-between px-4 py-3 text-sm font-semibold text-[var(--text-primary)] hover:bg-[var(--bg-hover)] transition-colors"
      >
        <span>Chronologie de l'enquete ({events.length} events)</span>
        <span className="text-[var(--text-muted)]">{collapsed ? '+' : '-'}</span>
      </button>

      {!collapsed && events.length > 0 && (
        <div className="border-t border-[var(--border)]" style={{ height: 300 }}>
          <Timeline
            groups={groups}
            items={timelineItems}
            defaultTimeStart={defaultStart}
            defaultTimeEnd={defaultEnd}
            sidebarWidth={100}
            lineHeight={36}
            canMove={false}
            canResize={false}
            canChangeGroup={false}
          />
        </div>
      )}

      {!collapsed && events.length === 0 && (
        <div className="px-4 py-8 text-center text-xs text-[var(--text-muted)] border-t border-[var(--border)]">
          Aucun evenement — les events apparaitront pendant l'investigation
        </div>
      )}

      {/* Legend */}
      {!collapsed && events.length > 0 && (
        <div className="flex gap-3 px-4 py-2 border-t border-[var(--border)]">
          {Object.entries(GROUP_COLORS).map(([key, color]) => (
            <span key={key} className="flex items-center gap-1 text-[9px] text-[var(--text-muted)]">
              <span className="w-2 h-2 rounded-sm" style={{ backgroundColor: color }} />
              {key}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
