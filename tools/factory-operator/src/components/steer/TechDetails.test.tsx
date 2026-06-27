// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import * as api from '../../api/operator'
import { TechDetails } from './TechDetails'

vi.mock('../../api/operator', () => ({
  getPrompt: vi.fn(),
  getProviders: vi.fn(),
  OperatorError: class extends Error {
    status: number
    constructor(status: number) {
      super('op')
      this.status = status
    }
  },
}))

const getPrompt = vi.mocked(api.getPrompt)
const getProviders = vi.mocked(api.getProviders)

afterEach(() => vi.restoreAllMocks())

describe('TechDetails', () => {
  it('renders nothing while collapsed (and fetches nothing)', () => {
    render(<TechDetails open={false} kind="preflight" provider="claude" />)
    expect(screen.queryByTestId('tech-details')).toBeNull()
    expect(getPrompt).not.toHaveBeenCalled()
  })

  it('shows the real assembled prompt and the provider diagnostic once opened', async () => {
    getPrompt.mockResolvedValue({ kind: 'preflight', provider: 'claude', content: 'ASSEMBLED PROMPT BODY' })
    getProviders.mockResolvedValue({ providers: ['claude', 'codex', 'local'] })
    render(<TechDetails open kind="preflight" provider="claude" />)
    await waitFor(() => expect(screen.getByText('ASSEMBLED PROMPT BODY')).toBeInTheDocument())
    expect(screen.getByText(/backend joignable \(3\)/)).toBeInTheDocument()
  })

  it('surfaces an honest error when the prompt is unavailable', async () => {
    getPrompt.mockRejectedValue(new api.OperatorError(404, '/api/prompt/x'))
    getProviders.mockResolvedValue({ providers: [] })
    render(<TechDetails open kind="missing" provider="claude" />)
    await waitFor(() => expect(screen.getByText(/prompt indisponible \(404\)/)).toBeInTheDocument())
  })
})
