import { useMemo } from "react";
import Markdown, { type Components } from "react-markdown";
import rehypeRaw from "rehype-raw";
import type { Language } from "../../../yap-frontend-rs/pkg";
import { TargetLanguageText } from "./TargetLanguageText";

interface FeedbackDisplayProps {
  encouragement?: string;
  explanation?: string;
  perfect?: boolean;
  targetLanguage: Language;
}

export function FeedbackDisplay({
  encouragement,
  explanation,
  perfect = false,
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
    <div className="rounded-lg p-4 border bg-info-surface border-info-border">
      <p className="text-sm font-medium mb-1 text-info-foreground">
        Feedback:
      </p>
      <div className="space-y-3">
        {encouragement && (
          <div className="animate-fade-in px-3 py-2 rounded-md bg-positive-surface border-l-2 border-positive-border">
            <div className="flex items-start gap-2">
              <span className="text-lg leading-none">
                {perfect ? "🎉" : "☀️"}
              </span>
              <div className="flex-1 font-medium text-positive-foreground">
                <Markdown rehypePlugins={[rehypeRaw]} components={components}>
                  {encouragement}
                </Markdown>
              </div>
            </div>
          </div>
        )}

        {explanation && (
          <div className="animate-fade-in-delay-2">
            <Markdown rehypePlugins={[rehypeRaw]} components={components}>
              {explanation}
            </Markdown>
          </div>
        )}
      </div>
    </div>
  );
}
