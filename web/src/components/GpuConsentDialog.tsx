// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `GpuConsentDialog` — Sprint 16 Phase C.
 *
 * Asks the user how much GPU they want to share with the public
 * SBFB network. Pattern: BOINC `UserOptInConsent` + GDPR Art. 7
 * (opt-in explicite, granular, withdrawable). Default = L1 ("mes
 * projets uniquement") so the dialog never pre-selects a sharing
 * level.
 *
 * The dialog mutates a local copy of the {@link ConsentConfig};
 * "Enregistrer" POSTs the full payload to `/consent/set`. The
 * coordinator persists it atomically and the worker's
 * `notify`-backed file watcher reloads its in-memory state on
 * the next claim tick — no restart needed.
 */

import { useEffect, useMemo, useState } from "react";
import { Heart, Info, Loader2, Plus, X } from "lucide-react";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Slider } from "@/components/ui/slider";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  type ConsentConfig,
  type ConsentLevel,
  DEFAULT_CONSENT,
  setConsent,
} from "@/api/consent";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  coordinatorUrl: string;
  initialConfig?: ConsentConfig;
  onSaved?: (cfg: ConsentConfig) => void;
}

const NODE_ID_RE = /^[0-9a-fA-F]{64}$/;

const LEVEL_LABELS: Record<
  ConsentLevel,
  { title: string; hint: string; threatNote: string }
> = {
  1: {
    title: "Mes projets uniquement",
    hint: "Aucun partage avec le réseau public. Sécurisé par défaut.",
    threatNote:
      "Aucune exposition tierce. Seules vos propres apps s'exécutent.",
  },
  2: {
    title: "Projets open source vérifiés",
    hint: "Accepte les apps publiées depuis un dépôt Git public et signées.",
    threatNote:
      "Apps open source vérifiées (SLSA L1). Exposition Sybil si contributeur malveillant.",
  },
  3: {
    title: "Projets spécifiques (whitelist)",
    hint: "Tu choisis manuellement chaque projet auquel tu contribues.",
    threatNote:
      "Apps sélectionnées manuellement. Vous êtes responsable de la vérification.",
  },
  4: {
    title: "Tous les projets publics",
    hint: "Le worker accepte n'importe quelle tâche publique du réseau.",
    threatNote:
      "Toute app publique du réseau. Risque maximum de consommation abusive.",
  },
};

