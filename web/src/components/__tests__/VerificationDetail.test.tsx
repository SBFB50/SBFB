// SPDX-License-Identifier: AGPL-3.0-or-later
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { VerificationDetail } from "../VerificationDetail";
import { primeAuthToken } from "@/api/auth";

const NOOP = () => {};

const MOCK_RECORD = {
  repo_url: "https://github.com/test/repo",
  commit_sha: "abc123def456abc123def456abc123def456abc1",
  artifact_hash: "cafebabe12345678cafebabe12345678",
  signature: "sig9876543210abcdef9876543210abcdef",
  node_id: "node_aaabbbcccddd111222333",
  timestamp: "2026-05-15T12:00:00Z",
  schema_version: 1,
};

beforeEach(() => {
  primeAuthToken("test-token");
});

afterEach(() => {
  vi.restoreAllMocks();
  primeAuthToken(null);
});

describe("VerificationDetail", () => {
  it("shows loading then provenance fields on open", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
      new Response(
        JSON.stringify({ record: MOCK_RECORD, verified: true }),
        { status: 200 },
      ),
    );

    render(
      <VerificationDetail
        open
        onOpenChange={NOOP}
        coordUrl="http://localhost:8000"
        projectId="proj1"
      />,
    );

    await waitFor(() => {
      expect(screen.getByTestId("verification-result")).toBeInTheDocument();
    });

    expect(screen.getByTestId("verification-result").textContent).toContain(
      "Signature valide",
    );
    expect(screen.getByTestId("prov-repo-url")).toHaveAttribute(
      "href",
      "https://github.com/test/repo",
    );
    expect(screen.getByTestId("prov-commit-sha")).toBeInTheDocument();
    expect(screen.getByTestId("prov-artifact-hash")).toBeInTheDocument();
    expect(screen.getByTestId("prov-signature")).toBeInTheDocument();
    expect(screen.getByTestId("prov-node-id")).toBeInTheDocument();
    expect(screen.getByTestId("prov-timestamp")).toBeInTheDocument();
  });

  it("shows empty state when 404", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
      new Response(JSON.stringify({ error: "not found" }), { status: 404 }),
    );

    render(
      <VerificationDetail
        open
        onOpenChange={NOOP}
        coordUrl="http://localhost:8000"
        projectId="unknown"
      />,
    );

    await waitFor(() => {
      expect(screen.getByTestId("verification-empty")).toBeInTheDocument();
    });
  });

  it("reverify button triggers a fresh fetch", async () => {
    const fetchSpy = vi.spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({ record: MOCK_RECORD, verified: true }),
          { status: 200 },
        ),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({ record: MOCK_RECORD, verified: false }),
          { status: 200 },
        ),
      );

    const user = userEvent.setup();
    render(
      <VerificationDetail
        open
        onOpenChange={NOOP}
        coordUrl="http://localhost:8000"
        projectId="proj1"
      />,
    );

    await waitFor(() => {
      expect(screen.getByTestId("verify-button")).toBeInTheDocument();
    });

    await user.click(screen.getByTestId("verify-button"));

    await waitFor(() => {
      expect(screen.getByTestId("verification-result").textContent).toContain(
        "Signature invalide",
      );
    });

    expect(fetchSpy).toHaveBeenCalledTimes(2);
  });
});
