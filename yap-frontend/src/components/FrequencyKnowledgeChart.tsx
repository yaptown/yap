import { useMemo } from "react";
import { ChartContainer, ChartTooltip } from "@/components/ui/chart";
import type { ChartConfig } from "@/components/ui/chart";
import { Line, LineChart, XAxis, YAxis, CartesianGrid } from "recharts";
import type { Deck } from "../../../yap-frontend-rs/pkg";

interface FrequencyKnowledgeChartProps {
  deck: Deck;
}

const chartConfig = {
  knowledge: {
    label: "Predicted Knowledge",
    color: "hsl(var(--chart-1))",
  },
} satisfies ChartConfig;

export function FrequencyKnowledgeChart({
  deck,
}: FrequencyKnowledgeChartProps) {
  // Compute data only when this component renders
  const data = useMemo(() => {
    const rawData = deck.get_frequency_knowledge_chart_data();
    return rawData.map((point) => ({
      frequency: point.frequency,
      knowledge: point.predicted_knowledge * 100, // Convert to percentage
      label:
        point.frequency >= 1000
          ? `${(point.frequency / 1000).toFixed(1)}k`
          : point.frequency.toString(),
      words: point.example_words,
      wordCount: point.word_count,
    }));
  }, [deck]);

  if (data.length === 0) {
    return (
      <div className="h-[400px] flex items-center justify-center text-muted-foreground">
        <p>No frequency data available</p>
      </div>
    );
  }

  return (
    <ChartContainer config={chartConfig} className="h-[400px] w-full">
      <LineChart data={data}>
        <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
        <XAxis
          dataKey="label"
          angle={-45}
          textAnchor="end"
          height={80}
          className="text-xs"
        />
        <YAxis
          domain={[0, 100]}
          ticks={[0, 25, 50, 75, 100]}
          label={{ value: "Knowledge (%)", angle: -90, position: "insideLeft" }}
          className="text-xs"
        />
        <ChartTooltip
          content={({ active, payload }) => {
            if (!active || !payload || !payload[0]) return null;
            const data = payload[0].payload;
            return (
              <div className="bg-background border rounded-lg p-3 shadow-lg">
                <p className="font-semibold">Frequency: {data.label}</p>
                <p className="text-sm">
                  Knowledge: {data.knowledge.toFixed(1)}%
                </p>
                {data.words && (
                  <>
                    <p className="text-sm text-muted-foreground mt-1">
                      Examples ({data.wordCount} words):
                    </p>
                    <p className="text-sm font-medium">{data.words}</p>
                  </>
                )}
              </div>
            );
          }}
        />
        <Line
          type="monotone"
          dataKey="knowledge"
          strokeWidth={2}
          dot={{
            r: 4,
          }}
          activeDot={{
            r: 6,
          }}
        />
      </LineChart>
    </ChartContainer>
  );
}
