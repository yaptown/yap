import Foundation
import Network
import Observation

enum DeckSelectionState {
    case loading
    case noLanguageSelected(onboardedLanguages: [Language], hasHeardAbout: Bool)
    case languageSelected(course: Course, startingFresh: Bool?, onboardedLanguages: [Language], hasHeardAbout: Bool)
}

/// A curriculum the learner has browsed to but not committed (YAP-85). Wrapped
/// so that "Essential" (`selection == nil`) is distinguishable from "no draft".
struct CurriculumDraft: Equatable { let selection: SentenceListSelection? }

@Observable @MainActor final class YapSession {
    let userId: String?
    let accessToken: () -> String?
    private(set) var weapon: Weapon?
    private(set) var deckSelection: DeckSelectionState = .loading
    private(set) var deckLoad = deck_load_start()
    private(set) var deck: Deck?
    var deckLoadView: DeckLoadView { deck_load_view(state: deckLoad) }
    private(set) var startingFresh: Bool?
    private(set) var historyKnown = false
    private(set) var online = true
    var syncError: String?
    let autoplay = AutoplayClaim()
    // These flows must survive immutable Deck snapshot replacements.
    var placementSession: PlacementSession?
    /// Browsed-but-uncommitted curriculum; lives here so deck snapshot
    /// replacements keep it, and a course change drops it.
    var curriculumDraft: CurriculumDraft?
    var choosingCourse = false
    var onboardingCourse: Course?
    var onboardingHasHeardAbout = false
    var dismissedAccomplishmentAtReview: UInt64?
    private var active = false
    private var startTask: Task<Void, Never>?
    private var stopTask: Task<Void, Error>?
    private var listeners: [ListenerKey] = []
    private var tasks: [UUID: Task<Void, Never>] = [:]
    private var timer: Task<Void, Never>?
    private var packTask: Task<Void, Never>?
    private var snapshotTask: Task<Void, Never>?
    private var monitor: NWPathMonitor?
    private(set) var course: Course?
    private var pendingInputs: String?
    private var generation = 0
    private var snapshotGeneration = 0

    init(userId: String?, accessToken: @escaping () -> String?) {
        self.userId = userId; self.accessToken = accessToken
    }

    func start() async {
        guard stopTask == nil else { return }
        if startTask == nil { startTask = Task { await load() } }
        await startTask?.value
    }

    private func load() async {
        guard !active, stopTask == nil else { return }
        active = true
        let monitor = NWPathMonitor()
        self.monitor = monitor
        monitor.pathUpdateHandler = { [weak self] path in
            let connected = path.status == .satisfied
            Task { @MainActor [weak self] in
                guard let self, self.active else { return }
                let wasOffline = !self.online
                self.online = connected
                if connected && wasOffline { self.syncSoon() }
            }
        }
        #if DEBUG
        if CommandLine.arguments.contains("--test-offline") {
            online = false
        } else { monitor.start(queue: DispatchQueue(label: "town.yap.network")) }
        #else
        monitor.start(queue: DispatchQueue(label: "town.yap.network"))
        #endif
        do {
            let weapon = try await Weapon.create(user_id: userId) { [weak self] key, stream in
                guard let self, self.active else { return }
                self.run { [weak self] in
                    guard let self, let weapon = self.weapon, self.active else { return }
                    do {
                        try await weapon.sync(stream_id: stream, access_token: self.accessToken(),
                            attempt_supabase: self.online, modifier: key, upload: true)
                    } catch { self.logSync(error) }
                }
            }
            guard active, !Task.isCancelled else { return }
            self.weapon = weapon
            // Start cached-course downloads before reading either event stream.
            if let key = UserDefaults.standard.string(forKey: "yap-last-course"),
               let cached = get_available_courses().first(where: { Self.courseKey($0) == key }) {
                course = cached
                dispatch(.CourseChanged(key: Self.courseKey(cached)))
            }
            for stream in ["reviews", "deck_selection"] {
                listeners.append(weapon.subscribe_to_stream(stream_id: stream) { [weak self] in self?.recompute() })
            }
            weapon.request_reviews(); weapon.request_deck_selection()
            for stream in ["reviews", "deck_selection"] { try await weapon.load_from_local_storage(stream_id: stream) }
            guard active else { return }
            recompute()
            syncSoon()
            timer = Task { [weak self] in
                while !Task.isCancelled {
                    do { try await Task.sleep(for: .seconds(30)) } catch { return }
                    await self?.syncWithSupabase()
                }
            }
        } catch { if active { dispatch(.DeckBuildFailed(message: String(describing: error))) } }
    }

    private static func courseKey(_ course: Course) -> String {
        "\(course.target_language):\(course.native_language)"
    }

