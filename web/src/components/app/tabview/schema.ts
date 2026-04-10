/**
 * Sprint 6 Phase B — TabView Zod schema.
 *
 * Mirrors the Pydantic source of truth in
 * `packages/nexus-sdk/src/nexus_sdk/view.py`. The snapshot
 * `packages/nexus-sdk/tests/snapshots/tabview_schema.json`
 * is the cross-language checkpoint: any Python-side schema
 * change must be mirrored here in the same commit or the
 * snapshot test fails.
 *
 * Versioning: `schema_version: 1` is a literal — bumping it is
 * a breaking change that requires a paired Python update and
 * a snapshot regeneration.
 */

import { z } from "zod";

// ---------------------------------------------------------------------------
// Shared primitives
// ---------------------------------------------------------------------------

export const BlockToneSchema = z.enum(["neutral", "ok", "warn", "danger"]);
export type BlockTone = z.infer<typeof BlockToneSchema>;

export const TableAlignSchema = z.enum(["left", "right", "center"]);
export type TableAlign = z.infer<typeof TableAlignSchema>;

export const HeadingLevelSchema = z.union([
  z.literal(1),
  z.literal(2),
  z.literal(3),
]);
export type HeadingLevel = z.infer<typeof HeadingLevelSchema>;

// ---------------------------------------------------------------------------
// Leaf item shapes (used inside blocks)
// ---------------------------------------------------------------------------

export const KVItemSchema = z
  .object({
    label: z.string(),
    value: z.union([z.string(), z.number()]),
    hint: z.string().nullable().optional(),
  })
  .strict();

export const TableColumnSchema = z
  .object({
    key: z.string(),
    label: z.string(),
    align: TableAlignSchema.default("left"),
  })
  .strict();

export const TableRowSchema = z.record(
  z.string(),
  z.union([z.string(), z.number(), z.null()]),
);

export const BadgeItemSchema = z
  .object({
    label: z.string(),
    tone: BlockToneSchema.default("neutral"),
  })
  .strict();

export const ActionRouteSchema = z
  .object({
    kind: z.literal("route"),
    path: z.string(),
  })
  .strict();

export const ActionTaskSubmitSchema = z
  .object({
    kind: z.literal("task_submit"),
    worker: z.string(),
    payload: z.unknown().nullable().optional(),
  })
  .strict();

export const ButtonActionSchema = z.discriminatedUnion("kind", [
  ActionRouteSchema,
  ActionTaskSubmitSchema,
]);
export type ButtonAction = z.infer<typeof ButtonActionSchema>;

export const ChartLinePointSchema = z
  .object({
    x: z.string(),
    y: z.number(),
  })
  .strict();

export const ChartBarSchema = z
  .object({
    label: z.string(),
    value: z.number(),
    tone: BlockToneSchema.default("neutral"),
  })
  .strict();

// ---------------------------------------------------------------------------
// Block schemas — one per kind, discriminated union at the end
// ---------------------------------------------------------------------------

export const TabBlockHeadingSchema = z
  .object({
    kind: z.literal("heading"),
    level: HeadingLevelSchema,
    text: z.string(),
  })
  .strict();

export const TabBlockTextSchema = z
  .object({
    kind: z.literal("text"),
    text: z.string(),
    muted: z.boolean().default(false),
  })
  .strict();

export const TabBlockKVSchema = z
  .object({
    kind: z.literal("kv"),
    items: z.array(KVItemSchema),
  })
  .strict();

export const TabBlockMetricSchema = z
  .object({
    kind: z.literal("metric"),
    label: z.string(),
    value: z.union([z.string(), z.number()]),
    delta: z.number().nullable().optional(),
    unit: z.string().nullable().optional(),
    tone: BlockToneSchema.default("neutral"),
  })
  .strict();

export const TabBlockTableSchema = z
  .object({
    kind: z.literal("table"),
    columns: z.array(TableColumnSchema),
    rows: z.array(TableRowSchema),
    empty_text: z.string().nullable().optional(),
  })
  .strict();

export const TabBlockBadgeListSchema = z
  .object({
    kind: z.literal("badge_list"),
    items: z.array(BadgeItemSchema),
  })
  .strict();

export const TabBlockButtonSchema = z
  .object({
    kind: z.literal("button"),
    label: z.string(),
    action: ButtonActionSchema,
    tone: BlockToneSchema.default("neutral"),
  })
  .strict();

export const TabBlockChartLineSchema = z
  .object({
    kind: z.literal("chart_line"),
    label: z.string(),
    points: z.array(ChartLinePointSchema),
    y_unit: z.string().nullable().optional(),
  })
  .strict();

