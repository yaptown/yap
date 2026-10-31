import { CartesianGrid, Line, LineChart, XAxis, YAxis } from "recharts";
import type { ReviewTotalView } from "../../../yap-frontend-rs/pkg";
import { ChartContainer, ChartTooltip } from "@/components/ui/chart";
import type { ChartConfig } from "@/components/ui/chart";

const chartConfig = {
  total: { color: "var(--chart-1)" },
} satisfies ChartConfig;

export function ReviewTotalChart({ view }: { view: ReviewTotalView }) {
  const labels = new Map(view.ticks.map((tick) => [tick.day, tick.label]));
  return (
    <ChartContainer config={chartConfig} className="h-[220px] w-full">
      <LineChart data={view.points} margin={{ left: 0, right: 16 }}>
        <CartesianGrid strokeWidth={0.5} vertical={false} />
        <XAxis
          dataKey="day"
          type="number"
          domain={["dataMin", "dataMax"]}
          ticks={view.ticks.map((tick) => tick.day)}
          tickFormatter={(day: number) => labels.get(day) ?? ""}
          className="text-xs"
        />
        <YAxis width={40} allowDecimals={false} className="text-xs" />
        <ChartTooltip
          content={({ active, payload }) =>
            active && payload?.[0] ? (
              <div className="bg-background border rounded-lg px-3 py-2 text-sm shadow-lg">
                {payload[0].payload.label}
              </div>
            ) : null
          }
        />
        {/* The total only moves on study days, so it holds flat between them. */}
        <Line
          type="stepAfter"
          dataKey="total"
          stroke="var(--color-total)"
          strokeWidth={2}
          dot={false}
          activeDot={{ r: 4 }}
          isAnimationActive={false}
        />
      </LineChart>
    </ChartContainer>
  );
}
