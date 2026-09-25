import SwiftUI

struct MoviePosterGrid: View {
    let movies: [MovieMetadataBasic]
    var body: some View {
        if !movies.isEmpty {
            LazyVGrid(columns: [GridItem(.flexible(), spacing: 12), GridItem(.flexible(), spacing: 12)], spacing: 12) {
                ForEach(Array(movies.prefix(2)), id: \.id) { movie in MoviePosterCard(movie: movie) }
            }
        }
    }
}

private struct MoviePosterCard: View {
    @Environment(\.reviewHost!) private var host
    let movie: MovieMetadataBasic
    @State private var image: UIImage?
    var body: some View {
        Color(uiColor: .tertiarySystemFill)
            .aspectRatio(2 / 3, contentMode: .fit)
            .overlay {
                if let image { Image(uiImage: image).resizable().scaledToFill() }
                else { Text("🎬").font(.system(size: 36)) }
            }
            .clipShape(RoundedRectangle(cornerRadius: 20))
            .overlay { RoundedRectangle(cornerRadius: 20).strokeBorder(Color(uiColor: .separator).opacity(0.5)) }
            .accessibilityLabel("Poster for \(movie.title)")
            .task(id: movie.id) { image = host.deck.get_movie_poster(movie_id: movie.id).flatMap { UIImage(data: Data($0)) } }
    }
}
