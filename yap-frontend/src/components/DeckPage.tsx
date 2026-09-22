import type { ReactNode } from "react";
import { Navigate, useNavigate, useOutletContext } from "react-router-dom";
import type { AppContextType } from "@/App";
import { useCourseDeck } from "@/contexts/course-study";
import type { Deck, Language } from "../../../yap-frontend-rs/pkg";
import { TopPageLayout } from "./TopPageLayout";
import { Card } from "./ui/card";
import { Button } from "./ui/button";
import { Progress } from "./ui/progress";
import { ErrorMessage } from "./ui/error-message";

export function DeckPage({
  children,
}: {
  children: (
    props: AppContextType & { deck: Deck; targetLanguage: Language },
  ) => ReactNode;
}) {
  const context = useOutletContext<AppContextType>();
  const state = useCourseDeck();
  const navigate = useNavigate();
  if (state?.type === "noLanguageSelected") return <Navigate to="/" replace />;
  if (state?.type === "deck" && state.deck) {
    return children({
      ...context,
      deck: state.deck,
      targetLanguage: state.targetLanguage,
    });
  }
  return (
    <TopPageLayout
      userInfo={context.userInfo}
      headerProps={{
        backButton: { label: "Home", onBack: () => navigate("/home") },
      }}
    >
      <div className="flex flex-1 items-center justify-center">
        {state?.type === "error" ? (
          <Card className="w-full p-4">
            <ErrorMessage
              message={state.message}
              title="Failed to load language data"
            />
            <Button onClick={state.retry} variant="outline">
              Try Again
            </Button>
          </Card>
        ) : (
          <div className="flex w-full max-w-md flex-col gap-4 text-center">
            <p className="text-muted-foreground">
              {state?.type === "loading" ? state.message : "Loading..."}
            </p>
            {state?.type === "loading" && <Progress value={state.progress} />}
          </div>
        )}
      </div>
    </TopPageLayout>
  );
}
