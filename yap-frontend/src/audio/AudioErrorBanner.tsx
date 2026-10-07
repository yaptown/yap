import { t } from "@/lib/i18n";
import { WifiOff } from "lucide-react";
import { Button } from "@/components/ui/button";

interface AudioErrorBannerProps {
  onSkip: () => void;
  label?: string;
}

export function AudioErrorBanner({
  onSkip,
  label = t("audio-error-banner-skip-for-now"),
}: AudioErrorBannerProps) {
  return (
    <div className="rounded-lg p-4 border bg-warning-surface border-warning-border flex flex-col items-center gap-3 animate-fade-in">
      <div className="flex items-center gap-2 text-warning-foreground">
        <WifiOff className="h-4 w-4" />
        <p className="text-sm font-medium">
          {t("audio-error-banner-audio-unavailable-you-may-be-offline")}
        </p>
      </div>
      <Button onClick={onSkip} variant="outline" size="sm">
        {label}
      </Button>
    </div>
  );
}
