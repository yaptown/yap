import { useNavigate } from "react-router-dom";
import type { Deck, StatsScreenView } from "../../../yap-frontend-rs/pkg";
import type { UserInfo } from "@/App";
import { DeckPage } from "@/components/DeckPage";
import { TopPageLayout } from "@/components/TopPageLayout";
import { Stats } from "@/components/stats";
import { useStudyScreenInputs } from "@/hooks/useStudyScreenInputs";

export function StatsPage() {
  return <DeckPage>{(props) => <StatsScreen {...props} />}</DeckPage>;
}

export function StatsScreen({
  deck,
  userInfo,
  view: injectedView,
}: {
  view?: StatsScreenView;
  deck: Deck;
  userInfo: UserInfo | undefined;
}) {
  const inputs = useStudyScreenInputs(!injectedView);
  const view =
    injectedView ?? deck.stats_screen_view(inputs.banned, inputs.timestamp_ms);
  const navigate = useNavigate();
  return (
    <TopPageLayout
      userInfo={userInfo}
      headerProps={{
        backButton: { label: "Home", onBack: () => navigate("/home") },
      }}
    >
      <Stats
        view={view}
        targetLanguage={view.target_language}
        timestampMs={inputs.timestamp_ms}
      />
    </TopPageLayout>
  );
}
