import { CourseAudioPrefetch } from "@/review/course-study";
import { useDeckSelection } from "@/core/useDeck";
import { useNavigate, useOutletContext, useSearchParams } from "react-router-dom";
import { CoursePicker } from "./CoursePicker";
import { useWeapon } from "../core/weapon";
import { TopPageLayout } from "@/components/TopPageLayout";
import { match } from "ts-pattern";
import type { AppContextType } from "@/app/context";

export function SelectLanguagePage() {
  const { userInfo } = useOutletContext<AppContextType>();
  const weapon = useWeapon();
  const deckSelection = useDeckSelection();
  const navigate = useNavigate();
  // Where to go once a course is chosen. Only same-app paths are honored.
  const [searchParams] = useSearchParams();
  const requested = searchParams.get("next");
  const next = requested?.startsWith("/") && !requested.startsWith("//") ? requested : "/learn";
  const purpose = next.startsWith("/anki") ? "AnkiDeck" : "App";

  return <>
    <CourseAudioPrefetch />
    {match(deckSelection)
    .with(
      { type: "languageSelected" },
      ({ targetLanguage, hasHeardAbout, onboardedLanguages }) => (
        <CoursePicker
          currentTargetLanguage={targetLanguage}
          showResumeButton={true}
          onResume={() => navigate(next)}
          onLanguagesConfirmed={(native, target) => {
            weapon.add_deck_selection_event({
              SelectBothLanguages: { native, target },
            });
            navigate(next);
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
          purpose={purpose}
          onBack={() => navigate(next)}
        />
      ),
    )
    .with(
      { type: "noLanguageSelected" },
      ({ hasHeardAbout, onboardedLanguages }) => (
        <CoursePicker
          onLanguagesConfirmed={(native, target) => {
            weapon.add_deck_selection_event({
              SelectBothLanguages: { native, target },
            });
            navigate(next);
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
          purpose={purpose}
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
    .exhaustive()}
  </>;
}
