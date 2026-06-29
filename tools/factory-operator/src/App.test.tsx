// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Shell-level test: the offline banner's anti-flash guard
// (!reachable && !loading), and the useFocalKeys wiring. The hooks + focal
// scenes are stubbed so the assertions isolate App's own logic.
import type { ReactNode } from 'react'
import { render, screen } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { I18nProvider } from '@lingui/react'
import { App } from './App'
import { i18n } from './i18n/i18n'
import * as railMod from './state/useRailStatus'
import * as opMod from './state/useOperator'
import * as focalMod from './state/useFocalKeys'
import type { RailHandle } from './state/useRailStatus'
import type { Operator } from './state/useOperator'

vi.mock('./state/useRailStatus', () => ({ useRailStatus: vi.fn() }))
vi.mock('./state/useOperator', () => ({ useOperator: vi.fn() }))
vi.mock('./state/useFocalKeys', () => ({ useFocalKeys: vi.fn() }))
vi.mock('./components/steer/SteerScene', () => ({ SteerScene: () => <div data-testid="steer-scene" /> }))
vi.mock('./components/verify/VerifyScene', () => ({ VerifyScene: () => <div data-testid="verify-scene" /> }))
vi.mock('./components/surfaces/SurfaceHost', () => ({ SurfaceHost: () => <div data-testid="surface-host" /> }))

const useRailStatus = vi.mocked(railMod.useRailStatus)
const useOperator = vi.mocked(opMod.useOperator)
const useFocalKeys = vi.mocked(focalMod.useFocalKeys)

// App renders <Trans> (skip-link, offline banner); the runtime needs an
// I18nProvider ancestor (the real provider lives in main.tsx). setup.ts has
// already activated the source locale on this i18n.
function I18nWrap({ children }: { children: ReactNode }) {
  return <I18nProvider i18n={i18n}>{children}</I18nProvider>
}

const setMode = vi.fn()
const baseOp = {
  mode: 'steer',
  setMode,
  surface: null,
  openSurface: vi.fn(),
  closeSurface: vi.fn(),
  preparePack: vi.fn(),
  provider: 'claude',
  setProvider: vi.fn(),
  sessionId: null,
  hasTurn: false,
  verifyReady: false,
  turn: {
    message: null,
    kind: null,
    status: 'idle',
    text: '',
    thinking: '',
    result: null,
    error: null,
    gate: null,
    busy: false,
    launchError: null,
  },
  launch: vi.fn(),
  relaunch: vi.fn(),
  interrupt: vi.fn(),
  dismissGate: vi.fn(),
  newSession: vi.fn(),
} as unknown as Operator

function rail(over: Partial<RailHandle>): RailHandle {
  return {
    sprint: 80,
    phase: 'I',
    branch: 'master',
    dirty: 0,
    staged: 0,
    gateCounts: null,
    reachable: true,
    loading: false,
    refresh: vi.fn(),
    ...over,
  }
}

beforeEach(() => {
  vi.clearAllMocks()
  useOperator.mockReturnValue(baseOp)
})

describe('App shell — bannière offline + câblage', () => {
  it('ne montre PAS la bannière quand le backend est joignable', () => {
    useRailStatus.mockReturnValue(rail({ reachable: true, loading: false }))
    render(<App />, { wrapper: I18nWrap })
    expect(screen.queryByTestId('offline-banner')).toBeNull()
  })

  it('ne montre PAS la bannière pendant la 1re liaison (anti-flash boot)', () => {
    useRailStatus.mockReturnValue(rail({ reachable: false, loading: true }))
    render(<App />, { wrapper: I18nWrap })
    expect(screen.queryByTestId('offline-banner')).toBeNull()
  })

  it('montre la bannière quand le backend est injoignable et la liaison terminée', () => {
    useRailStatus.mockReturnValue(rail({ reachable: false, loading: false }))
    render(<App />, { wrapper: I18nWrap })
    expect(screen.getByTestId('offline-banner')).toBeInTheDocument()
  })

  it('câble useFocalKeys sur op.setMode et rend la scène STEER par défaut', () => {
    useRailStatus.mockReturnValue(rail({}))
    render(<App />, { wrapper: I18nWrap })
    expect(useFocalKeys).toHaveBeenCalledWith(setMode)
    expect(screen.getByTestId('steer-scene')).toBeInTheDocument()
  })
})
