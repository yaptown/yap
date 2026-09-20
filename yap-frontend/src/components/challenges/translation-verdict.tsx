// Shared, WASM-value-free presentation for a translation grade — rendered by
// BOTH the app's TranslationChallenge and the yap-mcp widget's TranslationCard.
// The verdict layout (and the colored sentence, the class that produced the
// `<word>`-tag divergence bug) lives here once, so the two containers can never
// drift again. Everything imported here is `import type` from the pkg or a
// wasm-free leaf; the widget's wasm-guard build enforces that.
import type { Language, TranslationWordView } from "../../../../yap-frontend-rs/pkg";
import { cn } from "@/lib/utils";
import { Skeleton } from "@/components/ui/skeleton";
import { TargetLanguageText } from "@/components/TargetLanguageText";
import { FeedbackDisplay } from "@/components/FeedbackDisplay";

/** The normalized data a graded verdict renders from — same shape both sides. */
export interface TranslationVerdictData {
  userTranslation: string;
  correctTranslation: string;
  isPerfect: boolean;
  encouragement: string | null;
  explanation: string | null;
  autogradingError: string | null;
  submissionLabel?: string;
  correctLabel?: string;
}

interface ChallengeSentenceProps {
  words: TranslationWordView[];
  targetLanguage: Language;
  onWordTap?: (index: number) => void;
}

const tintClasses = {
  Neutral: "",
  Perfect: "text-green-600 dark:text-green-400",
  Remembered: "text-green-600 dark:text-green-400",
  Tapped: "text-yellow-500 dark:text-yellow-400",
  Forgot: "text-red-600 dark:text-red-400",
};

export function ChallengeSentence({ words, targetLanguage, onWordTap }: ChallengeSentenceProps) {
  return (
    <h2 className="text-2xl font-semibold">
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

export function YourTranslation({ userTranslation, label = "Your translation:" }: { userTranslation: string; label?: string }) {
  return (
    <div className="rounded-lg p-4 border">
      <p className="text-sm font-medium mb-1">{label}</p>
      <p className="text-lg font-medium">{userTranslation}</p>
    </div>
  );
}

export function CorrectTranslation({ sentence, label = "Correct translation:" }: { sentence: string; label?: string }) {
  return (
    <div className="bg-green-500/10 rounded-lg p-4 border border-green-500/20">
      <p className="text-sm font-medium text-green-600 dark:text-green-400 mb-1">
        {label}
      </p>
      <p className="text-lg font-medium">{sentence}</p>
    </div>
  );
}

export function FeedbackSkeleton() {
  return (
    <div className="space-y-4 mt-4 animate-feedback-in">
      <div className="space-y-3">
        <Skeleton className="h-4 w-3/4" />
        <Skeleton className="h-16 w-full" />
        <Skeleton className="h-4 w-1/2" />
      </div>
    </div>
  );
}

export function AutogradeError() {
  return (
    <div className="rounded-lg p-4 border bg-yellow-500/10 border-yellow-500/20">
      <p className="text-sm font-medium mb-1 text-yellow-600 dark:text-yellow-400">
        Your submission could not be graded automatically. Please grade the
        words manually below.
      </p>
    </div>
  );
}

/**
 * The graded-verdict feedback stack (your/correct translation, autograde
 * fallback notice, LLM feedback). The app renders its manual grade-adjust UI
 * (PhraseStatuses) as a sibling after this; the widget renders nothing after.
 * The DOM structure matches the app's prior inline markup exactly.
 */
export function TranslationVerdict({
  verdict,
  targetLanguage,
}: {
  verdict: TranslationVerdictData;
  targetLanguage: Language;
}) {
  if (verdict.isPerfect) {
    return (
      <div className="space-y-2">
        <CorrectTranslation sentence={verdict.correctTranslation} label={verdict.correctLabel} />
        <FeedbackDisplay
          encouragement={verdict.encouragement ?? undefined}
          explanation={verdict.explanation ?? undefined}
          perfect
          targetLanguage={targetLanguage}
        />
      </div>
    );
  }

  return (
    <>
      <div className="space-y-2">
        <YourTranslation userTranslation={verdict.userTranslation} label={verdict.submissionLabel} />
        <CorrectTranslation sentence={verdict.correctTranslation} label={verdict.correctLabel} />
      </div>

      {verdict.autogradingError && <AutogradeError />}

      <FeedbackDisplay
        encouragement={verdict.encouragement ?? undefined}
        explanation={verdict.explanation ?? undefined}
        targetLanguage={targetLanguage}
      />
    </>
  );
}
