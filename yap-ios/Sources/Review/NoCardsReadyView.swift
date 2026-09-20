import SwiftUI

struct NoCardsReadyView: View {
    let model: ReviewModel
    @State private var options: [ManualAddOption] = []
    @State private var showReleasePlan = false
    @AppStorage("yap-pimsleur-acknowledged") private var pimsleurAcknowledged = false
    private var awaitingAcknowledgement: Bool {
        if case .PimsleurLesson = model.deck.get_sentence_list() { !pimsleurAcknowledged } else { false }
    }
    private var nextDue: CardSummary? { model.deck.get_all_cards_summary().filter { $0.due_timestamp_ms > ReviewModel.now }.min { $0.due_timestamp_ms < $1.due_timestamp_ms } }
    var body: some View {
        let info = model.deck.get_no_cards_ready_info(banned_challenge_types: model.banned, sentence_list: model.deck.get_sentence_list())
        let idle = get_idle_study_state(has_future_card: nextDue != nil, cards_added: model.deck.num_cards_added(), smart_add_count: info.smart_add_count)
        if model.reviewInfo.due_but_audio_pending_count > 0 {
            StudyCard {
                Text("Just a moment…").font(.title2.bold())
                ProgressView("Downloading the audio for your next challenge.")
                if !model.session.online { Text("Reconnect to download audio.") }
            }
        } else if let offer = model.deck.get_release_offer(timestamp_ms: ReviewModel.now) {
            if showReleasePlan || model.deck.get_today_time_spent() == 0 || !model.deck.study_plan_was_recently_accepted(timestamp_ms: ReviewModel.now) {
                ReviewPlanView(cards: offer.release_preview) { model.session.addDeckEvent(offer.unlock_event) }
            } else {
                StudyCard {
                    Text("You completed the study plan!").font(.title2.bold())
                    Text("You studied for \(Int(model.deck.get_today_time_spent()) / 60) minutes today.")
                    nextReview
                    Button("Study more") { showReleasePlan = true }.buttonStyle(.bordered).controlSize(.large)
                }
                #if DEBUG
                .onChange(of: DebugHarness.shared.commandID) { _, _ in guard DebugHarness.shared.activeTab == .learn else { return }; if DebugHarness.shared.command == "next" { showReleasePlan = true } }
                #endif
            }
        } else {
            StudyCard {
                Text(idle.nothing_to_do ? "All done!" : idle.has_never_studied ? "Ready to start learning?" : idle.no_schedulable_cards ? (info.smart_add_regime == .Easy ? "Adding cards is how you learn more!" : "Ready for more?") : "All caught up!").font(.title2.bold())
                if idle.nothing_to_do { Text("You've learned all available words!") }
                else if idle.has_never_studied { Text("We'll start with a couple words you might know.") }
                else if idle.no_schedulable_cards {
                    Text(info.smart_add_regime == .Easy && info.easy_cards_remaining > 0 ? "\(info.easy_cards_remaining) more easy words, then we'll add harder ones." : "Add some cards to keep building your vocabulary.")
                } else { nextReview }
                if model.reviewInfo.due_but_banned_count > 0 {
                    Text("\(model.reviewInfo.due_but_banned_count) cards paused by listening/speaking restrictions")
                    Button("Undo restrictions") { model.undoRestrictions() }
                }
                SentenceListSelector(model: model, info: info)
                if !awaitingAcknowledgement, let event = info.smart_add_event {
                    Text(info.preview.joined(separator: " · ")).foregroundStyle(.secondary)
                    Button(idle.has_never_studied ? "Start learning" : "Learn \(info.smart_add_count) new cards") { model.session.addDeckEvent(event) }
                        .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
                }
                if !awaitingAcknowledgement {
                    DisclosureGroup("Choose cards to add") {
                        VStack(alignment: .leading, spacing: 12) {
                            ForEach(Array(options.enumerated()), id: \.offset) { _, option in
                                Button("Add \(option.count) \(label(option.card_type)) cards") {
                                    if let event = option.event { model.session.addDeckEvent(event) }
                                }.disabled(option.event == nil).frame(minHeight: 44)
                            }
                        }.task { options = model.deck.get_manual_add_options(sentence_list: model.deck.get_sentence_list(), is_signed_in: true) }
                    }
                }
                DailyGoalEditor(model: model)
            }
        }
    }
    @ViewBuilder private var nextReview: some View {
        if let card = nextDue {
            Text("You'll review \(card.card_text) \(Date(timeIntervalSince1970: card.due_timestamp_ms / 1000), style: .relative).")
                .foregroundStyle(.secondary)
        }
    }
    private func label(_ type: CardType) -> String {
        switch type { case .TargetLanguage: "vocabulary"; case .Listening: "listening"; case .LetterPronunciation: "pronunciation" }
    }
}
