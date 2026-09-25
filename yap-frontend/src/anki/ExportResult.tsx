import { useId, type ComponentType, type ReactNode } from "react";
import { Download } from "lucide-react";
import { toast } from "sonner";
import type { AnkiExportView } from "../../../yap-frontend-rs/pkg";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { saveFile } from "./save-file";

/**
 * What comes after the deck downloads. Signed-in learners are invited into
 * the app; signed-out visitors get one call to action, saving their deck.
 * Either way the deck can be saved again or fetched by link.
 */
export function ExportResult({ view, signedIn, file, downloadLink, onSignUp, onGoToYap, Title = "h2", Body = "p" }: {
  view: AnkiExportView;
  signedIn: boolean;
  file?: { blob: Blob; name: string };
  downloadLink?: string;
  onSignUp: () => void;
  onGoToYap: () => void;
  /** The dialog passes its own title and description for accessibility. */
  Title?: ComponentType<{ className?: string; children: ReactNode }> | "h2";
  Body?: ComponentType<{ className?: string; children: ReactNode }> | "p";
}) {
  return (
    <div className="flex flex-col gap-5">
      <div className="flex flex-col gap-2 text-left">
        <Title className="text-xl font-semibold leading-snug">{signedIn ? view.try_yap_heading : view.save_deck_heading}</Title>
        <Body className="text-muted-foreground">{signedIn ? view.try_yap_body : view.save_deck_body}</Body>
      </div>
      <Button type="button" size="lg" className="h-11 self-start text-base" onClick={signedIn ? onGoToYap : onSignUp}>
        {signedIn ? view.try_yap_label : view.sign_up_label}
      </Button>
      <div className="flex flex-col gap-3 border-t pt-5">
        {signedIn && <p>{view.enjoy_deck}</p>}
        {file && (
          <Button type="button" variant="outline" className="self-start" onClick={() => saveFile(file)}>
            <Download aria-hidden />
            {view.download_again_label}
          </Button>
        )}
        {downloadLink && <DownloadLink link={downloadLink} />}
      </div>
    </div>
  );
}

function DownloadLink({ link }: { link: string }) {
  const id = useId();
  return (
    <div className="flex flex-col gap-2 text-sm text-muted-foreground">
      <Label htmlFor={id} className="text-foreground">Download link</Label>
      <div className="flex gap-2">
        <Input id={id} readOnly value={link} className="min-w-0 font-mono" />
        <Button type="button" variant="outline" onClick={async () => {
          try {
            await navigator.clipboard.writeText(link);
            toast.success("Link copied");
          } catch {
            toast.error("Could not copy the link. Select and copy it manually.");
          }
        }}>Copy</Button>
      </div>
      <p>In AnkiMobile: Decks → Add → Download link. On AnkiDroid or desktop, open the downloaded file instead. The link works for 8 days.</p>
    </div>
  );
}
