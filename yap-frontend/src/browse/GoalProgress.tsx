import type { GoalCardView } from "../../../yap-frontend-rs/pkg";
import { Progress } from "../components/ui/progress";

export function GoalProgress({ goal }: { goal: GoalCardView }) {
  return (
    <>
      <div className="flex items-center gap-3">
        <h2 className="text-lg font-semibold">{goal.name}</h2>
        <span className="rounded-full border border-border px-2.5 py-0.5 text-xs text-muted-foreground whitespace-nowrap">
          {goal.level_label}
        </span>
        <span className="ml-auto text-lg font-semibold tabular-nums">
          {goal.percent_label}
        </span>
      </div>
      {/* Progress takes 0–100, matching GoalCardView (not Stats' 0–1). */}
      <Progress
        value={goal.percent}
        aria-label={goal.title}
        aria-valuenow={goal.percent}
        aria-valuetext={goal.percent_label}
      />
      <p className="text-sm text-muted-foreground">{goal.subtitle}</p>
    </>
  );
}
