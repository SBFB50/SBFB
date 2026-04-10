import { useState } from 'react';
import {
  Cpu, Server, Activity, Trophy, Award, Zap, Heart,
  Users, HardDrive, BarChart3, Wifi, Blocks,
} from 'lucide-react';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';
import { useComputeStats, useComputeLeaderboard, useComputeNodes, useComputeModelStatus, useComputeHybridStatus, useComputeSwarmStatus, useSelfWorkerStatus } from '../hooks/useCompute';
import { pauseSelfWorker, resumeSelfWorker } from '../api/compute';
import { StatsTab } from '../components/compute/StatsTab';
import { LeaderboardTab } from '../components/compute/LeaderboardTab';
import { NodesTab } from '../components/compute/NodesTab';
import { BadgesTab } from '../components/compute/BadgesTab';
import { SwarmTab } from '../components/compute/SwarmTab';
import { ContributeTab } from '../components/compute/ContributeTab';

function MetricCard({ label, value, icon: Icon, color }: {
  label: string; value: string | number; icon: typeof Cpu; color?: string;
}) {
  return (
    <div className="bg-[var(--bg-card)] border border-[var(--border)] rounded-lg p-3">
      <div className="flex items-center justify-between mb-1">
        <span className="text-[10px] uppercase tracking-wider text-[var(--text-muted)]">{label}</span>
        <Icon size={14} style={{ color: color || 'var(--text-muted)' }} />
      </div>
      <p className="text-xl font-bold" style={{ color: color || 'var(--text-primary)' }}>
        {typeof value === 'number' ? value.toLocaleString() : value}
      </p>
    </div>
  );
}

