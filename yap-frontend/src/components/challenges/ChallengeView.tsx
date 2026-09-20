import type { ComponentProps, ReactNode } from "react";
import {
  type Challenge,
  type Gram,
  get_flashcard_disclosure,
  should_show_challenge_tutorial,
} from "../../../../yap-frontend-rs/pkg";
import { Flashcard } from "../Flashcard";
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
}: Props) {
  return currentChallenge.type === "PronunciationChallenge" ? (
    <PronunciationChallenge
      pattern={currentChallenge.pattern}
      guide={currentChallenge.guide}
      cues={currentChallenge.cues}
      onRating={onRating}
      accessToken={accessToken}
      onCantSpeak={onCantSpeak}
      targetLanguage={targetLanguage}
      nativeLanguage={nativeLanguage}
      isNew={currentChallenge.is_new}
      showGuide={should_show_challenge_tutorial(
        currentChallenge.times_type_seen,
      )}
      key={totalReviewsCompleted}
    />
  ) : currentChallenge.type === "FlashCardReview" ? (
    <Flashcard
      audioRequest={currentChallenge.flashcard.audio}
      content={currentChallenge.flashcard.content}
      isNew={currentChallenge.is_new}
      disclosure={get_flashcard_disclosure(
        totalCount,
        currentChallenge.times_type_seen,
      )}
      onRating={onRating}
      accessToken={accessToken}
      key={totalReviewsCompleted}
      onCantListen={onCantListen}
      targetLanguage={targetLanguage}
      nativeLanguage={nativeLanguage}
      autoplayed={autoplayed}
      setAutoplayed={setAutoplayed}
      menuExtras={menuExtras}
    />
  ) : currentChallenge.type === "TranslateComprehensibleSentence" ? (
    <TranslationChallenge
      sentence={currentChallenge}
      onComplete={onTranslationComplete}
      accessToken={accessToken}
      key={totalReviewsCompleted}
      targetLanguage={targetLanguage}
      nativeLanguage={nativeLanguage}
      autoplayed={autoplayed}
      setAutoplayed={setAutoplayed}
      deck={deck}
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
      totalReviewsCompleted={totalReviewsCompleted}
    />
  );
}
