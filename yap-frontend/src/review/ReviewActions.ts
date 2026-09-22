import type { ReactNode } from "react";
import type { SentenceList } from "@/browse/useSentenceList";
import type { Deck, DeckEvent, PlacementSession, Rating, PartGraded, LiteralGrades, Gram, Heteronym } from "../../../yap-frontend-rs/pkg";

export type ReviewHost = {
  deck: Deck;
  accessToken: string | undefined;
  autoplayed: boolean;
  setAutoplayed: () => void;
  menuExtras?: ReactNode;
};
export type ReviewActions = {
  pendingReviewScope: string;
  onRating: (rating: Rating) => boolean;
  onTranslationComplete: (
    grade: { literalGrades: LiteralGrades; phrasesRemembered: Gram<string>[]; phrasesForgot: Gram<string>[] } | { perfect: string | null },
    tapped: Heteronym<string>[], submission: string, completedAtMs: number,
  ) => boolean;
  onTranscriptionComplete: (grade: PartGraded[], completedAtMs: number) => boolean;
  onCantListen: () => void;
  onCantSpeak: () => void;
  addEvent: (event: DeckEvent) => void;
  undoRestrictions: () => void;
  setSentenceList: (list: SentenceList) => void;
  commitSentenceList: (event: DeckEvent) => void;
  dismissAccomplishment: () => void;
  setPlacement: (session: PlacementSession) => void;
  completePlacementTest: (session: PlacementSession) => void;
  saveDisplayName: (name: string) => Promise<void> | void;
  skipDisplayName: () => void;
};