export default function NetworkPage() {
  const [activeTab, setActiveTab] = useState('stats');

  const statsQ = useComputeStats();
  const leaderboardQ = useComputeLeaderboard(20);
  const nodesQ = useComputeNodes();
  const modelQ = useComputeModelStatus();
  const hybridQ = useComputeHybridStatus();
  const swarmQ = useComputeSwarmStatus();
  const selfWorkerQ = useSelfWorkerStatus();

  const stats = statsQ.data || {};
  const selfWorker = selfWorkerQ.data || {};
  const leaderboard = leaderboardQ.data || { entries: [], total_contributors: 0 };
  const nodes = Array.isArray(nodesQ.data) ? nodesQ.data : [];
  const model = modelQ.data || {};
  const hybrid = hybridQ.data || {};
  const swarm = swarmQ.data || {};

  const nodesOnline = stats.nodes_online || 0;
  const vramTotal = stats.vram_total_gb || 0;
  const tasksToday = stats.tasks_today || 0;
  const currentModel = stats.current_model || '(aucun)';
  const modelTier = stats.model_tier || '';

  return (
    <div className="flex flex-col h-full gap-3">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="p-2.5 rounded-lg bg-emerald-500/10">
            <Cpu size={22} className="text-emerald-400" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-[var(--text-primary)]">
              Puissance Citoyenne
            </h2>
            <p className="text-xs text-[var(--text-muted)]">
              Reseau GPU distribue — les citoyens alimentent l'IA politique
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          {model.execution_mode === 'petals' && (
            <span className="px-2 py-0.5 rounded text-[10px] bg-fuchsia-500/20 text-fuchsia-400 border border-fuchsia-500/30">
              Petals Swarm ({swarm.coverage_pct?.toFixed(0) || 0}% blocs)
            </span>
          )}
          {model.execution_mode === 'distributed' && (
            <span className="px-2 py-0.5 rounded text-[10px] bg-purple-500/20 text-purple-400 border border-purple-500/30">
              Mode distribue (exo)
            </span>
          )}
          {model.transition_state === 'transitioning' && (
            <span className="px-2 py-0.5 rounded text-[10px] bg-yellow-500/20 text-yellow-400 border border-yellow-500/30 animate-pulse">
              Transition: {model.readiness_pct?.toFixed(0)}%
            </span>
          )}
        </div>
      </div>

      {/* Metrics */}
      <div className="grid grid-cols-2 xl:grid-cols-5 gap-3">
        <MetricCard label="Contributeurs" value={nodesOnline} icon={Users} color="#22c55e" />
        <MetricCard label="VRAM Totale" value={`${vramTotal.toFixed(0)} GB`} icon={HardDrive} color="#06b6d4" />
        <MetricCard label="Modele Actif" value={modelTier || currentModel.split('/').pop() || ''} icon={Zap} color="#a855f7" />
        <MetricCard label="Tasks Aujourd'hui" value={tasksToday} icon={Activity} color="#f59e0b" />
        <MetricCard label="Total Contributeurs" value={leaderboard.total_contributors || 0} icon={Server} color="#3b82f6" />
      </div>

      {/* Self-worker status + control */}
      <div className={`border rounded-lg px-4 py-2.5 flex items-center justify-between ${
        selfWorker.running && !selfWorker.paused
          ? 'bg-emerald-500/5 border-emerald-500/20'
          : selfWorker.paused
            ? 'bg-yellow-500/5 border-yellow-500/20'
            : 'bg-[var(--bg-card)] border-[var(--border)]'
      }`}>
        <div className="flex items-center gap-3">
          <div className={`w-2.5 h-2.5 rounded-full ${
            selfWorker.running && !selfWorker.paused ? 'bg-emerald-500 animate-pulse' :
            selfWorker.paused ? 'bg-yellow-500' : 'bg-gray-500'
          }`} />
          <div>
            <p className="text-xs font-medium text-[var(--text-primary)]">
              {selfWorker.running && !selfWorker.paused ? 'Votre GPU contribue au reseau' :
               selfWorker.paused ? 'GPU en pause' :
               selfWorker.gpu_model ? `GPU detecte : ${selfWorker.gpu_model}` : 'Aucun GPU detecte'}
            </p>
            <p className="text-[10px] text-[var(--text-muted)]">
              {selfWorker.running ? (
                <>
                  {selfWorker.gpu_model} ({(selfWorker.vram_mb / 1024).toFixed(0)} GB)
                  {' — '}{selfWorker.tasks_completed} taches
                  {selfWorker.last_tokens_per_sec > 0 && ` — ${selfWorker.last_tokens_per_sec.toFixed(1)} tok/s`}
                </>
              ) : 'Activez la contribution GPU pour alimenter l\'analyse politique'}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          {selfWorker.running && (
            <button
              onClick={() => selfWorker.paused ? resumeSelfWorker().then(() => selfWorkerQ.refetch()) : pauseSelfWorker().then(() => selfWorkerQ.refetch())}
              className={`px-3 py-1.5 rounded text-xs font-medium transition-colors ${
                selfWorker.paused
                  ? 'bg-emerald-600 hover:bg-emerald-500 text-white'
                  : 'bg-yellow-600/20 hover:bg-yellow-600/30 text-yellow-400 border border-yellow-600/30'
              }`}
            >
              {selfWorker.paused ? 'Reprendre' : 'Pause'}
            </button>
          )}
        </div>
      </div>

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1 flex flex-col min-h-0">
        <TabsList variant="line">
          <TabsTrigger value="stats"><BarChart3 size={14} /> Statistiques</TabsTrigger>
          <TabsTrigger value="leaderboard"><Trophy size={14} /> Leaderboard</TabsTrigger>
          <TabsTrigger value="nodes"><Server size={14} /> Nodes ({nodesOnline})</TabsTrigger>
          <TabsTrigger value="swarm"><Blocks size={14} /> Swarm Petals</TabsTrigger>
          <TabsTrigger value="contribute"><Heart size={14} /> Ma contribution</TabsTrigger>
          <TabsTrigger value="badges"><Award size={14} /> Badges</TabsTrigger>
        </TabsList>

        <TabsContent value="stats" className="flex-1 mt-3 overflow-auto">
          <StatsTab stats={stats} model={model} hybrid={hybrid} nodes={nodes} />
        </TabsContent>

        <TabsContent value="leaderboard" className="flex-1 mt-3 overflow-auto">
          <LeaderboardTab entries={leaderboard.entries || []} totalContributors={leaderboard.total_contributors || 0} />
        </TabsContent>

        <TabsContent value="nodes" className="flex-1 mt-3 overflow-auto">
          <NodesTab nodes={nodes} />
        </TabsContent>

        <TabsContent value="swarm" className="flex-1 mt-3 overflow-auto">
          <SwarmTab swarm={swarm} />
        </TabsContent>

        <TabsContent value="contribute" className="flex-1 mt-3 overflow-auto">
          <ContributeTab impact={null} loading={false} />
        </TabsContent>

        <TabsContent value="badges" className="flex-1 mt-3 overflow-auto">
          <BadgesTab />
        </TabsContent>
      </Tabs>
    </div>
  );
}
