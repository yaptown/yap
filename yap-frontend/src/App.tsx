import { HomePage } from "@/pages/home";
import { GoalsPage } from "@/pages/goals";
import { StatsPage } from "@/pages/stats";
import { DueWordsPage } from "@/pages/due";
import {
  CourseRoutes,
  useCourseDeck,
  useCourseStudy,
} from "@/contexts/course-study";
import { useDeckSelection } from "@/hooks/useDeck";
import { ReviewScreen } from "@/components/ReviewScreen";
import * as Sentry from "@sentry/react";
import { useState, useEffect } from "react";
import { useZeno } from "@/hooks/useZeno";
import { AuthDialogProvider } from "@/components/auth-dialog-provider";
import {
  createBrowserRouter,
  RouterProvider,
  Outlet,
  useNavigate,
  useOutletContext,
  ScrollRestoration,
} from "react-router-dom";
import { Deck, type DeckEvent, type Language } from "../../yap-frontend-rs/pkg";
import { Button } from "@/components/ui/button.tsx";
import { Progress } from "@/components/ui/progress.tsx";
import { Card } from "@/components/ui/card";
import { ThemeProvider } from "@/components/theme-provider";
import { RouteErrorScreen } from "@/components/route-error-screen";
import { supabase } from "@/lib/supabase";
import type { Session as SupabaseSession } from "@supabase/supabase-js";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { ReportIssueModal } from "@/components/challenges/ReportIssueModal";
import { ResetPassword } from "@/pages/reset-password";
import { ConfirmEmail } from "@/pages/confirm-email";
import { AcceptInvite } from "@/pages/accept-invite";
import { ForgotPassword } from "@/pages/forgot-password";
import { Connect } from "./pages/connect";
import { UserProfilePage } from "@/pages/user-profile";
import { AboutPage } from "@/pages/about";
import { PrivacyPage } from "@/pages/privacy";
import { TermsPage } from "@/pages/terms";
import { McpDocsPage } from "@/pages/mcp-docs";
import { LandingPage } from "@/pages/landing";
import { NotFoundPage } from "@/pages/not-found";
import { registerSW } from "virtual:pwa-register";
import {
  useSentenceList,
  sentenceListToSelection,
} from "@/hooks/useSentenceList";

import type { Dispatch, SetStateAction } from "react";
import type { RegisterSWOptions } from "vite-plugin-pwa/types";
declare module "virtual:pwa-register/react" {
  export function useRegisterSW(options?: RegisterSWOptions): {
    needRefresh: [boolean, Dispatch<SetStateAction<boolean>>];
    offlineReady: [boolean, Dispatch<SetStateAction<boolean>>];
    updateServiceWorker: (reloadPage?: boolean) => Promise<void>;
  };
}
import { useRegisterSW } from "virtual:pwa-register/react";
import { LanguageSelector } from "./components/LanguageSelector";
import {
  WeaponProvider,
  useWeapon,
  useWeaponState,
  useWeaponSupport,
  type WeaponToken,
} from "./weapon";
import { Toaster } from "sonner";
import { BrowserNotSupported } from "@/components/browser-not-supported";
import { Dictionary } from "@/components/Dictionary";
import { TopPageLayout } from "@/components/TopPageLayout";
import { match } from "ts-pattern";
import { DeckLoadStatus } from "@/components/DeckPage";
import { BackgroundShader } from "@/components/BackgroundShader";

// Essential user info to persist for offline functionality
export interface UserInfo {
  id: string;
  email: string;
  displayName: string | null | undefined;
}

export type AppContextType = {
  userInfo: UserInfo | undefined;
  accessToken: string | undefined;
};

function AppMain() {
  const updateIntervalMS = 60 * 5 * 1000; // every 5 minutes
  useEffect(() => {
    registerSW({ immediate: true });
  }, []);

  useRegisterSW({
    onRegistered(r) {
      if (r) {
        const update = () => {
          r.update().catch((e) => {
            // Skip network-level fetch failures (TypeError) and InvalidStateError
            // — these are transient errors that aren't actionable bugs.
            if (
              navigator.onLine &&
              e?.name !== "InvalidStateError" &&
              e?.name !== "TypeError"
            ) {
              Sentry.captureException(e, { tags: { "sw.online": true } });
            }
          });
        };
        update();
        setInterval(update, updateIntervalMS);
      }
    },
  });

  return <AppCheckBrowserSupport />;
}

