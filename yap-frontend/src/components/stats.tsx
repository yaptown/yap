import { lazy, Suspense } from "react";
import { Link } from "react-router-dom";
import type { Language, StatsScreenView } from "../../../yap-frontend-rs/pkg";
import { Card } from "./ui/card";
import { Leeches } from "./Leeches";

const FrequencyKnowledgeChart = lazy(() =>
  import("./FrequencyKnowledgeChart").then((module) => ({
    default: module.FrequencyKnowledgeChart,
  })),
);

export function Stats({
  view,
  targetLanguage,
  timestampMs,
}: {
  view: StatsScreenView;
  targetLanguage: Language;
  timestampMs: number;
}) {
  return (
    <main className="flex flex-col gap-6 py-4">
      <h1 className="text-2xl font-bold">{view.title}</h1>
      <div className="grid grid-cols-2 gap-4">
        <Card className="p-4 gap-2">
          <p className="text-xl font-semibold">{view.xp_label}</p>
        </Card>
        <Card className="p-4 gap-2">
          <p className="text-xl font-semibold">{view.total_reviews_label}</p>
        </Card>
        <Card className="p-4 gap-2">
          <h2 className="text-sm text-muted-foreground">{view.streak.title}</h2>
          <p className="text-xl font-semibold">{view.streak.days_label}</p>
          <p className="text-sm text-muted-foreground">
            {view.streak.today_label}
          </p>
        </Card>
        <Card className="p-4 gap-2">
          <p className="text-xl font-semibold">{view.percent_known_label}</p>
        </Card>
      </div>
      <Link
        to="/due"
        className="rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      >
        <Card className="p-4 gap-2 hover:bg-muted/50 transition-colors">
          <h2 className="text-lg font-semibold">{view.due.title} →</h2>
          <p className="text-sm text-muted-foreground">{view.due.label}</p>
        </Card>
      </Link>
      <Card className="p-4 gap-4">
        <h2 className="text-lg font-semibold">
          {view.frequency_knowledge_chart_title}
        </h2>
        <Suspense
          fallback={
            <div className="h-[400px] flex items-center justify-center text-muted-foreground">
              Loading chart...
            </div>
          }
        >
          <FrequencyKnowledgeChart
            points={view.frequency_knowledge_chart_data}
            targetLanguage={targetLanguage}
          />
        </Suspense>
      </Card>
      <Leeches
        leeches={view.leeches}
        label={view.leeches_label}
        targetLanguage={targetLanguage}
        timestampMs={timestampMs}
      />
    </main>
  );
}
