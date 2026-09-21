import SwiftUI

struct SentenceListsScreen: View {
    let deck: Deck
    let session: YapSession
    var onSelected: () -> Void
    @State private var lists: SentenceListModel?
    @State private var showAllMovies = false
    @AppStorage("yap-pimsleur-acknowledged") private var acknowledged = false
    var body: some View {
        List {
            if let lists {
                if let selection = deck.get_sentence_list() {
                    Section("Current selection") { row(selection, lists: lists) }
                }
                Section {
                    Text("Pick a sentence list to focus your learning on. Your cards will be tailored to help you reach it.")
                    row(nil, lists: lists)
                    Text("Level \(lists.tier.level) of \(lists.tier.total_levels). Complete this level to understand \(lists.tier.percent_of_usage, specifier: "%.1f")% of everyday \(get_language_metadata(language: deck.get_target_language()).common_name).")
                        .font(.caption).foregroundStyle(.secondary)
                } header: { Text("Essential") }
                Section("Movies") {
                    Text("You can usually watch comfortably once you know 95% of the words.").font(.subheadline).foregroundStyle(.secondary)
                    if let best = deck.get_best_movie_sentence_list() {
                        Button("Suggested: \(lists.title(best))") { choose(best, lists: lists) }
                    }
                    let native = get_language_metadata(language: deck.get_target_language()).iso6391
                    let movies = lists.movies.filter { lists.metadata[$0.id]?.original_language == native }
                    ForEach(Array((showAllMovies ? movies : Array(movies.prefix(5)))), id: \.id) { movie in row(.Movie(id: movie.id), lists: lists) }
                    if !showAllMovies && movies.count > 5 { Button("Show \(movies.count - 5) more movies") { showAllMovies = true } }
                }
                if !lists.lessons.isEmpty {
                    Section("Pimsleur") {
                        if !acknowledged {
                            Text("Yap has word lists for Pimsleur, but is not affiliated with Pimsleur in any way.")
                            Button("I understand") { acknowledged = true }
                        } else {
                            if let best = deck.get_best_pimsleur_sentence_list() { Button("Suggested: \(lists.title(best))") { choose(best, lists: lists) } }
                            ForEach(Array(Set(lists.lessons.map(\.level))).sorted(), id: \.self) { level in
                                DisclosureGroup("Level \(level)") {
                                    ForEach(lists.lessons.filter { $0.level == level }, id: \.lesson) { lesson in row(.PimsleurLesson(level: level, lesson: lesson.lesson), lists: lists) }
                                }
                            }
                        }
                    }
                }
            } else { ProgressView() }
        }.navigationTitle("Sentence lists")
            .task(id: ObjectIdentifier(deck)) { lists = SentenceListModel(deck: deck, session: session) }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard let lists else { return }
            let command = DebugHarness.shared.command
            if command == "acknowledge-pimsleur" { acknowledged = true }
            if command == "select-list essential" { choose(nil, lists: lists) }
            if command == "select-list movie", let best = deck.get_best_movie_sentence_list() { choose(best, lists: lists) }
            if command == "select-list pimsleur", acknowledged, let best = deck.get_best_pimsleur_sentence_list() { choose(best, lists: lists) }
            if command.hasPrefix("select-list movie ") {
                let id = String(command.dropFirst(18))
                if lists.movies.contains(where: { $0.id == id }) { choose(.Movie(id: id), lists: lists) }
            }
            if command.hasPrefix("select-list pimsleur "), acknowledged {
                let numbers = command.dropFirst(21).split(separator: " ").compactMap { UInt32($0) }
                if numbers.count == 2, lists.lessons.contains(where: { $0.level == numbers[0] && $0.lesson == numbers[1] }) { choose(.PimsleurLesson(level: numbers[0], lesson: numbers[1]), lists: lists) }
            }
        }
        #endif
    }
    private func row(_ selection: SentenceListSelection?, lists: SentenceListModel) -> some View {
        Button { choose(selection, lists: lists) } label: {
            HStack(spacing: 14) {
                if case let .Movie(id) = selection { MoviePoster(poster: { deck.get_movie_poster(movie_id: $0) }, id: id, title: lists.title(selection)) }
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        Text(lists.title(selection)).foregroundStyle(Color.yapText)
                        if selection == deck.get_sentence_list() { Image(systemName: "checkmark.circle.fill").accessibilityLabel("Selected") }
                    }
                    if case let .Movie(id) = selection, let year = lists.metadata[id]?.year {
                        Text(String(year)).font(.caption).foregroundStyle(Color(uiColor: .secondaryLabel))
                    }
                    SentenceListProgressView(progress: deck.get_sentence_list_progress(selection: selection, essential_percent_known: lists.tier.percent_known))
                }
            }.padding(.vertical, 6)
        }
    }
    private func choose(_ selection: SentenceListSelection?, lists: SentenceListModel) { lists.change(selection); onSelected() }
}
