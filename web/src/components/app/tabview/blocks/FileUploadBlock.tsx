/**
 * Sprint 9 Phase E — file upload drop zone block (v2 only).
 *
 * Renders a drag-and-drop zone that accepts files matching the
 * block's `accept` list. On drop or click-select, the component
 * POSTs the file to `POST /app/{appName}/files/upload` via the
 * coordinator API and shows a progress indicator. SSE progress
 * events from the D2 bridge are a nice-to-have for Sprint 10;
 * this initial version polls the upload response.
 */

import { useCallback, useRef, useState } from "react";
import type { TabBlockFileUpload } from "../schema";

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} o`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} Ko`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} Mo`;
}

export function FileUploadBlock({ block }: { block: TabBlockFileUpload }) {
  const [dragging, setDragging] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [result, setResult] = useState<{
    sha256: string;
    size: number;
    original_name: string;
  } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const handleFile = useCallback(
    async (file: File) => {
      if (file.size > block.max_size_bytes) {
        setError(
          `Le fichier dépasse la taille maximale (${formatSize(block.max_size_bytes)})`,
        );
        return;
      }
      setUploading(true);
      setError(null);
      setResult(null);
      try {
        const form = new FormData();
        form.append("file", file);
        // The app name is extracted from the current URL path
        // pattern /project/:project/app/:app/tabs/:tab
        const segments = window.location.pathname.split("/");
        const appIdx = segments.indexOf("app");
        const appName = appIdx >= 0 ? segments[appIdx + 1] : "unknown";

        const resp = await fetch(
          `http://127.0.0.1:18765/app/${appName}/files/upload`,
          { method: "POST", body: form },
        );
        if (!resp.ok) {
          const body = await resp.json().catch(() => ({}));
          throw new Error(
            (body as { detail?: string }).detail ??
              `Erreur serveur (${resp.status})`,
          );
        }
        const data = (await resp.json()) as {
          sha256: string;
          size: number;
          original_name: string;
        };
        setResult(data);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setUploading(false);
      }
    },
    [block.max_size_bytes],
  );

  const onDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setDragging(false);
      const file = e.dataTransfer.files[0];
      if (file) handleFile(file);
    },
    [handleFile],
  );

  const onDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragging(true);
  }, []);

  const onDragLeave = useCallback(() => setDragging(false), []);

  const onInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) handleFile(file);
    },
    [handleFile],
  );

  return (
    <div className="space-y-2">
      <p className="text-sm font-medium">{block.label}</p>
      <div
        role="button"
        tabIndex={0}
        data-testid="file-upload-dropzone"
        className={`flex min-h-[120px] cursor-pointer flex-col items-center justify-center rounded-md border-2 border-dashed p-6 text-center text-sm transition-colors ${
          dragging
            ? "border-primary bg-primary/5"
            : "border-muted-foreground/25 hover:border-primary/50"
        }`}
        onDrop={onDrop}
        onDragOver={onDragOver}
        onDragLeave={onDragLeave}
        onClick={() => inputRef.current?.click()}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") inputRef.current?.click();
        }}
      >
        <input
          ref={inputRef}
          type="file"
          className="hidden"
          accept={block.accept.join(",")}
          onChange={onInputChange}
        />
        {uploading ? (
          <p className="text-muted-foreground">Téléversement en cours...</p>
        ) : (
          <>
            <p className="text-muted-foreground">
              Glisser un fichier ici ou cliquer pour sélectionner
            </p>
            <p className="mt-1 text-xs text-muted-foreground/60">
              {block.accept.join(", ")} — max{" "}
              {formatSize(block.max_size_bytes)}
            </p>
          </>
        )}
      </div>
      {error && (
        <p className="text-sm text-destructive" data-testid="upload-error">
          {error}
        </p>
      )}
      {result && (
        <div
          className="rounded-md border bg-muted/50 p-3 text-sm"
          data-testid="upload-result"
        >
          <p className="font-medium">{result.original_name}</p>
          <p className="text-xs text-muted-foreground">
            SHA256: {result.sha256.slice(0, 16)}... — {formatSize(result.size)}
          </p>
        </div>
      )}
    </div>
  );
}
