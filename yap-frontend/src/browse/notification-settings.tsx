import { t } from "@/lib/i18n";
import { Button } from "@/components/ui/button";
import { useOneSignalNotifications } from "@/hooks/use-onesignal-notifications";
import { Bell } from "lucide-react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { useState, useEffect } from "react";
import { supabase } from "@/lib/supabase";

export function NotificationSettings() {
  const {
    isSupported,
    isSubscribed,
    isLoading,
    isInitialized,
    error,
    subscribe,
    sendTestNotification,
  } = useOneSignalNotifications();

  const [notificationsEnabled, setNotificationsEnabled] = useState(true);
  const [loadingProfile, setLoadingProfile] = useState(true);
  const [savingProfile, setSavingProfile] = useState(false);

  useEffect(() => {
    async function loadProfile() {
      try {
        const {
          data: { user },
        } = await supabase.auth.getUser();
        if (!user) return;

        const { data: profile, error } = await supabase
          .from("profiles")
          .select("notifications_enabled")
          .eq("id", user.id)
          .single();

        if (error && error.code !== "PGRST116") {
          // PGRST116 is "not found"
          console.error("Error loading profile:", error);
        } else if (profile) {
          setNotificationsEnabled(profile.notifications_enabled ?? true);
        }
      } catch (err) {
        console.error("Error loading profile:", err);
      } finally {
        setLoadingProfile(false);
      }
    }

    loadProfile();
  }, []);

  const handleToggleChange = async (checked: boolean) => {
    setSavingProfile(true);
    setNotificationsEnabled(checked);

    try {
      const {
        data: { user },
      } = await supabase.auth.getUser();
      if (!user) return;

      const { error } = await supabase.from("profiles").upsert({
        id: user.id,
        notifications_enabled: checked,
      });

      if (error) {
        console.error("Error updating profile:", error);
        setNotificationsEnabled(!checked);
      }
    } catch (err) {
      console.error("Error updating profile:", err);
      setNotificationsEnabled(!checked);
    } finally {
      setSavingProfile(false);
    }
  };

  const renderContent = () => {
    if (!isInitialized || loadingProfile) {
      return (
        <div className="text-center">
          <p className="text-muted-foreground">{t("common-loading")}</p>
        </div>
      );
    }

    return (
      <div className="space-y-6">
        {/* Global notification toggle */}
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label htmlFor="notifications-enabled">
              {t("notification-settings-notifications")}
            </Label>
            <p className="text-sm text-muted-foreground">
              {t(
                "notification-settings-receive-push-notifications-when-cards-are-due",
              )}
            </p>
          </div>
          <Switch
            id="notifications-enabled"
            checked={notificationsEnabled}
            onCheckedChange={handleToggleChange}
            disabled={savingProfile}
          />
        </div>

        {/* Push notification setup - only show if globally enabled */}
        {notificationsEnabled && (
          <div className="pt-4 border-t">
            {!isSupported ? (
              <div className="text-center">
                <p className="text-muted-foreground">
                  {t(
                    "notification-settings-push-notifications-are-not-supported-in-your-browser",
                  )}
                </p>
              </div>
            ) : !isSubscribed ? (
              <div className="text-center space-y-4">
                <Bell className="h-12 w-12 mx-auto text-muted-foreground" />
                <p className="text-muted-foreground">
                  {t(
                    "notification-settings-enable-browser-notifications-to-receive-push-alerts",
                  )}
                </p>
                <Button onClick={subscribe} disabled={isLoading}>
                  {isLoading
                    ? t("onboarding-enabling")
                    : t("notification-settings-enable-browser-notifications")}
                </Button>
              </div>
            ) : (
              <div className="space-y-4">
                <div className="flex items-center gap-2 text-positive-foreground">
                  <Bell className="h-5 w-5" />
                  <span className="text-sm font-medium">
                    {t("notification-settings-browser-notifications-enabled")}
                  </span>
                </div>

                <Button
                  onClick={sendTestNotification}
                  disabled={isLoading}
                  variant="outline"
                  className="w-full"
                >
                  {isLoading
                    ? t("notification-settings-sending")
                    : t("notification-settings-send-test-notification")}
                </Button>

                <p className="text-sm text-muted-foreground">
                  {t(
                    "notification-settings-to-disable-browser-notifications-use-your-browser-s-notification",
                  )}
                </p>
              </div>
            )}
          </div>
        )}

        {error && <p className="text-sm text-destructive">{error}</p>}
      </div>
    );
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("notification-settings-push-notifications")}</CardTitle>
        <CardDescription>
          {t("notification-settings-get-reminded-when-it-s-time-to-study-your")}
        </CardDescription>
      </CardHeader>
      <CardContent>{renderContent()}</CardContent>
    </Card>
  );
}
