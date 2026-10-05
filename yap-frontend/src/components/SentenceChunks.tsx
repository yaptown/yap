import { Fragment, type ReactNode } from "react";
import type { SentenceChunk } from "../../../yap-frontend-rs/pkg";

/** Rust owns the legal boundaries; hosts only arrange the unbreakable runs. */
export function SentenceChunks({
  chunks,
  children,
}: {
  chunks: SentenceChunk[];
  children: (index: number, last: boolean) => ReactNode;
}) {
  return chunks.map((chunk, index) => (
    <Fragment key={index}>
      {/* inline-flex + max-w-full: a run never outgrows the line; only
          shrinkable items inside it (dictation fields) give way. */}
      <span className="inline-flex max-w-full items-baseline whitespace-nowrap">
        {chunk.indices.map((token, offset) => (
          <Fragment key={token}>
            {children(token, offset === chunk.indices.length - 1)}
          </Fragment>
        ))}
      </span>
      {chunk.trailing_whitespace}
      {index + 1 < chunks.length && <wbr />}
    </Fragment>
  ));
}
