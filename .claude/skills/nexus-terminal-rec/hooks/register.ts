import type { EngineInterface, Register } from 'claude-code'

import { castEvent, castFileName, castHeader, currentPhase, maxSprint, rowToText } from './lib'

type $ = EngineInterface

const WIDTH = 120
const HEIGHT = 30
const FLUSH_MS = 2000
// $.fs.read plafonne à 4 Mio : on reste en dessous pour que le .cast reste relisible.
const MAX_BYTES = 3_500_000

// Repart de zéro à chaque rechargement : un rechargement ouvre un nouveau fichier.
const rec: {
  configuredRoot: string
  path: string | null
  header: string
  lines: string[]
  bytes: number
  startedAt: number
  isOn: boolean
  isDirty: boolean
  isFull: boolean
} = { configuredRoot: '', path: null, header: '', lines: [], bytes: 0, startedAt: 0, isOn: true, isDirty: false, isFull: false }

async function resolveRoot($: $): Promise<string | null> {
  const candidates: string[] = []
  if (rec.configuredRoot) candidates.push(rec.configuredRoot)
  try {
    const top = await $.process.run(['git', 'rev-parse', '--show-toplevel'], { cwd: await $.session.cwd(), timeoutMs: 5000 })
    if (top.exitCode === 0) candidates.push(top.stdout.trim())
  } catch {
    // pas de git
  }
  for (const c of candidates) {
    const base = c.replace(/\\/g, '/').replace(/\/$/, '')
    if (await $.fs.exists(`${base}/scripts/agent/agentctl.py`)) return base
  }
  return null
}

async function open($: $, root: string): Promise<void> {
  const entries = await $.fs.list(`${root}/.planning/active`).catch(() => [])
  const sprint = maxSprint(entries.map(e => e.name))
  const phase = sprint === null ? null : currentPhase(entries, sprint)
  rec.startedAt = await $.clock.now()
  rec.path = `${root}/.planning/terminal/${castFileName(sprint, phase, new Date(rec.startedAt).toISOString())}`
  rec.header = castHeader(WIDTH, HEIGHT, rec.startedAt / 1000)
  rec.lines = []
  rec.bytes = rec.header.length
  rec.isFull = false
  rec.isDirty = true
  await flush($)
}

async function push($: $, text: string): Promise<void> {
  if (rec.path === null || !rec.isOn || rec.isFull) return
  const line = castEvent(((await $.clock.now()) - rec.startedAt) / 1000, text)
  if (rec.bytes + line.length > MAX_BYTES) {
    rec.lines.push(castEvent(((await $.clock.now()) - rec.startedAt) / 1000, '\r\n[enregistrement arrêté : taille maximale atteinte]\r\n'))
    rec.isFull = true
  } else {
    rec.lines.push(line)
    rec.bytes += line.length + 1
  }
  rec.isDirty = true
}

async function flush($: $): Promise<void> {
  if (rec.path === null || !rec.isDirty) return
  rec.isDirty = false
  await $.fs.write(rec.path, `${[rec.header, ...rec.lines].join('\n')}\n`)
}

export const register: Register = (on, options) => {
  rec.configuredRoot = typeof options.nexusRoot === 'string' ? options.nexusRoot.trim() : ''

  on('session.start', async ($, e, next) => {
    const started = await next(e)
    const root = await resolveRoot($)
    if (root === null) return started
    await $.command.register({ name: 'nexus-rec', description: "Enregistrement .cast de la session : on, off ou état", argumentHint: 'on|off' })
    await open($, root)
    $.clock.every(FLUSH_MS, () => void flush($))
    return started
  })

  on('session.append', async ($, e, next) => {
    const stored = await next(e)
    if (e.agentId === undefined && rec.path !== null && rec.isOn) {
      const text = rowToText({ door: e.door, role: e.message.role, isMeta: e.message.isMeta, content: e.message.content })
      if (text !== null) await push($, text)
    }
    return stored
  }).catch(($, e, next) => next(e))

  on('session.end', async ($, e, next) => {
    await flush($).catch(() => undefined)
    return next(e)
  })

  on('command.run', { command: 'nexus-rec' }, async ($, e) => {
    const arg = e.args.trim().toLowerCase()
    if (arg === 'off') {
      await flush($)
      rec.isOn = false
    } else if (arg === 'on') {
      rec.isOn = true
    }
    const state = rec.path === null ? 'aucun fichier' : rec.path
    return { text: `Enregistrement ${rec.isOn ? 'actif' : 'en pause'}${rec.isFull ? ' (plein)' : ''} → ${state} (${rec.lines.length} événements)` }
  })
}
