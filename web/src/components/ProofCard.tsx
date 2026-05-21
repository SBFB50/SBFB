// SPDX-License-Identifier: AGPL-3.0-or-later
import { useState } from "react";
import {
  Award,
  ChevronDown,
  ChevronUp,
  FileCheck,
  GitBranch,
  Loader2,
  Scale,
  Shield,
  ShieldAlert,
  Timer,
  Users,
} from "lucide-react";

export interface ProofCardLayer {
  label: string;
  value: string;
  present: boolean;
}

export interface ProofCardData {
  project_id: string;
  project_name: string;
  hash: { archive_hash: string | null; provenance_hash: string | null };
  license: { spdx: string | null; source: string };
  freshness: {
    last_verified_at: string | null;
    age_days: number | null;
    state: "fresh" | "aging" | "stale" | "unknown";
  };
  provenance: {
    verified: boolean;
    repo_url: string | null;
    commit_sha: string | null;
    slsa_level: number;
  };
  risk: { level: "low" | "medium" | "high"; factors: string[] };
  curation: { curator_count: number; curator_names: string[] };
  confidence: number;
  formula_version: number;
}

interface Props {
  card: ProofCardData | null;
  loading?: boolean;
}

const RISK_FACTOR_LABELS: Record<string, string> = {
  no_provenance: "Pas de provenance",
  unverified_deploy: "Deploiement sans attestation",
  stale_source: "Source obsolete",
  old_release: "Version ancienne",
};

const FRESHNESS_LABELS: Record<string, string> = {
  fresh: "Recent",
  aging: "Vieillissant",
  stale: "Obsolete",
  unknown: "Inconnu",
};

const RISK_LEVEL_LABELS: Record<string, string> = {
  low: "Faible",
  medium: "Moyen",
  high: "Eleve",
};

function scoreColor(confidence: number): string {
  if (confidence >= 70) return "text-emerald-400";
  if (confidence >= 40) return "text-amber-400";
  return "text-red-400";
}

function scoreBgColor(confidence: number): string {
  if (confidence >= 70) return "bg-emerald-500/15";
  if (confidence >= 40) return "bg-amber-500/15";
  return "bg-red-500/15";
}

function riskBadgeColor(level: string): string {
  if (level === "low") return "bg-emerald-500/15 text-emerald-400";
  if (level === "medium") return "bg-amber-500/15 text-amber-400";
  return "bg-red-500/15 text-red-400";
}

function buildLayers(card: ProofCardData): ProofCardLayer[] {
  const layers: ProofCardLayer[] = [];

  layers.push({
    label: "Provenance",
    value: card.provenance.verified
      ? `Attestation SLSA L${card.provenance.slsa_level}`
      : "Sans attestation",
    present: card.provenance.verified,
  });

  layers.push({
    label: "Licence",
    value: card.license.spdx ?? "Non specifiee",
    present: card.license.spdx !== null,
  });

  layers.push({
    label: "Fraicheur",
    value: card.freshness.age_days !== null
      ? `${FRESHNESS_LABELS[card.freshness.state] ?? card.freshness.state} (${card.freshness.age_days} j)`
      : FRESHNESS_LABELS[card.freshness.state] ?? card.freshness.state,
    present: card.freshness.state === "fresh" || card.freshness.state === "aging",
  });

  layers.push({
    label: "Curation",
    value: card.curation.curator_count > 0
      ? `${card.curation.curator_count} curateur${card.curation.curator_count > 1 ? "s" : ""}`
      : "Aucun curateur",
    present: card.curation.curator_count > 0,
  });

  layers.push({
    label: "Archive",
    value: card.hash.archive_hash ? "Hash disponible" : "Pas de hash",
    present: card.hash.archive_hash !== null,
  });

  if (card.provenance.repo_url) {
    layers.push({
      label: "Source",
      value: card.provenance.repo_url,
      present: true,
    });
  }

  return layers;
}

const LAYER_ICONS: Record<string, typeof Shield> = {
  Provenance: FileCheck,
  Licence: Scale,
  Fraicheur: Timer,
  Curation: Users,
  Archive: Shield,
  Source: GitBranch,
};

