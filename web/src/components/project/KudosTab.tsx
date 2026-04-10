/**
 * Sprint 5 Phase B — project Kudos tab.
 *
 * Shows the hash-chain integrity status at the top (via
 * /kudos/verify) and the full entries table below (/kudos).
 * The integrity badge is the primary signal — a broken chain
 * means the ledger has been tampered with and the shell must
 * make that impossible to miss.
 */

import type { UseQueryResult } from "@tanstack/react-query";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Check, X, ShieldAlert } from "lucide-react";
import type { KudosList, KudosVerify } from "@/api/coordinator";
import { formatHash } from "@/lib/format";

interface Props {
  query: UseQueryResult<KudosList, Error>;
  verifyQuery: UseQueryResult<KudosVerify, Error>;
}

export function KudosTab({ query, verifyQuery }: Props) {
  return (
    <div className="space-y-4">
      <IntegrityBadge verifyQuery={verifyQuery} />

      <Card>
        <CardHeader>
          <CardTitle>Registre kudos</CardTitle>
          <CardDescription>
            Une ligne par entrée append-only.{" "}
            {query.data?.count ?? 0} entrée(s).
          </CardDescription>
        </CardHeader>
        <CardContent>
          {query.isLoading ? (
            <p className="text-sm text-muted-foreground">Chargement…</p>
          ) : query.isError ? (
            <p className="text-sm text-destructive">
              Erreur fetch kudos : {query.error.message}
            </p>
          ) : (query.data?.entries ?? []).length === 0 ? (
            <p className="text-xs text-muted-foreground">
              Pas encore de kudos distribués. Les workers gagnent
              des kudos dès qu'ils livrent un résultat vérifié par
              le validator.
            </p>
          ) : (
            <KudosTable entries={query.data!.entries} />
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function IntegrityBadge({
  verifyQuery,
}: {
  verifyQuery: UseQueryResult<KudosVerify, Error>;
}) {
  if (verifyQuery.isLoading) {
    return (
      <Card>
        <CardContent className="flex items-center gap-3 p-4 text-sm text-muted-foreground">
          Vérification du hash-chain…
        </CardContent>
      </Card>
    );
  }
  if (verifyQuery.isError) {
    return (
      <Card>
        <CardContent className="flex items-center gap-3 p-4 text-sm text-destructive">
          <ShieldAlert className="h-5 w-5" />
          Impossible de vérifier l'intégrité : {verifyQuery.error.message}
        </CardContent>
      </Card>
    );
  }
  const v = verifyQuery.data!;
  if (v.ok) {
    return (
      <Card>
        <CardContent className="flex items-center gap-3 p-4">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-emerald-500/10 text-emerald-500">
            <Check className="h-4 w-4" />
          </div>
          <div>
            <p className="text-sm font-medium text-emerald-500">
              Hash chain valide
            </p>
            <p className="text-xs text-muted-foreground">
              Toutes les entrées sont signées et liées sans
              modification détectable.
            </p>
          </div>
          <Badge
            variant="outline"
            className="ml-auto border-emerald-500/40 text-emerald-500"
          >
            OK
          </Badge>
        </CardContent>
      </Card>
    );
  }
  return (
    <Card className="border-destructive/40">
      <CardContent className="flex items-center gap-3 p-4">
        <div className="flex h-8 w-8 items-center justify-center rounded-full bg-destructive/10 text-destructive">
          <X className="h-4 w-4" />
        </div>
        <div>
          <p className="text-sm font-medium text-destructive">
            Hash chain corrompue
          </p>
          <p className="text-xs text-muted-foreground">
            Première ligne corrompue : {v.first_bad_row_id ?? "?"}. Ce
            registre ne doit pas être considéré comme de confiance.
          </p>
        </div>
        <Badge variant="outline" className="ml-auto border-destructive/40 text-destructive">
          INVALIDE
        </Badge>
      </CardContent>
    </Card>
  );
}

function KudosTable({
  entries,
}: {
  entries: KudosList["entries"];
}) {
  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left text-xs">
        <thead>
          <tr className="border-b border-border text-[10px] uppercase tracking-wider text-muted-foreground">
            <th className="py-2 pr-3">#</th>
            <th className="py-2 pr-3">Worker</th>
            <th className="py-2 pr-3">Task</th>
            <th className="py-2 pr-3 text-right">Tokens</th>
            <th className="py-2 pr-3 text-right">Qualité</th>
            <th className="py-2 pr-3 text-right">Trust</th>
            <th className="py-2 pr-3 text-right">Montant</th>
          </tr>
        </thead>
        <tbody>
          {entries.map((e) => (
            <tr key={e.id} className="border-b border-border/50">
              <td className="py-2 pr-3 text-muted-foreground">{e.id}</td>
              <td className="py-2 pr-3 font-mono">
                {formatHash(e.worker_pubkey_hex, 12)}
              </td>
              <td className="py-2 pr-3 font-mono">
                {formatHash(e.task_id, 12)}
              </td>
              <td className="py-2 pr-3 text-right">{e.tokens}</td>
              <td className="py-2 pr-3 text-right">
                {e.quality_factor.toFixed(2)}
              </td>
              <td className="py-2 pr-3 text-right">
                {e.trust_multiplier.toFixed(2)}
              </td>
              <td className="py-2 pr-3 text-right font-medium">
                {e.amount.toFixed(2)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
