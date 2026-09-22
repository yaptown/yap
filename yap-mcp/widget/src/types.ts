// Mirror of present_card / present_translation's structuredContent.challenge
// (see yap-mcp/src/server.rs). The card object is opaque: passed back
// verbatim to log_review / grade_translation, never inspected here.
import type {
  AudioRequest,
  CardContent,
  FlashcardView,
  Language,
  Literal,
  PronunciationView,
  Rating,
} from "../../../yap-frontend-rs/pkg";

interface ChallengeBase {
  // Server-minted id for this presentation, echoed back as the review's
  // idempotency token so a retried/re-rendered submit records it once.
  nonce: string;
  language: Language;
  card: unknown;
}

export interface FlashcardChallenge extends ChallengeBase {
  type: "flashcard";
  kind: "written" | "listening";
  audio: AudioRequest;
  // The exact CardContent the app builds — rendered by the app's FlashcardChallenge
  // component verbatim. Type-only import; erases at build (no WASM in the bundle).
  content: CardContent;
  // Product policy computed by Rust, shared with the web app.
  view: FlashcardView;
}

export interface PronunciationChallenge extends ChallengeBase {
  type: "pronunciation";
  // The native language, for tagging the "as in" connector gloss.
  native_language: Language;
  view: PronunciationView;
}

export interface TranslationChallenge extends ChallengeBase {
  type: "translation";
  audio: AudioRequest;
  sentence: {
    text: string;
    // Full literals (word + word_type + whitespace) so the widget renders the
    // app's shared ChallengeSentence verbatim. Type-only import; erases.
    literals: Literal<string>[];
    sources: string[];
  };
}

export type Challenge =
  | FlashcardChallenge
  | PronunciationChallenge
  | TranslationChallenge;

// Mirror of grade_translation's structured result.
export interface GradeResult {
  perfect: boolean;
  correct_translation: string | null;
  encouragement: string | null;
  explanation: string | null;
  autograding_error: string | null;
  literal_grades: ("remembered" | "forgot" | null)[];
  phrases_remembered: string[];
  phrases_forgot: string[];
  remaining_due: number;
}

export type { AudioRequest, Rating };
