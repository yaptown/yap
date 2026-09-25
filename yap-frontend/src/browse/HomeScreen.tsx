import { useMemo } from "react";
import { Link, useNavigate } from "react-router-dom";
import {
  ArrowRight,
  BookOpen,
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
import { CoursePill } from "@/components/CoursePill";
import { Card } from "@/components/ui/card";
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
          <CoursePill
            flag={view.course_flag}
            label={view.course_label}
            onClick={() => {
              if (!inert) navigate("/select-language");
            }}
          />
          {/* A tagline, not a headline: Up Next's word is the page's one headline. */}
          <h1 className="pt-2 text-2xl font-medium tracking-tight">
            {view.greeting}
          </h1>
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
              <Card className="p-6 gap-0 transition-colors group-hover:bg-muted/40">
                <p className="flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.15em] text-muted-foreground">
                  <KindIcon kind={upNext.kind} />
                  {upNext.eyebrow}
                </p>
                <p className="mt-1.5 text-4xl font-bold break-words">
                  <TargetLanguageText language={view.target_language}>
                    {upNext.headline}
                  </TargetLanguageText>
                </p>
                <span className="mt-5 flex items-center justify-between gap-2 h-12 px-5 rounded-xl bg-primary text-primary-foreground font-medium shadow-xs transition-all group-hover:bg-primary/90">
                  {upNext.action_label}
                  <ArrowRight
                    className="h-4 w-4 transition-transform group-hover:translate-x-0.5"
                    aria-hidden
                  />
                </span>
              </Card>
            </Link>
          )}
          {/* Everything below Up Next is one quiet panel of progress, so the
              next thing to study stays the only card that stands out. */}
          <Card variant="light" className="p-0 gap-0 divide-y divide-border/60 overflow-hidden">
            {view.goal && (
              <Link
                to="/goals"
                className="flex flex-col gap-3 p-5 hover:bg-muted/40 transition-colors focus-visible:outline-none focus-visible:bg-muted/40"
              >
                <GoalProgress goal={view.goal} />
              </Link>
            )}
            <section className="flex flex-col gap-4 p-5">
              <div className="flex items-baseline justify-between gap-4">
                <h2 className="text-lg font-semibold">{view.week.title}</h2>
                <span className="text-sm tabular-nums text-muted-foreground">
                  {view.week.today_label}
                </span>
              </div>
              <WeekProgressStrip week={view.week.days} />
            </section>
            <Link
              to="/stats"
              className="grid grid-cols-2 gap-4 p-5 hover:bg-muted/40 transition-colors focus-visible:outline-none focus-visible:bg-muted/40"
            >
              <HomeStat stat={view.xp} icon={Zap} />
              <HomeStat stat={view.cards} icon={BookOpen} />
            </Link>
          </Card>
          {/* Not a real field: tapping it morphs into the dictionary's search
              bar (same view-transition-name), where typing gets live results. */}
          <button
            type="button"
            aria-label={view.dictionary.title}
            onClick={() => {
              if (!inert)
                navigate("/dictionary", {
                  viewTransition: true,
                  state: { focusSearch: true },
                });
            }}
            className="dictionary-search flex h-11 w-full items-center gap-2 rounded-xl border border-border/60 bg-foreground/5 px-4 text-left text-muted-foreground backdrop-blur-sm hover:bg-foreground/10 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <Search className="h-4 w-4 shrink-0" aria-hidden />
            <span className="truncate">{view.dictionary.search_placeholder}</span>
          </button>
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

function KindIcon({ kind }: { kind: UpNextKind }) {
  const Icon = KIND_ICONS[kind];
  return <Icon className="h-3.5 w-3.5" aria-hidden />;
}

function HomeStat({ stat, icon: Icon }: { stat: HomeStatView; icon: LucideIcon }) {
  return (
    <div className="flex items-start gap-3 min-w-0">
      <Icon className="h-5 w-5 mt-1.5 shrink-0 text-muted-foreground" aria-hidden />
      <div className="flex flex-col min-w-0">
        <p className="text-2xl font-bold tabular-nums">{stat.value}</p>
        <p className="text-sm text-muted-foreground">{stat.caption}</p>
        {stat.note && (
          <p className="mt-1 text-xs text-muted-foreground">{stat.note}</p>
        )}
      </div>
    </div>
  );
}
