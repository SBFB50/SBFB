// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Secondary surfaces opened from the rail. They replace the focal scene body
// while open and return to it on back. Keep this list as the single source for
// the rail buttons and the SurfaceHost router.

export type SecondarySurface = 'procede' | 'sessions' | 'knowledge' | 'documents'

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
  {
    id: 'documents',
    glyph: '▦',
    label: 'Docs',
    hint: 'cartographie fichiers · rôles LLM · suivi live',
  },
] as const
