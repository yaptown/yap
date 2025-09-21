import { useState, useEffect, useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { ArrowRight, Check, ChevronsUpDown } from "lucide-react";
import {
  Carousel,
  CarouselContent,
  CarouselItem,
  type CarouselApi,
} from "@/components/ui/carousel";
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
import { cn } from "@/lib/utils";
import type { Language } from "../../../yap-frontend-rs/pkg/yap_frontend_rs";
import { useWeapon } from "@/weapon";
import { get_available_courses } from "../../../yap-frontend-rs/pkg/yap_frontend_rs";

type LanguageSelectionState =
  | { stage: "selectingNative" }
  | { stage: "selectingTarget"; nativeLanguage: Language }
  | {
      stage: "askingExperience";
      nativeLanguage: Language;
      targetLanguage: Language;
    }
  | { stage: "onboarding"; nativeLanguage: Language; targetLanguage: Language };

interface LanguageSelectorProps {
  onLanguagesConfirmed: (native: Language, target: Language) => void;
  skipOnboarding: boolean;
  currentTargetLanguage?: Language;
}

export function LanguageSelector({
  onLanguagesConfirmed,
  skipOnboarding,
  currentTargetLanguage,
}: LanguageSelectorProps) {
  const [selectionState, setSelectionState] = useState<LanguageSelectionState>({
    stage: "selectingNative",
  });
  const [api, setApi] = useState<CarouselApi>();
  const [current, setCurrent] = useState(0);
  const [comboboxOpen, setComboboxOpen] = useState(false);
  const [userKnowsLanguage, setUserKnowsLanguage] = useState<
    "knows_some" | "beginner" | null
  >(null);
  const weapon = useWeapon();

  // Get available courses
  const availableCourses = useMemo(() => get_available_courses(), []);

  // Get unique native languages - memoized for stability
  const nativeLanguages = useMemo(() => {
    const uniqueNative = new Set<Language>();
    availableCourses.forEach((course) => {
      uniqueNative.add(course.nativeLanguage);
    });
    return Array.from(uniqueNative);
  }, [availableCourses]);

  // Map browser language codes to our Language types and detect browser language
  // NOTE: This language map must be updated whenever a new language is added to the Language enum
  const detectedLanguage = useMemo(() => {
    const languageMap: Record<string, Language> = {
      en: "English",
      "en-US": "English",
      "en-GB": "English",
      fr: "French",
      "fr-FR": "French",
      es: "Spanish",
      "es-ES": "Spanish",
      ko: "Korean",
      "ko-KR": "Korean",
    };

    // Get browser language
    const browserLang = navigator.language || navigator.languages?.[0];

    // Check if browser language is supported
    const detectedLang = browserLang
      ? languageMap[browserLang] || languageMap[browserLang.split("-")[0]]
      : null;

    return detectedLang && nativeLanguages.includes(detectedLang)
      ? detectedLang
      : null;
  }, [nativeLanguages]);

  // Auto-select detected language on mount
  useEffect(() => {
    // Only run on initial mount when in native selection stage
    if (selectionState.stage !== "selectingNative") return;

    if (detectedLanguage) {
      setSelectionState({
        stage: "selectingTarget",
        nativeLanguage: detectedLanguage,
      });
    }
  }, []); // Only run on mount

  // Get target languages available for selected native language
  const targetLanguages =
    selectionState.stage === "selectingNative"
      ? []
      : availableCourses
          .filter(
            (course) => course.nativeLanguage === selectionState.nativeLanguage
          )
          .map((course) => course.targetLanguage);

  useEffect(() => {
    if (!api) {
      return;
    }

    setCurrent(api.selectedScrollSnap());

    api.on("select", () => {
      setCurrent(api.selectedScrollSnap());
    });
  }, [api]);

  const languageFlags: Record<string, string> = {
    French: "🇫🇷",
    Spanish: "🇪🇸",
    Korean: "🇰🇷",
    English: "🇬🇧",
  };

  const languageConfirmTexts: Record<string, string> = {
    French: "Allons-y",
    Spanish: "Vamos",
    Korean: "가자",
    English: "Let's go",
  };

  // Native names of languages
  const nativeLanguageNames: Record<string, string> = {
    English: "English",
    French: "Français",
    Spanish: "Español",
    Korean: "한국어",
  };

  // "I speak [language]" in each language
  const iSpeakPhrases: Record<string, string> = {
    English: "I speak English",
    French: "Je parle français",
    Spanish: "Hablo español",
    Korean: "한국어를 합니다",
  };

  const languageColors: Record<
    string,
    { primary: string; secondary: string; accent: string; gradient: string }
  > = {
    French: {
      primary: "#002395",
      secondary: "#FFFFFF",
      accent: "#ED2939",
      gradient:
        "linear-gradient(90deg, #002395 33%, #FFFFFF 33% 66%, #ED2939 66%)",
    },
    Spanish: {
      primary: "#C60B1E",
      secondary: "#FFC400",
      accent: "#C60B1E",
      gradient:
        "linear-gradient(180deg, #C60B1E 25%, #FFC400 25% 75%, #C60B1E 75%)",
    },
    Korean: {
      primary: "#003478",
      secondary: "#FFFFFF",
      accent: "#C60B1E",
      gradient: "linear-gradient(180deg, #FFFFFF 50%, #C60B1E 50%)",
    },
    English: {
      primary: "#012169",
      secondary: "#FFFFFF",
      accent: "#C8102E",
      gradient:
        "linear-gradient(90deg, #012169 33%, #FFFFFF 33% 66%, #C8102E 66%)",
    },
  };

  // Determine if languages are beta
  const isBeta = (lang: Language) => {
    // French is stable, others are beta for now
    return lang !== "French";
  };

  useEffect(() => {
    if (selectionState.stage === "onboarding") {
      weapon.cache_language_pack({
        nativeLanguage: selectionState.nativeLanguage,
        targetLanguage: selectionState.targetLanguage,
      });
    }
  }, [selectionState, weapon]);

  const acknowledgeExperience =
    userKnowsLanguage === "knows_some"
      ? {
          title: "Great! We'll find your level.",
          content:
            "The first few challenges will serve as a placement test. The difficulty will ramp up quickly to find where you're at.",
        }
      : {
          title: "Perfect! Welcome aboard.",
          content:
            "We'll start you at the very beginning and build your foundation step by step.",
        };

  const introScreens = skipOnboarding
    ? [acknowledgeExperience]
    : [
        acknowledgeExperience,
        {
          title: "Yap values your time.",
          content:
            "Every design decision in Yap is based on helping you learn the most in the time you spend.",
        },
        userKnowsLanguage === "knows_some"
          ? {
              title: "Yap adapts to your skill level.",
              content:
                "So you don't waste time reviewing what you already learned on Duolingo.",
            }
          : {
              title: "Yap teaches you the most common words first.",
              content:
                "It'll surprise you how much you can say with just a few words.",
            },
        {
          title: "Yap has no lesson plan.",
          content:
            "Every lesson adapts to what YOU struggle with, not what lesson 47 says you should know.",
        },
        {
          title: "Yap reminds you of words just before you forget them.",
          content:
            "We use spaced repetition to review each word at the perfect time.",
        },
      ];

  // Background floating elements with deterministic pseudo-random values
  const floatingWords = useMemo(
    () => [
      { text: "Bonjour", lang: "French", seed: 0.2 },
      { text: "Hola", lang: "Spanish", seed: 0.7 },
      { text: "안녕", lang: "Korean", seed: 0.4 },
      { text: "Hello", lang: "English", seed: 0.9 },
      { text: "Merci", lang: "French", seed: 0.3 },
      { text: "Gracias", lang: "Spanish", seed: 0.6 },
      { text: "감사", lang: "Korean", seed: 0.8 },
      { text: "Thanks", lang: "English", seed: 0.1 },
      { text: "Oui", lang: "French", seed: 0.5 },
      { text: "Sí", lang: "Spanish", seed: 0.35 },
      { text: "네", lang: "Korean", seed: 0.75 },
      { text: "Yes", lang: "English", seed: 0.45 },
    ],
    []
  );

  return (
    <>
      {/* Full-page animated background */}
      <div className="fixed inset-0 pointer-events-none overflow-hidden">
        {floatingWords.map((word, index) => (
          <motion.div
            key={`${word.text}-${index}`}
            className="absolute text-4xl md:text-6xl font-bold opacity-[0.06] dark:opacity-[0.1] select-none dark:mix-blend-plus-lighter brightness-100 dark:brightness-500"
            style={{
              left: `${10 + ((index * 25) % 80)}%`,
              top: `${10 + ((index * 15) % 70)}%`,
              color: languageColors[word.lang]?.primary || "#000",
            }}
            initial={{
              x: 0,
              y: 0,
              rotate: word.seed * 30 - 15,
            }}
            animate={{
              x: [0, 30, -20, 0],
              y: [0, -40, 20, 0],
              rotate: [
                word.seed * 30 - 15,
                ((word.seed * 2) % 1) * 30 - 15,
                ((word.seed * 3) % 1) * 30 - 15,
                ((word.seed * 4) % 1) * 30 - 15,
              ],
            }}
            transition={{
              duration: 20 + index * 2,
              repeat: Infinity,
              repeatType: "reverse",
              ease: "easeInOut",
            }}
          >
            {word.text}
          </motion.div>
        ))}

        {/* Gradient orbs */}
        {["French", "Spanish", "Korean", "English"].map((lang, index) => (
          <motion.div
            key={`orb-${lang}`}
            className="absolute rounded-full blur-3xl opacity-30 dark:opacity-50"
            style={{
              width: "500px",
              height: "500px",
              background: `radial-gradient(circle, ${
                languageColors[lang]?.primary + "40"
              }, transparent)`,
              left: `${index * 25}%`,
              top: `${index % 2 === 0 ? -10 : 60}%`,
            }}
            animate={{
              x: [0, 100, -50, 0],
              y: [0, -50, 100, 0],
              scale: [1, 1.2, 0.8, 1],
            }}
            transition={{
              duration: 30 + index * 5,
              repeat: Infinity,
              repeatType: "reverse",
              ease: "easeInOut",
            }}
          />
        ))}
      </div>

      {/* Main content */}
      <div className="relative z-10 flex items-center justify-center mt-8">
        <AnimatePresence mode="wait">
          {selectionState.stage === "selectingNative" ? (
            // Step 1: Select native language
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

              <div className="grid md:grid-cols-2 gap-8 w-full max-w-2xl">
                {nativeLanguages.map((lang) => (
                  <motion.div
                    key={lang}
                    whileHover={{ scale: 1.05 }}
                    whileTap={{ scale: 0.98 }}
                  >
                    <Card
                      className="relative overflow-hidden p-2 text-center group transition-all duration-300 hover:shadow-2xl cursor-pointer border-2 aspect-square flex items-center justify-center"
                      onClick={() => {
                        setSelectionState({
                          stage: "selectingTarget",
                          nativeLanguage: lang,
                        });
                      }}
                    >
                      <div
                        className="absolute inset-0 opacity-0 group-hover:opacity-10 transition-opacity duration-300"
                        style={{ background: languageColors[lang]?.gradient }}
                      />
                      <div className="relative z-10">
                        <div className="text-8xl mb-4">
                          {languageFlags[lang]}
                        </div>
                        <h2 className="text-2xl font-bold mb-1">
                          {iSpeakPhrases[lang]}
                        </h2>
                        <p className="text-lg text-muted-foreground">
                          {nativeLanguageNames[lang]}
                        </p>
                      </div>
                    </Card>
                  </motion.div>
                ))}
              </div>
            </motion.div>
          ) : selectionState.stage === "selectingTarget" ? (
            // Step 2: Select target language
            <motion.div
              key="target-selection"
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
                  What language will you speak next?
                </h1>
                <div className="flex items-center justify-center gap-2 mb-6">
                  <span className="text-lg text-muted-foreground">
                    Native language:
                  </span>
                  <Popover open={comboboxOpen} onOpenChange={setComboboxOpen}>
                    <PopoverTrigger asChild>
                      <Button
                        variant="outline"
                        role="combobox"
                        aria-expanded={comboboxOpen}
                        className="w-[180px] justify-between"
                      >
                        <>
                          <span className="mr-2">
                            {languageFlags[selectionState.nativeLanguage]}
                          </span>
                          {selectionState.nativeLanguage}
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
                                value={lang}
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
                                      : "opacity-0"
                                  )}
                                />
                                <span className="mr-2">
                                  {languageFlags[lang]}
                                </span>
                                {lang}
                              </CommandItem>
                            ))}
                          </CommandGroup>
                        </CommandList>
                      </Command>
                    </PopoverContent>
                  </Popover>
                </div>
              </div>

              <div className="grid md:grid-cols-3 grid-cols-2 gap-8 w-full">
                {targetLanguages.map((lang) => (
                  <motion.div
                    key={lang}
                    whileHover={{ scale: 1.05 }}
                    whileTap={{ scale: 0.98 }}
                  >
                    <Card
                      className="relative overflow-hidden p-2 text-center group transition-all duration-300 hover:shadow-2xl cursor-pointer border-2 aspect-square flex items-center justify-center"
                      onClick={() => {
                        setSelectionState({
                          stage: "askingExperience",
                          nativeLanguage: selectionState.nativeLanguage,
                          targetLanguage: lang,
                        });
                      }}
                    >
                      {isBeta(lang) && (
                        <Badge className="absolute bottom-1 right-1 z-20 gap-1">
                          Beta
                        </Badge>
                      )}
                      <div
                        className="absolute inset-0 opacity-0 group-hover:opacity-10 transition-opacity duration-300"
                        style={{ background: languageColors[lang]?.gradient }}
                      />
                      <div className="relative z-10">
                        <div className="md:text-8xl text-6xl mb-4">
                          {languageFlags[lang]}
                        </div>
                        <h2 className="md:text-3xl text-2xl font-bold mb-2">
                          {nativeLanguageNames[lang]}
                        </h2>
                      </div>
                    </Card>
                  </motion.div>
                ))}
              </div>

              <div className="text-center mb-12">
                <p className="text-xl text-muted-foreground/70">
                  (Yap.Town is great for beginner and intermediate students.)
                </p>
              </div>
            </motion.div>
          ) : selectionState.stage === "askingExperience" ? (
            // Step 3: Ask about experience level
            <motion.div
              key="experience-question"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -20 }}
              className="w-full max-w-2xl gap-4 flex flex-col items-center"
            >
              <div className="text-center mb-8">
                <h1
                  className="text-5xl font-bold mb-4"
                  style={{ textWrap: "balance" }}
                >
                  Do you already know some{" "}
                  {nativeLanguageNames[selectionState.targetLanguage]}?
                </h1>
              </div>

              <div className="flex flex-col gap-4 w-full max-w-md">
                <Button
                  size="lg"
                  variant="outline"
                  onClick={() => {
                    setUserKnowsLanguage("knows_some");
                    setSelectionState({
                      stage: "onboarding",
                      nativeLanguage: selectionState.nativeLanguage,
                      targetLanguage: selectionState.targetLanguage,
                    });
                  }}
                  className="text-lg py-8 hover:scale-105 transition-transform"
                >
                  Yes, I know some
                </Button>
                <Button
                  size="lg"
                  variant="outline"
                  onClick={() => {
                    setUserKnowsLanguage("beginner");
                    setSelectionState({
                      stage: "onboarding",
                      nativeLanguage: selectionState.nativeLanguage,
                      targetLanguage: selectionState.targetLanguage,
                    });
                  }}
                  className="text-lg py-8 hover:scale-105 transition-transform"
                >
                  No, I'm starting fresh
                </Button>
              </div>

              <Button
                variant="ghost"
                className="mt-6"
                onClick={() => {
                  setSelectionState({
                    stage: "selectingTarget",
                    nativeLanguage: selectionState.nativeLanguage,
                  });
                }}
              >
                Back
              </Button>
            </motion.div>
          ) : selectionState.stage === "onboarding" ? (
            // Step 4: Onboarding screens (if not skipping)
            <motion.div
              initial={{ opacity: 0, scale: 0.95 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.95 }}
              className="w-full max-w-4xl"
            >
              <Carousel
                setApi={setApi}
                className="w-full"
                opts={{
                  align: "start",
                }}
              >
                <CarouselContent>
                  {introScreens.map((screen, index) => (
                    <CarouselItem key={index}>
                      <Card className="p-4 pt-12 pb-12">
                        <div className="text-center">
                          <h2
                            className="text-3xl font-bold mb-6"
                            style={{ textWrap: "balance" }}
                          >
                            {screen.title}
                          </h2>
                          <p
                            className="text-lg mb-8"
                            style={{ textWrap: "balance" }}
                          >
                            {screen.content}
                          </p>
                        </div>
                      </Card>
                    </CarouselItem>
                  ))}
                  <CarouselItem>
                    <Card
                      className="p-12"
                      style={{
                        background: `linear-gradient(135deg, ${
                          languageColors[selectionState.targetLanguage]?.primary
                        }80, ${
                          languageColors[selectionState.targetLanguage]?.accent
                        }80)`,
                      }}
                    >
                      <div className="text-center">
                        <motion.div
                          initial={{ scale: 0 }}
                          animate={{ scale: 1 }}
                          transition={{
                            type: "spring",
                            stiffness: 200,
                            damping: 15,
                          }}
                          className="text-9xl mb-6"
                        >
                          {languageFlags[selectionState.targetLanguage]}
                        </motion.div>
                        <h2 className="text-3xl font-bold mb-6">
                          {selectionState.targetLanguage ===
                            currentTargetLanguage ||
                          userKnowsLanguage == "knows_some"
                            ? `Ready to continue learning ${
                                nativeLanguageNames[
                                  selectionState.targetLanguage
                                ]
                              }?`
                            : `Ready to start learning ${
                                nativeLanguageNames[
                                  selectionState.targetLanguage
                                ]
                              }?`}
                        </h2>
                      </div>
                    </Card>
                  </CarouselItem>
                </CarouselContent>
              </Carousel>
              <div className="flex gap-4 justify-center mt-6">
                <Button
                  variant="outline"
                  size="lg"
                  onClick={() => {
                    if (current === 0) {
                      setSelectionState({
                        stage: "selectingTarget",
                        nativeLanguage: selectionState.nativeLanguage,
                      });
                    } else {
                      api?.scrollPrev();
                    }
                  }}
                  className="min-w-[120px]"
                >
                  Back
                </Button>
                <Button
                  size="lg"
                  onClick={() => {
                    if (current === introScreens.length) {
                      onLanguagesConfirmed(
                        selectionState.nativeLanguage,
                        selectionState.targetLanguage
                      );
                    } else {
                      api?.scrollNext();
                    }
                  }}
                  className={`min-w-[120px] flex items-center gap-2 ${
                    current === introScreens.length
                      ? "text-white hover:opacity-90 active:scale-95 transition-all"
                      : ""
                  }`}
                  style={{
                    ...(current === introScreens.length
                      ? {
                          background: `linear-gradient(135deg, ${
                            languageColors[selectionState.targetLanguage]
                              ?.primary
                          }, ${
                            languageColors[selectionState.targetLanguage]
                              ?.accent
                          })`,
                        }
                      : {}),
                  }}
                >
                  {current === introScreens.length
                    ? languageConfirmTexts[selectionState.targetLanguage]
                    : "Next"}
                  <ArrowRight className="h-4 w-4" />
                </Button>
              </div>
            </motion.div>
          ) : null}
        </AnimatePresence>
      </div>
    </>
  );
}
