// SPDX-License-Identifier: AGPL-3.0-or-later
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import * as api from '../../api/operator'
import { ContextPackInspector } from './ContextPackInspector'

vi.mock('../../api/operator', () => ({
  postContextPack: vi.fn(),
  getChatLog: vi.fn(),
  OperatorError: class extends Error {},
}))

const postContextPack = vi.mocked(api.postContextPack)
const getChatLog = vi.mocked(api.getChatLog)

function ref(path: string, hash: string) {
  return { path, hash, exists: true }
}

function pack(animejsHash: string): api.ContextPack {
  return {
    base_prompt: ref('prompts/agent/base.md', 'b0000001'),
    universal_prompt: ref('prompts/agent/universal.md', 'b0000002'),
    handoff_prompt: ref('prompts/agent/handoff.md', 'b0000003'),
    specialized_prompt: null,
    agent_system: ref('docs/agent/AGENT_SYSTEM.md', 'b0000004'),
    process_docs: [ref('CLAUDE.md', 'b0000005')],
    authoring_knowledge: [ref('docs/factory/knowledge/animejs/MANIFEST.json', animejsHash)],
    active_artifacts: [ref('.planning/active/sprint80_plan.md', 'b0000006')],
    runtime_context: {},
    chat_history_authoritative: false,
    notice: 'private chat history is non-authoritative',
  }
}

// The REAL chat/session pack the backend seals (handle_chat_session) is
// REDUCED: no agent_system / specialized_prompt / process_docs / active_artifacts.
function reducedSessionPack(animejsHash: string): Partial<api.ContextPack> {
  return {
    base_prompt: ref('prompts/agent/base.md', 'b0000001'),
    universal_prompt: ref('prompts/agent/universal.md', 'b0000002'),
    handoff_prompt: ref('prompts/agent/handoff.md', 'b0000003'),
    authoring_knowledge: [ref('docs/factory/knowledge/animejs/MANIFEST.json', animejsHash)],
    runtime_context: {},
    chat_history_authoritative: false,
    notice: 'private chat history is non-authoritative',
  }
}

beforeEach(() => {
  vi.clearAllMocks()
  postContextPack.mockResolvedValue(pack('abc12345'))
})
afterEach(() => vi.restoreAllMocks())

describe('ContextPackInspector (sealed pack S2 + brouillon J13)', () => {
  it('renders hashed references and the non-authoritative handoff notice', async () => {
    render(<ContextPackInspector />)
    await waitFor(() => expect(screen.getByTestId('context-pack-inspector')).toBeInTheDocument())

    // S2: the hash chip is shown — provenance, never the file content.
    expect(await screen.findByText('abc12345')).toBeInTheDocument()
    // J13: the brouillon is explicit that the Operator grants no verdict.
    expect(screen.getByText(/ne clôt aucun verdict/)).toBeInTheDocument()
    // No drift marker without a sealed session baseline.
    expect(screen.queryByText(/dérive — relu/)).toBeNull()
  })

  it('marks ONLY the drifted reference (D2), not the unchanged ones', async () => {
    // The active session was sealed with an OLDER animejs hash; the fresh pack
    // hash differs → the on-disk file drifted → "◦ dérive — relu". Every OTHER
    // reference keeps the same hash in both packs, so exactly ONE marker shows
    // (freshness is per-file, never a blanket verdict).
    getChatLog.mockResolvedValue({ id: 's1', context_pack: pack('old00000'), messages: [] })
    render(<ContextPackInspector sessionId="s1" />)
    const markers = await screen.findAllByText(/dérive — relu/)
    expect(markers).toHaveLength(1)
    // The unchanged base prompt is rendered with its hash chip and no marker.
    expect(screen.getByText('b0000001')).toBeInTheDocument()
  })

  it('copie le chemin et l empreinte d une référence (clipboard)', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined)
    Object.assign(navigator, { clipboard: { writeText } })
    render(<ContextPackInspector />)
    await waitFor(() => expect(screen.getByTestId('context-pack-inspector')).toBeInTheDocument())
    const paths = await screen.findAllByTestId('copy-path')
    fireEvent.click(paths[0])
    expect(writeText).toHaveBeenCalledWith('prompts/agent/base.md')
    const hashes = screen.getAllByTestId('copy-hash')
    fireEvent.click(hashes[0])
    expect(writeText).toHaveBeenCalledWith('b0000001')
  })

  it('affiche « copié » après une copie réussie', async () => {
    Object.assign(navigator, { clipboard: { writeText: vi.fn().mockResolvedValue(undefined) } })
    render(<ContextPackInspector />)
    const paths = await screen.findAllByTestId('copy-path')
    fireEvent.click(paths[0])
    expect(await screen.findByText('copié')).toBeInTheDocument()
  })

  it('drifts against a REDUCED real session pack without crashing (Codex round-2)', async () => {
    // The backend chat/session pack omits process_docs/active_artifacts/agent_system.
    // groups() coalesces the missing arrays → no throw → D2 still compares the
    // shared authoring_knowledge (its real target) and flags the drifted pack.
    getChatLog.mockResolvedValue({ id: 's1', context_pack: reducedSessionPack('old00000'), messages: [] })
    render(<ContextPackInspector sessionId="s1" />)
    const markers = await screen.findAllByText(/dérive — relu/)
    expect(markers).toHaveLength(1)
  })
})
