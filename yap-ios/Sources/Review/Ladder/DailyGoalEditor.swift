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
