import { rich } from "@/lib/i18n-react";
import { t } from "@/lib/i18n";
import { Card } from "@/components/ui/card";
import { Link } from "react-router-dom";
import { InstallPwaButton } from "@/components/InstallPwaButton";
import { AppStoreButton } from "@/components/AppStoreButton";

export function AboutPage() {
  return (
    <div className="max-w-2xl mx-auto">
      <div className="flex flex-col items-center justify-center p-4 gap-4">
        <div className="flex items-center mb-4 gap-2 w-full">
          <Link to="/" className="text-2xl">
            ←
          </Link>
          <h1 className="text-3xl font-bold">{t("about-about-yap-town")}</h1>
        </div>
        <Card className="max-w-2xl w-full p-8 gap-2 text-muted-foreground">
          <h2 className="text-xl font-semibold text-foreground mb-2">
            {t("about-philosophy")}
          </h2>
          <div className="px-2 space-y-2">
            <p>
              {t(
                "about-i-believe-that-efficient-and-effective-language-learning-options",
              )}
            </p>
            <p>
              {t(
                "about-the-most-famous-concepts-in-language-learning-are-spaced",
              )}
            </p>
          </div>
        </Card>
        <Card className="max-w-2xl w-full p-8 gap-2 text-muted-foreground">
          <h2 className="text-xl font-semibold text-foreground mb-2">
            {t("about-how-it-works")}
          </h2>
          <div className="px-2 space-y-2">
            <p>{t("about-yap-tries-to-keep-track-of-what-words-you")}</p>
            <p>
              {t(
                "about-furthermore-yap-selects-sentences-that-also-contain-words-that",
              )}
            </p>
            <p>
              {t(
                "about-this-approach-helps-you-build-vocabulary-and-grammar-intuition",
              )}
            </p>
          </div>
        </Card>
        <Card className="max-w-2xl w-full p-8 gap-2 text-muted-foreground">
          <h2 className="text-xl font-semibold text-foreground mb-2">
            {t("about-offline-first-pwa-and-ios-app")}
          </h2>
          <div className="px-2 space-y-2">
            <p>{t("about-yap-town-works-fully-offline-this-means-you-can")}</p>
            <p>{t("about-to-make-use-of-this-install-the-ios-app-or")}</p>
            <div className="pt-2 flex flex-wrap gap-2">
              <AppStoreButton variant="outline" size="sm" />
              <InstallPwaButton />
            </div>
          </div>
        </Card>
        <Card className="max-w-2xl w-full p-8 gap-2 text-muted-foreground">
          <h2 className="text-xl font-semibold text-foreground mb-2">
            {t("about-who-am-i")}
          </h2>
          <div className="px-2 space-y-2">
            <p>
              {rich("about-created-by-author", {
                author: (
                  <a
                    href="https://twitter.com/chadnauseam"
                    className="underline text-foreground"
                  >
                    André Popovitch
                  </a>
                ),
              })}
            </p>
            <p>
              <a
                href="https://github.com/yaptown/yap"
                className="underline text-foreground"
              >
                {t("about-view-on-github")}
              </a>
              {" | "}
              <a
                href="https://discord.gg/mpgqfsH"
                className="underline text-foreground"
              >
                {t("about-join-the-discord")}
              </a>
            </p>
            <p>{t("about-here-is-a-picture-of-me-and-my-girlfriend")}</p>
            <img
              src="/selfie.webp"
              alt={t("about-a-man-and-woman-dressed-formally-both-are-smiling")}
              className="w-full rounded-lg"
            />
          </div>
        </Card>
        <Card className="max-w-2xl w-full p-8 gap-2 text-muted-foreground">
          <h2 className="text-xl font-semibold text-foreground mb-2">
            {t("common-credits")}
          </h2>
          <div className="px-2 space-y-2">
            <p>
              {t("about-yap-s-sentences-come-from-tatoeba-and-film-subtitles")}{" "}
              <a
                href="https://github.com/yaptown/yap/blob/main/CREDITS.md"
                className="underline text-foreground"
              >
                {t("about-see-the-full-credits-and-licences")}
              </a>
              .
            </p>
          </div>
        </Card>
      </div>
    </div>
  );
}
