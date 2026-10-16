import { rich } from "@/lib/i18n-react";
import { t } from "@/lib/i18n";
import { Button } from "@/components/ui/button";
import { APP_STORE_URL, IS_IOS } from "@/lib/ios-app";

export function BrowserNotSupported() {
  // Detect if user is on Safari
  const isSafari = /^((?!chrome|android).)*safari/i.test(navigator.userAgent);

  return (
    <div className="px-2 animate-fade-in">
      <div className="min-h-screen text-foreground">
        <div className="max-w-2xl mx-auto">
          <div className="min-h-screen flex flex-col items-center justify-center">
            <div className="bg-background/50 backdrop-blur-lg flex items-center justify-center p-4 rounded-lg">
              <div className="text-center space-y-6">
                <div className="space-y-2">
                  <div className="w-16 h-16 bg-warning-surface rounded-full flex items-center justify-center mx-auto">
                    <svg
                      className="w-8 h-8 text-warning-foreground"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                      />
                    </svg>
                  </div>
                  <h1 className="text-2xl font-semibold">
                    {t("browser-not-supported-browser-not-supported")}
                  </h1>
                  <p className="text-muted-foreground">
                    {t(
                      "browser-not-supported-yap-requires-modern-browser-features-that-aren-t-available",
                    )}
                  </p>
                </div>

                {IS_IOS ? (
                  <div className="bg-muted/50 rounded-lg p-4 space-y-3">
                    <p className="text-sm">
                      {t("browser-not-supported-the-ios-app-runs-on-ios-18")}
                    </p>
                    <Button asChild>
                      <a href={APP_STORE_URL}>{t("ios-app-download")}</a>
                    </Button>
                  </div>
                ) : (
                  <>
                    {isSafari && (
                      <div className="bg-caution-surface border border-caution-border rounded-lg p-4 text-left space-y-2">
                        <p className="text-sm font-semibold text-caution-foreground mb-1">
                          {t(
                            "browser-not-supported-apple-users-update-required",
                          )}
                        </p>
                        <p className="text-sm">
                          {t(
                            "browser-not-supported-for-iphone-ipad-users-yap-town-requires-ios-26",
                          )}
                        </p>
                        <p className="text-sm ">
                          {t(
                            "browser-not-supported-if-you-re-on-a-desktop-platform-you-can",
                          )}
                        </p>
                      </div>
                    )}

                    <div className="bg-muted/50 rounded-lg p-4 text-left space-y-3">
                      <p className="text-sm font-medium">
                        {t(
                          "browser-not-supported-please-use-one-of-these-browsers",
                        )}
                      </p>
                      <ul className="space-y-2 text-sm">
                        <li className="flex items-center gap-2">
                          <span className="text-positive">✓</span>
                          <span>
                            {t(
                              "browser-not-supported-google-chrome-version-86",
                            )}
                          </span>
                        </li>
                        <li className="flex items-center gap-2">
                          <span className="text-positive">✓</span>
                          <span>
                            {t(
                              "browser-not-supported-microsoft-edge-version-86",
                            )}
                          </span>
                        </li>
                        <li className="flex items-center gap-2">
                          <span className="text-positive">✓</span>
                          <span>
                            {t(
                              "browser-not-supported-safari-version-26-0-requires-ios-macos-18-currently",
                            )}
                          </span>
                        </li>
                        <li className="flex items-center gap-2">
                          <span className="text-positive">✓</span>
                          <span>
                            {t("browser-not-supported-firefox-version-111")}
                          </span>
                        </li>
                      </ul>
                    </div>

                    <div className="text-xs text-muted-foreground">
                      <p>
                        {rich("browser-storage-support", {
                          implementation: (
                            <a
                              href="https://developer.mozilla.org/en-US/docs/Web/API/FileSystemWritableFileStream"
                              target="_blank"
                              rel="noopener noreferrer"
                            >
                              {t(
                                "browser-not-supported-only-recently-been-implemented-in-safari",
                              )}
                            </a>
                          ),
                        })}
                      </p>
                    </div>
                  </>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
