import SwiftUI

struct CardSummaryList: View {
    let cards: [CardSummary]
    let timestampMs: Double
    @ScaledMetric(relativeTo: .caption) private var readyColumnWidth = 90.0
    @State private var revealed: Set<CardIndicator_Gram_String_String> = []
    var body: some View {
        StudyCard {
            HStack {
                Text("Word")
                Spacer()
                Text("Ready").frame(width: readyColumnWidth, alignment: .leading)
            }.font(.subheadline.bold())
            LazyVStack(alignment: .leading, spacing: 14) {
                ForEach(cards, id: \.card_indicator) { card in
                    Divider()
                    HStack(alignment: .top, spacing: 16) {
                        VStack(alignment: .leading, spacing: 4) {
                            if case .ListeningGram = card.card_indicator, !revealed.contains(card.card_indicator) {
                                Button { revealed.insert(card.card_indicator) } label: {
                                    VStack(alignment: .leading, spacing: 4) {
                                        Text(card.card_text).fontWeight(.semibold).blur(radius: 5).accessibilityHidden(true)
                                        Text("Tap to reveal").font(.caption).italic().foregroundStyle(.secondary)
                                    }
                                }.buttonStyle(.plain).accessibilityLabel("Reveal listening lexeme")
                            } else { Text(card.card_text).fontWeight(.semibold) }
                            if let subtitle = card.card_subtitle { Text(subtitle).font(.subheadline).foregroundStyle(.secondary) }
                        }.frame(maxWidth: .infinity, alignment: .leading)
                        Group {
                            if card.due_timestamp_ms <= timestampMs {
                                Text("Ready now").foregroundStyle(Color.yapText).padding(.horizontal, 8).padding(.vertical, 3)
                                    .overlay { Capsule().strokeBorder(Color(uiColor: .separator)) }
                            }
                            else { Text(Date(timeIntervalSince1970: card.due_timestamp_ms / 1000), style: .relative) }
                        }.font(.caption).foregroundStyle(.secondary).frame(width: readyColumnWidth, alignment: .leading)
                    }
                }
            }
        }
    }
}
