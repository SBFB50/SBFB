// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Typed client for the daemon HTTP API.
 *
 * Every response is parsed through a Zod schema before reaching the
 * React layer. No component in the shell is allowed to call `fetch`
 * directly — it must go through a helper in this module.
 *
 * Standard routes (tasks, kudos, invites, etc.) call the daemon at
 * `/api/v1/*`. App-specific routes (`/app/*`) still target the
 * coordinator Python runtime.
 */

import { z } from "zod";

import { authFetch } from "@/api/auth";
import { TabViewSchema, type TabView } from "@/components/app/tabview/schema";

// =================================================================
// Generic helpers
// =================================================================

export class ApiProtocolError extends Error {
  public readonly issues: z.ZodIssue[];
  public readonly rawBody: unknown;

  constructor(
    endpoint: string,
    issues: z.ZodIssue[],
    rawBody: unknown,
  ) {
    super(
      `API protocol error on ${endpoint}: ${issues
        .map((i) => `${i.path.join(".")}: ${i.message}`)
        .join(", ")}`,
    );
    this.name = "ApiProtocolError";
    this.issues = issues;
    this.rawBody = rawBody;
  }
}

export class ApiHttpError extends Error {
  public readonly status: number;
  public readonly endpoint: string;

  constructor(endpoint: string, status: number, statusText: string) {
    super(
      `API returned HTTP ${status} ${statusText} for ${endpoint}`,
    );
    this.name = "ApiHttpError";
    this.status = status;
    this.endpoint = endpoint;
  }
}

/** @deprecated Use {@link ApiProtocolError} */
export const CoordinatorProtocolError = ApiProtocolError;
/** @deprecated Use {@link ApiHttpError} */
export const CoordinatorHttpError = ApiHttpError;

export function normalizeApiUrl(raw: string): string {
  const trimmed = raw.trim().replace(/\/+$/, "");
  if (!trimmed) {
    throw new Error("API URL is empty");
  }
  return trimmed;
}

/** @deprecated Use {@link normalizeApiUrl} */
export const normalizeCoordinatorUrl = normalizeApiUrl;

async function getJson<T>(
  baseUrl: string,
  path: string,
  schema: z.ZodType<T>,
  init?: RequestInit,
): Promise<T> {
  const url = `${baseUrl}${path}`;
  const res = await authFetch(url, {
    ...init,
    headers: {
      accept: "application/json",
      ...(init?.headers ?? {}),
    },
  });
  if (!res.ok) {
    throw new ApiHttpError(path, res.status, res.statusText);
  }
  const raw: unknown = await res.json();
  const parsed = schema.safeParse(raw);
  if (!parsed.success) {
    throw new ApiProtocolError(path, parsed.error.issues, raw);
  }
  return parsed.data;
}

async function postJson<T>(
  baseUrl: string,
  path: string,
  body: unknown,
  schema: z.ZodType<T>,
): Promise<T> {
  const url = `${baseUrl}${path}`;
  const res = await authFetch(url, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      accept: "application/json",
    },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    throw new ApiHttpError(path, res.status, res.statusText);
  }
  const raw: unknown = await res.json();
  const parsed = schema.safeParse(raw);
  if (!parsed.success) {
    throw new ApiProtocolError(path, parsed.error.issues, raw);
  }
  return parsed.data;
}

async function deleteJson<T>(
  baseUrl: string,
  path: string,
  schema: z.ZodType<T>,
): Promise<T> {
  const url = `${baseUrl}${path}`;
  const res = await authFetch(url, {
    method: "DELETE",
    headers: { accept: "application/json" },
  });
  if (!res.ok) {
    throw new ApiHttpError(path, res.status, res.statusText);
  }
  const raw: unknown = await res.json();
  const parsed = schema.safeParse(raw);
  if (!parsed.success) {
    throw new ApiProtocolError(path, parsed.error.issues, raw);
  }
  return parsed.data;
}

