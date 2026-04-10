import { NavLink, useLocation, useNavigate } from 'react-router-dom';
import {
  LayoutDashboard,
  FileText,
  Users,
  Lightbulb,
  Network,
  Clock,
  Search,
  UserSearch,
  Gauge,
  BookOpen,
  FileOutput,
  Image,
  ChevronDown,
  Plus,
  Cpu,
  HardDrive,
  Thermometer,
  Landmark,
  Vote,
  Scale,
  Waypoints,
  Wrench,
  Activity,
  CircleDot,
  Container,
  type LucideIcon,
} from 'lucide-react';
import { useState } from 'react';
import { useCaseStore } from '../stores/caseStore';
import { useCases, useUnreadCount, useCreateCase, useEvidence, useEntities, useHypotheses } from '../hooks/useApi';
import { useSystemStats } from '../hooks/useSystemStats';

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
  SidebarSeparator,
} from '@/components/ui/sidebar';

// ── Navigation structure ──────────────────────────────────────

interface NavItem {
  to: string;
  icon: LucideIcon;
  label: string;
  badgeKey?: 'evidence' | 'entities' | 'hypotheses' | 'unread';
}

const investigationItems: NavItem[] = [
  { to: '/', icon: LayoutDashboard, label: 'Dashboard' },
  { to: '/evidence', icon: FileText, label: 'Evidence', badgeKey: 'evidence' },
  { to: '/entities', icon: Users, label: 'Entites', badgeKey: 'entities' },
  { to: '/hypotheses', icon: Lightbulb, label: 'Hypotheses', badgeKey: 'hypotheses' },
  { to: '/suspects', icon: UserSearch, label: 'Suspects' },
  { to: '/graph', icon: Network, label: 'Graphe' },
  { to: '/timeline', icon: Clock, label: 'Chronologie' },
  { to: '/investigation', icon: Search, label: 'Investigation', badgeKey: 'unread' },
];

const govItems: NavItem[] = [
  { to: '/government', icon: Landmark, label: 'Politiciens' },
  { to: '/government?tab=votes', icon: Vote, label: 'Hemicycle / Votes' },
  { to: '/government?tab=laws', icon: Scale, label: 'Lois' },
  { to: '/government?tab=network', icon: Waypoints, label: 'Reseau' },
];

const computeItems: NavItem[] = [
  { to: '/network', icon: Cpu, label: 'Reseau GPU' },
  { to: '/network?tab=leaderboard', icon: Activity, label: 'Leaderboard' },
];

const toolItems: NavItem[] = [
  { to: '/images', icon: Image, label: 'Image Search' },
  { to: '/wiki', icon: BookOpen, label: 'Wiki' },
  { to: '/reports', icon: FileOutput, label: 'Reports' },
  { to: '/benchmark', icon: Gauge, label: 'Benchmark' },
];

// ── Badge counts hook ─────────────────────────────────────────

function useBadgeCounts() {
  const evidence = useEvidence();
  const entities = useEntities();
  const hypotheses = useHypotheses();
  const unread = useUnreadCount();

  const unreadVal = typeof unread.data === 'number'
    ? unread.data
    : (unread.data?.count ?? 0);

  return {
    evidence: Array.isArray(evidence.data) ? evidence.data.length : 0,
    entities: Array.isArray(entities.data) ? entities.data.length : 0,
    hypotheses: Array.isArray(hypotheses.data) ? hypotheses.data.length : 0,
    unread: unreadVal as number,
  };
}

// ── Main component ────────────────────────────────────────────

