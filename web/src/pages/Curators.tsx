// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/curators` — glassmorphism curator management page.
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
import {
  selectActiveCoordinator,
  useProjectStore,
} from "@/stores/projectStore";

export default function Curators() {
  const active = useProjectStore(selectActiveCoordinator);

  if (!active) {
    return (
      <div className="space-y-6">
        <PageHeader />
        <div className="glass-card max-w-md p-6">
          <h3 className="mb-2 font-bold">
            Aucun noeud actif
          </h3>
          <p className="text-sm text-white/50">
            Connecte-toi a un noeud depuis l'en-tete pour gerer
            les curators via son daemon.

          </p>
        </div>
      </div>
    );
  }

  return <CuratorsContent coordUrl={active.url} />;
}

function PageHeader() {
  return (
    <div>
      <h1 className="text-3xl font-extrabold tracking-tight">
        Curators
      </h1>
      <p className="mt-1 text-sm text-white/50">
        Abonne-toi a des listes signees Ed25519. Chaque liste
        voitche des projets publics que tu verras dans{" "}
        <strong className="text-white/70">Explorer</strong>.
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
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-3">
        <PageHeader />
        <button
          onClick={() => query.refetch()}
          disabled={query.isFetching}
          className="glass-pill flex items-center gap-2 text-xs"
          data-testid="curators-refresh"
        >
          <RefreshCw
            className={`h-3.5 w-3.5 ${query.isFetching ? "animate-spin" : ""}`}
          />
          Rafraichir
        </button>
      </div>

      <div className="glass-card p-6">
        <h3 className="mb-1 font-bold">Ajouter un curator</h3>
        <p className="mb-4 text-sm text-white/50">
          Colle la cle publique Ed25519 (64 caracteres
          hexadecimaux) d'un curator.
        </p>
        <form
          className="flex flex-col gap-3 md:flex-row md:items-end"
          onSubmit={handleSubscribe}
        >
          <div className="flex-1">
            <label
              htmlFor="curator-pubkey"
              className="mb-1 block text-xs font-medium text-white/40"
            >
              Cle publique
            </label>
            <input
              id="curator-pubkey"
              data-testid="curator-pubkey-input"
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
                data-testid="curator-form-error"
              >
                {formError}
              </p>
            ) : null}
          </div>
          <button
            type="submit"
            disabled={subscribeMutation.isPending}
            className="flex items-center gap-2 rounded-lg bg-purple-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-purple-500 disabled:opacity-50"
            data-testid="curator-subscribe-submit"
          >
            <BookmarkPlus className="h-4 w-4" />
            S'abonner
          </button>
        </form>
      </div>

      {query.isLoading ? (
        <div className="glass-card p-6 text-sm text-white/50">
          Chargement...
        </div>
      ) : query.isError ? (
        <div className="glass-card p-6">
          <h3 className="mb-1 font-bold text-red-300">Erreur reseau</h3>
          <p className="text-sm text-white/50">
            {query.error instanceof Error
              ? query.error.message
              : "erreur inconnue"}
          </p>
        </div>
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
      <div className="glass-card p-6">
        <h3 className="mb-1 font-bold text-red-300">Proxy daemon refuse</h3>
        <p className="text-sm text-white/50">{result.reason}</p>
      </div>
    );
  }

  const { entries, subscribed_curators } = result.body;

  if (subscribed_curators.length === 0) {
    return (
      <div className="glass-card p-6">
        <h3 className="mb-1 font-bold">Aucun curator suivi</h3>
        <p className="text-sm text-white/50">
          Ajoute un curator ci-dessus pour commencer a recevoir
          ses listes signees.
        </p>
      </div>
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
    <div className="glass-card p-5" data-testid="curator-row">
      <div className="flex items-start justify-between gap-2">
        <div>
          <h3 className="text-base font-bold">{name}</h3>
          <p className="font-mono text-xs text-white/40">
            {truncateHex(pubkeyHex)}
          </p>
        </div>
        <button
          onClick={() => onUnsubscribe(pubkeyHex)}
          className="flex items-center gap-2 rounded-lg bg-white/[0.06] px-3 py-1.5 text-xs text-white/60 transition-colors hover:bg-red-500/20 hover:text-red-300"
          data-testid="curator-unsubscribe"
        >
          <Trash2 className="h-3.5 w-3.5" />
          Retirer
        </button>
      </div>
      <div className="mt-3 text-sm">
        {status === "waiting" ? (
          <p className="text-white/40">
            En attente d'une premiere annonce gossip...
          </p>
        ) : (
          <div className="flex items-center gap-2">
            <span className="glass-pill py-0.5 text-[10px] text-white/60">
              {count} projet(s) vouche(s)
            </span>
            {revision !== null ? (
              <span className="glass-pill py-0.5 text-[10px] text-white/60">
                rev. {revision}
              </span>
            ) : null}
          </div>
        )}
      </div>
    </div>
  );
}

function hexFromBytes(bytes: number[]): string {
  return bytes
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

function truncateHex(hex: string): string {
  if (hex.length <= 16) return hex;
  return `${hex.slice(0, 8)}...${hex.slice(-8)}`;
}

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = Curators;
