// SPDX-License-Identifier: AGPL-3.0-or-later

import type { ProofCardData } from "./types";

export function ProofCard({ proof }: { proof: ProofCardData }) {
  return (
    <div className="rounded-lg border border-[#30363d] bg-[#1c2128] p-4">
      <h3 className="mb-3 text-sm font-semibold text-[#58a6ff]">
        Preuve de provenance
      </h3>
      <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 text-xs">
        <dt className="text-[#8b949e]">Commit source</dt>
        <dd className="font-mono text-white">{proof.commit_source}</dd>

        <dt className="text-[#8b949e]">Hash archive</dt>
        <dd className="truncate font-mono text-white">{proof.archive_hash}</dd>

        <dt className="text-[#8b949e]">Signataire</dt>
        <dd className="truncate font-mono text-white">
          {proof.signer_pubkey}
        </dd>

        <dt className="text-[#8b949e]">Verdict</dt>
        <dd>
          {proof.verified ? (
            <span className="text-[#3fb950]">✓ Vérifié</span>
          ) : (
            <span className="text-[#d29922]">Non vérifié</span>
          )}
        </dd>
      </dl>
    </div>
  );
}
