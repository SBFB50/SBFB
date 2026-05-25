// SPDX-License-Identifier: AGPL-3.0-or-later

import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { postApi } from "@/hooks/useApi";
import { Button } from "@/components/ui/button";
import {
  Card, CardContent, CardDescription, CardHeader, CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  Tooltip, TooltipContent, TooltipProvider, TooltipTrigger,
} from "@/components/ui/tooltip";
import { TechnicalDetails } from "@/components/TechnicalDetails";
import {
  PackageIcon, CopyIcon, CheckIcon, LoaderIcon, SparklesIcon,
  UserIcon, ShieldCheckIcon, SearchIcon, InfoIcon,
} from "lucide-react";

const PROVIDERS = ["claude", "codex", "gpt", "local"] as const;
const ROLES = ["driver", "verifier", "auditor"] as const;

const ROLE_ICONS: Record<string, typeof UserIcon> = {
  driver: UserIcon,
  verifier: ShieldCheckIcon,
  auditor: SearchIcon,
};

export function ContextPackBuilder() {
  const { t } = useTranslation();
  const [provider, setProvider] = useState("claude");
  const [role, setRole] = useState("driver");
  const [sprint, setSprint] = useState("");
  const [phase, setPhase] = useState("");
  const [result, setResult] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);

  const handleBuild = useCallback(async () => {
    setLoading(true);
    setResult(null);
    try {
      const json = await postApi<{ pack: string }>("/context-pack", {
        provider, role,
        sprint: sprint ? Number(sprint) : undefined,
        phase: phase || undefined,
      });
      setResult(typeof json.pack === "string" ? json.pack : JSON.stringify(json, null, 2));
    } catch {
      setResult(t("contextPack.error"));
    } finally {
      setLoading(false);
    }
  }, [provider, role, sprint, phase, t]);

  const handleCopy = useCallback(() => {
    if (result) {
      navigator.clipboard.writeText(result);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  }, [result]);

  const roleKey = role === "driver"
    ? "transfer.roleDriver"
    : role === "verifier"
      ? "transfer.roleVerifier"
      : "contextPack.roleAuditor";

  return (
    <div className="space-y-6">
      <div>
        <div className="flex items-center gap-2">
          <PackageIcon className="size-5 text-primary" />
          <h1 className="text-xl font-bold">{t("contextPack.title")}</h1>
        </div>
        <p className="mt-1 text-sm text-muted-foreground">{t("contextPack.description")}</p>
      </div>

      <Separator />

      <div className="grid gap-4 sm:grid-cols-2">
        <Card size="sm">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-sm">
              {t("contextPack.provider")}
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger render={<span />}>
                    <InfoIcon className="size-3.5 text-muted-foreground" />
                  </TooltipTrigger>
                  <TooltipContent>{t("contextPack.providerTooltip")}</TooltipContent>
                </Tooltip>
              </TooltipProvider>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <Select value={provider} onValueChange={(v) => v && setProvider(v)}>
              <SelectTrigger className="w-full" aria-label={t("contextPack.provider")}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {PROVIDERS.map((p) => (
                  <SelectItem key={p} value={p}>{t(`agents.${p}`)}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </CardContent>
        </Card>

        <Card size="sm">
          <CardHeader>
            <CardTitle className="text-sm">{t("contextPack.role")}</CardTitle>
          </CardHeader>
          <CardContent>
            <Select value={role} onValueChange={(v) => v && setRole(v)}>
              <SelectTrigger className="w-full" aria-label={t("contextPack.role")}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {ROLES.map((r) => {
                  const Icon = ROLE_ICONS[r];
                  return (
                    <SelectItem key={r} value={r}>
                      <span className="flex items-center gap-2">
                        {Icon && <Icon className="size-3.5 text-muted-foreground" />}
                        {t(r === "driver" ? "transfer.roleDriver" : r === "verifier" ? "transfer.roleVerifier" : "contextPack.roleAuditor")}
                      </span>
                    </SelectItem>
                  );
                })}
              </SelectContent>
            </Select>
          </CardContent>
        </Card>

        <Card size="sm">
          <CardHeader>
            <CardTitle className="text-sm">{t("contextPack.sprintLabel")}</CardTitle>
          </CardHeader>
          <CardContent>
            <Input
              type="number"
              value={sprint}
              onChange={(e) => setSprint(e.target.value)}
              placeholder={t("contextPack.autoDetect")}
              aria-label={t("contextPack.sprintLabel")}
            />
          </CardContent>
        </Card>

        <Card size="sm">
          <CardHeader>
            <CardTitle className="text-sm">{t("contextPack.phaseLabel")}</CardTitle>
          </CardHeader>
          <CardContent>
            <Input
              value={phase}
              onChange={(e) => setPhase(e.target.value)}
              placeholder={t("contextPack.autoDetect")}
              aria-label={t("contextPack.phaseLabel")}
            />
          </CardContent>
        </Card>
      </div>

      <div className="flex flex-wrap items-center gap-3">
        <Button onClick={handleBuild} disabled={loading} aria-label={t("contextPack.generate")}>
          {loading
            ? <LoaderIcon className="size-4 animate-spin" />
            : <SparklesIcon className="size-4" />
          }
          {t("contextPack.generate")}
        </Button>
        <div className="flex items-center gap-2">
          <Badge variant="outline">{t(`agents.${provider}`)}</Badge>
          <Badge variant="secondary">{t(roleKey)}</Badge>
          {sprint && <Badge variant="secondary">S{sprint}</Badge>}
          {phase && <Badge variant="secondary">{phase.toUpperCase()}</Badge>}
        </div>
      </div>

      <TechnicalDetails command={`POST /api/context-pack { provider: "${provider}", role: "${role}"${sprint ? `, sprint: ${sprint}` : ""}${phase ? `, phase: "${phase}"` : ""} }`} />

      {result && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              <span className="text-sm text-primary">{t("contextPack.result")}</span>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleCopy}
                aria-label={copied ? t("contextPack.copied") : t("contextPack.copy")}
              >
                {copied
                  ? <><CheckIcon className="size-3.5 text-[var(--green)]" /> {t("contextPack.copied")}</>
                  : <><CopyIcon className="size-3.5" /> {t("contextPack.copy")}</>
                }
              </Button>
            </CardTitle>
            <CardDescription>{t("contextPack.resultHint")}</CardDescription>
          </CardHeader>
          <CardContent>
            <ScrollArea className="max-h-96 rounded-md border border-border bg-background p-3">
              <pre className="whitespace-pre-wrap font-mono text-xs text-muted-foreground">{result}</pre>
            </ScrollArea>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