export const TabBlockChartBarSchema = z
  .object({
    kind: z.literal("chart_bar"),
    label: z.string(),
    bars: z.array(ChartBarSchema),
  })
  .strict();

export const TabBlockEmptySchema = z
  .object({
    kind: z.literal("empty"),
    text: z.string(),
  })
  .strict();

// ---------------------------------------------------------------------------
// Exported TS types (hand-written because of the recursive Section).
// The Zod schema for recursive types works via z.lazy but TS
// struggles to narrow a discriminated-union lazy schema, so we
// type the runtime union as ZodType<TabBlock> explicitly.
// ---------------------------------------------------------------------------

export type KVItem = z.infer<typeof KVItemSchema>;
export type TableColumn = z.infer<typeof TableColumnSchema>;
export type TableRow = z.infer<typeof TableRowSchema>;
export type BadgeItem = z.infer<typeof BadgeItemSchema>;
export type ChartLinePoint = z.infer<typeof ChartLinePointSchema>;
export type ChartBar = z.infer<typeof ChartBarSchema>;

export type TabBlockHeading = z.infer<typeof TabBlockHeadingSchema>;
export type TabBlockText = z.infer<typeof TabBlockTextSchema>;
export type TabBlockKV = z.infer<typeof TabBlockKVSchema>;
export type TabBlockMetric = z.infer<typeof TabBlockMetricSchema>;
export type TabBlockTable = z.infer<typeof TabBlockTableSchema>;
export type TabBlockBadgeList = z.infer<typeof TabBlockBadgeListSchema>;
export type TabBlockButton = z.infer<typeof TabBlockButtonSchema>;
export type TabBlockChartLine = z.infer<typeof TabBlockChartLineSchema>;
export type TabBlockChartBar = z.infer<typeof TabBlockChartBarSchema>;
export type TabBlockEmpty = z.infer<typeof TabBlockEmptySchema>;

export type TabBlockSection = {
  kind: "section";
  title?: string | null;
  blocks: TabBlock[];
};

export type TabBlock =
  | TabBlockSection
  | TabBlockHeading
  | TabBlockText
  | TabBlockKV
  | TabBlockMetric
  | TabBlockTable
  | TabBlockBadgeList
  | TabBlockButton
  | TabBlockChartLine
  | TabBlockChartBar
  | TabBlockEmpty;

// Section is recursive — declared via z.lazy referencing TabBlockSchema.
// We use the 3-param ZodType<Output, Def, Input=unknown> form so the
// input side of the recursion stays permissive; the runtime validator
// still enforces the full shape via the inner `.strict()` object.
export const TabBlockSectionSchema: z.ZodType<
  TabBlockSection,
  z.ZodTypeDef,
  unknown
> = z.lazy(() =>
  z
    .object({
      kind: z.literal("section"),
      title: z.string().nullable().optional(),
      blocks: z.array(TabBlockSchema),
    })
    .strict() as unknown as z.ZodType<TabBlockSection, z.ZodTypeDef, unknown>,
);

export const TabBlockSchema: z.ZodType<TabBlock, z.ZodTypeDef, unknown> =
  z.lazy(() =>
    z.union([
      TabBlockSectionSchema,
      TabBlockHeadingSchema,
      TabBlockTextSchema,
      TabBlockKVSchema,
      TabBlockMetricSchema,
      TabBlockTableSchema,
      TabBlockBadgeListSchema,
      TabBlockButtonSchema,
      TabBlockChartLineSchema,
      TabBlockChartBarSchema,
      TabBlockEmptySchema,
    ]),
  );

// ---------------------------------------------------------------------------
// Top-level TabView
// ---------------------------------------------------------------------------

export const TabViewSchema = z
  .object({
    schema_version: z.literal(1),
    tab_name: z.string(),
    title: z.string().nullable().optional(),
    blocks: z.array(TabBlockSchema).default([]),
  })
  .strict();

export type TabView = z.infer<typeof TabViewSchema>;

/**
 * Parse an unknown value into a TabView, returning a discriminated
 * result so callers can branch on success without catching.
 */
export function parseTabView(
  raw: unknown,
):
  | { ok: true; value: TabView }
  | { ok: false; error: string } {
  const parsed = TabViewSchema.safeParse(raw);
  if (parsed.success) {
    return { ok: true, value: parsed.data };
  }
  const first = parsed.error.issues[0];
  const where = first ? first.path.join(".") || "(root)" : "(root)";
  const msg = first ? first.message : "unknown parse error";
  return { ok: false, error: `${where}: ${msg}` };
}
