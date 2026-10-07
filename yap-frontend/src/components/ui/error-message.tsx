import { t } from "@/lib/i18n";
import { useState } from "react";
import { Copy, Check, ExternalLink } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface ErrorMessageProps {
  message: string;
  title?: string;
  variant?: "default" | "compact";
  className?: string;
}

export function ErrorMessage({
  message,
  title,
  variant = "default",
  className,
}: ErrorMessageProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard
      .writeText(message)
      .then(() => {
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      })
      .catch((err) => {
        console.error("Failed to copy error:", err);
      });
  };

  const createGitHubIssueUrl = () => {
    const userAgent = navigator.userAgent;

    // Detect device type
    const isMobile =
      /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
        userAgent,
      );
    const isTablet = /iPad|Android(?!.*Mobile)/i.test(userAgent);
    const deviceType = isTablet
      ? t("report-tablet")
      : isMobile
        ? t("report-mobile")
        : t("report-desktop");

    // Detect OS
    let os = t("report-unknown");
    if (userAgent.includes("Win")) os = "Windows";
    else if (userAgent.includes("Mac")) os = "macOS";
    else if (userAgent.includes("Linux")) os = "Linux";
    else if (userAgent.includes("Android")) os = "Android";
    else if (
      userAgent.includes("iOS") ||
      userAgent.includes("iPhone") ||
      userAgent.includes("iPad")
    )
      os = "iOS";

    // Detect browser
    let browser = t("report-unknown");
    if (userAgent.includes("Firefox")) browser = "Firefox";
    else if (userAgent.includes("Edg")) browser = "Edge";
    else if (userAgent.includes("Chrome")) browser = "Chrome";
    else if (userAgent.includes("Safari")) browser = "Safari";
    else if (userAgent.includes("Opera") || userAgent.includes("OPR"))
      browser = "Opera";

    const issueTitle = encodeURIComponent(
      title || t("report-error-loading-language-data"),
    );
    const issueBody = encodeURIComponent(
      t("report-template", {
        device: deviceType,
        os,
        browser,
        userAgent,
        message,
      }),
    );
    return `https://github.com/yaptown/yap/issues/new?title=${issueTitle}&body=${issueBody}&labels[]=bug`;
  };

  return (
    <div
      className={cn(
        "p-3 bg-negative-field border border-negative-border rounded-lg overflow-hidden flex flex-col gap-2",
        className,
      )}
    >
      <div className="flex items-start justify-between gap-2">
        <p
          className={cn(
            "text-sm text-negative-foreground flex-1 line-clamp-3 break-all min-w-0",
            variant === "compact" ? "line-clamp-3" : "line-clamp-5",
          )}
        >
          {message}
        </p>
        <Button
          variant="ghost"
          size="sm"
          className="h-6 w-6 p-0 flex-shrink-0"
          onClick={handleCopy}
          title={
            copied
              ? t("error-message-copied")
              : t("error-message-copy-full-error-message")
          }
        >
          {copied ? (
            <Check className="h-3 w-3 text-positive-foreground" />
          ) : (
            <Copy className="h-3 w-3" />
          )}
        </Button>
      </div>

      {title && (
        <a
          href={createGitHubIssueUrl()}
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center text-s gap-1"
        >
          <ExternalLink className="h-3 w-3" />
          {t("error-message-report-on-github")}
        </a>
      )}
    </div>
  );
}
