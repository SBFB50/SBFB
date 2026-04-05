import axios from 'axios';
import { QueryClient } from '@tanstack/react-query';

export const api = axios.create({ baseURL: '/api' });
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5000,
      refetchInterval: 10000,
      retry: 1,
    },
  },
});

// Cases
export const getCases = () => api.get('/cases').then(r => r.data);
export const getCase = (id: string) => api.get(`/cases/${id}`).then(r => r.data);
export const getCaseStats = (id: string) => api.get(`/cases/${id}/stats`).then(r => r.data);
export const createCase = (data: { name: string; description?: string }) =>
  api.post('/cases', data).then(r => r.data);
export const deleteCase = (id: string) => api.delete(`/cases/${id}`);

// Evidence
export const getEvidence = (caseId: string) =>
  api.get(`/cases/${caseId}/evidence`).then(r => r.data);
export const submitTextEvidence = (caseId: string, data: { content: string; source?: string }) =>
  api.post(`/cases/${caseId}/evidence/text`, data).then(r => r.data);

// Entities
export const getEntities = (caseId: string) =>
  api.get(`/cases/${caseId}/entities`).then(r => r.data);

// Hypotheses
export const getHypotheses = (caseId: string) =>
  api.get(`/cases/${caseId}/hypotheses`).then(r => r.data);
export const getHypothesisEvolution = (hypId: string) =>
  api.get(`/hypotheses/${hypId}/evolution`).then(r => r.data);
export const generateHypotheses = (caseId: string) =>
  api.post(`/cases/${caseId}/hypotheses/generate`).then(r => r.data);
export const evaluateHypotheses = (caseId: string) =>
  api.post(`/cases/${caseId}/hypotheses/evaluate`).then(r => r.data);

// Graph
export const getGraph = (caseId: string) =>
  api.get(`/cases/${caseId}/graph`).then(r => r.data);
export const getGraphStats = (caseId: string) =>
  api.get(`/cases/${caseId}/graph/stats`).then(r => r.data);

// Analysis
export const triggerAnalysis = (caseId: string) =>
  api.post(`/cases/${caseId}/analyze`, { trigger: 'manual' }).then(r => r.data);
export const getAnalysisRuns = (caseId: string) =>
  api.get(`/cases/${caseId}/analysis-runs`).then(r => r.data);

// Investigation
export const startInvestigation = (caseId: string) =>
  api.post(`/cases/${caseId}/investigation/start`).then(r => r.data);
export const stopInvestigation = (caseId: string) =>
  api.post(`/cases/${caseId}/investigation/stop`).then(r => r.data);
export const getInvestigationStatus = (caseId: string) =>
  api.get(`/cases/${caseId}/investigation/status`).then(r => r.data);

// Monitoring
export const getMonitoringJobs = (caseId: string) =>
  api.get(`/cases/${caseId}/monitoring`).then(r => r.data);
export const getAlerts = (caseId: string) =>
  api.get(`/cases/${caseId}/alerts`).then(r => r.data);
export const getUnreadCount = (caseId: string) =>
  api.get(`/alerts/unread-count?case_id=${caseId}`).then(r => r.data);

// Audit
export const getAuditLog = (caseId: string, limit = 50) =>
  api.get(`/cases/${caseId}/audit?limit=${limit}`).then(r => r.data);

// Timeline
export const getTimeline = (caseId: string) =>
  api.get(`/cases/${caseId}/timeline`).then(r => r.data);

// Benchmark
export const runBenchmark = (data: { models?: string[] }) =>
  api.post('/benchmark/run', data).then(r => r.data);
export const getBenchmarkResults = () =>
  api.get('/benchmark/results').then(r => r.data);

// Suspects
export const getSuspects = (caseId: string) => api.get(`/cases/${caseId}/suspects`).then(r => r.data);
export const scoreAllSuspects = (caseId: string) => api.post(`/cases/${caseId}/suspects/score`).then(r => r.data);
export const evaluateSuspectProfile = (suspectId: string) => api.post(`/suspects/${suspectId}/evaluate-profile`).then(r => r.data);
export const getSuspectEvolution = (suspectId: string) => api.get(`/suspects/${suspectId}/evolution`).then(r => r.data);
export const updateSuspect = (suspectId: string, data: Record<string, unknown>) => api.put(`/suspects/${suspectId}`, data).then(r => r.data);

// Health
export const getHealth = () => api.get('/health').then(r => r.data);
export const getSystemStats = () => api.get('/system/stats').then(r => r.data);