// =================================================================
// Schemas — mirror the daemon Rust handler response shapes.
// =================================================================

const DaemonHealthRawSchema = z.object({
  status: z.string(),
  node_id: z.string(),
  daemon_version: z.string(),
  api_host: z.string(),
  api_port: z.number(),
  uptime_secs: z.number(),
});

export const HealthSchema = z.object({
  status: z.string(),
  project: z.string(),
  node_id: z.string().nullable(),
  doc_id: z.string().nullable(),
  author_id: z.string().nullable(),
  version: z.string(),
});
export type Health = z.infer<typeof HealthSchema>;

export const ProjectSchema = z.object({
  name: z.string(),
  description: z.string(),
  visibility: z.enum(["public", "private"]),
  doc_id: z.string().nullable(),
  author_id: z.string().nullable(),
  tasks_doc_ticket_prefix: z.string().nullable(),
});
export type Project = z.infer<typeof ProjectSchema>;

export const TaskRowSchema = z.object({
  task_id: z.string(),
  status: z.string(),
  project_id: z.string().optional(),
  model: z.string().optional(),
  created_at: z.number().or(z.string()).optional(),
  updated_at: z.number().or(z.string()).optional(),
  task_hash: z.string().optional(),
  worker_node_id: z.string().nullable().optional(),
  result_hash: z.string().nullable().optional(),
});
export type TaskRow = z.infer<typeof TaskRowSchema>;

export const TasksListSchema = z.object({
  tasks: z.array(TaskRowSchema),
  count: z.number(),
});
export type TasksList = z.infer<typeof TasksListSchema>;

export const SubmitTaskBodySchema = z.object({
  task_type: z.string().min(1),
  prompt: z.string().min(1),
  model: z.string().min(1),
  system_prompt: z.string().default(""),
  priority: z.number().int().min(1).max(10).default(5),
  parent_task_id: z.string().default(""),
  metadata: z.record(z.string(), z.string()).nullable().optional(),
  task_id: z.string().nullable().optional(),
});
export type SubmitTaskBody = z.infer<typeof SubmitTaskBodySchema>;

export const SubmitTaskResponseSchema = z.object({
  task_id: z.string(),
});
export type SubmitTaskResponse = z.infer<typeof SubmitTaskResponseSchema>;

export const KudosEntrySchema = z.object({
  entry_id: z.string(),
  worker_node_id: z.string(),
  task_id: z.string(),
  project_id: z.string(),
  amount: z.number(),
  created_at: z.number(),
  entry_hash: z.string(),
});
export type KudosEntry = z.infer<typeof KudosEntrySchema>;

export const KudosListSchema = z.object({
  entries: z.array(KudosEntrySchema),
  count: z.number(),
});
export type KudosList = z.infer<typeof KudosListSchema>;

export const KudosVerifySchema = z.object({
  valid: z.boolean(),
});
export type KudosVerify = z.infer<typeof KudosVerifySchema>;

export const InviteRecordSchema = z.object({
  id: z.string(),
  scope: z.string(),
  project_id: z.string(),
  expires_at: z.number(),
  max_uses: z.number().nullable(),
  uses_count: z.number(),
  revoked_at: z.number().nullable(),
  note: z.string().nullable(),
  created_at: z.number(),
});
export type InviteRecord = z.infer<typeof InviteRecordSchema>;

export const InviteListSchema = z.object({
  invites: z.array(InviteRecordSchema),
  count: z.number(),
});
export type InviteList = z.infer<typeof InviteListSchema>;

export const CreateInviteBodySchema = z.object({
  scope: z.enum(["worker", "observer"]).default("worker"),
  expiry_secs: z.number().int().min(60).default(7 * 24 * 3600),
  max_uses: z.number().int().min(1).nullable().default(null),
  note: z.string().nullable().default(null),
});
export type CreateInviteBody = z.infer<typeof CreateInviteBodySchema>;

