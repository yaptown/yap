import { t } from "@/lib/i18n";
import { useId, useRef, useState } from "react";
import { Check } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

/** A labelled, read-only value (a link, a URL) with a button that copies it. */
export function CopyField({ label, value }: { label: string; value: string }) {
  const id = useId();
  const input = useRef<HTMLInputElement>(null);
  // Feedback on the button itself: a toast can sit under iOS Safari's
  // floating toolbar, which made a working tap look like it did nothing.
  const [copied, setCopied] = useState(false);
  return (
    <div className="flex flex-col gap-2">
      <Label htmlFor={id} className="text-foreground">
        {label}
      </Label>
      <div className="flex gap-2">
        <Input
          ref={input}
          id={id}
          readOnly
          value={value}
          className="min-w-0 font-mono"
        />
        <Button
          type="button"
          variant="outline"
          onClick={async () => {
            // Copy the selection first, synchronously inside the tap: iOS
            // Safari can refuse the async clipboard API. The value stays
            // selected for copying by hand if both fail.
            input.current?.focus();
            input.current?.setSelectionRange(0, value.length);
            if (!document.execCommand("copy")) {
              try {
                await navigator.clipboard.writeText(value);
              } catch {
                toast.error(t("common-could-not-copy"));
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
              {t("common-copied")}
            </>
          ) : (
            t("common-copy")
          )}
        </Button>
      </div>
    </div>
  );
}
