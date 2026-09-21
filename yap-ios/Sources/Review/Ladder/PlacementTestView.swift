import SwiftUI

struct PlacementTestView: View {
    let actions: ReviewActions
    @State private var placement: PlacementSession?
    var body: some View {
        StudyCard {
            if let placement {
                let info = get_placement_session_info(session: placement)
                ProgressView(value: info.progress_percent, total: 100)
                if info.finished {
                    Text(info.too_advanced ? "You might be too advanced" : "Ready to start!").font(.title2.bold())
                    Text(info.too_advanced
                         ? "Yap.Town is designed for intermediate learners. We'll still try our best to find words you don't know!"
                         : "We've analyzed your knowledge level and will tailor your learning experience.")
                    Button(info.too_advanced ? "Continue anyway" : "Begin learning", action: next)
                        .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
                } else {
                    Text("Placement test").font(.title2.bold())
                    Text("Round \(placement.round) of \(info.total_rounds)").foregroundStyle(.secondary)
                    Text("Tap the words you know, then press Next.")
                    LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                        ForEach(placement.words, id: \.word) { word in
                            let selected = placement.selected_words.contains(word.word)
                            Button { toggle(word.word) } label: {
                                Text(selected ? word.definition : word.word)
                                    .frame(maxWidth: .infinity, minHeight: 52).padding(8)
                                    .background(selected ? Color.yapAccent.opacity(0.15) : Color(uiColor: .tertiarySystemGroupedBackground), in: RoundedRectangle(cornerRadius: 12))
                            }.buttonStyle(.plain).accessibilityLabel(word.word).accessibilityValue(selected ? "Known: \(word.definition)" : "Unknown")
                        }
                    }
                    Button("Next", action: next).buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
                    if info.can_restart { Button("Start over") { self.placement = actions.startPlacement() } }
                }
            }
        }
        .onAppear { if placement == nil { placement = actions.placement ?? actions.startPlacement() } }
        .onChange(of: placement) { _, value in if let value { actions.savePlacement(value) } }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeTab == .learn else { return }
            let command = DebugHarness.shared.command
            if command == "next" { next() }
            if command == "back" { placement = actions.startPlacement() }
            if command.hasPrefix("choose "), let i = Int(command.dropFirst(7)), let placement = placement, placement.words.indices.contains(i) { toggle(placement.words[i].word) }
        }
        #endif
    }
    private func toggle(_ word: String) {
        guard let placement else { return }
        self.placement = toggle_placement_word(session: placement, word: word)
    }
    private func next() {
        guard let placement = placement else { return }
        if get_placement_session_info(session: placement).finished {
            actions.completePlacementTest(placement)
        } else { self.placement = actions.advancePlacement(placement) }
    }
}
