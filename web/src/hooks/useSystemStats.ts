import { useEffect } from 'react';
import { useSystemStore } from '../stores/systemStore';
import { getHealth, getSystemStats } from '../api/client';

export function useSystemStats() {
  const { stats, healthy, setStats, setHealthy } = useSystemStore();

  useEffect(() => {
    let active = true;

    const poll = async () => {
      try {
        const healthData = await getHealth();
        if (active) setHealthy(healthData?.status === 'ok' || healthData?.status === 'healthy');
      } catch {
        if (active) setHealthy(false);
      }

      try {
        const sysData = await getSystemStats();
        if (active && sysData) setStats(sysData);
      } catch {
        // System stats endpoint may not exist yet
      }
    };

    poll();
    const interval = setInterval(poll, 5000);
    return () => {
      active = false;
      clearInterval(interval);
    };
  }, [setStats, setHealthy]);

  return { stats, healthy };
}
