import SwiftUI

@main struct YapApp: App {
    @State private var auth: AuthStore
    @State private var audio: AudioPlayer
    init() {
        do { try YapHost.initialize() } catch { fatalError("Unable to start Yap: \(error)") }
        Telemetry.start()
        _auth = State(initialValue: AuthStore())
        _audio = State(initialValue: AudioPlayer())
    }
    var body: some Scene {
        WindowGroup {
            Group {
                if auth.restoring { ProgressView("Restoring your session…") }
                else if let userId = auth.userId {
                    SessionRoot(userId: userId, auth: auth).id(userId)
                } else { SignInView() }
            }
            .environment(auth).environment(audio).tint(.yapAccent)
            #if DEBUG
            .task { await DebugHarness.shared.start(auth: auth) }
            #endif
        }
    }
}

struct SessionRoot: View {
    @Environment(\.scenePhase) private var scenePhase
    @Environment(AudioPlayer.self) private var audio
    @State private var session: YapSession
    let auth: AuthStore
    init(userId: String, auth: AuthStore) {
        self.auth = auth
        _session = State(initialValue: YapSession(userId: userId, accessToken: { [weak auth] in auth?.accessToken }))
    }
    var body: some View {
        Group {
            Group {
                if session.choosingCourse {
                    Color(uiColor: .systemGroupedBackground)
                } else if session.onboardingCourse != nil {
                    NavigationStack { CoursePickerView(session: session) }
                } else {
                    switch session.deckState {
                    case let .loading(message, progress):
                        VStack(spacing: 20) { ProgressView(value: progress); Text(message) }.padding(32)
                    case .noLanguageSelected: NavigationStack { CoursePickerView(session: session) }
                    case let .error(message):
                        ContentUnavailableView {
                            Label("Couldn't open your deck", systemImage: "exclamationmark.triangle")
                        } description: { Text(message) } actions: { Button("Try again") { session.retry() } }
                    case let .deck(deck, _, startingFresh, historyKnown):
                        CourseTabs(deck: deck, session: session, startingFresh: startingFresh, historyKnown: historyKnown)
                    }
                }
            }
        }
        .sheet(isPresented: Binding(get: { session.choosingCourse }, set: { session.choosingCourse = $0; if !$0 { session.onboardingCourse = nil } })) {
            NavigationStack {
                CoursePickerView(session: session) { session.choosingCourse = false }
                    .toolbar { Button("Done") { session.onboardingCourse = nil; session.choosingCourse = false } }
            }
        }
        .task { await session.start() }
        .onChange(of: auth.accessToken) { _, _ in session.tokenChanged() }
        .onChange(of: scenePhase) { _, phase in if phase == .active { session.sceneBecameActive() } }
        .onDisappear { session.stop(); audio.stop() }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            switch DebugHarness.shared.command {
            case "switch-course": session.choosingCourse = true
            case "force-display-name": DebugHarness.shared.forceDisplayName = true; auth.needsDisplayName = true; UserDefaults.standard.removeObject(forKey: "yap-skipped-set-display-name")
            case "sync": if DebugHarness.shared.activeTab != .settings { session.syncSoon() }
            case "status":
                if let weapon = session.weapon {
                    DebugHarness.log("selection=\(String(describing: weapon.get_deck_selection_state()))")
                    DebugHarness.log("events=\(weapon.num_events) remote=\(weapon.num_events_on_remote_as_of_last_sync(target: .Supabase)) finished=\(weapon.get_sync_state(target: .Supabase).last_sync_finished != nil) history=\(weapon.reviews_history_known())")
                }
            default: break
            }
        }
        #endif
    }
}
