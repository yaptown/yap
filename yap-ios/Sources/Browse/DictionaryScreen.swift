import SwiftUI

struct DictionaryScreen: View {
    let deck: Deck
    let session: YapSession
    @State private var query = ""
    @State private var entries: [GramDictionaryEntry] = []
    @State private var path: [UInt64] = []
    @State private var hasMore = false
    private let pageSize: UInt64 = 200
    var body: some View {
        List {
            Section {
                ForEach(entries, id: \.frequency_index) { entry in
                    Button { path.append(entry.frequency_index) } label: {
                        HStack(spacing: 12) {
                            VStack(alignment: .leading, spacing: 4) {
                                Text((entry.prefix.map { $0.prefix + $0.separator } ?? "") + entry.display_text).foregroundStyle(Color.yapText)
                                if entry.is_phrase { Text("Phrase").font(.caption).foregroundStyle(.secondary) }
                            }
                            Spacer()
                            if entry.is_in_deck { Image(systemName: "checkmark.circle").accessibilityLabel("In deck") }
                            Image(systemName: "chevron.right").font(.caption).foregroundStyle(.secondary)
                        }.frame(minHeight: 44)
                    }
                }
                if hasMore { Button("Load 200 more") { loadMore() } }
                if entries.isEmpty { ContentUnavailableView.search(text: query) }
            } header: { Text("\(entries.count) results · \(deck.get_gram_dictionary_count()) dictionary entries") }
        }
        .navigationTitle("Dictionary")
        .searchable(text: $query, prompt: "Search words or meanings")
        .navigationDestination(isPresented: Binding(get: { !path.isEmpty }, set: { if !$0 { path = [] } })) {
            if let index = path.last { DictionaryDetail(deck: deck, session: session, index: index) }
        }
        .task(id: query) {
            do { try await Task.sleep(for: .milliseconds(200)) } catch { return }
            reload()
        }
        .onChange(of: ObjectIdentifier(deck)) { _, _ in reload(preservingPageCount: true) }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            let command = DebugHarness.shared.command
            if command.hasPrefix("search ") { path = []; query = String(command.dropFirst(7)) }
            if command.hasPrefix("open "), let index = Int(command.dropFirst(5)), entries.indices.contains(index) { path = [entries[index].frequency_index] }
            if command == "page" { loadMore() }
            if command == "status" { DebugHarness.log("dictionary results=\(entries.count) first=\(entries.first?.display_text ?? "none") detail=\(String(describing: path.last))") }
        }
        #endif
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
    let deck: Deck
    let session: YapSession
    let index: UInt64
    @State private var added = false
    private var entry: GramDictionaryEntry? { deck.gram_dictionary_entry(frequency_index: index) }
    var body: some View {
        ScrollView {
            if let entry {
                StudyCard {
                    Text((entry.prefix.map { $0.prefix + $0.separator } ?? "") + entry.display_text).font(.largeTitle.bold())
                    if entry.is_phrase { Text("Phrase").font(.caption).foregroundStyle(.secondary) }
                    AudioButton(request: entry.audio_request, session: session, reviewCount: deck.get_total_reviews())
                    DefinitionView(entry: entry)
                    Button(added || entry.is_in_deck ? "In your deck" : "Add to deck", systemImage: added || entry.is_in_deck ? "checkmark.circle" : "plus.circle") { add() }
                        .buttonStyle(.borderedProminent).foregroundStyle(added || entry.is_in_deck ? Color(uiColor: .secondaryLabel) : Color.yapOnAccent)
                        .disabled(added || entry.is_in_deck)
                }.padding(20)
            }
        }.background(Color(uiColor: .systemGroupedBackground)).navigationTitle("Definition").navigationBarTitleDisplayMode(.inline)
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in if DebugHarness.shared.command == "add-word" { add() } }
        #endif
    }
    private func add() {
        guard !added, let entry, !entry.is_in_deck, let event = deck.add_gram_by_frequency_index(frequency_index: index) else { return }
        added = true; session.addDeckEvent(event)
    }
}
