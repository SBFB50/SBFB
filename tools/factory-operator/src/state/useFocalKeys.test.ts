// SPDX-License-Identifier: AGPL-3.0-or-later
import { renderHook } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { useFocalKeys } from './useFocalKeys'

function press(key: string, opts: KeyboardEventInit = {}) {
  document.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, ...opts }))
}

afterEach(() => {
  document.body.innerHTML = ''
})

describe('useFocalKeys (raccourci focal D6 manuel)', () => {
  it('bascule STEER/VERIFY sur s/v quand on ne tape pas', () => {
    const setMode = vi.fn()
    renderHook(() => useFocalKeys(setMode))
    press('v')
    expect(setMode).toHaveBeenLastCalledWith('verify')
    press('s')
    expect(setMode).toHaveBeenLastCalledWith('steer')
    press('S')
    expect(setMode).toHaveBeenLastCalledWith('steer')
  })

  it('ignore les combos avec modificateur (n entre pas en conflit avec les raccourcis)', () => {
    const setMode = vi.fn()
    renderHook(() => useFocalKeys(setMode))
    press('v', { ctrlKey: true })
    press('s', { metaKey: true })
    press('v', { altKey: true })
    press('v', { repeat: true })
    expect(setMode).not.toHaveBeenCalled()
  })

  it('ignore les frappes quand le focus est dans un champ de saisie', () => {
    const setMode = vi.fn()
    const input = document.createElement('input')
    document.body.appendChild(input)
    input.focus()
    renderHook(() => useFocalKeys(setMode))
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'v', bubbles: true }))
    expect(setMode).not.toHaveBeenCalled()
  })

  it('ignore les autres touches', () => {
    const setMode = vi.fn()
    renderHook(() => useFocalKeys(setMode))
    press('a')
    press('Enter')
    expect(setMode).not.toHaveBeenCalled()
  })
})
