import { t } from "@/lib/i18n";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Home } from "lucide-react";
import { PwaInstallInstructions } from "@/components/PwaInstallInstructions";

interface AddToHomeScreenModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function AddToHomeScreenModal({
  open,
  onOpenChange,
}: AddToHomeScreenModalProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Home className="h-5 w-5" />
            {t("add-to-home-screen-modal-add-yap-town-to-home-screen")}
          </DialogTitle>
          <DialogDescription>
            {t(
              "add-to-home-screen-modal-install-the-app-for-quick-access-and-a-better",
            )}
          </DialogDescription>
        </DialogHeader>

        <PwaInstallInstructions />

        <div className="flex justify-end gap-2 pt-4">
          <Button onClick={() => onOpenChange(false)} variant="outline">
            {t("add-to-home-screen-modal-got-it")}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
