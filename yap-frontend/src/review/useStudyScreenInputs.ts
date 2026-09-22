import { useState } from "react";
import type { HomeScreenInputs } from "../../../yap-frontend-rs/pkg";
import { useOptionalCourseStudy } from "@/review/course-study";

// Live screens share the controller's clock, restrictions, and deck selection.
// Injected captures are inert, even if rendered inside a live course route.
export function useStudyScreenInputs(enabled = true): HomeScreenInputs {
  const study = useOptionalCourseStudy();
  const [fixtureInputs] = useState<HomeScreenInputs>(() => ({
    banned: [],
    sentence_list: undefined,
    online: false,
    is_signed_in: false,
    timestamp_ms: Date.now(),
  }));
  if (!enabled) return fixtureInputs;
  if (!study) throw new Error("Live study screens require CourseRoutes");
  return study.inputs;
}
