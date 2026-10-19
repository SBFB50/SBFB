export type CodexStatus = 'absent' | 'ok' | 'gap' | 'partiel' | 'sans-verdict'

export type CiStatus = 'vert' | 'rouge' | 'en-cours' | 'inconnu'

export type NexusSnapshot = {
  root: string
  sprint: number | null
  phase: string | null
  preflightVerdict: string | null
  reviewVerdict: string | null
  isReviewPass: boolean
  codex: CodexStatus
  rounds: number
  dirty: number
  untrackedCode: string[]
  residueFiles: string[]
  mutationMarkers: string[]
  goldenErrors: string[]
  ahead: number | null
  ci: CiStatus
  ciFailed: string[]
  updatedAt: number
}

export type PhaseRow = {
  id: string
  title: string
  status: 'fait' | 'en-cours' | 'prévu'
  commit: string | null
  date: string | null
  children: { id: string; title: string }[]
}

export type SprintRow = { version: string; sprint: number; state: string }

export type NexusHistory = {
  sprint: number | null
  phases: PhaseRow[]
  next: { sprint: number; source: string; phases: PhaseRow[] } | null
  sprints: SprintRow[]
}

declare module 'claude-code' {
  interface PluginState {
    'nexus-process': {
      snapshot: NexusSnapshot | null
      history: NexusHistory | null
      lightcheck: string | null
    }
  }
}
