// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 (front rapid-add) — format a claude-session `updated_at` timestamp.
// The CLI writes it as a unix epoch in seconds OR milliseconds depending on
// version, so we detect ms by magnitude (>1e12). 0 / NaN → empty (no "1970").
// Lives in lib/ (not the component file) so the component file only exports
// components — react-refresh fast-refresh stays intact.
export function formatSessionDate(updated: number): string {
  if (!updated) return ''
  const ms = updated > 1e12 ? updated : updated * 1000
  const d = new Date(ms)
  return Number.isNaN(d.getTime()) ? '' : d.toLocaleString('fr-FR')
}
