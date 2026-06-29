// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import * as api from '../../api/operator'
import { DocumentsSurface } from './DocumentsSurface'

vi.mock('../../api/operator', () => ({
  getProjectDocuments: vi.fn(),
  OperatorError: class extends Error {
    status = 500
  },
}))

const getProjectDocuments = vi.mocked(api.getProjectDocuments)

const SNAPSHOT: api.ProjectDocuments = {
  branch: 'master',
  head: 'abc1234',
  generated_at: '2026-06-29T10:00:00Z',
  total: 3,
  truncated: false,
  session: {
    id: 's1',
    provider: 'claude',
    model: 'claude-opus-4-8[1m]',
    messages: 2,
    chat_history_authoritative: false,
  },
  pinned: [
    {
      path: 'prompts/agent/base.md',
      role: 'use',
      label: 'prompt de base',
      source: 'session LLM active',
      detail: 'ref hachee 11111111',
    },
    {
      path: 'tools/factory-operator/src/components/surfaces/DocumentsSurface.tsx',
      role: 'write',
      label: 'travail en cours',
      source: 'git status',
      detail: 'M',
    },
  ],
  documents: [
    {
      path: 'prompts/agent/base.md',
      name: 'base.md',
      dir: 'prompts/agent',
      ext: 'md',
      kind: 'prompt',
      status: '',
      tracked: true,
      size_bytes: 128,
      modified_ms: 1782727200000,
      roles: ['use', 'scan'],
      sources: [{ role: 'use', source: 'session LLM active', detail: 'ref hachee 11111111' }],
    },
    {
      path: 'docs/factory/WIRING_SPEC.md',
      name: 'WIRING_SPEC.md',
      dir: 'docs/factory',
      ext: 'md',
      kind: 'doc',
      status: '',
      tracked: true,
      size_bytes: 2048,
      modified_ms: 1782727300000,
      roles: ['read', 'scan'],
      sources: [{ role: 'read', source: 'context-pack frais', detail: 'ref hachee 22222222' }],
    },
    {
      path: 'tools/factory-operator/src/components/surfaces/DocumentsSurface.tsx',
      name: 'DocumentsSurface.tsx',
      dir: 'tools/factory-operator/src/components/surfaces',
      ext: 'tsx',
      kind: 'front',
      status: 'M',
      tracked: true,
      size_bytes: 4096,
      modified_ms: 1782727400000,
      roles: ['write', 'scan'],
      sources: [{ role: 'write', source: 'git status', detail: 'M' }],
    },
  ],
}

beforeEach(() => {
  vi.clearAllMocks()
  getProjectDocuments.mockResolvedValue(SNAPSHOT)
})

afterEach(() => vi.restoreAllMocks())

describe('DocumentsSurface', () => {
  it('renders the live project document map and pinned LLM usage', async () => {
    render(<DocumentsSurface sessionId="s1" />)

    await waitFor(() => expect(screen.getByTestId('documents-surface')).toBeInTheDocument())
    expect(getProjectDocuments).toHaveBeenCalledWith('s1', expect.any(AbortSignal))
    expect(screen.getByText(/3 fichiers/)).toBeInTheDocument()
    expect(screen.getByTestId('documents-pinned')).toHaveTextContent('prompts/agent/base.md')
    expect(screen.getByTestId('documents-pinned')).toHaveTextContent('session LLM active')
    expect(screen.getAllByTestId('project-document-card')).toHaveLength(3)
  })

  it('filters by role and search text without losing the pinned panel', async () => {
    const user = userEvent.setup()
    render(<DocumentsSurface sessionId="s1" />)

    await screen.findByText(/3 fichiers/)
    await user.click(screen.getByRole('button', { name: /écrit/ }))
    expect(screen.getAllByTestId('project-document-card')).toHaveLength(1)
    expect(screen.getByText('DocumentsSurface.tsx')).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: /tous/ }))
    await user.type(screen.getByLabelText('recherche fichiers'), 'WIRING')
    expect(await screen.findByText('WIRING_SPEC.md')).toBeInTheDocument()
    expect(screen.getAllByTestId('project-document-card')).toHaveLength(1)
    expect(screen.getByTestId('documents-pinned')).toHaveTextContent('prompt de base')
  })
})
