import type { ComponentProps, ReactNode } from "react";
import { Header } from "@/components/header";
import { SiteFooter } from "@/components/site-footer";
import type { UserInfo } from "@/app/context";

interface TopPageLayoutProps {
  userInfo: UserInfo | undefined;
  children: ReactNode;
  headerProps?: Omit<ComponentProps<typeof Header>, "userInfo">;
  /** Off where the page is a task to stay in, like a review session. */
  footer?: boolean;
}

export function TopPageLayout({
  userInfo,
  children,
  headerProps = {},
  footer = true,
}: TopPageLayoutProps) {
  return (
    <>
      <div className="flex flex-col py-2" style={{ minHeight: "calc(100dvh)" }}>
        <Header userInfo={userInfo} {...headerProps} />
        {children}
      </div>
      {footer && <SiteFooter className="mt-8" />}
    </>
  );
}
