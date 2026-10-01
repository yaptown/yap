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
            // Every option in view at once, like the web's segmented row.
            HStack(spacing: 0) {
                ForEach(Array(options.enumerated()), id: \.offset) { index, goal in
                    if index > 0 { Rectangle().fill(Color.yapBorder).frame(width: 1) }
                    let selected = goal.target == pendingTarget
                    Button { pendingTarget = goal.target } label: {
                        VStack(spacing: 2) {
                            Text(goal.label).font(.subheadline.weight(.medium))
                            Text(goal.duration_label).font(.caption).opacity(0.7)
                        }
                        .lineLimit(1).minimumScaleFactor(0.8)
                        .frame(maxWidth: .infinity, maxHeight: .infinity).padding(.vertical, 8).padding(.horizontal, 4)
                        .foregroundStyle(selected ? Color.yapOnAccent : Color.yapText)
                        .background(selected ? Color.yapAccent : .clear)
                        .contentShape(Rectangle())
                    }.buttonStyle(.plain).accessibilityAddTraits(selected ? .isSelected : [])
                }
            }
            .fixedSize(horizontal: false, vertical: true)
            .clipShape(RoundedRectangle(cornerRadius: 8))
            .overlay { RoundedRectangle(cornerRadius: 8).strokeBorder(Color.yapBorder) }
            Button("Set goal") {
                if let goal = options.first(where: { $0.target == pendingTarget }) { addEvent(goal.event) }
            }
            .font(.subheadline.weight(.medium)).foregroundStyle(Color.yapOnAccent)
            .padding(.horizontal, 14).frame(minHeight: 36)
            .background(Color.yapAccent, in: RoundedRectangle(cornerRadius: 8))
            // The web dims the whole button rather than graying it out.
            .opacity(pendingTarget == target ? 0.5 : 1)
            .disabled(pendingTarget == target)
        }.frame(maxWidth: .infinity)
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
                Button("\(goal.label) · \(goal.duration_label)") { addEvent(goal.event) }
                    .disabled(goal.target == target)
            }
        }
    }
}
