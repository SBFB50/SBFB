// SPDX-License-Identifier: AGPL-3.0-or-later
import { useEffect, useReducer } from "react";

interface ApiState<T> {
  data: T | null;
  error: string | null;
  loading: boolean;
}

type ApiAction<T> =
  | { type: "fetch" }
  | { type: "success"; data: T }
  | { type: "error"; error: string };

function reducer<T>(state: ApiState<T>, action: ApiAction<T>): ApiState<T> {
  switch (action.type) {
    case "fetch":
      return { ...state, loading: true, error: null };
    case "success":
      return { data: action.data, error: null, loading: false };
    case "error":
      return { ...state, error: action.error, loading: false };
  }
}

export function useApi<T>(path: string) {
  const [state, dispatch] = useReducer(reducer<T>, {
    data: null,
    error: null,
    loading: true,
  });

  useEffect(() => {
    let cancelled = false;
    dispatch({ type: "fetch" });

    fetch(`/api${path}`, { headers: { "Content-Type": "application/json" } })
      .then((res) => {
        if (!res.ok) throw new Error(`${res.status}`);
        return res.json();
      })
      .then((json: T) => {
        if (!cancelled) dispatch({ type: "success", data: json });
      })
      .catch((err: Error) => {
        if (!cancelled) dispatch({ type: "error", error: err.message });
      });

    return () => {
      cancelled = true;
    };
  }, [path]);

  return state;
}

export async function postApi<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`/api${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new Error(`API ${res.status}: ${text}`);
  }
  return res.json();
}
