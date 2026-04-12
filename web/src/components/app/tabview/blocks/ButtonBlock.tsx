// SPDX-License-Identifier: AGPL-3.0-or-later
import { useState } from "react";
import { useNavigate } from "react-router-dom";

import type { BlockTone, TabBlockButton } from "../schema";
import { useTabAppContext } from "../TabAppContext";
import { submitAppTask } from "@/api/coordinator";
import { Button } from "@/components/ui/button";

const TONE_VARIANT: Record<
  BlockTone,
  "default" | "secondary" | "destructive" | "outline"
> = {
  neutral: "outline",
  ok: "default",
  warn: "secondary",
  danger: "destructive",
};

type FeedbackKind = "idle" | "pending" | "success" | "error";

interface Feedback {
  kind: FeedbackKind;
  message?: string;
}

/**
 * Sprint 8 Phase A (D1/D4): the `task_submit` action is now
 * wired to the coordinator's ``POST /app/{name}/tasks/submit``
 * endpoint. The block reads the current coordinator URL and
 * app name from `TabAppContext`; if the context is missing (the
 * block is rendered outside an app tab, e.g. a unit test or a
 * future native tab), the button shows a disabled state with
 * an inline warning instead of throwing.
 *
 * The pre-Sprint-8 `console.warn` stub is gone — that was the
 * T4 tech debt Sprint 6 carved out and the Sprint 7 audit F-1
 * left to close in this sprint.
 */
export function ButtonBlock({ block }: { block: TabBlockButton }) {
  const navigate = useNavigate();
  const tabApp = useTabAppContext();
  const [feedback, setFeedback] = useState<Feedback>({ kind: "idle" });

  const onClick = async () => {
    if (block.action.kind === "route") {
      navigate(block.action.path);
      return;
    }

    // task_submit branch.
    if (tabApp === null) {
      setFeedback({
        kind: "error",
        message:
          "Action task_submit indisponible hors d'un contexte d'app " +
          "(AppsTab doit envelopper le rendu dans un TabAppContext).",
      });
      return;
    }

    setFeedback({ kind: "pending" });
    try {
      const response = await submitAppTask(
        tabApp.coordinatorUrl,
        tabApp.appName,
        {
          worker: block.action.worker,
          payload:
            (block.action.payload as Record<string, unknown> | null | undefined) ??
            {},
          priority: 5,
          parent_task_id: null,
        },
      );
      setFeedback({
        kind: "success",
        message: `Tâche soumise (${response.task_id}).`,
      });
    } catch (e) {
      setFeedback({
        kind: "error",
        message: e instanceof Error ? e.message : "Échec soumission",
      });
    }
  };

  const disabled =
    feedback.kind === "pending" ||
    (block.action.kind === "task_submit" && tabApp === null);

  return (
    <div className="space-y-1">
      <Button
        variant={TONE_VARIANT[block.tone]}
        size="sm"
        onClick={onClick}
        disabled={disabled}
      >
        {feedback.kind === "pending" ? `${block.label}…` : block.label}
      </Button>
      {feedback.kind === "success" && (
        <p className="text-[11px] text-green-500">{feedback.message}</p>
      )}
      {feedback.kind === "error" && (
        <p className="text-[11px] text-destructive">{feedback.message}</p>
      )}
    </div>
  );
}
