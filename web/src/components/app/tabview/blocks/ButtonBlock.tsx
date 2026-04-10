import { useNavigate } from "react-router-dom";
import type { BlockTone, TabBlockButton } from "../schema";
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

export function ButtonBlock({ block }: { block: TabBlockButton }) {
  const navigate = useNavigate();
  const onClick = () => {
    if (block.action.kind === "route") {
      navigate(block.action.path);
    } else {
      // Sprint 6: task_submit is a no-op placeholder — Sprint 7/8
      // will wire it to coordinator.submitTask once the app
      // context surfaces the per-tab coordinator URL.
      console.warn(
        "[tabview] task_submit action not yet wired",
        block.action,
      );
    }
  };
  return (
    <div>
      <Button variant={TONE_VARIANT[block.tone]} size="sm" onClick={onClick}>
        {block.label}
      </Button>
    </div>
  );
}
