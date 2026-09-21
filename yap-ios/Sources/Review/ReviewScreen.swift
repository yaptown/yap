import SwiftUI

struct ReviewScreen: View {
    @Environment(AudioPlayer.self) private var audio
    let view: ReviewScreenView
    let actions: ReviewActions
    var body: some View {
        let metadata = get_language_metadata(language: view.target_language)
        VStack(spacing: 0) {
            ProgressView(value: view.progress)
                .progressViewStyle(.linear).tint(Color.yapAccent).frame(height: 3)
            ScrollView {
                VStack(alignment: .leading, spacing: 12) {
                    if !view.online { Label("Offline · changes stay on this device until you reconnect", systemImage: "wifi.slash").font(.caption).foregroundStyle(.secondary) }
                    if let error = actions.packError {
                        HStack {
                            Text("Couldn't finish downloading the language pack: \(error)").font(.caption).foregroundStyle(.secondary)
                            Button("Retry", action: actions.retryPack).font(.caption)
                        }
                    }
                    if let error = actions.syncError { Text("Sync will retry: \(error)").font(.caption).foregroundStyle(.secondary) }
                    if let error = actions.authError { Text(error).font(.caption).foregroundStyle(.secondary) }
                    switch view.step {
                    case .PlacementTest: PlacementTestView(actions: actions)
                    case let .ReviewPlan(plan): ReviewPlanScreen(cards: plan.cards) { actions.addEvent(plan.event) }
                    case .SetDisplayName: SetDisplayNameView(reviewCount: view.total_reviews, actions: actions)
                    case let .Accomplishment(accomplishment): AccomplishmentScreen(view: accomplishment, addEvent: actions.addEvent, onDismiss: actions.dismissAccomplishment)
                    case let .Challenge(challenge): challengeView(challenge).id(challenge.challenge)
                    case let .Idle(idle): NoCardsReadyView(screen: view, actions: actions, view: idle)
                    }
                }.padding(12).frame(maxWidth: 600)
            }
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .principal) {
                HStack(spacing: 8) {
                    if Theme.emojiFontAvailable { Text(metadata.flag) }
                    else { Image(systemName: "globe").foregroundStyle(Color.yapAccent) }
                    Text(metadata.common_name).foregroundStyle(Color.yapText)
                }.font(.headline)
            }
            ToolbarItem(placement: .topBarTrailing) {
                Button("Switch course", systemImage: "globe", action: actions.switchCourse)
                    .labelStyle(.iconOnly).frame(width: 44, height: 44)
            }
        }
        .onDisappear { audio.stop() }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeTab == .learn else { return }
            let command = DebugHarness.shared.command
            if command == "dismiss-keyboard" { UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil) }
            guard command.hasPrefix("dump-fixture ") else { return }
            // Reducer hosts and idle capture their local state themselves.
            if case let .Challenge(challenge) = view.step {
                if case .TranslateComprehensibleSentence = challenge.challenge { return }
                if case .TranscribeComprehensibleSentence = challenge.challenge { return }
            }
            if case .Idle = view.step { return }
            DebugHarness.dumpFixture(view, name: String(command.dropFirst(13)))
        }
        #endif
    }
    @ViewBuilder private func challengeView(_ challenge: ChallengeView) -> some View {
        switch challenge.challenge {
        case let .FlashCardReview(indicator, flashcard, isNew, timesSeen):
            FlashcardView(screen: view, actions: actions, indicator: indicator, flashcard: flashcard, isNew: isNew, timesTypeSeen: timesSeen)
        case let .PronunciationChallenge(indicator, pattern, guide, cues, isNew, timesSeen):
            PronunciationChallengeView(screen: view, actions: actions, indicator: indicator, pattern: pattern, guide: guide, cues: cues, isNew: isNew, timesSeen: timesSeen)
        case let .TranslateComprehensibleSentence(sentence):
            TranslationChallengeView(screen: view, actions: actions, sentence: sentence, initialState: challenge.translation)
        case let .TranscribeComprehensibleSentence(sentence):
            TranscriptionChallengeView(screen: view, actions: actions, sentence: sentence, initialState: challenge.transcription)
        }
    }
}
