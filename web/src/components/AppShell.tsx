/**
 * Top-level shell layout: sidebar + header + routed outlet.
 *
 * Reuses the shadcn `Sidebar` primitive. The four nav entries
 * (`/my-projects`, `/my-network`, `/browse`, `/curators`) come
 * from the Sprint 5 plan §5 — `/browse` and `/curators` are
 * stubs Phase A and get real content in Phase D only for the
 * "coming soon" panels.
 *
 * The header holds the coordinator picker (dropdown over the
 * persisted `knownCoordinators`), a status dot driven by a
 * React Query `useQuery` on `getHealth(activeUrl)`, and the
 * "Add coordinator" trigger.
 */

import { NavLink, Outlet } from "react-router-dom";
import {
  FolderKanban,
  Cpu,
  Compass,
  BookmarkPlus,
  Plus,
  Check,
  Trash2,
  ChevronsUpDown,
} from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { lazy, Suspense, useState } from "react";

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarRail,
  SidebarTrigger,
} from "@/components/ui/sidebar";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Separator } from "@/components/ui/separator";
import {
  selectActiveCoordinator,
  useProjectStore,
} from "@/stores/projectStore";
import { getHealth } from "@/api/coordinator";
import { AddCoordinatorDialog } from "@/components/AddCoordinatorDialog";
import { useCommandPalette } from "@/components/command-palette/useCommandPalette";
import { RouteErrorBoundary } from "@/components/RouteErrorBoundary";
import { cn } from "@/lib/utils";

// Sprint 9 Phase A (D6) — palette is the largest non-vendor
// surface in the bundle (cmdk + lucide icons + the App
// commands group). We `React.lazy` it so the chunk loads on
// the first ⌘K toggle rather than on shell paint, keeping the
// initial main chunk under the 350 KB target.
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
  { to: "/my-projects", label: "Mes projets", icon: FolderKanban },
  { to: "/my-network", label: "Mon réseau", icon: Cpu },
  { to: "/browse", label: "Explorer", icon: Compass },
  { to: "/curators", label: "Curators", icon: BookmarkPlus },
];

export function AppShell() {
  const [addDialogOpen, setAddDialogOpen] = useState(false);
  const palette = useCommandPalette();

  return (
    <SidebarProvider>
      <Sidebar collapsible="icon">
        <SidebarHeader className="px-3 py-3">
          <div className="flex items-center gap-2.5">
            <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-primary text-primary-foreground">
              <span className="text-sm font-bold">N</span>
            </div>
            <div className="min-w-0 flex-1 group-data-[collapsible=icon]:hidden">
              <h1 className="text-sm font-bold tracking-wide text-foreground">
                nexus-grid
              </h1>
              <p className="text-[9px] uppercase tracking-widest text-muted-foreground">
                Shell P2P
              </p>
            </div>
          </div>
        </SidebarHeader>

        <SidebarContent>
          <SidebarGroup>
            <SidebarGroupLabel className="text-[10px] font-semibold uppercase tracking-wider">
              Navigation
            </SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {NAV_ENTRIES.map(({ to, label, icon: Icon }) => (
                  <SidebarMenuItem key={to}>
                    <SidebarMenuButton
                      tooltip={label}
                      render={
                        <NavLink
                          to={to}
                          className={({ isActive }) =>
                            cn(isActive && "bg-sidebar-accent font-medium")
                          }
                        />
                      }
                    >
                      <Icon size={16} />
                      <span>{label}</span>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        </SidebarContent>

        <SidebarFooter className="px-3 py-2">
          <p className="group-data-[collapsible=icon]:hidden text-[10px] text-muted-foreground">
            Sprint 5 MVP
          </p>
        </SidebarFooter>

        <SidebarRail />
      </Sidebar>

      <SidebarInset className="flex min-h-screen flex-col">
        <header className="flex h-14 shrink-0 items-center gap-2 border-b border-border px-4">
          <SidebarTrigger />
          <Separator orientation="vertical" className="mr-2 h-4" />
          <CoordinatorPicker onAddClick={() => setAddDialogOpen(true)} />
          <div className="ml-auto flex items-center gap-2">
            <Button
              size="sm"
              variant="ghost"
              onClick={palette.toggle}
              aria-label="Ouvrir la palette de commandes"
              data-testid="command-palette-trigger"
              className="gap-2 text-xs text-muted-foreground"
            >
              <span>Commandes</span>
              <kbd className="pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border border-border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground">
                {IS_MAC ? "⌘" : "Ctrl"}
                <span>K</span>
              </kbd>
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => setAddDialogOpen(true)}
            >
              <Plus className="h-4 w-4" /> Ajouter un coordinateur
            </Button>
          </div>
        </header>

        <main className="flex-1 overflow-auto p-6">
          <RouteErrorBoundary>
            <Outlet />
          </RouteErrorBoundary>
        </main>
      </SidebarInset>

      <AddCoordinatorDialog
        open={addDialogOpen}
        onOpenChange={setAddDialogOpen}
      />

      {/*
        Suspense fallback is null on purpose: the palette is
        invisible until `palette.open` flips, so an unmounted
        chunk and a loading-but-closed chunk look identical to
        the user. The lazy import resolves on first interaction
        with the trigger button (the click bubbles before
        Suspense reads `palette.open`).
      */}
      <Suspense fallback={null}>
        <CommandPalette
          palette={palette}
          onAddCoordinator={() => setAddDialogOpen(true)}
        />
      </Suspense>
    </SidebarProvider>
  );
}

/**
 * Header dropdown that lists every known coordinator, shows a
 * health dot for the active one, and lets the user switch
 * between them.
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
      ? "bg-muted-foreground"
      : healthQuery.isFetching && !healthQuery.data
        ? "bg-yellow-500"
        : healthy
          ? "bg-emerald-500"
          : "bg-red-500";

  if (knownCoordinators.length === 0) {
    return (
      <Button size="sm" variant="ghost" onClick={onAddClick}>
        Aucun coordinateur — cliquez pour ajouter
      </Button>
    );
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={<Button size="sm" variant="ghost" className="gap-2" />}
      >
        <span className={cn("h-2 w-2 rounded-full", dotColor)} />
        <span className="truncate max-w-[200px]">
          {active ? active.nickname || active.url : "Choisir un coordinateur"}
        </span>
        <ChevronsUpDown className="h-3.5 w-3.5 text-muted-foreground" />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-[320px]">
        <DropdownMenuLabel className="text-[10px] uppercase tracking-wider">
          Coordinateurs connus
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
                <Check className="h-4 w-4 text-emerald-500" />
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
          <span>Ajouter un coordinateur</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
