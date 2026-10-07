import type * as React from "react";

declare module "react" {
  namespace JSX {
    interface IntrinsicElements {
      install: React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement> & {
          installurl?: string;
          manifestid?: string;
        },
        HTMLElement
      >;
    }
  }
}
