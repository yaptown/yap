import { LanguageIcon } from "@/components/LanguageIcon";
import { TargetLanguageText } from "@/components/TargetLanguageText";
import {
  onboarding_start,
  onboarding_reduce,
  onboarding_view,
} from "../../../yap-frontend-rs/pkg";
import { useState, useRef, useLayoutEffect, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import {
  ArrowRight,
  ArrowLeft,
  Check,
  Users,
  Globe,
  Video,
  Search,
  Youtube,
  HelpCircle,
  Clock,
  GraduationCap,
  Heart,
  Briefcase,
  Plane,
  Sparkles,
  SignalZero,
  SignalLow,
  SignalMedium,
  SignalHigh,
  Signal,
  Bell,
  Brain,
  type LucideIcon,
} from "lucide-react";
import type {
  Language,
  HeardAbout,
  Motivation,
  ExperienceLevel,
  DailyReviewTarget,
  OnboardingSelections,
  OnboardingPurpose,
  OnboardingEvent,
  OnboardingChoice,
  OnboardingContent,
  OnboardingView,
  OnboardingChart,
  OnboardingCurve,
} from "../../../yap-frontend-rs/pkg/yap_frontend_rs";
import { useOneSignalNotifications } from "@/hooks/use-onesignal-notifications";

// Re-export for use in CoursePicker
export type { OnboardingSelections, HeardAbout };

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface OnboardingFlowProps {
  purpose: OnboardingPurpose;
  targetLanguage: Language;
  nativeLanguage: Language;
  hasHeardAbout: boolean;
  onHeardAbout: (value: HeardAbout) => void;
  onComplete: (selections: OnboardingSelections) => void;
  onBack: () => void;
}

// ---------------------------------------------------------------------------
// Forgetting Curve Chart
// ---------------------------------------------------------------------------

const curveColors = [
  "var(--chart-1)",
  "var(--chart-2)",
  "var(--chart-3)",
  "var(--chart-5)",
];

function ForgettingCurveChart({
  curves,
  chart,
}: {
  curves: OnboardingCurve[];
  chart: OnboardingChart;
}) {
  const W = 360;
  const H = 200;
  const left = 24;
  const right = 8;
  const top = 14;
  const bottom = 26;
  const plotW = W - left - right;
  const inset = 10;
  const plotH = H - top - bottom;
  const px = (t: number) => left + inset + t * (plotW - inset);
  const py = (memory: number) => top + (1 - memory) * plotH;
  const baseline = py(0);

  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      className="w-full max-w-sm mx-auto overflow-visible"
      aria-label={chart.accessibility_label}
    >
      {[0.25, 0.5, 0.75].map((memory) => (
        <line
          key={memory}
          x1={left}
          y1={py(memory)}
          x2={W - right}
          y2={py(memory)}
          stroke="currentColor"
          strokeOpacity={0.1}
          strokeDasharray="4 4"
        />
      ))}

      <AnimatePresence>
        {curves.map((curve, i) => {
          const points = Array.from({ length: 61 }, (_, s) => {
            const t = s / 60;
            const x = px(curve.start + t * (curve.end - curve.start));
            return `${s === 0 ? "M" : "L"}${x.toFixed(1)},${py(curve.retained ** t).toFixed(1)}`;
          }).join(" ");
          const fill = `${points} L${px(curve.end).toFixed(1)},${baseline} L${px(curve.start).toFixed(1)},${baseline} Z`;
          const color = curveColors[i % curveColors.length];
          return (
            <motion.g key={i}>
              {i > 0 && (
                <motion.line
                  x1={px(curve.start)}
                  y1={baseline}
                  x2={px(curve.start)}
                  y2={py(1)}
                  stroke={color}
                  strokeWidth={1.5}
                  strokeDasharray="4 3"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  transition={{ duration: 0.3 }}
                />
              )}
              <motion.path
                d={fill}
                fill={`color-mix(in oklch, ${color} 15%, transparent)`}
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ duration: 0.6, delay: 0.2 }}
              />
              <motion.path
                d={points}
                fill="none"
                stroke={color}
                strokeWidth={2.5}
                strokeLinecap="round"
                initial={{ pathLength: 0 }}
                animate={{ pathLength: 1 }}
                transition={{ duration: 0.8, ease: "easeOut" }}
              />
              <motion.circle
                cx={px(curve.start)}
                cy={py(1)}
                r={6}
                fill="var(--background)"
                stroke={color}
                strokeWidth={2.5}
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                transition={{ type: "spring", stiffness: 300, damping: 15 }}
              />
            </motion.g>
          );
        })}
      </AnimatePresence>

      {/* Axes */}
      <path
        d={`M${left},${top - 6} L${left},${baseline} L${W - right},${baseline}`}
        fill="none"
        stroke="currentColor"
        strokeOpacity={0.35}
        strokeWidth={1.5}
      />
      <text
        x={W - right}
        y={H - 6}
        textAnchor="end"
        className="fill-muted-foreground"
        fontSize={13}
        fontWeight={500}
      >
        {chart.x_label} →
      </text>
      <text
        x={0}
        y={0}
        textAnchor="end"
        className="fill-muted-foreground"
        fontSize={13}
        fontWeight={500}
        transform={`translate(${left - 8}, ${top}) rotate(-90)`}
      >
        {chart.y_label} →
      </text>
    </svg>
  );
}

