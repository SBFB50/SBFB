// SPDX-License-Identifier: AGPL-3.0-or-later
import * as React from "react"

const MOBILE_BREAKPOINT = 768

export function useIsMobile() {
  const subscribe = React.useCallback((cb: () => void) => {
    const mql = window.matchMedia(`(max-width: ${MOBILE_BREAKPOINT - 1}px)`)
    mql.addEventListener("change", cb)
    return () => mql.removeEventListener("change", cb)
  }, [])

  const getSnapshot = React.useCallback(
    () => window.innerWidth < MOBILE_BREAKPOINT,
    [],
  )

  return React.useSyncExternalStore(subscribe, getSnapshot, () => false)
}