export const CreateInviteResponseSchema = z.object({
  id: z.string(),
  wire: z.string(),
  scope: z.string(),
  expires_at: z.number(),
  max_uses: z.number().nullable(),
  note: z.string().nullable(),
});
export type CreateInviteResponse = z.infer<typeof CreateInviteResponseSchema>;

export const RevokeInviteResponseSchema = z.object({
  id: z.string(),
  revoked: z.boolean(),
});
export type RevokeInviteResponse = z.infer<typeof RevokeInviteResponseSchema>;

export const AppSummarySchema = z.object({
  name: z.string(),
  version: z.string(),
  description: z.string(),
  routes: z.number(),
  workers: z.number(),
  tabs: z.number(),
  // Sprint 8 Phase A (D2): the coordinator always ships a
  // `commands` count alongside the existing route/worker/tab
  // counters. Kept required (not `.default(0)`) because the
  // Sprint 8 Phase A coordinator always emits the field; an
  // older coordinator without the field is not a supported
  // pairing and would trip the schema validator loud.
  commands: z.number(),
});
export type AppSummary = z.infer<typeof AppSummarySchema>;

/**
 * Zod mirror of :class:`nexus_sdk.commands.CommandDescriptor`.
 *
 * Frozen at v1 — the Python side is `extra="forbid"` + `frozen=True`
 * so the shell never sees a field it doesn't know about.
 */
export const CommandDescriptorSchema = z
  .object({
    schema_version: z.literal(1),
    name: z.string().min(1).max(64),
    description: z.string().min(1).max(280),
    icon: z.string().max(32),
    group: z.string().max(32),
  })
  .strict();
export type CommandDescriptor = z.infer<typeof CommandDescriptorSchema>;

export const AppsListSchema = z.object({
  apps: z.array(AppSummarySchema),
  count: z.number(),
});
export type AppsList = z.infer<typeof AppsListSchema>;

export const AppManifestSchema = z.object({
  manifest: z.object({
    name: z.string(),
    version: z.string(),
    author: z.string(),
    description: z.string(),
    dependencies: z.array(z.string()),
    license: z.string(),
  }),
  routes: z.array(
    z.object({
      path: z.string(),
      methods: z.array(z.string()),
    }),
  ),
  workers: z.array(
    z.object({
      name: z.string(),
      model: z.string(),
    }),
  ),
  tabs: z.array(
    z.object({
      name: z.string(),
      icon: z.string(),
      descriptor: z.unknown(),
    }),
  ),
  // Sprint 8 Phase A (D2): full manifest always carries the
  // command list. Required (not `.default([])`) for the same
  // reason as `AppSummarySchema.commands` — the Sprint 8
  // coordinator always emits it.
  commands: z.array(CommandDescriptorSchema),
});
export type AppManifest = z.infer<typeof AppManifestSchema>;

export const ShellDiscoverCoordinatorSchema = z.object({
  node_id: z.string(),
  api_host: z.string(),
  api_port: z.number(),
  daemon_version: z.string(),
});

export const ShellDiscoverResponseSchema = z.object({
  schema_version: z.literal(1),
  coordinators: z.array(ShellDiscoverCoordinatorSchema),
  count: z.number(),
});
export type ShellDiscoverResponse = z.infer<typeof ShellDiscoverResponseSchema>;

// Worker state proxy response. The `state` object mirrors the
// Rust WorkerStateSnapshot verbatim — any schema drift surfaces
// as a Zod issue at runtime, which the shell surfaces as a
// "protocol error" banner rather than a crash.
export const GpuSnapshotSchema = z.object({
  name: z.string(),
  memory_total_mb: z.number().int().nonnegative(),
  memory_used_mb: z.number().int().nonnegative(),
  utilization_pct: z.number().int().min(0).max(100),
  temperature_c: z.number().int().nonnegative(),
  power_draw_w: z.number().nonnegative(),
});
export type GpuSnapshot = z.infer<typeof GpuSnapshotSchema>;

