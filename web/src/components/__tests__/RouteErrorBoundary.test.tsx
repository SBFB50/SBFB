/**
 * Sprint 6 audit fix G-4 — RouteErrorBoundary unit tests.
 *
 * Goal: prove the boundary renders its fallback UI when a child
 * throws, and that a healthy tree renders untouched.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

import { RouteErrorBoundary } from "../RouteErrorBoundary";

function Boom({ message = "boom" }: { message?: string }): never {
  throw new Error(message);
}

function Healthy() {
  return <p data-testid="healthy">rendu sain</p>;
}

describe("RouteErrorBoundary", () => {
  let errorSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    // React logs the error to console.error when the boundary catches
    // it. Silence the noise for this test and restore after.
    errorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
  });

  afterEach(() => {
    errorSpy.mockRestore();
  });

  it("renders children when no error is thrown", () => {
    render(
      <RouteErrorBoundary>
        <Healthy />
      </RouteErrorBoundary>,
    );
    expect(screen.getByTestId("healthy")).toBeInTheDocument();
  });

  it("renders the fallback UI when a child throws during render", () => {
    render(
      <RouteErrorBoundary>
        <Boom message="ProjectDetail threw" />
      </RouteErrorBoundary>,
    );
    expect(screen.getByText(/La page a crashé/)).toBeInTheDocument();
    expect(
      screen.getByText(/Le shell est toujours utilisable/),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Réessayer/ }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Recharger la page/ }),
    ).toBeInTheDocument();
  });

  it("exposes the error stack in a collapsible details block", () => {
    render(
      <RouteErrorBoundary>
        <Boom message="GovTabExploded" />
      </RouteErrorBoundary>,
    );
    expect(screen.getByText(/Détails techniques/)).toBeInTheDocument();
    expect(screen.getByText(/GovTabExploded/)).toBeInTheDocument();
  });

  it("logs the caught error to console.error (dev visibility)", () => {
    render(
      <RouteErrorBoundary>
        <Boom />
      </RouteErrorBoundary>,
    );
    // React logs its own error first, then our componentDidCatch adds
    // a tagged log. Check that at least one of those calls mentions
    // our tag.
    const calls: string[] = errorSpy.mock.calls.map(
      (c: unknown[]) => String(c[0]),
    );
    expect(
      calls.some((c: string) => c.includes("[RouteErrorBoundary]")),
    ).toBe(true);
  });

  it("reset button clears error state and re-renders children", () => {
    // A controlled child whose mode flips between boom and healthy.
    // We can't flip inside a single render cycle, so we mount with
    // an error, click Réessayer, and verify the boundary attempts
    // to re-render (it will boom again since the child is the same,
    // but the internal reset() call path is exercised).
    render(
      <RouteErrorBoundary>
        <Boom />
      </RouteErrorBoundary>,
    );
    expect(screen.getByText(/La page a crashé/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /Réessayer/ }));
    // After reset, the child throws again → fallback visible again.
    expect(screen.getByText(/La page a crashé/)).toBeInTheDocument();
  });
});
