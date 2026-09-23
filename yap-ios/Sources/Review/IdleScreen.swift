import SwiftUI

struct IdleScreen: View {
    @Environment(\.reviewScreen) private var screen: ReviewScreenView?
    @Environment(\.reviewActions!) private var actions
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
                        if let nextDue { NextReviewLine(card: nextDue) }
                        Button("Study more") { showReleasePlan = true }.buttonStyle(.bordered).controlSize(.large)
                        WeekProgressStrip(week: plan.week)
                    }
                    #if DEBUG
                    .onChange(of: DebugHarness.shared.commandID) { _, _ in
                        guard DebugHarness.shared.activeScreen == .review else { return }
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
            guard DebugHarness.shared.activeScreen == .review, command.hasPrefix("dump-fixture ") else { return }
            guard let screen else { return }
            let capture: IdleScreenView
            if case let .StudyPlanComplete(_, _, plan) = view, showReleasePlan { capture = .ReviewPlanOffer(plan) }
            else { capture = view }
            var snapshot = screen
            snapshot.step = .Idle(capture)
            DebugHarness.dumpFixture(.Review(snapshot), name: String(command.dropFirst(13)))
        }
        #endif
    }
    @ViewBuilder private func idleContent(_ idle: IdleView) -> some View {
        let awaitingAcknowledgement = if case .PimsleurLesson = idle.navigation.selection { !pimsleurAcknowledged } else { false }
        StudyCard {
            Text(idle.title).font(.title2.bold())
            if !idle.body.isEmpty { Text(idle.body) } else if let card = idle.next_due { NextReviewLine(card: card) }
            if let notice = idle.banned_notice {
                Text(notice)
                Button("Undo restrictions") { actions.undoRestrictions() }
            }
            if let label = idle.smart_add_label, let event = idle.info.smart_add_event {
                Button(label) { addEvent(event) }
                    .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
            }
            if idle.show_sentence_list {
                SentenceListSelector(view: idle)
                if let commit = idle.switch_curriculum {
                    Button(commit.label) { actions.commitSentenceList(commit.event) }
                        .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
                }
                if !awaitingAcknowledgement, let label = idle.curriculum_learn_label, let event = idle.info.smart_add_event {
                    Button(label) { addEvent(event) }
                        .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
                }
                if !awaitingAcknowledgement {
                    DisclosureGroup(idle.manual_add_heading) {
                        VStack(alignment: .leading, spacing: 12) {
                            ForEach(Array(idle.manual_add_options.enumerated()), id: \.offset) { _, option in
                                Button(option.label) {
                                    addEvent(option.event)
                                }.frame(minHeight: 44)
                            }
                        }
                    }
                }
                WeekProgressStrip(week: idle.week)
            }
        }
    }
}

/// Rust builds the sentence, rounds the countdown, and says when the words
/// would next change; this schedules exactly that one refresh.
struct NextReviewLine: View {
    let card: CardSummary
    @State private var now = Date()
    var body: some View {
        let live = next_review_line(card: card, now_ms: now.timeIntervalSince1970 * 1000)
        (Text(live.text.before) + Text(live.text.emphasis).bold() + Text(live.text.after))
            .foregroundStyle(.secondary)
            .onChange(of: card.due_timestamp_ms) { _, _ in now = Date() }
            .task(id: live.refresh_at_ms) {
                let delay = live.refresh_at_ms / 1000 - Date().timeIntervalSince1970
                try? await Task.sleep(for: .seconds(max(0, delay)))
                if !Task.isCancelled { now = Date() }
            }
    }
}
