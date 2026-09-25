import SwiftUI

/// The all-caught-up curriculum card: chevrons step through the curricula (both
/// slots always reserved so the headline's width never changes), and the learn
/// button adds from whichever one is showing.
struct SentenceListSelector: View {
    @Environment(\.reviewHost!) private var host
    @Environment(\.reviewActions!) private var actions
    let view: IdleView
    let add: (DeckEvent) -> Void
    @AppStorage("yap-pimsleur-acknowledged") private var pimsleurAcknowledged = false
    private var navigation: SentenceListNavigation { view.navigation }
    var body: some View {
        StudyCard(alignment: .center, spacing: 16, animated: true) {
            HStack(spacing: 0) {
                chevron(-1)
                (Text(view.curriculum_headline.before + "\n") + Text(view.curriculum_headline.emphasis.uppercased()).bold() + Text(view.curriculum_headline.after))
                    .font(.headline).multilineTextAlignment(.center)
                    // Beside the fixed-height chevrons, SwiftUI otherwise offers one line and truncates at the break.
                    .fixedSize(horizontal: false, vertical: true).frame(maxWidth: .infinity)
                chevron(1)
            }
            if case let .Movie(id) = navigation.selection,
               let bytes = host.deck.get_movie_poster(movie_id: id), let image = UIImage(data: Data(bytes)) {
                Image(uiImage: image).resizable().scaledToFit().frame(maxHeight: 220).clipShape(RoundedRectangle(cornerRadius: 12))
                    .accessibilityLabel("Poster for \(view.sentence_list_label)")
            }
            if case .PimsleurLesson = navigation.selection, !pimsleurAcknowledged {
                Image(systemName: "headphones").font(.title).foregroundStyle(.secondary)
                Text("Yap has word lists for Pimsleur, but is not affiliated with Pimsleur in any way.")
                    .font(.subheadline).foregroundStyle(.secondary).multilineTextAlignment(.center)
                Button("I understand") { pimsleurAcknowledged = true }
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
                SentenceListProgressView(progress: view.progress)
                if let note = view.level_note {
                    Text(note).font(.caption).foregroundStyle(.secondary).frame(maxWidth: .infinity, alignment: .leading)
                }
            }
        }
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
    private func chevron(_ step: Int) -> some View {
        let index = Int(navigation.selected_index) + step
        let enabled = view.sentence_list_options.indices.contains(index)
        return Button {
            actions.setSentenceList(view.sentence_list_options[index].selection)
        } label: {
            Image(systemName: step < 0 ? "chevron.left" : "chevron.right")
                .font(.title3.weight(.semibold)).frame(width: 36, height: 44).contentShape(Rectangle())
        }
        .buttonStyle(.plain).foregroundStyle(.secondary)
        .opacity(enabled ? 1 : 0).disabled(!enabled)
        .accessibilityLabel(step < 0 ? "Previous sentence list" : "Next sentence list")
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
                Label(label, systemImage: "sparkles").padding(.horizontal, 18).frame(minHeight: 50)
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