export const ProjectServedSchema = z.object({
  project_name: z.string(),
  doc_id: z.string(),
  kudos_total: z.number().int().nonnegative(),
  tasks_completed: z.number().int().nonnegative(),
});
export type ProjectServed = z.infer<typeof ProjectServedSchema>;

export const LastTaskSchema = z.object({
  task_id: z.string(),
  project_name: z.string(),
  prompt_preview: z.string(),
  status: z.string(),
  completed_at: z.string(),
});
export type LastTask = z.infer<typeof LastTaskSchema>;

export const WorkerStateV1Schema = z.object({
  schema_version: z.literal(1),
  node_id: z.string(),
  worker_version: z.string(),
  uptime_secs: z.number().int().nonnegative(),
  started_at: z.string(),
  last_updated_at: z.string(),
  gpu: GpuSnapshotSchema.nullable(),
  projects_served: z.array(ProjectServedSchema),
  last_task: LastTaskSchema.nullable(),
});
export type WorkerStateV1 = z.infer<typeof WorkerStateV1Schema>;

export const WorkerStateResponseSchema = z.discriminatedUnion("running", [
  z.object({
    running: z.literal(false),
    error: z.string().optional(),
  }),
  z.object({
    running: z.literal(true),
    stale: z.boolean(),
    state: WorkerStateV1Schema,
  }),
]);
export type WorkerStateResponse = z.infer<typeof WorkerStateResponseSchema>;

// =================================================================
// API functions — one per endpoint. Every caller goes through
// these so the component layer never touches `fetch` directly.
// =================================================================

export async function getHealth(baseUrl: string, init?: RequestInit): Promise<Health> {
  const raw = await getJson(baseUrl, "/api/v1/coordinator/health", DaemonHealthRawSchema, init);
  return {
    status: raw.status,
    project: "nexus-grid",
    node_id: raw.node_id,
    doc_id: null,
    author_id: null,
    version: raw.daemon_version,
  };
}

export function getProject(baseUrl: string, init?: RequestInit): Promise<Project> {
  return getJson(baseUrl, "/project", ProjectSchema, init);
}

export function listTasks(
  baseUrl: string,
  params: { state?: string; limit?: number } = {},
): Promise<TasksList> {
  const qs = new URLSearchParams();
  if (params.state) qs.set("state", params.state);
  if (params.limit !== undefined) qs.set("limit", String(params.limit));
  const query = qs.toString();
  const path = query ? `/api/v1/tasks?${query}` : "/api/v1/tasks";
  return getJson(baseUrl, path, TasksListSchema);
}

export function submitTask(
  baseUrl: string,
  body: SubmitTaskBody,
): Promise<SubmitTaskResponse> {
  return postJson(baseUrl, "/api/v1/tasks/submit", body, SubmitTaskResponseSchema);
}

export function listKudos(
  baseUrl: string,
  params: { workerNodeId?: string } = {},
): Promise<KudosList> {
  const qs = new URLSearchParams();
  if (params.workerNodeId) qs.set("worker_node_id", params.workerNodeId);
  const query = qs.toString();
  const path = query ? `/api/v1/kudos/entries?${query}` : "/api/v1/kudos/entries";
  return getJson(baseUrl, path, KudosListSchema);
}

export function verifyKudos(baseUrl: string, projectId?: string): Promise<KudosVerify> {
  const pid = projectId ?? "default";
  return getJson(baseUrl, `/api/v1/kudos/${encodeURIComponent(pid)}/verify`, KudosVerifySchema);
}

export function createInvite(
  baseUrl: string,
  body: CreateInviteBody,
): Promise<CreateInviteResponse> {
  return postJson(baseUrl, "/api/v1/invite/create", body, CreateInviteResponseSchema);
}

