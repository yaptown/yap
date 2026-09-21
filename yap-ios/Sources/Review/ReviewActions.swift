import Foundation

/// Host capabilities, kept separate from the captured screen's data.
@MainActor struct ReviewMedia {
    var accessToken: String?
    var claimAutoplay: (UInt64) -> Bool = { _ in false }
    var releaseAutoplay: () -> Void = {}
    var movieMetadata: (String) -> MovieMetadataBasic? = { _ in nil }
    var moviePoster: (String) -> [UInt8]? = { _ in nil }

    static func live(deck: Deck, session: YapSession) -> Self {
        Self(accessToken: session.accessToken(), claimAutoplay: { count in
            guard session.lastAutoPlayReviewCount != count else { return false }
            session.lastAutoPlayReviewCount = count
            return true
        }, releaseAutoplay: { session.lastAutoPlayReviewCount = nil },
        movieMetadata: { deck.get_movie_metadata(movie_ids: [$0]).first },
        moviePoster: { deck.get_movie_poster(movie_id: $0) })
    }
}

@MainActor struct ReviewActions {
    var media = ReviewMedia()
    var nativeLanguage: Language = .English
    var submitting = false
    /// Account/course scope; nil means reducer drafts cannot touch persistence.
    var pendingReviewKey: String?
    var rate: (CardIndicator_Gram_String_String, Rating) -> Void = { _, _ in log("rate") }
    var completeTranslationPerfect: (String, [Heteronym_String], Double) -> Bool = { _, _, _ in log("translation perfect"); return false }
    var completeTranslationWrong: (String, String, ManualTranslationGrade, [Heteronym_String], Double) -> Bool = { _, _, _, _, _ in log("translation wrong"); return false }
    var completeTranscription: ([PartGraded], Double) -> Bool = { _, _ in log("transcription"); return false }
    var cantListen: () -> Void = { log("can't listen") }
    var cantSpeak: () -> Void = { log("can't speak") }
    var undoRestrictions: () -> Void = { log("undo restrictions") }
    var addEvent: (DeckEvent) -> Void = { _ in log("add event") }
    var dismissAccomplishment: () -> Void = { log("dismiss accomplishment") }
    var startPlacement: () -> PlacementSession? = { log("start placement"); return nil }
    var advancePlacement: (PlacementSession) -> PlacementSession = { log("advance placement"); return $0 }
    var savePlacement: (PlacementSession) -> Void = { _ in log("save placement") }
    var placement: PlacementSession?
    var completePlacementTest: (PlacementSession) -> Void = { _ in log("complete placement") }
    var retryPack: () -> Void = { log("retry pack") }
    var switchCourse: () -> Void = { log("switch course") }
    var saveDisplayName: (String) async throws -> Void = { _ in log("save display name") }
    var dismissDisplayName: () -> Void = { log("dismiss display name") }
    var packError: String?
    var syncError: String?
    var authError: String?
    static var inert: Self { Self() }
    private static func log(_ action: String) {
        #if DEBUG
        DebugHarness.log("fixture action \(action)")
        #endif
    }
}
