import { useMemo } from 'react';
import { Clock, Calendar } from 'lucide-react';
import Card from '../components/Card';
import Badge from '../components/Badge';
import LoadingSpinner from '../components/LoadingSpinner';
import { useCaseStore } from '../stores/caseStore';
import { useTimeline } from '../hooks/useApi';

interface TimelineEvent {
  id?: string;
  event_id?: string;
  title?: string;
  description?: string;
  date?: string;
  timestamp?: string;
  type?: string;
  category?: string;
  source?: string;
  importance?: string;
  entities?: string[];
}

export default function Timeline() {
  const { caseId } = useCaseStore();
  const timelineQuery = useTimeline();

  if (!caseId) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center">
        <Clock size={48} className="text-[var(--text-muted)] mb-4" />
        <p className="text-[var(--text-secondary)]">Select a case to view the timeline.</p>
      </div>
    );
  }

  const events: TimelineEvent[] = Array.isArray(timelineQuery.data)
    ? timelineQuery.data
    : (timelineQuery.data?.events || []);

  const sorted = useMemo(() => {
    return [...events].sort((a, b) => {
      const dateA = a.date || a.timestamp || '';
      const dateB = b.date || b.timestamp || '';
      return new Date(dateA).getTime() - new Date(dateB).getTime();
    });
  }, [events]);

  // Group by date
  const grouped = useMemo(() => {
    const groups: Record<string, TimelineEvent[]> = {};
    sorted.forEach(ev => {
      const raw = ev.date || ev.timestamp || 'Unknown';
      const dateKey = raw !== 'Unknown' ? new Date(raw).toLocaleDateString('fr-FR', {
        year: 'numeric', month: 'long', day: 'numeric'
      }) : 'Date inconnue';
      if (!groups[dateKey]) groups[dateKey] = [];
      groups[dateKey].push(ev);
    });
    return groups;
  }, [sorted]);

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold text-[var(--text-primary)]">Timeline</h2>
        <p className="text-sm text-[var(--text-muted)]">{events.length} events</p>
      </div>

      {timelineQuery.isLoading ? (
        <LoadingSpinner text="Loading timeline..." />
      ) : events.length === 0 ? (
        <Card>
          <p className="text-sm text-[var(--text-muted)] text-center py-8">
            No timeline events yet. Events are extracted from evidence during analysis.
          </p>
        </Card>
      ) : (
        <div className="relative">
          {/* Vertical line */}
          <div className="absolute left-6 top-0 bottom-0 w-px bg-[var(--border)]" />

          <div className="space-y-8">
            {Object.entries(grouped).map(([dateKey, evs]) => (
              <div key={dateKey}>
                {/* Date header */}
                <div className="flex items-center gap-3 mb-4 relative">
                  <div className="w-12 h-12 rounded-full bg-[var(--bg-card)] border border-[var(--border)] flex items-center justify-center z-10">
                    <Calendar size={18} className="text-[var(--accent)]" />
                  </div>
                  <h3 className="text-sm font-semibold text-[var(--text-primary)]">{dateKey}</h3>
                </div>

                {/* Events */}
                <div className="ml-16 space-y-3">
                  {evs.map((ev, i) => (
                    <div
                      key={ev.id || ev.event_id || i}
                      className="bg-[var(--bg-card)] border border-[var(--border)] rounded-lg p-4 relative"
                    >
                      {/* Connector line */}
                      <div className="absolute -left-10 top-5 w-10 h-px bg-[var(--border)]" />
                      <div className="absolute -left-[42px] top-[17px] w-2 h-2 rounded-full bg-[var(--accent)] border-2 border-[var(--bg-primary)]" />

                      <div className="flex items-start justify-between mb-2">
                        <h4 className="text-sm font-medium text-[var(--text-primary)]">
                          {ev.title || ev.description || 'Unnamed event'}
                        </h4>
                        <div className="flex items-center gap-2 shrink-0 ml-3">
                          {ev.type && <Badge type={ev.type} />}
                          {ev.importance && <Badge type={ev.importance} />}
                        </div>
                      </div>

                      {ev.description && ev.title && (
                        <p className="text-xs text-[var(--text-secondary)] mb-2">{ev.description}</p>
                      )}

                      <div className="flex items-center gap-4 text-[10px] text-[var(--text-muted)]">
                        {(ev.date || ev.timestamp) && (
                          <span>{new Date(ev.date || ev.timestamp!).toLocaleTimeString()}</span>
                        )}
                        {ev.source && <span>Source: {ev.source}</span>}
                        {ev.entities && ev.entities.length > 0 && (
                          <span>{ev.entities.length} entities</span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
