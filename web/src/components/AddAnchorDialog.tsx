// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 75 Phase F — « Ajouter une ancre » (intention node-Browse).
 *
 * Une ancre EST une subscription dans le MÊME attention set que les curators
 * (kickoff D1/Q3/DQ3 : un seul attention set, pas de section `[directory]`) —
 * le dialog appelle donc la route existante `POST /api/daemon/curators/
 * subscribe` via l'alias `addAnchor`. L'annuaire du nœud arrive ensuite par
 * gossip ou par le re-pull boot (subscribe n'ingère PAS de façon synchrone) :
 * le parent rend la ligne « En attente d'une première annonce... » jusque-là.
 *
 * Verrou 3 (anti-recentralisation) : le placeholder est un texte INERTE
 * (jamais une clé pré-remplie qui s'auto-abonnerait) — l'ajout exige un
 * collage + une soumission explicites de l'utilisateur.
 */

import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Anchor, Loader2 } from "lucide-react";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  addAnchor,
  isValidCuratorPubkey,
  type DaemonResult,
  type SubscriptionsResponse,
} from "@/api/daemon";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  coordUrl: string;
  /**
   * UX-ARRIVAL : pré-remplissage du champ identité, déclenché UNIQUEMENT par
   * une action utilisateur explicite (le clic « S'abonner » d'une ligne de
   * nœud observé). Jamais un défaut : le dialog ouvert à la main garde son
   * placeholder INERTE (verrou 3), et la soumission reste un geste explicite
   * dans tous les cas. Le parent doit `key`-er le composant sur cette valeur
   * pour que l'état initial du champ suive (remount, pas d'effect).
   */
  initialPubkey?: string;
}

export function AddAnchorDialog({
  open,
  onOpenChange,
  coordUrl,
  initialPubkey,
}: Props) {
  const queryClient = useQueryClient();
  const [pubkeyInput, setPubkeyInput] = useState(initialPubkey ?? "");
  const [formError, setFormError] = useState<string | null>(null);

  const mutation = useMutation({
    mutationFn: (hex: string) => addAnchor(coordUrl, hex),
    onSuccess: (result: DaemonResult<SubscriptionsResponse>) => {
      if (result.kind === "data") {
        setPubkeyInput("");
        setFormError(null);
        // Le nœud apparaît dans /nodes dès que son annuaire est ingéré ;
        // en attendant, la liste des subscriptions rend la ligne « en
        // attente » — d'où l'invalidation des trois vues.
        void queryClient.invalidateQueries({ queryKey: ["daemon-nodes", coordUrl] });
        void queryClient.invalidateQueries({ queryKey: ["daemon-curators", coordUrl] });
        void queryClient.invalidateQueries({ queryKey: ["daemon-browse", coordUrl] });
        onOpenChange(false);
      } else {
        setFormError(result.reason);
      }
    },
    onError: (err: unknown) => {
      setFormError(err instanceof Error ? err.message : "erreur inconnue");
    },
  });

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const candidate = pubkeyInput.trim().toLowerCase();
    if (!isValidCuratorPubkey(candidate)) {
      setFormError(
        "L'identité du nœud doit faire 64 caractères hexadécimaux minuscules.",
      );
      return;
    }
    setFormError(null);
    mutation.mutate(candidate);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="sm:max-w-md"
        data-testid="add-anchor-dialog"
      >
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Anchor className="h-4 w-4 text-purple-300" />
            Ajouter une ancre
          </DialogTitle>
          <DialogDescription>
            Colle l&apos;identité (clé publique Ed25519, 64 caractères
            hexadécimaux) d&apos;un nœud qui publie un catalogue. Son
            catalogue apparaîtra dès sa première annonce — l&apos;ancre est
            une source de découverte, jamais une autorité sur les apps.
          </DialogDescription>
        </DialogHeader>

        <form className="space-y-3" onSubmit={handleSubmit}>
          <div>
            <label
              htmlFor="anchor-pubkey"
              className="mb-1 block text-xs font-medium text-white/40"
            >
              Identité du nœud
            </label>
            <input
              id="anchor-pubkey"
              data-testid="anchor-pubkey-input"
              value={pubkeyInput}
              onChange={(e) => setPubkeyInput(e.target.value)}
              placeholder="abcd1234..."
              autoComplete="off"
              spellCheck={false}
              className="w-full rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white/80 placeholder-white/30 outline-none focus:border-purple-500/40 focus:ring-1 focus:ring-purple-500/20"
            />
            {formError ? (
              <p
                className="mt-1 text-xs text-red-400"
                data-testid="anchor-form-error"
              >
                {formError}
              </p>
            ) : null}
          </div>
          <button
            type="submit"
            disabled={mutation.isPending}
            className="flex w-full items-center justify-center gap-2 rounded-lg bg-purple-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-purple-500 disabled:opacity-50"
            data-testid="anchor-subscribe-submit"
          >
            {mutation.isPending ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Anchor className="h-4 w-4" />
            )}
            S&apos;abonner à ce nœud
          </button>
        </form>
      </DialogContent>
    </Dialog>
  );
}