// ---------------------------------------------------------------------------
// Individual Screens
// ---------------------------------------------------------------------------

function ScreenWrapper({
  children,
  screenKey,
}: {
  children: React.ReactNode;
  screenKey: string;
}) {
  return (
    <div
      key={screenKey}
      className="w-full max-w-lg mx-auto flex flex-col items-center gap-6 animate-slide-in-from-right"
    >
      {children}
    </div>
  );
}

function OptionButton({
  label,
  detail,
  selected,
  onClick,
  icon: Icon,
}: {
  label: string;
  detail?: string;
  selected: boolean;
  onClick: () => void;
  icon?: LucideIcon;
}) {
  return (
    <Button
      variant="outline"
      size="lg"
      onClick={onClick}
      className={`w-full h-auto min-h-14 text-left justify-between text-base py-3 transition-all active:scale-[0.98] ${
        selected
          ? "border-primary bg-primary/10 ring-2 ring-primary/30"
          : "hover:border-primary/40"
      }`}
    >
      <span className="flex items-center gap-3">
        {Icon && <Icon className="h-5 w-5 shrink-0 text-muted-foreground" />}
        <span className="flex flex-col">
          {label}
          {detail && (
            <span className="text-sm font-normal text-muted-foreground">
              {detail}
            </span>
          )}
        </span>
      </span>
      {selected && <Check className="h-4 w-4 text-primary shrink-0" />}
    </Button>
  );
}

// Choosing advances; the pick shows for a beat first so the tap registers.
function ChoicesScreen({
  view,
  content,
  send,
}: {
  view: OnboardingView;
  content: Extract<OnboardingContent, { type: "Choices" }>;
  send: (event: OnboardingEvent) => void;
}) {
  const [picked, setPicked] = useState<OnboardingChoice | null>(null);
  // Leaving the step (Back) cancels a pick that hasn't landed yet.
  useEffect(() => {
    if (!picked) return;
    const timer = setTimeout(
      () => send({ type: "Choose", choice: picked }),
      CHOICE_BEAT_MS,
    );
    return () => clearTimeout(timer);
  }, [picked, send]);
  return (
    <ScreenWrapper screenKey={view.step}>
      <h2
        className="text-3xl md:text-4xl font-bold text-center"
        style={{ textWrap: "balance" }}
      >
        {view.title}
      </h2>
      <div className="flex flex-col gap-3 w-full">
        {content.options.map((option, i) => (
          <motion.div
            key={option.choice.value}
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.05 * i, duration: 0.25, ease: "easeOut" }}
          >
            <OptionButton
              label={option.label}
              detail={option.detail}
              icon={choiceIcon(option.choice)}
              selected={
                picked ? picked.value === option.choice.value : option.selected
              }
              onClick={() => setPicked((current) => current ?? option.choice)}
            />
          </motion.div>
        ))}
      </div>
    </ScreenWrapper>
  );
}

const CHOICE_BEAT_MS = 220;

