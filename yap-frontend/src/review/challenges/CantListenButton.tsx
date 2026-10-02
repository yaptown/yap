import { Button } from "@/components/ui/button";
import { VolumeX } from "lucide-react";

interface CantListenButtonProps {
  onClick: () => void;
  label: string;
}

export function CantListenButton({ onClick, label }: CantListenButtonProps) {
  return (
    <Button
      onClick={onClick}
      variant="outline"
      // Translucent so the report link it passes over shows through softly.
      className="w-full h-12 text-base font-medium bg-background/40 backdrop-blur-md"
    >
      <span className="relative flex items-center justify-center">
        <VolumeX className="absolute right-full mr-2 h-5 w-5" />
        {label}
      </span>
    </Button>
  );
}
