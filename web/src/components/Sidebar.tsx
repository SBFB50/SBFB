import { NavLink } from 'react-router-dom';
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
  ChevronDown,
  Plus,
  Heart,
  Cpu,
  HardDrive,
  Thermometer,
} from 'lucide-react';
import { useState } from 'react';
import { useCaseStore } from '../stores/caseStore';
import { useCases, useUnreadCount, useCreateCase } from '../hooks/useApi';
import { useSystemStats } from '../hooks/useSystemStats';

const navItems = [
  { to: '/', icon: LayoutDashboard, label: 'Dashboard' },
  { to: '/evidence', icon: FileText, label: 'Evidence' },
  { to: '/entities', icon: Users, label: 'Entities' },
  { to: '/hypotheses', icon: Lightbulb, label: 'Hypotheses' },
  { to: '/graph', icon: Network, label: 'Graph' },
  { to: '/timeline', icon: Clock, label: 'Timeline' },
  { to: '/investigation', icon: Search, label: 'Investigation' },
  { to: '/suspects', icon: UserSearch, label: 'Suspects' },
  { to: '/benchmark', icon: Gauge, label: 'Benchmark' },
];

export default function Sidebar() {
  const { caseId, caseName, setCaseId } = useCaseStore();
  const casesQuery = useCases();
  const unreadQuery = useUnreadCount();
  const createCase = useCreateCase();
  const { stats, healthy } = useSystemStats();
  const [caseDropdownOpen, setCaseDropdownOpen] = useState(false);
  const [newCaseName, setNewCaseName] = useState('');
  const [showNewCase, setShowNewCase] = useState(false);

  const cases = Array.isArray(casesQuery.data) ? casesQuery.data : [];
  const unread = typeof unreadQuery.data === 'number'
    ? unreadQuery.data
    : (unreadQuery.data?.count ?? 0);

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
    <aside className="w-64 h-screen flex flex-col bg-[var(--bg-secondary)] border-r border-[var(--border)] shrink-0">
      {/* Logo */}
      <div className="px-5 py-4 border-b border-[var(--border)]">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-lg bg-[var(--accent)] flex items-center justify-center">
            <span className="text-white font-bold text-sm">N</span>
          </div>
          <div>
            <h1 className="text-base font-bold text-[var(--text-primary)] tracking-wide">NEXUS</h1>
            <p className="text-[10px] text-[var(--text-muted)] uppercase tracking-widest">Cold Case Intel</p>
          </div>
          <div className={`ml-auto w-2 h-2 rounded-full ${healthy ? 'bg-[var(--accent-green)]' : 'bg-[var(--accent-red)]'}`}
            title={healthy ? 'API Online' : 'API Offline'} />
        </div>
      </div>

      {/* Case Selector */}
      <div className="px-3 py-3 border-b border-[var(--border)]">
        <div className="relative">
          <button
            onClick={() => setCaseDropdownOpen(!caseDropdownOpen)}
            className="w-full flex items-center justify-between px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg text-sm text-[var(--text-primary)] hover:border-[var(--accent)] transition-colors"
          >
            <span className="truncate">{caseName || 'Select case...'}</span>
            <ChevronDown size={14} className={`shrink-0 transition-transform ${caseDropdownOpen ? 'rotate-180' : ''}`} />
          </button>
          {caseDropdownOpen && (
            <div className="absolute z-50 top-full left-0 right-0 mt-1 bg-[var(--bg-card)] border border-[var(--border)] rounded-lg shadow-lg overflow-hidden">
              {cases.map((c: { id?: string; case_id?: string; name: string }) => (
                <button
                  key={c.id || c.case_id}
                  onClick={() => {
                    setCaseId((c.id || c.case_id)!, c.name);
                    setCaseDropdownOpen(false);
                  }}
                  className={`w-full text-left px-3 py-2 text-sm hover:bg-[var(--bg-hover)] transition-colors ${
                    (c.id || c.case_id) === caseId ? 'text-[var(--accent)]' : 'text-[var(--text-secondary)]'
                  }`}
                >
                  {c.name}
                </button>
              ))}
              {cases.length === 0 && (
                <p className="px-3 py-2 text-xs text-[var(--text-muted)]">No cases yet</p>
              )}
              <button
                onClick={() => { setShowNewCase(true); setCaseDropdownOpen(false); }}
                className="w-full flex items-center gap-2 px-3 py-2 text-sm text-[var(--accent)] hover:bg-[var(--bg-hover)] border-t border-[var(--border)]"
              >
                <Plus size={14} /> New Case
              </button>
            </div>
          )}
        </div>
        {showNewCase && (
          <div className="mt-2 flex gap-1">
            <input
              type="text"
              placeholder="Case name..."
              value={newCaseName}
              onChange={e => setNewCaseName(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && handleCreateCase()}
              className="flex-1 px-2 py-1.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded text-xs text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)]"
              autoFocus
            />
            <button onClick={handleCreateCase} className="px-2 py-1.5 bg-[var(--accent)] text-white rounded text-xs">
              OK
            </button>
          </div>
        )}
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto px-3 py-3">
        <div className="space-y-0.5">
          {navItems.map(({ to, icon: Icon, label }) => (
            <NavLink
              key={to}
              to={to}
              className={({ isActive }) =>
                `flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                  isActive
                    ? 'bg-[var(--accent)]/10 text-[var(--accent)]'
                    : 'text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]'
                }`
              }
            >
              <Icon size={18} />
              <span>{label}</span>
              {label === 'Investigation' && unread > 0 && (
                <span className="ml-auto bg-[var(--accent-red)] text-white text-[10px] font-bold px-1.5 py-0.5 rounded-full">
                  {unread}
                </span>
              )}
            </NavLink>
          ))}
        </div>
      </nav>

      {/* System Stats */}
      <div className="px-3 py-3 border-t border-[var(--border)]">
        <p className="text-[10px] font-semibold text-[var(--text-muted)] uppercase tracking-wider mb-2 px-1">System</p>
        {stats ? (
          <div className="space-y-2">
            <StatBar icon={Cpu} label="CPU" value={stats.cpu_percent} />
            <StatBar icon={HardDrive} label="RAM" value={stats.ram_percent}
              detail={`${stats.ram_used_gb?.toFixed(1)}/${stats.ram_total_gb?.toFixed(0)}GB`} />
            <StatBar icon={Thermometer} label="GPU" value={stats.gpu_percent}
              detail={`${stats.gpu_memory_used_gb?.toFixed(1)}/${stats.gpu_memory_total_gb?.toFixed(0)}GB`} />
          </div>
        ) : (
          <div className="flex items-center gap-2 px-1">
            <Heart size={14} className={healthy ? 'text-[var(--accent-green)]' : 'text-[var(--accent-red)]'} />
            <span className="text-xs text-[var(--text-muted)]">
              {healthy ? 'API Connected' : 'API Offline'}
            </span>
          </div>
        )}
      </div>
    </aside>
  );
}

function StatBar({ icon: Icon, label, value, detail }: {
  icon: typeof Cpu;
  label: string;
  value: number;
  detail?: string;
}) {
  const color = value > 80 ? 'var(--accent-red)' : value > 50 ? 'var(--accent-yellow)' : 'var(--accent-green)';
  return (
    <div className="px-1">
      <div className="flex items-center justify-between mb-1">
        <div className="flex items-center gap-1.5">
          <Icon size={12} className="text-[var(--text-muted)]" />
          <span className="text-[11px] text-[var(--text-secondary)]">{label}</span>
        </div>
        <span className="text-[11px] font-mono" style={{ color }}>
          {Math.round(value)}%
          {detail && <span className="text-[var(--text-muted)] ml-1">{detail}</span>}
        </span>
      </div>
      <div className="w-full h-1 bg-[var(--bg-primary)] rounded-full overflow-hidden">
        <div className="h-full rounded-full transition-all duration-700" style={{ width: `${value}%`, backgroundColor: color }} />
      </div>
    </div>
  );
}
