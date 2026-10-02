// Shared, WASM-value-free presentation for a translation grade — rendered by
// BOTH the app's TranslationChallenge and the yap-mcp widget's TranslationCard.
// The verdict layout (and the colored sentence, the class that produced the
// `<word>`-tag divergence bug) lives here once, so the two containers can never
// drift again. Everything imported here is `import type` from the pkg or a
// wasm-free leaf; the widget's wasm-guard build enforces that.
import type {
  Language,
  TranslationWordView,
  VerdictHeadline as VerdictHeadlineData,
  VerdictTone,
} from "../../../../yap-frontend-rs/pkg";
import type { ReactNode } from "react";
import { cn } from "@/lib/utils";
import { Skeleton } from "@/components/ui/skeleton";
import { TargetLanguageText } from "@/components/TargetLanguageText";
import { FeedbackDisplay } from "@/components/FeedbackDisplay";
import { Flag } from "lucide-react";

/** The normalized data a graded verdict renders from — same shape both sides. */
export interface TranslationVerdictData {
  correctTranslation: string;
  isPerfect: boolean;
  encouragement: string | null;
  explanation: string | null;
  autogradingError: string | null;
  /** Absent in the widget, whose server result carries no headline. */
  headline?: VerdictHeadlineData;
  correctLabel?: string;
}

interface ChallengeSentenceProps {
  words: TranslationWordView[];
  targetLanguage: Language;
  onWordTap?: (index: number) => void;
}

const tintClasses = {
  Neutral: "",
  Perfect: "text-positive-foreground",
  Remembered: "text-positive-foreground",
  Tapped: "text-caution-foreground",
  Forgot: "text-negative-foreground",
};

export function ChallengeSentence({
  words,
  targetLanguage,
  onWordTap,
}: ChallengeSentenceProps) {
  return (
    <h2 className="text-3xl font-bold leading-tight">
      {words.map((word, i) => {
        const colorClass = tintClasses[word.tint];
        const interactive = word.tappable && !!onWordTap;

        return (
          <span key={i}>
            <span
              className={cn(
                colorClass,
                interactive
                  ? "cursor-pointer underline-offset-3 underline decoration-dotted hover:decoration-solid hover:decoration-3 transition-transform hover:scale-105 inline-block"
                  : "",
              )}
              onClick={() => {
                if (interactive) {
                  onWordTap(i);
                }
              }}
            >
              <TargetLanguageText language={targetLanguage}>
                {word.text}
              </TargetLanguageText>
            </span>
            {word.whitespace}
          </span>
        );
      })}
    </h2>
  );
}

/** A quiet uppercase caption over one block of a challenge card. */
export function SectionLabel({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <p
      className={cn(
        "text-xs font-semibold uppercase tracking-[0.12em] text-muted-foreground",
        className,
      )}
    >
      {children}
    </p>
  );
}

const toneLine: Record<VerdictTone, string> = {
  Perfect: "bg-positive",
  Almost: "bg-caution",
  Wrong: "bg-negative",
};

const toneMark: Record<VerdictTone, string> = {
  Perfect: "bg-positive-surface text-positive-foreground",
  Almost: "bg-caution-surface text-caution-foreground",
  Wrong: "bg-negative-surface text-negative-foreground",
};

/**
 * The learner's answer on an underline: accent while the field inside has
 * focus, then the verdict's tint once graded.
 */
export function AnswerLine({
  label,
  tone,
  children,
}: {
  label: string;
  tone?: VerdictTone;
  children: ReactNode;
}) {
  return (
    <div className="group space-y-2 text-left">
      <SectionLabel>{label}</SectionLabel>
      <div
        className={cn(
          "text-xl",
          tone === "Perfect" && "text-positive-foreground",
        )}
      >
        {children}
      </div>
      <div
        className={cn(
          "rounded-full transition-colors",
          tone
            ? cn("h-0.5", toneLine[tone])
            : "h-px bg-muted-foreground/40 group-focus-within:h-0.5 group-focus-within:bg-primary",
        )}
      />
    </div>
  );
}

