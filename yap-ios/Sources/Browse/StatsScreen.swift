import SwiftUI

struct StatsScreen: View {
    let review: ReviewModel
    let open: (CourseRoute) -> Void
    var view: StatsScreenView? = nil
    var body: some View {
        TimelineView(.every(10)) { _ in
            let now = ReviewModel.now
            let view = self.view ?? review.deck.stats_screen_view(banned: review.banned, timestamp_ms: now)
            ScrollViewReader { proxy in
                ScrollView {
                    VStack(alignment: .leading, spacing: 24) {
                        Grid(horizontalSpacing: 16, verticalSpacing: 16) {
                            ForEach(Array(stride(from: 0, to: view.tiles.count, by: 2)), id: \.self) { index in
                                GridRow {
                                    ForEach(Array(view.tiles[index..<min(index + 2, view.tiles.count)].enumerated()), id: \.offset) { _, tile in
                                        StatTile(tile: tile)
                                    }
                                }
                            }
                        }.fixedSize(horizontal: false, vertical: true).id("stats")
                        ListLink(title: view.due.title, label: view.due.label) { open(.due) }.id("due")
                        Grid(horizontalSpacing: 16) {
                            GridRow {
                                ListLink(title: view.your_cards.title, label: view.your_cards.label) { open(.yourCards) }
                                ListLink(title: view.assumed_known.title, label: view.assumed_known.label) { open(.assumedKnown) }
                            }
                        }.fixedSize(horizontal: false, vertical: true).id("words")
                        VStack(alignment: .leading, spacing: 16) {
                            Text(view.leeches_label).font(.title2.bold()).foregroundStyle(Color.yapText)
                            Text(L("leeches-leeches-are-cards-you-re-really-struggling-with-the"))
                                .font(.subheadline).foregroundStyle(Color.yapMuted)
                            if !view.leeches.isEmpty { CardSummaryList(cards: view.leeches, timestampMs: now) }
                        }.id("leeches")
                    }.padding(20).frame(maxWidth: .readableColumn).frame(maxWidth: .infinity)
                }.background(.clear).navigationTitle(view.title)
                #if DEBUG
                .onAppear { DebugHarness.shared.activeScreen = .stats }
                .onChange(of: DebugHarness.shared.commandID) { _, _ in
                    guard DebugHarness.shared.activeScreen == .stats else { return }
                    let command = DebugHarness.shared.command
                    if command.hasPrefix("dump-fixture ") { DebugHarness.dumpFixture(.Stats(view), name: String(command.dropFirst(13))) }
                    if command.hasPrefix("stats-") { proxy.scrollTo(String(command.dropFirst(6)), anchor: .top) }
                    if command == "status" { DebugHarness.log("stats xp=\(view.xp) reviews=\(view.total_reviews)") }
                }
                #endif
            }
        }
    }
}

private struct ListLink: View {
    let title: String
    let label: String
    let action: () -> Void
    var body: some View {
        // Like StatTile, the surface fills the row so paired links match heights.
        Button(action: action) {
            VStack(alignment: .leading, spacing: 12) {
                Text("\(title) →").font(.headline).foregroundStyle(Color.yapText)
                Text(label).font(.subheadline).foregroundStyle(Color.yapMuted)
            }.frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                .padding(16).cardSurface()
        }.buttonStyle(.plain)
    }
}

private struct StatTile: View {
    let tile: StatTileView
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(tile.eyebrow).font(.subheadline).foregroundStyle(Color.yapMuted)
            Text(tile.value).font(.title2.bold())
            if let caption = tile.caption { Text(caption).font(.subheadline).foregroundStyle(Color.yapMuted) }
        }.frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            .padding(16).cardSurface().foregroundStyle(Color.yapText)
    }
}
