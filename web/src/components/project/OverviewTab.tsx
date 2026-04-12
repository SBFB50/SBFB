// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 5 Phase B — project Overview tab.
 *
 * Aggregates the project metadata (/project), the live health
 * status (/health), and derived counters from the other tabs'
 * queries (tasks count, kudos count, distinct worker set, apps
 * count). Read-only — no mutations here.
 */

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import type { Health, Project } from "@/api/coordinator";

interface Props {
  project: Project | undefined;
  health: Health | undefined;
  tasksCount: number;
  kudosCount: number;
  workerCount: number;
  appsCount: number;
}

export function OverviewTab({
  project,
  health,
  tasksCount,
  kudosCount,
  workerCount,
  appsCount,
}: Props) {
  return (
    <div className="space-y-4">
      <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
        <StatCard label="Tâches soumises" value={tasksCount} />
        <StatCard label="Entrées kudos" value={kudosCount} />
        <StatCard label="Workers distincts" value={workerCount} />
        <StatCard label="Apps installées" value={appsCount} />
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Identité du coordinateur</CardTitle>
          <CardDescription>
            Métadonnées depuis <code className="font-mono">/project</code>{" "}
            et <code className="font-mono">/health</code>.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <dl className="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-2 text-xs">
            <DtDd label="Nom" value={project?.name} />
            <DtDd label="Description" value={project?.description || "—"} />
            <DtDd label="Visibilité" value={project?.visibility} />
            <DtDd label="Doc id" value={project?.doc_id} mono />
            <DtDd label="Author id" value={project?.author_id} mono />
            <DtDd
              label="Tasks doc ticket"
              value={project?.tasks_doc_ticket_prefix}
              mono
            />
            <DtDd label="Node id" value={health?.node_id} mono />
            <DtDd label="Status" value={health?.status} />
            <DtDd label="Version" value={health?.version} />
          </dl>
        </CardContent>
      </Card>
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: number }) {
  return (
    <Card>
      <CardHeader className="pb-2">
        <CardDescription className="text-[10px] uppercase tracking-wider">
          {label}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <p className="text-3xl font-bold">{value}</p>
      </CardContent>
    </Card>
  );
}

function DtDd({
  label,
  value,
  mono = false,
}: {
  label: string;
  value: string | null | undefined;
  mono?: boolean;
}) {
  return (
    <>
      <dt className="text-muted-foreground">{label}</dt>
      <dd className={mono ? "truncate font-mono" : "truncate"}>
        {value ?? "—"}
      </dd>
    </>
  );
}
