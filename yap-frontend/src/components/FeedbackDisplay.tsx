import { useMemo } from "react";
import Markdown, { type Components } from "react-markdown";
import rehypeRaw from "rehype-raw";
import type { Language, VerdictTone } from "../../../yap-frontend-rs/pkg";
import { TargetLanguageText } from "./TargetLanguageText";

interface FeedbackDisplayProps {
  encouragement?: string;
  explanation?: string;
  tone: VerdictTone;
  label?: string;
  targetLanguage: Language;
}

export function FeedbackDisplay({
  encouragement,
  explanation,
  tone,
  label = "Feedback",
  targetLanguage,
}: FeedbackDisplayProps) {
  // Custom tags emitted by the autograder LLM. Keys are lowercased because the
  // HTML parser normalizes tag names. Cast to Components to bypass the
  // intrinsic-element type constraint.
  const components = useMemo(
    () =>
      ({
        word: ({ children }: { children?: React.ReactNode }) => (
          <strong>
            <TargetLanguageText language={targetLanguage}>
              {children}
            </TargetLanguageText>
          </strong>
        ),
      }) as Components,
    [targetLanguage],
  );

  if (!encouragement && !explanation) {
    return null;
  }

  return (
    <div className="space-y-1.5 animate-fade-in">
      <p
        className={`text-xs font-semibold uppercase tracking-[0.12em] ${toneText[tone]}`}
      >
        {label}
      </p>
      <div className="space-y-3 text-lg leading-relaxed">
        {encouragement && (
          <Markdown rehypePlugins={[rehypeRaw]} components={components}>
            {encouragement}
          </Markdown>
        )}
        {explanation && (
          <Markdown rehypePlugins={[rehypeRaw]} components={components}>
            {explanation}
          </Markdown>
        )}
      </div>
    </div>
  );
}

const toneText: Record<VerdictTone, string> = {
  Perfect: "text-positive-foreground",
  Almost: "text-caution-foreground",
  Wrong: "text-negative-foreground",
};
