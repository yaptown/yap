import * as Sentry from "@sentry/react";
import { Fragment, useState, useEffect, useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ArrowRight, Check, ChevronsUpDown } from "lucide-react";
import {
  OnboardingFlow,
  type OnboardingSelections,
  type HeardAbout,
} from "@/onboarding/OnboardingFlow";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { cn, nativeLanguageNames } from "@/lib/utils";
import { LanguageIcon } from "@/components/LanguageIcon";
import { LANGUAGES, detectBrowserLanguage } from "@/lib/languages";
import type {
  Language,
  OnboardingPurpose,
} from "../../../yap-frontend-rs/pkg/yap_frontend_rs";
import { useWeapon } from "@/core/weapon";
import {
  get_available_courses,
  get_language_name,
} from "../../../yap-frontend-rs/pkg/yap_frontend_rs";
import { TopPageLayout } from "@/components/TopPageLayout";
import type { UserInfo } from "@/app/context";

type LanguageSelectionState =
  | { stage: "selectingNative" }
  | { stage: "selectingTarget"; nativeLanguage: Language }
  | { stage: "onboarding"; nativeLanguage: Language; targetLanguage: Language };

interface CoursePickerProps {
  onLanguagesConfirmed: (native: Language, target: Language) => void;
  onOnboardingComplete: (
    selections: OnboardingSelections,
    target: Language,
  ) => void;
  onHeardAbout: (value: HeardAbout) => void;
  hasHeardAbout: boolean;
  onboardedLanguages: Language[];
  currentTargetLanguage?: Language;
  showResumeButton?: boolean;
  onResume?: () => void;
  userInfo?: UserInfo;
  onBack?: () => void;
  purpose?: OnboardingPurpose;
}

