// SPDX-License-Identifier: AGPL-3.0-or-later
// Shared constants for the hermetic T1 harness (playwright.config.ts +
// specs + e2e/serve-operator.mjs). A fixed 64-hex bearer keeps the boot
// deterministic; SBFB_HOME is redirected to a temp dir so no real
// ~/.sbfb is ever touched.
export const OPERATOR_TEST_PORT = Number(process.env.OPERATOR_TEST_PORT || 3111)
export const OPERATOR_TEST_TOKEN = 'a'.repeat(64)
