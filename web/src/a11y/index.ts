// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 83 Phase B — shared host→app accessibility module (public surface).
 *
 * See {@link ./profile} for the cardinal `needs[]`-never-crosses invariant.
 * This barrel re-exports the public surface; external consumers (the shell and
 * the Hub host) should prefer it. Sibling modules inside `a11y`/`bridge` may
 * import `./profile` directly to avoid an import cycle.
 */
export * from "./profile";
