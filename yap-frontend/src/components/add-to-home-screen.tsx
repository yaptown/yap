import { t } from "@/lib/i18n";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Home, Share, Plus, MoreVertical } from "lucide-react";

interface AddToHomeScreenProps {
  onDismiss?: () => void;
}

export function AddToHomeScreen({ onDismiss }: AddToHomeScreenProps) {
  const [showInstructions, setShowInstructions] = useState(false);

  const userAgent = navigator.userAgent.toLowerCase();
  let platform: "ios" | "android" | "desktop" | "unknown";
  if (/iphone|ipad|ipod/.test(userAgent)) {
    platform = "ios";
  } else if (/android/.test(userAgent)) {
    platform = "android";
  } else if (/windows|mac|linux/.test(userAgent) && !/mobile/.test(userAgent)) {
    platform = "desktop";
  } else {
    platform = "unknown";
  }

  const handleShowInstructions = () => {
    setShowInstructions(true);
  };

  const handleDismiss = () => {
    setShowInstructions(false);
    onDismiss?.();
  };

  if (!showInstructions) {
    return (
      <div className="space-y-2">
        <Button
          onClick={handleShowInstructions}
          variant="default"
          size="sm"
          className="w-full sm:w-auto"
        >
          <Home className="mr-2 h-4 w-4" />
          {t("add-to-home-screen-add-to-home-screen")}
        </Button>
      </div>
    );
  }

  return (
    <div className="space-y-4 p-4 bg-muted rounded-lg">
      <h3 className="font-semibold flex items-center gap-2">
        <Home className="h-5 w-5" />
        {t("add-to-home-screen-modal-add-yap-town-to-home-screen")}
      </h3>

      {platform === "ios" && (
        <div className="space-y-3 text-sm">
          <p>
            {t("pwa-install-instructions-to-add-this-app-to-your-home-screen")}
          </p>
          <ol className="space-y-2 ml-4">
            <li className="flex items-start gap-2">
              <Share className="h-4 w-4 mt-0.5 shrink-0" />
              <span>
                {t(
                  "add-to-home-screen-tap-the-share-button-at-the-bottom-of-safari",
                )}
              </span>
            </li>
            <li className="flex items-start gap-2">
              <Plus className="h-4 w-4 mt-0.5 shrink-0" />
              <span>
                {t("add-to-home-screen-scroll-down-and-tap-add-to-home-screen")}
              </span>
            </li>
            <li className="flex items-start gap-2">
              <span className="text-lg mt-[-4px]">✓</span>
              <span>
                {t("add-to-home-screen-tap-add-in-the-top-right-corner")}
              </span>
            </li>
          </ol>
        </div>
      )}

      {platform === "android" && (
        <div className="space-y-3 text-sm">
          <p>
            {t("pwa-install-instructions-to-add-this-app-to-your-home-screen")}
          </p>
          <ol className="space-y-2 ml-4">
            <li className="flex items-start gap-2">
              <MoreVertical className="h-4 w-4 mt-0.5 shrink-0" />
              <span>
                {t(
                  "pwa-install-instructions-tap-the-menu-button-three-dots-in-your-browser",
                )}
              </span>
            </li>
            <li className="flex items-start gap-2">
              <Home className="h-4 w-4 mt-0.5 shrink-0" />
              <span>
                {t(
                  "pwa-install-instructions-tap-add-to-home-screen-or-install-app",
                )}
              </span>
            </li>
            <li className="flex items-start gap-2">
              <span className="text-lg mt-[-4px]">✓</span>
              <span>{t("pwa-install-instructions-tap-add-or-install")}</span>
            </li>
          </ol>
        </div>
      )}

      {platform === "desktop" && (
        <div className="space-y-3 text-sm">
          <p>{t("add-to-home-screen-to-install-this-app-on-your-computer")}</p>
          <ol className="space-y-2 ml-4">
            <li>
              {t(
                "add-to-home-screen-look-for-an-install-button-in-your-browser-s",
              )}
            </li>
            <li>
              {t(
                "add-to-home-screen-or-use-the-browser-menu-to-find-install-yap",
              )}
            </li>
            <li>
              {t("add-to-home-screen-the-app-will-open-in-its-own-window")}
            </li>
          </ol>
        </div>
      )}

      <div className="flex gap-2 pt-2">
        <Button onClick={handleDismiss} variant="outline" size="sm">
          {t("common-maybe-later")}
        </Button>
      </div>
    </div>
  );
}
