import { t } from "@/lib/i18n";
import { Home, Share, Plus, MoreVertical } from "lucide-react";

export function PwaInstallInstructions() {
  const userAgent = navigator.userAgent.toLowerCase();
  const isSafari = /safari/.test(userAgent) && !/chrome/.test(userAgent);
  const isMacOS = /macintosh|mac os x/.test(userAgent);

  let platform: "ios" | "android" | "desktop" | "unknown";
  if (/iphone|ipad|ipod/.test(userAgent)) {
    platform = "ios";
  } else if (/android/.test(userAgent)) {
    platform = "android";
  } else if (isSafari && isMacOS) {
    // Desktop Safari doesn't support PWA installation
    platform = "ios"; // Show iOS-like instructions for Safari desktop
  } else {
    platform = "desktop";
  }

  return (
    <div className="space-y-4">
      {platform === "ios" && (
        <div className="space-y-3 text-sm">
          <p className="text-muted-foreground">
            {t("pwa-install-instructions-safari-is-required-for-this-feature")}
          </p>
          <ol className="space-y-3 ml-4 list-decimal">
            <li className="flex items-start gap-3">
              <MoreVertical className="h-5 w-5 mt-0.5 shrink-0 text-muted-foreground" />
              <span>
                {t(
                  "pwa-install-instructions-tap-the-three-dots-in-the-bottom-right",
                )}
              </span>
            </li>
            <li className="flex items-start gap-3">
              <Share className="h-5 w-5 mt-0.5 shrink-0 text-muted-foreground" />
              <span>{t("pwa-install-instructions-tap-share")}</span>
            </li>
            <li className="flex items-start gap-3">
              <span className="text-sm mt-0.5 shrink-0 text-muted-foreground">
                •••
              </span>
              <span>{t("pwa-install-instructions-tap-more")}</span>
            </li>
            <li className="flex items-start gap-3">
              <Plus className="h-5 w-5 mt-0.5 shrink-0 text-muted-foreground" />
              <span>
                {t("pwa-install-instructions-tap-add-to-home-screen")}
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
          <ol className="space-y-3 ml-4">
            <li className="flex items-start gap-3">
              <MoreVertical className="h-5 w-5 mt-0.5 shrink-0 text-muted-foreground" />
              <span>
                {t(
                  "pwa-install-instructions-tap-the-menu-button-three-dots-in-your-browser",
                )}
              </span>
            </li>
            <li className="flex items-start gap-3">
              <Home className="h-5 w-5 mt-0.5 shrink-0 text-muted-foreground" />
              <span>
                {t(
                  "pwa-install-instructions-tap-add-to-home-screen-or-install-app",
                )}
              </span>
            </li>
            <li className="flex items-start gap-3">
              <span className="text-xl mt-[-4px] text-muted-foreground">✓</span>
              <span>{t("pwa-install-instructions-tap-add-or-install")}</span>
            </li>
          </ol>
        </div>
      )}

      {platform === "desktop" && (
        <div className="space-y-3 text-sm">
          <p className="font-medium">
            {t("pwa-install-instructions-for-chrome-edge-or-brave")}
          </p>
          <div className="space-y-2 ml-4">
            <p className="flex items-start gap-2">
              <span className="text-muted-foreground">•</span>
              {t(
                "pwa-install-instructions-look-for-the-install-icon-in-your-address-bar",
              )}
            </p>
            <p className="flex items-start gap-2">
              <span className="text-muted-foreground">•</span>
              {t(
                "pwa-install-instructions-or-click-the-three-dot-menu-install-yap-town",
              )}
            </p>
          </div>

          <img
            src="/desktop-pwa-installation.png"
            alt={t("pwa-install-instructions-desktop-pwa-installation-example")}
            className="rounded-lg border mt-3 mb-3"
          />

          <p className="font-medium mt-4">
            {t("pwa-install-instructions-for-firefox")}
          </p>
          <div className="ml-4">
            <p className="text-muted-foreground">
              {t(
                "pwa-install-instructions-firefox-doesn-t-support-app-installation-consider-using-chrome",
              )}
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
