/**
 * Sprint 5 Phase B — project Invites tab.
 *
 * Lists existing invites and lets the user create / revoke.
 * Creates go through `POST /invite/create`; revokes go through
 * `DELETE /invite/{id}`. Both mutate and then invalidate the
 * `["invites", url]` query so the table refreshes without a
 * manual poll.
 */

import { useState } from "react";
import {
  type UseQueryResult,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query";
import { Copy, Check, Trash2, Plus } from "lucide-react";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
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
import { Badge } from "@/components/ui/badge";
import {
  createInvite,
  type CreateInviteBody,
  type InviteList,
  revokeInvite,
} from "@/api/coordinator";
import { formatHash } from "@/lib/format";

interface Props {
  url: string;
  query: UseQueryResult<InviteList, Error>;
}

export function InvitesTab({ url, query }: Props) {
  const [dialogOpen, setDialogOpen] = useState(false);
  const invites = query.data?.invites ?? [];

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-sm font-semibold">Invites</h2>
          <p className="text-xs text-muted-foreground">
            Les tokens <code className="font-mono">nx1v2…</code> que tu
            partages pour faire rejoindre un worker.
          </p>
        </div>
        <Button onClick={() => setDialogOpen(true)}>
          <Plus className="h-4 w-4" /> Nouveau
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">
            {invites.length} invite(s) émise(s)
          </CardTitle>
          <CardDescription>
            Révoquer un invite le marque comme invalide côté
            coordinateur mais ne peut pas le supprimer chez les
            workers qui l'ont déjà consommé.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {query.isLoading ? (
            <p className="text-sm text-muted-foreground">Chargement…</p>
          ) : query.isError ? (
            <p className="text-sm text-destructive">
              Erreur fetch invites : {query.error.message}
            </p>
          ) : invites.length === 0 ? (
            <p className="text-xs text-muted-foreground">
              Aucune invite. Utilise le bouton « Nouveau » pour en
              générer une.
            </p>
          ) : (
            <InvitesTable url={url} invites={invites} />
          )}
        </CardContent>
      </Card>

      <CreateInviteDialog
        url={url}
        open={dialogOpen}
        onOpenChange={setDialogOpen}
      />
    </div>
  );
}

