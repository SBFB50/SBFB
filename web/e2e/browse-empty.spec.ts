// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Browse against an empty daemon (hermetic).
 *
 * The honest replacement for the vacuous "browse renders" spec: the
 * hermetic daemon has zero browse entries, so the real
 * `/api/daemon/browse` + `/api/daemon/search` round-trips drive the
 * empty states. This exercises the live search path end to end
 * (input → daemon FTS5 query → "Aucun résultat") rather than a mock.
 */

import { test, expect } from "./fixtures";

test.describe("Browse — empty daemon", () => {
  test("search against an empty index renders the no-result state", async ({
    page,
  }) => {
    await page.goto("/browse");

    const search = page.getByTestId("browse-search-input");
    await expect(search).toBeVisible();

    await search.fill("zzz-aucune-app-correspondante-zzz");

    // The daemon's FTS5 index is empty → browse-search-empty (Browse.tsx:441).
    await expect(page.getByTestId("browse-search-empty")).toBeVisible();
    await expect(page.getByText("Aucun résultat")).toBeVisible();

    // The clear button appears once the query is non-empty and resets it.
    const clear = page.getByTestId("browse-search-clear");
    await expect(clear).toBeVisible();
    await clear.click();
    await expect(search).toHaveValue("");
  });
});