export function listInvites(baseUrl: string): Promise<InviteList> {
  return getJson(baseUrl, "/api/v1/invite", InviteListSchema);
}

export function revokeInvite(
  baseUrl: string,
  inviteId: string,
): Promise<RevokeInviteResponse> {
  return deleteJson(
    baseUrl,
    `/api/v1/invite/${encodeURIComponent(inviteId)}`,
    RevokeInviteResponseSchema,
  );
}

export function listApps(baseUrl: string): Promise<AppsList> {
  return getJson(baseUrl, "/app", AppsListSchema);
}

export function getAppManifest(
  baseUrl: string,
  name: string,
): Promise<AppManifest> {
  return getJson(
    baseUrl,
    `/app/${encodeURIComponent(name)}/manifest`,
    AppManifestSchema,
  );
}

/**
 * Sprint 6 Phase B — schema-driven tab descriptor fetch.
 *
 * Sprint 8 Phase A (D4) retired the one-release
 * `legacy_descriptor` fallback the Sprint 6 envelope used to
 * carry. The coordinator now returns `{ descriptor: <TabView> }`
 * on success or HTTP 422 on validation failure; the shell
 * surfaces a 422 as a regular error state instead of rendering
 * a degraded payload under a legacy flag.
 *
 * Return union shrinks to:
 *  - `{ kind: "schema", tabView }` — descriptor validated against
 *    the v1 TabView Zod schema and is safe to render
 *  - `{ kind: "error", message }` — HTTP or protocol error (422
 *    included — the coordinator surfaces its own TabView errors
 *    as 422 with a structured detail)
 */
export const AppTabDescriptorEnvelopeSchema = z.object({
  descriptor: z.unknown(),
});

export type AppTabDescriptorResult =
  | { kind: "schema"; tabView: TabView }
  | { kind: "error"; message: string };

export async function getAppTabDescriptor(
  baseUrl: string,
  appName: string,
  tabName: string,
): Promise<AppTabDescriptorResult> {
  const path = `/app/${encodeURIComponent(appName)}/tabs/${encodeURIComponent(tabName)}/descriptor`;
  try {
    const envelope = await getJson(baseUrl, path, AppTabDescriptorEnvelopeSchema);
    const parsed = TabViewSchema.safeParse(envelope.descriptor);
    if (!parsed.success) {
      return {
        kind: "error",
        message:
          "Tab descriptor failed Zod validation: " +
          parsed.error.issues
            .slice(0, 2)
            .map((i) => `${i.path.join(".") || "(root)"}: ${i.message}`)
            .join(" | "),
      };
    }
    return { kind: "schema", tabView: parsed.data };
  } catch (e) {
    if (e instanceof ApiHttpError || e instanceof ApiProtocolError) {
      return { kind: "error", message: e.message };
    }
    return {
      kind: "error",
      message: e instanceof Error ? e.message : "unknown error",
    };
  }
}

// =================================================================
// Sprint 8 Phase A — D1 submit_task + D2 commands
// =================================================================

export const SubmitAppTaskBodySchema = z.object({
  worker: z.string().min(1).max(128),
  payload: z.record(z.string(), z.unknown()).default({}),
  priority: z.number().int().min(0).max(10).default(5),
  parent_task_id: z.string().nullable().default(null),
});
export type SubmitAppTaskBody = z.infer<typeof SubmitAppTaskBodySchema>;

export const SubmitAppTaskResponseSchema = z.object({
  task_id: z.string(),
});
export type SubmitAppTaskResponse = z.infer<typeof SubmitAppTaskResponseSchema>;

/**
 * Submit a task via an app's :class:`AppContext.submit_task`
 * helper. The coordinator resolves `worker` (a routing key) to
 * a concrete ``WorkerDescriptor`` on the target app before
 * forwarding to the dispatcher.
 */
