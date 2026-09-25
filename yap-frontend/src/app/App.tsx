import { HomePage } from "@/browse/HomeScreen";
import { GoalsPage } from "@/browse/GoalsScreen";
import { StatsPage } from "@/browse/StatsScreen";
import { DueWordsPage } from "@/browse/DueWordsScreen";
import { CourseRoutes } from "@/review/course-study";
import {
  createBrowserRouter,
  RouterProvider,
  Outlet,
  ScrollRestoration,
} from "react-router-dom";
import { ThemeProvider } from "@/components/theme-provider";
import { RouteErrorScreen } from "@/components/route-error-screen";
import { ResetPassword } from "@/pages/reset-password";
import { ConfirmEmail } from "@/pages/confirm-email";
import { AcceptInvite } from "@/pages/accept-invite";
import { ForgotPassword } from "@/pages/forgot-password";
import { Connect } from "../pages/connect";
import { UserProfilePage } from "@/pages/user-profile";
import { AboutPage } from "@/pages/about";
import { PrivacyPage } from "@/pages/privacy";
import { TermsPage } from "@/pages/terms";
import { McpDocsPage } from "@/pages/mcp-docs";
import { LandingPage } from "@/pages/landing";
import { NotFoundPage } from "@/pages/not-found";
import { Toaster } from "sonner";
import { BackgroundShader } from "@/components/BackgroundShader";
import { AppMain } from "./SessionRoot";
import { ReviewPage } from "@/review/ReviewPage";
import { DictionaryPage } from "@/browse/DictionaryScreen";
import { SelectLanguagePage } from "@/onboarding/CoursePickerPage";

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
              {
                path: "anki",
                lazy: async () => {
                  const { AnkiPage } = await import("@/anki/AnkiPage");
                  return { Component: AnkiPage };
                },
              },
              { path: "select-language", element: <SelectLanguagePage /> },
            ],
          },
          ...(import.meta.env.DEV
            ? [
                {
                  path: "fixture/:name",
                  lazy: async () => {
                    const { FixturePage } = await import("../pages/fixture");
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

export default App;
