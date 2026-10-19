// Fonctions pures de l'enregistreur asciicast v2, testables sans I/O.
// Format et nommage repris des .cast existants de nexus/.planning/terminal
// et de crates/sbfb-factory/src/terminal.rs:27.

export type Block = { type: string; [field: string]: unknown }
export type Row = { door: string; role?: string; isMeta?: boolean; content: readonly Block[] }

const RESET = '\x1b[0m'
const DIM = '\x1b[2m'
const PROMPT = '\x1b[1;36m'
const TOOL = '\x1b[33m'
const ERROR = '\x1b[31m'
const RESULT_LINES = 12

export function maxSprint(names: readonly string[]): number | null {
  let best: number | null = null
  for (const name of names) {
    const m = /^sprint(\d+)_/.exec(name)
    if (m) best = Math.max(best ?? 0, Number(m[1]))
  }
  return best
}

export function currentPhase(entries: readonly { name: string; mtimeMs: number }[], sprint: number): string | null {
  const re = new RegExp(`^sprint${sprint}_phase_([a-z0-9]+)_preflight\\.md$`, 'i')
  let best: { phase: string; mtimeMs: number } | null = null
  for (const e of entries) {
    const m = re.exec(e.name)
    if (m && (best === null || e.mtimeMs > best.mtimeMs)) best = { phase: m[1]!, mtimeMs: e.mtimeMs }
  }
  return best?.phase ?? null
}

/** `sprint{N}_phase_{X}_{horodatage}.cast`, comme terminal.rs:27 (":" interdit sous Windows). */
export function castFileName(sprint: number | null, phase: string | null, iso: string): string {
  const ts = iso.replace(/:/g, '-')
  return `sprint${sprint ?? 0}_phase_${(phase ?? 'session').toUpperCase()}_${ts}.cast`
}

export function castHeader(width: number, height: number, epochSeconds: number): string {
  return JSON.stringify({
    version: 2,
    width,
    height,
    timestamp: Math.floor(epochSeconds),
    env: { TERM: 'xterm-256color', SHELL: 'claude' },
  })
}

export function castEvent(seconds: number, text: string): string {
  return JSON.stringify([Math.round(seconds * 1e6) / 1e6, 'o', text])
}

const SECRET_PATTERNS: readonly RegExp[] = [
  /(x-sbfb-token\s*[:=]\s*)[^\s"']+/gi,
  /((?:CODEBERG|GITHUB|GH|NEO4J|ANTHROPIC|OPENAI|SBFB)_[A-Z_]*(?:TOKEN|KEY|PASSWORD|SECRET)\s*[:=]\s*)[^\s"']+/gi,
  /(Bearer\s+)[A-Za-z0-9._~+/=-]{16,}/g,
  /()\bgh[pousr]_[A-Za-z0-9]{20,}/g,
  /()\bsk-[A-Za-z0-9_-]{20,}/g,
]

export function maskSecrets(text: string): string {
  let out = text
  for (const re of SECRET_PATTERNS) out = out.replace(re, (_all, keep: string) => `${keep}***`)
  return out
}

function terminalText(text: string): string {
  return text.replace(/\r?\n/g, '\r\n')
}

function textOf(blocks: readonly Block[]): string {
  return blocks
    .filter(b => b.type === 'text' && typeof b.text === 'string')
    .map(b => b.text as string)
    .join('\n')
}

function summarize(input: unknown): string {
  if (input === null || typeof input !== 'object') return ''
  const o = input as Record<string, unknown>
  for (const key of ['command', 'file_path', 'pattern', 'description', 'url', 'query', 'prompt']) {
    const v = o[key]
    if (typeof v === 'string') return v.split('\n')[0]!.slice(0, 160)
  }
  return JSON.stringify(o).slice(0, 160)
}

function resultText(block: Block): string {
  const c = block.content
  if (typeof c === 'string') return c
  if (Array.isArray(c)) {
    return c
      .map(part => (part && typeof part === 'object' && (part as Block).type === 'text' ? String((part as Block).text ?? '') : '[contenu non texte]'))
      .join('\n')
  }
  return ''
}

function clip(text: string, max: number): string {
  const lines = text.split('\n')
  if (lines.length <= max) return text
  return `${lines.slice(0, max).join('\n')}\n… (+${lines.length - max} lignes)`
}

function tag(text: string, name: string): string | null {
  const m = new RegExp(`<${name}>([\\s\\S]*?)</${name}>`).exec(text)
  return m ? m[1]!.trim() : null
}

/** Les lignes d'une commande slash arrivent balisées (<command-name>, <local-command-stdout>…). */
export function commandText(text: string): string | null {
  if (!text.trim()) return null
  const name = tag(text, 'command-name')
  if (name !== null) {
    const args = tag(text, 'command-args')
    return `\n${PROMPT}❯ ${name}${args ? ` ${args}` : ''}${RESET}\n`
  }
  const output = tag(text, 'local-command-stdout') ?? tag(text, 'local-command-stderr')
  const body = (output ?? text.replace(/<[^>]+>/g, '')).trim()
  return body ? `${DIM}${clip(body, RESULT_LINES)}${RESET}\n` : null
}

/** Une ligne du transcript → texte de terminal (ANSI), ou null si rien à montrer. */
export function rowToText(row: Row): string | null {
  if (row.isMeta) return null
  let out = ''
  if (row.door === 'prompt') {
    const text = textOf(row.content)
    if (!text) return null
    out = `\n${PROMPT}❯ ${text}${RESET}\n`
  } else if (row.door === 'response') {
    const parts: string[] = []
    for (const b of row.content) {
      if (b.type === 'text' && typeof b.text === 'string' && b.text.trim()) parts.push(b.text)
      if (b.type === 'tool_use') parts.push(`${TOOL}● ${String(b.name ?? '?')}${RESET}${DIM} ${summarize(b.input)}${RESET}`)
    }
    if (parts.length === 0) return null
    out = `\n${parts.join('\n')}\n`
  } else if (row.door === 'tool-result') {
    const parts = row.content
      .filter(b => b.type === 'tool_result')
      .map(b => `${b.is_error === true ? ERROR : DIM}  ⎿ ${clip(resultText(b), RESULT_LINES).replace(/\n/g, '\n    ')}${RESET}`)
    if (parts.length === 0) return null
    out = `${parts.join('\n')}\n`
  } else if (row.door === 'command') {
    const text = commandText(textOf(row.content))
    if (!text) return null
    out = text
  } else {
    return null
  }
  return terminalText(maskSecrets(out))
}
