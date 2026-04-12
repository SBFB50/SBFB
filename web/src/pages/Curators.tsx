// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/curators` — Sprint 7 Phase E curator management page.
 *
 * Shows:
 *   1. The attention set (curators the user has subscribed
 *      to), each as a card with an Unsubscribe button.
 *   2. Cached curator lists, one card per verified entry.
 *   3. A small form at the top to subscribe to a new curator
 *      by hex pubkey.
 *
 * Every call goes through the Sprint 7 D1 path:
 *
 *   shell ─▶ coordinator /daemon/curators* ─▶ nexus-shell-daemon
 *
 * The helpers return `DaemonResult<T>` so "daemon offline"
 * is a first-class render path (banner + CTA) rather than an
 * error boundary trip.
 */

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { BookmarkPlus, RefreshCw, Trash2 } from "lucide-react";
import { useState } from "react";

import {
  isValidCuratorPubkey,
  listCurators,
  subscribeCurator,
  unsubscribeCurator,
  type CuratorListEntry,
  type DaemonResult,
  type DaemonCuratorsResponse,
  type SubscriptionsResponse,
} from "@/api/daemon";
import { DaemonOfflineBanner } from "@/pages/Browse";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import {
  selectActiveCoordinator,
  useProjectStore,
} from "@/stores/projectStore";

export default function Curators() {
  const active = useProjectStore(selectActiveCoordinator);

  if (!active) {
    return (
      <div className="space-y-4">
        <PageHeader />
        <Card>
          <CardHeader>
            <CardTitle>Aucun coordinateur sélectionné</CardTitle>
            <CardDescription>
              Ajoute un coordinateur depuis l'en-tête pour gérer
              les curators via son daemon.
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  return <CuratorsContent coordUrl={active.url} />;
}

function PageHeader() {
  return (
    <div>
      <h1 className="text-2xl font-bold">Curators</h1>
      <p className="text-sm text-muted-foreground">
        Abonne-toi à des listes signées Ed25519. Chaque liste
        voitche des projets publics que tu verras dans{" "}
        <strong>Explorer</strong>.
      </p>
    </div>
  );
}

function CuratorsContent({ coordUrl }: { coordUrl: string }) {
  const queryClient = useQueryClient();
  const [pubkeyInput, setPubkeyInput] = useState("");
  const [formError, setFormError] = useState<string | null>(null);

  const query = useQuery({
    queryKey: ["daemon-curators", coordUrl],
    queryFn: () => listCurators(coordUrl),
    staleTime: 30_000,
    refetchOnWindowFocus: false,
  });

  const subscribeMutation = useMutation({
    mutationFn: (hex: string) => subscribeCurator(coordUrl, hex),
    onSuccess: (result: DaemonResult<SubscriptionsResponse>) => {
      if (result.kind === "data") {
        setPubkeyInput("");
        setFormError(null);
        queryClient.invalidateQueries({
          queryKey: ["daemon-curators", coordUrl],
        });
        queryClient.invalidateQueries({
          queryKey: ["daemon-browse", coordUrl],
        });
      } else {
        setFormError(result.reason);
      }
    },
    onError: (err: unknown) => {
      setFormError(err instanceof Error ? err.message : "erreur inconnue");
    },
  });

  const unsubscribeMutation = useMutation({
    mutationFn: (hex: string) => unsubscribeCurator(coordUrl, hex),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["daemon-curators", coordUrl],
      });
      queryClient.invalidateQueries({
        queryKey: ["daemon-browse", coordUrl],
      });
    },
  });

  const handleSubscribe = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const candidate = pubkeyInput.trim().toLowerCase();
    if (!isValidCuratorPubkey(candidate)) {
      setFormError(
        "La clé publique doit faire 64 caractères hexadécimaux minuscules.",
      );
      return;
    }
    setFormError(null);
    subscribeMutation.mutate(candidate);
  };

  return (
    <div className="space-y-4">
      <div className="flex items-start justify-between gap-3">
        <PageHeader />
        {/* Sprint 8 audit F-1: mirror Browse's manual refresh so
            the user can force a list re-fetch when they bring
            the daemon up after the page is already open. */}
        <Button
          variant="outline"
          size="sm"
          onClick={() => query.refetch()}
          disabled={query.isFetching}
          data-testid="curators-refresh"
        >
          <RefreshCw className="mr-2 h-4 w-4" />
          Rafraîchir
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Ajouter un curator</CardTitle>
          <CardDescription>
            Colle la clé publique Ed25519 (64 caractères
            hexadécimaux) d'un curator de confiance.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form
            className="flex flex-col gap-3 md:flex-row md:items-end"
            onSubmit={handleSubscribe}
          >
            <div className="flex-1">
              <label
                htmlFor="curator-pubkey"
                className="mb-1 block text-xs font-medium text-muted-foreground"
              >
                Clé publique
              </label>
              <Input
                id="curator-pubkey"
                data-testid="curator-pubkey-input"
                value={pubkeyInput}
                onChange={(e) => setPubkeyInput(e.target.value)}
                placeholder="abcd1234…"
                autoComplete="off"
                spellCheck={false}
              />
              {formError ? (
                <p
                  className="mt-1 text-xs text-destructive"
                  data-testid="curator-form-error"
                >
                  {formError}
                </p>
              ) : null}
            </div>
            <Button
              type="submit"
              disabled={subscribeMutation.isPending}
              data-testid="curator-subscribe-submit"
            >
              <BookmarkPlus className="mr-2 h-4 w-4" />
              S'abonner
            </Button>
          </form>
        </CardContent>
      </Card>

      {query.isLoading ? (
        <Card>
          <CardContent className="py-6 text-sm text-muted-foreground">
            Chargement…
          </CardContent>
        </Card>
      ) : query.isError ? (
        <Card>
          <CardHeader>
            <CardTitle>Erreur réseau</CardTitle>
            <CardDescription>
              {query.error instanceof Error
                ? query.error.message
                : "erreur inconnue"}
            </CardDescription>
          </CardHeader>
        </Card>
      ) : (
        <CuratorListView
          result={query.data!}
          onUnsubscribe={(hex) => unsubscribeMutation.mutate(hex)}
        />
      )}
    </div>
  );
}

