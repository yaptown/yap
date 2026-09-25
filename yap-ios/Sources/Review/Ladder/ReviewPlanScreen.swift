import SwiftUI

/// Both initial lockup and subsequent release offers commit the shared Rust
/// plan. Mirrors web's `ReviewPlanCard`: centered title, one heading per card
/// type over its words separated by hairlines, a full-width commit button,
/// and the week strip pinned to the bottom of the screen.
struct ReviewPlanScreen: View {
    let plan: ReviewPlanView
    let onAccept: () -> Void
    var body: some View {
        ReviewStepScrollView {
            StudyCard(alignment: .center, spacing: 24, animated: true) {
                Text(plan.title).font(.title2.weight(.semibold)).multilineTextAlignment(.center)
                ForEach(plan.groups, id: \.heading) { group in
                    VStack(spacing: 8) {
                        Text(group.heading).font(.subheadline.weight(.medium)).foregroundStyle(.secondary)
                        SentenceFlow(spacing: 0, alignment: .center) {
                            ForEach(Array(group.cards.enumerated()), id: \.offset) { index, card in
                                Text(card).font(.subheadline.weight(.medium))
                                    .padding(.horizontal, 12).padding(.vertical, 2)
                                    .overlay(alignment: .leading) {
                                        if index > 0 { Rectangle().fill(Color(uiColor: .separator)).frame(width: 1) }
                                    }
                            }
                        }
                    }
                }
                Button(action: onAccept) { Text(plan.accept_label).frame(maxWidth: .infinity) }
                    .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
            }
            .padding(.horizontal, 8)
        } actions: {
            WeekProgressStrip(week: plan.week)
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in guard DebugHarness.shared.activeScreen == .review else { return }; if DebugHarness.shared.command == "next" { onAccept() } }
        #endif
    }
}
