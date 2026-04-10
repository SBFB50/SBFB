/**
 * Sprint 5 Phase B — project Apps tab.
 *
 * Lists every app mounted on the coordinator (from `/app`) and
 * lets the user expand each one to see its manifest
 * (`/app/{name}/manifest`). Per decision D2, tab descriptors
 * are rendered as raw JSON in a `<pre>` block; async ones are
 * re-fetched via `/app/{name}/tabs/{tab_name}/descriptor` on
 * click ("Invoquer"). Sprint 6 will replace this raw rendering
 * with the schema-driven vocabulary listed in sprint5_plan.md
 * §2.2.
 */

import { useState } from "react";
import { type UseQueryResult, useQuery } from "@tanstack/react-query";
import { ChevronDown, ChevronRight, Play } from "lucide-react";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  type AppManifest,
  type AppsList,
  getAppManifest,
} from "@/api/coordinator";

const ASYNC_NOTE_KIND = "async descriptor";

interface Props {
  url: string;
  query: UseQueryResult<AppsList, Error>;
}

export function AppsTab({ url, query }: Props) {
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  if (query.isLoading) {
    return <Card><CardContent className="p-6 text-sm text-muted-foreground">Chargement…</CardContent></Card>;
  }
  if (query.isError) {
    return (
      <Card>
        <CardContent className="p-6 text-sm text-destructive">
          Erreur fetch apps : {query.error.message}
        </CardContent>
      </Card>
    );
  }
  const apps = query.data?.apps ?? [];

  return (
    <div className="space-y-3">
      {apps.length === 0 ? (
        <Card>
          <CardContent className="p-6 text-xs text-muted-foreground">
            Aucune app installée. Une app est une dépendance Python
            déclarée via <code className="font-mono">entry_points</code>{" "}
            du groupe <code className="font-mono">nexus.apps</code>.
          </CardContent>
        </Card>
      ) : (
        apps.map((app) => {
          const isOpen = expanded.has(app.name);
          return (
            <AppAccordion
              key={app.name}
              url={url}
              name={app.name}
              version={app.version}
              description={app.description}
              routesCount={app.routes}
              workersCount={app.workers}
              tabsCount={app.tabs}
              isOpen={isOpen}
              onToggle={() => {
                setExpanded((prev) => {
                  const next = new Set(prev);
                  if (next.has(app.name)) next.delete(app.name);
                  else next.add(app.name);
                  return next;
                });
              }}
            />
          );
        })
      )}
    </div>
  );
}

function AppAccordion({
  url,
  name,
  version,
  description,
  routesCount,
  workersCount,
  tabsCount,
  isOpen,
  onToggle,
}: {
  url: string;
  name: string;
  version: string;
  description: string;
  routesCount: number;
  workersCount: number;
  tabsCount: number;
  isOpen: boolean;
  onToggle: () => void;
}) {
  const manifestQuery = useQuery({
    queryKey: ["app-manifest", url, name],
    queryFn: () => getAppManifest(url, name),
    enabled: isOpen,
    staleTime: 30_000,
  });

  return (
    <Card>
      <CardHeader className="cursor-pointer" onClick={onToggle}>
        <div className="flex items-center gap-2">
          {isOpen ? (
            <ChevronDown className="h-4 w-4 text-muted-foreground" />
          ) : (
            <ChevronRight className="h-4 w-4 text-muted-foreground" />
          )}
          <div className="min-w-0 flex-1">
            <CardTitle className="text-base">{name}</CardTitle>
            <CardDescription>{description || "—"}</CardDescription>
          </div>
          <Badge variant="outline">v{version}</Badge>
          <Badge variant="outline" className="text-[10px]">
            {routesCount} route / {workersCount} worker / {tabsCount} tab
          </Badge>
        </div>
      </CardHeader>
      {isOpen && (
        <CardContent className="space-y-4">
          {manifestQuery.isLoading && (
            <p className="text-xs text-muted-foreground">
              Chargement du manifest…
            </p>
          )}
          {manifestQuery.isError && (
            <p className="text-xs text-destructive">
              Erreur fetch manifest : {manifestQuery.error.message}
            </p>
          )}
          {manifestQuery.data && (
            <ManifestSections
              url={url}
              appName={name}
              manifest={manifestQuery.data}
            />
          )}
        </CardContent>
      )}
    </Card>
  );
}

