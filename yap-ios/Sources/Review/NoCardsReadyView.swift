import SwiftUI

struct NoCardsReadyView: View {
    let screen: ReviewScreenView
    let actions: ReviewActions
    let view: IdleScreenView
    @State private var showReleasePlan = false
    @AppStorage("yap-pimsleur-acknowledged") private var pimsleurAcknowledged = false
    private func addEvent(_ event: DeckEvent) { actions.addEvent(event) }
    var body: some View {
        Group {
            switch view {
            case let .AudioPending(_, online, _):
                StudyCard {
                    Text("Just a moment…").font(.title2.bold())
                    ProgressView("Downloading the audio for your next challenge.")
                    if !online { Text("Reconnect to download audio.") }
                }
            case let .ReviewPlanOffer(plan):
                ReviewPlanScreen(cards: plan.cards) { addEvent(plan.event) }
            case let .StudyPlanComplete(title, nextDue, plan):
                if showReleasePlan {
                    ReviewPlanScreen(cards: plan.cards) { addEvent(plan.event) }
                } else {
                    StudyCard {
                        Text(title).font(.title2.bold())
                        nextReview(nextDue)
                        Button("Study more") { showReleasePlan = true }.buttonStyle(.bordered).controlSize(.large)
                    }
                    #if DEBUG
                    .onChange(of: DebugHarness.shared.commandID) { _, _ in
                        guard DebugHarness.shared.activeTab == .learn else { return }
                        if DebugHarness.shared.command == "next" { showReleasePlan = true }
                    }
                    #endif
                }
            case let .Idle(idle): idleContent(idle)
            }
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            let command = DebugHarness.shared.command
            guard DebugHarness.shared.activeTab == .learn, command.hasPrefix("dump-fixture ") else { return }
            let capture: IdleScreenView
            if case let .StudyPlanComplete(_, _, plan) = view, showReleasePlan { capture = .ReviewPlanOffer(plan) }
            else { capture = view }
            var snapshot = screen
            snapshot.step = .Idle(capture)
            DebugHarness.dumpFixture(snapshot, name: String(command.dropFirst(13)))
        }
        #endif
    }
    @ViewBuilder private func idleContent(_ idle: IdleView) -> some View {
        let awaitingAcknowledgement = if case .PimsleurLesson = idle.navigation.selection { !pimsleurAcknowledged } else { false }
        StudyCard {
            Text(idle.title).font(.title2.bold())
            if !idle.body.isEmpty { Text(idle.body) } else { nextReview(idle.next_due) }
            if let notice = idle.banned_notice {
                Text(notice)
                Button("Undo restrictions") { actions.undoRestrictions() }
            }
            SentenceListSelector(actions: actions, view: idle)
            if !awaitingAcknowledgement, let event = idle.info.smart_add_event {
                Text(idle.info.preview.joined(separator: " · ")).foregroundStyle(.secondary)
                Button(idle.kind == .FirstRun ? "Start learning" : "Learn \(idle.info.smart_add_count) new cards") { addEvent(event) }
                    .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
            }
            if !awaitingAcknowledgement {
                DisclosureGroup("Choose cards to add") {
                    VStack(alignment: .leading, spacing: 12) {
                        ForEach(Array(idle.manual_add_options.enumerated()), id: \.offset) { _, option in
                            Button("Add \(option.count) \(label(option.card_type)) cards") {
                                if let event = option.event { addEvent(event) }
                            }.disabled(option.event == nil).frame(minHeight: 44)
                        }
                    }
                }
            }
            SnapshotGoalEditor(target: idle.target, goals: idle.goals, addEvent: addEvent)
        }
    }
    @ViewBuilder private func nextReview(_ card: CardSummary?) -> some View {
        if let card {
            Text("You'll review \(card.card_text) \(Date(timeIntervalSince1970: card.due_timestamp_ms / 1000), style: .relative).")
                .foregroundStyle(.secondary)
        }
    }
    private func label(_ type: CardType) -> String {
        switch type { case .TargetLanguage: "vocabulary"; case .Listening: "listening"; case .LetterPronunciation: "pronunciation" }
    }
}
