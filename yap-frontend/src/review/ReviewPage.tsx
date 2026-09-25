import {
  CourseAudioPrefetch,
  useCourseDeck,
  useCourseStudy,
} from "@/review/course-study";
import { ReviewScreen } from "@/review/ReviewScreen";
import { useState, useEffect } from "react";
import { useZeno } from "@/hooks/useZeno";
import { useNavigate, useOutletContext } from "react-router-dom";
import {
  Deck,
  type DeckEvent,
  type Language,
  report_issue_copy,
} from "../../../yap-frontend-rs/pkg";
import { Button } from "@/components/ui/button.tsx";
import { Progress } from "@/components/ui/progress.tsx";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { ReportIssueModal } from "@/review/challenges/ReportIssueModal";
import {
  useSentenceList,
  sentenceListToSelection,
} from "@/browse/useSentenceList";
import { TopPageLayout } from "@/components/TopPageLayout";
import { DeckLoadStatus } from "@/app/DeckPage";
import type { AppContextType, UserInfo } from "@/app/context";

function LoadingProgress({
  message,
  progress,
}: {
  message: string;
  progress: number;
}) {
  const smoothProgress = useZeno(progress);
  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="w-full max-w-md space-y-4">
        <p className="text-muted-foreground text-center">{message}</p>
        <Progress value={smoothProgress} className="w-full" disableTransition />
      </div>
    </div>
  );
}

export function ReviewPage() {
  const { userInfo, accessToken } = useOutletContext<AppContextType>();
  const state = useCourseDeck();
  const navigate = useNavigate();
  const [lastAutoPlayReviewCount, setLastAutoPlayReviewCount] = useState<
    bigint | null
  >(null);
  const phase = state.view.phase;
  useEffect(() => {
    if (phase.type === "NoLanguageSelected") navigate("/", { replace: true });
  }, [phase.type, navigate]);
  if (phase.type === "Ready" && state.deck && state.course) {
    const totalReviewsCompleted = state.deck.get_total_reviews();
    return (
      <div className="flex flex-col gap-6">
        <CourseAudioPrefetch />
        {state.view.pack_banner && (
          <div
            className="flex items-center justify-between gap-4 p-4 text-sm text-muted-foreground"
            role="status"
          >
            <p>{state.view.pack_banner.message}</p>
            <Button variant="outline" onClick={state.retry}>
              {state.view.pack_banner.retry_label}
            </Button>
          </div>
        )}
        <Review
          userInfo={userInfo}
          accessToken={accessToken}
          deck={state.deck}
          targetLanguage={state.course.targetLanguage}
          autoplayed={lastAutoPlayReviewCount === totalReviewsCompleted}
          setAutoplayed={() =>
            setLastAutoPlayReviewCount(totalReviewsCompleted)
          }
        />
      </div>
    );
  }
  return (
    <TopPageLayout
      userInfo={userInfo}
      headerProps={{
        title: "Review",
        backButton: { label: "Home", onBack: () => navigate("/home") },
        showSignupNag: false,
      }}
    >
      {phase.type === "Loading" ? (
        <LoadingProgress message={phase.message} progress={phase.percent} />
      ) : (
        <div className="flex-1 flex items-center justify-center p-4">
          <DeckLoadStatus phase={phase} retry={state.retry} />
        </div>
      )}
    </TopPageLayout>
  );
}

interface ReviewProps {
  userInfo: UserInfo | undefined;
  accessToken: string | undefined;
  deck: Deck;
  targetLanguage: Language;
  autoplayed: boolean;
  setAutoplayed: () => void;
}

function Review({
  userInfo,
  accessToken,
  deck,
  targetLanguage,
  autoplayed,
  setAutoplayed,
}: ReviewProps) {
  const navigate = useNavigate();
  const { sentenceList, setSentenceList, clearSentenceList } = useSentenceList(
    deck.get_sentence_list(),
  );
  const study = useCourseStudy();
  const view = study.getReviewView(sentenceListToSelection(sentenceList));
  const commitSentenceList = (event: DeckEvent) => {
    study.actions.addEvent(event);
    clearSentenceList();
  };
  const currentChallenge = study.currentChallenge;
  const [showReportModal, setShowReportModal] = useState(false);

  return (
    <TopPageLayout
      userInfo={userInfo}
      headerProps={{
        title: "Review",
        showSignupNag: view.show_account_prompt,
        dailyGoalPercent: view.progress * 100,
        backButton: { label: "Home", onBack: () => navigate("/home") },
      }}
    >
      <ReviewScreen
        view={view}
        host={{
          deck,
          accessToken,
          autoplayed,
          setAutoplayed,
          menuExtras: (
            <DropdownMenuItem onClick={() => setShowReportModal(true)}>
              {report_issue_copy().menu_label}
            </DropdownMenuItem>
          ),
        }}
        actions={{ ...study.actions, setSentenceList, commitSentenceList }}
      />

      {currentChallenge?.type === "FlashCardReview" && (
        <ReportIssueModal
          subject={{ Flashcard: currentChallenge.flashcard.content }}
          open={showReportModal}
          onOpenChange={setShowReportModal}
          targetLanguage={targetLanguage}
        />
      )}
    </TopPageLayout>
  );
}
