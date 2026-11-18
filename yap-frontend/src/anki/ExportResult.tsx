import { t } from "@/lib/i18n";
import { type ComponentType, type ReactNode } from "react";
import { Download } from "lucide-react";
import type { AnkiExportView } from "../../../yap-frontend-rs/pkg";
import { Button } from "@/components/ui/button";
import { CopyField } from "@/components/CopyField";
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
      <div className={`flex flex-col gap-5 ${files ? "border-t pt-5" : ""}`}>
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
  return (
    <div className="flex flex-col gap-2 text-sm text-muted-foreground">
      <CopyField label={t("export-result-download-link")} value={link} />
      <p>
        {t(
          "export-result-in-ankimobile-decks-add-download-link-on-ankidroid-or",
        )}
      </p>
    </div>
  );
}
