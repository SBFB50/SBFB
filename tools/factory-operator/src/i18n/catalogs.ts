// SPDX-License-Identifier: AGPL-3.0-or-later
import { directionOf, i18n, type Locale } from './i18n'

/**
 * Lazily load + activate one locale catalog. The `.po` import is code-split per
 * language, so switching locales fetches only that language's chunk. Also
 * reflects the language + direction on `<html>` (the document `lang` for
 * assistive tech, `dir` for RTL mirroring - `index.html` ships `lang="fr"` but
 * no `dir`, so this is where direction first lands).
 */
export async function dynamicActivate(locale: Locale): Promise<void> {
  const { messages } = await import(`./locales/${locale}.po`)
  i18n.load(locale, messages)
  i18n.activate(locale)
  const el = document.documentElement
  el.lang = locale
  el.dir = directionOf(locale)
}
