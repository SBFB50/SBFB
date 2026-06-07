// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Netflix-style app shell — slim glassmorphism left rail + top bar.
 */

import { NavLink, Outlet, useLocation } from "react-router-dom";
import {
  FolderKanban,
  Cpu,
  Compass,
  BookmarkPlus,
  Rocket,
  Plus,
  Check,
  Trash2,
  ChevronsUpDown,
  Search,
  Zap,
} from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { lazy, Suspense, useState } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  selectActiveCoordinator,
  useProjectStore,
} from "@/stores/projectStore";
import { getHealth } from "@/api/coordinator";
import { AddCoordinatorDialog } from "@/components/AddCoordinatorDialog";
import { useCommandPalette } from "@/components/command-palette/useCommandPalette";
import { PanicWipeKeybind } from "@/components/PanicWipeKeybind";
import { RouteErrorBoundary } from "@/components/RouteErrorBoundary";
import { cn } from "@/lib/utils";

const CommandPalette = lazy(() =>
  import("@/components/command-palette/CommandPalette").then((mod) => ({
    default: mod.CommandPalette,
  })),
);

const IS_MAC =
  typeof navigator !== "undefined" &&
  /mac|iphone|ipad/i.test(navigator.platform || navigator.userAgent);

type NavEntry = {
  to: string;
  label: string;
  icon: typeof FolderKanban;
};

const NAV_ENTRIES: NavEntry[] = [
  { to: "/browse", label: "Explorer", icon: Compass },
  { to: "/my-projects", label: "Projets", icon: FolderKanban },
  { to: "/my-network", label: "Reseau", icon: Cpu },
  { to: "/curators", label: "Curators", icon: BookmarkPlus },
  { to: "/deploy", label: "Publier", icon: Rocket },
];

