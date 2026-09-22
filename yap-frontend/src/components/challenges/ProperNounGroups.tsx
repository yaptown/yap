import type {
  Language,
  ProperNounGroup,
} from "../../../../yap-frontend-rs/pkg/yap_frontend_rs";
import { TargetLanguageText } from "../TargetLanguageText";

export function ProperNounGroups({
  groups,
  targetLanguage,
}: {
  groups: ProperNounGroup[];
  targetLanguage: Language;
}) {
  if (groups.length === 0) return null;

  return (
    <div className="text-sm space-y-1">
      {groups.map((group, index) => (
        <div key={index} className="p-3 border border-card/50 bg-card/30 rounded-md">
          {group.spans.map((span, spanIndex) =>
            span.target_language ? (
              <span key={spanIndex} className="font-semibold">
                <TargetLanguageText language={targetLanguage}>
                  {span.text}
                </TargetLanguageText>
              </span>
            ) : (
              <span key={spanIndex} className="text-muted-foreground">
                {span.text}
              </span>
            ),
          )}
        </div>
      ))}
    </div>
  );
}
