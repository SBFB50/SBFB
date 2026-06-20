// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Deploy — publish-form client logic (hermetic, no real git clone).
 * Deepens pages-smoke.spec.ts (initial disabled) with the exact
 * submit-enable boolean, the absence of a client URL-format gate, the
 * static truth line, the gated success-only nodes, and the query-param
 * prefill of the "remettre en ligne" flow. Submit is never clicked
 * (would POST a real clone).
 */

import { test, expect } from "./fixtures";

test.describe("Deploy — form logic", () => {
  test("enables submit only when BOTH repo-url and project-name are filled", async ({
    page,
  }) => {
    await page.goto("/deploy");
    const submit = page.getByTestId("deploy-submit");
    await expect(submit).toBeDisabled();

    await page.getByTestId("repo-url").fill("https://github.com/user/repo.git");
    await expect(submit).toBeDisabled();

    await page.getByTestId("project-name").fill("   ");
    await expect(submit).toBeDisabled(); // .trim() rejects whitespace-only

    await page.getByTestId("project-name").fill("mon-app");
    await expect(submit).toBeEnabled();
  });

  test("a syntactically invalid repo URL still enables submit (no JS format gate)", async ({
    page,
  }) => {
    await page.goto("/deploy");
    await page.getByTestId("repo-url").fill("not-a-valid-url");
    await page.getByTestId("project-name").fill("mon-app");
    await expect(page.getByTestId("deploy-submit")).toBeEnabled();
  });

  test("renders the truth line and hides the success-only tech toggle/details", async ({
    page,
  }) => {
    await page.goto("/deploy");
    const truth = page.getByTestId("deploy-truth-line");
    await expect(truth).toBeVisible();
    await expect(truth).toContainText("Ton noeud signe cette app");

    await expect(page.getByTestId("deploy-tech-toggle")).toHaveCount(0);
    await expect(page.getByTestId("deploy-tech-details")).toHaveCount(0);
    await expect(page.getByTestId("deploy-success")).toHaveCount(0);
    await expect(page.getByTestId("deploy-error")).toHaveCount(0);
  });

  test("?repo_url & ?project_name prefill both fields and enable submit on load", async ({
    page,
  }) => {
    await page.goto(
      "/deploy?repo_url=https%3A%2F%2Fgithub.com%2Fu%2Fr.git&project_name=remise-en-ligne",
    );
    await expect(page.getByTestId("repo-url")).toHaveValue(
      "https://github.com/u/r.git",
    );
    await expect(page.getByTestId("project-name")).toHaveValue("remise-en-ligne");
    await expect(page.getByTestId("deploy-submit")).toBeEnabled();
  });
});
