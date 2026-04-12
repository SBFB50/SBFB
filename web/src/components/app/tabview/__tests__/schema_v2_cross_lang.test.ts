// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 9 Phase E — cross-language v2 fixture roundtrip (Vitest side).
 *
 * Mirrors `packages/nexus-sdk/tests/test_view_v2.py::
 * test_cross_lang_fixture_v2_roundtrip_python_side`. Both sides parse
 * the same `tabview_v2_canonical.json` fixture and assert a clean
 * roundtrip, catching any Python-vs-TypeScript schema drift.
 */

import { describe, it, expect } from "vitest";
import { TabViewSchema, parseTabView } from "../schema";
import fixture from "../../../../../../packages/nexus-sdk/tests/snapshots/tabview_v2_canonical.json";

describe("v2 cross-language fixture", () => {
  it("tabview_v2_canonical.json parses under TabViewSchema", () => {
    const result = TabViewSchema.safeParse(fixture);
    expect(result.success).toBe(true);
  });

  it("parseTabView returns ok for the v2 canonical fixture", () => {
    const result = parseTabView(fixture);
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value.schema_version).toBe(2);
      expect(result.value.tab_name).toBe("test_v2");
      expect(result.value.blocks).toHaveLength(3);
      expect(result.value.blocks[0].kind).toBe("heading");
      expect(result.value.blocks[1].kind).toBe("file_upload");
      expect(result.value.blocks[2].kind).toBe("empty");
    }
  });
});
