import { expect, test } from 'claude-code/testing'

const PANE = {
  plugin: 'nexus-process',
  component: 'Pane',
  requestId: 'nexus-phase',
  props: {
    title: 'nexus — phase',
    isFocused: false,
    bodyColumns: 60,
    placement: 'dock',
    scroll: { offset: 0, bodyRows: 20 },
    view: {},
  },
} as const

test('hors nexus, le panneau le dit sur chaque surface', async $ => {
  for (const surface of ['terminal', 'desktop'] as const) {
    const ui = await $.ui.mount({ ...PANE, surface })
    expect(await ui.find({ type: 'Text', text: /Pas de repo nexus/ })).toBeDefined()
    await ui.unmount()
  }
})
