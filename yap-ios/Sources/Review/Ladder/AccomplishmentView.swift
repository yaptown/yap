import SwiftUI

struct DailyGoalEditor: View {
    let target: DailyReviewTarget
    let options: [GoalOptionView]
    let addEvent: (DeckEvent) -> Void
    @State private var pendingTarget: DailyReviewTarget
    init(target: DailyReviewTarget, options: [GoalOptionView], addEvent: @escaping (DeckEvent) -> Void) {
        self.target = target; self.options = options; self.addEvent = addEvent
        _pendingTarget = State(initialValue: target)
    }
    var body: some View {
        VStack(spacing: 12) {
            Picker("Daily goal", selection: $pendingTarget) {
                ForEach(options, id: \.target) { goal in
                    Text("\(String(describing: goal.target)) · \(goal.minutes)m").tag(goal.target)
                }
            }.pickerStyle(.menu)
            Button("Set goal") {
                if let goal = options.first(where: { $0.target == pendingTarget }) { addEvent(goal.event) }
            }.buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).disabled(pendingTarget == target)
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeScreen == .goals else { return }
            let command = DebugHarness.shared.command
            if command.hasPrefix("goal "), let index = Int(command.dropFirst(5)), options.indices.contains(index) {
                addEvent(options[index].event)
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
        .onChange(of: DebugHarness.shared.commandID) { _, _ in if DebugHarness.shared.activeScreen == .review && DebugHarness.shared.command == "next" { dismiss() } }
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
