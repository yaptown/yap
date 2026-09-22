import type { GoalCardView } from "../../../yap-frontend-rs/pkg";
import { Progress } from "../components/ui/progress";

export function GoalProgress({ goal }: { goal: GoalCardView }) {
  return (
    <>
      <div className="flex items-center justify-between gap-4">
        <h2 className="text-lg font-semibold">{goal.title}</h2>
        <span className="text-sm tabular-nums text-muted-foreground">
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
