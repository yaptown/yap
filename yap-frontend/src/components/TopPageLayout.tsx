import type { ComponentProps, ReactNode } from "react";
import { Header } from "@/components/header";
import { supabase } from "@/lib/supabase";
import type { UserInfo } from "@/App";

interface TopPageLayoutProps {
  userInfo: UserInfo | undefined;
  children: ReactNode;
  headerProps?: Omit<ComponentProps<typeof Header>, "userInfo" | "onSignOut">;
}

export function TopPageLayout({
  userInfo,
  children,
  headerProps = {},
}: TopPageLayoutProps) {
  return (
    <div className="flex flex-col py-2" style={{ minHeight: "calc(100dvh)" }}>
      <Header
        userInfo={userInfo}
        onSignOut={() => supabase.auth.signOut()}
        {...headerProps}
      />
      {children}
    </div>
  );
}
