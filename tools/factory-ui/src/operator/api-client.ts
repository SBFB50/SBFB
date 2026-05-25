// SPDX-License-Identifier: AGPL-3.0-or-later

const BASE_URL = "http://127.0.0.1:3001";

async function fetchApi<T>(
  path: string,
  options?: RequestInit,
): Promise<T> {
  const res = await fetch(`${BASE_URL}${path}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...options?.headers,
    },
  });
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new Error(`API ${res.status}: ${text}`);
  }
  return res.json();
}

export function getStatus() {
  return fetchApi<Record<string, unknown>>("/api/status");
}

export function getLint() {
  return fetchApi<Record<string, unknown>>("/api/lint");
}

export function getAudit(rev: string) {
  return fetchApi<Record<string, unknown>>(`/api/audit/${encodeURIComponent(rev)}`);
}

export function getPrompt(kind: string) {
  return fetchApi<Record<string, unknown>>(`/api/prompt/${encodeURIComponent(kind)}`);
}

export function getContext() {
  return fetchApi<Record<string, unknown>>("/api/context");
}

export function postContextPack(body: Record<string, unknown>) {
  return fetchApi<Record<string, unknown>>("/api/context-pack", {
    method: "POST",
    body: JSON.stringify(body),
  });
}

export function getProviders() {
  return fetchApi<Record<string, unknown>>("/api/providers");
}

export function postActionRun(body: { command: string; args?: Record<string, unknown> }) {
  return fetchApi<Record<string, unknown>>("/api/actions/run", {
    method: "POST",
    body: JSON.stringify(body),
  });
}

export function getActionLog() {
  return fetchApi<Array<Record<string, unknown>>>("/api/actions/log");
}

export function postArtifactDraft(body: { path: string; content: string }) {
  return fetchApi<Record<string, unknown>>("/api/artifacts/draft", {
    method: "POST",
    body: JSON.stringify(body),
  });
}

export function postChatSession(body: { context_pack: Record<string, unknown> }) {
  return fetchApi<Record<string, unknown>>("/api/chat/session", {
    method: "POST",
    body: JSON.stringify(body),
  });
}

export function postChatMessage(body: { session_id: string; message: string }) {
  return fetchApi<Record<string, unknown>>("/api/chat/message", {
    method: "POST",
    body: JSON.stringify(body),
  });
}

export function getChatLog(id: string) {
  return fetchApi<Record<string, unknown>>(`/api/chat/${encodeURIComponent(id)}/log`);
}
