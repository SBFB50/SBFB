// SPDX-License-Identifier: AGPL-3.0-or-later

import { useState } from "react";
import { useTranslation } from "react-i18next";

export function TechnicalDetails({ command }: { command: string }) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);

  return (
    <div className="mt-3">
      <button
        type="button"
        onClick={(e) => {
          e.stopPropagation();
          setOpen(!open);
        }}
        className="text-xs text-muted-foreground hover:text-primary transition-colors"
      >
        {open ? "▾" : "▸"} {t("technical.toggle")}
      </button>
      {open && (
        <pre className="mt-2 rounded-md border border-border bg-background p-3 text-xs text-muted-foreground overflow-x-auto">
          {command}
        </pre>
      )}
    </div>
  );
}
