import { Button } from "@/components/ui/button";
import { MicOff } from "lucide-react";

interface CantSpeakButtonProps {
  onClick: () => void;
}

export function CantSpeakButton({ onClick }: CantSpeakButtonProps) {
  return (
    <Button
      onClick={onClick}
      variant="outline"
      className="w-full h-12 text-base font-medium"
    >
      <MicOff className="mr-2 h-5 w-5" />
      Can't speak now
    </Button>
  );
}
