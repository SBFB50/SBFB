// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D — the secondary surfaces opened from the rail. These are
// auxiliary INSPECTORS (procédé · sessions · knowledge), distinct from the
// two focal MODES (STEER / VERIFY): they replace the focal scene body while
// open and return to it on "← retour". The terminal-PTY is NOT here — it is
// elevated INTO the VERIFY focal scene (kickoff: "terminal xterm élevé en
// surface VERIFY de démarrage"), reachable via the VERIFY mode toggle.
//
// A named constant (no magic strings): the same list backs the rail buttons
// and the SurfaceHost router (mirror of the union below).

export type SecondarySurface = 'procede' | 'sessions' | 'knowledge'

export interface SurfaceDef {
  id: SecondarySurface
  glyph: string
  label: string
  hint: string
}

export const SECONDARY_SURFACES: readonly SurfaceDef[] = [
  {
    id: 'procede',
    glyph: '⊢',
    label: 'Procédé',
    hint: 'arbre sprint · phase · commit · artefact',
  },
  {
    id: 'sessions',
    glyph: '≣',
    label: 'Sessions',
    hint: 'journal · refus du mur · rejeu',
  },
  {
    id: 'knowledge',
    glyph: '◇',
    label: 'Knowledge',
    hint: 'packs consultatifs · context-pack scellé',
  },
] as const
