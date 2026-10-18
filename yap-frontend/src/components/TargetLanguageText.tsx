import { Fragment, type ReactNode } from "react";
import type { Language, RubySegment } from "../../../yap-frontend-rs/pkg";
import { languageToLangAttr } from "@/lib/utils";

export function TargetLanguageText({
  children,
  language,
}: {
  children: ReactNode;
  language: Language;
}) {
  return <span lang={languageToLangAttr(language)}>{children}</span>;
}

/** `children` with readings set over it; Rust sends no segments when there are
 * none to show, and their bases spell exactly the text they replace. */
export function Ruby({
  segments,
  children,
}: {
  segments: RubySegment[];
  children: ReactNode;
}) {
  if (segments.length === 0) return children;
  return segments.map(({ base, ruby }, index) => (
    <Fragment key={index}>
      {ruby ? (
        // Browsers don't grow the line box for the reading, so the tall
        // line-height makes room for it rather than letting it overlap.
        <ruby className="leading-[2.4]">
          {base}
          <rt className="font-medium leading-none text-muted-foreground">
            {ruby}
          </rt>
        </ruby>
      ) : (
        base
      )}
    </Fragment>
  ));
}
