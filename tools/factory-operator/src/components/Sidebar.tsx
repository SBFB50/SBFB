// SPDX-License-Identifier: AGPL-3.0-or-later

import { NavLink } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  LayoutDashboard,
  Users,
  Compass,
  AlertTriangle,
  GitCommitHorizontal,
  ArrowRightLeft,
  Package,
  Zap,
  Play,
  MessageSquare,
  ScrollText,
  History,
  Menu,
  X,
} from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";

const NAV_ENTRIES = [
  { to: "/", key: "sprint", icon: LayoutDashboard },
  { to: "/agents", key: "agents", icon: Users },
  { to: "/phase", key: "phase", icon: Compass },
  { to: "/lint", key: "lint", icon: AlertTriangle },
  { to: "/audit", key: "audit", icon: GitCommitHorizontal },
  { to: "/transfer", key: "transfer", icon: ArrowRightLeft },
  { to: "/context", key: "context", icon: Package },
  { to: "/actions", key: "actions", icon: Zap },
  { to: "/execute", key: "execute", icon: Play },
  { to: "/chat", key: "chat", icon: MessageSquare },
  { to: "/log", key: "log", icon: ScrollText },
  { to: "/history", key: "history", icon: History },
] as const;

export function Sidebar() {
  const { t } = useTranslation();
  const [mobileOpen, setMobileOpen] = useState(false);

  const navContent = (
    <ScrollArea className="flex-1">
      <nav className="flex flex-col gap-1 p-3">
        {NAV_ENTRIES.map((entry) => (
          <NavLink
            key={entry.to}
            to={entry.to}
            end={entry.to === "/"}
            onClick={() => setMobileOpen(false)}
            className={({ isActive }) =>
              `flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors ${
                isActive
                  ? "bg-primary/20 text-primary"
                  : "text-muted-foreground hover:bg-accent hover:text-foreground"
              }`
            }
          >
            <entry.icon size={18} />
            <span className="hidden xl:inline">{t(`nav.${entry.key}`)}</span>
          </NavLink>
        ))}
      </nav>
    </ScrollArea>
  );

  return (
    <>
      <Button
        variant="outline"
        size="icon"
        onClick={() => setMobileOpen(true)}
        className="fixed left-4 top-4 z-50 lg:hidden"
        aria-label="Menu"
      >
        <Menu size={20} />
      </Button>

      {mobileOpen && (
        <div
          className="fixed inset-0 z-40 bg-black/60 lg:hidden"
          onClick={() => setMobileOpen(false)}
        />
      )}

      <aside
        className={`fixed left-0 top-0 z-50 flex h-full flex-col border-r border-border bg-sidebar transition-transform lg:static lg:translate-x-0 ${
          mobileOpen ? "translate-x-0" : "-translate-x-full"
        } w-60 lg:w-16 xl:w-60`}
      >
        <div className="flex h-14 items-center justify-between border-b border-border px-4">
          <span className="text-sm font-semibold text-primary">
            <span className="hidden xl:inline">{t("app.title")}</span>
            <span className="xl:hidden">{t("app.shortTitle")}</span>
          </span>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setMobileOpen(false)}
            className="lg:hidden"
          >
            <X size={18} />
          </Button>
        </div>
        <Separator />
        {navContent}
      </aside>
    </>
  );
}
