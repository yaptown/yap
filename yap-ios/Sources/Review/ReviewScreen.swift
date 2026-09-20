import SwiftUI

struct ReviewScreen: View {
    @Environment(AuthStore.self) private var auth
    @Environment(AudioPlayer.self) private var audio
    let model: ReviewModel
    @AppStorage("yap-skipped-set-display-name") private var displayNameDismissed = false
    private var step: ReviewStep? {
        #if DEBUG
        if DebugHarness.shared.fixture != nil { return nil }
        #endif
        let session = model.session
        if model.deck.should_offer_placement_test(starting_fresh: model.startingFresh, history_known: model.historyKnown) { return .placementTest }
        if model.lockupOffer != nil { return .lockupOffer }
        #if DEBUG
        if DebugHarness.shared.forceDisplayName && auth.needsDisplayName && !displayNameDismissed { return .setDisplayName }
        #endif
        let prompts = get_review_prompts(total_reviews_completed: model.deck.get_total_reviews(), total_card_count: model.reviewInfo.total_count,
            context: ReviewPromptContext(is_idle: model.reviewInfo.due_count == 0 && model.currentChallenge == nil,
                is_online: session.online, is_signed_in: true, needs_display_name: auth.needsDisplayName,
                display_name_dismissed: displayNameDismissed, has_access_token: auth.accessToken != nil))
        if prompts.offer_display_name { return .setDisplayName }
        if model.accomplishmentView != nil, session.dismissedAccomplishmentAtReview != model.deck.get_total_reviews() { return .accomplishment }
        return nil
    }
    var body: some View {
        let metadata = get_language_metadata(language: model.deck.get_target_language())
        VStack(spacing: 0) {
            ProgressView(value: min(Double(model.deck.get_today_time_spent()) / max(Double(model.deck.get_daily_review_target()), 1), 1))
                .progressViewStyle(.linear).tint(Color.yapAccent).frame(height: 3)
            ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                if !model.session.online { Label("Offline · changes stay on this device until you reconnect", systemImage: "wifi.slash").font(.caption).foregroundStyle(.secondary) }
                if let error = model.session.packError {
                    HStack {
                        Text("Couldn't finish downloading the language pack: \(error)").font(.caption).foregroundStyle(.secondary)
                        Button("Retry") { model.session.retry() }.font(.caption)
                    }
                }
                if let error = model.session.syncError {
                    Text("Sync will retry: \(error)").font(.caption).foregroundStyle(.secondary)
                }
                if let error = auth.error { Text(error).font(.caption).foregroundStyle(.secondary) }
                if hasFixture {
                    fixtureView
                } else if let step {
                    switch step {
                    case .placementTest: PlacementTestView(model: model)
                    case .lockupOffer:
                        if let offer = model.lockupOffer { ReviewPlanScreen(cards: offer.cards) { model.session.addDeckEvent(offer.event) } }
                    case .setDisplayName: SetDisplayNameView(reviewCount: model.deck.get_total_reviews())
                    case .accomplishment:
                        if let view = model.accomplishmentView { AccomplishmentScreen(view: view, addEvent: model.session.addDeckEvent, onDismiss: dismissAccomplishment) }
                    }
                } else if let challenge = model.currentChallenge {
                    challengeView(challenge).id(challenge)
                } else if let view = model.idleView { NoCardsReadyView(model: model, view: view) }
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
                Button("Switch course", systemImage: "globe") { model.session.choosingCourse = true }
                    .labelStyle(.iconOnly).frame(width: 44, height: 44)
            }
        }
        .onDisappear { audio.stop() }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in guard DebugHarness.shared.activeTab == .learn else { return }; handleDebugCommand() }
        #endif
    }
    private func dismissAccomplishment() { model.session.dismissedAccomplishmentAtReview = model.deck.get_total_reviews() }
    private var hasFixture: Bool {
        #if DEBUG
        DebugHarness.shared.fixture != nil
        #else
        false
        #endif
    }
    @ViewBuilder private var fixtureView: some View {
        #if DEBUG
        Group {
            switch DebugHarness.shared.fixture {
            case let .Challenge(view): challengeView(view.challenge)
            case let .Idle(view): NoCardsReadyView(model: model, view: view, isFixture: true)
            case let .Accomplishment(view): AccomplishmentScreen(view: view, addEvent: { _ in }, onDismiss: {})
            case nil: EmptyView()
            }
        }.onAppear { DebugHarness.log("fixture rendered \(DebugHarness.shared.fixtureName)") }
        #endif
    }
    @ViewBuilder private func challengeView(_ challenge: Challenge_Gram_String) -> some View {
        switch challenge {
        case let .FlashCardReview(indicator, flashcard, isNew, timesSeen):
            FlashcardView(model: model, indicator: indicator, flashcard: flashcard, isNew: isNew, timesTypeSeen: timesSeen)
        case let .PronunciationChallenge(indicator, pattern, guide, cues, isNew, timesSeen):
            PronunciationChallengeView(model: model, indicator: indicator, pattern: pattern, guide: guide, cues: cues, isNew: isNew, timesSeen: timesSeen)
        case let .TranslateComprehensibleSentence(sentence):
            if let course = model.course { TranslationChallengeView(model: model, sentence: sentence, course: course) }
        case let .TranscribeComprehensibleSentence(sentence):
            TranscriptionChallengeView(model: model, sentence: sentence)
        }
    }
    #if DEBUG
    private func handleDebugCommand() {
        switch DebugHarness.shared.command {
        case let command where command.hasPrefix("dump-fixture "):
            let name = String(command.dropFirst(13))
            if case let .Accomplishment(view) = DebugHarness.shared.fixture {
                DebugHarness.dumpFixture(.Accomplishment(view), name: name)
                return
            }
            if step == .lockupOffer, let plan = model.lockupOffer {
                DebugHarness.dumpFixture(.Idle(.ReviewPlanOffer(plan)), name: name)
            } else if step == .accomplishment, let view = model.accomplishmentView {
                DebugHarness.dumpFixture(.Accomplishment(view), name: name)
            } else if step == nil, let challenge = model.currentChallenge {
                if case .TranscribeComprehensibleSentence = challenge { return }
                if case .TranslateComprehensibleSentence = challenge { return }
                DebugHarness.dumpFixture(.Challenge(ChallengeFixture(challenge: challenge, transcription: nil, translation: nil)), name: name)
            } else if step == nil {
                // The mounted idle view captures its local plan-expansion state.
                return
            } else { DebugHarness.log("fixture unsupported step \(step?.rawValue ?? "none")") }
        case "dismiss-keyboard": UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
        case "status":
            DebugHarness.log("placement: startingFresh=\(String(describing: model.startingFresh)) historyKnown=\(model.historyKnown) taken=\(model.deck.has_taken_placement_test()) list=\(String(describing: model.deck.get_sentence_list()))")
            DebugHarness.log("today: seconds=\(model.deck.get_today_time_spent()) target=\(model.deck.get_daily_review_target())")
            let info = model.reviewInfo
            DebugHarness.log("review: due=\(info.due_count) total=\(info.total_count) banned=\(info.due_but_banned_count) audioPending=\(info.due_but_audio_pending_count) locked=\(info.due_but_locked_count) step=\(step?.rawValue ?? "none")")
            switch model.currentChallenge {
            case let .PronunciationChallenge(_, pattern, _, _, _, _): DebugHarness.log("challenge=pronunciation \(pattern)")
            case let .TranslateComprehensibleSentence(sentence): DebugHarness.log("challenge=translation \(sentence.target_language)")
            case let .TranscribeComprehensibleSentence(sentence): DebugHarness.log("challenge=transcription \(sentence.target_language)")
            case let .FlashCardReview(indicator, _, _, _): DebugHarness.log("challenge=flashcard \(String(describing: indicator))")
            case nil: DebugHarness.log("challenge=none")
            }
        case let command where command.hasPrefix("goal "):
            let options = get_daily_goal_options()
            if let i = Int(command.dropFirst(5)), options.indices.contains(i) {
                model.session.addDeckEvent(model.deck.set_daily_review_target(daily_review_target: options[i].value))
            }
        case "cant-listen": model.cantListen()
        case "dismiss-step": model.session.dismissedAccomplishmentAtReview = model.deck.get_total_reviews()
        case "force-translation": model.forceTranslation()
        case "cant-speak": model.cantSpeak()
        case "undo": model.undoRestrictions()
        case "add-listening", "add-pronunciation":
            let type: CardType = DebugHarness.shared.command == "add-listening" ? .Listening : .LetterPronunciation
            let options = model.deck.get_manual_add_options(sentence_list: model.deck.get_sentence_list(), is_signed_in: true)
            if let event = options.first(where: { $0.card_type == type })?.event { model.session.addDeckEvent(event) }
        case "add":
            guard model.currentChallenge == nil, step == nil else { return }
            if case let .Idle(view) = model.idleView, let event = view.info.smart_add_event { model.session.addDeckEvent(event) }
        default: break
        }
    }
    #endif
}
