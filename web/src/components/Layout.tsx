import { Outlet } from 'react-router-dom';
import { AppSidebar } from './AppSidebar';
import TopBar from './TopBar';
import ToastContainer from './Toast';
import { CommandPalette } from './CommandPalette';
import { SidebarInset, SidebarProvider } from '@/components/ui/sidebar';
import { useCaseSSE, useGovSSE } from '../hooks/useSSE';
import { useSystemStats } from '../hooks/useSystemStats';
import { Loader2 } from 'lucide-react';

function StartupBanner() {
  const { healthy } = useSystemStats();

  if (healthy) return null;

  return (
    <div className="flex items-center gap-3 px-4 py-2.5 bg-blue-600/10 border-b border-blue-500/20">
      <Loader2 size={14} className="animate-spin text-blue-400 shrink-0" />
      <span className="text-xs text-blue-300">
        Backend en cours de demarrage (Neo4j + ChromaDB + GLiNER)... Les donnees apparaitront automatiquement.
      </span>
    </div>
  );
}

export default function Layout() {
  // Establish SSE connections for real-time updates (app-wide)
  useCaseSSE();
  useGovSSE();

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset className="flex flex-col bg-[var(--bg-primary)]">
        <TopBar />
        <StartupBanner />
        <main className="flex-1 overflow-auto p-6">
          <Outlet />
        </main>
      </SidebarInset>
      <ToastContainer />
      <CommandPalette />
    </SidebarProvider>
  );
}