function AppCheckBrowserSupport() {
  const token = useWeaponSupport();
  const supported = token.browserSupported;
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    if (supported !== null) return;

    const start = Date.now();
    const timer = setInterval(() => {
      const diff = Date.now() - start;
      setProgress(Math.max(1, Math.min(diff / 30, 100)));
    }, 480);

    return () => clearInterval(timer);
  }, [supported]);

  const smoothProgress = useZeno(progress);

  if (supported === null) {
    return (
      <div className="min-h-screen flex flex-col items-center justify-center space-y-4">
        <p className="text-muted-foreground animate-fade-in-delay-2">
          Checking device compatibility...
        </p>
        <Progress
          value={smoothProgress}
          className="w-64 animate-fade-in-delay-2"
          disableTransition
        />
      </div>
    );
  } else if (supported === false) {
    return <BrowserNotSupported />;
  } else {
    return <AppCheckLoggedIn weaponToken={{ browserSupported: supported }} />;
  }
}

function AppCheckLoggedIn({ weaponToken }: { weaponToken: WeaponToken }) {
  void weaponToken;
  const [session, setSession] = useState<SupabaseSession | null>(null);
  const [signedOut, setSignedOut] = useState(false);
  const [displayName, setDisplayName] = useState<string | null | undefined>(
    undefined,
  );

  useEffect(() => {
    supabase.auth
      .getSession()
      .then(({ data: { session } }) => {
        setSession(session);
      })
      .catch(() => {
        setSession(null);
      });

    const { data: authListener } = supabase.auth.onAuthStateChange(
      (event, session) => {
        setSession(session);
        if (event === "SIGNED_IN") {
          Sentry.setUser({ id: session?.user.id, email: session?.user.email });
          localStorage.setItem(
            "yap-user-info",
            JSON.stringify({
              id: session?.user.id,
              email: session?.user.email,
              displayName: undefined, // Will be fetched from profiles table
            }),
          );
          setSignedOut(false);
        } else if (event === "SIGNED_OUT") {
          Sentry.setUser(null);
          localStorage.removeItem("yap-user-info");

          if (window.OneSignal) {
            window.OneSignal.logout();
          }

          setSession(null);
          setDisplayName(undefined);
          setSignedOut(true);
        }
      },
    );

    return () => {
      authListener.subscription.unsubscribe();
    };
  }, []);

  // Fetch display name from Supabase when logged in
  useEffect(() => {
    if (!session?.user.id) return;

    // Fetch initial display name
    const fetchDisplayName = async () => {
      const { data, error } = await supabase
        .from("profiles")
        .select("display_name")
        .eq("id", session.user.id)
        .single();

      if (!error && data) {
        setDisplayName(data.display_name);
      }
    };

    fetchDisplayName();

    const channel = supabase
      .channel(`profile_${session.user.id}`)
      .on(
        "postgres_changes",
        {
          event: "UPDATE",
          schema: "public",
          table: "profiles",
          filter: `id=eq.${session.user.id}`,
        },
        (payload) => {
          if (payload.new && "display_name" in payload.new) {
            setDisplayName(payload.new.display_name as string | null);
          }
        },
      )
      .subscribe();

    return () => {
      supabase.removeChannel(channel);
    };
  }, [session?.user.id]);

  useEffect(() => {
    if (session?.user.id && session?.user.email && displayName !== undefined) {
      localStorage.setItem(
        "yap-user-info",
        JSON.stringify({
          id: session.user.id,
          email: session.user.email,
          displayName: displayName,
        }),
      );
    }
  }, [session?.user.id, session?.user.email, displayName]);

  let userInfo: UserInfo | undefined;

  if (session) {
    userInfo = {
      id: session.user.id,
      email: session.user.email!,
      displayName: displayName,
    };
  } else if (!signedOut) {
    const cachedUserInfo = localStorage.getItem("yap-user-info");
    if (cachedUserInfo) {
      try {
        userInfo = JSON.parse(cachedUserInfo);
      } catch {
        localStorage.removeItem("yap-user-info");
      }
    }
  }

  const accessToken = session?.access_token;

  return (
    <WeaponProvider userId={userInfo?.id} accessToken={accessToken}>
      <AppTestWeapon userInfo={userInfo} accessToken={accessToken} />
    </WeaponProvider>
  );
}

