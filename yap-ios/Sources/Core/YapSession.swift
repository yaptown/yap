import Foundation
import Network
import Observation

enum DeckSelectionState {
    case loading
    case noLanguageSelected(onboardedLanguages: [Language], hasHeardAbout: Bool)
    case languageSelected(course: Course, startingFresh: Bool?, onboardedLanguages: [Language], hasHeardAbout: Bool)
}

enum DeckState {
    case loading(message: String, progress: Double)
    case noLanguageSelected
    case deck(Deck, course: Course, startingFresh: Bool?, historyKnown: Bool)
    case error(String)
}

@Observable @MainActor final class YapSession {
    let userId: String
    let accessToken: () -> String?
    private(set) var weapon: Weapon?
    private(set) var deckSelection: DeckSelectionState = .loading
    private(set) var deckState: DeckState = .loading(message: "Opening your deck…", progress: 0)
    private(set) var online = true
    var syncError: String?
    /// The full pack failed to download after the core was already usable.
    var packError: String?
    let autoplay = AutoplayClaim()
    // These flows must survive immutable Deck snapshot replacements.
    var placementSession: PlacementSession?
    var choosingCourse = false
    var onboardingCourse: Course?
    var onboardingHasHeardAbout = false
    var dismissedAccomplishmentAtReview: UInt64?
    private var active = false
    private var listeners: [ListenerKey] = []
    private var tasks: [UUID: Task<Void, Never>] = [:]
    private var timer: Task<Void, Never>?
    private var packTask: Task<Void, Never>?
    private var snapshotTask: Task<Void, Never>?
    private var monitor: NWPathMonitor?
    private var course: Course?
    private var coreReady = false
    private var builtDeckInputs: String?
    private var generation = 0
    private var snapshotGeneration = 0

    init(userId: String, accessToken: @escaping () -> String?) {
        self.userId = userId; self.accessToken = accessToken
    }

