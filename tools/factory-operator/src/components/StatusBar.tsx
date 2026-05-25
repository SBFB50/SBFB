// SPDX-License-Identifier: AGPL-3.0-or-later

import { useTranslation } from "react-i18next";

export function StatusBar({ head, sprint }: { head: string; sprint: number }) {
  const { t } = useTranslation();
  return (
    <footer className="fixed bottom-0 left-0 right-0 z-50 flex h-8 items-center gap-4 border-t border-border bg-sidebar px-4 text-xs text-muted-foreground">
      <span className="font-mono">{t("sprint.head", { sha: head })}</span>
      <span>{t("sprint.title", { number: sprint })}</span>
    </footer>
  );
}