function AppTestWeapon({ userInfo, accessToken }: AppContextType) {
  const weaponState = useWeaponState();

  if (weaponState.type === "loading") {
    return (
      <div>
        <div className="min-h-screen flex items-center justify-center">
          <p className="text-muted-foreground animate-fade-in-delayed">
            Loading...
          </p>
        </div>
      </div>
    );
  } else if (weaponState.type === "error") {
    return (
      <div>
        <div className="min-h-screen bg-background flex items-center justify-center p-4">
          <Card className="max-w-md w-full p-6 text-center gap-0">
            <div className="w-12 h-12 bg-negative-surface rounded-full flex items-center justify-center mx-auto mb-4">
              <span className="text-negative-foreground text-xl">⚠</span>
            </div>
            <h2 className="text-lg font-semibold mb-2">
              Failed to Initialize Deck
            </h2>
            <p className="text-muted-foreground mb-4">{weaponState.message}</p>
            <Button onClick={() => window.location.reload()} variant="outline">
              Try Again
            </Button>
          </Card>
        </div>
      </div>
    );
  } else if (weaponState.type === "ready") {
    return <AppContent userInfo={userInfo} accessToken={accessToken} />;
  }
}

function AppContent({ userInfo, accessToken }: AppContextType) {
  return (
    <div className="px-2 overflow-x-clip">
      <div className="min-h-screen text-foreground">
        <div className="max-w-2xl mx-auto">
          <AuthDialogProvider>
            <Outlet context={{ userInfo, accessToken }} />
          </AuthDialogProvider>
          <div className="p-2"></div>
        </div>
      </div>
    </div>
  );
}

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

function ReviewPage() {
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

function DictionaryPage() {
  const { userInfo, accessToken } = useOutletContext<AppContextType>();
  const deck = useCourseDeck();
  const weapon = useWeapon();
  const navigate = useNavigate();

  const phase = deck.view.phase;
  useEffect(() => {
    if (phase.type === "NoLanguageSelected") navigate("/", { replace: true });
  }, [phase.type, navigate]);
  if (phase.type === "NoLanguageSelected") return null;
  if (phase.type !== "Ready" || !deck.deck || !deck.course) {
    return (
      <TopPageLayout
        userInfo={userInfo}
        headerProps={{
          backButton: { label: "Home", onBack: () => navigate("/home") },
        }}
      >
        <div className="flex-1 flex items-center justify-center p-4">
          <DeckLoadStatus phase={phase} retry={deck.retry} />
        </div>
      </TopPageLayout>
    );
  }

  return (
    <TopPageLayout
      userInfo={userInfo}
      headerProps={{
        backButton: { label: "Home", onBack: () => navigate("/home") },
      }}
    >
      <Dictionary
        deck={deck.deck}
        weapon={weapon}
        targetLanguage={deck.course.targetLanguage}
        nativeLanguage={deck.course.nativeLanguage}
        accessToken={accessToken}
      />
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
        showSignupNag: true,
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
              Report an Issue
            </DropdownMenuItem>
          ),
        }}
        actions={{ ...study.actions, setSentenceList, commitSentenceList }}
      />

      <ReportIssueModal
        context={
          currentChallenge?.type === "FlashCardReview"
            ? JSON.stringify(currentChallenge.flashcard.content)
            : ""
        }
        open={showReportModal}
        onOpenChange={setShowReportModal}
        targetLanguage={targetLanguage}
      />
    </TopPageLayout>
  );
}

function AppShell() {
  return (
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <BackgroundShader>
        <ScrollRestoration />
        <Outlet />
        <Toaster />
      </BackgroundShader>
    </ThemeProvider>
  );
}

