import SwiftUI
import Charts

struct YourCardsScreen: View {
    let review: ReviewModel
    var view: YourCardsScreenView? = nil
    @State private var page: UInt32 = 0
    var body: some View {
        TimelineView(.every(10)) { _ in
            let now = ReviewModel.now
            let view = self.view ?? review.deck.your_cards_view(page: page)
            WordList(title: view.summary.title, label: view.summary.label, pagination: view.pagination, page: $page) {
                if !view.cards.isEmpty { CardSummaryList(cards: view.cards, timestampMs: now) }
            }
        }
        #if DEBUG
        .onAppear { DebugHarness.shared.activeScreen = .yourCards }
        #endif
    }
}

struct AssumedKnownScreen: View {
    @Environment(AudioPlayer.self) private var audio
    let review: ReviewModel
    let session: YapSession
    var view: AssumedKnownScreenView? = nil
    @State private var page: UInt32 = 0
    var body: some View {
        let view = self.view ?? review.deck.assumed_known_view(page: page)
        WordList(title: view.summary.title, label: view.summary.label, pagination: view.pagination, page: $page) {
            Text(view.note).font(.subheadline).foregroundStyle(Color.yapMuted)
            StudyCard {
                Text(view.frequency_knowledge_chart_title).font(.title2.bold()).foregroundStyle(Color.yapText)
                KnowledgeChart(points: view.frequency_knowledge_chart_data)
            }
            if !view.words.isEmpty {
                StudyCard {
                    LazyVStack(alignment: .leading, spacing: 14) {
                        ForEach(Array(view.words.enumerated()), id: \.element.frequency_index) { index, word in
                            if index > 0 { Divider() }
                            AssumedKnownRow(word: word, reviewCount: review.deck.get_total_reviews()) { add(word) }
                        }
                    }
                }
            }
        }
        .onDisappear { audio.stop() }
        #if DEBUG
        .onAppear { DebugHarness.shared.activeScreen = .assumedKnown }
        #endif
    }
    /// The word leaves the list once the next deck snapshot lands.
    private func add(_ word: AssumedKnownWord) {
        guard let event = review.deck.add_gram_by_frequency_index(frequency_index: word.frequency_index) else { return }
        session.addDeckEvent(event)
    }
}

private struct AssumedKnownRow: View {
    let word: AssumedKnownWord
    let reviewCount: UInt64
    let add: () -> Void
    var body: some View {
        HStack(spacing: 12) {
            AudioButton(request: word.audio_request, reviewCount: reviewCount)
            VStack(alignment: .leading, spacing: 4) {
                HStack(alignment: .lastTextBaseline, spacing: 0) {
                    if let prefix = word.prefix { Text(prefix.prefix + prefix.separator).foregroundStyle(Color.yapMuted) }
                    RubyText(text: word.display_text, segments: word.ruby, rubyFont: .caption2.weight(.medium))
                }.fontWeight(.semibold).foregroundStyle(Color.yapText)
                if !word.gloss.isEmpty { Text(word.gloss).font(.subheadline).foregroundStyle(Color.yapMuted) }
            }.frame(maxWidth: .infinity, alignment: .leading)
            Text(word.frequency_rank_label).font(.caption).monospacedDigit().foregroundStyle(Color.yapMuted)
                .accessibilityLabel(word.frequency_rank_accessibility_label)
            Button(word.add_label, systemImage: "plus.circle", action: add)
                .labelStyle(.iconOnly).font(.title3).foregroundStyle(Color.yapText)
                .accessibilityLabel("\(word.add_label): \(word.display_text)")
        }
    }
}

private struct WordList<Content: View>: View {
    let title: String
    let label: String
    let pagination: PaginationView?
    @Binding var page: UInt32
    @ViewBuilder let content: Content
    var body: some View {
        ScrollViewReader { proxy in
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    Text(label).foregroundStyle(Color.yapMuted).id("top")
                    content
                    if let pagination { Pagination(view: pagination, page: $page) }
                }.padding(20).frame(maxWidth: .readableColumn).frame(maxWidth: .infinity)
            }.background(.clear).navigationTitle(title)
                .onChange(of: page) { _, _ in proxy.scrollTo("top", anchor: .top) }
        }
    }
}

/// The web's Pagination: previous, a menu of word ranges to jump to, next.
private struct Pagination: View {
    let view: PaginationView
    @Binding var page: UInt32
    var body: some View {
        HStack(spacing: 8) {
            step(view.previous, label: view.previous_label, systemImage: "chevron.left")
            Menu {
                Picker(view.status, selection: $page) {
                    ForEach(view.options, id: \.page) { option in
                        Text(option.label).monospacedDigit().tag(option.page)
                    }
                }
            } label: {
                Text(view.status).monospacedDigit().frame(minWidth: 140)
            }.buttonStyle(.bordered)
            step(view.next, label: view.next_label, systemImage: "chevron.right")
        }.frame(maxWidth: .infinity).foregroundStyle(Color.yapText)
    }
    private func step(_ target: UInt32?, label: String, systemImage: String) -> some View {
        Button(label, systemImage: systemImage) { if let target { page = target } }
            .labelStyle(.iconOnly).buttonStyle(.bordered).disabled(target == nil)
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
        if points.isEmpty { Text(L("frequency-knowledge-chart-no-frequency-data-available")) }
        else {
            Chart {
                ForEach(points, id: \.frequency) { point in
                    LineMark(x: .value(L("stats-screen-frequency-rank"), point.frequency), y: .value(L("stats-screen-knowledge"), point.predicted_knowledge * 100))
                        .foregroundStyle(Tokens.palette.chart_1.color).lineStyle(StrokeStyle(lineWidth: 2))
                    PointMark(x: .value(L("stats-screen-frequency-rank"), point.frequency), y: .value(L("stats-screen-knowledge"), point.predicted_knowledge * 100))
                        .symbol {
                            Circle().strokeBorder(Tokens.palette.chart_1.color, lineWidth: 2)
                                .background(Circle().fill(.white))
                                .frame(width: selected?.frequency == point.frequency ? 12 : 8,
                                       height: selected?.frequency == point.frequency ? 12 : 8)
                        }
                }
                if let selected {
                    RuleMark(x: .value(L("stats-screen-frequency-rank"), selected.frequency)).foregroundStyle(.secondary.opacity(0.4))
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
                .chartYAxisLabel(L("stats-screen-knowledge"))
                .chartXSelection(value: $frequency).frame(height: 300)
            if let selected {
                Text(L("stats-frequency-knowledge", ["frequency": frequency_rank_label(rank: selected.frequency), "knowledge": String(format: "%.1f", selected.predicted_knowledge * 100)])).font(.caption)
                Text(L("stats-examples-with-words", ["count": selected.word_count, "examples": selected.example_words])).font(.caption).foregroundStyle(Color.yapMuted)
            }
        }
    }
}
