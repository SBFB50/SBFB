// SPDX-License-Identifier: AGPL-3.0-or-later

import { useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import {
  Globe,
  Rocket,
  CheckCircle2,
  AlertCircle,
  Loader2,
  ChevronDown,
  ChevronRight,
  Signal,
} from "lucide-react";
import { useMutation } from "@tanstack/react-query";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  type DeployFromRepoRequest,
  type DeployResponse,
  type DaemonResult,
  deployFromRepo,
} from "@/api/daemon";
import {
  selectActiveCoordinator,
  useProjectStore,
} from "@/stores/projectStore";

export default function Deploy() {
  const active = useProjectStore(selectActiveCoordinator);

  if (!active) {
    return (
      <div className="flex min-h-[80vh] items-center justify-center">
        <div className="glass-card max-w-md p-8 text-center">
          <Globe className="mx-auto mb-4 h-12 w-12 text-purple-400" />
          <h2 className="mb-2 text-xl font-bold">
            Aucun noeud actif
          </h2>
          <p className="text-sm text-white/60">
            Connecte-toi a un noeud depuis l&apos;en-tete pour publier une
            app.
          </p>
        </div>
      </div>
    );
  }

  return <DeployForm coordUrl={active.url} />;
}

function DeployForm({ coordUrl }: { coordUrl: string }) {
  // Greffe D (app tombee → "La remettre en ligne") prefills the publish form
  // via query params so the user lands on a ready-to-submit form. Phase C
  // turns this into a true fork→redeploy under the local node identity.
  const [searchParams] = useSearchParams();
  const [repoUrl, setRepoUrl] = useState(
    () => searchParams.get("repo_url") ?? "",
  );
  const [projectName, setProjectName] = useState(
    () => searchParams.get("project_name") ?? "",
  );
  const [description, setDescription] = useState("");

  const mutation = useMutation<
    DaemonResult<DeployResponse>,
    Error,
    DeployFromRepoRequest
  >({
    mutationFn: (req) => deployFromRepo(coordUrl, req),
  });

  const result = mutation.data;
  const deployed = result?.kind === "data" && result.body.deployed;

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!repoUrl.trim() || !projectName.trim()) return;
    mutation.mutate({
      repo_url: repoUrl.trim(),
      project_name: projectName.trim(),
      description: description.trim(),
    });
  }

  return (
    <div className="mx-auto max-w-2xl space-y-8">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Publier une app</h1>
        <p className="mt-1 text-sm text-white/60">
          Clone un depot Git public, verifie l&apos;identite, et met l&apos;app
          en ligne sur le reseau.
        </p>
      </div>

      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="space-y-2">
          <label htmlFor="repo-url" className="text-sm font-medium text-white/80">
            URL du depot Git
          </label>
          <Input
            id="repo-url"
            data-testid="repo-url"
            type="url"
            placeholder="https://github.com/user/repo.git"
            value={repoUrl}
            onChange={(e) => setRepoUrl(e.target.value)}
            required
            className="bg-white/[0.04] border-white/10"
          />
        </div>

        <div className="space-y-2">
          <label htmlFor="project-name" className="text-sm font-medium text-white/80">
            Nom du projet
          </label>
          <Input
            id="project-name"
            data-testid="project-name"
            placeholder="mon-app"
            value={projectName}
            onChange={(e) => setProjectName(e.target.value)}
            required
            className="bg-white/[0.04] border-white/10"
          />
        </div>

        <div className="space-y-2">
          <label htmlFor="description" className="text-sm font-medium text-white/80">
            Description
          </label>
          <Input
            id="description"
            data-testid="description"
            placeholder="Une app web P2P..."
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            className="bg-white/[0.04] border-white/10"
          />
        </div>

        <Button
          type="submit"
          disabled={mutation.isPending || !repoUrl.trim() || !projectName.trim()}
          className="w-full"
          data-testid="deploy-submit"
        >
          {mutation.isPending ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              Publication en cours…
            </>
          ) : (
            <>
              <Rocket className="mr-2 h-4 w-4" />
              Publier sur le reseau
            </>
          )}
        </Button>

        {/*
          Ligne de verite (design §5 Flow 1) — surfaced BEFORE the click so the
          IPFS "upload != perpetual availability" trap is pre-empted. There is
          NO host/target field anywhere on this form (verrou §8(1)): publishing
          is a local signed identity act, choosing where would re-attribute the
          author.
        */}
        <p
          className="flex items-start gap-2 text-xs leading-relaxed text-white/40"
          data-testid="deploy-truth-line"
        >
          <Globe className="mt-0.5 h-3.5 w-3.5 shrink-0" />
          Ton noeud signe cette app et la garde en ligne. Elle reste joignable
          tant que ton noeud tourne.
        </p>
      </form>

      {deployed && result.kind === "data" && (
        <PublishSuccessCard body={result.body} />
      )}

      {result && result.kind !== "data" && (
        <div className="rounded-lg border border-red-500/20 bg-red-500/5 p-4" data-testid="deploy-error">
          <div className="flex items-center gap-2 text-red-400">
            <AlertCircle className="h-5 w-5" />
            <span className="font-medium">Erreur</span>
          </div>
          <p className="mt-2 text-sm text-white/60">{result.reason}</p>
        </div>
      )}
    </div>
  );
}

