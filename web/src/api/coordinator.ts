/**
 * Typed client for the `nexus-coordinator` HTTP API.
 *
 * Every response coming off the coordinator is parsed through a
 * Zod schema before reaching the React layer. No component in the
 * shell is allowed to call `fetch` directly — it must go through
 * a helper in this module. See `.planning/sprint5_plan.md` §4.3
 * and the operational rule R2 (no raw fetch, no `as any`).
 *
 * The types exported here (`Health`, `Project`, etc.) are
 * `z.infer`red from the schemas so every response shape has
 * exactly one source of truth.
 */

import { z } from "zod";

// =================================================================
// Generic helpers
// =================================================================

/**
 * Thrown when the coordinator returns HTTP 2xx but the body does
 * not match the expected schema. Carries the original Zod issues
 * so callers (and the React error boundary) can render a useful
 * diagnostic.
 */
export class CoordinatorProtocolError extends Error {
  public readonly issues: z.ZodIssue[];
  public readonly rawBody: unknown;

  constructor(
    endpoint: string,
    issues: z.ZodIssue[],
    rawBody: unknown,
  ) {
    super(
      `coordinator protocol error on ${endpoint}: ${issues
        .map((i) => `${i.path.join(".")}: ${i.message}`)
        .join(", ")}`,
    );
    this.name = "CoordinatorProtocolError";
    this.issues = issues;
    this.rawBody = rawBody;
  }
}

/**
 * Thrown when the coordinator returns a non-2xx status.
 */
export class CoordinatorHttpError extends Error {
  public readonly status: number;
  public readonly endpoint: string;

  constructor(endpoint: string, status: number, statusText: string) {
    super(
      `coordinator returned HTTP ${status} ${statusText} for ${endpoint}`,
    );
    this.name = "CoordinatorHttpError";
    this.status = status;
    this.endpoint = endpoint;
  }
}

/**
 * Normalize a user-entered coordinator URL:
 * - trims whitespace
 * - strips any trailing slash
 * - rejects empty strings (the caller shows a validation error)
 */
export function normalizeCoordinatorUrl(raw: string): string {
  const trimmed = raw.trim().replace(/\/+$/, "");
  if (!trimmed) {
    throw new Error("coordinator URL is empty");
  }
  return trimmed;
}

async function getJson<T>(
  baseUrl: string,
  path: string,
  schema: z.ZodType<T>,
  init?: RequestInit,
): Promise<T> {
  const url = `${baseUrl}${path}`;
  const res = await fetch(url, {
    ...init,
    headers: {
      accept: "application/json",
      ...(init?.headers ?? {}),
    },
  });
  if (!res.ok) {
    throw new CoordinatorHttpError(path, res.status, res.statusText);
  }
  const raw: unknown = await res.json();
  const parsed = schema.safeParse(raw);
  if (!parsed.success) {
    throw new CoordinatorProtocolError(path, parsed.error.issues, raw);
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
  const res = await fetch(url, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      accept: "application/json",
    },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    throw new CoordinatorHttpError(path, res.status, res.statusText);
  }
  const raw: unknown = await res.json();
  const parsed = schema.safeParse(raw);
  if (!parsed.success) {
    throw new CoordinatorProtocolError(path, parsed.error.issues, raw);
  }
  return parsed.data;
}

async function deleteJson<T>(
  baseUrl: string,
  path: string,
  schema: z.ZodType<T>,
): Promise<T> {
  const url = `${baseUrl}${path}`;
  const res = await fetch(url, {
    method: "DELETE",
    headers: { accept: "application/json" },
  });
  if (!res.ok) {
    throw new CoordinatorHttpError(path, res.status, res.statusText);
  }
  const raw: unknown = await res.json();
  const parsed = schema.safeParse(raw);
  if (!parsed.success) {
    throw new CoordinatorProtocolError(path, parsed.error.issues, raw);
  }
  return parsed.data;
}

