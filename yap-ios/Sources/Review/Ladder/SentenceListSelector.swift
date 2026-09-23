import SwiftUI

struct SentenceListSelector: View {
    @Environment(\.reviewHost!) private var host
    @Environment(\.reviewActions!) private var actions
    let view: IdleView
    @AppStorage("yap-pimsleur-acknowledged") private var pimsleurAcknowledged = false
    private var info: NoCardsReadyInfo { view.info }
    private var navigation: SentenceListNavigation { view.navigation }
    private var title: String { view.sentence_list_label }
    var body: some View {
        let progress = view.progress
        VStack(alignment: .leading, spacing: 16) {
            Picker("Sentence list", selection: Binding(get: { navigation.categories[Int(navigation.selected_index)] }, set: { select($0) })) {
                ForEach(navigation.categories, id: \.self) { category in Text(String(describing: category)).tag(category) }
            }.pickerStyle(.segmented)
            (Text(view.curriculum_headline.before + "\n") + Text(view.curriculum_headline.emphasis.uppercased()).bold() + Text(view.curriculum_headline.after))
                .font(.headline).multilineTextAlignment(.center).frame(maxWidth: .infinity)
            if case let .Movie(id) = navigation.selection,
               let bytes = host.deck.get_movie_poster(movie_id: id), let image = UIImage(data: Data(bytes)) {
                Image(uiImage: image).resizable().scaledToFit().frame(maxHeight: 220).clipShape(RoundedRectangle(cornerRadius: 12)).accessibilityLabel("Poster for \(title)")
            }
            if case .PimsleurLesson = navigation.selection, !pimsleurAcknowledged {
                Text("Yap has word lists for Pimsleur, but is not affiliated with Pimsleur in any way.")
                Button("I understand") { pimsleurAcknowledged = true }
            } else {
                SentenceListProgressView(progress: progress)
                if let label = view.next_sentence_list_label, let next = view.next_sentence_list {
                    Button(label) { actions.setSentenceList(next) }
                } else if let note = view.all_learned_note {
                    Text(note).font(.subheadline)
                }
                if let note = view.level_note {
                    Text(note).font(.caption).foregroundStyle(.secondary)
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
    private func select(_ category: SentenceListCategory) {
        guard let option = view.sentence_list_options.first(where: { $0.category == category }),
              option.selection != view.navigation.selection else { return }
        actions.setSentenceList(option.selection)
    }
}
