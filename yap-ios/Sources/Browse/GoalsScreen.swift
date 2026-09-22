import SwiftUI

struct GoalsScreen: View {
    @Environment(\.reviewActions!) private var actions
    let review: ReviewModel
    var view: GoalsScreenView? = nil
    private var deck: Deck { review.deck }
    @State private var lists: SentenceListModel?
    @State private var showAllMovies = false
    @AppStorage("yap-pimsleur-acknowledged") private var acknowledged = false
    var body: some View {
        let view = self.view ?? deck.goals_screen_view(banned: review.banned, sentence_list: deck.get_sentence_list())
        ScrollViewReader { proxy in
            ScrollView {
                VStack(alignment: .leading, spacing: 24) {
                    StudyCard { GoalProgress(goal: view.goal) }.id("goal")
                    StudyCard {
                        Text(view.daily_goal_title).font(.headline)
                        Text(view.daily_goal_label).font(.subheadline).foregroundStyle(.secondary)
                        DailyGoalEditor(target: view.daily_goal, options: view.daily_goal_options, addEvent: actions.addEvent)
                            .id(view.daily_goal)
                    }.id("daily-goal")
                    Text(view.curriculum.title).font(.title2.bold())
                    curriculum(view)
                }.padding(20).frame(maxWidth: 600).frame(maxWidth: .infinity)
            }.background(Color(uiColor: .systemGroupedBackground)).navigationTitle(view.title)
                .task(id: ObjectIdentifier(deck)) {
                    // Movie/Pimsleur lists are not captured in GoalsScreenView.
                    guard self.view == nil else { return }
                    lists = SentenceListModel(deck: deck, session: review.session)
                }
            #if DEBUG
            .onAppear { DebugHarness.shared.activeScreen = .goals }
            .onChange(of: DebugHarness.shared.commandID) { _, _ in
                guard DebugHarness.shared.activeScreen == .goals else { return }
                let command = DebugHarness.shared.command
                if command.hasPrefix("dump-fixture ") { DebugHarness.dumpFixture(.Goals(view), name: String(command.dropFirst(13))) }
                if command.hasPrefix("stats-") || command.hasPrefix("goals-") { proxy.scrollTo(String(command.dropFirst(6)), anchor: .top) }
                debugCommand()
            }
            #endif
        }
    }
    private func curriculum(_ view: GoalsScreenView) -> some View {
        let curriculum = view.curriculum
        let selectedIndex = Int(curriculum.navigation.selected_index)
        return StudyCard {
            Picker(curriculum.title, selection: Binding(
                get: { selectedIndex },
                set: { actions.setSentenceList(curriculum.sentence_list_options[$0].event) }
            )) {
                ForEach(Array(curriculum.sentence_list_options.enumerated()), id: \.offset) { index, option in
                    Text(option.label).tag(index)
                }
            }.pickerStyle(.segmented)
            switch curriculum.sentence_list_options[selectedIndex].category {
            case .Essential:
                Text(curriculum.sentence_list_label).font(.headline)
                SentenceListProgressView(progress: curriculum.progress)
                if let event = curriculum.next_sentence_list_event {
                    Button(nextLabel(curriculum.next_sentence_list)) { actions.setSentenceList(event) }.buttonStyle(.bordered)
                }
            case .Movie:
                if self.view == nil, let lists { movies(lists).id("movies") }
            case .Pimsleur:
                if self.view == nil, let lists { pimsleur(lists).id("pimsleur") }
            }
        }
    }
    private func movies(_ lists: SentenceListModel) -> some View {
        let native = get_language_metadata(language: deck.get_target_language()).iso6391
        let movies = lists.movies.sorted { (lists.metadata[$0.id]?.original_language == native ? 0 : 1) < (lists.metadata[$1.id]?.original_language == native ? 0 : 1) }
        return VStack(alignment: .leading, spacing: 12) {
            Text("Movies").font(.title2.bold())
            Text("You can usually watch a movie comfortably once you know 95% of the words.").font(.subheadline).foregroundStyle(.secondary)
            ForEach(showAllMovies ? movies : Array(movies.prefix(8)), id: \.id) { movie in
                row(.Movie(id: movie.id), lists: lists)
            }
            if !showAllMovies && movies.count > 8 { Button("Show all \(movies.count) movies") { showAllMovies = true } }
        }
    }
    private func pimsleur(_ lists: SentenceListModel) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Pimsleur Lessons").font(.title2.bold())
            if !acknowledged {
                Text("Yap has word lists for Pimsleur, but is not affiliated with Pimsleur in any way.")
                Button("I understand") { acknowledged = true }
            } else {
                Text("Focus on vocabulary from a specific Pimsleur lesson.").font(.subheadline).foregroundStyle(.secondary)
                ForEach(Array(Set(lists.lessons.map(\.level))).sorted(), id: \.self) { level in
                    DisclosureGroup("Level \(level)") {
                        ForEach(lists.lessons.filter { $0.level == level }, id: \.lesson) { lesson in
                            row(.PimsleurLesson(level: level, lesson: lesson.lesson), lists: lists)
                        }
                    }
                }
            }
        }
    }
    private func row(_ selection: SentenceListSelection, lists: SentenceListModel) -> some View {
        Button { lists.change(selection) } label: {
            HStack(spacing: 14) {
                if case let .Movie(id) = selection { MoviePoster(id: id, title: lists.title(selection)) }
                else { Image(systemName: "headphones") }
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        Text(rowTitle(selection, lists: lists)).foregroundStyle(Color.yapText)
                        if selection == deck.get_sentence_list() { Image(systemName: "checkmark.circle.fill").accessibilityLabel("Selected") }
                    }
                    if case let .Movie(id) = selection {
                        if let year = lists.metadata[id]?.year { Text(String(year)).font(.caption).foregroundStyle(.secondary) }
                        if let movie = lists.movies.first(where: { $0.id == id }), let count = movie.cards_to_next_milestone {
                            Text("\(count) \(count == 1 ? "card" : "cards") to \(Int(ceil(movie.percent_known / 5) * 5))%")
                                .font(.caption).foregroundStyle(.secondary)
                        }
                    }
                    SentenceListProgressView(progress: deck.get_sentence_list_progress(selection: selection, essential_percent_known: lists.tier.percent_known))
                }.frame(maxWidth: .infinity, alignment: .leading)
            }.padding(.vertical, 6)
        }.buttonStyle(.plain)
    }
    private func rowTitle(_ selection: SentenceListSelection, lists: SentenceListModel) -> String {
        if case let .PimsleurLesson(_, lesson) = selection { return "Lesson \(lesson)" }
        return lists.title(selection)
    }
    private func nextLabel(_ selection: SentenceListSelection?) -> String {
        if case .Movie = selection { return "Next movie" }
        return "Next lesson"
    }
    #if DEBUG
    private func debugCommand() {
        guard view == nil, let lists else { return }
        let command = DebugHarness.shared.command
        if command == "acknowledge-pimsleur" { acknowledged = true }
        if command == "select-list essential" { lists.change(nil) }
        if command == "select-list movie", let best = deck.get_best_movie_sentence_list() { lists.change(best) }
        if command == "select-list pimsleur", acknowledged, let best = deck.get_best_pimsleur_sentence_list() { lists.change(best) }
        if command.hasPrefix("select-list movie ") {
            let id = String(command.dropFirst(18))
            if lists.movies.contains(where: { $0.id == id }) { lists.change(.Movie(id: id)) }
        }
        if command.hasPrefix("select-list pimsleur "), acknowledged {
            let numbers = command.dropFirst(21).split(separator: " ").compactMap { UInt32($0) }
            if numbers.count == 2, lists.lessons.contains(where: { $0.level == numbers[0] && $0.lesson == numbers[1] }) { lists.change(.PimsleurLesson(level: numbers[0], lesson: numbers[1])) }
        }
    }
    #endif
}