// =================================================================
// Schemas — mirror the FastAPI response shapes verified in
// packages/nexus-coordinator/src/nexus_coordinator/api/*.py on
// 2026-04-10 and frozen in sprint5_plan.md §1.
// =================================================================

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
  state: z.string(),
  submitted_at: z.number().or(z.string()),
  claimed_by: z.string().nullable().optional(),
  claimed_at: z.number().nullable().optional(),
  completed_at: z.number().nullable().optional(),
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
  id: z.number(),
  worker_pubkey_hex: z.string(),
  task_id: z.string(),
  tokens: z.number(),
  quality_factor: z.number(),
  trust_multiplier: z.number(),
  amount: z.number(),
  awarded_at: z.number().or(z.string()),
  entry_hash_hex: z.string(),
});
export type KudosEntry = z.infer<typeof KudosEntrySchema>;

export const KudosListSchema = z.object({
  entries: z.array(KudosEntrySchema),
  count: z.number(),
});
export type KudosList = z.infer<typeof KudosListSchema>;

export const KudosVerifySchema = z.object({
  ok: z.boolean(),
  first_bad_row_id: z.number().nullable(),
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
});
export type AppSummary = z.infer<typeof AppSummarySchema>;

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
});
export type AppManifest = z.infer<typeof AppManifestSchema>;

export const ShellDiscoverEntrySchema = z.object({
  schema_version: z.literal(1),
  project_name: z.string(),
  node_id: z.string(),
  doc_id: z.string(),
  api_host: z.string(),
  api_port: z.number(),
  pid: z.number(),
  started_at: z.string(),
  visibility: z.enum(["public", "private"]),
});
export type ShellDiscoverEntry = z.infer<typeof ShellDiscoverEntrySchema>;

export const ShellDiscoverResponseSchema = z.object({
  schema_version: z.literal(1),
  coordinators: z.array(ShellDiscoverEntrySchema),
  count: z.number(),
  self: z.object({
    project_name: z.string(),
    node_id: z.string().nullable(),
    api_host: z.string(),
    api_port: z.number(),
  }),
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

export function getHealth(baseUrl: string, init?: RequestInit): Promise<Health> {
  return getJson(baseUrl, "/health", HealthSchema, init);
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
  const path = query ? `/tasks?${query}` : "/tasks";
  return getJson(baseUrl, path, TasksListSchema);
}

export function submitTask(
  baseUrl: string,
  body: SubmitTaskBody,
): Promise<SubmitTaskResponse> {
  return postJson(baseUrl, "/tasks/submit", body, SubmitTaskResponseSchema);
}

export function listKudos(
  baseUrl: string,
  params: { workerPubkeyHex?: string } = {},
): Promise<KudosList> {
  const qs = new URLSearchParams();
  if (params.workerPubkeyHex) qs.set("worker_pubkey_hex", params.workerPubkeyHex);
  const query = qs.toString();
  const path = query ? `/kudos?${query}` : "/kudos";
  return getJson(baseUrl, path, KudosListSchema);
}

export function verifyKudos(baseUrl: string): Promise<KudosVerify> {
  return getJson(baseUrl, "/kudos/verify", KudosVerifySchema);
}

export function createInvite(
  baseUrl: string,
  body: CreateInviteBody,
): Promise<CreateInviteResponse> {
  return postJson(baseUrl, "/invite/create", body, CreateInviteResponseSchema);
}

export function listInvites(baseUrl: string): Promise<InviteList> {
  return getJson(baseUrl, "/invite", InviteListSchema);
}

export function revokeInvite(
  baseUrl: string,
  inviteId: string,
): Promise<RevokeInviteResponse> {
  return deleteJson(
    baseUrl,
    `/invite/${encodeURIComponent(inviteId)}`,
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

export function shellDiscover(baseUrl: string): Promise<ShellDiscoverResponse> {
  return getJson(baseUrl, "/shell/discover", ShellDiscoverResponseSchema);
}

export function getWorkerState(baseUrl: string): Promise<WorkerStateResponse> {
  return getJson(baseUrl, "/worker-state", WorkerStateResponseSchema);
}
