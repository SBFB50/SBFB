import { useQuery } from '@tanstack/react-query';
import * as compute from '../api/compute';

export function useComputeStats() {
  return useQuery({
    queryKey: ['compute-stats'],
    queryFn: compute.getComputeStats,
    refetchInterval: 30_000,
  });
}

export function useComputeNodes(status?: string) {
  return useQuery({
    queryKey: ['compute-nodes', status],
    queryFn: () => compute.getComputeNodes(status),
    refetchInterval: 30_000,
  });
}

export function useComputeLeaderboard(limit: number = 20) {
  return useQuery({
    queryKey: ['compute-leaderboard', limit],
    queryFn: () => compute.getComputeLeaderboard(limit),
    refetchInterval: 60_000,
  });
}

export function useComputeModelStatus() {
  return useQuery({
    queryKey: ['compute-model-status'],
    queryFn: compute.getComputeModelStatus,
    refetchInterval: 30_000,
  });
}

export function useComputeHybridStatus() {
  return useQuery({
    queryKey: ['compute-hybrid-status'],
    queryFn: compute.getComputeHybridStatus,
    refetchInterval: 30_000,
  });
}

export function useComputeSwarmStatus() {
  return useQuery({
    queryKey: ['compute-swarm-status'],
    queryFn: compute.getComputeSwarmStatus,
    refetchInterval: 30_000,
  });
}

export function useSelfWorkerStatus() {
  return useQuery({
    queryKey: ['self-worker-status'],
    queryFn: compute.getSelfWorkerStatus,
    refetchInterval: 5_000,
  });
}

export function useComputeHealth() {
  return useQuery({
    queryKey: ['compute-health'],
    queryFn: compute.getComputeHealth,
    refetchInterval: 15_000,
  });
}

export function useComputeUptime() {
  return useQuery({
    queryKey: ['compute-uptime'],
    queryFn: compute.getComputeUptime,
    refetchInterval: 60_000,
  });
}

export function useNodeImpact(nodeId: string | null) {
  return useQuery({
    queryKey: ['node-impact', nodeId],
    queryFn: () => compute.getNodeImpact(nodeId!),
    enabled: !!nodeId,
    refetchInterval: 60_000,
  });
}

export function useComputeBadges(nodeId?: string) {
  return useQuery({
    queryKey: ['compute-badges', nodeId],
    queryFn: () => compute.getComputeBadges(nodeId),
    refetchInterval: 120_000,
  });
}
