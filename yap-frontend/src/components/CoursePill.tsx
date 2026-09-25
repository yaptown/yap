import { ChevronDown } from "lucide-react";

/** The current course as a flag and label; tapping it opens the course picker. */
export function CoursePill({ flag, label, onClick }: { flag: string; label: string; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="self-start flex items-center gap-2 rounded-xl border border-border/60 px-3 py-2 backdrop-blur-sm hover:bg-muted/50 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      <span className="text-lg leading-none" aria-hidden>
        {flag}
      </span>
      <span className="font-medium">{label}</span>
      <ChevronDown className="h-4 w-4 text-muted-foreground" aria-hidden />
    </button>
  );
}
