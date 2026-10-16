import { t } from "@/lib/i18n";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Bell, Sparkles } from "lucide-react";
import { useOneSignalNotifications } from "@/hooks/use-onesignal-notifications";
import { useIsInstalled } from "@/hooks/use-is-installed";
import { InstallAppButton } from "@/components/InstallAppButton";
import { Card } from "@/components/ui/card";
import { LANGUAGES } from "@/lib/languages";
import { IS_IOS } from "@/lib/ios-app";
import type { Language } from "../../../yap-frontend-rs/pkg";

interface EngagementPromptsProps {
  language: Language;
}

export function EngagementPrompts({ language }: EngagementPromptsProps) {
  const {
    isSupported,
    isSubscribed,
    isLoading: isNotificationLoading,
    isInitialized,
    subscribe,
  } = useOneSignalNotifications();

  const { isInstalled, isLoading: isInstalledLoading } = useIsInstalled();
  const [promptsDismissed, setPromptsDismissed] = useState(() => {
    const dismissalCount = parseInt(
      localStorage.getItem("engagement-prompts-dismissal-count") || "0",
      10,
    );

    if (dismissalCount >= 3) {
      return true;
    }

    const dismissedTime = localStorage.getItem("engagement-prompts-dismissed");
    if (dismissedTime) {
      const dismissedTimestamp = parseInt(dismissedTime, 10);
      const now = Date.now();
      const oneDayInMs = 24 * 60 * 60 * 1000;

      if (now - dismissedTimestamp < oneDayInMs) {
        return true;
      }
    }
    return false;
  });

  const handleDismiss = () => {
    setPromptsDismissed(true);

    const currentCount = parseInt(
      localStorage.getItem("engagement-prompts-dismissal-count") || "0",
      10,
    );
    const newCount = currentCount + 1;
    localStorage.setItem(
      "engagement-prompts-dismissal-count",
      newCount.toString(),
    );

    if (newCount < 3) {
      const now = Date.now();
      localStorage.setItem("engagement-prompts-dismissed", now.toString());
    }
  };

  // An installed PWA on iOS still gets nudged toward the native app.
  const shouldShowAddToHomeScreen =
    IS_IOS || (!isInstalledLoading && !isInstalled);
  const shouldShowNotifications = isInitialized && isSupported && !isSubscribed;
  const shouldShowAnything =
    (shouldShowAddToHomeScreen || shouldShowNotifications) && !promptsDismissed;

  if (!shouldShowAnything) {
    return null;
  }

  const headingText = t("review-engagement-heading", {
    language: String(LANGUAGES[language].commonName),
  });

  return (
    <Card variant="light" animate className="gap-0 px-3 py-4 sm:p-6">
      <div className="flex items-center gap-2 mb-3 sm:mb-4">
        <Sparkles className="h-5 w-5 text-primary shrink-0" />
        <h3 className="font-semibold text-sm sm:text-base">{headingText}</h3>
      </div>

      <p className="text-xs sm:text-sm text-muted-foreground mb-3 sm:mb-4">
        {t(
          "engagement-prompts-research-shows-that-consistent-daily-practice-is-key-to",
        )}
      </p>

      <div className="flex flex-col gap-3 sm:grid sm:grid-cols-[auto_1fr]">
        {shouldShowAddToHomeScreen && (
          <>
            <InstallAppButton
              variant="outline"
              size="sm"
              className="justify-start"
            />
            <p className="text-xs text-muted-foreground self-center">
              {window.navigator.userAgent.match(/mobile/i)
                ? t(
                    "engagement-prompts-quick-access-from-your-home-screen-makes-it-easier",
                  )
                : t(
                    "engagement-prompts-install-as-a-desktop-app-for-quick-access-and",
                  )}
            </p>
          </>
        )}

        {shouldShowNotifications && (
          <>
            <Button
              onClick={subscribe}
              disabled={isNotificationLoading}
              variant="outline"
              size="sm"
              className="justify-start"
            >
              <Bell className="mr-2 h-4 w-4" />
              {isNotificationLoading
                ? t("onboarding-enabling")
                : t("engagement-prompts-enable-reminders")}
            </Button>
            <p className="text-xs text-muted-foreground self-center">
              {t(
                "engagement-prompts-get-gentle-reminders-when-you-have-cards-ready-to",
              )}
            </p>
          </>
        )}
      </div>

      <div className="flex justify-end mt-3 sm:mt-4">
        <Button
          onClick={handleDismiss}
          variant="ghost"
          size="sm"
          className="text-xs"
        >
          {t("common-maybe-later")}
        </Button>
      </div>
    </Card>
  );
}
