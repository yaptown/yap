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
            case let .StudyPlanComplete(title, nextReview, plan):
                if showReleasePlan {
                    ReviewPlanScreen(cards: plan.cards) { addEvent(plan.event) }
                } else {
                    StudyCard {
                        Text(title).font(.title2.bold())
                        if let nextReview { NextReviewLine(line: nextReview, language: plan.target_language) }
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
            if !idle.body.isEmpty { Text(idle.body) } else if let line = idle.next_review { NextReviewLine(line: line, language: idle.target_language) }
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

/// Rust builds the sentence; the emphasized run is the target-language word.
struct NextReviewLine: View {
    let line: EmphasizedText
    let language: Language
    var body: some View {
        (Text(line.before) + Text(line.emphasis).bold() + Text(line.after))
            .foregroundStyle(.secondary)
    }
}
