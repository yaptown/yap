import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { ArrowRight, ArrowLeft } from "lucide-react";
import {
  Carousel,
  CarouselContent,
  CarouselItem,
  type CarouselApi,
} from "@/components/ui/carousel";
import type { Language } from "../../../yap-frontend-rs/pkg/yap_frontend_rs";
import { useWeapon } from "@/weapon";
import { get_available_courses } from "../../../yap-frontend-rs/pkg/yap_frontend_rs";

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
  const [nativeLanguage, setNativeLanguage] = useState<Language | null>(null);
  const [targetLanguage, setTargetLanguage] = useState<Language | null>(null);
  const [api, setApi] = useState<CarouselApi>();
  const [current, setCurrent] = useState(0);
  const weapon = useWeapon();

  // Get available courses
  const availableCourses = get_available_courses();

  // Get unique native languages
  const uniqueNative = new Set<Language>();
  availableCourses.forEach((course) => {
    uniqueNative.add(course.native_language);
  });
  const nativeLanguages = Array.from(uniqueNative);

  // Get target languages available for selected native language
  const targetLanguages = !nativeLanguage
    ? []
    : availableCourses
        .filter((course) => course.native_language === nativeLanguage)
        .map((course) => course.target_language);

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
    if (targetLanguage) {
      weapon.cache_language_pack(targetLanguage);
    }
  }, [targetLanguage, weapon]);

  const introScreens = skipOnboarding
    ? []
    : [
        {
          title: "Most language apps waste your time.",
          content:
            'They make you learn "animals" and "body parts" when you really need everyday words that matter.',
        },
        {
          title: "Yap teaches you the 1,000 most common words first.",
          content: "These words make up 80% of everyday conversation.",
        },
        {
          title: "Your brain forgets on a schedule. Yap knows it.",
          content:
            "We use spaced repetition—reviewing each word at the exact moment before you'd forget it.",
        },
        {
          title: "Other apps follow their plan. Yap follows your brain.",
          content:
            "Every lesson adapts to what YOU struggle with, not what lesson 47 says you should know.",
        },
        {
          title: "It's science, and it works.",
          content:
            "Once you feel the difference, you'll never want to go back.",
        },
      ];

  return (
    <div className="flex items-center justify-center mt-8">
      <AnimatePresence mode="wait">
        {!nativeLanguage ? (
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
                      setNativeLanguage(lang);
                    }}
                  >
                    <div
                      className="absolute inset-0 opacity-0 group-hover:opacity-10 transition-opacity duration-300"
                      style={{ background: languageColors[lang]?.gradient }}
                    />
                    <div className="relative z-10">
                      <div className="text-8xl mb-4">{languageFlags[lang]}</div>
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
        ) : !targetLanguage ? (
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
                What do you want to learn?
              </h1>
            </div>

            <div className="grid md:grid-cols-3 gap-8 w-full">
              {targetLanguages.map((lang) => (
                <motion.div
                  key={lang}
                  whileHover={{ scale: 1.05 }}
                  whileTap={{ scale: 0.98 }}
                >
                  <Card
                    className="relative overflow-hidden p-2 text-center group transition-all duration-300 hover:shadow-2xl cursor-pointer border-2 aspect-square flex items-center justify-center"
                    onClick={() => {
                      setTargetLanguage(lang);
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
                      <div className="text-8xl mb-4">{languageFlags[lang]}</div>
                      <h2 className="text-3xl font-bold mb-2">
                        {nativeLanguageNames[lang]}
                      </h2>
                    </div>
                  </Card>
                </motion.div>
              ))}
            </div>

            <div className="flex gap-4 justify-center mt-6">
              <Button
                variant="outline"
                size="lg"
                onClick={() => {
                  setNativeLanguage(null);
                }}
                className="min-w-[120px] flex items-center gap-2"
              >
                <ArrowLeft className="h-4 w-4" />
                Back
              </Button>
            </div>

            <div className="text-center mb-12">
              <p className="text-xl">
                (Yap.Town is great for beginner and intermediate students.)
              </p>
            </div>
          </motion.div>
        ) : targetLanguage ? (
          // Step 3: Onboarding screens (if not skipping)
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
                      background: `linear-gradient(135deg, ${languageColors[targetLanguage]?.primary}10, ${languageColors[targetLanguage]?.accent}10)`,
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
                        {languageFlags[targetLanguage]}
                      </motion.div>
                      <h2 className="text-3xl font-bold mb-6">
                        {targetLanguage === currentTargetLanguage
                          ? `Ready to continue learning ${nativeLanguageNames[targetLanguage]}?`
                          : `Ready to start learning ${nativeLanguageNames[targetLanguage]}?`}
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
                    setTargetLanguage(null);
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
                    onLanguagesConfirmed(nativeLanguage, targetLanguage);
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
                        background: `linear-gradient(135deg, ${languageColors[targetLanguage]?.primary}, ${languageColors[targetLanguage]?.accent})`,
                      }
                    : {}),
                }}
              >
                {current === introScreens.length
                  ? languageConfirmTexts[targetLanguage]
                  : "Next"}
                <ArrowRight className="h-4 w-4" />
              </Button>
            </div>
          </motion.div>
        ) : null}
      </AnimatePresence>
    </div>
  );
}