// The finish line: the course icon lands with a burst of confetti.
function ReadyScreen({
  view,
  content,
  send,
}: {
  view: OnboardingView;
  content: Extract<OnboardingContent, { type: "Ready" }>;
  send: (event: OnboardingEvent) => void;
}) {
  return (
    <ScreenWrapper screenKey={view.step}>
      <div className="relative size-40 flex items-center justify-center mt-6">
        <motion.div
          className="absolute inset-4 rounded-full bg-primary/25 blur-2xl"
          animate={{ scale: [1, 1.2, 1] }}
          transition={{ duration: 2.4, repeat: Infinity, ease: "easeInOut" }}
        />
        {Array.from({ length: 14 }, (_, i) => {
          const angle = (i / 14) * 2 * Math.PI;
          const distance = 70 + (i % 3) * 14;
          return (
            <motion.span
              key={i}
              className={`absolute ${i % 2 ? "size-2 rounded-full" : "w-1.5 h-3 rounded-sm"}`}
              style={{ backgroundColor: curveColors[i % curveColors.length] }}
              initial={{ x: 0, y: 0, opacity: 0, scale: 0 }}
              animate={{
                x: Math.cos(angle) * distance,
                y: Math.sin(angle) * distance,
                opacity: [0, 1, 1, 0],
                scale: [0, 1.2, 1, 0.6],
                rotate: i * 40,
              }}
              transition={{ delay: 0.25, duration: 1.2, ease: "easeOut" }}
            />
          );
        })}
        <motion.div
          initial={{ scale: 0, rotate: -20 }}
          animate={{ scale: 1, rotate: 0 }}
          transition={{ type: "spring", stiffness: 220, damping: 12 }}
          className="relative"
        >
          <LanguageIcon icon={content.icon} className="size-24" />
        </motion.div>
      </div>
      <motion.h2
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.35, duration: 0.4 }}
        className="text-4xl md:text-5xl font-bold text-center"
        style={{ textWrap: "balance" }}
      >
        {view.title}
      </motion.h2>
      <motion.p
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.55, duration: 0.4 }}
        className="text-muted-foreground text-center text-lg"
        style={{ textWrap: "balance" }}
      >
        {content.body}
      </motion.p>
      {content.start_fresh_label && (
        <Button
          size="lg"
          variant="outline"
          className="w-full max-w-sm"
          onClick={() => send({ type: "StartFromScratch" })}
        >
          {content.start_fresh_label}
        </Button>
      )}
    </ScreenWrapper>
  );
}

// Screen: SRS teaser
function SrsTeaserScreen({
  view,
  content,
}: {
  view: OnboardingView;
  content: Extract<OnboardingContent, { type: "Studies" }>;
}) {
  return (
    <ScreenWrapper screenKey="srs-teaser">
      <h2
        className="text-3xl md:text-4xl font-bold text-center leading-snug"
        style={{ textWrap: "balance" }}
      >
        {view.title}
      </h2>
      <div
        className="w-full relative h-56 select-none overflow-x-clip"
        aria-hidden
      >
        {content.studies.map((study, i) => (
          <motion.a
            key={study.title}
            href={study.url}
            target="_blank"
            rel="noopener noreferrer"
            initial={{ y: 60, opacity: 0 }}
            animate={{ y: 0, opacity: 1, rotate: (i - 1.5) * 2 }}
            transition={{
              delay: 0.3 + i * 0.5,
              duration: 0.5,
              ease: "easeOut",
            }}
            className="absolute inset-x-4 bg-white text-black rounded shadow-md px-5 py-4 border border-neutral-200 block"
            style={{ zIndex: i, top: `${i * 8}px`, height: "180px" }}
          >
            <p className="text-[11px] font-semibold leading-tight truncate">
              {study.title}
            </p>
            <p className="text-[9px] text-neutral-500 mt-1 truncate">
              {study.authors} ({study.year}).{" "}
              <span className="italic">{study.journal}</span>
            </p>
            {/* Simulated text lines */}
            <div className="mt-3 flex flex-col gap-1.5">
              {Array.from({ length: 6 }).map((_, j) => (
                <div
                  key={j}
                  className="h-1.5 bg-neutral-200 rounded-full"
                  style={{
                    width: `${j === 5 ? 40 + ((i * 10) % 30) : 75 + (((i + j) * 7) % 25)}%`,
                  }}
                />
              ))}
            </div>
          </motion.a>
        ))}
      </div>
      <div className="-mt-2 h-10 flex flex-col items-center justify-start">
        <motion.h3
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{
            delay: 0.3 + content.studies.length * 0.5 + 0.3,
            duration: 0.6,
          }}
          className="text-3xl font-bold italic text-accent-foreground"
        >
          {content.conclusion}
        </motion.h3>
      </div>
    </ScreenWrapper>
  );
}

