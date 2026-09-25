import Foundation
import Observation

@Observable @MainActor final class ReviewModel {
    let deck: Deck
    let session: YapSession
    let auth: AuthStore
    let startingFresh: Bool?
    var historyKnown: Bool { didSet { refresh() } }
    private(set) var banned: [ChallengeRequirements] = []
    private(set) var view: ReviewScreenView?
    private(set) var currentChallenge: Challenge_Gram_String?
    private(set) var submitting = false
    private var tasks: [Task<Void, Never>] = []
    private var prefetch: Task<Void, Never>?
    private var dueTask: Task<Void, Never>?
    private var active = false

    init(deck: Deck, session: YapSession, auth: AuthStore, startingFresh: Bool?, historyKnown: Bool) {
        self.deck = deck; self.session = session; self.auth = auth; self.startingFresh = startingFresh; self.historyKnown = historyKnown
    }
    static var now: Double { Date().timeIntervalSince1970 * 1000 }
    #if DEBUG
    /// Driver hook: pose a translation challenge even when none is due, so
    /// fixtures can be captured from the test deck.
    func forceTranslation() { currentChallenge = deck.any_translation_challenge(); refresh() }
    #endif
    func start() {
        #if DEBUG
        guard DebugHarness.shared.fixture == nil else { return }
        #endif
        guard !active else { return }; active = true
        let next = restrictions().banned
        if next != banned { currentChallenge = nil }
        banned = next; refresh()
        tasks.append(Task { [weak self] in
            var audioVersion = get_audio_cache_version()
            var clipVersion = get_clip_manifest_version()
            var ticks = 0
            while !Task.isCancelled {
                do { try await Task.sleep(for: .seconds(2)) } catch { return }
                guard let self else { return }
                ticks += 1
                let audio = get_audio_cache_version(), clip = get_clip_manifest_version()
                self.refreshRestrictions(preservingChallenge: true)
                if audio != audioVersion || clip != clipVersion || ticks % 30 == 0 { self.refresh() }
                audioVersion = audio; clipVersion = clip
            }
        })
        tasks.append(Task { [weak self] in
            guard let self, self.session.online else { return }
            do { try await refresh_clip_manifest(language: self.deck.get_target_language(), access_token: self.session.accessToken()) }
            catch { if !Task.isCancelled { print("Yap manifest: \(error)") } }
        })
        if session.online, let userId = session.userId, let token = session.accessToken() {
            tasks.append(Task { [deck] in
                do { try await deck.submit_push_notifications(access_token: token, user_id: userId) }
                catch { if !Task.isCancelled { print("Yap notification schedule: \(error)") } }
            })
            tasks.append(Task { [deck] in
                do { try await deck.submit_language_stats(access_token: token) }
                catch { if !Task.isCancelled { print("Yap language stats: \(error)") } }
            })
        }
    }
    private func restrictions() -> ChallengeRestrictions {
        let defaults = UserDefaults.standard
        return get_challenge_restrictions(
            listening_since: defaults.object(forKey: "yap-cant-listen-timestamp") as? Double,
            speaking_since: defaults.object(forKey: "yap-cant-speak-timestamp") as? Double,
            now_ms: Self.now)
    }
    private func refreshRestrictions(preservingChallenge: Bool = false) {
        let next = restrictions().banned
        if next != banned {
            banned = next
            // Expiry updates the hub's inputs without discarding an in-progress review.
            if !preservingChallenge { currentChallenge = nil }
            refresh()
        }
    }
    func refresh() {
        #if DEBUG
        guard DebugHarness.shared.fixture == nil else { return }
        #endif
        guard active else { return }
        let now = Self.now
        let view = deck.review_screen_view(inputs: inputs(sentenceList: session.curriculumDraft.map(\.selection) ?? deck.get_sentence_list()))
        self.view = view
        // Non-nil challenges are held for this Deck's lifetime, even as caches change.
        if case let .Challenge(challenge) = view.step { currentChallenge = challenge.challenge }
        prefetch?.cancel()
        prefetch = Task { [deck, banned, session] in
            guard session.online else { return }
            await deck.cache_challenge_audio(banned_challenge_types: banned, access_token: session.accessToken())
        }
        dueTask?.cancel()
        if let due = deck.get_all_cards_summary().map(\.due_timestamp_ms).filter({ $0 > now }).min(), due - now < 86_400_000 {
            dueTask = Task { [weak self] in
                do { try await Task.sleep(for: .milliseconds(due - now + 1)) } catch { return }
                self?.refresh()
            }
        }
    }
    func cantListen() { setRestriction("yap-cant-listen-timestamp") }
    func cantSpeak() { setRestriction("yap-cant-speak-timestamp") }
    private func setRestriction(_ key: String) { UserDefaults.standard.set(Self.now, forKey: key); refreshRestrictions() }
    func undoRestrictions() {
        for key in ["yap-cant-listen-timestamp", "yap-cant-speak-timestamp"] { UserDefaults.standard.removeObject(forKey: key) }
        refreshRestrictions()
    }
    func rate(_ indicator: CardIndicator_Gram_String_String, _ rating: Rating) {
        guard active, !submitting, let event = deck.review_card(reviewed: indicator, rating: rating) else { return }
        submitting = true
        session.addDeckEvent(event)
        print("Yap review appended: events=\(session.weapon?.num_events ?? 0)")
    }
    var course: Course? {
        if case let .languageSelected(course, _, _, _) = session.deckSelection { return course }
        return nil
    }
    @discardableResult
    func completeTranslationPerfect(_ sentence: String, tapped: [UInt64], completedAtMs: Double) -> Bool {
        completeSentence(deck.translate_sentence_perfect(hinted_literal_indices: tapped, challenge_sentence: sentence), at: completedAtMs)
    }
    @discardableResult
    func completeTranslationWrong(_ sentence: String, submission: String, grade: ManualTranslationGrade,
                                  tapped: [UInt64], completedAtMs: Double) -> Bool {
        completeSentence(deck.translate_sentence_wrong(challenge_sentence: sentence, submission: submission,
            literal_grades: LiteralGrades(value: grade.literal_grades), hinted_literal_indices: tapped,
            phrases_remembered: grade.phrases_remembered, phrases_forgot: grade.phrases_forgot), at: completedAtMs)
    }
    @discardableResult
    func completeTranscription(_ parts: [PartGraded], completedAtMs: Double) -> Bool {
        completeSentence(deck.transcribe_sentence(challenge: parts), at: completedAtMs)
    }
    private func completeSentence(_ event: DeckEvent?, at timestamp: Double) -> Bool {
        guard active, !submitting, let event else { return false }
        submitting = true
        session.addDeckEventAt(event, timestampMs: timestamp)
        #if DEBUG
        DebugHarness.log("sentence appended: events=\(session.weapon?.num_events ?? 0) completedAtMs=\(timestamp)")
        #endif
        return true
    }
    func inputs(sentenceList: SentenceListSelection?) -> ReviewScreenInputs {
        ReviewScreenInputs(banned: banned, sentence_list: sentenceList, online: session.online,
            is_signed_in: auth.userId != nil, needs_display_name: auth.needsDisplayName,
            display_name_dismissed: UserDefaults.standard.bool(forKey: "yap-skipped-set-display-name"),
            has_access_token: auth.accessToken != nil, starting_fresh: startingFresh, history_known: historyKnown,
            dismissed_accomplishment_at_review: session.dismissedAccomplishmentAtReview, placement: session.placementSession, current_challenge: currentChallenge, timestamp_ms: Self.now)
    }
    var host: ReviewHost {
        ReviewHost(deck: deck, weapon: session.weapon, online: session.online, accessToken: session.accessToken(), autoplay: session.autoplay,
            packBanner: session.deckLoadView.pack_banner, syncError: session.syncError, authError: auth.error)
    }
    var actions: ReviewActions {
        #if DEBUG
        if DebugHarness.shared.fixture != nil { return .inert }
        #endif
        var actions = ReviewActions()
        actions.submitting = submitting
        actions.pendingReviewKey = "\(session.userId ?? "anon")-\(String(describing: course))"
        actions.rate = rate
        actions.completeTranslationPerfect = { self.completeTranslationPerfect($0, tapped: $1, completedAtMs: $2) }
        actions.completeTranslationWrong = { self.completeTranslationWrong($0, submission: $1, grade: $2, tapped: $3, completedAtMs: $4) }
        actions.completeTranscription = { self.completeTranscription($0, completedAtMs: $1) }
        actions.cantListen = cantListen; actions.cantSpeak = cantSpeak; actions.undoRestrictions = undoRestrictions
        actions.addEvent = session.addDeckEvent
        actions.setSentenceList = { self.session.curriculumDraft = CurriculumDraft(selection: $0); self.refresh() }
        actions.commitSentenceList = { self.session.addDeckEvent($0); self.session.curriculumDraft = nil; self.refresh() }
        actions.dismissAccomplishment = { self.session.dismissedAccomplishmentAtReview = self.view?.total_reviews; self.refresh() }
        actions.setPlacement = { self.session.placementSession = $0; self.refresh() }
        actions.completePlacementTest = { self.session.addDeckEvent(self.deck.complete_placement_test(known_words: $0.known_words, unknown_words: $0.unknown_words)) }
        actions.retryPack = session.retry
        actions.switchCourse = { self.session.choosingCourse = true }
        actions.skipDisplayName = { UserDefaults.standard.set(true, forKey: "yap-skipped-set-display-name"); self.refresh() }
        actions.saveDisplayName = { name in
            guard let token = self.auth.accessToken else { return }
            _ = try await update_profile(display_name: name, bio: nil, access_token: token)
            self.auth.displayName = name; self.auth.needsDisplayName = false; self.refresh()
        }
        return actions
    }
    func stop() {
        active = false; tasks.forEach { $0.cancel() }; tasks = []
        prefetch?.cancel(); dueTask?.cancel()
    }
    #if DEBUG
    func handleDebugCommand() {
        switch DebugHarness.shared.command {
        case "dismiss-keyboard": break
        case "status":
            DebugHarness.log("placement: startingFresh=\(String(describing: startingFresh)) historyKnown=\(historyKnown) taken=\(deck.has_taken_placement_test()) list=\(String(describing: deck.get_sentence_list()))")
            DebugHarness.log("today: seconds=\(deck.get_today_time_spent()) target=\(deck.get_daily_review_target())")
            let info = deck.get_review_info(banned_challenge_types: banned, timestamp_ms: Self.now)
            DebugHarness.log("review: due=\(info.due_count) total=\(info.total_count) banned=\(info.due_but_banned_count) audioPending=\(info.due_but_audio_pending_count) locked=\(info.due_but_locked_count) step=\(String(describing: view?.step))")
            switch currentChallenge {
            case let .PronunciationChallenge(_, pattern, _, _, _, _): DebugHarness.log("challenge=pronunciation \(pattern)")
            case let .TranslateComprehensibleSentence(sentence): DebugHarness.log("challenge=translation \(sentence.target_language)")
            case let .TranscribeComprehensibleSentence(sentence): DebugHarness.log("challenge=transcription \(sentence.target_language)")
            case let .FlashCardReview(indicator, _, _, _): DebugHarness.log("challenge=flashcard \(String(describing: indicator))")
            case nil: DebugHarness.log("challenge=none")
            }
        case let command where command.hasPrefix("goal "):
            let options = get_daily_goal_options()
            if let i = Int(command.dropFirst(5)), options.indices.contains(i) {
                session.addDeckEvent(deck.set_daily_review_target(daily_review_target: options[i].value))
            }
        case "cant-listen": cantListen()
        case "dismiss-step": actions.dismissAccomplishment()
        case "force-translation": forceTranslation()
        case "cant-speak": cantSpeak()
        case "undo": undoRestrictions()
        case "add-listening", "add-pronunciation":
            let type: CardType = DebugHarness.shared.command == "add-listening" ? .Listening : .LetterPronunciation
            let options = deck.get_manual_add_options(sentence_list: deck.get_sentence_list(), is_signed_in: auth.userId != nil)
            if let event = options.first(where: { $0.card_type == type })?.event { session.addDeckEvent(event) }
        case "add":
            guard currentChallenge == nil else { return }
            if case let .Idle(.Idle(idle)) = view?.step, let event = idle.info.smart_add_event { session.addDeckEvent(event) }
        default: break
        }
    }
    #endif
    isolated deinit { stop() }
}
