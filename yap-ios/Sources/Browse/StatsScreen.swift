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
                        LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 16) {
                            StudyCard { Text(view.xp_label).font(.title2.bold()) }
                            StudyCard { Text(view.total_reviews_label).font(.title2.bold()) }
                            StudyCard {
                                Text(view.streak.title).font(.subheadline).foregroundStyle(.secondary)
                                Text(view.streak.days_label).font(.title2.bold())
                                Text(view.streak.today_label).font(.subheadline).foregroundStyle(.secondary)
                            }
                            StudyCard { Text(view.percent_known_label).font(.title2.bold()) }
                        }.id("stats")
                        Button(action: showDue) {
                            StudyCard {
                                Text("\(view.due.title) →").font(.headline)
                                Text(view.due.label).font(.subheadline).foregroundStyle(.secondary)
                            }
                        }.buttonStyle(.plain).id("due")
                        StudyCard {
                            Text(view.frequency_knowledge_chart_title).font(.title2.bold())
                            KnowledgeChart(points: view.frequency_knowledge_chart_data)
                        }.id("chart")
                        VStack(alignment: .leading, spacing: 16) {
                            Text(view.leeches_label).font(.title2.bold())
                            Text("Leeches are cards you're really struggling with. The hardest few cards can take disproportionate time, so it's more efficient to set them aside for a while.")
                                .font(.subheadline).foregroundStyle(.secondary)
                            if !view.leeches.isEmpty { CardSummaryList(cards: view.leeches, timestampMs: now) }
                        }.id("leeches")
                    }.padding(20).frame(maxWidth: 600).frame(maxWidth: .infinity)
                }.background(Color(uiColor: .systemGroupedBackground)).navigationTitle(view.title)
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

private struct KnowledgeChart: View {
    @Environment(\.colorScheme) private var scheme
    let points: [FrequencyKnowledgePoint]
    @State private var frequency: Double?
    private var selected: FrequencyKnowledgePoint? { frequency.flatMap { x in points.min { abs($0.frequency - x) < abs($1.frequency - x) } } }
    private var ink: Color { scheme == .dark ? Color(red: 192 / 255, green: 112 / 255, blue: 186 / 255) : .yapAccent }
    var body: some View {
        Text("Yap uses this estimate to avoid teaching words you already know.").font(.caption).foregroundStyle(.secondary)
        if points.isEmpty { Text("No frequency data available") }
        else {
            Chart(points, id: \.frequency) { point in
                LineMark(x: .value("Frequency rank", point.frequency), y: .value("Predicted knowledge (%)", point.predicted_knowledge * 100))
                    .foregroundStyle(ink).lineStyle(StrokeStyle(lineWidth: 2))
                if let selected, selected.frequency == point.frequency {
                    RuleMark(x: .value("Frequency", point.frequency)).foregroundStyle(.secondary)
                    PointMark(x: .value("Frequency rank", point.frequency), y: .value("Predicted knowledge (%)", point.predicted_knowledge * 100)).symbolSize(64).foregroundStyle(ink)
                }
            }.chartYScale(domain: 0...100)
                .chartYAxis { AxisMarks(values: [0, 25, 50, 75, 100]) { _ in AxisGridLine(stroke: StrokeStyle(lineWidth: 0.5)); AxisValueLabel() } }
                .chartXAxis { AxisMarks { _ in AxisGridLine(stroke: StrokeStyle(lineWidth: 0.5)); AxisValueLabel() } }
                .chartXAxisLabel("Word frequency rank").chartYAxisLabel("Knowledge (%)")
                .chartXSelection(value: $frequency).frame(height: 240)
            if let selected {
                Text("Rank \(Int(selected.frequency)): \(selected.predicted_knowledge * 100, specifier: "%.1f")% predicted knowledge").font(.caption)
                Text("\(selected.example_words) (\(selected.word_count) words)").font(.caption).foregroundStyle(.secondary)
            }
            DisclosureGroup("Chart data") {
                ForEach(points, id: \.frequency) { point in
                    VStack(alignment: .leading, spacing: 4) {
                        LabeledContent("Rank \(Int(point.frequency))", value: String(format: "%.1f%%", point.predicted_knowledge * 100))
                        Text("\(point.example_words) · \(point.word_count) words").font(.caption).foregroundStyle(.secondary)
                    }
                }
            }
        }
    }
}
