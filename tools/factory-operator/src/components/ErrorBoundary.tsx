// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 (front rapid-add) — a global render-error boundary. Without it a
// throw in ANY surface unmounts the whole tree to a white screen; with it the
// operator sees a recoverable fallback panel instead. Pure React class
// component (the only way to implement `getDerivedStateFromError` /
// `componentDidCatch`), 0 runtime dep, CSP-safe (no inline script, design
// tokens only). It restitutes the error message — it never swallows it: the
// raw error still goes to the console for the operator to inspect.
import { Component, type ErrorInfo, type ReactNode } from 'react'

interface Props {
  children: ReactNode
  /** Optional short label of the boundary scope (e.g. "VERIFY"), shown in the
   * fallback so a scoped boundary names what failed. */
  scope?: string
}

interface State {
  error: Error | null
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null }

  static getDerivedStateFromError(error: Error): State {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    // Keep the raw error visible to the operator — never silently swallow it.
    console.error('[factory-operator] render error', error, info.componentStack)
  }

  private reset = () => this.setState({ error: null })

  render(): ReactNode {
    const { error } = this.state
    if (error === null) return this.props.children

    const scope = this.props.scope
    return (
      <div
        role="alert"
        data-testid="error-boundary-fallback"
        className="flex h-full min-h-0 flex-1 flex-col items-center justify-center gap-4 bg-s0 p-8 text-center"
      >
        <div className="max-w-lg rounded-md border border-bd bg-s1 p-6">
          <p className="font-sans text-sec font-semibold text-bad">
            {scope ? `${scope} — erreur de rendu` : 'erreur de rendu'}
          </p>
          <p className="mt-3 font-sans text-body text-tx2">
            Une erreur a interrompu l'affichage. Le nœud n'est pas affecté ; tu peux réessayer.
          </p>
          <pre className="mt-3 max-h-40 overflow-auto rounded-sm border border-bd bg-s0 p-2 text-left font-mono text-meta text-tx3">
            {error.message}
          </pre>
          <div className="mt-4 flex items-center justify-center gap-2">
            <button
              type="button"
              onClick={this.reset}
              className="rounded-sm border border-bd2 bg-s2 px-3 py-1.5 font-mono text-meta text-tx hover:bg-s3"
            >
              Réessayer le rendu
            </button>
            <button
              type="button"
              onClick={() => window.location.reload()}
              className="rounded-sm border border-bd bg-s1 px-3 py-1.5 font-mono text-meta text-tx2 hover:bg-s2"
            >
              Recharger
            </button>
          </div>
        </div>
      </div>
    )
  }
}