// Screen: SRS Intro
function SrsIntroScreen({
  view,
  content,
}: {
  view: OnboardingView;
  content: Extract<OnboardingContent, { type: "Review" }>;
}) {
  const reviews = content.curves.length - 1;

  return (
    <ScreenWrapper screenKey="srs-intro">
      <Card className="w-full p-6 md:p-8 flex flex-col gap-4" animate>
        <p className="text-sm font-semibold text-primary tracking-wide uppercase">
          {content.eyebrow}
        </p>
        <h2 className="text-2xl md:text-3xl font-bold leading-snug">
          {view.title}
          <span className="italic text-primary">{content.title_emphasis}</span>
        </h2>

        <AnimatePresence mode="wait">
          {content.learned ? (
            <motion.div
              key="learned"
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              className="flex flex-col items-center gap-3 py-10 text-center"
            >
              <motion.div
                initial={{ scale: 0, rotate: -30 }}
                animate={{ scale: 1, rotate: 0 }}
                transition={{
                  type: "spring",
                  stiffness: 260,
                  damping: 14,
                  delay: 0.1,
                }}
                className="rounded-full bg-primary/15 p-4"
              >
                <Check className="h-10 w-10 text-primary" strokeWidth={3} />
              </motion.div>
              <p className="text-3xl font-bold">{content.learned_title}</p>
              <p className="text-muted-foreground text-lg">
                {content.learned_body}
              </p>
            </motion.div>
          ) : (
            <motion.div
              key="chart"
              exit={{ opacity: 0 }}
              className="flex flex-col gap-3"
            >
              {/* One dot per review, in its curve's color */}
              <div className="flex items-center gap-2 h-5">
                {content.curves.slice(1).map((_, i) => (
                  <motion.span
                    key={i}
                    className="w-3 h-3 rounded-full"
                    style={{
                      backgroundColor:
                        curveColors[(i + 1) % curveColors.length],
                    }}
                    initial={{ scale: 0 }}
                    animate={{ scale: 1 }}
                    transition={{ type: "spring", stiffness: 300, damping: 15 }}
                  />
                ))}
                {content.review_label && (
                  <span className="text-sm text-muted-foreground ml-1">
                    {reviews} {content.review_label}
                  </span>
                )}
              </div>
              <ForgettingCurveChart
                curves={content.curves}
                chart={content.chart}
              />
              <AnimatePresence mode="wait">
                <motion.p
                  key={content.caption}
                  initial={{ opacity: 0, y: 6 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -6 }}
                  transition={{ duration: 0.25 }}
                  className="text-muted-foreground text-center min-h-12"
                  style={{ textWrap: "balance" }}
                >
                  {content.caption}
                </motion.p>
              </AnimatePresence>
            </motion.div>
          )}
        </AnimatePresence>
      </Card>
    </ScreenWrapper>
  );
}

// Screen: words drifting into memory, and staying there
function WordsIntoMemory({
  words,
  language,
}: {
  words: string[];
  language: Language;
}) {
  const radius = 130;
  const cycle = 4;
  return (
    <div className="relative w-full h-64 select-none" aria-hidden>
      <div className="absolute inset-0 flex items-center justify-center">
        <motion.div
          className="absolute size-36 rounded-full bg-primary/20 blur-2xl"
          animate={{ scale: [1, 1.25, 1], opacity: [0.6, 1, 0.6] }}
          transition={{
            duration: cycle / words.length,
            repeat: Infinity,
            ease: "easeInOut",
          }}
        />
        <div className="relative rounded-full bg-primary/10 p-6 ring-1 ring-primary/20">
          <Brain className="size-16 text-primary" strokeWidth={1.5} />
        </div>
      </div>
      {words.map((word, i) => {
        const angle = (i / words.length) * 2 * Math.PI + 0.4;
        const x = Math.cos(angle) * radius;
        const y = Math.sin(angle) * radius * 0.6;
        return (
          <motion.span
            key={word}
            className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 whitespace-nowrap rounded-full border border-border bg-card/80 px-3 py-1 text-base font-medium shadow-sm backdrop-blur"
            initial={{ x, y, opacity: 0, scale: 1 }}
            animate={{
              x: [x, x * 0.8, 0],
              y: [y, y * 0.8, 0],
              opacity: [0, 1, 1, 0],
              scale: [0.9, 1, 0.3],
            }}
            transition={{
              duration: cycle,
              times: [0, 0.35, 1],
              ease: "easeIn",
              repeat: Infinity,
              delay: (i * cycle) / words.length,
            }}
          >
            <TargetLanguageText language={language}>{word}</TargetLanguageText>
          </motion.span>
        );
      })}
    </div>
  );
}

