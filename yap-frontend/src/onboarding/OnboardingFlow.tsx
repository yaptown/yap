import {
  onboarding_start,
  onboarding_reduce,
  onboarding_view,
} from "../../../yap-frontend-rs/pkg";
import { useState, useRef, useLayoutEffect } from "react";
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

function calcDecay(startY: number, endY: number, width: number) {
  return -Math.log(endY / startY) / width;
}

function forgetting(startY: number, decay: number, t: number) {
  return startY * Math.exp(-decay * t);
}

function ForgettingCurveChart({
  visibleCount,
  chart,
}: {
  visibleCount: number;
  chart: OnboardingChart;
}) {
  const W = 360;
  const H = 160;
  const topPad = 20;
  const botPad = 10;
  const chartH = H - topPad - botPad;

  const segments = [
    {
      startY: 1.0,
      endY: 0.28,
      widthFrac: 0.22,
      color: "var(--chart-1)",
      fillColor: "color-mix(in oklch, var(--chart-1) 15%, transparent)",
    },
    {
      startY: 1.0,
      endY: 0.38,
      widthFrac: 0.3,
      color: "var(--chart-2)",
      fillColor: "color-mix(in oklch, var(--chart-2) 15%, transparent)",
    },
    {
      startY: 1.0,
      endY: 0.55,
      widthFrac: 0.48,
      color: "var(--chart-3)",
      fillColor: "color-mix(in oklch, var(--chart-3) 15%, transparent)",
    },
  ];

  // Build curve paths
  let offsetX = 0;
  const curves: Array<{
    linePath: string;
    fillPath: string;
    color: string;
    fillColor: string;
    reviewX: number;
    reviewY: number;
  }> = [];

  for (const seg of segments) {
    const segWidth = W * seg.widthFrac;
    const decay = calcDecay(seg.startY, seg.endY, segWidth);
    const startX = offsetX;

    const points: Array<[number, number]> = [];
    const steps = 60;
    for (let s = 0; s <= steps; s++) {
      const t = (s / steps) * segWidth;
      const y = forgetting(seg.startY, decay, t);
      const px = startX + t;
      const py = topPad + (1 - y) * chartH;
      points.push([px, py]);
    }

    const linePath = points
      .map(
        (p, j) => `${j === 0 ? "M" : "L"}${p[0].toFixed(1)},${p[1].toFixed(1)}`,
      )
      .join(" ");
    const fillPath =
      linePath +
      ` L${points[points.length - 1][0].toFixed(1)},${topPad + chartH} L${startX},${topPad + chartH} Z`;

    curves.push({
      linePath,
      fillPath,
      color: seg.color,
      fillColor: seg.fillColor,
      reviewX: startX,
      reviewY: topPad,
    });

    offsetX += segWidth;
  }

  const gridLines = [0.25, 0.5, 0.75].map((frac) => topPad + frac * chartH);

  return (
    <svg
      viewBox={`0 0 ${W} ${H + 20}`}
      className="w-full max-w-sm mx-auto"
      aria-label={chart.accessibility_label}
    >
      {/* Grid lines */}
      {gridLines.map((y, i) => (
        <line
          key={i}
          x1={0}
          y1={y}
          x2={W}
          y2={y}
          stroke="#e5e5e5"
          strokeWidth={1}
          strokeDasharray="4 4"
        />
      ))}

      {/* Curves — only render up to visibleCount */}
      <AnimatePresence>
        {curves.slice(0, visibleCount).map((curve, i) => (
          <motion.g key={i} initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
            <motion.path
              d={curve.fillPath}
              fill={curve.fillColor}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ duration: 0.6 }}
            />
            <motion.path
              d={curve.linePath}
              fill="none"
              stroke={curve.color}
              strokeWidth={2.5}
              strokeLinecap="round"
              initial={{ pathLength: 0 }}
              animate={{ pathLength: 1 }}
              transition={{ duration: 0.8, ease: "easeOut" }}
            />
            {/* Review point dot + dashed line for curves after the first */}
            {i > 0 && (
              <>
                <motion.line
                  x1={curve.reviewX}
                  y1={topPad + chartH}
                  x2={curve.reviewX}
                  y2={curve.reviewY}
                  stroke={curve.color}
                  strokeWidth={1.5}
                  strokeDasharray="4 3"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  transition={{ duration: 0.3 }}
                />
                <motion.circle
                  cx={curve.reviewX}
                  cy={curve.reviewY}
                  r={6}
                  fill="white"
                  stroke={curve.color}
                  strokeWidth={2}
                  initial={{ scale: 0 }}
                  animate={{ scale: 1 }}
                  transition={{
                    delay: 0.2,
                    type: "spring",
                    stiffness: 300,
                    damping: 15,
                  }}
                />
              </>
            )}
          </motion.g>
        ))}
      </AnimatePresence>

      {/* TIME label */}
      <text
        x={W - 5}
        y={H + 12}
        textAnchor="end"
        className="fill-muted-foreground"
        fontSize={11}
        fontWeight={500}
      >
        {chart.x_label}
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
  selected,
  onClick,
  icon: Icon,
}: {
  label: string;
  selected: boolean;
  onClick: () => void;
  icon?: LucideIcon;
}) {
  return (
    <Button
      variant="outline"
      size="lg"
      onClick={onClick}
      className={`w-full text-left justify-between text-base py-6 transition-all ${
        selected
          ? "border-primary bg-primary/10 ring-2 ring-primary/30"
          : "hover:border-primary/40"
      }`}
    >
      <span className="flex items-center gap-3">
        {Icon && <Icon className="h-5 w-5 shrink-0 text-muted-foreground" />}
        {label}
      </span>
      {selected && <Check className="h-4 w-4 text-primary shrink-0" />}
    </Button>
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
        className="w-full relative h-64 select-none overflow-x-clip"
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
            style={{ zIndex: i, top: `${i * 8}px`, height: "200px" }}
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
  const dotColors = ["var(--chart-1)", "var(--chart-2)", "var(--chart-3)"];

  return (
    <ScreenWrapper screenKey="srs-intro">
      <Card className="w-full p-8 flex flex-col items-center gap-4" animate>
        <p className="text-sm font-semibold text-primary tracking-wide uppercase self-start">
          {content.eyebrow}
        </p>
        <h2 className="text-2xl md:text-3xl font-bold text-left self-start leading-snug">
          {view.title}
          <span className="italic text-primary">{content.title_emphasis}</span>
        </h2>

        {/* Review dots legend — only show dots for completed reviews */}
        <div className="flex items-center gap-2 self-start h-5">
          {dotColors
            .slice(0, Math.min(content.demo_reviews, 3))
            .map((color, i) => (
              <motion.span
                key={i}
                className="w-3 h-3 rounded-full"
                style={{ backgroundColor: color }}
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                transition={{ type: "spring", stiffness: 300, damping: 15 }}
              />
            ))}
          {content.review_label && (
            <span className="text-sm text-muted-foreground ml-1">
              {content.review_label}
            </span>
          )}
        </div>

        <AnimatePresence mode="wait">
          {content.learned ? (
            <motion.div
              key="learned"
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              className="flex flex-col items-center gap-3 py-8"
            >
              <div className="text-4xl font-bold flex items-center gap-2">
                <Check className="h-8 w-8 text-primary" />{" "}
                {content.learned_title}
              </div>
              <p className="text-muted-foreground">{content.learned_body}</p>
            </motion.div>
          ) : (
            <motion.div key="chart" exit={{ opacity: 0 }}>
              <ForgettingCurveChart
                visibleCount={Math.min(content.demo_reviews, 3)}
                chart={content.chart}
              />
            </motion.div>
          )}
        </AnimatePresence>
      </Card>
    </ScreenWrapper>
  );
}

// Screen: SRS conclusion with growth chart
function GrowthChart({ chart }: { chart: OnboardingChart }) {
  const w = 280;
  const h = 160;
  const pad = { top: 10, right: 10, bottom: 30, left: 40 };
  const cw = w - pad.left - pad.right;
  const ch = h - pad.top - pad.bottom;

  // Quadratic growth curve: y = t^2
  const points = Array.from({ length: 50 }, (_, i) => {
    const t = i / 49;
    return { x: pad.left + t * cw, y: pad.top + ch - t * t * ch };
  });
  const linePath = points
    .map((p, i) => `${i === 0 ? "M" : "L"}${p.x},${p.y}`)
    .join(" ");
  const areaPath = `${linePath} L${pad.left + cw},${pad.top + ch} L${pad.left},${pad.top + ch} Z`;

  return (
    <svg
      viewBox={`0 0 ${w} ${h}`}
      className="w-full max-w-xs"
      aria-label={chart.accessibility_label}
    >
      {/* Area fill */}
      <motion.path
        d={areaPath}
        fill="var(--chart-1)"
        fillOpacity={0.15}
        initial={{ clipPath: "inset(0 100% 0 0)" }}
        animate={{ clipPath: "inset(0 0% 0 0)" }}
        transition={{ duration: 1.5, ease: "easeOut", delay: 0.3 }}
      />
      {/* Line */}
      <motion.path
        d={linePath}
        fill="none"
        stroke="var(--chart-1)"
        strokeWidth={2.5}
        strokeLinecap="round"
        initial={{ pathLength: 0 }}
        animate={{ pathLength: 1 }}
        transition={{ duration: 1.5, ease: "easeOut", delay: 0.3 }}
      />
      {/* Axes */}
      <line
        x1={pad.left}
        y1={pad.top}
        x2={pad.left}
        y2={pad.top + ch}
        stroke="currentColor"
        strokeOpacity={0.2}
      />
      <line
        x1={pad.left}
        y1={pad.top + ch}
        x2={pad.left + cw}
        y2={pad.top + ch}
        stroke="currentColor"
        strokeOpacity={0.2}
      />
      {/* Labels */}
      <text
        x={pad.left + cw / 2}
        y={h - 4}
        textAnchor="middle"
        className="fill-muted-foreground text-[10px]"
      >
        {chart.x_label}
      </text>
      <text
        x={12}
        y={pad.top + ch / 2}
        textAnchor="middle"
        className="fill-muted-foreground text-[10px]"
        transform={`rotate(-90, 12, ${pad.top + ch / 2})`}
      >
        {chart.y_label}
      </text>
    </svg>
  );
}

function SrsConclusionScreen({
  view,
  content,
}: {
  view: OnboardingView;
  content: Extract<OnboardingContent, { type: "Growth" }>;
}) {
  return (
    <ScreenWrapper screenKey="srs-conclusion">
      <h2
        className="text-3xl md:text-4xl font-bold text-center leading-snug"
        style={{ textWrap: "balance" }}
      >
        {view.title}
      </h2>
      <GrowthChart chart={content.chart} />
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
}: {
  view: OnboardingView;
  send: (event: OnboardingEvent) => void;
}) {
  const content = view.content;
  switch (content.type) {
    case "Studies":
      return <SrsTeaserScreen view={view} content={content} />;
    case "Review":
      return <SrsIntroScreen view={view} content={content} />;
    case "Growth":
      return <SrsConclusionScreen view={view} content={content} />;
    case "Notifications":
      return (
        <NotificationScreen
          view={view}
          content={content}
          onDone={() => send({ type: "NotificationsDone" })}
        />
      );
    default:
      return (
        <ScreenWrapper screenKey={view.step}>
          {content.type === "Ready" && (
            <motion.div
              initial={{ scale: 0 }}
              animate={{ scale: 1 }}
              transition={{ type: "spring", stiffness: 200, damping: 15 }}
              className="text-8xl"
            >
              {content.flag}
            </motion.div>
          )}
          <h2
            className="text-3xl md:text-4xl font-bold text-center"
            style={{ textWrap: "balance" }}
          >
            {view.title}
          </h2>
          {content.type === "Choices" && (
            <>
              <div className="flex flex-col gap-3 w-full">
                {content.options.map((option) => (
                  <OptionButton
                    key={option.choice.value}
                    label={option.label}
                    icon={choiceIcon(option.choice)}
                    selected={option.selected}
                    onClick={() =>
                      send({ type: "Choose", choice: option.choice })
                    }
                  />
                ))}
              </div>
              <AnimatePresence mode="wait">
                {content.hint && (
                  <motion.p
                    key={content.hint}
                    initial={{ opacity: 0, y: 5 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0 }}
                    className="text-base text-primary font-medium text-center"
                  >
                    {content.hint}
                  </motion.p>
                )}
              </AnimatePresence>
            </>
          )}
          {content.type === "Achievements" && (
            <div className="flex flex-col gap-4 w-full">
              {content.items.map((item) => (
                <Card
                  key={item.text}
                  className="p-5 flex items-center gap-4"
                  animate
                >
                  <span className="text-3xl">{item.emoji}</span>
                  <span className="text-lg font-medium">{item.text}</span>
                </Card>
              ))}
            </div>
          )}
          {content.type === "Ready" && (
            <>
              <p
                className="text-muted-foreground text-center text-lg"
                style={{ textWrap: "balance" }}
              >
                {content.body}
              </p>
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
            </>
          )}
        </ScreenWrapper>
      );
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
    onboarding_start(targetLanguage, hasHeardAbout, offerNotifications, purpose),
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
      <ScreenContent key={view.step} view={view} send={send} />
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
