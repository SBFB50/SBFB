// SPDX-License-Identifier: AGPL-3.0-or-later
import { useCallback, useEffect, useRef, useState } from "react";
import { Check, Copy, Loader2, ShieldCheck, ShieldX } from "lucide-react";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { authFetch } from "@/api/auth";

interface ProvenanceRecord {
  repo_url: string;
  commit_sha: string;
  artifact_hash: string;
  signature: string;
  node_id: string;
  timestamp: string;
  schema_version: number;
}

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  coordUrl: string;
  projectId: string;
}

type FetchResult =
  | { kind: "loaded"; record: ProvenanceRecord; verified: boolean }
  | { kind: "empty" }
  | { kind: "error"; message: string };

function truncate(s: string, len = 12): string {
  return s.length > len ? s.slice(0, len) + "…" : s;
}

async function doFetch(
  coordUrl: string,
  projectId: string,
): Promise<FetchResult> {
  try {
    const resp = await authFetch(
      `${coordUrl}/api/v1/project/${encodeURIComponent(projectId)}/provenance`,
    );
    if (resp.status === 404) return { kind: "empty" };
    if (!resp.ok) return { kind: "error", message: `HTTP ${resp.status}` };
    const data = (await resp.json()) as {
      record: ProvenanceRecord;
      verified: boolean;
    };
    return { kind: "loaded", record: data.record, verified: data.verified };
  } catch (e) {
    return {
      kind: "error",
      message: e instanceof Error ? e.message : String(e),
    };
  }
}

export function VerificationDetail({
  open,
  onOpenChange,
  coordUrl,
  projectId,
}: Props) {
  const [result, setResult] = useState<FetchResult | null>(null);
  const [copied, setCopied] = useState<string | null>(null);
  const fetchIdRef = useRef(0);

  useEffect(() => {
    if (!open) return;
    const id = ++fetchIdRef.current;
    void doFetch(coordUrl, projectId).then((res) => {
      if (fetchIdRef.current === id) setResult(res);
    });
  }, [open, coordUrl, projectId]);

  const handleOpenChange = useCallback(
    (next: boolean) => {
      if (!next) {
        fetchIdRef.current++;
        setResult(null);
      }
      onOpenChange(next);
    },
    [onOpenChange],
  );

  const reverify = useCallback(() => {
    setResult(null);
    const id = ++fetchIdRef.current;
    void doFetch(coordUrl, projectId).then((res) => {
      if (fetchIdRef.current === id) setResult(res);
    });
  }, [coordUrl, projectId]);

  const copyToClipboard = (text: string, field: string) => {
    void navigator.clipboard.writeText(text).then(() => {
      setCopied(field);
      setTimeout(() => setCopied(null), 1500);
    });
  };

  const loading = open && result === null;

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-md" data-testid="verification-detail">
        <DialogHeader>
          <DialogTitle>Details de verification</DialogTitle>
          <DialogDescription>
            Provenance et integrite du deploiement verifie.
          </DialogDescription>
        </DialogHeader>

        {loading && (
          <div className="flex items-center justify-center py-8" data-testid="verification-loading">
            <Loader2 className="h-6 w-6 animate-spin text-white/40" />
          </div>
        )}

        {result?.kind === "empty" && (
          <div className="py-6 text-center text-sm text-white/50" data-testid="verification-empty">
            Aucune provenance enregistree pour ce projet.
          </div>
        )}

        {result?.kind === "error" && (
          <div className="py-6 text-center text-sm text-red-400" data-testid="verification-error">
            Erreur : {result.message}
          </div>
        )}

        {result?.kind === "loaded" && (
          <div className="space-y-3">
            <div className="flex items-center gap-2">
              {result.verified ? (
                <ShieldCheck className="h-5 w-5 text-emerald-400" />
              ) : (
                <ShieldX className="h-5 w-5 text-red-400" />
              )}
              <span
                className={`text-sm font-medium ${result.verified ? "text-emerald-400" : "text-red-400"}`}
                data-testid="verification-result"
              >
                {result.verified ? "Signature valide" : "Signature invalide"}
              </span>
            </div>

            <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-2 text-xs">
              <dt className="text-white/40">Repo</dt>
              <dd>
                <a
                  href={result.record.repo_url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-400 underline underline-offset-2 hover:text-blue-300"
                  data-testid="prov-repo-url"
                >
                  {result.record.repo_url}
                </a>
              </dd>

              <dt className="text-white/40">Commit</dt>
              <dd className="flex items-center gap-1">
                <code className="text-white/70" data-testid="prov-commit-sha">
                  {truncate(result.record.commit_sha, 10)}
                </code>
                <button
                  type="button"
                  className="text-white/30 hover:text-white/60"
                  onClick={() => copyToClipboard(result.record.commit_sha, "commit")}
                >
                  {copied === "commit" ? (
                    <Check className="h-3 w-3 text-emerald-400" />
                  ) : (
                    <Copy className="h-3 w-3" />
                  )}
                </button>
              </dd>

              <dt className="text-white/40">Artifact</dt>
              <dd>
                <code className="text-white/70" data-testid="prov-artifact-hash">
                  {truncate(result.record.artifact_hash, 16)}
                </code>
              </dd>

              <dt className="text-white/40">Signature</dt>
              <dd>
                <code className="text-white/70" data-testid="prov-signature">
                  {truncate(result.record.signature, 16)}
                </code>
              </dd>

              <dt className="text-white/40">Noeud</dt>
              <dd>
                <code className="text-white/70" data-testid="prov-node-id">
                  {truncate(result.record.node_id, 12)}
                </code>
              </dd>

              <dt className="text-white/40">Date</dt>
              <dd className="text-white/70" data-testid="prov-timestamp">
                {new Date(result.record.timestamp).toLocaleString("fr-FR")}
              </dd>

              <dt className="text-white/40">Schema</dt>
              <dd className="text-white/70">
                v{result.record.schema_version}
              </dd>
            </dl>

            <Button
              variant="outline"
              size="sm"
              className="w-full"
              onClick={reverify}
              data-testid="verify-button"
            >
              <ShieldCheck className="mr-2 h-4 w-4" />
              Reverifier maintenant
            </Button>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