export function submitAppTask(
  baseUrl: string,
  appName: string,
  body: SubmitAppTaskBody,
): Promise<SubmitAppTaskResponse> {
  return postJson(
    baseUrl,
    `/app/${encodeURIComponent(appName)}/tasks/submit`,
    body,
    SubmitAppTaskResponseSchema,
  );
}

/**
 * Fetch an app's Sprint 8 D2 command palette descriptors.
 */
export function listAppCommands(
  baseUrl: string,
  appName: string,
): Promise<CommandDescriptor[]> {
  return getJson(
    baseUrl,
    `/app/${encodeURIComponent(appName)}/commands`,
    z.array(CommandDescriptorSchema),
  );
}

export const InvokeAppCommandResponseSchema = z.object({
  result: z.unknown(),
});
export type InvokeAppCommandResponse = z.infer<typeof InvokeAppCommandResponseSchema>;

/**
 * Invoke an app-provided command palette entry. The returned
 * ``result`` field is forwarded verbatim from the Python side;
 * the caller is expected to narrow it (typically to a
 * ``{navigation: {path: string}}`` shape for the command
 * palette navigate flow).
 */
export function invokeAppCommand(
  baseUrl: string,
  appName: string,
  cmdName: string,
): Promise<InvokeAppCommandResponse> {
  return postJson(
    baseUrl,
    `/app/${encodeURIComponent(appName)}/commands/${encodeURIComponent(cmdName)}/invoke`,
    {},
    InvokeAppCommandResponseSchema,
  );
}

export const SetAppStateResponseSchema = z.object({
  ok: z.literal(true),
});
export type SetAppStateResponse = z.infer<typeof SetAppStateResponseSchema>;

/**
 * Sprint 9 Phase B (D1 typed namespace setter). Push a JSON
 * payload into one of an app's typed storage namespaces.
 *
 * The coordinator dispatches the body through the
 * :class:`nexus_sdk.TypedNamespace` registered by the app on
 * :attr:`AppContext.namespaces` at ``on_start`` time, validating
 * against the bound Pydantic schema before persisting. The
 * caller therefore does not need to know the schema — it only
 * needs to send a JSON object that the app's namespace will
 * accept. The route returns 422 with a structured detail when
 * the body fails validation.
 *
 * Used by the gov Politiciens filter persist flow (Phase B
 * consumer): the Playwright spec POSTs the active filter to
 * ``namespaceKey="politicians_filter"`` then reloads the page
 * to assert the descriptor still reflects the persisted state.
 */
export function setAppState(
  baseUrl: string,
  appName: string,
  namespaceKey: string,
  body: Record<string, unknown>,
): Promise<SetAppStateResponse> {
  return postJson(
    baseUrl,
    `/app/${encodeURIComponent(appName)}/state/${encodeURIComponent(namespaceKey)}`,
    body,
    SetAppStateResponseSchema,
  );
}

/**
 * Sprint 11 Phase C — determine if a browsed project is hosted on
 * the local coordinator and, if so, return its app list. Returns
 * `null` when the project lives on a different node.
 *
 * Fix sprint11 D-01: compare against the daemon's node_id (from
 * GET /daemon/info), not the coordinator's (GET /health). The
 * BrowseEntry project_id is the announcing daemon's iroh node_id
 * which differs from the coordinator's own iroh node_id.
 */
export async function getProjectApps(
  baseUrl: string,
  projectId: string,
): Promise<AppsList | null> {
  const { getDaemonInfo } = await import("./daemon");
  const info = await getDaemonInfo(baseUrl);
  if (info.kind !== "data" || info.body.node_id !== projectId) return null;
  return listApps(baseUrl);
}

export function shellDiscover(baseUrl: string): Promise<ShellDiscoverResponse> {
  return getJson(baseUrl, "/api/v1/shell/discover", ShellDiscoverResponseSchema);
}

export function getWorkerState(baseUrl: string): Promise<WorkerStateResponse> {
  return getJson(baseUrl, "/api/v1/worker/state", WorkerStateResponseSchema);
}
