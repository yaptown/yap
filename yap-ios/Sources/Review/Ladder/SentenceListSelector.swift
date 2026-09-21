import SwiftUI

struct SentenceListSelector: View {
    let actions: ReviewActions
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
            Text(title).font(.headline)
            if case let .Movie(id) = navigation.selection,
               let bytes = actions.media.moviePoster(id), let image = UIImage(data: Data(bytes)) {
                Image(uiImage: image).resizable().scaledToFit().frame(maxHeight: 220).clipShape(RoundedRectangle(cornerRadius: 12)).accessibilityLabel("Poster for \(title)")
            }
            if case .PimsleurLesson = navigation.selection, !pimsleurAcknowledged {
                Text("Yap has word lists for Pimsleur, but is not affiliated with Pimsleur in any way.")
                Button("I understand") { pimsleurAcknowledged = true }
            } else {
                SentenceListProgressView(progress: progress)
                if progress.all_available_learned {
                    Text("You're all done with \(title)!")
                    if let event = view.next_sentence_list_event {
                        Button("Next \(isMovie ? "movie" : "lesson")") {
                            actions.addEvent(event)
                        }
                    }
                } else if let milestone = next_progress_milestone(current: progress.percent_known, projected: info.percent_known_after) {
                    Text(info.recommend_more_cards ? "Soon you'll hit \(Int(milestone))%!" : "Learn \(info.smart_add_count) cards to hit \(Int(milestone))%.")
                } else { Text(info.recommend_more_cards ? "Keep up the momentum!" : "You're doing great!") }
                if navigation.selection == nil {
                    Text("When you complete this level, you'll understand \(info.tier_info.percent_of_usage, specifier: "%.1f")% of everyday \(get_language_metadata(language: view.target_language).common_name).")
                        .font(.caption).foregroundStyle(.secondary)
                }
            }
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeTab == .learn else { return }
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
    private var isMovie: Bool { if case .Movie = navigation.selection { true } else { false } }
    private func select(_ category: SentenceListCategory) {
        guard let option = view.sentence_list_options.first(where: { $0.category == category }),
              option.selection != view.navigation.selection else { return }
        actions.addEvent(option.event)
    }
}
