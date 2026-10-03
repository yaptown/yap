import type {
  CardContent,
  CardReview,
  Rating,
} from "../../../../yap-frontend-rs/pkg";

/// The explicit reviews a flashcard grade records: every card behind each
/// meaning row gets that row's rating; other flashcards rate just their own
/// card. Type-only imports, so the MCP widget can share it.
export function flashcardReviews(
  content: CardContent,
  card: CardReview["card"],
  rating: Rating,
  meaningRatings: Rating[] | undefined,
): CardReview[] {
  if (content.type === "Gram" && meaningRatings) {
    return content.meanings.flatMap((meaning, index) =>
      meaning.cards.map((card) => ({ card, rating: meaningRatings[index] })),
    );
  }
  return [{ card, rating }];
}