function SrsConclusionScreen({
  view,
  content,
  language,
}: {
  view: OnboardingView;
  content: Extract<OnboardingContent, { type: "Remember" }>;
  language: Language;
}) {
  return (
    <ScreenWrapper screenKey="srs-conclusion">
      <h2
        className="text-3xl md:text-4xl font-bold text-center leading-snug"
        style={{ textWrap: "balance" }}
      >
        {view.title}
      </h2>
      <WordsIntoMemory words={content.words} language={language} />
    </ScreenWrapper>
  );
}

// Icons are platform presentation; the choices, order and labels come from Rust.
function choiceIcon(choice: OnboardingChoice): LucideIcon {
  switch (choice.type) {
    case "HeardAbout": {
      const icons: Record<HeardAbout, LucideIcon> = {
        FriendsOrFamily: Users,
        Reddit: Globe,
        TikTok: Video,
        GoogleSearch: Search,
        YouTube: Youtube,
        TwitterX: Globe,
        Other: HelpCircle,
      };
      return icons[choice.value];
    }
    case "Motivation": {
      const icons: Record<Motivation, LucideIcon> = {
        SpendTimeProductively: Clock,
        SupportMyEducation: GraduationCap,
        ConnectWithPeople: Heart,
        BoostMyCareer: Briefcase,
        PrepareForTravel: Plane,
        JustForFun: Sparkles,
        Other: HelpCircle,
      };
      return icons[choice.value];
    }
    case "Experience": {
      const icons: Record<ExperienceLevel, LucideIcon> = {
        New: SignalZero,
        CommonWords: SignalLow,
        BasicConversations: SignalMedium,
        VariousTopics: SignalHigh,
        MostTopics: Signal,
      };
      return icons[choice.value];
    }
    case "StudyGoal": {
      const icons: Record<DailyReviewTarget, LucideIcon> = {
        Casual: SignalLow,
        Regular: SignalMedium,
        Serious: SignalHigh,
        Intense: Signal,
      };
      return icons[choice.value];
    }
  }
}

function NotificationScreen({
  view,
  content,
  onDone,
}: {
  view: OnboardingView;
  content: Extract<OnboardingContent, { type: "Notifications" }>;
  onDone: () => void;
}) {
  const { subscribe, isLoading } = useOneSignalNotifications();
  const [requested, setRequested] = useState(false);
  const handleEnable = async () => {
    setRequested(true);
    await subscribe();
    onDone();
  };
  return (
    <ScreenWrapper screenKey="notifications">
      <div className="flex flex-col items-center gap-2">
        <div className="rounded-full bg-primary/10 p-4">
          <Bell className="h-10 w-10 text-primary" />
        </div>
      </div>
      <h2
        className="text-3xl md:text-4xl font-bold text-center leading-snug"
        style={{ textWrap: "balance" }}
      >
        {view.title}
      </h2>
      <p className="text-muted-foreground text-center text-base">
        {content.body}
      </p>
      <div className="flex flex-col gap-3 w-full">
        <Button
          size="lg"
          onClick={handleEnable}
          disabled={isLoading || requested}
          className="w-full"
        >
          {isLoading ? content.enabling_label : content.enable_label}
        </Button>
        <Button
          variant="ghost"
          size="lg"
          onClick={onDone}
          className="w-full text-muted-foreground"
        >
          {content.skip_label}
        </Button>
      </div>
    </ScreenWrapper>
  );
}

