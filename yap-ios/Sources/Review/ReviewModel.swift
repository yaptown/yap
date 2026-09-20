import Foundation
import Observation

enum ReviewStep: String, Hashable {
    case placementTest = "Placement test"
    case lockupOffer = "Review plan"
    case setDisplayName = "Set display name"
    case accomplishment = "Today's accomplishment"
}

@Observable @MainActor final class ReviewModel {
    let deck: Deck
    let session: YapSession
    let startingFresh: Bool?
    var historyKnown: Bool
    private(set) var banned: [ChallengeRequirements] = []
    private(set) var reviewInfo: ReviewInfo
    private(set) var lockupOffer: LockupOffer?
    private(set) var currentChallenge: Challenge_Gram_String?
    private(set) var submitting = false
    private var tasks: [Task<Void, Never>] = []
    private var prefetch: Task<Void, Never>?
    private var dueTask: Task<Void, Never>?
    private var active = false

    init(deck: Deck, session: YapSession, startingFresh: Bool?, historyKnown: Bool) {
        self.deck = deck; self.session = session; self.startingFresh = startingFresh; self.historyKnown = historyKnown
        reviewInfo = deck.get_review_info(banned_challenge_types: [], timestamp_ms: Self.now)
    }
    static var now: Double { Date().timeIntervalSince1970 * 1000 }
    func start() {
        guard !active else { return }; active = true
        refreshRestrictions(); refresh()
        tasks.append(Task { [weak self] in
            var audioVersion = get_audio_cache_version()
            var clipVersion = get_clip_manifest_version()
            var ticks = 0
            while !Task.isCancelled {
                do { try await Task.sleep(for: .seconds(2)) } catch { return }
                guard let self else { return }
                ticks += 1
                let audio = get_audio_cache_version(), clip = get_clip_manifest_version()
                if self.currentChallenge == nil { self.refreshRestrictions() }
                if audio != audioVersion || clip != clipVersion || ticks % 30 == 0 { self.refresh() }
                audioVersion = audio; clipVersion = clip
            }
        })
        tasks.append(Task { [weak self] in
            guard let self, self.session.online else { return }
            do { try await refresh_clip_manifest(language: self.deck.get_target_language(), access_token: self.session.accessToken()) }
            catch { if !Task.isCancelled { print("Yap manifest: \(error)") } }
        })
        if session.online, let token = session.accessToken() {
            tasks.append(Task { [deck, session] in
                do { try await deck.submit_push_notifications(access_token: token, user_id: session.userId) }
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
    private func refreshRestrictions() {
        let next = restrictions().banned
        if next != banned { banned = next; currentChallenge = nil; refresh() }
    }
    func refresh() {
        guard active else { return }
        let now = Self.now
        reviewInfo = deck.get_review_info(banned_challenge_types: banned, timestamp_ms: now)
        lockupOffer = deck.get_lockup_offer(banned_challenge_types: banned, timestamp_ms: now)
        // Non-nil challenges are held for this Deck's lifetime, even as caches change.
        #if DEBUG
        if let fixture = DebugHarness.shared.fixture { currentChallenge = fixture.challenge }
        #endif
        if currentChallenge == nil { currentChallenge = reviewInfo.get_next_challenge(deck: deck) }
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
    func completeTranslationPerfect(_ sentence: String, tapped: [Heteronym_String], completedAtMs: Double) -> Bool {
        completeSentence(deck.translate_sentence_perfect(words_tapped: tapped, challenge_sentence: sentence), at: completedAtMs)
    }
    @discardableResult
    func completeTranslationWrong(_ sentence: String, submission: String, grade: ManualTranslationGrade,
                                  tapped: [Heteronym_String], completedAtMs: Double) -> Bool {
        completeSentence(deck.translate_sentence_wrong(challenge_sentence: sentence, submission: submission,
            literal_grades: LiteralGrades(value: grade.literal_grades), words_tapped: tapped,
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
    func stop() {
        active = false; tasks.forEach { $0.cancel() }; tasks = []
        prefetch?.cancel(); dueTask?.cancel()
    }
    isolated deinit { stop() }
}
