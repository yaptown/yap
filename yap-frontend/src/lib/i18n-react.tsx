import { Fragment, type ReactNode } from "react";
import type { FluentVariable } from "@fluent/bundle";
import { t } from "./i18n";

/** Translate a whole sentence while keeping links and styled runs as React nodes. */
export function rich(
  id: string,
  slots: Record<string, ReactNode>,
  args?: Record<string, FluentVariable>,
): ReactNode {
  const markers = Object.fromEntries(
    Object.keys(slots).map((name) => [name, `￰${name}￱`]),
  );
  return t(id, { ...args, ...markers })
    .split(/(￰[\w-]+￱)/g)
    .map((text, index) =>
      text.startsWith("￰") ? (
        <Fragment key={index}>{slots[text.slice(1, -1)]}</Fragment>
      ) : (
        text
      ),
    );
}
