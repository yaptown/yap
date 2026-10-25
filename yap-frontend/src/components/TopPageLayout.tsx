import type { ComponentProps, ReactNode } from "react";
import { Header } from "@/components/header";
import type { UserInfo } from "@/app/context";

interface TopPageLayoutProps {
  userInfo: UserInfo | undefined;
  children: ReactNode;
  headerProps?: Omit<ComponentProps<typeof Header>, "userInfo">;
}

export function TopPageLayout({
  userInfo,
  children,
  headerProps = {},
}: TopPageLayoutProps) {
  return (
    <div className="flex flex-col py-2" style={{ minHeight: "calc(100dvh)" }}>
      <Header userInfo={userInfo} {...headerProps} />
      {children}
    </div>
  );
}
