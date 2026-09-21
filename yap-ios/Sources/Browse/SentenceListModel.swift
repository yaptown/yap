import SwiftUI

/// One immutable snapshot shared by the inline selector and the full browser.
@MainActor struct SentenceListModel {
    let deck: Deck
    let session: YapSession
    let tier: TierInfo
    let navigation: SentenceListNavigation
    let movies: [MovieStats]
    let metadata: [String: MovieMetadataBasic]
    let lessons: [PimsleurStats]
    init(deck: Deck, session: YapSession) {
        self.deck = deck; self.session = session
        tier = deck.get_current_tier()
        movies = deck.get_movie_stats()
        lessons = deck.get_pimsleur_stats()
        metadata = Dictionary(uniqueKeysWithValues: deck.get_movie_metadata(movie_ids: movies.map(\.id)).map { ($0.id, $0) })
        navigation = get_sentence_list_navigation(selection: deck.get_sentence_list(), has_movies: !movies.isEmpty, has_pimsleur: !lessons.isEmpty)
    }
    func title(_ selection: SentenceListSelection?) -> String {
        switch selection {
        case let .Movie(id): metadata[id]?.title ?? "Movie"
        case let .PimsleurLesson(level, lesson): "Pimsleur Level \(level), Lesson \(lesson)"
        case nil: "\(tier.name) \(get_language_metadata(language: deck.get_target_language()).common_name) Level \(tier.level)"
        }
    }
    func select(_ category: SentenceListCategory) {
        guard navigation.categories.contains(category) else { return }
        let fallback: String?
        if case let .Movie(id) = deck.get_best_movie_sentence_list() { fallback = id } else { fallback = nil }
        change(deck.get_sentence_list_for_category(category: category, fallback_movie_id: fallback))
    }
    func change(_ selection: SentenceListSelection?) {
        guard selection != deck.get_sentence_list() else { return }
        session.addDeckEvent(deck.change_sentence_list(sentence_list: selection))
    }
}

struct SentenceListProgressView: View {
    let progress: SentenceListProgress
    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            ProgressView(value: progress.percent_known, total: 100)
            Text(progress.all_available_learned ? "Done!" : "\(Int(progress.percent_known))% known").font(.caption).foregroundStyle(Color(uiColor: .secondaryLabel))
        }
    }
}

struct MoviePoster: View {
    @Environment(\.reviewHost!) private var host
    let id: String
    let title: String
    @State private var image: UIImage?
    var body: some View {
        Group {
            if let image { Image(uiImage: image).resizable().scaledToFit() }
            else { Image(systemName: "film").resizable().scaledToFit().padding(8).foregroundStyle(.secondary) }
        }.frame(width: 48, height: 72).clipShape(RoundedRectangle(cornerRadius: 6))
            .accessibilityLabel("Poster for \(title)")
            .task(id: id) { image = host.deck.get_movie_poster(movie_id: id).flatMap { UIImage(data: Data($0)) } }
    }
}