export function AppSidebar() {
  const location = useLocation();
  const navigate = useNavigate();
  const { caseId, caseName, setCaseId } = useCaseStore();
  const casesQuery = useCases();
  const createCase = useCreateCase();
  const { stats, healthy } = useSystemStats();
  const badges = useBadgeCounts();

  const [caseDropdownOpen, setCaseDropdownOpen] = useState(false);
  const [newCaseName, setNewCaseName] = useState('');
  const [showNewCase, setShowNewCase] = useState(false);

  const cases = Array.isArray(casesQuery.data) ? casesQuery.data : [];

  const handleCreateCase = () => {
    if (!newCaseName.trim()) return;
    createCase.mutate({ name: newCaseName.trim() }, {
      onSuccess: (data) => {
        setCaseId(data.id || data.case_id, newCaseName.trim());
        setNewCaseName('');
        setShowNewCase(false);
      },
    });
  };

  return (
    <Sidebar collapsible="icon" className="border-r border-[var(--border)]">

      {/* ── Header: Logo + Health ── */}
      <SidebarHeader className="px-3 py-3">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-lg bg-blue-600 flex items-center justify-center shrink-0">
            <span className="text-white font-bold text-sm">N</span>
          </div>
          <div className="flex-1 min-w-0 group-data-[collapsible=icon]:hidden">
            <h1 className="text-sm font-bold text-[var(--text-primary)] tracking-wide">NEXUS</h1>
            <p className="text-[9px] text-[var(--text-muted)] uppercase tracking-widest">Cold Case Intel</p>
          </div>
          <div
            className={`w-2 h-2 rounded-full shrink-0 ${healthy ? 'bg-emerald-500' : 'bg-red-500'}`}
            title={healthy ? 'API Online' : 'API Offline'}
          />
        </div>
      </SidebarHeader>

      {/* ── Case Selector ── */}
      <div className="px-3 pb-2 group-data-[collapsible=icon]:hidden">
        <div className="relative">
          <button
            onClick={() => setCaseDropdownOpen(!caseDropdownOpen)}
            className="w-full flex items-center justify-between px-2.5 py-1.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-md text-xs text-[var(--text-primary)] hover:border-blue-500/50 transition-colors"
          >
            <span className="truncate">{caseName || 'Select case...'}</span>
            <ChevronDown size={12} className={`shrink-0 transition-transform ${caseDropdownOpen ? 'rotate-180' : ''}`} />
          </button>
          {caseDropdownOpen && (
            <div className="absolute z-50 top-full left-0 right-0 mt-1 bg-[var(--bg-card)] border border-[var(--border)] rounded-md shadow-xl overflow-hidden">
              {cases.map((c: { id?: string; case_id?: string; name: string }) => (
                <button
                  key={c.id || c.case_id}
                  onClick={() => {
                    setCaseId((c.id || c.case_id)!, c.name);
                    setCaseDropdownOpen(false);
                  }}
                  className={`w-full text-left px-2.5 py-1.5 text-xs hover:bg-[var(--bg-hover)] transition-colors ${
                    (c.id || c.case_id) === caseId ? 'text-blue-400' : 'text-[var(--text-secondary)]'
                  }`}
                >
                  {c.name}
                </button>
              ))}
              {cases.length === 0 && (
                <p className="px-2.5 py-1.5 text-[10px] text-[var(--text-muted)]">No cases yet</p>
              )}
              <button
                onClick={() => { setShowNewCase(true); setCaseDropdownOpen(false); }}
                className="w-full flex items-center gap-1.5 px-2.5 py-1.5 text-xs text-blue-400 hover:bg-[var(--bg-hover)] border-t border-[var(--border)]"
              >
                <Plus size={12} /> New Case
              </button>
            </div>
          )}
        </div>
        {showNewCase && (
          <div className="mt-1.5 flex gap-1">
            <input
              type="text"
              placeholder="Case name..."
              value={newCaseName}
              onChange={e => setNewCaseName(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && handleCreateCase()}
              className="flex-1 px-2 py-1 bg-[var(--bg-primary)] border border-[var(--border)] rounded text-[11px] text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-blue-500"
              autoFocus
            />
            <button onClick={handleCreateCase} className="px-2 py-1 bg-blue-600 text-white rounded text-[11px]">
              OK
            </button>
          </div>
        )}
      </div>

      {/* ── Command palette hint ── */}
      <div className="px-3 pb-1 group-data-[collapsible=icon]:hidden">
        <button
          onClick={() => {
            window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', metaKey: true }));
          }}
          className="w-full flex items-center gap-2 px-2.5 py-1.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-md text-[11px] text-[var(--text-muted)] hover:border-blue-500/30 transition-colors"
        >
          <Search size={12} />
          <span>Rechercher...</span>
          <kbd className="ml-auto text-[9px] bg-[var(--bg-hover)] px-1 py-0.5 rounded font-mono">Ctrl+K</kbd>
        </button>
      </div>

      <SidebarSeparator className="my-1" />

      {/* ── Navigation ── */}
      <SidebarContent>

        {/* Investigation */}
        <SidebarGroup>
          <SidebarGroupLabel className="text-[10px] uppercase tracking-wider text-[var(--text-muted)] font-semibold">
            Investigation
          </SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {investigationItems.map(({ to, icon: Icon, label, badgeKey }) => {
                const isActive = to === '/' ? location.pathname === '/' : location.pathname.startsWith(to);
                const count = badgeKey ? badges[badgeKey] : 0;
                return (
                  <SidebarMenuItem key={to}>
                    <SidebarMenuButton
                      isActive={isActive}
                      tooltip={label}
                      render={<NavLink to={to} />}
                    >
                      <Icon size={16} />
                      <span>{label}</span>
                    </SidebarMenuButton>
                    {badgeKey && count > 0 && (
                      <SidebarMenuBadge className={badgeKey === 'unread' ? 'bg-red-500/20 text-red-400 text-[10px]' : 'text-[10px] text-[var(--text-muted)]'}>
                        {count > 999 ? '999+' : count}
                      </SidebarMenuBadge>
                    )}
                  </SidebarMenuItem>
                );
              })}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        {/* Gouvernement */}
        <SidebarGroup>
          <SidebarGroupLabel className="text-[10px] uppercase tracking-wider text-cyan-400/70 font-semibold">
            Gouvernement
          </SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {govItems.map(({ to, icon: Icon, label }) => {
                const isActive = location.pathname + location.search === to ||
                  (to === '/government' && location.pathname === '/government' && !location.search);
                return (
                  <SidebarMenuItem key={to}>
                    <SidebarMenuButton
                      isActive={isActive}
                      tooltip={label}
                      render={<NavLink to={to} />}
                    >
                      <Icon size={16} />
                      <span>{label}</span>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                );
              })}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        {/* Reseau GPU */}
        <SidebarGroup>
          <SidebarGroupLabel className="text-[10px] uppercase tracking-wider text-emerald-400/70 font-semibold">
            Puissance Citoyenne
          </SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {computeItems.map(({ to, icon: Icon, label }) => {
                const isActive = location.pathname + location.search === to ||
                  (to === '/network' && location.pathname === '/network' && !location.search);
                return (
                  <SidebarMenuItem key={to}>
                    <SidebarMenuButton
                      isActive={isActive}
                      tooltip={label}
                      render={<NavLink to={to} />}
                    >
                      <Icon size={16} />
                      <span>{label}</span>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                );
              })}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        {/* Outils */}
        <SidebarGroup>
          <SidebarGroupLabel className="text-[10px] uppercase tracking-wider text-[var(--text-muted)] font-semibold">
            Outils
          </SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {toolItems.map(({ to, icon: Icon, label }) => {
                const isActive = location.pathname.startsWith(to);
                return (
                  <SidebarMenuItem key={to}>
                    <SidebarMenuButton
                      isActive={isActive}
                      tooltip={label}
                      render={<NavLink to={to} />}
                    >
                      <Icon size={16} />
                      <span>{label}</span>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                );
              })}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      {/* ── Footer: System stats ── */}
      <SidebarFooter className="px-3 py-2">
        <div className="group-data-[collapsible=icon]:hidden space-y-1.5">
          <p className="text-[9px] font-semibold text-[var(--text-muted)] uppercase tracking-wider px-0.5">Systeme</p>
          {stats ? (
            <>
              <StatBar icon={Cpu} label="CPU" value={stats.cpu_percent} />
              <StatBar icon={HardDrive} label="RAM" value={stats.ram_percent}
                detail={`${stats.ram_used_gb?.toFixed(1)}/${stats.ram_total_gb?.toFixed(0)}G`} />
              <StatBar icon={Thermometer} label="GPU" value={stats.gpu_percent}
                detail={`${stats.gpu_memory_used_gb?.toFixed(1)}/${stats.gpu_memory_total_gb?.toFixed(0)}G`} />
            </>
          ) : (
            <div className="flex items-center gap-1.5 px-0.5">
              <CircleDot size={10} className={healthy ? 'text-emerald-500' : 'text-red-500'} />
              <span className="text-[10px] text-[var(--text-muted)]">{healthy ? 'API Connected' : 'API Offline'}</span>
            </div>
          )}

          {/* Docker status */}
          <div className="flex items-center gap-1.5 px-0.5 pt-1">
            <Container size={10} className="text-[var(--text-muted)]" />
            <span className="text-[10px] text-[var(--text-muted)]">Neo4j</span>
            <div className={`w-1.5 h-1.5 rounded-full ${healthy ? 'bg-emerald-500' : 'bg-zinc-600'}`} />
            <span className="text-[10px] text-[var(--text-muted)] ml-1">Chroma</span>
            <div className={`w-1.5 h-1.5 rounded-full ${healthy ? 'bg-emerald-500' : 'bg-zinc-600'}`} />
          </div>
        </div>

        {/* Collapsed: just icons */}
        <div className="hidden group-data-[collapsible=icon]:flex flex-col items-center gap-1">
          <div className={`w-2 h-2 rounded-full ${healthy ? 'bg-emerald-500' : 'bg-red-500'}`} title={healthy ? 'Online' : 'Offline'} />
          {stats && (
            <div
              className={`w-2 h-2 rounded-full ${stats.gpu_percent > 80 ? 'bg-red-500' : stats.gpu_percent > 50 ? 'bg-yellow-500' : 'bg-emerald-500'}`}
              title={`GPU ${Math.round(stats.gpu_percent)}%`}
            />
          )}
        </div>
      </SidebarFooter>

      <SidebarRail />
    </Sidebar>
  );
}

// ── Stat bar sub-component ────────────────────────────────────

function StatBar({ icon: Icon, label, value, detail }: {
  icon: LucideIcon;
  label: string;
  value: number;
  detail?: string;
}) {
  const color = value > 80 ? '#ef4444' : value > 50 ? '#eab308' : '#22c55e';
  return (
    <div className="px-0.5">
      <div className="flex items-center justify-between mb-0.5">
        <div className="flex items-center gap-1">
          <Icon size={10} className="text-[var(--text-muted)]" />
          <span className="text-[10px] text-[var(--text-secondary)]">{label}</span>
        </div>
        <span className="text-[10px] font-mono" style={{ color }}>
          {Math.round(value)}%
          {detail && <span className="text-[var(--text-muted)] ml-0.5 text-[9px]">{detail}</span>}
        </span>
      </div>
      <div className="w-full h-0.5 bg-[var(--bg-primary)] rounded-full overflow-hidden">
        <div className="h-full rounded-full transition-all duration-700" style={{ width: `${value}%`, backgroundColor: color }} />
      </div>
    </div>
  );
}
