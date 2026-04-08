import { useEffect, useRef } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useCaseStore } from '../stores/caseStore';
import { useEventStore } from '../stores/eventStore';

/**
 * Map SSE event types to React Query cache keys that should be invalidated.
 */
const INVALIDATION_MAP: Record<string, string[]> = {
  evidence_added: ['evidence', 'caseStats'],
  evidence_processed: ['evidence'],
  evidence_chunked: ['evidence'],
  entity_discovered: ['entities'],
  entity_enriched: ['entities'],
  monitoring_result: ['monitoring', 'alerts'],
  analysis_completed: ['analysisRuns', 'hypotheses'],
  hypothesis_created: ['hypotheses'],
  hypothesis_scored: ['hypotheses'],
  contradiction_found: ['evidence', 'hypotheses'],
  forensic_result: ['evidence'],
  suspect_scored: ['suspects'],
  location_geocoded: ['entities'],
  timeline_rebuilt: ['timeline'],
  wiki_updated: ['wiki'],
};

/**
 * Opens an EventSource connection to the case SSE endpoint and
 * invalidates React Query caches when events arrive.
 *
 * Call once at app level (Layout) — the connection is scoped to
 * the active caseId from caseStore.
 */
export function useCaseSSE() {
  const caseId = useCaseStore((s) => s.caseId);
  const addEvent = useEventStore((s) => s.addEvent);
  const setConnected = useEventStore((s) => s.setConnected);
  const qc = useQueryClient();
  const sourceRef = useRef<EventSource | null>(null);

  useEffect(() => {
    if (!caseId) {
      setConnected(false);
      return;
    }

    const url = `/api/cases/${caseId}/events`;
    const source = new EventSource(url);
    sourceRef.current = source;

    source.onopen = () => setConnected(true);
    source.onerror = () => setConnected(false);

    // Listen for each event type we care about
    for (const eventType of Object.keys(INVALIDATION_MAP)) {
      source.addEventListener(eventType, (e: MessageEvent) => {
        let data: Record<string, unknown> = {};
        try {
          data = JSON.parse(e.data);
        } catch {
          // Ignore malformed SSE data
        }

        addEvent({
          type: eventType,
          case_id: (data.case_id as string) || caseId,
          payload: (data.payload as Record<string, unknown>) || {},
          source_worker: (data.source_worker as string) || '',
          timestamp: (data.timestamp as string) || new Date().toISOString(),
        });

        // Invalidate relevant React Query caches
        const keys = INVALIDATION_MAP[eventType] || [];
        for (const key of keys) {
          qc.invalidateQueries({ queryKey: [key, caseId] });
        }
      });
    }

    // Also invalidate investigation status on any event
    // so PipelineTools refreshes worker states
    source.addEventListener('message', () => {
      qc.invalidateQueries({ queryKey: ['investigationStatus', caseId] });
    });

    return () => {
      source.close();
      sourceRef.current = null;
      setConnected(false);
    };
  }, [caseId, addEvent, setConnected, qc]);
}