function ScreenContent({
  view,
  send,
  language,
}: {
  view: OnboardingView;
  send: (event: OnboardingEvent) => void;
  language: Language;
}) {
  const content = view.content;
  switch (content.type) {
    case "Choices":
      return <ChoicesScreen view={view} content={content} send={send} />;
    case "Achievements":
      return (
        <ScreenWrapper screenKey={view.step}>
          <h2
            className="text-3xl md:text-4xl font-bold text-center"
            style={{ textWrap: "balance" }}
          >
            {view.title}
          </h2>
          <div className="flex flex-col gap-4 w-full">
            {content.items.map((item, i) => (
              <motion.div
                key={item.text}
                initial={{ opacity: 0, x: 24 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: 0.15 + i * 0.15, duration: 0.35 }}
              >
                <Card className="p-5 flex-row items-center gap-4">
                  <span className="text-4xl">{item.emoji}</span>
                  <span className="text-lg font-medium">{item.text}</span>
                </Card>
              </motion.div>
            ))}
          </div>
        </ScreenWrapper>
      );
    case "Studies":
      return <SrsTeaserScreen view={view} content={content} />;
    case "Review":
      return <SrsIntroScreen view={view} content={content} />;
    case "Remember":
      return (
        <SrsConclusionScreen
          view={view}
          content={content}
          language={language}
        />
      );
    case "Notifications":
      return (
        <NotificationScreen
          view={view}
          content={content}
          onDone={() => send({ type: "NotificationsDone" })}
        />
      );
    case "Ready":
      return <ReadyScreen view={view} content={content} send={send} />;
  }
}

export function OnboardingFlow({
  purpose,
  targetLanguage,
  hasHeardAbout,
  onHeardAbout,
  onComplete,
  onBack,
}: OnboardingFlowProps) {
  const { isSupported, isSubscribed, isInitialized } =
    useOneSignalNotifications();
  const offerNotifications = isInitialized && isSupported && !isSubscribed;
  const [state, setState] = useState(() =>
    onboarding_start(
      targetLanguage,
      hasHeardAbout,
      offerNotifications,
      purpose,
    ),
  );
  const currentState = useRef(state);
  useLayoutEffect(() => {
    currentState.current = state;
  }, [state]);
  const [previousOffer, setPreviousOffer] = useState(offerNotifications);
  // Adjust this component's state during render, before its children render.
  // Rust only changes the itinerary on step one and preserves its answer.
  if (previousOffer !== offerNotifications) {
    setPreviousOffer(offerNotifications);
    setState(
      onboarding_reduce(state, {
        type: "RefreshNotificationOffer",
        offer: offerNotifications,
      }).state,
    );
  }
  const view = onboarding_view(state);
  function send(event: OnboardingEvent) {
    const transition = onboarding_reduce(currentState.current, event);
    currentState.current = transition.state;
    setState(transition.state);
    for (const effect of transition.effects) {
      switch (effect.type) {
        case "SaveHeardAbout":
          onHeardAbout(effect.value);
          break;
        case "Complete":
          onComplete(effect.selections);
          break;
        case "Exit":
          onBack();
          break;
      }
    }
  }
  return (
    <div className="w-full flex flex-col items-center px-4 pt-8 pb-28 overflow-x-clip">
      <div className="w-full max-w-lg mx-auto mb-6">
        <Progress
          value={view.progress_percent}
          aria-label={view.progress_label}
        />
      </div>
      <div className="w-full max-w-lg mb-4">
        <Button
          variant="ghost"
          size="sm"
          onClick={() => send({ type: "Back" })}
          className="text-muted-foreground"
        >
          <ArrowLeft className="h-4 w-4 mr-1" />
          {view.back_label}
        </Button>
      </div>
      <ScreenContent
        key={view.step}
        view={view}
        send={send}
        language={targetLanguage}
      />
      {view.primary && (
        <div className="fixed inset-x-0 bottom-0 z-20 bg-background px-4 py-4 pb-[max(1rem,env(safe-area-inset-bottom))] md:pointer-events-none md:bg-transparent">
          <div className="mx-auto flex w-full max-w-lg md:justify-end">
            <Button
              size="lg"
              disabled={!view.primary.enabled}
              onClick={() => send({ type: "Next" })}
              className="w-full px-8 text-base md:pointer-events-auto md:w-auto"
            >
              {view.primary.label}
              {view.primary.show_arrow && (
                <ArrowRight className="h-4 w-4 ml-2" />
              )}
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
