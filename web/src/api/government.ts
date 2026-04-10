import { api } from './client';

// Stats
export const getGovStats = () => api.get('/government/stats').then(r => r.data);

// Politicians
export const getGovPoliticians = (params?: Record<string, string>) =>
  api.get('/government/politicians', { params }).then(r => r.data);
export const getGovPolitician = (id: string) =>
  api.get(`/government/politicians/${id}`).then(r => r.data);
export const createGovPolitician = (data: Record<string, unknown>) =>
  api.post('/government/politicians', data).then(r => r.data);
export const updateGovPolitician = (id: string, data: Record<string, unknown>) =>
  api.put(`/government/politicians/${id}`, data).then(r => r.data);
export const searchGovPoliticians = (q: string) =>
  api.get('/government/politicians/search', { params: { q } }).then(r => r.data);

// Positions
export const getGovPositions = (politicianId: string, params?: Record<string, string>) =>
  api.get(`/government/politicians/${politicianId}/positions`, { params }).then(r => r.data);
export const createGovPosition = (data: Record<string, unknown>) =>
  api.post('/government/positions', data).then(r => r.data);

// Contradictions
export const getGovPoliticianContradictions = (id: string) =>
  api.get(`/government/politicians/${id}/contradictions`).then(r => r.data);
export const getGovContradictions = (params?: Record<string, string>) =>
  api.get('/government/contradictions', { params }).then(r => r.data);

// Scan — start / stop / status
export const triggerGovScan = () => api.post('/government/scan').then(r => r.data);
export const stopGovScan = () => api.delete('/government/scan').then(r => r.data);
export const getGovScanStatus = () => api.get('/government/scan/status').then(r => r.data);
export const getGovScanLog = () => api.get('/government/scans').then(r => r.data);

// Detect contradictions — start / stop / status
export const detectGovContradictions = () =>
  api.post('/government/detect-contradictions').then(r => r.data);
export const stopGovDetection = () =>
  api.delete('/government/detect-contradictions').then(r => r.data);
export const getGovDetectionStatus = () =>
  api.get('/government/detect-contradictions/status').then(r => r.data);

// Subjects
export const getGovSubjects = () => api.get('/government/subjects').then(r => r.data);

// Graph
export const getGovGraph = (params?: Record<string, string>) =>
  api.get('/government/graph', { params: { ...params, min_positions: '5', max_nodes: '150', max_edges: '500' } }).then(r => r.data);
export const getGovPoliticianGraph = (id: string) =>
  api.get(`/government/graph/politician/${id}`).then(r => r.data);
export const getGovSubjectGraph = (subject: string) =>
  api.get(`/government/graph/subject/${encodeURIComponent(subject)}`).then(r => r.data);

// Social Media
export const getGovSocial = (politicianId: string, params?: Record<string, string>) =>
  api.get(`/government/politicians/${politicianId}/social`, { params }).then(r => r.data);
export const getGovAllSocial = (params?: Record<string, string>) =>
  api.get('/government/social', { params }).then(r => r.data);

// Transcriptions
export const getGovTranscriptions = (politicianId: string) =>
  api.get(`/government/politicians/${politicianId}/transcriptions`).then(r => r.data);
export const getGovAllTranscriptions = (params?: Record<string, string>) =>
  api.get('/government/transcriptions', { params }).then(r => r.data);

// Alerts
export const getGovAlerts = (params?: Record<string, string>) =>
  api.get('/government/alerts', { params }).then(r => r.data);
export const markGovAlertRead = (alertId: string) =>
  api.put(`/government/alerts/${alertId}/read`).then(r => r.data);

// Press
export const getGovPress = (params?: Record<string, string>) =>
  api.get('/government/press', { params }).then(r => r.data);
export const getGovPressByPolitician = (politicianId: string) =>
  api.get(`/government/politicians/${politicianId}/press`).then(r => r.data);

// Affairs
export const getGovAffairs = (params?: Record<string, string>) =>
  api.get('/government/affairs', { params }).then(r => r.data);
export const getGovAffairsByPolitician = (politicianId: string) =>
  api.get(`/government/politicians/${politicianId}/affairs`).then(r => r.data);

// Laws
export const getGovLaws = (params?: Record<string, string>) =>
  api.get('/government/laws', { params }).then(r => r.data);

// Declarations
export const getGovDeclarations = (politicianId: string) =>
  api.get(`/government/politicians/${politicianId}/declarations`).then(r => r.data);

// Factchecks
export const getGovFactchecks = (params?: Record<string, string>) =>
  api.get('/government/factchecks', { params }).then(r => r.data);
export const getGovFactchecksByPolitician = (politicianId: string) =>
  api.get(`/government/politicians/${politicianId}/factchecks`).then(r => r.data);

// Search (RAG)
export const searchGov = (q: string, limit?: number) =>
  api.get('/government/search', { params: { q, limit } }).then(r => r.data);
export const askGov = (q: string) =>
  api.get('/government/ask', { params: { q } }).then(r => r.data);

// Pipeline
export const getGovWorkers = () => api.get('/government/workers').then(r => r.data);
export const getGovPipeline = () => api.get('/government/pipeline').then(r => r.data);
