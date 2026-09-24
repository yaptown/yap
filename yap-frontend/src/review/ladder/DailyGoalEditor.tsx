import { useState } from "react";
import type {
  DailyReviewTarget,
  DeckEvent,
  GoalOptionView,
} from "../../../../yap-frontend-rs/pkg";
import { Button } from "../../components/ui/button";
import { cn } from "@/lib/utils";

export function DailyGoalEditor({
  target,
  options,
  addEvent,
  onSave,
}: {
  target: DailyReviewTarget;
  options: GoalOptionView[];
  addEvent: (event: DeckEvent) => void;
  onSave?: () => void;
}) {
  const [pendingTarget, setPendingTarget] = useState(target);
  return (
    <div className="flex flex-col items-center gap-3">
      <div className="flex w-full rounded-lg border overflow-hidden">
        {options.map((option) => (
          <button
            key={option.target}
            type="button"
            aria-pressed={option.target === pendingTarget}
            onClick={() => setPendingTarget(option.target)}
            className={cn(
              "flex-1 px-3 py-2 text-sm font-medium transition-colors border-r last:border-r-0",
              option.target === pendingTarget
                ? "bg-primary text-primary-foreground"
                : "hover:bg-muted",
            )}
          >
            <div>{option.label}</div>
            <div className="text-xs opacity-70">{option.duration_label}</div>
          </button>
        ))}
      </div>
      <Button
        size="sm"
        disabled={pendingTarget === target}
        onClick={() => {
          const goal = options.find(
            (option) => option.target === pendingTarget,
          );
          if (goal) addEvent(goal.event);
          onSave?.();
        }}
      >
        Set goal
      </Button>
    </div>
  );
}