export function CoursePicker({
  onLanguagesConfirmed,
  onOnboardingComplete,
  onHeardAbout,
  hasHeardAbout,
  onboardedLanguages,
  currentTargetLanguage,
  showResumeButton,
  onResume,
  userInfo,
  onBack,
  purpose = "App",
}: CoursePickerProps) {
  const [selectionState, setSelectionState] = useState<LanguageSelectionState>({
    stage: "selectingNative",
  });
  const [comboboxOpen, setComboboxOpen] = useState(false);
  const weapon = useWeapon();

  const handleTargetLanguageSelected = (
    nativeLanguage: Language,
    lang: Language,
  ) => {
    if (onboardedLanguages.includes(lang)) {
      onLanguagesConfirmed(nativeLanguage, lang);
    } else {
      setSelectionState({
        stage: "onboarding",
        nativeLanguage,
        targetLanguage: lang,
      });
    }
  };

  const availableCourses = useMemo(() => get_available_courses(), []);

  const nativeLanguages = useMemo(() => {
    const uniqueNative = new Set<Language>();
    availableCourses.forEach((course) => {
      uniqueNative.add(course.nativeLanguage);
    });
    return Array.from(uniqueNative);
  }, [availableCourses]);

  const detectedLanguage = useMemo(() => {
    const detectedLang = detectBrowserLanguage();
    return detectedLang && nativeLanguages.includes(detectedLang)
      ? detectedLang
      : null;
  }, [nativeLanguages]);

  useEffect(() => {
    if (selectionState.stage !== "selectingNative") return;

    if (detectedLanguage) {
      setSelectionState({
        stage: "selectingTarget",
        nativeLanguage: detectedLanguage,
      });
    }
  }, [selectionState.stage, detectedLanguage]);

  const targetLanguages =
    selectionState.stage === "selectingNative"
      ? []
      : availableCourses
          .filter(
            (course) => course.nativeLanguage === selectionState.nativeLanguage,
          )
          .map((course) => course.targetLanguage);

  useEffect(() => {
    if (selectionState.stage === "onboarding") {
      Sentry.addBreadcrumb({
        category: "language-pack",
        message: `Caching ${selectionState.nativeLanguage} → ${selectionState.targetLanguage}`,
        level: "info",
      });
      weapon
        .cache_language_pack({
          nativeLanguage: selectionState.nativeLanguage,
          targetLanguage: selectionState.targetLanguage,
        })
        .catch((e: unknown) => {
          Sentry.captureException(e);
        });
    }
  }, [selectionState, weapon]);

  const yaptownTitle =
    selectionState.stage === "onboarding"
      ? LANGUAGES[selectionState.targetLanguage].yaptownName
      : "Yap.Town";

  return (
    <TopPageLayout
      userInfo={userInfo}
      headerProps={{
        showSignupNag: false,
        title: yaptownTitle,
        backButton: onBack ? { label: yaptownTitle, onBack } : undefined,
      }}
    >
      {/* Main content */}
      <div className="relative z-10 flex items-center justify-center">
        <AnimatePresence mode="wait">
          {selectionState.stage === "selectingNative" ? (
            <motion.div
              key="native-selection"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -20 }}
              className="w-full max-w-4xl gap-4 flex flex-col items-center"
            >
              <div className="text-center">
                <h1
                  className="text-5xl font-bold mb-4"
                  style={{ textWrap: "balance" }}
                >
                  What's your native language?
                </h1>
                <p className="text-xl text-muted-foreground mb-8">
                  So we can talk to you!
                </p>
              </div>

              <div className="grid grid-cols-2 gap-x-6 gap-y-10 w-full max-w-2xl">
                {nativeLanguages.map((lang) => (
                  <motion.button
                    key={lang}
                    type="button"
                    whileHover={{ scale: 1.05 }}
                    whileTap={{ scale: 0.97 }}
                    className="flex flex-col items-center gap-1 rounded-2xl p-2 cursor-pointer text-center"
                    onClick={() => {
                      setSelectionState({
                        stage: "selectingTarget",
                        nativeLanguage: lang,
                      });
                    }}
                  >
                    <LanguageIcon
                      icon={LANGUAGES[lang].icon}
                      className="size-32 md:size-40"
                    />
                    <span>
                      <span className="block text-2xl font-bold">
                        {LANGUAGES[lang].iSpeak}
                      </span>
                      <span className="block text-lg text-muted-foreground">
                        {nativeLanguageNames[lang]}
                      </span>
                    </span>
                  </motion.button>
                ))}
              </div>
            </motion.div>
          ) : selectionState.stage === "selectingTarget" ? (
            <div
              key="target-selection"
              className="w-full max-w-4xl gap-4 flex flex-col items-center gap-8"
            >
              <div className="text-center">
                <h1
                  className="text-5xl font-bold mb-4 mt-16"
                  style={{ textWrap: "balance" }}
                >
                  <span className="highlight animate-fade-in">
                    {purpose === "AnkiDeck"
                      ? "What language would you like an Anki deck for?"
                      : "What language will you speak next?"}
                  </span>
                </h1>
              </div>

              {showResumeButton && currentTargetLanguage && onResume && (
                <>
                  <Card
                    className="w-full max-w-md flex-row items-center gap-4 p-5 cursor-pointer transition-shadow hover:shadow-xl"
                    onClick={onResume}
                    animate
                  >
                    <LanguageIcon
                      icon={LANGUAGES[currentTargetLanguage].icon}
                      className="size-14"
                    />
                    <div className="text-left">
                      <h3 className="text-xl md:text-2xl font-bold">
                        Resume{" "}
                        {
                          get_language_name(
                            currentTargetLanguage,
                            selectionState.nativeLanguage,
                          ).full
                        }
                      </h3>
                      <p className="text-sm text-muted-foreground">
                        Continue where you left off
                      </p>
                    </div>
                    <ArrowRight className="h-6 w-6 ml-auto" />
                  </Card>
                  <SectionDivider label="Or choose a different language" />
                </>
              )}

              {(
                [
                  ["stable", null],
                  ["beta", "Beta Languages"],
                  ["alpha", "Alpha Languages"],
                ] as const
              ).map(([status, label]) => {
                const languages = targetLanguages.filter(
                  (lang) => LANGUAGES[lang].status === status,
                );
                if (languages.length === 0) return null;
                return (
                  <Fragment key={status}>
                    {label && <SectionDivider label={label} />}
                    <div className="grid md:grid-cols-3 grid-cols-2 gap-x-6 gap-y-10 w-full">
                      {languages.map((lang) => (
                        <LanguageTile
                          key={lang}
                          language={lang}
                          reader={selectionState.nativeLanguage}
                          onClick={() =>
                            handleTargetLanguageSelected(
                              selectionState.nativeLanguage,
                              lang,
                            )
                          }
                        />
                      ))}
                    </div>
                  </Fragment>
                );
              })}

              {/* Native language selector */}
              <div className="flex items-center justify-center gap-2 mb-6">
                <span className="text-lg text-muted-foreground animate-fade-in">
                  Native language:
                </span>
                <Popover open={comboboxOpen} onOpenChange={setComboboxOpen}>
                  <PopoverTrigger asChild>
                    <Button
                      variant="outline"
                      role="combobox"
                      aria-expanded={comboboxOpen}
                      className="w-[180px] justify-between animate-fade-in"
                      animate
                    >
                      <>
                        <LanguageIcon
                          icon={LANGUAGES[selectionState.nativeLanguage].icon}
                          className="size-6"
                        />
                        {nativeLanguageNames[selectionState.nativeLanguage]}
                      </>
                      <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent className="w-[180px] p-0">
                    <Command>
                      <CommandInput placeholder="Search language..." />
                      <CommandList>
                        <CommandEmpty>No language found.</CommandEmpty>
                        <CommandGroup>
                          {nativeLanguages.map((lang) => (
                            <CommandItem
                              key={lang}
                              value={nativeLanguageNames[lang]}
                              onSelect={() => {
                                setSelectionState({
                                  stage: "selectingTarget",
                                  nativeLanguage: lang,
                                });
                                setComboboxOpen(false);
                              }}
                            >
                              <Check
                                className={cn(
                                  "mr-2 h-4 w-4",
                                  selectionState.nativeLanguage === lang
                                    ? "opacity-100"
                                    : "opacity-0",
                                )}
                              />
                              <LanguageIcon
                                icon={LANGUAGES[lang].icon}
                                className="size-6"
                              />
                              {nativeLanguageNames[lang]}
                            </CommandItem>
                          ))}
                        </CommandGroup>
                      </CommandList>
                    </Command>
                  </PopoverContent>
                </Popover>
              </div>

              <div className="text-center mb-12">
                <p className="text-xl text-muted-foreground/70">
                  (Yap.Town is great for beginner and intermediate students.)
                </p>
              </div>
            </div>
          ) : selectionState.stage === "onboarding" ? (
            <OnboardingFlow
              purpose={purpose}
              targetLanguage={selectionState.targetLanguage}
              nativeLanguage={selectionState.nativeLanguage}
              hasHeardAbout={hasHeardAbout}
              onHeardAbout={onHeardAbout}
              onComplete={(selections) => {
                onOnboardingComplete(selections, selectionState.targetLanguage);
                onLanguagesConfirmed(
                  selectionState.nativeLanguage,
                  selectionState.targetLanguage,
                );
              }}
              onBack={() => {
                setSelectionState({
                  stage: "selectingTarget",
                  nativeLanguage: selectionState.nativeLanguage,
                });
              }}
            />
          ) : null}
        </AnimatePresence>
      </div>
    </TopPageLayout>
  );
}

function LanguageTile({
  language,
  reader,
  onClick,
}: {
  language: Language;
  reader: Language;
  onClick: () => void;
}) {
  const { name, variant } = get_language_name(language, reader);
  return (
    <motion.button
      type="button"
      whileHover={{ scale: 1.05 }}
      whileTap={{ scale: 0.97 }}
      className="flex flex-col items-center gap-1 rounded-2xl p-2 cursor-pointer text-center"
      onClick={onClick}
    >
      <LanguageIcon
        icon={LANGUAGES[language].icon}
        className="size-32 md:size-40"
      />
      <span>
        <span className="block text-2xl md:text-3xl font-bold">{name}</span>
        {variant && (
          <span className="block text-base md:text-lg text-muted-foreground">
            {variant}
          </span>
        )}
      </span>
    </motion.button>
  );
}

function SectionDivider({ label }: { label: string }) {
  return (
    <div className="w-full max-w-md flex items-center gap-3 text-xs uppercase tracking-wider text-muted-foreground">
      <span className="flex-1 border-t" />
      {label}
      <span className="flex-1 border-t" />
    </div>
  );
}
