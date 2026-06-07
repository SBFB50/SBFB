// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Dialog for adding a new coordinator to the shell's known list.
 *
 * Flow:
 * 1. User types a URL (default: the same origin that serves the
 *    shell — i.e. the local daemon — so "Tester" works out of the
 *    box; most users never touch this since the boot helper
 *    auto-registers that origin).
 * 2. "Tester" button hits `GET /health` and shows a success /
 *    failure indicator. The health payload's `project_name` is
 *    pre-filled as the suggested nickname.
 * 3. "Ajouter" commits the entry to `useProjectStore` and closes.
 *
 * Sprint 5 decision D4: the dialog does NOT spawn a daemon
 * process. If the URL is unreachable, the shell tells the user
 * to start the daemon themselves via the CLI and includes
 * a copy-paste command.
 */

import { useState } from "react";
import { Check, Copy, Loader2, X } from "lucide-react";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  ApiHttpError,
  ApiProtocolError,
  getHealth,
  normalizeApiUrl,
} from "@/api/coordinator";
import { useProjectStore } from "@/stores/projectStore";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/**
 * Default URL prefilled in the dialog: the same origin that serves
 * the shell (the local daemon), so "Tester" succeeds immediately.
 * Falls back to a loopback hint when there is no http(s) origin
 * (e.g. SSR / non-browser test contexts).
 */
function defaultCoordinatorUrl(): string {
  if (
    typeof window !== "undefined" &&
    /^https?:\/\//i.test(window.location.origin)
  ) {
    return window.location.origin;
  }
  return "http://127.0.0.1";
}

type ProbeStatus =
  | { kind: "idle" }
  | { kind: "testing" }
  | { kind: "ok"; nickname: string; nodeId: string | null }
  | { kind: "error"; message: string };

export function AddCoordinatorDialog({ open, onOpenChange }: Props) {
  const [url, setUrl] = useState(defaultCoordinatorUrl);
  const [nickname, setNickname] = useState("");
  const [status, setStatus] = useState<ProbeStatus>({ kind: "idle" });
  const addCoordinator = useProjectStore((s) => s.addCoordinator);

  const reset = () => {
    setUrl(defaultCoordinatorUrl());
    setNickname("");
    setStatus({ kind: "idle" });
  };

  const handleOpenChange = (next: boolean) => {
    if (!next) reset();
    onOpenChange(next);
  };

  const onTest = async () => {
    setStatus({ kind: "testing" });
    try {
      const normalized = normalizeApiUrl(url);
      const health = await getHealth(normalized);
      setStatus({
        kind: "ok",
        nickname: health.project,
        nodeId: health.node_id,
      });
      if (!nickname) {
        setNickname(health.project);
      }
    } catch (e) {
      let message = "Erreur inconnue";
      if (e instanceof ApiHttpError) {
        message = `HTTP ${e.status} — le noeud a répondu mais refuse la requête`;
      } else if (e instanceof ApiProtocolError) {
        message = `Réponse invalide du noeud : ${e.issues[0]?.message ?? "schema mismatch"}`;
      } else if (e instanceof Error) {
        message = e.message;
      }
      setStatus({ kind: "error", message });
    }
  };

  const onAdd = () => {
    if (status.kind !== "ok") return;
    addCoordinator(url, {
      nickname: nickname || status.nickname,
      nodeId: status.nodeId,
    });
    handleOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Se connecter a un noeud</DialogTitle>
          <DialogDescription>
            Entre l'URL d'un daemon nexus-shell-daemon joignable (par
            défaut, ce nœud). Le shell ne lance pas de process — tu dois
            démarrer le daemon toi-même via la CLI ou le launcher.
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-col gap-4 py-2">
          <div className="flex flex-col gap-1.5">
            <label htmlFor="coord-url" className="text-xs font-medium">
              URL du noeud
            </label>
            <div className="flex gap-2">
              <Input
                id="coord-url"
                value={url}
                onChange={(e) => {
                  setUrl(e.target.value);
                  setStatus({ kind: "idle" });
                }}
                placeholder="http://127.0.0.1:PORT"
                autoFocus
              />
              <Button
                variant="outline"
                onClick={onTest}
                disabled={status.kind === "testing" || !url.trim()}
              >
                {status.kind === "testing" ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  "Tester"
                )}
              </Button>
            </div>
            {status.kind === "ok" && (
              <div className="flex items-center gap-2 text-xs text-emerald-500">
                <Check className="h-3.5 w-3.5" /> Noeud joignable —
                projet <span className="font-mono">{status.nickname}</span>
              </div>
            )}
            {status.kind === "error" && (
              <div className="flex items-start gap-2 text-xs text-destructive">
                <X className="mt-0.5 h-3.5 w-3.5 shrink-0" />
                <span>{status.message}</span>
              </div>
            )}
          </div>

          <div className="flex flex-col gap-1.5">
            <label htmlFor="coord-nickname" className="text-xs font-medium">
              Nom affiché (optionnel)
            </label>
            <Input
              id="coord-nickname"
              value={nickname}
              onChange={(e) => setNickname(e.target.value)}
              placeholder="Nom lisible pour ta sidebar"
            />
          </div>

          <CliHint />
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={() => handleOpenChange(false)}>
            Annuler
          </Button>
          <Button onClick={onAdd} disabled={status.kind !== "ok"}>
            Ajouter
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function CliHint() {
  const [copied, setCopied] = useState(false);
  const cmd = "nexus-shell-daemon start";

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(cmd);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // Clipboard API can be unavailable in some test environments
      // — fall back silently, the user can still manually select.
    }
  };

  return (
    <div className="rounded-md border border-border bg-muted/30 p-3">
      <p className="text-[10px] uppercase tracking-wider text-muted-foreground">
        Pas encore de daemon ?
      </p>
      <p className="mt-1 text-xs">
        Lance-en un dans un autre terminal :
      </p>
      <div className="mt-2 flex items-start gap-2 rounded bg-background/70 p-2 font-mono text-[11px]">
        <code className="flex-1 break-all text-foreground">{cmd}</code>
        <button
          type="button"
          onClick={copy}
          className="shrink-0 rounded p-1 hover:bg-muted"
          aria-label="Copier la commande"
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
