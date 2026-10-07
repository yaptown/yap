import SwiftUI

/// Every curriculum, one tab per category. Picking an item is the choice: it
/// commits the switch and closes, so there is nothing left to confirm.
struct CurriculumBrowser: View {
    @Environment(\.reviewHost!) private var host
    @Environment(\.reviewActions!) private var actions
    @Environment(\.dismiss) private var dismiss
    @State var tab: SentenceListCategory
    @State private var lists: SentenceListModel?
    @State private var showAllMovies = false
    @AppStorage("yap-pimsleur-acknowledged") private var acknowledged = false
    private var deck: Deck { host.deck }
    private var selected: SentenceListSelection? { deck.get_sentence_list() }
    private static let columns = Array(repeating: GridItem(.flexible(), spacing: 10), count: 3)

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    if let lists {
                        tabs(lists)
                        switch tab {
                        case .Essential: essential(lists)
                        case .Movie: movies(lists)
                        case .Pimsleur: pimsleur(lists)
                        }
                    } else {
                        ProgressView().frame(maxWidth: .infinity).padding(.top, 40)
                    }
                }.frame(maxWidth: .infinity, alignment: .leading).padding(20)
            }
            .navigationTitle(L("curriculum-browser-choose-what-to-study")).navigationBarTitleDisplayMode(.inline)
            .toolbar { ToolbarItem(placement: .confirmationAction) { Button(L("common-done")) { dismiss() } } }
        }
        // Built once per opening; the lists are the costly part.
        .task { lists = SentenceListModel(deck: deck) }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            let command = DebugHarness.shared.command
            if command == "acknowledge-pimsleur" { acknowledged = true }
            if command.hasPrefix("browse-tab "), let index = Int(command.dropFirst(11)), let lists,
               lists.navigation.categories.indices.contains(index) { tab = lists.navigation.categories[index] }
            if command.hasPrefix("browse-pick "), let index = Int(command.dropFirst(12)), let lists {
                switch tab {
                case .Essential: choose(nil)
                case .Movie: if sortedMovies(lists).indices.contains(index) { choose(.Movie(id: sortedMovies(lists)[index].id)) }
                case .Pimsleur: if lists.lessons.indices.contains(index) { choose(.PimsleurLesson(level: lists.lessons[index].level, lesson: lists.lessons[index].lesson)) }
                }
            }
        }
        #endif
    }

    private func choose(_ selection: SentenceListSelection?) {
        actions.commitSentenceList(deck.change_sentence_list(sentence_list: selection))
        dismiss()
    }

    /// The web's tabs: a pale track with the selected tab as a dark pill.
    private func tabs(_ lists: SentenceListModel) -> some View {
        HStack(spacing: 0) {
            ForEach(lists.navigation.categories, id: \.self) { category in
                let selected = category == tab
                Button { tab = category } label: {
                    Text(label(category)).font(.subheadline.weight(.medium)).lineLimit(1)
                        .frame(maxWidth: .infinity, minHeight: 30)
                        .foregroundStyle(selected ? Color.yapOnAccent : Color.yapText)
                        .background { if selected { RoundedRectangle(cornerRadius: 6).fill(Color.yapAccent).shadow(radius: 1, y: 1) } }
                        .contentShape(Rectangle())
                }.buttonStyle(.plain).accessibilityAddTraits(selected ? .isSelected : [])
            }
        }
        .padding(3).background(Color.yapMutedSurface, in: RoundedRectangle(cornerRadius: 8))
        .animation(.easeOut(duration: 0.15), value: tab)
    }

    private func label(_ category: SentenceListCategory) -> String {
        switch category {
        case .Essential: L("curriculum-essential")
        case .Movie: L("common-movies")
        case .Pimsleur: "Pimsleur"
        }
    }

    private func essential(_ lists: SentenceListModel) -> some View {
        row(title: lists.title(nil), selection: nil, progress: deck.get_sentence_list_progress(selection: nil, essential_percent_known: lists.tier.percent_known), icon: nil)
    }

    private func sortedMovies(_ lists: SentenceListModel) -> [MovieStats] {
        let native = get_language_metadata(language: deck.get_target_language()).iso6391
        // The current movie first, then films made in the language.
        func rank(_ movie: MovieStats) -> Int {
            if case let .Movie(id) = selected, id == movie.id { return 0 }
            return lists.metadata[movie.id]?.original_language == native ? 1 : 2
        }
        return lists.movies.sorted { rank($0) < rank($1) }
    }

    private func movies(_ lists: SentenceListModel) -> some View {
        let movies = sortedMovies(lists)
        return VStack(alignment: .leading, spacing: 12) {
            Text(L("movies-you-can-usually-watch-a-movie-comfortably-once-you"))
                .font(.subheadline).foregroundStyle(Color.yapMuted)
            LazyVGrid(columns: Self.columns, spacing: 10) {
                ForEach(showAllMovies ? movies : Array(movies.prefix(12)), id: \.id) { movie in
                    let isSelected = selected == .Movie(id: movie.id)
                    Button { choose(.Movie(id: movie.id)) } label: {
                        VStack(spacing: 0) {
                            PosterImage(id: movie.id).aspectRatio(2 / 3, contentMode: .fit)
                            Text(L("curriculum-percent-known", ["percent": Int(movie.percent_known)])).font(.caption.monospaced().weight(.semibold))
                                .foregroundStyle(Color.yapText).padding(.vertical, 6).frame(maxWidth: .infinity)
                                .background(alignment: .leading) {
                                    GeometryReader { geometry in
                                        Color.yapText.opacity(0.1).frame(width: geometry.size.width * movie.percent_known / 100)
                                    }
                                }
                        }
                        .background(Color(uiColor: .systemBackground).opacity(0.6))
                        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
                        .overlay {
                            RoundedRectangle(cornerRadius: 10, style: .continuous)
                                .strokeBorder(isSelected ? Color.yapAccent : Color.yapBorder, lineWidth: isSelected ? 2.5 : 1)
                        }
                    }
                    .buttonStyle(.plain)
                    .accessibilityLabel(lists.title(.Movie(id: movie.id)))
                    .accessibilityAddTraits(isSelected ? .isSelected : [])
                }
            }
            if !showAllMovies && movies.count > 12 {
                Button(L("curriculum-show-all-movies", ["count": movies.count])) { showAllMovies = true }
                    .frame(maxWidth: .infinity).buttonStyle(.bordered)
            }
        }
    }

    @ViewBuilder private func pimsleur(_ lists: SentenceListModel) -> some View {
        if !acknowledged {
            VStack(spacing: 12) {
                Text(L("curriculum-browser-yap-has-word-lists-for-pimsleur-but-is-not"))
                    .font(.subheadline).foregroundStyle(Color.yapMuted).multilineTextAlignment(.center)
                Button(L("curriculum-browser-i-understand")) { acknowledged = true }
                    .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent)
            }.frame(maxWidth: .infinity).padding(.vertical, 16)
        } else {
            Text(L("curriculum-browser-focus-on-vocabulary-from-a-specific-pimsleur-lesson")).font(.subheadline).foregroundStyle(Color.yapMuted)
            ForEach(Array(Set(lists.lessons.map(\.level))).sorted(), id: \.self) { level in
                PimsleurLevel(level: level, open: { if case let .PimsleurLesson(selectedLevel, _) = selected { selectedLevel == level } else { false } }()) {
                    ForEach(lists.lessons.filter { $0.level == level }, id: \.lesson) { lesson in
                        let selection = SentenceListSelection.PimsleurLesson(level: level, lesson: lesson.lesson)
                        row(title: L("curriculum-lesson", ["lesson": lesson.lesson]), selection: selection,
                            progress: deck.get_sentence_list_progress(selection: selection, essential_percent_known: lists.tier.percent_known),
                            icon: "headphones")
                    }
                }
            }
        }
    }

    private func row(title: String, selection: SentenceListSelection?, progress: SentenceListProgress, icon: String?) -> some View {
        let isSelected = selection == selected
        return Button { choose(selection) } label: {
            HStack(spacing: 12) {
                if let icon { Image(systemName: icon).foregroundStyle(Color.yapMuted) }
                VStack(alignment: .leading, spacing: 8) {
                    HStack(spacing: 6) {
                        Text(title).font(.subheadline.weight(.semibold)).foregroundStyle(Color.yapText)
                        if isSelected { Image(systemName: "checkmark").font(.caption.weight(.bold)).foregroundStyle(Color.yapAccent) }
                    }
                    HStack(spacing: 8) {
                        ProgressView(value: progress.percent_known, total: 100)
                        Text(progress.bar_label).font(.caption.monospacedDigit()).foregroundStyle(Color.yapMuted)
                    }
                }
            }
            .padding(12).frame(maxWidth: .infinity, alignment: .leading)
            .insetSurface()
            .overlay { if isSelected { RoundedRectangle(cornerRadius: 12).strokeBorder(Color.yapAccent, lineWidth: 2) } }
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain).accessibilityAddTraits(isSelected ? .isSelected : [])
    }
}

private struct PimsleurLevel<Content: View>: View {
    let level: UInt32
    @State var open: Bool
    @ViewBuilder var content: Content
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Button { withAnimation(.snappy) { open.toggle() } } label: {
                HStack(spacing: 6) {
                    Text(L("curriculum-level", ["level": level])).font(.subheadline.weight(.semibold))
                    Image(systemName: "chevron.down").font(.caption2.weight(.semibold)).foregroundStyle(Color.yapMuted)
                        .rotationEffect(.degrees(open ? 180 : 0))
                }.foregroundStyle(Color.yapText).contentShape(Rectangle())
            }.buttonStyle(.plain)
            if open { content }
        }
    }
}

/// A movie poster that fills whatever frame it's given.
struct PosterImage: View {
    @Environment(\.reviewHost!) private var host
    let id: String
    @State private var image: UIImage?
    var body: some View {
        Rectangle().fill(Color.yapMutedSurface)
            .overlay {
                if let image { Image(uiImage: image).resizable().scaledToFill() }
                else { Image(systemName: "film").font(.title2).foregroundStyle(Color.yapMuted) }
            }
            .clipped()
            .task(id: id) { image = host.deck.get_movie_poster(movie_id: id).flatMap { UIImage(data: Data($0)) } }
    }
}
