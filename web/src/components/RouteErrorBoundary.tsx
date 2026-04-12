// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 6 audit fix G-4 — ErrorBoundary for the route outlet.
 *
 * Without a boundary, a render crash in any route element (e.g., a
 * gov tab throwing during data fetch) propagates up through AppShell
 * and unmounts the entire React tree — taking the sidebar, header,
 * AND the command palette down with it. The D5 promise that Ctrl+K
 * keeps working on a broken page was relying on the palette being
 * a sibling of Outlet, but "sibling" does not protect you from React
 * unwinding an unhandled throw up to its root.
 *
 * Wrapping only `<Outlet />` in this boundary confines the damage:
 * a crashed route renders a fallback message, the shell chrome and
 * palette stay alive. Ctrl+K keeps navigating. Users can recover
 * without a full reload.
 *
 * Kept as a simple class component — this is one of the few places
 * in React where class components are the only way (function
 * components can't implement the ErrorBoundary lifecycle methods).
 */

import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

export class RouteErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { error: null, errorInfo: null };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { error };
  }

  override componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    this.setState({ error, errorInfo });
    // Log to console so the dev inspects the stack. Production
    // telemetry (Sentry, etc.) is out of scope until the project
    // has an ops story.
    console.error("[RouteErrorBoundary] route crashed:", error, errorInfo);
  }

  private readonly reset = (): void => {
    this.setState({ error: null, errorInfo: null });
  };

  override render(): ReactNode {
    if (this.state.error) {
      return (
        <div className="flex min-h-[60vh] flex-col items-center justify-center gap-4 p-8 text-center">
          <div className="space-y-2">
            <h1 className="text-xl font-semibold text-destructive">
              La page a crashé
            </h1>
            <p className="max-w-md text-sm text-muted-foreground">
              Le shell est toujours utilisable — la palette de commandes
              (Ctrl/Cmd + K) et la navigation latérale fonctionnent. Vous
              pouvez essayer de relancer le rendu ou de changer de page.
            </p>
          </div>
          <div className="flex gap-2">
            <button
              type="button"
              className="rounded border border-border bg-background px-3 py-1.5 text-xs font-medium hover:bg-muted"
              onClick={this.reset}
            >
              Réessayer
            </button>
            <button
              type="button"
              className="rounded border border-border bg-background px-3 py-1.5 text-xs font-medium hover:bg-muted"
              onClick={() => window.location.reload()}
            >
              Recharger la page
            </button>
          </div>
          <details className="mt-2 w-full max-w-xl rounded border border-border bg-muted/20 text-left">
            <summary className="cursor-pointer px-3 py-2 text-[10px] uppercase tracking-wider text-muted-foreground">
              Détails techniques
            </summary>
            <pre className="max-h-64 overflow-auto px-3 py-2 text-[11px] leading-snug">
              {String(this.state.error?.stack ?? this.state.error)}
            </pre>
          </details>
        </div>
      );
    }
    return this.props.children;
  }
}
