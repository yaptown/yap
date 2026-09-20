import SwiftUI
import Charts

struct StatsScreen: View {
    let deck: Deck
    let session: YapSession
    @State private var lists: SentenceListModel?
    @State private var points: [FrequencyKnowledgePoint] = []
    @State private var allMovies = false
    @State private var cardLimit = 10
    @State private var revealed: Set<Int> = []
    var body: some View {
        ScrollViewReader { proxy in
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                today.id("today")
                StudyCard {
                    metric("XP", "\(deck.get_xp())")
                    Text("You get more XP for words you didn't remember.").font(.caption).foregroundStyle(.secondary)
                    let cards = deck.get_all_cards_summary()
                    metric("Total cards", "\(cards.count)")
                    Text("\(cards.filter { $0.due_timestamp_ms <= ReviewModel.now }.count) ready now").font(.caption)
                    metric("Words known", String(format: "%.1f%%", deck.get_percent_of_words_known() * 100))
                    metric("Cards added", "\(deck.num_cards_added())")
                    metric("Daily streak", "\(deck.get_daily_streak()) days")
                    metric("Total reviews", "\(deck.get_total_reviews())")
                    let tier = deck.get_current_tier()
                    Text("\(tier.name) · Level \(tier.level) of \(tier.total_levels)").font(.headline)
                    ProgressView(value: tier.percent_known, total: 100)
                }
                StudyCard {
                    Text("Cards").font(.title2.bold())
                    TimelineView(.periodic(from: .now, by: 10)) { context in
                        let now = context.date.timeIntervalSince1970 * 1000
                        let cards = deck.get_all_cards_summary().sorted { ($0.due_timestamp_ms <= now ? 0 : 1) < ($1.due_timestamp_ms <= now ? 0 : 1) }
                        VStack(alignment: .leading, spacing: 14) {
                            ForEach(Array(cards.prefix(cardLimit).enumerated()), id: \.offset) { index, card in
                                VStack(alignment: .leading, spacing: 4) {
                                    if case .ListeningGram = card.card_indicator, !revealed.contains(index) {
                                        Button("Listening word · Tap to reveal") { revealed.insert(index) }
                                    } else { Text(card.card_text).fontWeight(.semibold) }
                                    if let subtitle = card.card_subtitle { Text(subtitle).font(.caption) }
                                    HStack {
                                        Text(card.state)
                                        Spacer()
                                        if card.due_timestamp_ms <= now { Text("Ready now") }
                                        else { Text(Date(timeIntervalSince1970: card.due_timestamp_ms / 1000), style: .relative) }
                                    }.font(.caption).foregroundStyle(.secondary)
                                }
                            }
                            if cards.count > cardLimit { Button("Show more cards") { cardLimit += 100 } }
                        }
                    }
                }
                StudyCard { KnowledgeChart(points: points) }.id("chart")
                if let lists {
                    StudyCard {
                        Text("Movies").font(.title2.bold()).id("movies")
                        Text("You can usually watch comfortably once you know 95% of the words.").font(.subheadline).foregroundStyle(.secondary)
                        let language = get_language_metadata(language: deck.get_target_language()).iso6391
                        let sorted = lists.movies.sorted { (lists.metadata[$0.id]?.original_language == language ? 0 : 1) < (lists.metadata[$1.id]?.original_language == language ? 0 : 1) }
                        ForEach(allMovies ? sorted : Array(sorted.prefix(8)), id: \.id) { movie in
                            HStack(spacing: 14) {
                                MoviePoster(deck: deck, id: movie.id, title: lists.metadata[movie.id]?.title ?? movie.id)
                                VStack(alignment: .leading, spacing: 8) {
                                    Text(lists.metadata[movie.id]?.title ?? movie.id).font(.headline)
                                    Text("\(Int(movie.percent_known))% known").font(.caption)
                                    if let count = movie.cards_to_next_milestone { Text("\(count) \(count == 1 ? "card" : "cards") to \(Int(ceil(movie.percent_known / 5) * 5))%").font(.caption).foregroundStyle(.secondary) }
                                }
                            }
                        }
                        if !allMovies && sorted.count > 8 { Button("Show all \(sorted.count) movies") { allMovies = true } }
                    }
                    if !lists.lessons.isEmpty {
                        StudyCard {
                            Text("Pimsleur").font(.title2.bold())
                            ForEach(Array(Set(lists.lessons.map(\.level))).sorted(), id: \.self) { level in
                                DisclosureGroup("Level \(level)") {
                                    ForEach(lists.lessons.filter { $0.level == level }, id: \.lesson) { lesson in
                                        LabeledContent("Lesson \(lesson.lesson)", value: lesson.all_available_learned ? "Done!" : "\(Int(lesson.percent_known))% known")
                                    }
                                }
                            }
                        }
                    }
                }
                StudyCard {
                    Text("Tools").font(.title2.bold())
                    Text("Browse words in Dictionary and choose your focus in Lists.")
                    Link("Leeches on Yap.town", destination: URL(string: "https://yap.town/leeches")!)
                    Link("Simulate on Yap.town", destination: URL(string: "https://yap.town/simulate")!)
                }
            }.padding(20).frame(maxWidth: 600)
        }.background(Color(uiColor: .systemGroupedBackground)).navigationTitle("Stats")
            .task(id: ObjectIdentifier(deck)) { lists = SentenceListModel(deck: deck, session: session); points = deck.get_frequency_knowledge_chart_data() }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            if DebugHarness.shared.command.hasPrefix("stats-") { proxy.scrollTo(String(DebugHarness.shared.command.dropFirst(6)), anchor: .top) }
            if DebugHarness.shared.command == "status" { DebugHarness.log("stats xp=\(deck.get_xp()) reviews=\(deck.get_total_reviews()) cards=\(deck.num_cards_added()) chart=\(points.count)") }
        }
        #endif
        }
    }
    private var today: some View {
        StudyCard {
            let summary = deck.get_today_summary()
            Text("Today").font(.title2.bold())
            Text("\(summary.time_spent_seconds / 60)m \(summary.time_spent_seconds % 60)s / \(deck.get_daily_review_target() / 60)m").font(.title.bold())
            Text("\(deck.get_today_reviews()) reviews today").foregroundStyle(.secondary)
            HStack(spacing: 8) {
                ForEach(deck.get_current_week_progress(), id: \.weekday) { day in
                    VStack(spacing: 8) {
                        Text(["M", "T", "W", "T", "F", "S", "S"][Int(day.weekday) % 7]).font(.caption)
                        Image(systemName: day.met_goal ? "checkmark.circle.fill" : day.is_future ? "circle.dotted" : "circle")
                            .foregroundStyle(day.is_today ? Color.yapAccent : .secondary)
                        Text("\(day.seconds / 60)m").font(.caption2)
                    }.frame(maxWidth: .infinity)
                        .accessibilityElement(children: .ignore)
                        .accessibilityLabel("\(day.is_today ? "Today, " : "")\(day.reviews) reviews, \(day.seconds / 60) minutes\(day.met_goal ? ", goal reached" : "")")
                }
            }
            DailyGoalEditor(deck: deck, session: session)
            LabeledContent("New today", value: "\(summary.new_cards.count)")
            LabeledContent("Learned", value: "\(summary.learned_cards.count)")
            LabeledContent("Back on track", value: "\(summary.locked_in_cards.count)")
            if let recall = summary.recall_percent { LabeledContent("Recall", value: "\(recall)%") }
            if !summary.reviewed_words.isEmpty { DisclosureGroup("Reviewed words") { Text(summary.reviewed_words.joined(separator: " · ")) } }
        }
    }
    private func metric(_ title: String, _ value: String) -> some View { LabeledContent { Text(value).font(.title3.bold()) } label: { Text(title) } }
}

private struct KnowledgeChart: View {
    @Environment(\.colorScheme) private var scheme
    let points: [FrequencyKnowledgePoint]
    @State private var frequency: Double?
    private var selected: FrequencyKnowledgePoint? { frequency.flatMap { x in points.min { abs($0.frequency - x) < abs($1.frequency - x) } } }
    private var ink: Color { scheme == .dark ? Color(red: 192 / 255, green: 112 / 255, blue: 186 / 255) : .yapAccent }
    var body: some View {
        Text("Pre-existing knowledge by word frequency").font(.title2.bold())
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
