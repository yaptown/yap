import SwiftUI

/// Both lists can run to tens of thousands of words, so they grow a page at a time.
private let pageSize: UInt32 = 200

struct YourCardsScreen: View {
    let review: ReviewModel
    var view: YourCardsScreenView? = nil
    @State private var limit = pageSize
    var body: some View {
        TimelineView(.every(10)) { _ in
            let now = ReviewModel.now
            let view = self.view ?? review.deck.your_cards_view(limit: limit)
            WordList(title: view.summary.title, label: view.summary.label, showMore: view.show_more, onShowMore: { limit += pageSize }) {
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
    @State private var limit = pageSize
    var body: some View {
        let view = self.view ?? review.deck.assumed_known_view(limit: limit)
        WordList(title: view.summary.title, label: view.summary.label, showMore: view.show_more, onShowMore: { limit += pageSize }) {
            Text(view.note).font(.subheadline).foregroundStyle(Color.yapMuted)
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
            Button(word.add_label, systemImage: "plus.circle", action: add)
                .labelStyle(.iconOnly).font(.title3).foregroundStyle(Color.yapText)
                .accessibilityLabel("\(word.add_label): \(word.display_text)")
        }
    }
}

private struct WordList<Content: View>: View {
    let title: String
    let label: String
    let showMore: String?
    let onShowMore: () -> Void
    @ViewBuilder let content: Content
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                Text(label).foregroundStyle(Color.yapMuted)
                content
                if let showMore { Button(showMore, action: onShowMore).buttonStyle(.bordered).frame(maxWidth: .infinity) }
            }.padding(20).frame(maxWidth: .readableColumn).frame(maxWidth: .infinity)
        }.background(.clear).navigationTitle(title)
    }
}