export function ProofCard({ card, loading }: Props) {
  const [expanded, setExpanded] = useState(false);

  if (loading) {
    return (
      <div
        className="flex items-center gap-1 rounded-full bg-white/[0.08] px-3 py-1.5 text-[11px] text-white/50"
        data-testid="proof-card-loading"
      >
        <Loader2 className="h-3 w-3 animate-spin" />
        Preuve...
      </div>
    );
  }

  if (!card) return null;

  const layers = buildLayers(card);

  return (
    <div className="relative" data-testid="proof-card">
      {/* Badge (always visible) */}
      <button
        type="button"
        onClick={() => setExpanded((prev) => !prev)}
        className={`flex items-center gap-1.5 rounded-full px-3 py-1.5 text-[11px] font-medium transition-colors ${scoreBgColor(card.confidence)} ${scoreColor(card.confidence)} hover:opacity-80`}
        data-testid="proof-card-toggle"
        title="Carte de preuve — cliquer pour les details"
      >
        <Award className="h-3 w-3" />
        <span data-testid="proof-card-score">{card.confidence}/100</span>
        {expanded ? (
          <ChevronUp className="h-3 w-3" />
        ) : (
          <ChevronDown className="h-3 w-3" />
        )}
      </button>

      {/* Expanded card */}
      {expanded && (
        <div
          className="absolute right-0 top-full z-50 mt-2 w-80 rounded-xl border border-white/[0.08] bg-[#12121a]/95 p-4 shadow-2xl backdrop-blur-xl"
          data-testid="proof-card-details"
        >
          {/* Header */}
          <div className="mb-3 flex items-center justify-between">
            <h3 className="text-sm font-bold text-white">Carte de preuve</h3>
            <span
              className={`rounded-full px-2 py-0.5 text-[10px] font-medium ${riskBadgeColor(card.risk.level)}`}
              data-testid="proof-card-risk-level"
            >
              Risque {RISK_LEVEL_LABELS[card.risk.level] ?? card.risk.level}
            </span>
          </div>

          {/* Score bar */}
          <div className="mb-4">
            <div className="mb-1 flex items-baseline justify-between">
              <span className="text-xs text-white/60">Score de completude</span>
              <span className={`text-lg font-bold ${scoreColor(card.confidence)}`}>
                {card.confidence}
              </span>
            </div>
            <div className="h-1.5 w-full overflow-hidden rounded-full bg-white/[0.08]">
              <div
                className={`h-full rounded-full transition-all ${
                  card.confidence >= 70
                    ? "bg-emerald-400"
                    : card.confidence >= 40
                      ? "bg-amber-400"
                      : "bg-red-400"
                }`}
                style={{ width: `${card.confidence}%` }}
              />
            </div>
          </div>

          {/* Evidence layers */}
          <div className="mb-3 space-y-2" data-testid="proof-card-layers">
            {layers.map((layer) => {
              const Icon = LAYER_ICONS[layer.label] ?? Shield;
              return (
                <div
                  key={layer.label}
                  className="flex items-center gap-2 text-xs"
                  data-testid={`proof-card-layer-${layer.label.toLowerCase()}`}
                >
                  <Icon className={`h-3.5 w-3.5 ${layer.present ? "text-emerald-400" : "text-white/30"}`} />
                  <span className="text-white/60">{layer.label}</span>
                  <span className={`ml-auto truncate ${layer.present ? "text-white" : "text-white/40"}`}>
                    {layer.value}
                  </span>
                </div>
              );
            })}
          </div>

          {/* Risk factors */}
          {card.risk.factors.length > 0 && (
            <div className="border-t border-white/[0.06] pt-3" data-testid="proof-card-risk-factors">
              <div className="mb-1.5 flex items-center gap-1 text-xs text-white/60">
                <ShieldAlert className="h-3 w-3" />
                Facteurs de risque
              </div>
              <div className="space-y-1">
                {card.risk.factors.map((factor) => (
                  <div
                    key={factor}
                    className="flex items-center gap-1.5 text-[11px] text-amber-400/80"
                    data-testid="proof-card-risk-factor"
                  >
                    <span className="h-1 w-1 rounded-full bg-amber-400" />
                    {RISK_FACTOR_LABELS[factor] ?? factor}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Footer */}
          <div className="mt-3 border-t border-white/[0.06] pt-2 text-[10px] text-white/30">
            Formule v{card.formula_version} — score calcule localement
          </div>
        </div>
      )}
    </div>
  );
}
