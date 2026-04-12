/**
 * Sprint 9 Phase A (D6) — shell root with code-split routes.
 *
 * The six page components live behind `react-router` v6
 * `lazy:` entries so the main bundle only ships the shell
 * chrome (AppShell, CommandPalette, providers, shared
 * components) and each page is fetched on first navigation.
 * Each page module exports a named `Component` binding that
 * `lazy()` resolves — see `P12` in `docs/shell/PATTERNS.md`
 * for the convention.
 *
 * The `createBrowserRouter` factory is the 2025-canonical
 * React Router shape (`<BrowserRouter>` + `<Routes>` was
 * sunset in the v6 docs refresh alongside the CRA replacement
 * guide). It enables the `lazy` key per route and the future
 * `loader` / `action` surface without another migration.
 */

import { createBrowserRouter, Navigate, RouterProvider } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { TooltipProvider } from "@/components/ui/tooltip";

import { AppShell } from "@/components/AppShell";

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

const router = createBrowserRouter([
  {
    element: <AppShell />,
    children: [
      { index: true, element: <Navigate to="/my-projects" replace /> },
      {
        path: "/my-projects",
        lazy: () => import("@/pages/Projects"),
      },
      {
        path: "/project/:name",
        lazy: () => import("@/pages/ProjectDetail"),
      },
      {
        path: "/my-network",
        lazy: () => import("@/pages/Network"),
      },
      {
        path: "/browse",
        lazy: () => import("@/pages/Browse"),
      },
      {
        path: "/curators",
        lazy: () => import("@/pages/Curators"),
      },
      {
        path: "/app/:appName/tabs/:tabName",
        lazy: () => import("@/pages/AppTabPage"),
      },
    ],
  },
]);

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <RouterProvider router={router} />
      </TooltipProvider>
    </QueryClientProvider>
  );
}
