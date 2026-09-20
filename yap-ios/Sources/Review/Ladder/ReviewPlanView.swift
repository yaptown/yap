import SwiftUI

/// Both initial lockup and subsequent release offers commit the shared Rust plan.
struct ReviewPlanView: View {
    let cards: [CardSummary]
    let onAccept: () -> Void
    private func group(_ card: CardSummary) -> String {
        switch card.card_indicator {
        case .WrittenGram: "Reading"
        case .ListeningGram: "Listening"
        case .LetterPronunciation: "Pronunciation"
        }
    }
    var body: some View {
        StudyCard {
            Text("Today's review plan").font(.title2.bold())
            ForEach(["Reading", "Listening", "Pronunciation"], id: \.self) { label in
                let entries = cards.filter { group($0) == label }
                if !entries.isEmpty {
                    Text("\(entries.count) \(label.lowercased()) cards").font(.headline)
                    Text(entries.map(\.card_text).joined(separator: " · ")).textSelection(.enabled)
                }
            }
            Button("Let's go!", action: onAccept).buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in guard DebugHarness.shared.activeTab == .learn else { return }; if DebugHarness.shared.command == "next" { onAccept() } }
        #endif
    }
}
