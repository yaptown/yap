import {
  frequency_knowledge_ticks,
  frequency_rank_label,
} from "../../../yap-frontend-rs/pkg";
import { useMemo } from "react";
import { ChartContainer, ChartTooltip } from "@/components/ui/chart";
import type { ChartConfig } from "@/components/ui/chart";
import { Line, LineChart, XAxis, YAxis, CartesianGrid } from "recharts";
import type {
  FrequencyKnowledgePoint,
  Language,
} from "../../../yap-frontend-rs/pkg";
import { TargetLanguageText } from "../components/TargetLanguageText";

interface FrequencyKnowledgeChartProps {
  points: FrequencyKnowledgePoint[];
  targetLanguage: Language;
}

const chartConfig = {
  knowledge: {
    label: "Predicted Knowledge",
    color: "var(--chart-1)",
  },
} satisfies ChartConfig;

export function FrequencyKnowledgeChart({
  points,
  targetLanguage,
}: FrequencyKnowledgeChartProps) {
  const data = useMemo(() => {
    return points.map((point) => ({
      frequency: point.frequency,
      knowledge: point.predicted_knowledge * 100, // Convert to percentage
      label: frequency_rank_label(point.frequency),
      words: point.example_words,
      wordCount: point.word_count,
    }));
  }, [points]);

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
        <CartesianGrid strokeWidth={0.5} className="stroke-muted" />
        <XAxis
          dataKey="frequency"
          type="number"
          scale="log"
          domain={[1, 10000]}
          ticks={frequency_knowledge_ticks().map((tick) => tick.value)}
          tickFormatter={frequency_rank_label}
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
              <div className="bg-background border rounded-lg p-3 shadow-lg max-w-[min(20rem,calc(100vw-2rem))] whitespace-normal break-words">
                <p className="font-semibold">Frequency: {data.label}</p>
                <p className="text-sm">
                  Knowledge: {data.knowledge.toFixed(1)}%
                </p>
                {data.words && (
                  <>
                    <p className="text-sm text-muted-foreground mt-1">
                      Examples ({data.wordCount} words):
                    </p>
                    <p className="text-sm font-medium">
                      <TargetLanguageText language={targetLanguage}>
                        {data.words}
                      </TargetLanguageText>
                    </p>
                  </>
                )}
              </div>
            );
          }}
        />
        <Line
          type="linear"
          dataKey="knowledge"
          stroke="var(--color-knowledge)"
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