export function AppShell() {
  const [addDialogOpen, setAddDialogOpen] = useState(false);
  const palette = useCommandPalette();
  const location = useLocation();
  // Sprint 20 Phase B : the panic wipe keybind fires on
  // Ctrl+Shift+Alt+W x5 in 3s and POSTs to the daemon via the
  // active coordinator's /daemon/panic/wipe proxy. Null URL =
  // no coordinator picked yet = nothing to wipe, so skip mount.
  const activeCoordinatorUrl = useProjectStore(
    (s) => s.activeCoordinatorUrl,
  );

  // Full-screen pages hide the shell chrome
  const isFullScreen = location.pathname.startsWith("/browse/");

  if (isFullScreen) {
    return (
      <>
        <RouteErrorBoundary>
          <Outlet />
        </RouteErrorBoundary>
        <Suspense fallback={null}>
          <CommandPalette
            palette={palette}
            onAddCoordinator={() => setAddDialogOpen(true)}
          />
        </Suspense>
        <AddCoordinatorDialog
          open={addDialogOpen}
          onOpenChange={setAddDialogOpen}
        />
        {activeCoordinatorUrl && (
          <PanicWipeKeybind coordinatorBaseUrl={activeCoordinatorUrl} />
        )}
      </>
    );
  }

  return (
    <div className="flex min-h-screen bg-[#0a0a0f]">
      {/* ---- Left rail ---- */}
      <nav className="fixed left-0 top-0 z-40 flex h-screen w-[68px] flex-col items-center border-r border-white/[0.04] bg-black/40 py-4 backdrop-blur-xl">
        {/* Logo */}
        <div className="mb-6 flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-purple-600 to-indigo-600 shadow-lg shadow-purple-600/20">
          <Zap className="h-5 w-5 text-white" />
        </div>

        {/* Nav items */}
        <div className="flex flex-1 flex-col items-center gap-1">
          {NAV_ENTRIES.map(({ to, label, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              className={({ isActive }) =>
                cn(
                  "group relative flex h-11 w-11 items-center justify-center rounded-xl transition-all duration-200",
                  isActive
                    ? "bg-white/10 text-white shadow-lg shadow-white/5"
                    : "text-white/40 hover:bg-white/[0.06] hover:text-white/70",
                )
              }
              title={label}
            >
              {({ isActive }) => (
                <>
                  <Icon className="h-[18px] w-[18px]" />
                  {/* Active indicator */}
                  {isActive && (
                    <span className="absolute -left-[5px] top-1/2 h-5 w-[3px] -translate-y-1/2 rounded-r-full bg-purple-400" />
                  )}
                  {/* Tooltip */}
                  <span className="pointer-events-none absolute left-full ml-3 whitespace-nowrap rounded-lg bg-black/90 px-3 py-1.5 text-xs font-medium text-white opacity-0 shadow-lg backdrop-blur-sm transition-opacity group-hover:opacity-100">
                    {label}
                  </span>
                </>
              )}
            </NavLink>
          ))}
        </div>

        {/* Bottom: add coordinator */}
        <button
          onClick={() => setAddDialogOpen(true)}
          className="flex h-10 w-10 items-center justify-center rounded-xl text-white/30 transition-colors hover:bg-white/[0.06] hover:text-white/60"
          title="Se connecter a un noeud"
        >
          <Plus className="h-5 w-5" />
        </button>
      </nav>

      {/* ---- Main area ---- */}
      <div className="ml-[68px] flex flex-1 flex-col">
        {/* Top bar */}
        <header className="sticky top-0 z-30 flex h-14 items-center gap-3 border-b border-white/[0.04] bg-[#0a0a0f]/80 px-6 backdrop-blur-xl">
          <CoordinatorPicker onAddClick={() => setAddDialogOpen(true)} />

          <div className="ml-auto flex items-center gap-2">
            <button
              onClick={palette.toggle}
              className="flex items-center gap-2 rounded-lg bg-white/[0.04] px-3 py-1.5 text-xs text-white/40 transition-colors hover:bg-white/[0.08] hover:text-white/60"
              aria-label="Ouvrir la palette de commandes"
              data-testid="command-palette-trigger"
            >
              <Search className="h-3.5 w-3.5" />
              <span>Rechercher</span>
              <kbd className="ml-2 rounded border border-white/10 bg-white/[0.04] px-1.5 py-0.5 font-mono text-[10px]">
                {IS_MAC ? "\u2318" : "Ctrl"}K
              </kbd>
            </button>
          </div>
        </header>

        {/* Page content */}
        <main className="flex-1 overflow-auto p-6">
          <RouteErrorBoundary>
            <Outlet />
          </RouteErrorBoundary>
        </main>
      </div>

      <AddCoordinatorDialog
        open={addDialogOpen}
        onOpenChange={setAddDialogOpen}
      />

      <Suspense fallback={null}>
        <CommandPalette
          palette={palette}
          onAddCoordinator={() => setAddDialogOpen(true)}
        />
      </Suspense>

      {activeCoordinatorUrl && (
        <PanicWipeKeybind coordinatorBaseUrl={activeCoordinatorUrl} />
      )}
    </div>
  );
}

/**
 * Coordinator picker — glassmorphism style.
 */
function CoordinatorPicker({ onAddClick }: { onAddClick: () => void }) {
  const knownCoordinators = useProjectStore((s) => s.knownCoordinators);
  const activeCoordinatorUrl = useProjectStore((s) => s.activeCoordinatorUrl);
  const setActive = useProjectStore((s) => s.setActive);
  const removeCoordinator = useProjectStore((s) => s.removeCoordinator);
  const active = useProjectStore(selectActiveCoordinator);

  const healthQuery = useQuery({
    queryKey: ["health", activeCoordinatorUrl],
    queryFn: () => {
      if (!activeCoordinatorUrl) throw new Error("no active coordinator");
      return getHealth(activeCoordinatorUrl);
    },
    enabled: !!activeCoordinatorUrl,
    refetchInterval: 5000,
    retry: 0,
  });

  const healthy = healthQuery.isSuccess && healthQuery.data.status === "ok";
  const dotColor =
    activeCoordinatorUrl === null
      ? "bg-white/20"
      : healthQuery.isFetching && !healthQuery.data
        ? "bg-yellow-500"
        : healthy
          ? "bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.5)]"
          : "bg-red-400";

  if (knownCoordinators.length === 0) {
    return (
      <button
        onClick={onAddClick}
        className="flex items-center gap-2 text-sm text-white/40 hover:text-white/70"
      >
        <Plus className="h-4 w-4" />
        Se connecter a un noeud
      </button>
    );
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={
          <button className="flex items-center gap-2 rounded-lg px-2 py-1 text-sm transition-colors hover:bg-white/[0.06]" />
        }
      >
        <span className={cn("h-2 w-2 rounded-full", dotColor)} />
        <span className="max-w-[200px] truncate text-white/80">
          {active ? active.nickname || active.url : "Choisir"}
        </span>
        <ChevronsUpDown className="h-3.5 w-3.5 text-white/30" />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-[320px]">
        <DropdownMenuLabel className="text-[10px] uppercase tracking-wider">
          Noeuds
        </DropdownMenuLabel>
        {knownCoordinators.map((coord) => {
          const isActive = coord.url === activeCoordinatorUrl;
          return (
            <DropdownMenuItem
              key={coord.url}
              className="flex items-center gap-2"
              onSelect={(e) => {
                e.preventDefault();
                setActive(coord.url);
              }}
            >
              {isActive ? (
                <Check className="h-4 w-4 text-emerald-400" />
              ) : (
                <span className="h-4 w-4" />
              )}
              <div className="flex min-w-0 flex-1 flex-col">
                <span className="truncate text-sm font-medium">
                  {coord.nickname || coord.url}
                </span>
                <span className="truncate text-[10px] text-muted-foreground">
                  {coord.url}
                </span>
              </div>
              <button
                type="button"
                className="rounded p-1 hover:bg-destructive/10 hover:text-destructive"
                onClick={(e) => {
                  e.stopPropagation();
                  removeCoordinator(coord.url);
                }}
                aria-label={`Retirer ${coord.url}`}
              >
                <Trash2 className="h-3.5 w-3.5" />
              </button>
            </DropdownMenuItem>
          );
        })}
        <DropdownMenuSeparator />
        <DropdownMenuItem onSelect={onAddClick}>
          <Plus className="h-4 w-4" />
          <span>Se connecter a un noeud</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
