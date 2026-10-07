import { rich } from "@/lib/i18n-react";
import { t } from "@/lib/i18n";
import { Link } from "react-router-dom";

export function About() {
  return (
    <div className="text-center text-xs text-muted-foreground mt-4">
      {rich("about-author-credit", {
        author: (
          <a href="https://twitter.com/chadnauseam" className="underline">
            André Popovitch
          </a>
        ),
      })}{" "}
      <a href="https://github.com/yaptown/yap" className="underline">
        GitHub
      </a>
      {" | "}
      <a href="https://discord.gg/mpgqfsH" className="underline">
        Discord
      </a>
      {" | "}
      <Link to="/about" className="underline">
        {t("common-about")}
      </Link>
      {" | "}
      <Link to="/privacy" className="underline">
        {t("common-privacy")}
      </Link>
      {" | "}
      <Link to="/terms" className="underline">
        {t("common-terms")}
      </Link>
    </div>
  );
}
