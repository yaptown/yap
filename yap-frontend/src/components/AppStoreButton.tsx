import { t } from "@/lib/i18n";
import type { ComponentProps } from "react";
import { Smartphone } from "lucide-react";
import { Button } from "@/components/ui/button";
import { APP_STORE_URL } from "@/lib/ios-app";

export function AppStoreButton(
  props: Pick<ComponentProps<typeof Button>, "variant" | "size" | "className">,
) {
  return (
    <Button asChild {...props}>
      <a href={APP_STORE_URL}>
        <Smartphone className="mr-2 h-4 w-4" />
        {t("ios-app-download")}
      </a>
    </Button>
  );
}