    /// Streams and pack callbacks feed facts; Rust alone decides whether to rebuild.
    private func recompute() {
        guard active, let weapon,
              weapon.get_stream_num_events(stream_id: "reviews") != nil,
              weapon.get_stream_num_events(stream_id: "deck_selection") != nil else { return }
        let selection = weapon.get_deck_selection_state()
        let onboarded = selection?.onboarded_languages ?? []
        let heard = selection?.heard_about != nil
        startingFresh = selection?.onboarding_selections?.starting_fresh
        historyKnown = weapon.reviews_history_known()
        guard let native = selection?.native_language, let target = selection?.target_language else {
            deckSelection = .noLanguageSelected(onboardedLanguages: onboarded, hasHeardAbout: heard)
            course = nil
            dispatch(.CourseChanged(key: nil))
            dispatch(.StreamsReady(language_selected: false))
            return
        }
        let selected = Course(native_language: native, target_language: target)
        deckSelection = .languageSelected(course: selected, startingFresh: startingFresh,
                                          onboardedLanguages: onboarded, hasHeardAbout: heard)
        UserDefaults.standard.set(Self.courseKey(selected), forKey: "yap-last-course")
        course = selected
        dispatch(.CourseChanged(key: Self.courseKey(selected)))
        dispatch(.StreamsReady(language_selected: true))
        // A cached pack can arrive before StreamsReady, so always refresh after it.
        dispatch(.InputsChanged(key: weapon.deck_inputs_key(course: selected)))
    }

    private func dispatch(_ event: DeckLoadEvent) {
        guard active else { return }
        if case let .CourseChanged(key) = event, key != deckLoad.course_key {
            generation += 1; packTask?.cancel()
            placementSession = nil; dismissedAccomplishmentAtReview = nil; curriculumDraft = nil
        }
        // Reject a pending B even if the inputs revert to already-built A and
        // Rust (correctly) emits no new BuildDeck effect for A.
        if case let .InputsChanged(key) = event, key != pendingInputs {
            snapshotGeneration += 1; snapshotTask?.cancel(); pendingInputs = nil
        }
        let step = deck_load_transition(state: deckLoad, event: event)
        deckLoad = step.state
        for effect in step.effects {
            switch effect {
            case .ClearDeck:
                snapshotGeneration += 1; snapshotTask?.cancel(); pendingInputs = nil
                deck = nil
            case let .LoadPack(key, from):
                if let course, Self.courseKey(course) == key { loadPack(course, from: from) }
            case .RefreshInputs:
                if let weapon, let course { dispatch(.InputsChanged(key: weapon.deck_inputs_key(course: course))) }
            case let .BuildDeck(inputs): rebuildDeck(inputs: inputs)
            case let .ReportError(phase, message, network):
                if !network { Telemetry.breadcrumb("language-pack", "\(phase) failed: \(message)", failed: true) }
            }
        }
    }

    private func loadPack(_ selected: Course, from: PackStage) {
        guard let weapon else { return }
        Telemetry.breadcrumb("language-pack", "Loading language pack: \(selected.target_language) → \(selected.native_language)")
        generation += 1
        let expected = generation
        packTask?.cancel()
        packTask = Task { [weak self] in
            let progress: @MainActor (String, Float) -> Void = { [weak self] message, percent in
                guard let self, self.active, self.generation == expected else { return }
                Telemetry.breadcrumb("language-pack", "\(message) (\(Int(percent.rounded()))%)")
                self.dispatch(.PackProgress(message: message, percent: percent))
            }
            do {
                if from == .None {
                    try await weapon.load_language_pack_core(course: selected, on_progress: progress)
                    guard let self, self.active, self.generation == expected, !Task.isCancelled else { return }
                    self.dispatch(.CoreLoaded)
                }
                try await weapon.load_language_pack(course: selected, on_progress: progress)
                guard let self, self.active, self.generation == expected, !Task.isCancelled else { return }
                self.dispatch(.FullLoaded)
                Telemetry.breadcrumb("language-pack", "Full language pack loaded")
            } catch {
                guard let self, self.active, self.generation == expected, !Task.isCancelled else { return }
                let network: Bool
                switch error as? LanguageDataError {
                case .Download, .Timeout: network = true
                default: network = false
                }
                self.dispatch(.PackFailed(message: String(describing: error), network: network))
            }
        }
    }

