/**
 * Sprint 6 Phase C — global Ctrl+K command palette.
 *
 * Four command groups:
 *  - Navigation: the four top-level routes
 *  - Projets: one entry per known coordinator from the Zustand
 *    store (each navigates to /project/{nickname})
 *  - App: <name>: Sprint 8 Phase E — every app enrolled on the
 *    active coordinator contributes its `@nexus_command`
 *    entries here; selecting one invokes the handler and
 *    forwards any returned `{navigation: {path}}` to React
 *    Router.
 *  - Actions: Ajouter un coordinateur, Recharger
 */

import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import {
  FolderKanban,
  Cpu,
  Compass,
  BookmarkPlus,
  Plus,
  RefreshCw,
  Folder,
  Sparkles,
} from "lucide-react";

import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "@/components/ui/command";
import {
  type AppSummary,
  invokeAppCommand,
  listAppCommands,
  listApps,
} from "@/api/coordinator";
import {
  selectActiveCoordinator,
  useProjectStore,
} from "@/stores/projectStore";
import { extractNavigationPath } from "./extractNavigationPath";
import type { useCommandPalette } from "./useCommandPalette";

interface Props {
  palette: ReturnType<typeof useCommandPalette>;
  onAddCoordinator: () => void;
}

export function CommandPalette({ palette, onAddCoordinator }: Props) {
  const navigate = useNavigate();
  const knownCoordinators = useProjectStore((s) => s.knownCoordinators);
  const active = useProjectStore(selectActiveCoordinator);

  const appsQuery = useQuery({
    queryKey: ["palette-apps", active?.url],
    queryFn: () => {
      if (!active) throw new Error("no active coordinator");
      return listApps(active.url);
    },
    enabled: Boolean(active) && palette.open,
    staleTime: 30_000,
  });

  const go = (path: string) => {
    palette.setOpen(false);
    navigate(path);
  };

  const triggerAdd = () => {
    palette.setOpen(false);
    onAddCoordinator();
  };

  const triggerReload = () => {
    palette.setOpen(false);
    window.location.reload();
  };

  const runAppCommand = async (appName: string, cmdName: string) => {
    if (!active) return;
    palette.setOpen(false);
    try {
      const envelope = await invokeAppCommand(active.url, appName, cmdName);
      const path = extractNavigationPath(envelope.result);
      if (path) navigate(path);
    } catch (e) {
      // The palette closes regardless of outcome — surfacing
      // errors via a toast is deferred to a Sprint 9 polish
      // pass. Logging here keeps the failure discoverable in
      // devtools without breaking the UX flow.
      console.error("[palette] invokeAppCommand failed", {
        appName,
        cmdName,
        error: e,
      });
    }
  };

  const apps = appsQuery.data?.apps ?? [];

  return (
    <CommandDialog
      open={palette.open}
      onOpenChange={palette.setOpen}
      title="Palette de commandes"
      description="Tapez pour rechercher une action ou un projet."
    >
      {/*
        Wrapping in <Command> is required: shadcn's CommandDialog
        template does not include the cmdk root primitive, so the
        children would crash with "Cannot read subscribe of
        undefined" when reading the store context. Keeping the wrap
        local to the palette means we don't modify the vendor UI
        file (T1 policy — keep shadcn regen-safe).
      */}
      <Command label="Palette de commandes" shouldFilter>
      <CommandInput placeholder="Rechercher une action ou un projet…" />
      <CommandList>
        <CommandEmpty>Aucun résultat.</CommandEmpty>

        <CommandGroup heading="Navigation">
          <CommandItem onSelect={() => go("/my-projects")}>
            <FolderKanban className="size-4" />
            <span>Mes projets</span>
          </CommandItem>
          <CommandItem onSelect={() => go("/my-network")}>
            <Cpu className="size-4" />
            <span>Mon réseau</span>
          </CommandItem>
          <CommandItem onSelect={() => go("/browse")}>
            <Compass className="size-4" />
            <span>Explorer</span>
          </CommandItem>
          <CommandItem onSelect={() => go("/curators")}>
            <BookmarkPlus className="size-4" />
            <span>Curators</span>
          </CommandItem>
        </CommandGroup>

        {knownCoordinators.length > 0 && (
          <>
            <CommandSeparator />
            <CommandGroup heading="Projets">
              {knownCoordinators.map((coord) => {
                const label = coord.nickname || coord.url;
                return (
                  <CommandItem
                    key={coord.url}
                    value={`projet ${label} ${coord.url}`}
                    onSelect={() =>
                      go(`/project/${encodeURIComponent(coord.nickname || "")}`)
                    }
                  >
                    <Folder className="size-4" />
                    <span>{label}</span>
                    <span className="ml-auto text-[10px] text-muted-foreground">
                      {coord.url}
                    </span>
                  </CommandItem>
                );
              })}
            </CommandGroup>
          </>
        )}

        {active &&
          apps
            .filter((app) => app.commands > 0)
            .map((app) => (
              <AppCommandsGroup
                key={app.name}
                baseUrl={active.url}
                app={app}
                onRun={(cmd) => runAppCommand(app.name, cmd)}
              />
            ))}

        <CommandSeparator />
        <CommandGroup heading="Actions">
          <CommandItem onSelect={triggerAdd}>
            <Plus className="size-4" />
            <span>Ajouter un coordinateur</span>
          </CommandItem>
          <CommandItem onSelect={triggerReload}>
            <RefreshCw className="size-4" />
            <span>Recharger la page</span>
          </CommandItem>
        </CommandGroup>
      </CommandList>
      </Command>
    </CommandDialog>
  );
}

/**
 * One group per enrolled app with `commands > 0`. Split into its
 * own component so we can `useQuery` per app without breaking
 * the rules of hooks — the parent only renders the group when
 * the app has at least one command, so mount / unmount is
 * keyed on `app.name` and stable across re-renders.
 */
function AppCommandsGroup({
  baseUrl,
  app,
  onRun,
}: {
  baseUrl: string;
  app: AppSummary;
  onRun: (cmdName: string) => void;
}) {
  const query = useQuery({
    queryKey: ["palette-app-commands", baseUrl, app.name],
    queryFn: () => listAppCommands(baseUrl, app.name),
    staleTime: 30_000,
    refetchInterval: 30_000,
  });

  const commands = query.data ?? [];
  if (commands.length === 0) return null;

  return (
    <>
      <CommandSeparator />
      <CommandGroup heading={`App : ${app.name}`}>
        {commands.map((cmd) => (
          <CommandItem
            key={cmd.name}
            value={`app ${app.name} ${cmd.name} ${cmd.description}`}
            onSelect={() => onRun(cmd.name)}
          >
            <Sparkles className="size-4" />
            <span>{cmd.description}</span>
            <span className="ml-auto text-[10px] text-muted-foreground">
              {cmd.group}
            </span>
          </CommandItem>
        ))}
      </CommandGroup>
    </>
  );
}

