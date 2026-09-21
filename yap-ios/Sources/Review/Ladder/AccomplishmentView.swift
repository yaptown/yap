import SwiftUI

struct DailyGoalEditor: View {
    let deck: Deck
    let session: YapSession
    init(deck: Deck, session: YapSession) { self.deck = deck; self.session = session }
    @State private var expanded = false
    #if DEBUG
    @State private var visible = false
    #endif
    var body: some View {
        DisclosureGroup("Change daily goal", isExpanded: $expanded) {
            VStack(spacing: 12) {
                ForEach(get_daily_goal_options(), id: \.value) { goal in
                    Button {
                        session.addDeckEvent(deck.set_daily_review_target(daily_review_target: goal.value))
                    } label: {
                        HStack {
                            Text("\(goal.minutes) min/day — \(String(describing: goal.value))")
                            Spacer()
                            if deck.get_daily_review_target_setting() == goal.value { Image(systemName: "checkmark") }
                        }.frame(minHeight: 44)
                    }
                }
            }
        }
        #if DEBUG
        .onAppear { visible = true }.onDisappear { visible = false }
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard visible else { return }
            if DebugHarness.shared.command == "edit-goal" { expanded = true }
            // Learn routes this command through ReviewScreen; Stats owns its editor.
            let command = DebugHarness.shared.command
            if DebugHarness.shared.activeTab == .stats, command.hasPrefix("goal "), let index = Int(command.dropFirst(5)) {
                let goals = get_daily_goal_options()
                if goals.indices.contains(index) { session.addDeckEvent(deck.set_daily_review_target(daily_review_target: goals[index].value)) }
            }
        }
        #endif
    }
}

struct AccomplishmentScreen: View {
    let view: AccomplishmentView
    let addEvent: (DeckEvent) -> Void
    let onDismiss: () -> Void
    var body: some View {
        let summary = view.today
        StudyCard {
            Label("Goal reached!", systemImage: "trophy.fill").font(.title.bold())
            Text("You studied \(summary.time_spent_seconds / 60) min! (\(summary.reviews) challenges)")
            Text("\(view.streak) day streak").font(.headline)
            SnapshotGoalEditor(target: view.target, goals: view.goals, addEvent: addEvent)
            cards("New today", summary.new_cards)
            cards("Learned", summary.learned_cards)
            cards("Back on track", summary.locked_in_cards)
            if !summary.reviewed_words.isEmpty {
                Text("Reviewed (\(summary.reviewed_words.count))").font(.headline)
                Text(summary.reviewed_words.joined(separator: " · "))
            }
            if let recall = summary.recall_percent { Text("\(recall)% recall") }
            Text("\(view.words_known) words known")
            Button("Continue", action: dismiss).buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
        }
        .task {
            let now = Date()
            guard let midnight = Calendar.current.date(byAdding: .day, value: 1, to: Calendar.current.startOfDay(for: now)) else { return }
            do { try await Task.sleep(for: .seconds(midnight.timeIntervalSince(now))) } catch { return }
            dismiss()
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in if DebugHarness.shared.activeTab == .learn && DebugHarness.shared.command == "next" { dismiss() } }
        #endif
    }
    @ViewBuilder private func cards(_ title: String, _ cards: [TodayNewCard]) -> some View {
        if !cards.isEmpty {
            VStack(alignment: .leading, spacing: 12) {
                Text(title).font(.headline)
                ForEach(Array(cards.enumerated()), id: \.offset) { _, card in
                    VStack(alignment: .leading, spacing: 4) {
                        HStack {
                            Text(card.word).fontWeight(.semibold)
                            Spacer()
                            Text(card.card_type).font(.caption).foregroundStyle(.secondary)
                        }
                        Text(card.translation).foregroundStyle(.secondary)
                    }
                }
            }
        }
    }
    private func dismiss() { onDismiss() }
}

/// The review screens use captured goal options, including their actions.
struct SnapshotGoalEditor: View {
    let target: DailyReviewTarget
    let goals: [GoalOptionView]
    let addEvent: (DeckEvent) -> Void
    var body: some View {
        DisclosureGroup("Change daily goal") {
            ForEach(Array(goals.enumerated()), id: \.offset) { _, goal in
                Button("\(goal.minutes) min/day — \(String(describing: goal.target))") { addEvent(goal.event) }
                    .disabled(goal.target == target)
            }
        }
    }
}
