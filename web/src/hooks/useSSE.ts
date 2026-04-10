import { useEffect, useRef, useState } from 'react';
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

/**
 * Map GOV SSE event types to React Query cache keys that should be invalidated.
 */
const GOV_INVALIDATION_MAP: Record<string, string[]> = {
  gov_position_added: ['gov-stats', 'gov-politicians', 'gov-positions'],
  gov_contradiction_found: ['gov-all-contradictions', 'gov-stats', 'gov-alerts'],
  gov_press_added: ['gov-press', 'gov-press-politician'],
  gov_social_post_added: ['gov-all-social', 'gov-social'],
  gov_affair_added: ['gov-affairs', 'gov-affairs-politician', 'gov-stats'],
  gov_pattern_detected: ['gov-stats'],
  gov_alert_created: ['gov-alerts'],
  gov_politician_added: ['gov-politicians', 'gov-stats'],
  gov_declaration_added: ['gov-declarations'],
  gov_factcheck_added: ['gov-factchecks'],
  gov_transcription_ready: ['gov-all-transcriptions', 'gov-transcriptions'],
};

/**
 * Opens an EventSource connection to the GOV SSE endpoint and
 * invalidates React Query caches when government events arrive.
 *
 * Call once at app level (Layout) — the connection stays open as
 * long as the app is mounted.  EventSource handles reconnection
 * natively on disconnect.
 */
export function useGovSSE() {
  const qc = useQueryClient();
  const sourceRef = useRef<EventSource | null>(null);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    const baseUrl = import.meta.env.VITE_API_URL || '';
    const url = `${baseUrl}/api/gov/events`;
    const source = new EventSource(url);
    sourceRef.current = source;

    source.onopen = () => setConnected(true);
    source.onerror = () => {
      setConnected(false);
      // EventSource reconnects automatically
    };

    // Listen for each GOV event type
    for (const eventType of Object.keys(GOV_INVALIDATION_MAP)) {
      source.addEventListener(eventType, () => {
        const keys = GOV_INVALIDATION_MAP[eventType] || [];
        for (const key of keys) {
          qc.invalidateQueries({ queryKey: [key] });
        }
      });
    }

    return () => {
      source.close();
      sourceRef.current = null;
      setConnected(false);
    };
  }, [qc]);

  return connected;
}
