import { useMemo, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { ChevronRight } from "lucide-react";
import type {
  Deck,
  DeckEvent,
  HomeScreenView,
} from "../../../yap-frontend-rs/pkg";
import type { UserInfo } from "@/app/context";
import { DeckPage } from "@/app/DeckPage";
import { TopPageLayout } from "@/components/TopPageLayout";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { GoalProgress } from "@/browse/GoalProgress";
import { TargetLanguageText } from "@/components/TargetLanguageText";
import { About } from "@/components/about";
import { IdleScreen } from "@/review/IdleScreen";
import {
  sentenceListToSelection,
  useSentenceList,
  type SentenceList,
} from "@/browse/useSentenceList";
import { useCourseStudy } from "@/review/course-study";

export function HomePage() {
  return <DeckPage>{(props) => <HomeScreen {...props} />}</DeckPage>;
}

type HomeProps = {
  deck: Deck;
  userInfo: UserInfo | undefined;
};

export function HomeScreen({
  view,
  ...props
}: HomeProps & { view?: HomeScreenView }) {
  return view ? (
    <HomeContent {...props} view={view} inert />
  ) : (
    <LiveHomeScreen {...props} />
  );
}

function LiveHomeScreen({ deck, userInfo }: HomeProps) {
  const study = useCourseStudy();
  const { sentenceList, setSentenceList, clearSentenceList } = useSentenceList(
    deck.get_sentence_list(),
  );
  const { getHomeView } = study;
  const view = useMemo(
    () => getHomeView(sentenceListToSelection(sentenceList)),
    [getHomeView, sentenceList],
  );
  const navigate = useNavigate();
  const addEvent = (event: DeckEvent) => {
    study.actions.addEvent(event);
    // Adding cards from Home means "I want to study these now".
    navigate("/learn");
  };
  // Switching curriculum is not adding cards: it stays on Home.
  const commitSentenceList = (event: DeckEvent) => {
    study.actions.addEvent(event);
    clearSentenceList();
  };
  return (
    <HomeContent
      deck={deck}
      userInfo={userInfo}
      view={view}
      addEvent={addEvent}
      undoRestrictions={study.actions.undoRestrictions}
      setSentenceList={setSentenceList}
      commitSentenceList={commitSentenceList}
    />
  );
}

function HomeContent({
  deck,
  userInfo,
  view,
  inert = false,
  addEvent = () => {},
  undoRestrictions = () => {},
  setSentenceList = () => {},
  commitSentenceList = () => {},
}: HomeProps & {
  view: HomeScreenView;
  inert?: boolean;
  addEvent?: (event: DeckEvent) => void;
  undoRestrictions?: () => void;
  setSentenceList?: (selection: SentenceList) => void;
  commitSentenceList?: (event: DeckEvent) => void;
}) {
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const upNext = view.up_next;

  return (
    <>
      <TopPageLayout
        userInfo={userInfo}
        headerProps={{
          title: "Yap.Town",
          showSignupNag: inert ? false : undefined,
        }}
      >
        <main className="flex flex-col gap-4 py-4" aria-label={view.title}>
          <button
            type="button"
            onClick={() => {
              if (!inert) navigate("/select-language");
            }}
            className="text-left rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <Card className="p-5 flex-row items-center justify-between gap-3 hover:bg-muted/50 transition-colors">
              <h2 className="text-lg font-semibold">{view.course_label}</h2>
              <ChevronRight
                className="h-5 w-5 text-muted-foreground"
                aria-hidden
              />
            </Card>
          </button>
          {upNext.idle ? (
            <IdleScreen
              view={upNext.idle}
              deck={deck}
              addEvent={addEvent}
              undoRestrictions={undoRestrictions}
              setSentenceList={setSentenceList}
              commitSentenceList={commitSentenceList}
              showEngagementPrompts={false}
            />
          ) : (
            <Card className="relative p-5 gap-4">
              <h2 className="text-sm text-muted-foreground">{upNext.title}</h2>
              <Link
                to="/learn"
                className="after:absolute after:inset-0 after:rounded-xl focus-visible:outline-none focus-visible:after:ring-2 focus-visible:after:ring-ring"
              >
                <p className="text-xl font-semibold">
                  <TargetLanguageText language={view.target_language}>
                    {upNext.headline}
                  </TargetLanguageText>
                </p>
                <p className="text-sm text-muted-foreground">
                  {upNext.kind_label}
                </p>
              </Link>
              <Button asChild className="relative z-10 self-start">
                <Link to="/learn">Review</Link>
              </Button>
            </Card>
          )}
          {!upNext.idle && (
            <Link
              to="/due"
              className="self-center text-sm text-muted-foreground hover:text-foreground"
            >
              {upNext.ready_label} →
            </Link>
          )}
          <Link
            to="/goals"
            className="rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <Card className="p-5 gap-3 hover:bg-muted/50 transition-colors">
              <GoalProgress goal={view.goal} />
            </Card>
          </Link>
          <div className="grid grid-cols-2 gap-4">
            <Card className="p-4 gap-2">
              <h2 className="text-sm text-muted-foreground">
                {view.streak.title}
              </h2>
              <p className="text-xl font-semibold">{view.streak.days_label}</p>
              <p className="text-sm text-muted-foreground">
                {view.streak.today_label}
              </p>
            </Card>
            <Link
              to="/stats"
              className="rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <Card className="h-full p-4 gap-2 hover:bg-muted/50 transition-colors">
                <h2 className="text-sm text-muted-foreground">
                  {view.stats.title}
                </h2>
                <p className="text-xl font-semibold">
                  {view.stats.cards_label}
                </p>
                <p className="text-sm text-muted-foreground">
                  {view.stats.percent_known_label}
                </p>
              </Card>
            </Link>
          </div>
          <Card className="p-5 gap-3">
            <h2 className="text-lg font-semibold">
              <Link to="/dictionary">{view.dictionary.title}</Link>
            </h2>
            <form
              className="flex gap-2"
              onSubmit={(event) => {
                event.preventDefault();
                // Keep searches out of URLs, request logs, and navigation telemetry.
                if (!inert) navigate("/dictionary", { state: { query } });
              }}
            >
              <Input
                type="search"
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder={view.dictionary.search_placeholder}
                aria-label={view.dictionary.search_placeholder}
              />
              <Button
                type="submit"
                variant="outline"
                aria-label={view.dictionary.title}
              >
                →
              </Button>
            </form>
          </Card>
        </main>
      </TopPageLayout>
      <About />
    </>
  );
}
