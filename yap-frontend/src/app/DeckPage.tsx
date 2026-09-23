import type { ReactNode } from "react";
import { Navigate, useNavigate, useOutletContext } from "react-router-dom";
import type { AppContextType } from "@/app/context";
import { CourseAudioPrefetch, useCourseDeck } from "@/review/course-study";
import type {
  Deck,
  Language,
  DeckLoadPhase,
} from "../../../yap-frontend-rs/pkg";
import { TopPageLayout } from "../components/TopPageLayout";
import { Card } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Progress } from "../components/ui/progress";
import { ErrorMessage } from "../components/ui/error-message";

export function DeckPage({
  children,
  prefetchAudio = true,
}: {
  prefetchAudio?: boolean;
  children: (
    props: AppContextType & { deck: Deck; targetLanguage: Language },
  ) => ReactNode;
}) {
  const context = useOutletContext<AppContextType>();
  const state = useCourseDeck();
  const navigate = useNavigate();
  if (state.view.phase.type === "NoLanguageSelected")
    return <Navigate to="/" replace />;
  if (state.view.phase.type === "Ready" && state.deck && state.course) {
    return <>
      {prefetchAudio && <CourseAudioPrefetch />}
      {children({
        ...context,
        deck: state.deck,
        targetLanguage: state.course.targetLanguage,
      })}
    </>;
  }
  return (
    <TopPageLayout
      userInfo={context.userInfo}
      headerProps={{
        backButton: { label: "Home", onBack: () => navigate("/home") },
      }}
    >
      <div className="flex flex-1 items-center justify-center">
        <DeckLoadStatus phase={state.view.phase} retry={state.retry} />
      </div>
    </TopPageLayout>
  );
}

// Hosts render the projection; loading/error copy lives only in Rust.
export function DeckLoadStatus({
  phase,
  retry,
}: {
  phase: DeckLoadPhase;
  retry: () => void;
}) {
  if (phase.type === "Error")
    return (
      <Card className="w-full max-w-md p-6 gap-4">
        <h2 className="text-lg font-semibold text-center">{phase.heading}</h2>
        <p className="text-muted-foreground text-center">{phase.body}</p>
        <ErrorMessage message={phase.message} title={phase.title} />
        <Button onClick={retry} variant="outline">
          {phase.retry_label}
        </Button>
      </Card>
    );
  if (phase.type === "Loading")
    return (
      <div className="flex w-full max-w-md flex-col gap-4 text-center">
        <p className="text-muted-foreground">{phase.message}</p>
        <Progress value={phase.percent} />
      </div>
    );
  return null;
}