function ManifestSections({
  url,
  appName,
  manifest,
}: {
  url: string;
  appName: string;
  manifest: AppManifest;
}) {
  return (
    <div className="space-y-4">
      <Section title="Routes">
        {manifest.routes.length === 0 ? (
          <EmptyRow />
        ) : (
          <ul className="space-y-1.5 text-xs">
            {manifest.routes.map((r) => (
              <li key={r.path} className="flex items-center gap-2">
                <Badge variant="outline" className="font-mono text-[10px]">
                  {r.methods.join(",")}
                </Badge>
                <code className="font-mono">
                  /app/{appName}
                  {r.path}
                </code>
              </li>
            ))}
          </ul>
        )}
      </Section>

      <Section title="Workers">
        {manifest.workers.length === 0 ? (
          <EmptyRow />
        ) : (
          <ul className="space-y-1.5 text-xs">
            {manifest.workers.map((w) => (
              <li key={w.name} className="flex items-center gap-2">
                <span className="font-medium">{w.name}</span>
                <code className="text-muted-foreground">{w.model}</code>
              </li>
            ))}
          </ul>
        )}
      </Section>

      <Section title="Tabs">
        {manifest.tabs.length === 0 ? (
          <EmptyRow />
        ) : (
          <ul className="space-y-3">
            {manifest.tabs.map((t) => (
              <TabRow
                key={t.name}
                url={url}
                appName={appName}
                tabName={t.name}
                icon={t.icon}
                initialDescriptor={t.descriptor}
              />
            ))}
          </ul>
        )}
      </Section>
    </div>
  );
}

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <p className="mb-2 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        {title}
      </p>
      {children}
    </div>
  );
}

function EmptyRow() {
  return (
    <p className="text-xs italic text-muted-foreground">aucune entrée</p>
  );
}

function TabRow({
  url,
  appName,
  tabName,
  icon,
  initialDescriptor,
}: {
  url: string;
  appName: string;
  tabName: string;
  icon: string;
  initialDescriptor: unknown;
}) {
  const [invokedDescriptor, setInvokedDescriptor] = useState<unknown>(null);
  const [invoking, setInvoking] = useState(false);
  const [invokeError, setInvokeError] = useState<string | null>(null);

  const isAsyncPlaceholder =
    typeof initialDescriptor === "object" &&
    initialDescriptor !== null &&
    "note" in initialDescriptor &&
    typeof (initialDescriptor as { note: unknown }).note === "string" &&
    ((initialDescriptor as { note: string }).note.includes(ASYNC_NOTE_KIND));

  const descriptor =
    invokedDescriptor !== null ? invokedDescriptor : initialDescriptor;

  const onInvoke = async () => {
    setInvoking(true);
    setInvokeError(null);
    try {
      const res = await fetch(
        `${url}/app/${encodeURIComponent(appName)}/tabs/${encodeURIComponent(tabName)}/descriptor`,
        { headers: { accept: "application/json" } },
      );
      if (!res.ok) {
        throw new Error(`HTTP ${res.status}`);
      }
      const body = (await res.json()) as { descriptor: unknown };
      setInvokedDescriptor(body.descriptor);
    } catch (e) {
      setInvokeError(e instanceof Error ? e.message : "erreur inconnue");
    } finally {
      setInvoking(false);
    }
  };

  return (
    <li className="rounded-md border border-border bg-muted/20 p-3">
      <div className="mb-2 flex items-center gap-2">
        <span className="text-xs font-medium">{tabName}</span>
        <Badge variant="outline" className="text-[10px]">
          {icon}
        </Badge>
        {isAsyncPlaceholder && (
          <Button
            size="xs"
            variant="outline"
            onClick={onInvoke}
            disabled={invoking}
            className="ml-auto"
          >
            <Play className="h-3 w-3" />
            {invoking ? "Invocation…" : "Invoquer"}
          </Button>
        )}
      </div>
      {invokeError && (
        <p className="mb-2 text-[11px] text-destructive">{invokeError}</p>
      )}
      <pre className="max-h-60 overflow-auto rounded bg-background/70 p-2 text-[11px] leading-snug">
        {JSON.stringify(descriptor, null, 2)}
      </pre>
    </li>
  );
}
