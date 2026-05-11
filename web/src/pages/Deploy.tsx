// SPDX-License-Identifier: AGPL-3.0-or-later

import { useState } from "react";
import { Globe, Rocket, CheckCircle2, AlertCircle, Loader2 } from "lucide-react";
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
            Aucun coordinateur
          </h2>
          <p className="text-sm text-white/60">
            Ajoute un coordinateur depuis l&apos;en-tete pour deployer une
            app.
          </p>
        </div>
      </div>
    );
  }

  return <DeployForm coordUrl={active.url} />;
}

function DeployForm({ coordUrl }: { coordUrl: string }) {
  const [repoUrl, setRepoUrl] = useState("");
  const [projectName, setProjectName] = useState("");
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
        <h1 className="text-2xl font-bold tracking-tight">Deployer une app</h1>
        <p className="mt-1 text-sm text-white/60">
          Clone un depot Git public, verifie l&apos;identite, et publie
          l&apos;app sur le reseau P2P.
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
              Deploiement en cours…
            </>
          ) : (
            <>
              <Rocket className="mr-2 h-4 w-4" />
              Deployer
            </>
          )}
        </Button>
      </form>

      {deployed && result.kind === "data" && (
        <div className="rounded-lg border border-emerald-500/20 bg-emerald-500/5 p-4" data-testid="deploy-success">
          <div className="flex items-center gap-2 text-emerald-400">
            <CheckCircle2 className="h-5 w-5" />
            <span className="font-medium">App deployee</span>
          </div>
          <dl className="mt-3 space-y-2 text-sm">
            <div>
              <dt className="text-white/50">Hash</dt>
              <dd className="font-mono text-xs text-white/80 break-all">
                {result.body.hash}
              </dd>
            </div>
            {result.body.provenance_hash && (
              <div>
                <dt className="text-white/50">Provenance</dt>
                <dd className="font-mono text-xs text-white/80 break-all">
                  {result.body.provenance_hash}
                </dd>
              </div>
            )}
            {result.body.commit_sha && (
              <div>
                <dt className="text-white/50">Commit</dt>
                <dd className="font-mono text-xs text-white/80 break-all">
                  {result.body.commit_sha}
                </dd>
              </div>
            )}
          </dl>
        </div>
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

export const Component = Deploy;
