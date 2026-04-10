import { useEffect, useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
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
  Landmark,
  Vote,
  Scale,
  Waypoints,
  Zap,
  Square,
} from 'lucide-react';
import {
  CommandDialog,
  Command,
  CommandInput,
  CommandList,
  CommandEmpty,
  CommandGroup,
  CommandItem,
  CommandSeparator,
  CommandShortcut,
} from '@/components/ui/command';
import { useCases, useTriggerAnalysis } from '../hooks/useApi';
import { useCaseStore } from '../stores/caseStore';

// ── Navigation commands ──────────────────────────────────────

const navCommands = [
  { label: 'Dashboard', to: '/', icon: LayoutDashboard, group: 'Navigation' },
  { label: 'Evidence / Preuves', to: '/evidence', icon: FileText, group: 'Navigation' },
  { label: 'Entites', to: '/entities', icon: Users, group: 'Navigation' },
  { label: 'Hypotheses', to: '/hypotheses', icon: Lightbulb, group: 'Navigation' },
  { label: 'Suspects', to: '/suspects', icon: UserSearch, group: 'Navigation' },
  { label: 'Graphe', to: '/graph', icon: Network, group: 'Navigation' },
  { label: 'Chronologie / Timeline', to: '/timeline', icon: Clock, group: 'Navigation' },
  { label: 'Investigation', to: '/investigation', icon: Search, group: 'Navigation' },
  { label: 'Image Search', to: '/images', icon: Image, group: 'Navigation' },
  { label: 'Wiki', to: '/wiki', icon: BookOpen, group: 'Navigation' },
  { label: 'Reports', to: '/reports', icon: FileOutput, group: 'Navigation' },
  { label: 'Benchmark', to: '/benchmark', icon: Gauge, group: 'Navigation' },
];

const govCommands = [
  { label: 'Gouvernement - Politiciens', to: '/government', icon: Landmark, group: 'Gouvernement' },
  { label: 'Hemicycle / Votes', to: '/government?tab=votes', icon: Vote, group: 'Gouvernement' },
  { label: 'Lois / Legislation', to: '/government?tab=laws', icon: Scale, group: 'Gouvernement' },
  { label: 'Reseau d\'influence', to: '/government?tab=network', icon: Waypoints, group: 'Gouvernement' },
];

// ── Component ─────────────────────────────────────────────────

export function CommandPalette() {
  const [open, setOpen] = useState(false);
  const navigate = useNavigate();
  const casesQuery = useCases();
  const { setCaseId } = useCaseStore();
  const triggerAnalysis = useTriggerAnalysis();

  const cases = Array.isArray(casesQuery.data) ? casesQuery.data : [];

  // Ctrl+K / Cmd+K to open
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setOpen(prev => !prev);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  const runCommand = useCallback((fn: () => void) => {
    setOpen(false);
    fn();
  }, []);

  return (
    <CommandDialog open={open} onOpenChange={setOpen} title="Recherche NEXUS" description="Naviguer, changer de case, lancer une action...">
      <Command>
        <CommandInput placeholder="Rechercher une page, case, action..." />
        <CommandList>
          <CommandEmpty>Aucun resultat.</CommandEmpty>

          {/* Navigation */}
          <CommandGroup heading="Pages">
            {navCommands.map(({ label, to, icon: Icon }) => (
              <CommandItem key={to} onSelect={() => runCommand(() => navigate(to))}>
                <Icon size={14} className="text-[var(--text-muted)]" />
                <span>{label}</span>
              </CommandItem>
            ))}
          </CommandGroup>

          <CommandSeparator />

          {/* Government */}
          <CommandGroup heading="Gouvernement">
            {govCommands.map(({ label, to, icon: Icon }) => (
              <CommandItem key={to} onSelect={() => runCommand(() => navigate(to))}>
                <Icon size={14} className="text-cyan-400" />
                <span>{label}</span>
              </CommandItem>
            ))}
          </CommandGroup>

          {/* Cases */}
          {cases.length > 0 && (
            <>
              <CommandSeparator />
              <CommandGroup heading="Cases">
                {cases.map((c: { id?: string; case_id?: string; name: string }) => (
                  <CommandItem
                    key={c.id || c.case_id}
                    onSelect={() => runCommand(() => {
                      setCaseId((c.id || c.case_id)!, c.name);
                      navigate('/');
                    })}
                  >
                    <FileText size={14} className="text-[var(--text-muted)]" />
                    <span>Case: {c.name}</span>
                  </CommandItem>
                ))}
              </CommandGroup>
            </>
          )}

          <CommandSeparator />

          {/* Actions */}
          <CommandGroup heading="Actions">
            <CommandItem onSelect={() => runCommand(() => triggerAnalysis.mutate())}>
              <Zap size={14} className="text-blue-400" />
              <span>Lancer une analyse</span>
              <CommandShortcut>Analyse</CommandShortcut>
            </CommandItem>
          </CommandGroup>
        </CommandList>
      </Command>
    </CommandDialog>
  );
}
