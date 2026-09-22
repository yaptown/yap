import { useDeckSelection } from "@/core/useDeck";
import { useNavigate, useOutletContext } from "react-router-dom";
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

  return match(deckSelection)
    .with(
      { type: "languageSelected" },
      ({ targetLanguage, hasHeardAbout, onboardedLanguages }) => (
        <CoursePicker
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
        <CoursePicker
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
