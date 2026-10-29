import { Link } from "react-router-dom";
import { t } from "@/lib/i18n";
import { cn } from "@/lib/utils";

/** The wordmark and site links along the bottom of every page, in the app
 *  theme's tokens (a page with its own palette can remap them). The pink is
 *  the landing page's, which the theme's paler primary doesn't match. */
export function SiteFooter({ className }: { className?: string }) {
  return (
    <footer
      className={cn(
        "flex flex-wrap items-baseline justify-between gap-4 border-t pt-7 pb-9 [&_a]:transition-colors [&_a:hover]:text-[#b93cb6] dark:[&_a:hover]:text-[#e273df]",
        className,
      )}
    >
      <Link
        to="/"
        className="font-heading text-[24px] leading-none font-bold tracking-[-0.03em]"
      >
        Yap<span className="text-[#b93cb6] dark:text-[#e273df]">.</span>Town
      </Link>
      <nav className="flex flex-wrap gap-x-6 gap-y-2 text-[15px] text-muted-foreground">
        <a href="/d/">{t("common-dictionary")}</a>
        <Link to="/anki">{t("landing-anki-decks")}</Link>
        <Link to="/select-language">{t("landing-languages-2")}</Link>
        <a href="/blog/">{t("landing-blog")}</a>
        <Link to="/about">{t("common-about")}</Link>
        <a href="https://github.com/yaptown/yap">GitHub</a>
        <a href="https://discord.gg/mpgqfsH">Discord</a>
        <Link to="/privacy">{t("common-privacy")}</Link>
        <Link to="/terms">{t("common-terms")}</Link>
      </nav>
    </footer>
  );
}
