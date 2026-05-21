// SPDX-License-Identifier: AGPL-3.0-or-later
import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { ProofCard, type ProofCardData } from "../ProofCard";

function makeCard(overrides: Partial<ProofCardData> = {}): ProofCardData {
  return {
    project_id: "aa".repeat(32),
    project_name: "test-project",
    hash: { archive_hash: "deadbeef", provenance_hash: "cafebabe" },
    license: { spdx: "AGPL-3.0-or-later", source: "manifest" },
    freshness: {
      last_verified_at: "2026-05-20T12:00:00Z",
      age_days: 1,
      state: "fresh",
    },
    provenance: {
      verified: true,
      repo_url: "https://github.com/org/app",
      commit_sha: "abc123",
      slsa_level: 1,
    },
    risk: { level: "low", factors: [] },
    curation: {
      curator_count: 3,
      curator_names: ["Alice", "Bob", "Charlie"],
    },
    confidence: 100,
    formula_version: 1,
    ...overrides,
  };
}

describe("ProofCard", () => {
  it("renders the confidence score", () => {
    render(<ProofCard card={makeCard()} />);
    expect(screen.getByTestId("proof-card-score")).toHaveTextContent("100/100");
  });

  it("renders evidence layers when expanded", async () => {
    const user = userEvent.setup();
    render(<ProofCard card={makeCard()} />);

    await user.click(screen.getByTestId("proof-card-toggle"));

    expect(screen.getByTestId("proof-card-layers")).toBeInTheDocument();
    expect(screen.getByTestId("proof-card-layer-provenance")).toBeInTheDocument();
    expect(screen.getByTestId("proof-card-layer-licence")).toBeInTheDocument();
    expect(screen.getByTestId("proof-card-layer-fraicheur")).toBeInTheDocument();
    expect(screen.getByTestId("proof-card-layer-curation")).toBeInTheDocument();
    expect(screen.getByTestId("proof-card-layer-archive")).toBeInTheDocument();
  });

  it("expands on click and collapses on second click", async () => {
    const user = userEvent.setup();
    render(<ProofCard card={makeCard()} />);

    expect(screen.queryByTestId("proof-card-details")).not.toBeInTheDocument();

    await user.click(screen.getByTestId("proof-card-toggle"));
    expect(screen.getByTestId("proof-card-details")).toBeInTheDocument();

    await user.click(screen.getByTestId("proof-card-toggle"));
    expect(screen.queryByTestId("proof-card-details")).not.toBeInTheDocument();
  });

  it("shows risk factors when present", async () => {
    const user = userEvent.setup();
    const card = makeCard({
      confidence: 15,
      risk: {
        level: "high",
        factors: ["no_provenance", "stale_source"],
      },
      provenance: {
        verified: false,
        repo_url: "https://github.com/org/app",
        commit_sha: null,
        slsa_level: 0,
      },
    });

    render(<ProofCard card={card} />);
    await user.click(screen.getByTestId("proof-card-toggle"));

    expect(screen.getByTestId("proof-card-risk-factors")).toBeInTheDocument();
    const factors = screen.getAllByTestId("proof-card-risk-factor");
    expect(factors).toHaveLength(2);
    expect(factors[0]).toHaveTextContent("Pas de provenance");
    expect(factors[1]).toHaveTextContent("Source obsolete");
  });

  it("does not show risk factors section when none present", async () => {
    const user = userEvent.setup();
    render(<ProofCard card={makeCard()} />);
    await user.click(screen.getByTestId("proof-card-toggle"));

    expect(screen.queryByTestId("proof-card-risk-factors")).not.toBeInTheDocument();
  });

  it("renders loading state", () => {
    render(<ProofCard card={null} loading />);
    expect(screen.getByTestId("proof-card-loading")).toBeInTheDocument();
  });

  it("renders nothing when card is null and not loading", () => {
    const { container } = render(<ProofCard card={null} />);
    expect(container.firstChild).toBeNull();
  });

  it("shows risk level badge in expanded view", async () => {
    const user = userEvent.setup();
    const card = makeCard({
      risk: { level: "medium", factors: ["stale_source"] },
    });
    render(<ProofCard card={card} />);
    await user.click(screen.getByTestId("proof-card-toggle"));

    expect(screen.getByTestId("proof-card-risk-level")).toHaveTextContent("Risque Moyen");
  });
});