function CuratorListView({
  result,
  onUnsubscribe,
}: {
  result: DaemonResult<DaemonCuratorsResponse>;
  onUnsubscribe: (hex: string) => void;
}) {
  if (result.kind === "unavailable") {
    return <DaemonOfflineBanner reason={result.reason} />;
  }
  if (result.kind === "error") {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Proxy daemon refusé</CardTitle>
          <CardDescription>{result.reason}</CardDescription>
        </CardHeader>
      </Card>
    );
  }

  const { entries, subscribed_curators } = result.body;

  if (subscribed_curators.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Aucun curator suivi</CardTitle>
          <CardDescription>
            Ajoute un curator ci-dessus pour commencer à recevoir
            ses listes signées.
          </CardDescription>
        </CardHeader>
      </Card>
    );
  }

  const byCurator = new Map(entries.map((entry) => [hexFromBytes(entry.curator_pubkey), entry]));

  return (
    <div className="space-y-3" data-testid="curator-list">
      {subscribed_curators.map((hex) => (
        <CuratorRow
          key={hex}
          pubkeyHex={hex}
          entry={byCurator.get(hex)}
          onUnsubscribe={onUnsubscribe}
        />
      ))}
    </div>
  );
}

function CuratorRow({
  pubkeyHex,
  entry,
  onUnsubscribe,
}: {
  pubkeyHex: string;
  entry: CuratorListEntry | undefined;
  onUnsubscribe: (hex: string) => void;
}) {
  const name = entry?.list.curator_name ?? "Curator inconnu";
  const revision = entry?.list.revision ?? null;
  const count = entry?.list.entries.length ?? 0;
  const status = entry === undefined ? "waiting" : "active";

  return (
    <Card data-testid="curator-row">
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between gap-2">
          <div>
            <CardTitle className="text-base">{name}</CardTitle>
            <CardDescription className="font-mono text-xs">
              {truncateHex(pubkeyHex)}
            </CardDescription>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => onUnsubscribe(pubkeyHex)}
            data-testid="curator-unsubscribe"
          >
            <Trash2 className="mr-2 h-4 w-4" />
            Retirer
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-2 text-sm">
        {status === "waiting" ? (
          <p className="text-muted-foreground">
            En attente d'une première annonce gossip…
          </p>
        ) : (
          <div className="flex items-center gap-2 text-muted-foreground">
            <Badge variant="outline">{count} projet(s) vouché(s)</Badge>
            {revision !== null ? (
              <Badge variant="outline">rev. {revision}</Badge>
            ) : null}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function hexFromBytes(bytes: number[]): string {
  return bytes
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

function truncateHex(hex: string): string {
  if (hex.length <= 16) return hex;
  return `${hex.slice(0, 8)}…${hex.slice(-8)}`;
}

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = Curators;
