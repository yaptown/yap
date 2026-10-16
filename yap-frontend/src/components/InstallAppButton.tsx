import { t } from "@/lib/i18n";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Home, Smartphone } from "lucide-react";
import { AddToHomeScreenModal } from "@/components/add-to-home-screen-modal";
import { APP_STORE_URL, IS_IOS } from "@/lib/ios-app";

const hasInstallElement =
  typeof window !== "undefined" && "HTMLInstallElement" in window;

interface InstallAppButtonProps {
  variant?:
    "default" | "outline" | "ghost" | "link" | "destructive" | "secondary";
  size?: "default" | "sm" | "lg" | "icon";
  className?: string;
}

/** On iOS the native app beats the PWA, so this links to the App Store;
 *  elsewhere it installs the PWA. */
export function InstallAppButton({
  variant = "outline",
  size = "sm",
  className,
}: InstallAppButtonProps) {
  const [showModal, setShowModal] = useState(false);

  if (IS_IOS) {
    return (
      <Button asChild variant={variant} size={size} className={className}>
        <a href={APP_STORE_URL}>
          <Smartphone className="mr-2 h-4 w-4" />
          {t("ios-app-download")}
        </a>
      </Button>
    );
  }

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
