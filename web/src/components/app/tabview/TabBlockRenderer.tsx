/**
 * Switch over TabBlock.kind — one case per block type. The
 * `never` branch guarantees at compile-time that every new kind
 * added to the Zod schema has a matching render arm.
 */

import type { TabBlock } from "./schema";
import { SectionBlock } from "./blocks/SectionBlock";
import { HeadingBlock } from "./blocks/HeadingBlock";
import { TextBlock } from "./blocks/TextBlock";
import { KVBlock } from "./blocks/KVBlock";
import { MetricBlock } from "./blocks/MetricBlock";
import { TableBlock } from "./blocks/TableBlock";
import { BadgeListBlock } from "./blocks/BadgeListBlock";
import { ButtonBlock } from "./blocks/ButtonBlock";
import { ChartLineBlock } from "./blocks/ChartLineBlock";
import { ChartBarBlock } from "./blocks/ChartBarBlock";
import { EmptyBlock } from "./blocks/EmptyBlock";
import { FileUploadBlock } from "./blocks/FileUploadBlock";

export function TabBlockRenderer({ block }: { block: TabBlock }) {
  switch (block.kind) {
    case "section":
      return <SectionBlock block={block} />;
    case "heading":
      return <HeadingBlock block={block} />;
    case "text":
      return <TextBlock block={block} />;
    case "kv":
      return <KVBlock block={block} />;
    case "metric":
      return <MetricBlock block={block} />;
    case "table":
      return <TableBlock block={block} />;
    case "badge_list":
      return <BadgeListBlock block={block} />;
    case "button":
      return <ButtonBlock block={block} />;
    case "chart_line":
      return <ChartLineBlock block={block} />;
    case "chart_bar":
      return <ChartBarBlock block={block} />;
    case "empty":
      return <EmptyBlock block={block} />;
    case "file_upload":
      return <FileUploadBlock block={block} />;
    default: {
      const _exhaustive: never = block;
      return _exhaustive;
    }
  }
}