    private func rebuildDeck(inputs: String) {
        guard let weapon, let selected = course, pendingInputs != inputs else { return }
        pendingInputs = inputs
        snapshotGeneration += 1
        let expected = snapshotGeneration
        snapshotTask?.cancel()
        snapshotTask = Task { [weak self] in
            do {
                let deck = try await weapon.get_deck_state(course: selected, utc_offset_seconds: Int32(TimeZone.current.secondsFromGMT()))
                guard let self, self.active, self.snapshotGeneration == expected, !Task.isCancelled,
                      weapon.deck_inputs_key(course: selected) == inputs else { return }
                self.pendingInputs = nil
                self.deck = deck
                self.dispatch(.DeckBuilt(present: deck != nil, inputs: inputs))
            } catch {
                guard let self, self.active, self.snapshotGeneration == expected, !Task.isCancelled,
                      weapon.deck_inputs_key(course: selected) == inputs else { return }
                self.pendingInputs = nil
                self.dispatch(.DeckBuildFailed(message: String(describing: error)))
            }
        }
    }
    func retry() { dispatch(.Retry) }
    func sceneBecameActive() { recompute(); syncSoon() }
    func tokenChanged() { syncSoon() }
    func syncSoon() { beginSync() }
    func syncWithSupabase(forceUpload: Bool = true) async {
        await beginSync(forceUpload: forceUpload)?.value
    }
    @discardableResult private func beginSync(forceUpload: Bool = true) -> Task<Void, Never>? {
        guard active, online, let token = accessToken(), let weapon else { return nil }
        return run { [self] in
            do {
                try await weapon.sync_with_supabase(access_token: token, modifier: nil, upload: forceUpload)
                guard active else { return }
                syncError = nil
                print("Yap sync: events=\(weapon.num_events) remote=\(weapon.num_events_on_remote_as_of_last_sync(target: .Supabase)) finished=\(weapon.get_sync_state(target: .Supabase).last_sync_finished != nil)")
            } catch { logSync(error) }
        }
    }
    private func logSync(_ error: Error) {
        guard active, !Task.isCancelled else { return }
        Telemetry.breadcrumb("sync", "Sync failed: \(error)", failed: true)
        syncError = String(describing: error); print("Yap sync failed: \(error)")
    }
    @discardableResult private func run(_ operation: @escaping @MainActor () async -> Void) -> Task<Void, Never>? {
        guard active else { return nil }
        let id = UUID()
        let task = Task { [weak self] in await operation(); self?.tasks[id] = nil }
        tasks[id] = task
        return task
    }
    func addDeckEvent(_ event: DeckEvent) { guard active else { return }; weapon?.add_deck_event(event: event) }
    func addDeckEventAt(_ event: DeckEvent, timestampMs: Double) { guard active else { return }; weapon?.add_deck_event_at(event: event, timestamp_ms: timestampMs) }
    func addDeckSelectionEvent(_ event: DeckSelectionEvent) {
        guard active else { return }
        autoplay.reviewCount = nil
        weapon?.add_deck_selection_event(event: event)
    }
    func prefetchPack(_ course: Course) {
        Telemetry.breadcrumb("language-pack", "Caching \(course.native_language) → \(course.target_language)")
        run { [weak self] in
            guard let self, let weapon = self.weapon else { return }
            do { try await weapon.cache_language_pack(course: course) }
            catch { if !Task.isCancelled { print("Yap pack prefetch: \(error)") } }
        }
    }
    /// Freeze synchronously, then flush everything in memory to disk. The app
    /// shell awaits this task before constructing the next identity's Weapon.
    func stop() -> Task<Void, Error> {
        if let stopTask { return stopTask }
        active = false; generation += 1; snapshotGeneration += 1
        monitor?.cancel(); monitor = nil
        // Network syncs and pack prefetches are disposable: the final flush
        // below writes every event in memory, so cancelling them loses nothing
        // and a stalled request can't hold sign-in or sign-out hostage. Only
        // the start task is awaited, because it creates the Weapon and runs
        // the one-time import of anonymous data.
        timer?.cancel(); packTask?.cancel(); snapshotTask?.cancel()
        tasks.values.forEach { $0.cancel() }; tasks.removeAll()
        let starting = startTask
        if let weapon { for key in listeners { weapon.unsubscribe(key: key) } }
        listeners.removeAll()
        let task = Task {
            await starting?.value
            if let weapon {
                for stream in ["reviews", "deck_selection"] {
                    try await weapon.sync(stream_id: stream, access_token: nil,
                        attempt_supabase: false, modifier: nil, upload: false)
                }
            }
            weapon = nil; deck = nil; course = nil; pendingInputs = nil
            deckLoad = deck_load_start()
        }
        stopTask = task
        return task
    }
}

/// Lives outside identity-keyed SwiftUI content, so teardown cannot race import.
@Observable @MainActor final class SessionLifecycle {
    private(set) var session: YapSession?
    private(set) var error: String?
    private var transition: Task<Void, Never>?
    private var retiring: YapSession?
    private var generation = 0

    func changeIdentity(auth: AuthStore) {
        generation += 1
        let expected = generation
        let userId = auth.userId
        let previous = transition
        if let session { retiring = session }
        let stopped = retiring?.stop()
        session = nil
        transition = Task {
            await previous?.value
            // A failed flush must not release the old data and import an incomplete log.
            guard error == nil else { return }
            do {
                try await stopped?.value
                guard generation == expected else { return }
                retiring = nil
                session = YapSession(userId: userId, accessToken: { [weak auth] in
                    guard auth?.userId == userId else { return nil }
                    return auth?.accessToken
                })
            } catch { self.error = String(describing: error) }
        }
    }
}
