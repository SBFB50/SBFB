/**
 * Empty-state page rendered when the shell has no known
 * coordinators yet.
 *
 * Sprint 5 decision D4: the shell is not a process spawner, so
 * onboarding walks the user through the CLI commands and offers
 * an "Add coordinator" dialog once they've started one.
 */

import { useState } from "react";
import { Copy, Check, Plus } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { AddCoordinatorDialog } from "@/components/AddCoordinatorDialog";

const INIT_CMD = "uv run --package nexus-coordinator nexus-coordinator init demo";
const START_CMD =
  "uv run --package nexus-coordinator nexus-coordinator start demo";

export default function OnboardingEmpty() {
  const [dialogOpen, setDialogOpen] = useState(false);

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <div className="space-y-2">
        <h1 className="text-2xl font-bold">Bienvenue sur nexus-grid</h1>
        <p className="text-sm text-muted-foreground">
          Ce shell affiche les coordinateurs nexus-grid qui tournent sur
          ta machine. Il ne démarre pas de process — c'est toi qui lances
          un coordinateur via la CLI, puis tu l'ajoutes ici.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">
            1. Démarre un coordinateur
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <CommandBlock step="Crée le projet" command={INIT_CMD} />
          <CommandBlock
            step="Lance le coordinateur (reste bloqué dans ce terminal)"
            command={START_CMD}
          />
          <p className="text-xs text-muted-foreground">
            Le coordinateur écoute par défaut sur{" "}
            <code className="font-mono">http://127.0.0.1:8765</code>.
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">2. Ajoute-le au shell</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <p className="text-sm text-muted-foreground">
            Une fois le coordinateur démarré, clique ci-dessous et entre
            son URL. Le shell va le joindre via{" "}
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
