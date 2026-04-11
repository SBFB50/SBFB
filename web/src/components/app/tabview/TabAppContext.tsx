/**
 * Sprint 8 Phase A — per-tab coordinator + app-name context.
 *
 * Tab descriptors render through `<TabViewRenderer>`, which
 * walks the block tree and instantiates per-kind components
 * (HeadingBlock, ButtonBlock, MetricBlock, ...). Any block that
 * needs to talk to the coordinator (currently: `ButtonBlock`
 * with a `task_submit` action) reads `coordinatorUrl` + `appName`
 * from this context instead of receiving them prop-drilled
 * through every intermediate block.
 *
 * The renderer has no global state; the provider is mounted
 * exactly once per tab instance (typically in `AppsTab.tsx`
 * around the TabViewRenderer for the chosen app).
 *
 * `null` default lets the context be consumed outside a provider
 * without throwing — consumers are expected to check the value
 * and render a disabled state (or log a warning) if it's
 * missing.
 */

import { createContext, useContext } from "react";

export interface TabAppContextValue {
  /** Base URL of the coordinator currently serving this tab. */
  coordinatorUrl: string;
  /** Name of the app whose tab is being rendered. */
  appName: string;
}

export const TabAppContext = createContext<TabAppContextValue | null>(null);

export function useTabAppContext(): TabAppContextValue | null {
  return useContext(TabAppContext);
}
