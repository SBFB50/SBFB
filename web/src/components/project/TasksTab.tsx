// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 5 Phase B — project Tasks tab.
 *
 * Table of the coordinator's tasks via `GET /tasks`. Read-only —
 * task submission goes through the apps or the CLI in this
 * sprint; Phase B does not build a task submission form.
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
import type { TasksList, TaskRow } from "@/api/coordinator";
import { formatHash } from "@/lib/format";

interface Props {
  query: UseQueryResult<TasksList, Error>;
}

export function TasksTab({ query }: Props) {
  if (query.isLoading) {
    return <Card><CardContent className="p-6 text-sm text-muted-foreground">Chargement…</CardContent></Card>;
  }
  if (query.isError) {
    return (
      <Card>
        <CardContent className="p-6 text-sm text-destructive">
          Erreur lors du fetch des tâches : {query.error.message}
        </CardContent>
      </Card>
    );
  }
  const tasks = query.data?.tasks ?? [];

  return (
    <Card>
      <CardHeader>
        <CardTitle>Tâches</CardTitle>
        <CardDescription>
          {tasks.length > 0
            ? `${tasks.length} tâche(s) — les 100 plus récentes`
            : "Aucune tâche soumise."}
        </CardDescription>
      </CardHeader>
      <CardContent>
        {tasks.length === 0 ? (
          <p className="text-xs text-muted-foreground">
            Les tâches apparaissent ici dès qu'une app ou un appel
            direct à <code className="font-mono">POST /tasks/submit</code>{" "}
            en soumet une.
          </p>
        ) : (
          <TasksTable rows={tasks} />
        )}
      </CardContent>
    </Card>
  );
}

function TasksTable({ rows }: { rows: TaskRow[] }) {
  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left text-xs">
        <thead>
          <tr className="border-b border-border text-[10px] uppercase tracking-wider text-muted-foreground">
            <th className="py-2 pr-3">Task id</th>
            <th className="py-2 pr-3">État</th>
            <th className="py-2 pr-3">Soumis</th>
            <th className="py-2 pr-3">Claim par</th>
            <th className="py-2 pr-3">Terminé</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.task_id} className="border-b border-border/50">
              <td className="py-2 pr-3 font-mono">{formatHash(row.task_id, 12)}</td>
              <td className="py-2 pr-3">
                <Badge
                  variant="outline"
                  className={stateBadgeClass(row.state)}
                >
                  {row.state}
                </Badge>
              </td>
              <td className="py-2 pr-3 text-muted-foreground">
                {typeof row.submitted_at === "number"
                  ? new Date(row.submitted_at * 1000).toLocaleTimeString()
                  : row.submitted_at}
              </td>
              <td className="py-2 pr-3 font-mono text-muted-foreground">
                {row.claimed_by ? formatHash(row.claimed_by, 10) : "—"}
              </td>
              <td className="py-2 pr-3 text-muted-foreground">
                {row.completed_at
                  ? new Date(row.completed_at * 1000).toLocaleTimeString()
                  : "—"}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function stateBadgeClass(state: string): string {
  if (state === "completed") return "border-emerald-500/40 text-emerald-500";
  if (state === "claimed") return "border-yellow-500/40 text-yellow-500";
  if (state === "failed") return "border-destructive/40 text-destructive";
  return "border-border text-muted-foreground";
}
