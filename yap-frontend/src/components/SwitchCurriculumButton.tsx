import type {
  DeckEvent,
  SwitchCurriculumView,
} from "../../../yap-frontend-rs/pkg";
import { Button } from "@/components/ui/button";

/**
 * The floating commit for a browsed-but-not-chosen curriculum. Rust decides
 * when it appears and what it says; tapping it appends the one switch event.
 */
export function SwitchCurriculumButton({
  commit,
  onCommit,
}: {
  commit: SwitchCurriculumView | null | undefined;
  onCommit: (event: DeckEvent) => void;
}) {
  if (!commit) return null;
  return (
    <div className="sticky bottom-4 z-20 flex justify-center pointer-events-none">
      <Button
        size="lg"
        className="pointer-events-auto shadow-lg max-w-full h-auto whitespace-normal text-center"
        onClick={() => onCommit(commit.event)}
      >
        {commit.label}
      </Button>
    </div>
  );
}
