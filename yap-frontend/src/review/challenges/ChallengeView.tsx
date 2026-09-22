import type { ComponentProps, ReactNode } from "react";
import {
  type Challenge,
  type Gram,
  flashcard_view,
  pronunciation_view,
} from "../../../../yap-frontend-rs/pkg";
import { FlashcardChallenge } from "./FlashcardChallenge";
import { PronunciationChallenge } from "./PronunciationChallenge";
import { TranslationChallenge } from "./TranslationChallenge";
import { TranscriptionChallenge } from "./TranscriptionChallenge";

type Props = Omit<
  ComponentProps<typeof TranscriptionChallenge>,
  "challenge" | "onComplete"
> & {
  challenge: Challenge<Gram<string>>;
  onRating: ComponentProps<typeof PronunciationChallenge>["onRating"];
  onCantSpeak: () => void;
  onTranslationComplete: ComponentProps<
    typeof TranslationChallenge
  >["onComplete"];
  onTranscriptionComplete: ComponentProps<
    typeof TranscriptionChallenge
  >["onComplete"];
  menuExtras?: ReactNode;
  translationState?: ComponentProps<typeof TranslationChallenge>["initialState"];
};

export function ChallengeView({
  challenge: currentChallenge,
  onRating,
  accessToken,
  onCantSpeak,
  onCantListen,
  targetLanguage,
  nativeLanguage,
  totalReviewsCompleted,
  totalCount,
  autoplayed,
  setAutoplayed,
  onTranslationComplete,
  onTranscriptionComplete,
  deck,
  menuExtras,
  initialState,
  translationState,
  pendingReviewScope,
}: Props) {
  return currentChallenge.type === "PronunciationChallenge" ? (
    <PronunciationChallenge
      view={pronunciation_view(
        currentChallenge.pattern,
        currentChallenge.guide,
        currentChallenge.cues,
        currentChallenge.is_new,
        currentChallenge.times_type_seen,
      )}
      onRating={onRating}
      accessToken={accessToken}
      onCantSpeak={onCantSpeak}
      targetLanguage={targetLanguage}
      nativeLanguage={nativeLanguage}
      key={totalReviewsCompleted}
    />
  ) : currentChallenge.type === "FlashCardReview" ? (
    <FlashcardChallenge
      audioRequest={currentChallenge.flashcard.audio}
      content={currentChallenge.flashcard.content}
      view={flashcard_view(
        currentChallenge.flashcard,
        currentChallenge.is_new,
        totalCount,
        currentChallenge.times_type_seen,
        targetLanguage,
        nativeLanguage,
      )}
      onRating={onRating}
      accessToken={accessToken}
      key={totalReviewsCompleted}
      onCantListen={onCantListen}
      targetLanguage={targetLanguage}
      autoplayed={autoplayed}
      setAutoplayed={setAutoplayed}
      menuExtras={menuExtras}
    />
  ) : currentChallenge.type === "TranslateComprehensibleSentence" ? (
    <TranslationChallenge
      initialState={translationState}
      sentence={currentChallenge}
      onComplete={onTranslationComplete}
      accessToken={accessToken}
      key={`${totalReviewsCompleted}:${currentChallenge.target_language}`}
      targetLanguage={targetLanguage}
      nativeLanguage={nativeLanguage}
      autoplayed={autoplayed}
      setAutoplayed={setAutoplayed}
      deck={deck}
      pendingReviewScope={pendingReviewScope}
      totalReviewsCompleted={totalReviewsCompleted}
    />
  ) : (
    <TranscriptionChallenge
      initialState={initialState}
      challenge={currentChallenge}
      onComplete={onTranscriptionComplete}
      totalCount={totalCount}
      accessToken={accessToken}
      key={`${totalReviewsCompleted}:${currentChallenge.target_language}`}
      onCantListen={onCantListen}
      targetLanguage={targetLanguage}
      nativeLanguage={nativeLanguage}
      autoplayed={autoplayed}
      setAutoplayed={setAutoplayed}
      deck={deck}
      pendingReviewScope={pendingReviewScope}
      totalReviewsCompleted={totalReviewsCompleted}
    />
  );
}
