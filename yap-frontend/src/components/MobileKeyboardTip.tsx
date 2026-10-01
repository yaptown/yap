import { useState } from "react";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";

interface MobileKeyboardTipProps {
  tip: string;
  className?: string;
}

const DISMISS_KEY = "mobile-keyboard-tip-dismissed";

export function MobileKeyboardTip({ tip, className = "" }: MobileKeyboardTipProps) {
  const [isDismissed, setIsDismissed] = useState(
    () => localStorage.getItem(DISMISS_KEY) === "true",
  );

  const handleDismiss = () => {
    setIsDismissed(true);
    localStorage.setItem(DISMISS_KEY, "true");
  };

  if (isDismissed) {
    return null;
  }

  return (
    <div
      className={`md:hidden flex items-center justify-between gap-2 p-3 mt-3 border rounded-lg bg-muted/30 ${className}`}
    >
      <p className="text-sm text-muted-foreground flex-1">
        <span className="font-medium">Tip:</span> {tip}
      </p>
      <Button
        variant="ghost"
        size="icon"
        className="h-6 w-6 shrink-0"
        onClick={handleDismiss}
      >
        <X className="h-4 w-4" />
      </Button>
    </div>
  );
}
