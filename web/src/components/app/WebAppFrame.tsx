// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 11 Phase C — sandboxed iframe skeleton for web app blobs.
 *
 * Sprint 11 ships the component shell with a placeholder. The
 * actual blob fetch + render via `blobUrl` is Sprint 12+ scope
 * (cross-node P2P fetch). The iframe uses a restrictive `sandbox`
 * attribute per Day 0 decision D1.
 */

interface WebAppFrameProps {
  blobUrl?: string;
}

export function WebAppFrame({ blobUrl }: WebAppFrameProps) {
  if (!blobUrl) {
    return (
      <div
        className="flex h-64 items-center justify-center rounded-lg border border-dashed border-border text-sm text-muted-foreground"
        data-testid="webapp-frame-placeholder"
      >
        Application web non disponible
      </div>
    );
  }
  return (
    <iframe
      src={blobUrl}
      sandbox="allow-scripts allow-same-origin"
      className="h-full w-full border-0"
      title="Application web"
      data-testid="webapp-frame-iframe"
    />
  );
}