/**
 * Success card (design §5) — replaces the raw `<dl>` Hash/Provenance/Commit
 * dump. Surfaces the human truth ("Ton noeud la garde en ligne") with the
 * cryptographic detail folded behind "Details techniques" (advanced). ZERO
 * host field.
 */
function PublishSuccessCard({ body }: { body: DeployResponse }) {
  const [showTech, setShowTech] = useState(false);

  return (
    <div
      className="rounded-lg border border-emerald-500/20 bg-emerald-500/5 p-4"
      data-testid="deploy-success"
    >
      <div className="flex items-center gap-2 text-emerald-400">
        <CheckCircle2 className="h-5 w-5" />
        <span className="font-medium">App publiee et en ligne</span>
      </div>

      <p className="mt-1 text-sm text-white/70">
        Ton noeud la garde en ligne.
      </p>

      <span
        className="mt-3 inline-flex items-center gap-1.5 rounded-full bg-emerald-500/10 px-2.5 py-0.5 text-[11px] font-medium text-emerald-300"
        data-testid="deploy-online-pill"
      >
        <Signal className="h-3 w-3" />
        En ligne sur ton noeud
      </span>

      <div className="mt-3 flex items-start gap-2 rounded-md border border-amber-500/15 bg-amber-500/[0.06] p-2.5 text-xs leading-relaxed text-amber-200/80">
        <AlertCircle className="mt-0.5 h-3.5 w-3.5 shrink-0" />
        Quand tu fermes ton noeud, l&apos;app reste en ligne seulement si un
        autre pair la garde.
      </div>

      <div className="mt-4 flex flex-wrap items-center gap-3">
        <Link
          to="/browse"
          className="inline-flex items-center gap-1.5 rounded-lg bg-white/[0.08] px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-white/[0.15]"
          data-testid="deploy-view-app"
        >
          Voir la fiche de l&apos;app
        </Link>
        <button
          type="button"
          onClick={() => setShowTech((v) => !v)}
          className="inline-flex items-center gap-1 text-xs text-white/40 transition-colors hover:text-white/70"
          data-testid="deploy-tech-toggle"
          aria-expanded={showTech}
        >
          {showTech ? (
            <ChevronDown className="h-3.5 w-3.5" />
          ) : (
            <ChevronRight className="h-3.5 w-3.5" />
          )}
          Details techniques
        </button>
      </div>

      {showTech && (
        <dl className="mt-3 space-y-2 text-sm" data-testid="deploy-tech-details">
          <div>
            <dt className="text-white/50">Hash</dt>
            <dd className="font-mono text-xs text-white/80 break-all">
              {body.hash}
            </dd>
          </div>
          {body.provenance_hash && (
            <div>
              <dt className="text-white/50">Provenance</dt>
              <dd className="font-mono text-xs text-white/80 break-all">
                {body.provenance_hash}
              </dd>
            </div>
          )}
          {body.commit_sha && (
            <div>
              <dt className="text-white/50">Commit</dt>
              <dd className="font-mono text-xs text-white/80 break-all">
                {body.commit_sha}
              </dd>
            </div>
          )}
        </dl>
      )}
    </div>
  );
}

export const Component = Deploy;
