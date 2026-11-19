import SwiftUI

/// Asks to turn on reminders, both during onboarding and on the review screen
/// once the learner runs out of cards.
struct NotificationPromptCard: View {
    let icon: String
    let title: String
    let message: String
    let enableLabel: String
    let skipLabel: String
    let enable: () async -> Void
    let skip: () -> Void
    @State private var enabling = false
    var body: some View {
        StudyCard(alignment: .center, spacing: 16, animated: true) {
            Image(icon).resizable().scaledToFit().frame(width: 112, height: 112)
            Text(title).font(.title2.bold()).multilineTextAlignment(.center).fixedSize(horizontal: false, vertical: true)
            Text(message).foregroundStyle(Color.yapMuted).multilineTextAlignment(.center)
            VStack(spacing: 8) {
                Button {
                    enabling = true
                    Task { await enable(); enabling = false }
                } label: { Text(enableLabel).frame(maxWidth: .infinity) }
                    .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent)
                Button(action: skip) { Text(skipLabel).frame(maxWidth: .infinity) }
                    // Clear tint keeps the bordered size and padding without the fill, matching the button above.
                    .buttonStyle(.bordered).tint(.clear).foregroundStyle(Color.yapAccent)
            }.controlSize(.large).disabled(enabling)
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            if DebugHarness.shared.command == "enable-notifications" { Task { await enable() } }
            if DebugHarness.shared.command == "skip-notifications" { skip() }
        }
        #endif
    }
}

extension NotificationPromptCard {
    init(_ view: NotificationPromptView, actions: ReviewActions) {
        self.init(icon: view.icon, title: view.title, message: view.body, enableLabel: view.enable_label,
                  skipLabel: view.skip_label, enable: actions.enableNotifications, skip: actions.skipNotifications)
    }
}
