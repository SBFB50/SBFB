/**
 * Sprint 5 shell root — six flat routes under a single shell layout.
 *
 * Routing stays declarative (`<BrowserRouter>` + `<Routes>`)
 * rather than `createBrowserRouter` — we do not use route loaders
 * because every page fetches its data through React Query
 * directly, and the flat layout keeps Sprint 6 Phase B
 * (`/project/:name`) and Sprint 8 Phase E
 * (`/app/:appName/tabs/:tabName`) additions trivial.
 */

import { Navigate, BrowserRouter, Route, Routes } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { TooltipProvider } from "@/components/ui/tooltip";

import { AppShell } from "@/components/AppShell";
import Projects from "@/pages/Projects";
import ProjectDetail from "@/pages/ProjectDetail";
import Network from "@/pages/Network";
import Browse from "@/pages/Browse";
import Curators from "@/pages/Curators";
import AppTabPage from "@/pages/AppTabPage";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Coordinator data is extremely dynamic but the shell is
      // local-only, so 5s is a reasonable default that we override
      // per-query where a tighter polling loop is needed
      // (e.g. /worker-state in Phase C).
      staleTime: 5_000,
      retry: 1,
    },
  },
});

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <BrowserRouter>
          <Routes>
            <Route element={<AppShell />}>
              <Route index element={<Navigate to="/my-projects" replace />} />
              <Route path="/my-projects" element={<Projects />} />
              <Route path="/project/:name" element={<ProjectDetail />} />
              <Route path="/my-network" element={<Network />} />
              <Route path="/browse" element={<Browse />} />
              <Route path="/curators" element={<Curators />} />
              <Route
                path="/app/:appName/tabs/:tabName"
                element={<AppTabPage />}
              />
            </Route>
          </Routes>
        </BrowserRouter>
      </TooltipProvider>
    </QueryClientProvider>
  );
}
