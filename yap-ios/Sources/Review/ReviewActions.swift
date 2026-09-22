import SwiftUI

@MainActor final class AutoplayClaim { var reviewCount: UInt64? }

@MainActor struct ReviewHost {
    let deck: Deck
    let weapon: Weapon?
    let online: Bool
    let accessToken: String?
    let autoplay: AutoplayClaim
    let packBanner: PackBanner?
    let syncError: String?
    let authError: String?
}

extension EnvironmentValues {
    @Entry var reviewHost: ReviewHost?
    @Entry var reviewScreen: ReviewScreenView?
    @Entry var reviewActions: ReviewActions?
}

@MainActor struct ReviewActions {
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
    /// Changing the active curriculum. Kept separate from `addEvent` because it
    /// does not add cards — Home wraps `addEvent` to jump into Review, and a
    /// curriculum change must not trigger that jump (mirrors web's split props).
    var setSentenceList: (DeckEvent) -> Void = { _ in log("set sentence list") }
    var dismissAccomplishment: () -> Void = { log("dismiss accomplishment") }
    var setPlacement: (PlacementSession) -> Void = { _ in log("set placement") }
    var completePlacementTest: (PlacementSession) -> Void = { _ in log("complete placement") }
    var retryPack: () -> Void = { log("retry pack") }
    var switchCourse: () -> Void = { log("switch course") }
    var saveDisplayName: (String) async throws -> Void = { _ in log("save display name") }
    var skipDisplayName: () -> Void = { log("dismiss display name") }
    static var inert: Self { Self() }
    private static func log(_ action: String) {
        print("fixture action \(action)")
    }
}
