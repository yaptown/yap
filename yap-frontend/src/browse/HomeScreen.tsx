import { useMemo, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import {
  ArrowRight,
  BookOpen,
  ChevronDown,
  Headphones,
  Keyboard,
  Languages,
  MessageCircle,
  Mic,
  Search,
  Sparkles,
  Zap,
  type LucideIcon,
} from "lucide-react";
import type {
  Deck,
  DeckEvent,
  HomeScreenView,
  HomeStatView,
  UpNextKind,
} from "../../../yap-frontend-rs/pkg";
import type { UserInfo } from "@/app/context";
import { DeckPage } from "@/app/DeckPage";
import { TopPageLayout } from "@/components/TopPageLayout";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { GoalProgress } from "@/browse/GoalProgress";
import { WeekProgressStrip } from "@/review/WeekProgressStrip";
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
            className="self-start flex items-center gap-2 rounded-xl border border-border/60 px-3 py-2 backdrop-blur-sm hover:bg-muted/50 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <span className="text-lg leading-none" aria-hidden>
              {view.course_flag}
            </span>
            <span className="font-medium">{view.course_label}</span>
            <ChevronDown className="h-4 w-4 text-muted-foreground" aria-hidden />
          </button>
          <div className="flex flex-col gap-1 pt-2 pb-1">
            <h1 className="text-3xl sm:text-4xl font-bold tracking-tight">
              {view.greeting}
            </h1>
            {view.greeting_detail && (
              <p className="text-lg text-muted-foreground">
                {view.greeting_detail}
              </p>
            )}
          </div>
          {upNext.idle ? (
            <IdleScreen
              view={upNext.idle}
              deck={deck}
              addEvent={addEvent}
              undoRestrictions={undoRestrictions}
              setSentenceList={setSentenceList}
              commitSentenceList={commitSentenceList}
              showEngagementPrompts={false}
              showWeek={false}
            />
          ) : (
            // The whole card starts the review, not just its button.
            <Link
              to="/learn"
              className="group rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <Card className="p-6 flex-row items-stretch gap-4 transition-colors group-hover:bg-muted/40">
                <div className="flex flex-col gap-1 min-w-0 flex-1">
                  <p className="text-xs font-semibold uppercase tracking-[0.2em] text-muted-foreground">
                    {upNext.title}
                  </p>
                  <p className="text-3xl sm:text-4xl font-bold break-words">
                    <TargetLanguageText language={view.target_language}>
                      {upNext.headline}
                    </TargetLanguageText>
                  </p>
                  <p className="text-muted-foreground">{upNext.kind_label}</p>
                  <span className="mt-4 self-start inline-flex items-center gap-2 h-11 px-5 rounded-lg bg-primary/85 text-primary-foreground font-medium shadow-xs transition-all group-hover:bg-primary/95 group-hover:gap-3">
                    {upNext.action_label}
                    <ArrowRight className="h-4 w-4" aria-hidden />
                  </span>
                </div>
                <div className="flex flex-col items-end justify-between gap-3 shrink-0">
                  <CardStack kind={upNext.kind} />
                  <p className="text-sm text-muted-foreground whitespace-nowrap">
                    {upNext.ready_label}
                  </p>
                </div>
              </Card>
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
          <Card className="p-5 gap-4">
            <div className="flex items-baseline justify-between gap-4">
              <h2 className="text-lg font-semibold">{view.week.title}</h2>
              <span className="text-sm tabular-nums text-muted-foreground">
                {view.week.today_label}
              </span>
            </div>
            <WeekProgressStrip week={view.week.days} />
          </Card>
          <div className="grid grid-cols-2 gap-4">
            <StatTile stat={view.xp} icon={Zap} />
            <StatTile stat={view.cards} icon={BookOpen} />
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
              <div className="relative flex-1">
                <Search
                  className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground pointer-events-none"
                  aria-hidden
                />
                <Input
                  type="search"
                  value={query}
                  onChange={(event) => setQuery(event.target.value)}
                  placeholder={view.dictionary.search_placeholder}
                  aria-label={view.dictionary.search_placeholder}
                  className="pl-9"
                />
              </div>
              <Button
                type="submit"
                variant="outline"
                size="icon"
                aria-label={view.dictionary.title}
              >
                <ArrowRight className="h-4 w-4" />
              </Button>
            </form>
          </Card>
        </main>
      </TopPageLayout>
      <About />
    </>
  );
}

const KIND_ICONS: Record<UpNextKind, LucideIcon> = {
  Flashcard: MessageCircle,
  Listening: Headphones,
  Pronunciation: Mic,
  Translation: Languages,
  Transcription: Keyboard,
  Other: Sparkles,
};

/** Two tilted cards with the challenge's icon: Up Next's illustration. */
function CardStack({ kind }: { kind: UpNextKind }) {
  const Icon = KIND_ICONS[kind];
  return (
    <div className="relative w-24 h-28 sm:w-28 sm:h-32" aria-hidden>
      <div className="absolute inset-0 translate-x-3 rotate-[10deg] rounded-2xl border border-border/60 bg-card/30 backdrop-blur-sm" />
      <div className="absolute inset-0 -rotate-6 rounded-2xl border border-border/60 bg-card/60 backdrop-blur-md shadow-sm flex items-center justify-center transition-transform group-hover:-rotate-3">
        <Icon className="h-9 w-9 text-muted-foreground" strokeWidth={1.5} />
      </div>
    </div>
  );
}

function StatTile({ stat, icon: Icon }: { stat: HomeStatView; icon: LucideIcon }) {
  return (
    <Link
      to="/stats"
      className="rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      <Card className="h-full p-5 flex-row items-start gap-3 hover:bg-muted/50 transition-colors">
        <Icon className="h-6 w-6 mt-1 shrink-0 text-muted-foreground" aria-hidden />
        <div className="flex flex-col min-w-0">
          <p className="text-2xl font-bold tabular-nums">{stat.value}</p>
          <p className="text-sm text-muted-foreground">{stat.caption}</p>
          {stat.note && (
            <p className="mt-1 text-xs text-muted-foreground">{stat.note}</p>
          )}
        </div>
      </Card>
    </Link>
  );
}
