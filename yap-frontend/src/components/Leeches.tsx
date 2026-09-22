import type { CardSummary, Language } from "../../../yap-frontend-rs/pkg";
import { CardSummaryList } from "./CardSummaryList";

export function Leeches({
  leeches,
  label,
  targetLanguage,
  timestampMs,
}: {
  leeches: CardSummary[];
  label: string;
  targetLanguage: Language;
  timestampMs: number;
}) {
  return (
    <section className="flex flex-col gap-4">
      <h2 className="text-xl font-semibold">{label}</h2>
      <p className="text-sm text-muted-foreground">
        Leeches are cards you're really struggling with. The hardest few cards
        can take disproportionate time, so it's more efficient to set them aside
        for a while.
      </p>
      {leeches.length > 0 && (
        <CardSummaryList
          cards={leeches}
          targetLanguage={targetLanguage}
          timestampMs={timestampMs}
        />
      )}
    </section>
  );
}
