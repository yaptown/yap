import { t } from "@/lib/i18n";
import { Navigate, useNavigate, useOutletContext } from "react-router-dom";
import { LogOut, User } from "lucide-react";
import { supabase } from "@/lib/supabase";
import type { AppContextType } from "@/app/context";
import { TopPageLayout } from "@/components/TopPageLayout";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { NotificationSettings } from "@/browse/notification-settings";
import { PasskeySettings } from "@/browse/passkey-settings";
import { DeleteAccountSettings } from "@/browse/delete-account-settings";

// Everything about the signed-in account, in one place. Like the iOS
// SettingsScreen, the account card comes first and ends with sign-out and
// account deletion.
export function SettingsPage() {
  const { userInfo } = useOutletContext<AppContextType>();
  const navigate = useNavigate();

  // Signing out (or deleting the account) from here lands back on the start.
  if (!userInfo) {
    return <Navigate to="/" replace />;
  }

  const shown = userInfo.displayName || userInfo.email;

  return (
    <TopPageLayout
      userInfo={userInfo}
      headerProps={{
        backButton: {
          label: t("common-home"),
          onBack: () => navigate("/home"),
        },
      }}
    >
      <main className="flex flex-col gap-4 py-4">
        <h1 className="text-2xl font-bold">{t("common-settings")}</h1>
        <Card>
          <CardHeader>
            <CardTitle>{t("settings-screen-account")}</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-2">
            <div className="flex items-center gap-3 pb-2">
              <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-full bg-primary text-lg font-semibold text-primary-foreground">
                {shown.slice(0, 1).toUpperCase()}
              </div>
              <div className="min-w-0">
                <p className="truncate font-medium">{shown}</p>
                {shown !== userInfo.email && (
                  <p className="truncate text-sm text-muted-foreground">
                    {userInfo.email}
                  </p>
                )}
              </div>
            </div>
            <Button
              variant="ghost"
              className="justify-start"
              onClick={() => navigate(`/user/id/${userInfo.id}`)}
            >
              <User />
              {t("home-profile")}
            </Button>
            <Button
              variant="ghost"
              className="justify-start"
              onClick={() => supabase.auth.signOut()}
            >
              <LogOut />
              {t("header-sign-out")}
            </Button>
            <DeleteAccountSettings />
          </CardContent>
        </Card>
        <NotificationSettings />
        <PasskeySettings />
      </main>
    </TopPageLayout>
  );
}
