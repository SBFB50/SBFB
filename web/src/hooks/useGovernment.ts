import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import * as gov from '../api/government';

export function useGovStats() {
  return useQuery({
    queryKey: ['gov-stats'],
    queryFn: gov.getGovStats,
    refetchInterval: 30_000,
  });
}

export function useGovPoliticians(filters?: Record<string, string>) {
  return useQuery({
    queryKey: ['gov-politicians', filters],
    queryFn: () => gov.getGovPoliticians(filters),
  });
}

export function useGovPolitician(id: string | null) {
  return useQuery({
    queryKey: ['gov-politician', id],
    queryFn: () => gov.getGovPolitician(id!),
    enabled: !!id,
  });
}

export function useGovPositions(politicianId: string | null) {
  return useQuery({
    queryKey: ['gov-positions', politicianId],
    queryFn: () => gov.getGovPositions(politicianId!),
    enabled: !!politicianId,
  });
}

export function useGovPoliticianContradictions(politicianId: string | null) {
  return useQuery({
    queryKey: ['gov-contradictions', politicianId],
    queryFn: () => gov.getGovPoliticianContradictions(politicianId!),
    enabled: !!politicianId,
  });
}

export function useGovAllContradictions() {
  return useQuery({
    queryKey: ['gov-all-contradictions'],
    queryFn: () => gov.getGovContradictions(),
  });
}

// Scan — start / stop / live status
export function useGovScanStatus() {
  return useQuery({
    queryKey: ['gov-scan-status'],
    queryFn: gov.getGovScanStatus,
    refetchInterval: 2_000,
  });
}

export function useTriggerGovScan() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: gov.triggerGovScan,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['gov-scan-status'] }),
  });
}

export function useStopGovScan() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: gov.stopGovScan,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['gov-scan-status'] });
      qc.invalidateQueries({ queryKey: ['gov-stats'] });
      qc.invalidateQueries({ queryKey: ['gov-politicians'] });
    },
  });
}

// Detection — start / stop / live status
export function useGovDetectionStatus() {
  return useQuery({
    queryKey: ['gov-detect-status'],
    queryFn: gov.getGovDetectionStatus,
    refetchInterval: 2_000,
  });
}

export function useDetectGovContradictions() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: gov.detectGovContradictions,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['gov-detect-status'] }),
  });
}

export function useStopGovDetection() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: gov.stopGovDetection,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['gov-detect-status'] });
      qc.invalidateQueries({ queryKey: ['gov-stats'] });
      qc.invalidateQueries({ queryKey: ['gov-all-contradictions'] });
    },
  });
}

export function useGovGraph(params?: Record<string, string>) {
  return useQuery({
    queryKey: ['gov-graph', params],
    queryFn: () => gov.getGovGraph(params),
    refetchInterval: 60_000,
  });
}

// Social
export function useGovSocial(politicianId: string | null, platform?: string) {
  return useQuery({
    queryKey: ['gov-social', politicianId, platform],
    queryFn: () => gov.getGovSocial(politicianId!, platform ? { platform } : undefined),
    enabled: !!politicianId,
  });
}
export function useGovAllSocial(platform?: string) {
  return useQuery({
    queryKey: ['gov-all-social', platform],
    queryFn: () => gov.getGovAllSocial(platform ? { platform } : undefined),
  });
}

// Transcriptions
export function useGovTranscriptions(politicianId: string | null) {
  return useQuery({
    queryKey: ['gov-transcriptions', politicianId],
    queryFn: () => gov.getGovTranscriptions(politicianId!),
    enabled: !!politicianId,
  });
}
export function useGovAllTranscriptions() {
  return useQuery({ queryKey: ['gov-all-transcriptions'], queryFn: gov.getGovAllTranscriptions });
}

// Alerts
export function useGovAlerts() {
  return useQuery({ queryKey: ['gov-alerts'], queryFn: () => gov.getGovAlerts(), refetchInterval: 10_000 });
}
export function useMarkAlertRead() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: gov.markGovAlertRead,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['gov-alerts'] }),
  });
}

// Press
export function useGovPress(sentiment?: string) {
  return useQuery({
    queryKey: ['gov-press', sentiment],
    queryFn: () => gov.getGovPress(sentiment ? { sentiment } : undefined),
  });
}
export function useGovPressByPolitician(politicianId: string | null) {
  return useQuery({
    queryKey: ['gov-press-politician', politicianId],
    queryFn: () => gov.getGovPressByPolitician(politicianId!),
    enabled: !!politicianId,
  });
}

// Affairs
export function useGovAffairs() {
  return useQuery({ queryKey: ['gov-affairs'], queryFn: () => gov.getGovAffairs() });
}
export function useGovAffairsByPolitician(politicianId: string | null) {
  return useQuery({
    queryKey: ['gov-affairs-politician', politicianId],
    queryFn: () => gov.getGovAffairsByPolitician(politicianId!),
    enabled: !!politicianId,
  });
}

// Laws
export function useGovLaws() {
  return useQuery({ queryKey: ['gov-laws'], queryFn: () => gov.getGovLaws() });
}

// Declarations
export function useGovDeclarations(politicianId: string | null) {
  return useQuery({
    queryKey: ['gov-declarations', politicianId],
    queryFn: () => gov.getGovDeclarations(politicianId!),
    enabled: !!politicianId,
  });
}

// Factchecks
export function useGovFactchecks() {
  return useQuery({ queryKey: ['gov-factchecks'], queryFn: () => gov.getGovFactchecks() });
}

// Search (RAG)
export function useGovSearch(query: string) {
  return useQuery({
    queryKey: ['gov-search', query],
    queryFn: () => gov.searchGov(query),
    enabled: query.length >= 2,
  });
}

// Pipeline
export function useGovWorkers() {
  return useQuery({ queryKey: ['gov-workers'], queryFn: gov.getGovWorkers, refetchInterval: 5_000 });
}
export function useGovPipeline() {
  return useQuery({ queryKey: ['gov-pipeline'], queryFn: gov.getGovPipeline, refetchInterval: 10_000 });
}