const router = createBrowserRouter([
  {
    element: <AppShell />,
    errorElement: <RouteErrorScreen />,
    children: [
      { path: "/reset-password", element: <ResetPassword /> },
      { path: "/confirm-email", element: <ConfirmEmail /> },
      { path: "/accept-invite", element: <AcceptInvite /> },
      { path: "/forgot-password", element: <ForgotPassword /> },
      { path: "/connect", element: <Connect /> },
      { path: "/about", element: <AboutPage /> },
      { path: "/privacy", element: <PrivacyPage /> },
      { path: "/terms", element: <TermsPage /> },
      { path: "/mcp", element: <McpDocsPage /> },
      {
        path: "/*",
        element: <AppMain />,
        // Keeps the themed shell (background, toaster) around the error screen
        errorElement: <RouteErrorScreen />,
        children: [
          { index: true, element: <LandingPage /> },
          {
            element: <CourseRoutes />,
            children: [
              { path: "learn", element: <ReviewPage /> },
              { path: "home", element: <HomePage /> },
              { path: "stats", element: <StatsPage /> },
              { path: "due", element: <DueWordsPage /> },
              { path: "dictionary", element: <DictionaryPage /> },
              { path: "goals", element: <GoalsPage /> },
              { path: "select-language", element: <SelectLanguagePage /> },
            ],
          },
          ...(import.meta.env.DEV
            ? [
                {
                  path: "fixture/:name",
                  lazy: async () => {
                    const { FixturePage } = await import("./pages/fixture");
                    return { Component: FixturePage };
                  },
                },
              ]
            : []),
          { path: "user/id/:id", element: <UserProfilePage /> },
          { path: "*", element: <NotFoundPage /> },
        ],
      },
    ],
  },
]);

function App() {
  return <RouterProvider router={router} />;
}

function SelectLanguagePage() {
  const { userInfo } = useOutletContext<AppContextType>();
  const weapon = useWeapon();
  const deckSelection = useDeckSelection();
  const navigate = useNavigate();

  return match(deckSelection)
    .with(
      { type: "languageSelected" },
      ({ targetLanguage, hasHeardAbout, onboardedLanguages }) => (
        <LanguageSelector
          currentTargetLanguage={targetLanguage}
          showResumeButton={true}
          onResume={() => navigate("/learn")}
          onLanguagesConfirmed={(native, target) => {
            weapon.add_deck_selection_event({
              SelectBothLanguages: { native, target },
            });
            navigate("/learn");
          }}
          onOnboardingComplete={(selections, language) => {
            weapon.add_deck_selection_event({
              SetOnboardingSelections: {
                selections,
                target_language: language,
              },
            });
          }}
          hasHeardAbout={hasHeardAbout}
          onHeardAbout={(heard_about) => {
            weapon.add_deck_selection_event({ SetHeardAbout: { heard_about } });
          }}
          onboardedLanguages={onboardedLanguages}
          userInfo={userInfo}
          onBack={() => navigate("/learn")}
        />
      ),
    )
    .with(
      { type: "noLanguageSelected" },
      ({ hasHeardAbout, onboardedLanguages }) => (
        <LanguageSelector
          onLanguagesConfirmed={(native, target) => {
            weapon.add_deck_selection_event({
              SelectBothLanguages: { native, target },
            });
            navigate("/learn");
          }}
          onOnboardingComplete={(selections, language) => {
            weapon.add_deck_selection_event({
              SetOnboardingSelections: {
                selections,
                target_language: language,
              },
            });
          }}
          hasHeardAbout={hasHeardAbout}
          onHeardAbout={(heard_about) => {
            weapon.add_deck_selection_event({ SetHeardAbout: { heard_about } });
          }}
          onboardedLanguages={onboardedLanguages}
          userInfo={userInfo}
        />
      ),
    )
    .with(null, () => (
      <TopPageLayout
        userInfo={userInfo}
        headerProps={{
          backButton: { label: "Yap.Town", onBack: () => navigate("/") },
        }}
      >
        <div className="flex-1 flex items-center justify-center">
          <p className="text-muted-foreground animate-fade-in-delayed">
            Loading...
          </p>
        </div>
      </TopPageLayout>
    ))
    .exhaustive();
}

export { useDeck, useDeckSelection } from "@/hooks/useDeck";

export default App;
