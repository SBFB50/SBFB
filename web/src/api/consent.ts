// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Typed client for the `/consent/*` endpoints (Sprint 16 Phase C).
 *
 * Mirrors the Pydantic schema in
 * `packages/nexus-coordinator/src/nexus_coordinator/api/consent.py`
 * and the Rust struct in
 * `crates/nexus-worker-core/src/consent.rs`. All three sides MUST
 * agree byte-for-byte on the JSON shape — the worker watches the
 * file the coordinator writes from the body the dialog POSTs.
 */

import { z } from "zod";

import { authFetch } from "@/api/auth";

// =================================================================
// Schemas
// =================================================================

/** Consent level, stable on the wire as the integer 1..=4. */
export const ConsentLevelSchema = z.union([
  z.literal(1),
  z.literal(2),
  z.literal(3),
  z.literal(4),
]);
export type ConsentLevel = z.infer<typeof ConsentLevelSchema>;

export const CapsSchema = z.object({
  max_watts: z.number().int().min(1).max(2000).nullable(),
  max_vram_mb: z.number().int().min(1).nullable(),
  max_hours_day: z.number().min(0).max(24).nullable(),
});
export type Caps = z.infer<typeof CapsSchema>;

export const ConsentConfigSchema = z.object({
  level: ConsentLevelSchema,
  caps: CapsSchema,
  allowed_project_ids: z.array(z.string().regex(/^[0-9a-fA-F]{64}$/)),
  own_node_id: z.string(),
});
export type ConsentConfig = z.infer<typeof ConsentConfigSchema>;

/**
 * Default payload the dialog seeds when the coordinator returns
 * a fresh ConsentConfig (no prior save). Also used by tests as
 * the "blank slate" baseline.
 */
export const DEFAULT_CONSENT: ConsentConfig = {
  level: 1,
  caps: {
    max_watts: 400,
    max_vram_mb: 16 * 1024,
    max_hours_day: 12.0,
  },
  allowed_project_ids: [],
  own_node_id: "",
};

// =================================================================
// HTTP helpers (loopback through authFetch)
// =================================================================

class ConsentHttpError extends Error {
  readonly status: number;

  constructor(endpoint: string, status: number, statusText: string) {
    super(`coordinator returned HTTP ${status} ${statusText} for ${endpoint}`);
    this.name = "ConsentHttpError";
    this.status = status;
  }
}

async function consentGet(baseUrl: string): Promise<ConsentConfig> {
  const res = await authFetch(`${baseUrl}/consent/get`, {
    headers: { accept: "application/json" },
  });
  if (!res.ok) {
    throw new ConsentHttpError("/consent/get", res.status, res.statusText);
  }
  const body: unknown = await res.json();
  return ConsentConfigSchema.parse(body);
}

async function consentPost<TBody>(
  baseUrl: string,
  path: string,
  body: TBody,
): Promise<ConsentConfig> {
  const res = await authFetch(`${baseUrl}${path}`, {
    method: "POST",
    headers: {
      accept: "application/json",
      "content-type": "application/json",
    },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    throw new ConsentHttpError(path, res.status, res.statusText);
  }
  const raw: unknown = await res.json();
  return ConsentConfigSchema.parse(raw);
}

// =================================================================
// Public API surface
// =================================================================

export function getConsent(baseUrl: string): Promise<ConsentConfig> {
  return consentGet(baseUrl);
}

export function setConsent(
  baseUrl: string,
  cfg: ConsentConfig,
): Promise<ConsentConfig> {
  return consentPost(baseUrl, "/consent/set", cfg);
}

export function addToWhitelist(
  baseUrl: string,
  projectId: string,
): Promise<ConsentConfig> {
  return consentPost(baseUrl, "/consent/whitelist/add", {
    project_id: projectId,
  });
}

export function removeFromWhitelist(
  baseUrl: string,
  projectId: string,
): Promise<ConsentConfig> {
  return consentPost(baseUrl, "/consent/whitelist/remove", {
    project_id: projectId,
  });
}
