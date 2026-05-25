// SPDX-License-Identifier: AGPL-3.0-or-later

import { useState, useEffect, useId } from "react";
import { useTranslation } from "react-i18next";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  AlertTriangle,
  Code2,
  ShieldCheck,
  Bot,
  Terminal,
  Cpu,
  User,
  BrainCircuit,
} from "lucide-react";

const PROVIDER_KEYS = ["claude", "codex", "gpt", "local", "human"] as const;
type Provider = (typeof PROVIDER_KEYS)[number];

const PROVIDER_ICONS: Record<Provider, typeof Bot> = {
  claude: BrainCircuit,
  codex: Terminal,
  gpt: Bot,
  local: Cpu,
  human: User,
};

function ProviderCard({
  role,
  roleKey,
  icon: RoleIcon,
  value,
  onChange,
}: {
  role: string;
  roleKey: string;
  icon: typeof Code2;
  value: Provider;
  onChange: (v: Provider) => void;
}) {
  const { t } = useTranslation();
  const selectId = useId();
  const ProviderIcon = PROVIDER_ICONS[value];

  return (
    <Card className="transition-shadow duration-200 hover:ring-2 hover:ring-primary/20">
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-sm">
          <RoleIcon className="size-4 text-primary" />
          {role}
        </CardTitle>
        <CardDescription>{t(`agents.${roleKey}Desc`)}</CardDescription>
      </CardHeader>
      <CardContent>
        <label htmlFor={selectId} className="sr-only">
          {role}
        </label>
        <Select
          value={value}
          onValueChange={(v) => v && onChange(v as Provider)}
        >
          <SelectTrigger id={selectId} className="w-full">
            <SelectValue>
              <span className="flex items-center gap-2">
                <ProviderIcon className="size-4 text-muted-foreground" />
                {t(`agents.${value}`)}
              </span>
            </SelectValue>
          </SelectTrigger>
          <SelectContent>
            {PROVIDER_KEYS.map((p) => {
              const Icon = PROVIDER_ICONS[p];
              return (
                <SelectItem key={p} value={p}>
                  <Icon className="size-4 text-muted-foreground" />
                  {t(`agents.${p}`)}
                </SelectItem>
              );
            })}
          </SelectContent>
        </Select>
      </CardContent>
    </Card>
  );
}

export function AgentSelector() {
  const { t } = useTranslation();

  const [driver, setDriver] = useState<Provider>(
    () => (localStorage.getItem("factory-driver") as Provider) ?? "claude",
  );
  const [verifier, setVerifier] = useState<Provider>(
    () => (localStorage.getItem("factory-verifier") as Provider) ?? "codex",
  );

  useEffect(() => {
    localStorage.setItem("factory-driver", driver);
  }, [driver]);
  useEffect(() => {
    localStorage.setItem("factory-verifier", verifier);
  }, [verifier]);

  const sameAgent = driver === verifier;

  return (
    <TooltipProvider>
      <div className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">{t("agents.title")}</CardTitle>
            <CardDescription>{t("agents.subtitle")}</CardDescription>
          </CardHeader>
        </Card>

        <div className="grid gap-4 sm:grid-cols-2">
          <ProviderCard
            role={t("agents.whoCode")}
            roleKey="whoCode"
            icon={Code2}
            value={driver}
            onChange={setDriver}
          />
          <ProviderCard
            role={t("agents.whoVerify")}
            roleKey="whoVerify"
            icon={ShieldCheck}
            value={verifier}
            onChange={setVerifier}
          />
        </div>

        {sameAgent && (
          <Card className="border-[var(--yellow)]/40 bg-[var(--yellow)]/5">
            <CardContent className="flex items-start gap-3 p-4">
              <AlertTriangle className="mt-0.5 size-5 shrink-0 text-[var(--yellow)]" />
              <div className="space-y-1">
                <p className="text-sm font-medium text-[var(--yellow)]">
                  {t("agents.sameWarningTitle")}
                </p>
                <p className="text-xs text-muted-foreground">
                  {t("agents.sameWarning")}
                </p>
              </div>
            </CardContent>
          </Card>
        )}

        <Separator />

        <section aria-label={t("agents.currentConfig")}>
          <h2 className="mb-3 text-sm font-semibold text-muted-foreground">
            {t("agents.currentConfig")}
          </h2>
          <div className="flex flex-wrap gap-3">
            <Tooltip>
              <TooltipTrigger>
                <Badge variant="secondary" className="gap-1.5 px-3 py-1">
                  <Code2 className="size-3" />
                  {t(`agents.${driver}`)}
                </Badge>
              </TooltipTrigger>
              <TooltipContent>{t("agents.whoCode")}</TooltipContent>
            </Tooltip>
            <span className="self-center text-xs text-muted-foreground">+</span>
            <Tooltip>
              <TooltipTrigger>
                <Badge variant="outline" className="gap-1.5 px-3 py-1">
                  <ShieldCheck className="size-3" />
                  {t(`agents.${verifier}`)}
                </Badge>
              </TooltipTrigger>
              <TooltipContent>{t("agents.whoVerify")}</TooltipContent>
            </Tooltip>
          </div>
        </section>
      </div>
    </TooltipProvider>
  );
}
