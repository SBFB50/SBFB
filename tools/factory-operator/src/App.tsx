// SPDX-License-Identifier: AGPL-3.0-or-later

import { Routes, Route } from "react-router-dom";
import { Sidebar } from "@/components/Sidebar";
import { StatusBar } from "@/components/StatusBar";
import { useApi } from "@/hooks/useApi";
import { SprintOverview } from "@/pages/SprintOverview";
import { AgentSelector } from "@/pages/AgentSelector";
import { PhaseAssistant } from "@/pages/PhaseAssistant";
import { LintOperator } from "@/pages/LintOperator";
import { CommitAuditor } from "@/pages/CommitAuditor";
import { AgentTransfer } from "@/pages/AgentTransfer";
import { ContextPackBuilder } from "@/pages/ContextPackBuilder";
import { ActionCenter } from "@/pages/ActionCenter";
import { AgentChat } from "@/pages/AgentChat";
import { ActionLog } from "@/pages/ActionLog";
import { SprintHistory } from "@/pages/SprintHistory";

interface StatusData {
  sprint: number;
  head: string;
}

export function App() {
  const { data } = useApi<StatusData>("/status");

  return (
    <div className="flex min-h-screen">
      <Sidebar />
      <main className="flex-1 overflow-auto p-6 pb-12">
        <Routes>
          <Route path="/" element={<SprintOverview />} />
          <Route path="/agents" element={<AgentSelector />} />
          <Route path="/phase" element={<PhaseAssistant />} />
          <Route path="/lint" element={<LintOperator />} />
          <Route path="/audit" element={<CommitAuditor />} />
          <Route path="/transfer" element={<AgentTransfer />} />
          <Route path="/context" element={<ContextPackBuilder />} />
          <Route path="/actions" element={<ActionCenter />} />
          <Route path="/chat" element={<AgentChat />} />
          <Route path="/log" element={<ActionLog />} />
          <Route path="/history" element={<SprintHistory />} />
        </Routes>
      </main>
      <StatusBar head={data?.head ?? "..."} sprint={data?.sprint ?? 0} />
    </div>
  );
}
