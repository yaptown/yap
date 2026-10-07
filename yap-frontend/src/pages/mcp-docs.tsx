import { t } from "@/lib/i18n";
import { rich } from "@/lib/i18n-react";
import { Card } from "@/components/ui/card";
import { Link } from "react-router-dom";

const CONTACT_EMAIL = "support@yap.town";
const SECURITY_EMAIL = "security@yap.town";
const CONNECTOR_URL = "https://mcp.yap.town/mcp";

export function McpDocsPage() {
  return (
    <div className="max-w-2xl mx-auto">
      <div className="flex flex-col items-center justify-center p-4 gap-4">
        <div className="flex items-center mb-4 gap-2 w-full">
          <Link to="/" className="text-2xl">
            ←
          </Link>
          <h1 className="text-3xl font-bold">
            {t("mcp-docs-yap-for-ai-assistants")}
          </h1>
        </div>

        <Card className="max-w-2xl w-full p-8 gap-2 text-muted-foreground">
          <div className="px-2 space-y-2">
            <p>
              {rich("mcp-docs-overview", {
                link: (
                  <a
                    href="https://modelcontextprotocol.io"
                    className="underline text-foreground"
                  >
                    {t("mcp-docs-label-mcp-server")}
                  </a>
                ),
              })}
            </p>
            <p>
              {rich("mcp-docs-url", {
                url: <code className="text-foreground">{CONNECTOR_URL}</code>,
              })}
            </p>
          </div>
        </Card>

        <Card className="max-w-2xl w-full p-8 gap-2 text-muted-foreground">
          <h2 className="text-xl font-semibold text-foreground mb-2">
            {t("mcp-docs-what-it-can-do")}
          </h2>
          <div className="px-2 space-y-2">
            <ul className="list-disc pl-5 space-y-1">
              <li>{t("mcp-docs-list-the-flashcards-you-re-due-to-review")}</li>
              <li>
                {t(
                  "mcp-docs-quiz-you-with-example-sentences-made-of-words-you",
                )}
              </li>
              <li>{t("mcp-docs-search-the-dictionary-of-your-course")}</li>
              <li>{t("mcp-docs-add-new-words-and-phrases-to-your-deck")}</li>
              <li>{t("mcp-docs-release-cards-you-ve-set-aside-in-lockup")}</li>
              <li>
                {t(
                  "mcp-docs-show-your-stats-streak-xp-deck-size-comprehension-tier",
                )}
              </li>
            </ul>
            <p>{t("mcp-docs-capability-limits")}</p>
          </div>
        </Card>

        <Card className="max-w-2xl w-full p-8 gap-2 text-muted-foreground">
          <h2 className="text-xl font-semibold text-foreground mb-2">
            {t("mcp-docs-connecting")}
          </h2>
          <div className="px-2 space-y-2">
            <p>
              {rich("mcp-docs-connect-claude", {
                label: <span className="text-foreground">Claude:</span>,
              })}
            </p>
            <p>
              {rich("mcp-docs-connect-chatgpt", {
                label: <span className="text-foreground">ChatGPT:</span>,
              })}
            </p>
            <p>{t("mcp-docs-approve-connection")}</p>
          </div>
        </Card>

        <Card className="max-w-2xl w-full p-8 gap-2 text-muted-foreground">
          <h2 className="text-xl font-semibold text-foreground mb-2">
            {t("common-privacy")}
          </h2>
          <div className="px-2 space-y-2">
            <p>
              {rich("mcp-docs-privacy-summary", {
                link: (
                  <Link to="/privacy" className="underline text-foreground">
                    {t("mcp-docs-label-yap-s-privacy-policy")}
                  </Link>
                ),
              })}
            </p>
          </div>
        </Card>

        <Card className="max-w-2xl w-full p-8 gap-2 text-muted-foreground">
          <h2 className="text-xl font-semibold text-foreground mb-2">
            {t("mcp-docs-troubleshooting")}
          </h2>
          <div className="px-2 space-y-2">
            <p>
              {rich("mcp-docs-missing-course", {
                label: (
                  <span className="text-foreground">
                    {t(
                      "mcp-docs-label-could-not-detect-course-or-an-empty-deck",
                    )}
                  </span>
                ),
                link: (
                  <a
                    href="https://yap.town"
                    className="underline text-foreground"
                  >
                    yap.town
                  </a>
                ),
              })}
            </p>
            <p>
              {rich("mcp-docs-connection-fails", {
                label: (
                  <span className="text-foreground">
                    {t("mcp-docs-label-connection-fails")}
                  </span>
                ),
              })}
            </p>
            <p>
              {rich("mcp-docs-stale-data", {
                label: (
                  <span className="text-foreground">
                    {t("mcp-docs-label-stale-data")}
                  </span>
                ),
              })}
            </p>
            <p>
              {rich("mcp-docs-rate-limit", {
                label: (
                  <span className="text-foreground">
                    {t("mcp-docs-label-too-many-requests")}
                  </span>
                ),
              })}
            </p>
          </div>
        </Card>

        <Card className="max-w-2xl w-full p-8 gap-2 text-muted-foreground">
          <h2 className="text-xl font-semibold text-foreground mb-2">
            {t("mcp-docs-support-and-security")}
          </h2>
          <div className="px-2 space-y-2">
            <p>
              {rich("mcp-docs-support-contact", {
                link: (
                  <a
                    href={`mailto:${CONTACT_EMAIL}`}
                    className="underline text-foreground"
                  >
                    {CONTACT_EMAIL}
                  </a>
                ),
              })}
            </p>
            <p>
              {rich("mcp-docs-security-contact", {
                link: (
                  <a
                    href={`mailto:${SECURITY_EMAIL}`}
                    className="underline text-foreground"
                  >
                    {SECURITY_EMAIL}
                  </a>
                ),
              })}
            </p>
          </div>
        </Card>
      </div>
    </div>
  );
}
