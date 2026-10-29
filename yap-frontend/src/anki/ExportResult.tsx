import { t } from "@/lib/i18n";
import { useId, useRef, useState, type ComponentType, type ReactNode } from "react";
import { Check, Download } from "lucide-react";
import { toast } from "sonner";
import type { AnkiExportView } from "../../../yap-frontend-rs/pkg";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { AppStoreButton } from "@/components/AppStoreButton";
import { IS_IOS } from "@/lib/ios-app";
import { saveFile } from "./save-file";

/**
 * What comes after the deck downloads: the deck can be saved again or
 * fetched by link, and signed-in learners are invited into the app (the App
 * Store one, on iOS). Signed-out visitors are asked to sign in by the deck
 * card instead, so there is one call to action, not two.
 */
export function ExportResult({
  view,
  signedIn,
  file,
  downloadLink,
  onGoToYap,
  Title = "h2",
  Body = "p",
}: {
  view: AnkiExportView;
  signedIn: boolean;
  file?: { blob: Blob; name: string };
  downloadLink?: string;
  onGoToYap: () => void;
  /** The dialog passes its own title and description for accessibility. */
  Title?: ComponentType<{ className?: string; children: ReactNode }> | "h2";
  Body?: ComponentType<{ className?: string; children: ReactNode }> | "p";
}) {
  const files = (file || downloadLink) && (
    <div className="flex flex-col gap-3">
      {file && (
        <Button
          type="button"
          variant="outline"
          className="self-start"
          onClick={() => saveFile(file)}
        >
          <Download aria-hidden />
          {view.download_again_label}
        </Button>
      )}
      {downloadLink && <DownloadLink link={downloadLink} />}
    </div>
  );
  if (!signedIn) return files;
  return (
    <div className="flex flex-col gap-5">
      {files}
      <div
        className={`flex flex-col gap-5 ${files ? "border-t pt-5" : ""}`}
      >
        <div className="flex flex-col gap-2 text-left">
          <Title className="text-xl font-semibold leading-snug">
            {view.try_yap_heading}
          </Title>
          <Body className="text-muted-foreground">{view.try_yap_body}</Body>
        </div>
        {IS_IOS ? (
          <AppStoreButton size="lg" className="h-11 self-start text-base" />
        ) : (
          <Button
            type="button"
            size="lg"
            className="h-11 self-start text-base"
            onClick={onGoToYap}
          >
            {view.try_yap_label}
          </Button>
        )}
        <p>{view.enjoy_deck}</p>
      </div>
    </div>
  );
}

function DownloadLink({ link }: { link: string }) {
  const id = useId();
  const input = useRef<HTMLInputElement>(null);
  // Feedback on the button itself: a toast can sit under iOS Safari's
  // floating toolbar, which made a working tap look like it did nothing.
  const [copied, setCopied] = useState(false);
  return (
    <div className="flex flex-col gap-2 text-sm text-muted-foreground">
      <Label htmlFor={id} className="text-foreground">
        {t("export-result-download-link")}
      </Label>
      <div className="flex gap-2">
        <Input ref={input} id={id} readOnly value={link} className="min-w-0 font-mono" />
        <Button
          type="button"
          variant="outline"
          onClick={async () => {
            // Copy the selection first, synchronously inside the tap: iOS
            // Safari can refuse the async clipboard API. The link stays
            // selected for copying by hand if both fail.
            input.current?.focus();
            input.current?.setSelectionRange(0, link.length);
            if (!document.execCommand("copy")) {
              try {
                await navigator.clipboard.writeText(link);
              } catch {
                toast.error(
                  t("export-result-could-not-copy-the-link-select-and-copy-it"),
                );
                return;
              }
            }
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
          }}
        >
          {copied ? (
            <>
              <Check aria-hidden />
              {t("export-result-link-copied")}
            </>
          ) : (
            t("export-result-copy")
          )}
        </Button>
      </div>
      <p>
        {t(
          "export-result-in-ankimobile-decks-add-download-link-on-ankidroid-or",
        )}
      </p>
    </div>
  );
}
