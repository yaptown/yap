import SwiftUI

enum CourseTab: String, CaseIterable {
    case learn = "Learn", stats = "Stats", dictionary = "Dictionary", lists = "Lists", settings = "Settings"
    var icon: String {
        switch self { case .learn: "rectangle.on.rectangle"; case .stats: "chart.xyaxis.line"; case .dictionary: "character.book.closed"; case .lists: "list.bullet"; case .settings: "gearshape" }
    }
}

struct CourseTabs: View {
    @Environment(AudioPlayer.self) private var audio
    let deck: Deck
    let session: YapSession
    let auth: AuthStore
    let startingFresh: Bool?
    let historyKnown: Bool
    @State private var tab: CourseTab = .learn
    @State private var review: ReviewModel
    #if DEBUG
    @State private var debugClip: String?
    @State private var debugClipAvailable: Bool?
    #endif
    init(deck: Deck, session: YapSession, auth: AuthStore, startingFresh: Bool?, historyKnown: Bool) {
        self.deck = deck; self.session = session; self.auth = auth; self.startingFresh = startingFresh; self.historyKnown = historyKnown
        _review = State(initialValue: ReviewModel(deck: deck, session: session, auth: auth, startingFresh: startingFresh, historyKnown: historyKnown))
    }
    var body: some View {
        TabView(selection: Binding(get: { tab }, set: { select($0) })) {
            NavigationStack {
                reviewScreen.id(ObjectIdentifier(review.deck))
            }.tabItem { Label("Learn", systemImage: CourseTab.learn.icon) }.tag(CourseTab.learn)
            NavigationStack { StatsScreen(deck: deck, session: session) }
                .tabItem { Label("Stats", systemImage: CourseTab.stats.icon) }.tag(CourseTab.stats)
            NavigationStack { DictionaryScreen(deck: deck, session: session) }
                .tabItem { Label("Dictionary", systemImage: CourseTab.dictionary.icon) }.tag(CourseTab.dictionary)
            NavigationStack { SentenceListsScreen(deck: deck, session: session) { select(.learn) } }
                .tabItem { Label("Lists", systemImage: CourseTab.lists.icon) }.tag(CourseTab.lists)
            NavigationStack { SettingsScreen(session: session) }
                .tabItem { Label("Settings", systemImage: CourseTab.settings.icon) }.tag(CourseTab.settings)
        }
        // The tab container, not Learn's visibility, owns background readiness.
        .onAppear {
            review.start()
            #if DEBUG
            DebugHarness.shared.activeTab = tab
            #endif
        }
        .onDisappear { review.stop(); audio.stop() }
        .onChange(of: auth.needsDisplayName) { _, _ in review.refresh() }
        .onChange(of: auth.accessToken) { _, _ in review.refresh() }
        .onChange(of: session.online) { _, _ in review.refresh() }
        .onChange(of: historyKnown) { _, known in review.historyKnown = known }
        .onChange(of: ObjectIdentifier(deck)) { _, _ in
            review.stop()
            review = ReviewModel(deck: deck, session: session, auth: auth, startingFresh: startingFresh, historyKnown: historyKnown)
            review.start()
        }
        .overlay(alignment: .top) { VoiceActorBanner() }
        #if DEBUG
        .sheet(isPresented: Binding(get: { debugClip != nil }, set: { if !$0 { debugClip = nil } })) {
            if let text = debugClip {
                VStack(spacing: 20) {
                    Text(text)
                    VideoClipView( language: deck.get_target_language(), text: text, media: .live(deck: deck, session: session),
                        reviewCount: deck.get_total_reviews(), available: $debugClipAvailable)
                    Button("Done") { debugClip = nil }
                }.padding()
            }
        }
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            let command = DebugHarness.shared.command
            if command.hasPrefix("clip-text ") { audio.stop(); debugClipAvailable = nil; debugClip = String(command.dropFirst(10)) }
            if command == "clip-close" { debugClip = nil }
            if command.hasPrefix("tab "), let next = CourseTab.allCases.first(where: { $0.rawValue.lowercased() == command.dropFirst(4).lowercased() }) { select(next) }
            if tab == .learn, command.hasPrefix("audio-text ") {
                let request = AudioRequest(request: TtsRequest(text: String(command.dropFirst(11)), language: deck.get_target_language(), is_ssml: false, instructions: nil, speed: 1, verification_hints: []), provider: .Google)
                Task {
                    do { try await audio.play(request: request, accessToken: session.accessToken()) }
                    catch { DebugHarness.log("audio-text failed: \(error)") }
                }
            }
            if command == "status" { DebugHarness.log("tab=\(tab.rawValue) audioPlaying=\(audio.isPlaying)") }
        }
        #endif
    }
    @ViewBuilder private var reviewScreen: some View {
        #if DEBUG
        if let fixture = DebugHarness.shared.fixture {
            ReviewScreen(view: fixture, actions: .inert)
                .onAppear { DebugHarness.log("fixture rendered \(DebugHarness.shared.fixtureName)") }
        } else {
            liveReview
        }
        #else
        liveReview
        #endif
    }
    private var liveReview: some View {
        ReviewScreen(view: review.view, actions: review.actions)
        #if DEBUG
            .onChange(of: DebugHarness.shared.commandID) { _, _ in
                guard DebugHarness.shared.activeTab == .learn else { return }
                review.handleDebugCommand()
            }
        #endif
    }
    private func select(_ next: CourseTab) {
        audio.stop()
        tab = next
        #if DEBUG
        DebugHarness.shared.activeTab = next
        #endif
    }

}
