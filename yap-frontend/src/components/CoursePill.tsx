import { LanguageIcon } from "@/components/LanguageIcon";
import { ChevronDown } from "lucide-react";

/** The current course as an icon and label, deliberately the quietest control on
 * the page (no surface); tapping it opens the course picker. */
export function CoursePill({ icon, label, onClick }: { icon: string; label: string; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="self-start flex items-center gap-1.5 rounded-md py-1 text-sm text-muted-foreground hover:text-foreground transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      <LanguageIcon icon={icon} variant="bare" className="size-[1em]" />
      <span>{label}</span>
      <ChevronDown className="h-3.5 w-3.5" aria-hidden />
    </button>
  );
}
