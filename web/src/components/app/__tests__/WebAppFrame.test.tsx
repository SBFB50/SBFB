// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 11 Phase C — WebAppFrame unit tests.
 *
 * Two scenarios: placeholder when no blob URL, and iframe with
 * sandbox attrs when a blob URL is provided.
 */

import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { WebAppFrame } from "../WebAppFrame";

describe("WebAppFrame", () => {
  it("renders placeholder when no blobUrl is provided", () => {
    render(<WebAppFrame />);
    expect(
      screen.getByTestId("webapp-frame-placeholder"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Application web non disponible"),
    ).toBeInTheDocument();
  });

  it("renders a sandboxed iframe when blobUrl is provided", () => {
    render(<WebAppFrame blobUrl="https://example.com/app.html" />);
    const iframe = screen.getByTestId("webapp-frame-iframe");
    expect(iframe).toBeInTheDocument();
    expect(iframe).toHaveAttribute("src", "https://example.com/app.html");
    expect(iframe).toHaveAttribute("sandbox", "allow-scripts allow-same-origin");
    expect(iframe).toHaveAttribute("title", "Application web");
  });
});
