import SwiftUI

enum CourseRoute: Hashable {
    case review, goals, stats, dictionary(searching: Bool = false), due, settings
}

struct CourseHome: View {
    @Environment(AudioPlayer.self) private var audio
    let deck: Deck
    let session: YapSession
    let auth: AuthStore
    let startingFresh: Bool?
    let historyKnown: Bool
    @State private var path: [CourseRoute] = [.review]
    @State private var review: ReviewModel
    /// Home's search bar zooms into the dictionary.
    @Namespace private var dictionarySearch
    #if DEBUG
    @State private var debugClip: String?
    @State private var debugClipAvailable: Bool?
    #endif
    init(deck: Deck, session: YapSession, auth: AuthStore, startingFresh: Bool?, historyKnown: Bool) {
        self.deck = deck; self.session = session; self.auth = auth; self.startingFresh = startingFresh; self.historyKnown = historyKnown
        _review = State(initialValue: ReviewModel(deck: deck, session: session, auth: auth, startingFresh: startingFresh, historyKnown: historyKnown))
        #if DEBUG
        if case .Home = DebugHarness.shared.fixture { _path = State(initialValue: []) }
        #endif
    }
    var body: some View {
        NavigationStack(path: $path) {
            homeScreen
                .containerBackground(.clear, for: .navigation)
                .navigationDestination(for: CourseRoute.self) { route in
                    Group {
                        switch route {
                        case .review: reviewScreen.id(ObjectIdentifier(review.deck))
                        case .goals: GoalsScreen(review: review)
                        case .stats: StatsScreen(review: review) { navigate(.due) }
                        case let .dictionary(searching):
                            DictionaryScreen(session: session, searching: searching)
                                .navigationTransition(.zoom(sourceID: DictionarySearchBar.id, in: dictionarySearch))
                        case .due: DueWordsScreen(review: review)
                        case .settings: SettingsScreen(session: session)
                        }
                    }.containerBackground(.clear, for: .navigation)
                }
        }
        // The shell, not Review's visibility, owns background readiness.
        .onAppear { review.start() }
        // Leaving a route drops any browsed-but-uncommitted curriculum, as web's page state does.
        .onChange(of: path) { _, _ in audio.stop(); session.curriculumDraft = nil; review.refresh() }
        .onDisappear { review.stop(); audio.stop() }
        .onChange(of: auth.needsDisplayName) { _, _ in review.refresh() }
        .onChange(of: auth.accessToken) { _, _ in review.refresh() }
        .onChange(of: session.online) { _, _ in review.refresh() }
        .onChange(of: historyKnown) { _, known in review.historyKnown = known }
        .onChange(of: ObjectIdentifier(deck)) { _, _ in
            review.stop(); audio.stop()
            review = ReviewModel(deck: deck, session: session, auth: auth, startingFresh: startingFresh, historyKnown: historyKnown)
            review.start()
        }
        .overlay(alignment: .top) { VoiceActorBanner() }
        #if DEBUG
        .sheet(isPresented: Binding(get: { debugClip != nil }, set: { if !$0 { debugClip = nil } })) {
            if let text = debugClip {
                VStack(spacing: 20) {
                    Text(text)
                    VideoClipView( language: deck.get_target_language(), text: text,
                        reviewCount: deck.get_total_reviews(), available: $debugClipAvailable, movieId: .constant(nil))
                    Button("Done") { debugClip = nil }
                }.padding()
            }
        }
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.fixture == nil else { return }
            let command = DebugHarness.shared.command
            if command.hasPrefix("clip-text ") { audio.stop(); debugClipAvailable = nil; debugClip = String(command.dropFirst(10)) }
            if command == "clip-close" { debugClip = nil }
            if command.hasPrefix("tab ") {
                switch command.dropFirst(4).lowercased() {
                case "home": path = []
                case "learn", "review": navigate(.review)
                case "lists", "goals": navigate(.goals)
                case "stats": navigate(.stats)
                case "dictionary": navigate(.dictionary())
                case "due": navigate(.due)
                case "settings": navigate(.settings)
                default: break
                }
            }
            if DebugHarness.shared.activeScreen == .review, command.hasPrefix("audio-text ") {
                let request = AudioRequest(request: TtsRequest(text: String(command.dropFirst(11)), language: deck.get_target_language(), is_ssml: false, instructions: nil, speed: 1, verification_hints: []), provider: .Google)
                Task {
                    do { try await audio.play(request: request, accessToken: session.accessToken()) }
                    catch { DebugHarness.log("audio-text failed: \(error)") }
                }
            }
            if command == "status" { DebugHarness.log("screen=\(DebugHarness.shared.activeScreen.rawValue) audioPlaying=\(audio.isPlaying)") }
        }
        #endif
        .environment(\.reviewHost, review.host)
        .environment(\.reviewActions, review.actions)
    }
    @ViewBuilder private var homeScreen: some View {
        #if DEBUG
        if case let .Home(view) = DebugHarness.shared.fixture {
            fixtureScreen(.Home(view))
        } else {
            HomeScreen(review: review, navigate: navigate, searchTransition: dictionarySearch, isVisible: path.isEmpty)
        }
        #else
        HomeScreen(review: review, navigate: navigate, searchTransition: dictionarySearch, isVisible: path.isEmpty)
        #endif
    }
    @ViewBuilder private var reviewScreen: some View {
        #if DEBUG
        if let fixture = DebugHarness.shared.fixture {
            fixtureScreen(fixture)
        } else {
            liveReview
        }
        #else
        liveReview
        #endif
    }
    #if DEBUG
    private func fixtureScreen(_ fixture: Fixture) -> some View {
        Group {
            switch fixture {
            case let .Review(view): ReviewScreen(view: view)
            case let .Home(view): HomeScreen(review: review, navigate: { _ in }, searchTransition: dictionarySearch, view: view)
            case let .Stats(view): StatsScreen(review: review, showDue: {}, view: view)
            case let .Goals(view): GoalsScreen(review: review, view: view)
            case let .Due(view): DueWordsScreen(review: review, view: view)
            }
        }
        .environment(\.reviewActions, ReviewActions.inert)
        .allowsHitTesting(isReviewFixture(fixture))
        .onAppear { DebugHarness.log("fixture rendered \(DebugHarness.shared.fixtureName)") }
    }
    private func isReviewFixture(_ fixture: Fixture) -> Bool {
        if case .Review = fixture { return true }
        return false
    }
    #endif
    @ViewBuilder private var liveReview: some View {
        if let view = review.view {
            ReviewScreen(view: view)
            #if DEBUG
                .onChange(of: DebugHarness.shared.commandID) { _, _ in
                    guard DebugHarness.shared.activeScreen == .review else { return }
                    review.handleDebugCommand()
                }
            #endif
        } else {
            ProgressView()
        }
    }
    private func navigate(_ route: CourseRoute) {
        audio.stop()
        // Every top-level page returns directly to Home, including Stats → Due.
        path = [route]
    }

}
