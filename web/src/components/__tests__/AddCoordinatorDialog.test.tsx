// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 74 Phase A — AddCoordinatorDialog rename coverage.
 *
 * This dialog carried the densest set of former "coordinateur" strings
 * (title, URL label, reachability hint, error copy). The shell rename test
 * renders it with an EMPTY picker so the dialog itself is never mounted there;
 * this focused test mounts it open and asserts the new "noeud" vocabulary.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";

import { AddCoordinatorDialog } from "../AddCoordinatorDialog";
import { useProjectStore } from "@/stores/projectStore";

beforeEach(() => {
  vi.restoreAllMocks();
  useProjectStore.setState({ knownCoordinators: [], activeCoordinatorUrl: null });
});

afterEach(() => {
  useProjectStore.setState({ knownCoordinators: [], activeCoordinatorUrl: null });
});

describe("AddCoordinatorDialog rename", () => {
  it("uses the 'noeud' vocabulary, not 'coordinateur'", () => {
    render(<AddCoordinatorDialog open onOpenChange={() => {}} />);
    expect(screen.getByText("Se connecter a un noeud")).toBeInTheDocument();
    expect(screen.getByText("URL du noeud")).toBeInTheDocument();
    expect(screen.queryByText(/coordinateur/i)).not.toBeInTheDocument();
  });
});