export function GpuConsentDialog({
  open,
  onOpenChange,
  coordinatorUrl,
  initialConfig,
  onSaved,
}: Props) {
  const baseline = initialConfig ?? DEFAULT_CONSENT;

  const [level, setLevel] = useState<ConsentLevel>(baseline.level);
  const [maxWatts, setMaxWatts] = useState<number>(
    baseline.caps.max_watts ?? 400,
  );
  const [maxVramMb, setMaxVramMb] = useState<number>(
    baseline.caps.max_vram_mb ?? 16 * 1024,
  );
  const [maxHoursDay, setMaxHoursDay] = useState<number>(
    baseline.caps.max_hours_day ?? 12,
  );
  const [allowedIds, setAllowedIds] = useState<string[]>(
    baseline.allowed_project_ids,
  );
  const [pendingId, setPendingId] = useState("");
  const [pendingError, setPendingError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  // Re-sync local state when the parent passes a new initialConfig
  // (e.g. the dialog is reopened with a freshly fetched server state).
  useEffect(() => {
    if (!open) return;
    setLevel(baseline.level);
    setMaxWatts(baseline.caps.max_watts ?? 400);
    setMaxVramMb(baseline.caps.max_vram_mb ?? 16 * 1024);
    setMaxHoursDay(baseline.caps.max_hours_day ?? 12);
    setAllowedIds(baseline.allowed_project_ids);
    setPendingId("");
    setPendingError(null);
    setSaveError(null);
  }, [open, baseline]);

  const builtConfig = useMemo<ConsentConfig>(
    () => ({
      level,
      caps: {
        max_watts: maxWatts,
        max_vram_mb: maxVramMb,
        max_hours_day: maxHoursDay,
      },
      allowed_project_ids: allowedIds,
      own_node_id: baseline.own_node_id,
      level_threat_note: baseline.level_threat_note,
      residual_threats_acknowledged: baseline.residual_threats_acknowledged,
    }),
    [level, maxWatts, maxVramMb, maxHoursDay, allowedIds, baseline.own_node_id, baseline.level_threat_note, baseline.residual_threats_acknowledged],
  );

  function handleAddPending() {
    const trimmed = pendingId.trim().toLowerCase();
    if (!NODE_ID_RE.test(trimmed)) {
      setPendingError("Format attendu : node_id hex 64 caractères.");
      return;
    }
    if (allowedIds.includes(trimmed)) {
      setPendingError("Ce projet est déjà dans la whitelist.");
      return;
    }
    setAllowedIds((prev) => [...prev, trimmed]);
    setPendingId("");
    setPendingError(null);
  }

  function handleRemove(id: string) {
    setAllowedIds((prev) => prev.filter((p) => p !== id));
  }

  async function handleSave() {
    setSaving(true);
    setSaveError(null);
    try {
      const saved = await setConsent(coordinatorUrl, builtConfig);
      onSaved?.(saved);
      onOpenChange(false);
    } catch (e) {
      setSaveError(e instanceof Error ? e.message : "Erreur inconnue.");
    } finally {
      setSaving(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg" data-testid="gpu-consent-dialog">
        <DialogHeader>
          <DialogTitle>Partage GPU avec le réseau</DialogTitle>
          <DialogDescription>
            SBFB est un réseau P2P de calcul. Choisis comment ton
            GPU peut être utilisé. Tu peux modifier ce choix à tout
            moment depuis « Mon réseau ».
          </DialogDescription>
        </DialogHeader>

        <TooltipProvider>
          <RadioGroup<ConsentLevel>
            value={level}
            onValueChange={(v) => setLevel(v as ConsentLevel)}
            aria-label="Niveau de partage GPU"
          >
            {([1, 2, 3, 4] as const).map((lvl) => (
              <label
                key={lvl}
                className="flex cursor-pointer items-start gap-3 rounded-lg border border-white/[0.06] p-3 hover:bg-white/[0.04]"
              >
                <RadioGroupItem
                  value={lvl}
                  aria-label={LEVEL_LABELS[lvl].title}
                  data-testid={`consent-level-${lvl}`}
                />
                <div className="flex-1 space-y-0.5">
                  <div className="flex items-center gap-1.5">
                    <p className="text-sm font-medium">
                      L{lvl} — {LEVEL_LABELS[lvl].title}
                    </p>
                    <Tooltip>
                      <TooltipTrigger
                        className="inline-flex shrink-0 text-white/30 hover:text-white/60"
                        data-testid={`consent-threat-note-${lvl}`}
                      >
                        <Info className="h-3.5 w-3.5" />
                      </TooltipTrigger>
                      <TooltipContent side="right" className="max-w-xs">
                        <p className="text-xs">
                          {LEVEL_LABELS[lvl].threatNote}
                        </p>
                      </TooltipContent>
                    </Tooltip>
                  </div>
                  <p className="text-xs text-white/50">
                    {LEVEL_LABELS[lvl].hint}
                  </p>
                </div>
              </label>
            ))}
          </RadioGroup>
        </TooltipProvider>

        {level === 3 && (
          <section
            className="space-y-3 rounded-lg border border-white/[0.06] p-3"
            data-testid="consent-whitelist-section"
          >
            <div>
              <h4 className="text-sm font-medium">Whitelist L3</h4>
              <p className="text-xs text-white/40">
                Ajoute manuellement chaque projet (node_id hex 64
                caractères). Le bouton « Contribuer mon GPU » sur
                la page Parcourir fait la même chose en un clic.
              </p>
            </div>
            <div className="flex gap-2">
              <Input
                value={pendingId}
                onChange={(e) => setPendingId(e.target.value)}
                placeholder="node_id hex"
                aria-label="node_id hex à ajouter"
                data-testid="consent-whitelist-input"
              />
              <Button
                type="button"
                size="sm"
                onClick={handleAddPending}
                data-testid="consent-whitelist-add"
              >
                <Plus className="h-3.5 w-3.5" />
                Ajouter
              </Button>
            </div>
            {pendingError && (
              <p
                className="text-xs text-destructive"
                role="alert"
                data-testid="consent-whitelist-error"
              >
                {pendingError}
              </p>
            )}
            {allowedIds.length === 0 ? (
              <p className="text-xs text-white/40">
                Aucun projet pour le moment. Utilise « Contribuer
                mon GPU » sur la page Parcourir pour ajouter un
                projet en un clic.
              </p>
            ) : (
              <ul className="space-y-1.5">
                {allowedIds.map((id) => (
                  <li
                    key={id}
                    className="flex items-center justify-between gap-2 rounded bg-white/[0.04] px-2 py-1.5"
                  >
                    <code className="truncate font-mono text-[10px] text-white/60">
                      {id.slice(0, 16)}…{id.slice(-8)}
                    </code>
                    <button
                      type="button"
                      onClick={() => handleRemove(id)}
                      className="text-white/40 hover:text-white/80"
                      aria-label={`Retirer ${id}`}
                      data-testid={`consent-whitelist-remove-${id}`}
                    >
                      <X className="h-3.5 w-3.5" />
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </section>
        )}

        <section className="space-y-4 rounded-lg border border-white/[0.06] p-3">
          <div>
            <h4 className="text-sm font-medium">Limites strictes</h4>
            <p className="text-xs text-white/40">
              Le worker rejette toute tâche qui dépasse une de ces
              valeurs. Aucune contribution ne s'exécutera au-delà.
            </p>
          </div>
          <CapSlider
            label="Puissance max (W)"
            value={maxWatts}
            onChange={setMaxWatts}
            min={10}
            max={500}
            step={10}
            unit="W"
            testId="consent-cap-watts"
          />
          <CapSlider
            label="VRAM max (GB)"
            value={Math.round(maxVramMb / 1024)}
            onChange={(gb) => setMaxVramMb(gb * 1024)}
            min={1}
            max={24}
            step={1}
            unit="GB"
            testId="consent-cap-vram"
          />
          <CapSlider
            label="Heures par jour"
            value={maxHoursDay}
            onChange={setMaxHoursDay}
            min={0}
            max={24}
            step={0.5}
            unit="h"
            testId="consent-cap-hours"
          />
        </section>

        {saveError && (
          <p
            className="text-xs text-destructive"
            role="alert"
            data-testid="consent-save-error"
          >
            Erreur sauvegarde : {saveError}
          </p>
        )}

        <DialogFooter>
          <Button
            variant="outline"
            type="button"
            onClick={() => onOpenChange(false)}
            disabled={saving}
          >
            Annuler
          </Button>
          <Button
            type="button"
            onClick={handleSave}
            disabled={saving}
            data-testid="consent-save"
          >
            {saving ? (
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
            ) : (
              <Heart className="h-3.5 w-3.5" />
            )}
            Enregistrer
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

interface CapSliderProps {
  label: string;
  value: number;
  onChange: (value: number) => void;
  min: number;
  max: number;
  step: number;
  unit: string;
  testId: string;
}

function CapSlider({
  label,
  value,
  onChange,
  min,
  max,
  step,
  unit,
  testId,
}: CapSliderProps) {
  return (
    <div data-testid={testId}>
      <div className="mb-1.5 flex items-center justify-between text-xs">
        <span className="text-white/60">{label}</span>
        <span className="font-mono text-white/80">
          {value} {unit}
        </span>
      </div>
      <Slider
        value={value}
        onValueChange={(v) => onChange(typeof v === "number" ? v : v[0])}
        min={min}
        max={max}
        step={step}
        aria-label={label}
      />
    </div>
  );
}
