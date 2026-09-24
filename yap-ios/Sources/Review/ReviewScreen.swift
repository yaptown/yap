import SwiftUI

struct ReviewScreen: View {
    @Environment(AuthSheet.self) private var authSheet
    @Environment(AudioPlayer.self) private var audio
    @Environment(\.reviewHost!) private var host
    let view: ReviewScreenView
    @Environment(\.reviewActions!) private var actions
    var body: some View {
        VStack(spacing: 0) {
            ProgressView(value: view.progress)
                .progressViewStyle(.linear).tint(Color.yapAccent).frame(height: 3)
            VStack(alignment: .leading, spacing: 12) {
                if view.show_account_prompt {
                    let copy = account_copy()
                    StudyCard {
                        Text(copy.prompt_title).font(.subheadline).foregroundStyle(.secondary)
                        Text(copy.prompt_body).font(.caption).foregroundStyle(.secondary)
                        Button(copy.prompt_action) { authSheet.present(tab: .signUp) }
                            .buttonStyle(.bordered)
                    }.padding(.top, 12)
                }
                TimelineView(.periodic(from: .now, by: 1)) { context in
                    if let weapon = host.weapon {
                        let sync = weapon.sync_status(online: host.online, now_ms: context.date.timeIntervalSince1970 * 1000,
                            manual_sync_in_flight: false, host_sync_error: host.syncError)
                        VStack(alignment: .leading, spacing: 12) {
                            if let banner = sync.offline_banner { Label(banner, systemImage: "wifi.slash").font(.caption).foregroundStyle(.secondary) }
                            if let error = sync.error { Text(error).font(.caption).foregroundStyle(Color.yapNegativeForeground) }
                        }
                    }
                }
                if let banner = host.packBanner {
                    HStack {
                        Text(banner.message).font(.caption).foregroundStyle(.secondary)
                        Button(banner.retry_label, action: actions.retryPack).font(.caption)
                    }
                }
                if let error = host.authError { Text(error).font(.caption).foregroundStyle(.secondary) }
            }.padding(.horizontal, 12).frame(maxWidth: 600)
            switch view.step {
            case let .Challenge(challenge): challengeView(challenge).id(challenge.challenge)
            case let .Accomplishment(accomplishment): AccomplishmentScreen(view: accomplishment, addEvent: actions.addEvent, onDismiss: actions.dismissAccomplishment)
            case let .ReviewPlan(plan): ReviewPlanScreen(plan: plan) { actions.addEvent(plan.event) }
            case let .Idle(idle): IdleScreen(view: idle)
            default:
                ScrollView {
                    VStack(alignment: .leading, spacing: 12) {
                        switch view.step {
                        case let .PlacementTest(placement): PlacementTestView(placement: placement)
                        case .SetDisplayName: SetDisplayNameView(reviewCount: view.total_reviews)
                        default: EmptyView()
                        }
                    }.padding(12).frame(maxWidth: 600).frame(maxWidth: .infinity)
                }
            }
        }
        .environment(\.reviewScreen, view)
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle("Review")
        .navigationBarTitleDisplayMode(.inline)
        .onDisappear { audio.stop() }
        #if DEBUG
        .onAppear { DebugHarness.shared.activeScreen = .review }
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeScreen == .review else { return }
            let command = DebugHarness.shared.command
            if command == "dismiss-keyboard" { UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil) }
            guard command.hasPrefix("dump-fixture ") else { return }
            // Reducer hosts and idle capture their local state themselves.
            if case let .Challenge(challenge) = view.step {
                if case .TranslateComprehensibleSentence = challenge.challenge { return }
                if case .TranscribeComprehensibleSentence = challenge.challenge { return }
            }
            if case .Idle = view.step { return }
            DebugHarness.dumpFixture(.Review(view), name: String(command.dropFirst(13)))
        }
        #endif
    }
    @ViewBuilder private func challengeView(_ challenge: ChallengeView) -> some View {
        switch challenge.challenge {
        case let .FlashCardReview(indicator, flashcard, isNew, timesSeen):
            FlashcardChallengeView(indicator: indicator, flashcard: flashcard, isNew: isNew, timesTypeSeen: timesSeen)
        case let .PronunciationChallenge(indicator, pattern, guide, cues, isNew, timesSeen):
            PronunciationChallengeView(indicator: indicator, pattern: pattern, guide: guide, cues: cues, isNew: isNew, timesSeen: timesSeen)
        case let .TranslateComprehensibleSentence(sentence):
            TranslationChallengeView(sentence: sentence, initialState: challenge.translation ?? translation_start(sentence: sentence, course: Course(native_language: view.native_language, target_language: view.target_language)))
        case let .TranscribeComprehensibleSentence(sentence):
            TranscriptionChallengeView(sentence: sentence, initialState: challenge.transcription)
        }
    }
}

struct ReviewStepScrollView<Content: View, Actions: View>: View {
    @ViewBuilder let content: () -> Content
    @ViewBuilder let actions: () -> Actions
    var body: some View {
        ScrollView {
            VStack(spacing: 12, content: content).padding(12).frame(maxWidth: 600).frame(maxWidth: .infinity)
        }.safeAreaInset(edge: .bottom, spacing: 0) {
            VStack(spacing: 12, content: actions).padding(12).frame(maxWidth: 600).frame(maxWidth: .infinity)
                .background(Color(uiColor: .systemGroupedBackground))
        }
    }
}