function InvitesTable({
  url,
  invites,
}: {
  url: string;
  invites: InviteList["invites"];
}) {
  const qc = useQueryClient();
  const revoke = useMutation({
    mutationFn: (id: string) => revokeInvite(url, id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["invites", url] }),
  });

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left text-xs">
        <thead>
          <tr className="border-b border-border text-[10px] uppercase tracking-wider text-muted-foreground">
            <th className="py-2 pr-3">Id</th>
            <th className="py-2 pr-3">Scope</th>
            <th className="py-2 pr-3">Expire</th>
            <th className="py-2 pr-3 text-right">Usages</th>
            <th className="py-2 pr-3">État</th>
            <th className="py-2 pr-3">Note</th>
            <th className="py-2 pr-3"></th>
          </tr>
        </thead>
        <tbody>
          {invites.map((inv) => {
            const revoked = inv.revoked_at !== null;
            return (
              <tr key={inv.id} className="border-b border-border/50">
                <td className="py-2 pr-3 font-mono">{formatHash(inv.id, 10)}</td>
                <td className="py-2 pr-3">{inv.scope}</td>
                <td className="py-2 pr-3 text-muted-foreground">
                  {new Date(inv.expires_at * 1000).toLocaleString()}
                </td>
                <td className="py-2 pr-3 text-right">
                  {inv.uses_count}
                  {inv.max_uses !== null && `/${inv.max_uses}`}
                </td>
                <td className="py-2 pr-3">
                  <Badge
                    variant="outline"
                    className={
                      revoked
                        ? "border-destructive/40 text-destructive"
                        : "border-emerald-500/40 text-emerald-500"
                    }
                  >
                    {revoked ? "Révoquée" : "Active"}
                  </Badge>
                </td>
                <td className="py-2 pr-3 text-muted-foreground">
                  {inv.note ?? "—"}
                </td>
                <td className="py-2 pr-3">
                  <button
                    type="button"
                    disabled={revoked || revoke.isPending}
                    onClick={() => revoke.mutate(inv.id)}
                    className="rounded p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive disabled:opacity-30 disabled:hover:bg-transparent"
                    aria-label={`Révoquer ${inv.id}`}
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </button>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function CreateInviteDialog({
  url,
  open,
  onOpenChange,
}: {
  url: string;
  open: boolean;
  onOpenChange: (o: boolean) => void;
}) {
  const qc = useQueryClient();
  const [scope, setScope] = useState<"worker" | "observer">("worker");
  const [expiryDays, setExpiryDays] = useState("7");
  const [maxUses, setMaxUses] = useState("");
  const [note, setNote] = useState("");
  const [wire, setWire] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const mutation = useMutation({
    mutationFn: (body: CreateInviteBody) => createInvite(url, body),
    onSuccess: (data) => {
      setWire(data.wire);
      qc.invalidateQueries({ queryKey: ["invites", url] });
    },
  });

  const reset = () => {
    setScope("worker");
    setExpiryDays("7");
    setMaxUses("");
    setNote("");
    setWire(null);
    setCopied(false);
    mutation.reset();
  };

  const handleOpenChange = (next: boolean) => {
    if (!next) reset();
    onOpenChange(next);
  };

  const onSubmit = () => {
    const days = Number(expiryDays);
    const body: CreateInviteBody = {
      scope,
      expiry_secs: Math.max(60, Math.round((Number.isFinite(days) ? days : 7) * 86400)),
      max_uses: maxUses.trim() === "" ? null : Math.max(1, Math.round(Number(maxUses))),
      note: note.trim() === "" ? null : note.trim(),
    };
    mutation.mutate(body);
  };

  const copyWire = async () => {
    if (!wire) return;
    try {
      await navigator.clipboard.writeText(wire);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      /* silent */
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Créer un invite</DialogTitle>
          <DialogDescription>
            Un invite signé <code className="font-mono">nx1v2…</code>{" "}
            que tu partages à un worker pour qu'il rejoigne ce
            projet via <code className="font-mono">nexus-worker join</code>.
          </DialogDescription>
        </DialogHeader>

        {wire ? (
          <div className="space-y-3">
            <p className="text-xs text-emerald-500">
              Invite généré. Copie-le dans un terminal :
            </p>
            <div className="flex items-start gap-2 rounded-md border border-border bg-muted/30 p-3 font-mono text-[11px]">
              <code className="flex-1 break-all">{wire}</code>
              <button
                type="button"
                onClick={copyWire}
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
        ) : (
          <div className="flex flex-col gap-3 py-2">
            <div className="flex flex-col gap-1.5">
              <label className="text-xs font-medium">Scope</label>
              <div className="flex gap-2">
                {(["worker", "observer"] as const).map((s) => (
                  <button
                    key={s}
                    type="button"
                    onClick={() => setScope(s)}
                    className={`rounded-md border px-3 py-1.5 text-xs ${
                      scope === s
                        ? "border-primary bg-primary/10 text-primary"
                        : "border-border text-muted-foreground"
                    }`}
                  >
                    {s}
                  </button>
                ))}
              </div>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div className="flex flex-col gap-1.5">
                <label htmlFor="inv-days" className="text-xs font-medium">
                  Expire dans (jours)
                </label>
                <Input
                  id="inv-days"
                  type="number"
                  min={1}
                  value={expiryDays}
                  onChange={(e) => setExpiryDays(e.target.value)}
                />
              </div>
              <div className="flex flex-col gap-1.5">
                <label htmlFor="inv-uses" className="text-xs font-medium">
                  Usages max (vide = illimité)
                </label>
                <Input
                  id="inv-uses"
                  type="number"
                  min={1}
                  value={maxUses}
                  onChange={(e) => setMaxUses(e.target.value)}
                />
              </div>
            </div>
            <div className="flex flex-col gap-1.5">
              <label htmlFor="inv-note" className="text-xs font-medium">
                Note (optionnelle)
              </label>
              <Input
                id="inv-note"
                value={note}
                onChange={(e) => setNote(e.target.value)}
                placeholder="Ex: worker d'Alice"
              />
            </div>
            {mutation.isError && (
              <p className="text-xs text-destructive">
                {mutation.error.message}
              </p>
            )}
          </div>
        )}

        <DialogFooter>
          <Button variant="ghost" onClick={() => handleOpenChange(false)}>
            {wire ? "Fermer" : "Annuler"}
          </Button>
          {!wire && (
            <Button onClick={onSubmit} disabled={mutation.isPending}>
              {mutation.isPending ? "Création…" : "Créer"}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
