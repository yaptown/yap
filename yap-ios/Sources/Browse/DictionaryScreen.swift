import SwiftUI

struct DictionaryScreen: View {
    @Environment(AudioPlayer.self) private var audio
    @Environment(\.reviewHost!) private var host
    private var deck: Deck { host.deck }
    let session: YapSession
    @State private var query = ""
    @State private var entries: [DictionaryWord] = []
    @State private var path: [UInt64] = []
    @State private var hasMore = false
    @FocusState private var searchFocused: Bool
    private let searching: Bool
    private let pageSize: UInt64 = 200
    init(session: YapSession, searching: Bool = false) {
        self.session = session
        self.searching = searching
    }
    var body: some View {
        results
        .scrollContentBackground(.hidden)
        .navigationTitle("Dictionary")
        .searchable(text: $query, prompt: "Search words or meanings")
        .searchFocused($searchFocused)
        .onAppear { if searching { searchFocused = true } }
        .navigationDestination(isPresented: Binding(get: { !path.isEmpty }, set: { if !$0 { path = [] } })) {
            if let index = path.last { DictionaryDetail(session: session, index: index) }
        }
        .task(id: query) {
            // Typing is debounced, but the first page loads at once so the
            // zoom in from Home never shows an empty list.
            if !entries.isEmpty {
                do { try await Task.sleep(for: .milliseconds(200)) } catch { return }
            }
            reload()
        }
        .onChange(of: ObjectIdentifier(deck)) { _, _ in reload(preservingPageCount: true) }
        .onDisappear { audio.stop() }
        #if DEBUG
        .onAppear { DebugHarness.shared.activeScreen = .dictionary }
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeScreen == .dictionary else { return }
            let command = DebugHarness.shared.command
            if command.hasPrefix("search ") { path = []; query = String(command.dropFirst(7)) }
            if command.hasPrefix("open "), let index = Int(command.dropFirst(5)), entries.indices.contains(index) { path = [entries[index].frequency_index] }
            if command == "page" { loadMore() }
            if command == "status" { DebugHarness.log("dictionary results=\(entries.count) first=\(entries.first?.display_text ?? "none") detail=\(String(describing: path.last))") }
        }
        #endif
    }
    private var results: some View {
        List {
            Section {
                ForEach(entries, id: \.frequency_index) { entry in
                    Button { path.append(entry.frequency_index) } label: {
                        HStack(spacing: 12) {
                            VStack(alignment: .leading, spacing: 4) {
                                Text((entry.prefix.map { $0.prefix + $0.separator } ?? "") + entry.display_text).foregroundStyle(Color.yapText)
                                let gloss = entry.gloss
                                if !gloss.isEmpty { Text(gloss).font(.subheadline).foregroundStyle(.secondary) }
                                if entry.is_phrase { Text("(phrase)").font(.caption).foregroundStyle(.secondary) }
                            }
                            Spacer()
                            if entry.senses.contains(where: { $0.is_in_deck }) { Image(systemName: "checkmark.circle").accessibilityLabel("In deck") }
                            Image(systemName: "chevron.right").font(.caption).foregroundStyle(.secondary)
                        }.frame(minHeight: 44)
                    }
                }
                if hasMore { Button("Load 200 more") { loadMore() } }
                if entries.isEmpty { ContentUnavailableView.search(text: query) }
            } header: { Text("\(entries.count) results · \(deck.get_gram_dictionary_count()) dictionary entries") }
            // One glass card behind hundreds of lazy rows would be costly, so the
            // rows take the card's translucent fallback instead of opaque white.
            .listRowBackground(Rectangle().fill(.ultraThinMaterial).opacity(0.65).overlay(Tokens.palette.card.color.opacity(0.18)))
        }
    }
    private func reload(preservingPageCount: Bool = false) {
        let pages = preservingPageCount ? max(1, (entries.count + Int(pageSize) - 1) / Int(pageSize)) : 1
        entries = []
        for _ in 0..<pages { loadMore() }
    }
    private func loadMore() {
        let text = query.trimmingCharacters(in: .whitespacesAndNewlines)
        let page = deck.get_gram_dictionary_page(search_query: text.isEmpty ? nil : text, offset: UInt64(entries.count), limit: pageSize)
        entries += page; hasMore = page.count == Int(pageSize)
    }
}

private struct DictionaryDetail: View {
    @Environment(AudioPlayer.self) private var audio
    @Environment(\.reviewHost!) private var host
    private var deck: Deck { host.deck }
    let session: YapSession
    let index: UInt64
    @State private var added: Set<UInt64> = []
    private var entry: DictionaryWord? { deck.gram_dictionary_entry(frequency_index: index) }
    var body: some View {
        ScrollView {
            if let entry {
                StudyCard {
                    Text((entry.prefix.map { $0.prefix + $0.separator } ?? "") + entry.display_text).font(.largeTitle.bold())
                    if entry.is_phrase { Text("(phrase)").font(.caption).foregroundStyle(.secondary) }
                    AudioButton(request: entry.audio_request, reviewCount: deck.get_total_reviews())
                    ForEach(entry.senses, id: \.frequency_index) { sense in
                        VStack(alignment: .leading, spacing: 12) {
                            DefinitionBoxesView(definition: sense.definition)
                            let isAdded = added.contains(sense.frequency_index) || sense.is_in_deck
                            Button(isAdded ? "Added" : "Add to deck", systemImage: isAdded ? "checkmark.circle" : "plus.circle") { add(sense) }
                                .buttonStyle(.borderedProminent).foregroundStyle(isAdded ? Color(uiColor: .secondaryLabel) : Color.yapOnAccent)
                                .disabled(isAdded)
                        }
                    }
                }.padding(20)
            }
        }.background(.clear).navigationTitle("Definition").navigationBarTitleDisplayMode(.inline)
        .onDisappear { audio.stop() }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            if DebugHarness.shared.activeScreen == .dictionary && DebugHarness.shared.command == "add-word", let sense = entry?.senses.first { add(sense) }
        }
        #endif
    }
    private func add(_ sense: DictionarySense) {
        guard !added.contains(sense.frequency_index), !sense.is_in_deck, let event = deck.add_gram_by_frequency_index(frequency_index: sense.frequency_index) else { return }
        added.insert(sense.frequency_index); session.addDeckEvent(event)
    }
}
