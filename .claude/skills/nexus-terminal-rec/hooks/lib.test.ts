import { describe, expect, test } from 'claude-code/testing'

import { castEvent, castFileName, castHeader, currentPhase, maskSecrets, maxSprint, rowToText } from './lib'

describe('format asciicast', () => {
  test('en-tête identique aux .cast existants', async () => {
    expect(castHeader(120, 30, 1782425632.9)).toBe(
      '{"version":2,"width":120,"height":30,"timestamp":1782425632,"env":{"TERM":"xterm-256color","SHELL":"claude"}}',
    )
  })

  test('événement de sortie', async () => {
    expect(castEvent(1.5, 'a\r\nb')).toBe('[1.5,"o","a\\r\\nb"]')
  })

  test('nommage de terminal.rs:27, sans deux-points', async () => {
    expect(castFileName(83, 'm1', '2026-10-19T10:39:14.107Z')).toBe('sprint83_phase_M1_2026-10-19T10-39-14.107Z.cast')
    expect(castFileName(null, null, '2026-10-19T10:39:14.107Z')).toBe('sprint0_phase_SESSION_2026-10-19T10-39-14.107Z.cast')
  })

  test('sprint et phase', async () => {
    expect(maxSprint(['sprint83_plan.md', 'sprint9_x.md'])).toBe(83)
    expect(currentPhase([{ name: 'sprint83_phase_m1_preflight.md', mtimeMs: 2 }, { name: 'sprint83_phase_m_preflight.md', mtimeMs: 1 }], 83)).toBe('m1')
  })
})

describe('masquage des secrets', () => {
  test('jetons connus', async () => {
    expect(maskSecrets('curl -H "x-sbfb-token: abc123def" http://x')).toBe('curl -H "x-sbfb-token: ***" http://x')
    expect(maskSecrets('CODEBERG_TOKEN=s3cr3tvalue')).toBe('CODEBERG_TOKEN=***')
    expect(maskSecrets('token ghp_abcdefghijklmnopqrstuvwxyz0123')).toBe('token ***')
    expect(maskSecrets('Authorization: Bearer abcdefghijklmnop1234')).toBe('Authorization: Bearer ***')
    expect(maskSecrets('rien de secret')).toBe('rien de secret')
  })
})

describe('lignes du transcript', () => {
  test('prompt en cyan, retours chariot de terminal', async () => {
    const text = rowToText({ door: 'prompt', role: 'user', content: [{ type: 'text', text: 'lance\nles tests' }] })
    expect(text).toBe('\r\n\x1b[1;36m❯ lance\r\nles tests\x1b[0m\r\n')
  })

  test('appel d’outil résumé', async () => {
    const text = rowToText({
      door: 'response',
      role: 'assistant',
      content: [
        { type: 'thinking', thinking: 'secret' },
        { type: 'tool_use', name: 'Bash', input: { command: 'cargo test\n--all' } },
      ],
    })
    expect(text).toContain('● Bash')
    expect(text).toContain('cargo test')
    expect(text).not.toContain('secret')
    expect(text).not.toContain('--all')
  })

  test('résultat tronqué et masqué', async () => {
    const long = Array.from({ length: 30 }, (_, i) => `ligne ${i}`).join('\n')
    const text = rowToText({ door: 'tool-result', content: [{ type: 'tool_result', content: `${long}\nCODEBERG_TOKEN=abc` }] })
    expect(text).toContain('(+19 lignes)')
    expect(text).not.toContain('ligne 29')
  })

  test('commande slash lisible', async () => {
    const typed = rowToText({
      door: 'command',
      content: [{ type: 'text', text: '<command-name>/nexus-rec</command-name>\n<command-message>nexus-rec</command-message>\n<command-args>off</command-args>' }],
    })
    expect(typed).toBe('\r\n\x1b[1;36m❯ /nexus-rec off\x1b[0m\r\n')
    const output = rowToText({ door: 'command', content: [{ type: 'text', text: '<local-command-stdout>Enregistrement en pause</local-command-stdout>' }] })
    expect(output).toBe('\x1b[2mEnregistrement en pause\x1b[0m\r\n')
  })

  test('les lignes méta et les pièces jointes sont ignorées', async () => {
    expect(rowToText({ door: 'prompt', isMeta: true, content: [{ type: 'text', text: 'x' }] })).toBe(null)
    expect(rowToText({ door: 'attachment', content: [{ type: 'text', text: 'x' }] })).toBe(null)
  })
})
