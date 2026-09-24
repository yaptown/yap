import SwiftUI

@main struct YapApp: App {
    @State private var auth: AuthStore
    @State private var audio: AudioPlayer
    @State private var background = BackgroundController()
    @State private var authSheet = AuthSheet()
    @State private var sessions = SessionLifecycle()
    init() {
        do { try YapHost.initialize() } catch { fatalError("Unable to start Yap: \(error)") }
        Telemetry.start()
        _auth = State(initialValue: AuthStore())
        _audio = State(initialValue: AudioPlayer())
    }
    var body: some Scene {
        WindowGroup {
            ZStack {
                AnimatedBackground()
                if auth.restoring { ProgressView("Restoring your session…") }
                else if let session = sessions.session {
                    SessionRoot(session: session, auth: auth).id(session.userId ?? "anon")
                } else if let error = sessions.error {
                    Text(error).foregroundStyle(Color.yapNegativeForeground).padding()
                } else { ProgressView() }
            }
            .onChange(of: auth.restoring ? "restoring" : auth.userId ?? "anon", initial: true) { _, _ in
                guard !auth.restoring else { return }
                audio.stopAll()
                sessions.changeIdentity(auth: auth)
            }
            .sheet(isPresented: $authSheet.isPresented) { SignInView() }
            .environment(background).environment(auth).environment(audio).environment(authSheet).tint(.yapAccent)
            #if DEBUG
            .task { await DebugHarness.shared.start(auth: auth) }
            .onChange(of: DebugHarness.shared.commandID) { _, _ in
                switch DebugHarness.shared.command {
                case "auth-signin": authSheet.present(tab: .signIn)
                case "auth-signup": authSheet.present(tab: .signUp)
                case "auth-close": authSheet.isPresented = false
                default: break
                }
            }
            #endif
        }
    }
}

struct SessionRoot: View {
    @Environment(\.scenePhase) private var scenePhase
    @Environment(AudioPlayer.self) private var audio
    let session: YapSession
    let auth: AuthStore
    var body: some View {
        Group {
            Group {
                if session.choosingCourse {
                    Color.clear
                } else if session.onboardingCourse != nil {
                    NavigationStack { CoursePickerView(session: session) }
                } else {
                    switch session.deckLoadView.phase {
                    case let .Loading(message, percent):
                        VStack(spacing: 20) { ProgressView(value: Double(percent) / 100); Text(message) }.padding(32)
                    case .NoLanguageSelected: NavigationStack { CoursePickerView(session: session) }
                    case let .Error(_, _, title, message, retryLabel):
                        ContentUnavailableView {
                            Label(title, systemImage: "exclamationmark.triangle")
                        } description: { Text(message) } actions: { Button(retryLabel) { session.retry() } }
                    case .Ready:
                        CourseHome(deck: session.deck!, session: session, auth: auth, startingFresh: session.startingFresh, historyKnown: session.historyKnown)
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
        .onDisappear { audio.stopAll() }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            switch DebugHarness.shared.command {
            case "switch-course": session.choosingCourse = true
            case "force-display-name": auth.needsDisplayName = true; UserDefaults.standard.removeObject(forKey: "yap-skipped-set-display-name")
            case "sync": if DebugHarness.shared.activeScreen != .settings { session.syncSoon() }
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
