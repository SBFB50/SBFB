// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Empty-state page rendered when the shell has no known
 * coordinators yet.
 *
 * Sprint 5 decision D4: the shell is not a process spawner. In
 * the common case the boot helper auto-registers the same-origin
 * daemon (see `api/bootstrap.ts`), so this page only shows when no
 * daemon is serving the shell — it then walks the user through
 * starting `nexus-shell-daemon` and adding it manually.
 */

import { useState } from "react";
import { Copy, Check, Plus } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { AddCoordinatorDialog } from "@/components/AddCoordinatorDialog";

const START_CMD = "nexus-shell-daemon start";

export default function OnboardingEmpty() {
  const [dialogOpen, setDialogOpen] = useState(false);

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <div className="space-y-2">
        <h1 className="text-2xl font-bold">Bienvenue sur nexus-grid</h1>
        <p className="text-sm text-muted-foreground">
          Ce shell se connecte au daemon nexus-shell-daemon local qui le
          sert. Normalement il est détecté automatiquement — si tu vois cet
          écran, aucun daemon n'est joignable sur cette origine.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">1. Démarre le daemon</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <CommandBlock
            step="Lance le daemon (reste bloqué dans ce terminal)"
            command={START_CMD}
          />
          <p className="text-xs text-muted-foreground">
            Le launcher fait la même chose et ouvre directement ton
            navigateur sur le shell.
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">2. Ajoute-le au shell</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <p className="text-sm text-muted-foreground">
            Une fois le daemon démarré, clique ci-dessous et entre son URL.
            Le shell va le joindre via{" "}
            <code className="font-mono">GET /health</code>.
          </p>
          <Button onClick={() => setDialogOpen(true)}>
            <Plus className="h-4 w-4" /> Ajouter un coordinateur
          </Button>
        </CardContent>
      </Card>

      <AddCoordinatorDialog open={dialogOpen} onOpenChange={setDialogOpen} />
    </div>
  );
}

function CommandBlock({ step, command }: { step: string; command: string }) {
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(command);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      /* silent */
    }
  };
  return (
    <div className="space-y-1.5">
      <p className="text-xs font-medium text-muted-foreground">{step}</p>
      <div className="flex items-start gap-2 rounded-md border border-border bg-muted/30 p-3 font-mono text-[11px]">
        <code className="flex-1 break-all text-foreground">{command}</code>
        <button
          type="button"
          onClick={copy}
          className="shrink-0 rounded p-1 hover:bg-muted"
          aria-label="Copier"
        >
          {copied ? (
            <Check className="h-3.5 w-3.5 text-emerald-500" />
          ) : (
            <Copy className="h-3.5 w-3.5 text-muted-foreground" />
          )}
        </button>
      </div>
    </div>
  );
}

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = OnboardingEmpty;
