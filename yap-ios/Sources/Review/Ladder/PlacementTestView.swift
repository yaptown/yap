import SwiftUI

struct PlacementTestView: View {
    @Environment(\.reviewActions!) private var actions
    @Environment(\.reviewHost!) private var host
    let placement: PlacementSession
    var body: some View {
        StudyCard(animated: true) {
            let info = get_placement_session_info(session: placement)
            ProgressView(value: info.progress_percent, total: 100)
            if info.finished {
                Text(info.too_advanced ? L("home-you-might-be-too-advanced") : L("placement-test-view-ready-to-start")).font(.title2.bold())
                Text(info.too_advanced
                     ? L("placement-test-view-yap-town-is-designed-for-intermediate-learners-we-ll")
                     : L("placement-test-view-we-ve-analyzed-your-knowledge-level-and-will-tailor"))
                Button(action: next) { Text(info.too_advanced ? L("placement-test-view-continue-anyway") : L("placement-test-view-begin-learning")).frame(maxWidth: .infinity) }
                    .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
            } else {
                Text(L("home-placement-test-2")).font(.title2.bold())
                Text(L("placement-round", ["round": placement.round, "total": info.total_rounds])).foregroundStyle(Color.yapMuted)
                Text(L("placement-test-view-tap-the-words-you-know-then-press-next"))
                LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                    ForEach(placement.words, id: \.word) { word in
                        let selected = placement.selected_words.contains(word.word)
                        Button { toggle(word.word) } label: {
                            Text(selected ? word.definition : word.word)
                                .frame(maxWidth: .infinity, minHeight: 52).padding(8)
                                .insetSurface(fill: selected ? Color.yapAccent.opacity(0.15) : Color(uiColor: .systemBackground).opacity(0.35))
                        }.buttonStyle(.plain).accessibilityLabel(word.word).accessibilityValue(selected ? L("placement-known", ["definition": word.definition]) : L("placement-test-view-unknown"))
                    }
                }
                Button(action: next) { Text(L("common-next")).frame(maxWidth: .infinity) }.buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
                if info.can_restart { Button(L("placement-test-view-start-over")) { actions.setPlacement(host.deck.start_placement_session()) }.frame(maxWidth: .infinity) }
            }
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeScreen == .review else { return }
            let command = DebugHarness.shared.command
            if command == "next" { next() }
            if command == "back" { actions.setPlacement(host.deck.start_placement_session()) }
            if command.hasPrefix("choose "), let i = Int(command.dropFirst(7)), placement.words.indices.contains(i) { toggle(placement.words[i].word) }
        }
        #endif
    }
    private func toggle(_ word: String) {
        actions.setPlacement(toggle_placement_word(session: placement, word: word))
    }
    private func next() {
        if get_placement_session_info(session: placement).finished {
            actions.completePlacementTest(placement)
        } else { actions.setPlacement(host.deck.advance_placement_session(session: placement)) }
    }
}
