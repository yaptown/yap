import { useNavigate } from "react-router-dom";
import type { Deck, DueWordsScreenView } from "../../../yap-frontend-rs/pkg";
import type { UserInfo } from "@/app/context";
import { DeckPage } from "@/app/DeckPage";
import { TopPageLayout } from "@/components/TopPageLayout";
import { CardSummaryList } from "@/browse/CardSummaryList";
import { useStudyScreenInputs } from "@/review/useStudyScreenInputs";

export function DueWordsPage() {
  return <DeckPage>{(props) => <DueWordsScreen {...props} />}</DeckPage>;
}

export function DueWordsScreen({
  deck,
  userInfo,
  view: injectedView,
}: {
  view?: DueWordsScreenView;
  deck: Deck;
  userInfo: UserInfo | undefined;
}) {
  const inputs = useStudyScreenInputs(!injectedView);
  const view =
    injectedView ?? deck.due_words_view(inputs.banned, inputs.timestamp_ms);
  const navigate = useNavigate();
  return (
    <TopPageLayout
      userInfo={userInfo}
      headerProps={{
        backButton: { label: "Home", onBack: () => navigate("/home") },
      }}
    >
      <main className="flex flex-col gap-4 py-4">
        <h1 className="text-2xl font-bold">{view.title}</h1>
        <p className="text-muted-foreground">{view.summary_label}</p>
        <CardSummaryList
          cards={view.cards}
          targetLanguage={view.target_language}
          timestampMs={inputs.timestamp_ms}
        />
      </main>
    </TopPageLayout>
  );
}
