/**
 * Sprint 6 Phase C — global Ctrl+K command palette.
 *
 * Three command groups:
 *  - Navigation: the four top-level routes
 *  - Projets: one entry per known coordinator from the Zustand
 *    store (each navigates to /project/{nickname})
 *  - Actions: Ajouter un coordinateur, Recharger
 *
 * Sprint 7 will extend with "Subscribe to curator list" and
 * "Browse DHT" entries once nexus-shell-daemon is wired.
 * Sprint 8 will allow apps to contribute command entries via a
 * SDK hook.
 */

import { useNavigate } from "react-router-dom";
import {
  FolderKanban,
  Cpu,
  Compass,
  BookmarkPlus,
  Plus,
  RefreshCw,
  Folder,
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
import { useProjectStore } from "@/stores/projectStore";
import type { useCommandPalette } from "./useCommandPalette";

interface Props {
  palette: ReturnType<typeof useCommandPalette>;
  onAddCoordinator: () => void;
}

export function CommandPalette({ palette, onAddCoordinator }: Props) {
  const navigate = useNavigate();
  const knownCoordinators = useProjectStore((s) => s.knownCoordinators);

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
