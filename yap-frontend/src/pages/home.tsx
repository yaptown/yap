import { useMemo, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { ChevronRight } from "lucide-react";
import type { Deck, DeckEvent, HomeScreenView } from "../../../yap-frontend-rs/pkg";
import type { UserInfo } from "@/App";
import { DeckPage } from "@/components/DeckPage";
import { TopPageLayout } from "@/components/TopPageLayout";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { GoalProgress } from "@/components/GoalProgress";
import { TargetLanguageText } from "@/components/TargetLanguageText";
import { About } from "@/components/about";
import { NoCardsReady } from "@/components/no-cards-ready";
import { sentenceListToSelection, useSentenceList } from "@/hooks/useSentenceList";
import { useStudyScreenInputs } from "@/hooks/useStudyScreenInputs";
import { useWeapon } from "@/weapon";

export function HomePage() {
  return <DeckPage>{(props) => <HomeScreen {...props} />}</DeckPage>;
}

export function HomeScreen({
  deck,
  userInfo,
  view: injectedView,
}: {
  view?: HomeScreenView;
  deck: Deck;
  userInfo: UserInfo | undefined;
}) {
  const { inputs, refresh } = useStudyScreenInputs(
    deck,
    userInfo !== undefined,
    !injectedView,
  );
  const { sentenceList, setSentenceList } = useSentenceList(
    deck.get_sentence_list(),
  );
  const view = useMemo(
    () =>
      injectedView ??
      deck.home_screen_view({
        ...inputs,
        sentence_list: sentenceListToSelection(sentenceList),
      }),
    [deck, injectedView, inputs, sentenceList],
  );
  const navigate = useNavigate();
  const weapon = useWeapon();
  const addEvent = (event: DeckEvent) => {
    if (injectedView) return;
    weapon.add_deck_event(event);
    // Adding cards from Home means "I want to study these now" — take the user
    // straight to Review so they can learn the cards they just committed to.
    navigate("/learn");
  };
  const undoRestrictions = () => {
    if (injectedView) return;
    localStorage.removeItem("yap-cant-listen-timestamp");
    localStorage.removeItem("yap-cant-speak-timestamp");
    // Recompute inputs now so the restriction notice clears immediately.
    refresh();
  };
  const [query, setQuery] = useState("");
  const upNext = view.up_next;

  return (
    <>
      <TopPageLayout
        userInfo={userInfo}
        headerProps={{
          title: "Yap",
          showSignupNag: injectedView ? false : undefined,
        }}
      >
        <main className="flex flex-col gap-4 py-4" aria-label={view.title}>
          <button
            type="button"
            onClick={() => {
              if (!injectedView) navigate("/select-language");
            }}
            className="text-left rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <Card className="p-5 flex-row items-center justify-between gap-3 hover:bg-muted/50 transition-colors">
              <h2 className="text-lg font-semibold">{view.course_label}</h2>
              <ChevronRight className="h-5 w-5 text-muted-foreground" aria-hidden />
            </Card>
          </button>
          {upNext.idle ? (
            <NoCardsReady
              view={upNext.idle}
              deck={deck}
              addEvent={addEvent}
              undoRestrictions={undoRestrictions}
              setSentenceList={injectedView ? () => {} : setSentenceList}
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
                navigate("/dictionary", { state: { query } });
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
