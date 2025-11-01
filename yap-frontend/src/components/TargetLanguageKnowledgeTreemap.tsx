import { useMemo } from "react";
import { ChartContainer, ChartTooltip } from "@/components/ui/chart";
import type { ChartConfig } from "@/components/ui/chart";
import { Treemap } from "recharts";
import type { Deck, TargetLanguageKnowledge } from "../../../yap-frontend-rs/pkg";

const chartConfig = {
  known: {
    label: "Known",
    color: "hsl(var(--chart-1))",
  },
  unknown: {
    label: "Unknown",
    color: "hsl(var(--chart-2))",
  },
} satisfies ChartConfig;

interface TargetLanguageKnowledgeTreemapProps {
  deck: Deck;
}

type TreemapDatum = {
  name: string;
  value: number;
  known: boolean;
  children?: TreemapDatum[];
};

type TreemapNodeProps = {
  depth: number;
  x: number;
  y: number;
  width: number;
  height: number;
  name: string;
  payload: TreemapDatum;
};

type TreemapTooltipPayload = {
  payload: TreemapDatum;
};

type TreemapTooltipProps = {
  active?: boolean;
  payload?: TreemapTooltipPayload[];
};

const NODE_TEXT_PADDING = 6;

function buildTreemapData(
  entries: TargetLanguageKnowledge[]
): {
  data: TreemapDatum[];
  totals: {
    known: number;
    unknown: number;
    total: number;
    knownFrequency: number;
    unknownFrequency: number;
  };
} {
  const sorted = [...entries].sort((a, b) => b.frequency - a.frequency);
  const limited = sorted.slice(0, 300);

  const knownChildren: TreemapDatum[] = [];
  const unknownChildren: TreemapDatum[] = [];

  for (const entry of limited) {
    const node = {
      name: entry.word,
      value: entry.frequency,
      known: entry.known,
    } satisfies TreemapDatum;

    if (entry.known) {
      knownChildren.push(node);
    } else {
      unknownChildren.push(node);
    }
  }

  const knownFrequency = knownChildren.reduce((sum, item) => sum + item.value, 0);
  const unknownFrequency = unknownChildren.reduce(
    (sum, item) => sum + item.value,
    0
  );

  const data: TreemapDatum[] = [];

  if (knownChildren.length > 0) {
    data.push({
      name: "Known words",
      value: knownFrequency,
      known: true,
      children: knownChildren,
    });
  }

  if (unknownChildren.length > 0) {
    data.push({
      name: "Unknown words",
      value: unknownFrequency,
      known: false,
      children: unknownChildren,
    });
  }

  return {
    data,
    totals: {
      known: knownChildren.length,
      unknown: unknownChildren.length,
      total: limited.length,
      knownFrequency,
      unknownFrequency,
    },
  };
}

function renderTreemapNode({
  depth,
  x,
  y,
  width,
  height,
  name,
  payload,
}: TreemapNodeProps) {
  if (!width || !height) {
    return null;
  }

  const groupColor = payload.known
    ? "hsl(var(--chart-1))"
    : "hsl(var(--chart-2))";

  if (depth === 1) {
    return (
      <g>
        <rect
          x={x}
          y={y}
          width={width}
          height={height}
          fill={groupColor}
          fillOpacity={0.12}
          stroke="hsl(var(--border))"
          strokeWidth={1}
          rx={6}
        />
        {width > 80 && height > 24 ? (
          <text
            x={x + NODE_TEXT_PADDING}
            y={y + 20}
            fill="hsl(var(--foreground))"
            fontSize={14}
            fontWeight={600}
          >
            {name}
          </text>
        ) : null}
      </g>
    );
  }

  const textVisible = width > 60 && height > 20;

  return (
    <g>
      <rect
        x={x}
        y={y}
        width={width}
        height={height}
        fill={groupColor}
        fillOpacity={0.4}
        stroke="hsl(var(--border))"
        strokeWidth={0.5}
        rx={4}
      />
      {textVisible ? (
        <text
          x={x + NODE_TEXT_PADDING}
          y={y + 18}
          fill="hsl(var(--card-foreground))"
          fontSize={12}
          fontWeight={500}
        >
          {name}
        </text>
      ) : null}
    </g>
  );
}

export function TargetLanguageKnowledgeTreemap({
  deck,
}: TargetLanguageKnowledgeTreemapProps) {
  const { data, totals } = useMemo(() => {
    const entries = deck.get_target_language_knowledge();
    return buildTreemapData(entries);
  }, [deck]);

  if (data.length === 0) {
    return (
      <div className="h-[360px] flex items-center justify-center text-muted-foreground text-sm">
        No target language knowledge data yet.
      </div>
    );
  }

  const totalFrequency = totals.knownFrequency + totals.unknownFrequency;

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap gap-4 text-sm text-muted-foreground">
        <div>
          <span className="font-semibold text-foreground">
            {totals.known.toLocaleString()}
          </span>{" "}
          known words ({((totals.known / totals.total) * 100 || 0).toFixed(1)}%)
        </div>
        <div>
          <span className="font-semibold text-foreground">
            {totals.unknown.toLocaleString()}
          </span>{" "}
          unknown words ({((totals.unknown / totals.total) * 100 || 0).toFixed(1)}%)
        </div>
        <div>
          Weighted by frequency, you know
          {" "}
          <span className="font-semibold text-foreground">
            {totalFrequency === 0
              ? "0.0%"
              : `${((totals.knownFrequency / totalFrequency) * 100).toFixed(1)}%`}
          </span>{" "}
          of the top words we considered.
        </div>
      </div>
      <ChartContainer config={chartConfig} className="h-[360px] w-full">
        <Treemap
          data={data}
          dataKey="value"
          isAnimationActive={false}
          stroke="hsl(var(--border))"
          content={renderTreemapNode}
        >
          <ChartTooltip
            cursor={false}
            wrapperStyle={{ outline: "none" }}
            content={({ active, payload }: TreemapTooltipProps) => {
              if (!active || !payload || payload.length === 0) {
                return null;
              }

              const node = payload[0]?.payload;

              if (!node || node.children) {
                return null;
              }

              return (
                <div className="border border-border bg-background rounded-md p-3 shadow-lg text-xs space-y-1">
                  <p className="font-semibold text-sm text-foreground">{node.name}</p>
                  <p className="text-muted-foreground">
                    {node.known ? "Known" : "Unknown"} ·{" "}
                    {node.value.toLocaleString()} frequency
                  </p>
                </div>
              );
            }}
          />
        </Treemap>
      </ChartContainer>
    </div>
  );
}
