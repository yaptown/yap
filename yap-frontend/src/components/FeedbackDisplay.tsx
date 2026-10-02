import { useMemo } from "react";
import Markdown, { type Components } from "react-markdown";
import rehypeRaw from "rehype-raw";
import type { Language, VerdictTone } from "../../../yap-frontend-rs/pkg";
import { TargetLanguageText } from "./TargetLanguageText";

interface FeedbackDisplayProps {
  encouragement?: string;
  explanation?: string;
  tone: VerdictTone;
  targetLanguage: Language;
}

export function FeedbackDisplay({
  encouragement,
  explanation,
  tone,
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

  // The encouragement heads the feedback in the verdict's tint.
  return (
    <div className="space-y-3 animate-fade-in">
      {encouragement && (
        <div className={`font-semibold ${toneText[tone]}`}>
          <Markdown rehypePlugins={[rehypeRaw]} components={components}>
            {encouragement}
          </Markdown>
        </div>
      )}
      {explanation && (
        <Markdown rehypePlugins={[rehypeRaw]} components={components}>
          {explanation}
        </Markdown>
      )}
    </div>
  );
}

const toneText: Record<VerdictTone, string> = {
  Perfect: "text-positive-foreground",
  Almost: "text-caution-foreground",
  Wrong: "text-negative-foreground",
};
