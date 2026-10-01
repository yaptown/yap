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
        StudyCard(alignment: .center, spacing: 12, animated: true) {
            (Text(view.curriculum_headline.before + "\n") + Text(view.curriculum_headline.emphasis.uppercased()).bold() + Text(view.curriculum_headline.after))
                .font(.headline).multilineTextAlignment(.center)
                .fixedSize(horizontal: false, vertical: true).frame(maxWidth: .infinity)
            if case let .Movie(id) = navigation.selection, let bytes = host.deck.get_movie_poster(movie_id: id) {
                SentenceListPoster(bytes: bytes, title: view.sentence_list_label).equatable()
            }
            if case .PimsleurLesson = navigation.selection, !pimsleurAcknowledged {
                Image(systemName: "headphones").font(.title).foregroundStyle(Color.yapMuted)
                Text("Yap has word lists for Pimsleur, but is not affiliated with Pimsleur in any way.")
                    .font(.subheadline).foregroundStyle(Color.yapMuted).multilineTextAlignment(.center)
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
                LevelProgressBar(progress: view.progress,
                                 projected: view.progress.all_available_learned ? nil : view.info.percent_known_after).equatable()
                if let note = view.level_note {
                    Text(note).font(.caption).foregroundStyle(Color.yapMuted).multilineTextAlignment(.center)
                }
                Button(view.change_sentence_list_label, action: actions.openGoals)
                    .font(.caption).underline().foregroundStyle(Color.yapText.opacity(0.6)).buttonStyle(.plain)
            }
            // Below everything, as on the web's phone layout.
            if view.sentence_list_options.count > 1 {
                HStack { chevron(-1); Spacer(); chevron(1) }
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
        .buttonStyle(.plain).foregroundStyle(Color.yapText.opacity(0.6))
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

/// The web's level bar: thick, with a lighter segment for where the offered
/// cards would take it and the percentage printed inside, inverted over the fill.
private struct LevelProgressBar: View, Equatable {
    let progress: SentenceListProgress
    let projected: Double?
    var body: some View {
        let known = min(max(progress.percent_known / 100, 0), 1)
        let after = min(max((projected ?? 0) / 100, known), 1)
        let label = Text(progress.bar_label).font(.subheadline.monospaced().weight(.semibold))
        GeometryReader { geometry in
            let width = geometry.size.width
            ZStack(alignment: .leading) {
                Color.yapAccent.opacity(0.1)
                Color.yapAccent.opacity(0.3).frame(width: width * after)
                Color.yapAccent.frame(width: width * known)
                label.foregroundStyle(Color.yapAccent).frame(maxWidth: .infinity)
                label.foregroundStyle(Color.yapOnAccent).frame(maxWidth: .infinity)
                    .mask(alignment: .leading) { Rectangle().frame(width: width * known) }
            }
        }
        .frame(height: 24).clipShape(Capsule())
        .accessibilityElement().accessibilityLabel(progress.caption)
    }
}

/// Image decoding is independent of the selector's live navigation/add actions.
private struct SentenceListPoster: View, Equatable {
    let bytes: [UInt8]
    let title: String

    var body: some View {
        if let image = UIImage(data: Data(bytes)) {
            Image(uiImage: image).resizable().scaledToFit().frame(maxHeight: 220).clipShape(RoundedRectangle(cornerRadius: 12))
                .accessibilityLabel("Poster for \(title)")
        }
    }
}
