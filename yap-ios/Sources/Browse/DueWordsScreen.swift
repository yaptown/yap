import SwiftUI

struct DueWordsScreen: View {
    let review: ReviewModel
    var view: DueWordsScreenView? = nil
    var body: some View {
        TimelineView(.periodic(from: .now, by: 10)) { _ in
            let now = ReviewModel.now
            let view = self.view ?? review.deck.due_words_view(banned: review.banned, timestamp_ms: now)
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    Text(view.summary_label).foregroundStyle(.secondary)
                    CardSummaryList(cards: view.cards, timestampMs: now)
                }.padding(20).frame(maxWidth: 600).frame(maxWidth: .infinity)
            }.background(.clear).navigationTitle(view.title)
            #if DEBUG
            .onChange(of: DebugHarness.shared.commandID) { _, _ in
                let command = DebugHarness.shared.command
                guard DebugHarness.shared.activeScreen == .due, command.hasPrefix("dump-fixture ") else { return }
                DebugHarness.dumpFixture(.Due(view), name: String(command.dropFirst(13)))
            }
            #endif
        }
        #if DEBUG
        .onAppear { DebugHarness.shared.activeScreen = .due }
        #endif
    }
}
