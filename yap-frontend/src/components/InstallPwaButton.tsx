import { t } from "@/lib/i18n";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Home } from "lucide-react";
import { AddToHomeScreenModal } from "@/components/add-to-home-screen-modal";

const hasInstallElement =
  typeof window !== "undefined" && "HTMLInstallElement" in window;

interface InstallPwaButtonProps {
  variant?:
    "default" | "outline" | "ghost" | "link" | "destructive" | "secondary";
  size?: "default" | "sm" | "lg" | "icon";
  className?: string;
}

export function InstallPwaButton({
  variant = "outline",
  size = "sm",
  className,
}: InstallPwaButtonProps) {
  const [showModal, setShowModal] = useState(false);

  if (hasInstallElement) {
    return <install className={className} />;
  }

  return (
    <>
      <Button
        onClick={() => setShowModal(true)}
        variant={variant}
        size={size}
        className={className}
      >
        <Home className="mr-2 h-4 w-4" />
        {window.navigator.userAgent.match(/mobile/i)
          ? t("add-to-home-screen-add-to-home-screen")
          : t("install-pwa-button-install-app")}
      </Button>
      <AddToHomeScreenModal open={showModal} onOpenChange={setShowModal} />
    </>
  );
}
