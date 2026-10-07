import SwiftUI

/// One immutable snapshot shared by the inline selector and the full browser.
@MainActor struct SentenceListModel {
    let deck: Deck
    let tier: TierInfo
    let navigation: SentenceListNavigation
    let movies: [MovieStats]
    let metadata: [String: MovieMetadataBasic]
    let lessons: [PimsleurStats]
    init(deck: Deck) {
        self.deck = deck
        tier = deck.get_current_tier()
        movies = deck.get_movie_stats()
        lessons = deck.get_pimsleur_stats()
        metadata = Dictionary(uniqueKeysWithValues: deck.get_movie_metadata(movie_ids: movies.map(\.id)).map { ($0.id, $0) })
        navigation = get_sentence_list_navigation(selection: deck.get_sentence_list(), has_movies: !movies.isEmpty, has_pimsleur: !lessons.isEmpty)
    }
    func title(_ selection: SentenceListSelection?) -> String {
        switch selection {
        case let .Movie(id): metadata[id]?.title ?? L("curriculum-movie")
        case let .PimsleurLesson(level, lesson): L("curriculum-pimsleur-level-level-lesson-lesson", ["level": level, "lesson": lesson])
        case nil: L("curriculum-tier-level", ["tier": tier.name, "language": get_language_metadata(language: deck.get_target_language()).common_name, "level": tier.level])
        }
    }
}

struct SentenceListProgressView: View, Equatable {
    let progress: SentenceListProgress
    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            ProgressView(value: progress.percent_known, total: 100)
            Text(progress.caption).font(.caption).foregroundStyle(Color.yapMuted)
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
            else { Image(systemName: "film").resizable().scaledToFit().padding(8).foregroundStyle(Color.yapMuted) }
        }.frame(width: 48, height: 72).clipShape(RoundedRectangle(cornerRadius: 6))
            .accessibilityLabel(L("common-poster-for", ["title": title]))
            .task(id: id) { image = host.deck.get_movie_poster(movie_id: id).flatMap { UIImage(data: Data($0)) } }
    }
}
