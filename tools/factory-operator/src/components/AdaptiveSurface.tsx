// SPDX-License-Identifier: AGPL-3.0-or-later
import type { ComponentPropsWithoutRef, ElementType, ReactNode } from 'react'

export type AdaptiveSurfaceKind =
  | 'steer'
  | 'verify'
  | 'surface-host'
  | 'procede'
  | 'sessions'
  | 'knowledge'
  | 'documents'
  | 'diff'
  | 'terminal'
  | 'context-pack'

type AdaptiveSurfaceProps<T extends ElementType> = {
  as?: T
  kind: AdaptiveSurfaceKind
  children: ReactNode
  className?: string
  testId?: string
  labelledBy?: string
  dir?: 'ltr' | 'rtl' | 'auto'
} & Omit<ComponentPropsWithoutRef<T>, 'as' | 'children' | 'className' | 'dir'>

export function AdaptiveSurface<T extends ElementType = 'div'>({
  as,
  kind,
  children,
  className,
  testId,
  labelledBy,
  ...props
}: AdaptiveSurfaceProps<T>) {
  const Component = as ?? 'div'
  return (
    <Component
      data-adaptive-surface={kind}
      data-testid={testId}
      aria-labelledby={labelledBy}
      className={className ? `adaptive-surface ${className}` : 'adaptive-surface'}
      {...props}
    >
      {children}
    </Component>
  )
}
