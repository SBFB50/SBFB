import { api } from './client';

// Stats
export const getComputeStats = () => api.get('/compute/stats').then(r => r.data);

// Nodes
export const getComputeNodes = (status?: string) =>
  api.get('/compute/nodes', { params: status ? { status } : {} }).then(r => r.data);

// Leaderboard
export const getComputeLeaderboard = (limit: number = 20) =>
  api.get('/compute/leaderboard', { params: { limit } }).then(r => r.data);

// Model status
export const getComputeModelStatus = () => api.get('/compute/model/status').then(r => r.data);

// Model assignments
export const getComputeModelAssignments = () => api.get('/compute/model/assignments').then(r => r.data);

// Model transitions
export const getComputeModelTransitions = (limit: number = 20) =>
  api.get('/compute/model/transitions', { params: { limit } }).then(r => r.data);

// Hybrid status
export const getComputeHybridStatus = () => api.get('/compute/hybrid/status').then(r => r.data);

// Swarm (Petals)
export const getComputeSwarmStatus = () => api.get('/compute/swarm/status').then(r => r.data);

// Test tasks
export const submitTestTask = (taskType: string, prompt: string, priority: number = 5) =>
  api.post('/compute/tasks', { task_type: taskType, prompt, priority }).then(r => r.data);

// Self-worker control
export const getSelfWorkerStatus = () => api.get('/compute/self-worker/status').then(r => r.data);
export const pauseSelfWorker = () => api.post('/compute/self-worker/pause').then(r => r.data);
export const resumeSelfWorker = () => api.post('/compute/self-worker/resume').then(r => r.data);

// Health + Uptime + Impact (Phase 8)
export const getComputeHealth = () => api.get('/compute/health').then(r => r.data);
export const getComputeUptime = () => api.get('/compute/uptime').then(r => r.data);
export const getNodeImpact = (nodeId: string) =>
  api.get(`/compute/nodes/${nodeId}/impact`).then(r => r.data);

// Badges
export const getComputeBadges = (nodeId?: string) =>
  api.get('/compute/badges', { params: nodeId ? { node_id: nodeId } : {} }).then(r => r.data);
