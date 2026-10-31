import { useLayoutEffect, useRef } from "react";
import type { ActivityView } from "../../../yap-frontend-rs/pkg";
import { Card } from "../components/ui/card";

// Level 0 is a faint neutral; 1–4 deepen the chart color, like GitHub's greens.
const LEVEL_COLORS = [
  "color-mix(in oklab, var(--muted-foreground) 15%, transparent)",
  "color-mix(in oklab, var(--chart-1) 45%, transparent)",
  "color-mix(in oklab, var(--chart-1) 65%, transparent)",
  "color-mix(in oklab, var(--chart-1) 85%, transparent)",
  "var(--chart-1)",
];

// Month labels take the first row; days fill rows 2–8.
const ROWS = "grid-rows-[auto_repeat(7,minmax(0,1fr))]";

export function ActivityCalendar({ view }: { view: ActivityView }) {
  const scroller = useRef<HTMLDivElement>(null);
  // When the year doesn't fit, start at the most recent weeks.
  useLayoutEffect(() => {
    const element = scroller.current;
    if (element) element.scrollLeft = element.scrollWidth;
  }, []);
  return (
    <Card className="p-4 gap-3 min-w-0">
      <div className="flex flex-col gap-1">
        <h2 className="text-lg font-semibold">{view.title}</h2>
        <p className="text-sm text-muted-foreground">{view.summary}</p>
      </div>
      <div className="flex gap-1.5 text-[10px] leading-none text-muted-foreground">
        {/* Zero-height labels add no height of their own, so these rows stretch
            to match the squares' rows exactly. */}
        <div aria-hidden className={`grid gap-[2px] ${ROWS}`}>
          <span className="pb-1">&nbsp;</span>
          {view.weekdays.map((label, day) => (
            <span key={day} className="flex h-0 items-center self-center">
              {label}
            </span>
          ))}
        </div>
        <div ref={scroller} className="min-w-0 flex-1 overflow-x-auto">
          <div
            className={`grid gap-[2px] ${ROWS}`}
            style={{
              gridTemplateColumns: `repeat(${view.weeks.length}, minmax(8px, 1fr))`,
            }}
          >
            {view.weeks.map((week, column) => [
              week.month_label && (
                <span
                  key={`month-${column}`}
                  aria-hidden
                  className="w-0 whitespace-nowrap pb-1"
                  style={{ gridColumn: column + 1, gridRow: 1 }}
                >
                  {week.month_label}
                </span>
              ),
              ...week.days.map(
                (day, row) =>
                  day && (
                    <div
                      key={day.date}
                      role="img"
                      title={day.label}
                      aria-label={day.label}
                      className="aspect-square rounded-[2px]"
                      style={{
                        gridColumn: column + 1,
                        gridRow: row + 2,
                        background: LEVEL_COLORS[day.level],
                      }}
                    />
                  ),
              ),
            ])}
          </div>
        </div>
      </div>
      <div
        aria-hidden
        className="flex items-center justify-end gap-[3px] text-xs text-muted-foreground"
      >
        <span className="pr-1">{view.less_label}</span>
        {LEVEL_COLORS.map((color) => (
          <span
            key={color}
            className="size-[10px] rounded-[2px]"
            style={{ background: color }}
          />
        ))}
        <span className="pl-1">{view.more_label}</span>
      </div>
    </Card>
  );
}
