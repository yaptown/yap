import SwiftUI

/// The all-caught-up curriculum card: chevrons on either side step through the
/// curricula (both slots always reserved so the content's width never changes),
/// and the learn button adds from whichever one is showing.
struct SentenceListSelector: View {
    @Environment(\.reviewHost!) private var host
    @Environment(\.reviewActions!) private var actions
    let view: IdleView
    let add: (DeckEvent) -> Void
    @AppStorage("yap-pimsleur-acknowledged") private var pimsleurAcknowledged = false
    @State private var browsing: SentenceListCategory?
    private var navigation: SentenceListNavigation { view.navigation }
    var body: some View {
        StudyCard(alignment: .center, spacing: 12, animated: true) {
            (Text(view.curriculum_headline.before + "\n") + Text(view.curriculum_headline.emphasis.uppercased()).bold() + Text(view.curriculum_headline.after))
                .font(.headline).multilineTextAlignment(.center)
                .fixedSize(horizontal: false, vertical: true).frame(maxWidth: .infinity)
            // The chevrons flank everything below the headline, centered on it, as on
            // the web's wide layout; they reach into the card's padding to keep the width.
            HStack(spacing: 0) {
                chevron(-1)
                VStack(spacing: 12) {
                    if case let .Movie(id) = navigation.selection, let bytes = host.deck.get_movie_poster(movie_id: id) {
                        SentenceListPoster(bytes: bytes, title: view.sentence_list_label).equatable()
                    }
                    if case .PimsleurLesson = navigation.selection, !pimsleurAcknowledged {
                        Image(systemName: "headphones").font(.title).foregroundStyle(Color.yapMuted)
                        Text(L("curriculum-browser-yap-has-word-lists-for-pimsleur-but-is-not"))
                            .font(.subheadline).foregroundStyle(Color.yapMuted).multilineTextAlignment(.center)
                        Button(L("curriculum-browser-i-understand")) { pimsleurAcknowledged = true }
                            .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent)
                    } else {
                        if let label = view.next_sentence_list_label, let next = view.next_sentence_list {
                            Button { actions.setSentenceList(next) } label: { Label(label, systemImage: "chevron.right") }
                                .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
                        } else if let note = view.all_learned_note {
                            Text(note).font(.subheadline)
                        } else if let label = view.curriculum_learn_label, let event = view.info.smart_add_event {
                            LearnButton(label: label, heading: view.manual_add_heading, options: view.manual_add_options,
                                        learn: { add(event) }, add: add)
                        }
                        LabeledProgressBar(percent: view.progress.percent_known, label: view.progress.bar_label,
                                           projected: view.progress.all_available_learned ? nil : view.info.percent_known_after)
                            .accessibilityLabel(view.progress.caption)
                        if let note = view.level_note {
                            Text(note).font(.caption).foregroundStyle(Color.yapMuted).multilineTextAlignment(.center)
                        }
                        Button(view.change_curriculum_label) { browsing = view.sentence_list_options[Int(navigation.selected_index)].category }
                            .buttonStyle(.bordered).controlSize(.small)
                    }
                }
                .frame(maxWidth: .infinity)
                chevron(1)
            }
            .fixedSize(horizontal: false, vertical: true).padding(.horizontal, -12)
        }
        .sheet(item: $browsing) { category in CurriculumBrowser(tab: category) }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeScreen == .review else { return }
            switch DebugHarness.shared.command {
            case "acknowledge-pimsleur": pimsleurAcknowledged = true
            case "list essential": select(.Essential)
            case "list movie": select(.Movie)
            case "list pimsleur": select(.Pimsleur)
            default: break
            }
        }
        #endif
    }
    @ViewBuilder private func chevron(_ step: Int) -> some View {
        let index = Int(navigation.selected_index) + step
        let enabled = view.sentence_list_options.indices.contains(index)
        if view.sentence_list_options.count > 1 {
            Button {
                actions.setSentenceList(view.sentence_list_options[index].selection)
            } label: {
                // Full height, so the whole side of the card is the tap target.
                Image(systemName: step < 0 ? "chevron.left" : "chevron.right")
                    .font(.title3.weight(.semibold)).frame(width: 32).frame(maxHeight: .infinity).contentShape(Rectangle())
            }
            .buttonStyle(.plain).foregroundStyle(Color.yapText.opacity(0.6))
            .opacity(enabled ? 1 : 0).disabled(!enabled)
            .accessibilityLabel(step < 0 ? L("idle-screen-previous-sentence-list") : L("idle-screen-next-sentence-list"))
        }
    }
    private func select(_ category: SentenceListCategory) {
        guard let option = view.sentence_list_options.first(where: { $0.category == category }),
              option.selection != view.navigation.selection else { return }
        actions.setSentenceList(option.selection)
    }
}

/// The web's split learn button: the smart add, with the manual card-type adds
/// in a menu on its trailing segment.
private struct LearnButton: View {
    let label: String
    let heading: String
    let options: [ManualAddOption]
    let learn: () -> Void
    let add: (DeckEvent) -> Void
    var body: some View {
        HStack(spacing: 0) {
            Button(action: learn) {
                Label(label, systemImage: "sparkles").lineLimit(1).minimumScaleFactor(0.7).padding(.horizontal, 14).frame(minHeight: 50)
            }
            Rectangle().fill(Tokens.palette.primary_foreground.color.opacity(0.2)).frame(width: 1, height: 50)
            Menu {
                ForEach(Array(options.enumerated()), id: \.offset) { _, option in
                    Button(option.label) { add(option.event) }
                }
            } label: {
                Image(systemName: "chevron.down").frame(width: 48, height: 50).contentShape(Rectangle())
            }.accessibilityLabel(heading)
        }
        .font(.headline).foregroundStyle(Tokens.palette.primary_foreground.color)
        .background(Tokens.palette.primary.color.opacity(0.85))
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
    }
}

/// Image decoding is independent of the selector's live navigation/add actions.
private struct SentenceListPoster: View, Equatable {
    let bytes: [UInt8]
    let title: String

    var body: some View {
        if let image = UIImage(data: Data(bytes)) {
            Image(uiImage: image).resizable().scaledToFit().frame(maxHeight: 220).clipShape(RoundedRectangle(cornerRadius: 12))
                .accessibilityLabel(L("common-poster-for", ["title": title]))
        }
    }
}
