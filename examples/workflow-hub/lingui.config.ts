// SPDX-License-Identifier: AGPL-3.0-or-later
/*
 * Sprint 83 Phase B — Workflow Hub i18n socle (Lingui, PO format).
 *
 * The Lingui OVERRIDE (PO decision, memory `factory_universal_ux_pivot`) is the
 * project i18n choice; this config seeds it for the Hub, mirroring the Factory
 * Operator's setup (`tools/factory-operator/lingui.config.ts`) WITHOUT importing
 * it. The Hub scaffold in this phase is framework-agnostic (the React authoring
 * surface is scaffolded later, kickoff Q8=A); the catalogs + gates are the
 * FOUNDATION, so a French source and an RTL locale (ar) exist and stay in
 * key-parity from the start.
 *
 * HONEST SCOPE (AM-8): in this amorce the Lingui RUNTIME is not yet wired — the
 * visible amorce content is static French in index.html/app.js, and the .po
 * catalogs are the Lingui-ready source of truth validated by the 3 discipline
 * gates (key parity / verdict cross-locale / no-literal). Catalog COMPILATION +
 * runtime activation land with the React authoring surface; this config + the
 * catalogs + the gates are the deliberate logistics-not-conformity deliverable.
 *
 * NOT executable in this amorce (Codex S83-B): there is deliberately no Hub
 * `package.json`/lockfile/`@lingui/cli` yet — the Lingui tooling (catalog
 * compile + a `lingui compile` CI step) is added when the React authoring
 * surface is scaffolded (Phase R). Until then this is a PREPARED plain-object
 * config (no `@lingui/conf` import, so it stays loadable from a repo without Hub
 * node_modules); the EXECUTABLE half of the socle is the 3 discipline gates,
 * which parse and validate the `.po` catalogs on every push.
 *
 * Source locale = fr. Empty msgstr is allowed for the source-fallback policy;
 * key parity (not translation completeness) is what the gate enforces.
 */

const config = {
  sourceLocale: "fr",
  // fr = source, en = reference translation, ar = RTL foundation locale.
  locales: ["fr", "en", "ar"],
  catalogs: [
    {
      path: "<rootDir>/i18n/locales/{locale}",
      include: ["<rootDir>/src", "<rootDir>/app.js"],
    },
  ],
  format: "po",
};

export default config;
