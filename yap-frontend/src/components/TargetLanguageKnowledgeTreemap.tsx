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
  stability?: number;
  children?: TreemapDatum[];
};

type TreemapNodeProps = {
  depth?: number;
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  name?: string;
  value?: number;
  known?: boolean;
  stability?: number;
  children?: TreemapDatum[];
  [key: string]: unknown;
};

type TreemapTooltipProps = {
  active?: boolean;
  payload?: Array<{
    payload?: TreemapDatum;
    [key: string]: unknown;
  }>;
  [key: string]: unknown;
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

  const knownChildren: TreemapDatum[] = [];
  const unknownChildren: TreemapDatum[] = [];

  for (const entry of sorted) {
    const node = {
      name: entry.word,
      value: entry.frequency,
      known: entry.known,
      stability: entry.stability,
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
      total: sorted.length,
      knownFrequency,
      unknownFrequency,
    },
  };
}

function renderTreemapNode(props: TreemapNodeProps) {
  const { depth, x, y, width, height, name, known, stability } = props;

  // Only filter out nodes with no dimensions or completely missing data
  if (width === undefined || height === undefined || width <= 0 || height <= 0) {
    return null;
  }

  if (x === undefined || y === undefined) {
    return null;
  }

  // Access known directly from props (recharts spreads the data properties)
  const isKnown = known ?? false;

  // Use CSS variables directly (they're already complete oklch() values)
  const groupColor = isKnown
    ? "var(--chart-1)"
    : "var(--chart-2)";

  // Calculate opacity based on stability for known words
  // Stability typically ranges from 1 to 100+ days
  // We'll use logarithmic scale: log2(stability + 1) normalized to 0.3-0.8 range
  let fillOpacity = 0.5;
  if (depth === 2 && isKnown && stability !== undefined && stability > 0) {
    const normalizedStability = Math.log2(stability + 1) / 10; // log2(1024) = 10
    fillOpacity = 0.3 + Math.min(normalizedStability, 1) * 0.5; // Range: 0.3 to 0.8
  }

  if (depth === 1) {
    return (
      <g>
        <rect
          x={x}
          y={y}
          width={width}
          height={height}
          fill={groupColor}
          fillOpacity={0.2}
          stroke="hsl(var(--border))"
          strokeWidth={1}
        />
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
        fillOpacity={fillOpacity}
        stroke="hsl(var(--border))"
        strokeWidth={0.5}
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
          content={renderTreemapNode as any}
        >
          <ChartTooltip
            cursor={false}
            wrapperStyle={{ outline: "none" }}
            content={((props: TreemapTooltipProps) => {
              const { active, payload } = props;
              if (!active || !payload || payload.length === 0) {
                return null;
              }

              const node = payload[0]?.payload;

              if (!node || node.children) {
                return null;
              }

              return (
                <div className="border border-border bg-background rounded-md p-3 shadow-lg text-xs">
                  <p className="font-semibold text-sm text-foreground">{node.name}</p>
                  <p className="text-muted-foreground">
                    {node.known ? "Known" : "Unknown"}
                  </p>
                </div>
              );
            }) as any}
          />
        </Treemap>
      </ChartContainer>
    </div>
  );
}
