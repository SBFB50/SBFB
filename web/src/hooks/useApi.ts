import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useCaseStore } from '../stores/caseStore';
import * as api from '../api/client';

export function useCases() {
  return useQuery({ queryKey: ['cases'], queryFn: api.getCases });
}

export function useEvidence() {
  const { caseId } = useCaseStore();
  return useQuery({
    queryKey: ['evidence', caseId],
    queryFn: () => api.getEvidence(caseId!),
    enabled: !!caseId,
  });
}

export function useEntities() {
  const { caseId } = useCaseStore();
  return useQuery({
    queryKey: ['entities', caseId],
    queryFn: () => api.getEntities(caseId!),
    enabled: !!caseId,
  });
}

export function useHypotheses() {
  const { caseId } = useCaseStore();
  return useQuery({
    queryKey: ['hypotheses', caseId],
    queryFn: () => api.getHypotheses(caseId!),
    enabled: !!caseId,
  });
}

export function useGraph() {
  const { caseId } = useCaseStore();
  return useQuery({
    queryKey: ['graph', caseId],
    queryFn: () => api.getGraph(caseId!),
    enabled: !!caseId,
    refetchInterval: 30000,
  });
}

export function useTimeline() {
  const { caseId } = useCaseStore();
  return useQuery({
    queryKey: ['timeline', caseId],
    queryFn: () => api.getTimeline(caseId!),
    enabled: !!caseId,
  });
}

export function useAlerts() {
  const { caseId } = useCaseStore();
  return useQuery({
    queryKey: ['alerts', caseId],
    queryFn: () => api.getAlerts(caseId!),
    enabled: !!caseId,
  });
}

export function useUnreadCount() {
  const { caseId } = useCaseStore();
  return useQuery({
    queryKey: ['unreadCount', caseId],
    queryFn: () => api.getUnreadCount(caseId!),
    enabled: !!caseId,
    refetchInterval: 5000,
  });
}

export function useAuditLog(limit = 50) {
  const { caseId } = useCaseStore();
  return useQuery({
    queryKey: ['audit', caseId, limit],
    queryFn: () => api.getAuditLog(caseId!, limit),
    enabled: !!caseId,
  });
}

export function useInvestigationStatus() {
  const { caseId } = useCaseStore();
  return useQuery({
    queryKey: ['investigationStatus', caseId],
    queryFn: () => api.getInvestigationStatus(caseId!),
    enabled: !!caseId,
    refetchInterval: 5000,
  });
}

export function useAnalysisRuns() {
  const { caseId } = useCaseStore();
  return useQuery({
    queryKey: ['analysisRuns', caseId],
    queryFn: () => api.getAnalysisRuns(caseId!),
    enabled: !!caseId,
  });
}

export function useMonitoringJobs() {
  const { caseId } = useCaseStore();
  return useQuery({
    queryKey: ['monitoring', caseId],
    queryFn: () => api.getMonitoringJobs(caseId!),
    enabled: !!caseId,
  });
}

export function useBenchmarkResults() {
  return useQuery({
    queryKey: ['benchmarkResults'],
    queryFn: api.getBenchmarkResults,
  });
}

export function useSubmitEvidence() {
  const { caseId } = useCaseStore();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { content: string; source?: string }) =>
      api.submitTextEvidence(caseId!, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['evidence', caseId] });
      qc.invalidateQueries({ queryKey: ['caseStats', caseId] });
    },
  });
}

export function useCreateCase() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { name: string; description?: string }) => api.createCase(data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['cases'] }),
  });
}

export function useTriggerAnalysis() {
  const { caseId } = useCaseStore();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => api.triggerAnalysis(caseId!),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['analysisRuns', caseId] });
      qc.invalidateQueries({ queryKey: ['hypotheses', caseId] });
      qc.invalidateQueries({ queryKey: ['entities', caseId] });
    },
  });
}

export function useStartInvestigation() {
  const { caseId } = useCaseStore();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => api.startInvestigation(caseId!),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['investigationStatus', caseId] }),
  });
}

export function useStopInvestigation() {
  const { caseId } = useCaseStore();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => api.stopInvestigation(caseId!),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['investigationStatus', caseId] }),
  });
}

export function useGenerateHypotheses() {
  const { caseId } = useCaseStore();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => api.generateHypotheses(caseId!),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['hypotheses', caseId] }),
  });
}

export function useEvaluateHypotheses() {
  const { caseId } = useCaseStore();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => api.evaluateHypotheses(caseId!),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['hypotheses', caseId] }),
  });
}
