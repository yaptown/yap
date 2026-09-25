import SwiftUI
import Charts

struct StatsScreen: View {
    let review: ReviewModel
    let showDue: () -> Void
    var view: StatsScreenView? = nil
    var body: some View {
        TimelineView(.periodic(from: .now, by: 10)) { _ in
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
                        Button(action: showDue) {
                            StudyCard {
                                Text("\(view.due.title) →").font(.headline).foregroundStyle(Color.yapText)
                                Text(view.due.label).font(.subheadline).foregroundStyle(.secondary)
                            }
                        }.buttonStyle(.plain).id("due")
                        StudyCard {
                            Text(view.frequency_knowledge_chart_title).font(.title2.bold()).foregroundStyle(Color.yapText)
                            KnowledgeChart(points: view.frequency_knowledge_chart_data)
                        }.id("chart")
                        VStack(alignment: .leading, spacing: 16) {
                            Text(view.leeches_label).font(.title2.bold()).foregroundStyle(Color.yapText)
                            Text("Leeches are cards you're really struggling with. The hardest few cards can take disproportionate time, so it's more efficient to set them aside for a while.")
                                .font(.subheadline).foregroundStyle(.secondary)
                            if !view.leeches.isEmpty { CardSummaryList(cards: view.leeches, timestampMs: now) }
                        }.id("leeches")
                    }.padding(20).frame(maxWidth: 600).frame(maxWidth: .infinity)
                }.background(.clear).navigationTitle(view.title)
                #if DEBUG
                .onAppear { DebugHarness.shared.activeScreen = .stats }
                .onChange(of: DebugHarness.shared.commandID) { _, _ in
                    guard DebugHarness.shared.activeScreen == .stats else { return }
                    let command = DebugHarness.shared.command
                    if command.hasPrefix("dump-fixture ") { DebugHarness.dumpFixture(.Stats(view), name: String(command.dropFirst(13))) }
                    if command.hasPrefix("stats-") { proxy.scrollTo(String(command.dropFirst(6)), anchor: .top) }
                    if command == "status" { DebugHarness.log("stats xp=\(view.xp) reviews=\(view.total_reviews) chart=\(view.frequency_knowledge_chart_data.count)") }
                }
                #endif
            }
        }
    }
}

private struct StatTile: View {
    let tile: StatTileView
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(tile.eyebrow).font(.subheadline).foregroundStyle(.secondary)
            Text(tile.value).font(.title2.bold())
            if let caption = tile.caption { Text(caption).font(.subheadline).foregroundStyle(.secondary) }
        }.frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            .padding(16).cardSurface().foregroundStyle(Color.yapText)
    }
}

private struct KnowledgeChart: View {
    let points: [FrequencyKnowledgePoint]
    @State private var frequency: Double?
    private let ticks = frequency_knowledge_ticks()
    // Snap in log space, just as the plotted distances are logarithmic.
    private var selected: FrequencyKnowledgePoint? {
        guard let frequency, frequency > 0 else { return nil }
        return points.min { abs(log($0.frequency / frequency)) < abs(log($1.frequency / frequency)) }
    }
    var body: some View {
        if points.isEmpty { Text("No frequency data available") }
        else {
            Chart {
                ForEach(points, id: \.frequency) { point in
                    LineMark(x: .value("Frequency rank", point.frequency), y: .value("Knowledge (%)", point.predicted_knowledge * 100))
                        .foregroundStyle(Tokens.palette.chart_1.color).lineStyle(StrokeStyle(lineWidth: 2))
                    PointMark(x: .value("Frequency rank", point.frequency), y: .value("Knowledge (%)", point.predicted_knowledge * 100))
                        .symbol {
                            Circle().strokeBorder(Tokens.palette.chart_1.color, lineWidth: 2)
                                .background(Circle().fill(.white))
                                .frame(width: selected?.frequency == point.frequency ? 12 : 8,
                                       height: selected?.frequency == point.frequency ? 12 : 8)
                        }
                }
                if let selected {
                    RuleMark(x: .value("Frequency rank", selected.frequency)).foregroundStyle(.secondary.opacity(0.4))
                }
            }
                .chartXScale(domain: 1.0...10000.0, type: .log)
                .chartYScale(domain: 0...100)
                .chartYAxis { AxisMarks(position: .leading, values: [0, 25, 50, 75, 100]) { _ in AxisGridLine(stroke: StrokeStyle(lineWidth: 0.5)).foregroundStyle(.secondary.opacity(0.25)); AxisValueLabel() } }
                .chartXAxis {
                    AxisMarks(values: ticks.map(\.value)) { value in
                        AxisGridLine(stroke: StrokeStyle(lineWidth: 0.5)).foregroundStyle(.secondary.opacity(0.25))
                        AxisValueLabel(anchor: .topTrailing, collisionResolution: .greedy(priority: value.index == ticks.count - 1 ? 1 : 0)) {
                            if let rank = value.as(Double.self), let tick = ticks.first(where: { $0.value == rank }) { Text(tick.label).fixedSize() }
                        }
                    }
                }
                .chartYAxisLabel("Knowledge (%)")
                .chartXSelection(value: $frequency).frame(height: 300)
            if let selected {
                Text("Frequency: \(frequency_rank_label(rank: selected.frequency)) · Knowledge: \(selected.predicted_knowledge * 100, specifier: "%.1f")%").font(.caption)
                Text("Examples (\(selected.word_count) words): \(selected.example_words)").font(.caption).foregroundStyle(.secondary)
            }
        }
    }
}
