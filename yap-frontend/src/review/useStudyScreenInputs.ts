import { useState } from "react";
import type { ReviewScreenInputs } from "../../../yap-frontend-rs/pkg";
import { useOptionalCourseStudy } from "@/review/course-study";

// Live screens share the controller's clock, restrictions, and deck selection.
// Injected captures are inert, even if rendered inside a live course route.
export function useStudyScreenInputs(enabled = true): ReviewScreenInputs {
  const study = useOptionalCourseStudy();
  const [fixtureInputs] = useState<ReviewScreenInputs>(() => ({
    banned: [],
    sentence_list: undefined,
    online: false,
    is_signed_in: false,
    needs_display_name: false,
    display_name_dismissed: false,
    has_access_token: false,
    starting_fresh: undefined,
    history_known: false,
    dismissed_accomplishment_at_review: undefined,
    placement: undefined,
    current_challenge: undefined,
    timestamp_ms: Date.now(),
  }));
  if (!enabled) return fixtureInputs;
  if (!study) throw new Error("Live study screens require CourseRoutes");
  return study.inputs;
}