/** "Nailed it!" beside a mark in the verdict's tint. */
export function VerdictHeadline({
  headline,
}: {
  headline: VerdictHeadlineData;
}) {
  return (
    <div className="flex items-center gap-3 animate-feedback-in">
      <span
        aria-hidden
        className={cn(
          "flex h-8 w-8 items-center justify-center rounded-full font-bold",
          toneMark[headline.tone],
        )}
      >
        {{ Perfect: "✓", Almost: "~", Wrong: "✗" }[headline.tone]}
      </span>
      <p className="text-2xl font-bold">{headline.text}</p>
    </div>
  );
}

/**
 * Ends a challenge's scrolling content: pushed to the bottom of the column
 * so it rests just above the sticky actions, which pass over it on scroll.
 */
export function ReportIssueLink({
  label,
  onClick,
}: {
  label: string;
  onClick: () => void;
}) {
  return (
    <div className="mt-auto flex justify-end">
      <button
        type="button"
        onClick={onClick}
        className="inline-flex items-center gap-2 py-2 text-sm text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
      >
        <Flag className="h-4 w-4" />
        {label}
      </button>
    </div>
  );
}

/**
 * The graded half of a sentence challenge: how it went, the reference answer
 * and the autograder's notes. While grading, skeleton bars hold the
 * headline's and feedback's places so nothing jumps when they arrive; the
 * reference answer shows from the start.
 */
export function SentenceVerdict({
  grading,
  headline,
  correctLabel,
  correct,
  encouragement,
  explanation,
  autogradingError,
  isPerfect,
  targetLanguage,
}: {
  grading: boolean;
  headline?: VerdictHeadlineData;
  correctLabel: string;
  correct: ReactNode;
  encouragement?: string | null;
  explanation?: string | null;
  autogradingError?: string | null;
  isPerfect: boolean;
  targetLanguage: Language;
}) {
  const tone: VerdictTone = headline?.tone ?? (isPerfect ? "Perfect" : "Wrong");
  return (
    <div className="space-y-5 text-left animate-feedback-in">
      {grading ? (
        <Skeleton className="h-3 w-2/5" />
      ) : (
        headline && <VerdictHeadline headline={headline} />
      )}
      {(grading || !isPerfect) && (
        <div className="space-y-1.5">
          <SectionLabel>{correctLabel}</SectionLabel>
          <p className="text-xl">{correct}</p>
        </div>
      )}
      {grading ? (
        <div className="space-y-2.5">
          <Skeleton className="h-3 w-1/3" />
          <Skeleton className="h-3 w-11/12" />
          <Skeleton className="h-3 w-2/3" />
        </div>
      ) : (
        <>
          {autogradingError && (
            <p className="text-sm text-caution-foreground">
              Your submission could not be graded automatically. Please grade
              the words manually below.
            </p>
          )}
          <FeedbackDisplay
            encouragement={encouragement ?? undefined}
            explanation={explanation ?? undefined}
            tone={tone}
            targetLanguage={targetLanguage}
          />
        </>
      )}
    </div>
  );
}

/** The graded verdict, for the app's translation challenge and the widget. */
export function TranslationVerdict({
  verdict,
  targetLanguage,
}: {
  verdict: TranslationVerdictData;
  targetLanguage: Language;
}) {
  return (
    <SentenceVerdict
      grading={false}
      headline={verdict.headline}
      correctLabel={verdict.correctLabel ?? "Correct translation"}
      correct={verdict.correctTranslation}
      encouragement={verdict.encouragement}
      explanation={verdict.explanation}
      autogradingError={verdict.autogradingError}
      isPerfect={verdict.isPerfect}
      targetLanguage={targetLanguage}
    />
  );
}