    func start() async {
        guard !active else { return }
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
                loadPack(cached)
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
        } catch { if active { deckState = .error(String(describing: error)) } }
    }

    private static func courseKey(_ course: Course) -> String {
        "\(course.target_language):\(course.native_language)"
    }

    /// Rust owns the Deck inputs: rebuild only when its key changes, so a
    /// foreground or no-op sync cannot replace the learner's current challenge.
    private func recompute() {
        guard active, let weapon,
              weapon.get_stream_num_events(stream_id: "reviews") != nil,
              weapon.get_stream_num_events(stream_id: "deck_selection") != nil else { return }
        let selection = weapon.get_deck_selection_state()
        let onboarded = selection?.onboarded_languages ?? []
        let heard = selection?.heard_about != nil
        guard let native = selection?.native_language, let target = selection?.target_language else {
            deckSelection = .noLanguageSelected(onboardedLanguages: onboarded, hasHeardAbout: heard)
            deckState = .noLanguageSelected
            snapshotGeneration += 1; snapshotTask?.cancel()
            return
        }
        let selected = Course(native_language: native, target_language: target)
        deckSelection = .languageSelected(course: selected, startingFresh: selection?.onboarding_selections?.starting_fresh,
                                          onboardedLanguages: onboarded, hasHeardAbout: heard)
        UserDefaults.standard.set(Self.courseKey(selected), forKey: "yap-last-course")
        if selected != course { loadPack(selected) }
        guard coreReady else { return }
        let inputs = weapon.deck_inputs_key(course: selected)
        if inputs != builtDeckInputs { rebuildDeck(inputs: inputs) }
    }

    private func loadPack(_ selected: Course) {
        guard let weapon else { return }
        Telemetry.breadcrumb("language-pack", "Loading language pack: \(selected.target_language) → \(selected.native_language)")
        generation += 1
        let expected = generation
        packTask?.cancel(); snapshotTask?.cancel(); snapshotGeneration += 1
        if course != selected { placementSession = nil; dismissedAccomplishmentAtReview = nil }
        course = selected; coreReady = false; builtDeckInputs = nil; packError = nil
        deckState = .loading(message: "Downloading language pack…", progress: 0)
        packTask = Task { [weak self] in
            let progress: @MainActor (String, Float) -> Void = { [weak self] message, percent in
                guard let self, self.active, self.generation == expected else { return }
                Telemetry.breadcrumb("language-pack", "\(message) (\(Int(percent.rounded()))%)")
                if case .deck = self.deckState { return }
                if case .noLanguageSelected = self.deckSelection { return }
                self.deckState = .loading(message: message, progress: Double(percent) / 100)
            }
            do {
                try await weapon.load_language_pack_core(course: selected, on_progress: progress)
                guard let self, self.active, self.generation == expected, !Task.isCancelled else { return }
                self.coreReady = true; self.recompute()
                for attempt in 0...5 {
                    do { try await weapon.load_language_pack(course: selected, on_progress: progress); break }
                    catch {
                        guard self.active, self.generation == expected, !Task.isCancelled else { return }
                        if attempt == 5 { throw error }
                        try await Task.sleep(for: .seconds(min(30, 2 << attempt)))
                    }
                }
                guard self.active, self.generation == expected, !Task.isCancelled else { return }
                self.recompute()
                Telemetry.breadcrumb("language-pack", "Full language pack loaded")
                print("Yap: full language pack loaded")
            } catch {
                guard let self, self.active, self.generation == expected, !Task.isCancelled else { return }
                Telemetry.breadcrumb("language-pack", "Loading failed: \(error)", failed: true)
                // A deck already on screen stays usable; only the upgrade is reported.
                self.packError = String(describing: error)
                if case .deck = self.deckState {} else { self.deckState = .error(String(describing: error)) }
            }
        }
    }

    private func rebuildDeck(inputs: String) {
        guard let weapon, case let .languageSelected(selected, startingFresh, _, _) = deckSelection,
              selected == course else { return }
        builtDeckInputs = inputs
        snapshotGeneration += 1
        let expected = snapshotGeneration
        snapshotTask?.cancel()
        snapshotTask = Task { [weak self] in
            do {
                let deck = try await weapon.get_deck_state(course: selected, utc_offset_seconds: Int32(TimeZone.current.secondsFromGMT()))
                guard let self, self.active, self.snapshotGeneration == expected, !Task.isCancelled else { return }
                // nil while only the core half of the pack is loaded and this
                // learner would not see the placement test; the loading screen
                // stays up and the full pack triggers another rebuild.
                guard let deck else {
                    if case .deck = self.deckState {
                        // The sentence download is either still running or has already failed.
                        self.deckState = self.packError.map { .error($0) } ?? .loading(message: "Downloading sentences…", progress: 0)
                    }
                    return
                }
                self.deckState = .deck(deck, course: selected, startingFresh: startingFresh, historyKnown: weapon.reviews_history_known())
            } catch {
                guard let self, self.active, self.snapshotGeneration == expected, !Task.isCancelled else { return }
                self.deckState = .error(String(describing: error))
            }
        }
    }
    func retry() { if let course { loadPack(course) } }
    func sceneBecameActive() { recompute(); syncSoon() }
    func tokenChanged() { syncSoon() }
    func syncSoon() { run { [weak self] in await self?.syncWithSupabase() } }
    func syncWithSupabase(forceUpload: Bool = true) async {
        guard active, online, let token = accessToken(), let weapon else { return }
        do {
            try await weapon.sync_with_supabase(access_token: token, modifier: nil, upload: forceUpload)
            guard active else { return }
            syncError = nil
            print("Yap sync: events=\(weapon.num_events) remote=\(weapon.num_events_on_remote_as_of_last_sync(target: .Supabase)) finished=\(weapon.get_sync_state(target: .Supabase).last_sync_finished != nil)")
        } catch { logSync(error) }
    }
    private func logSync(_ error: Error) {
        guard active, !Task.isCancelled else { return }
        Telemetry.breadcrumb("sync", "Sync failed: \(error)", failed: true)
        syncError = String(describing: error); print("Yap sync failed: \(error)")
    }
    private func run(_ operation: @escaping @MainActor () async -> Void) {
        let id = UUID()
        tasks[id] = Task { [weak self] in await operation(); self?.tasks[id] = nil }
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
    func stop() {
        active = false; generation += 1; snapshotGeneration += 1
        monitor?.cancel(); monitor = nil
        timer?.cancel(); packTask?.cancel(); snapshotTask?.cancel()
        tasks.values.forEach { $0.cancel() }; tasks.removeAll()
        if let weapon { for key in listeners { weapon.unsubscribe(key: key) } }
        listeners.removeAll(); weapon = nil
    }
    isolated deinit { stop() }
}
