import Foundation

private func check(_ condition: Bool, file: StaticString = #file, line: UInt = #line) { precondition(condition, file: file, line: line) }
/// The deck once it is available: after the full pack loads, or while the core half alone would offer the placement test.
@MainActor private func deck(_ weapon: Weapon, _ course: Course, file: StaticString = #file, line: UInt = #line) async throws -> Deck {
    guard let deck = try await weapon.get_deck_state(course: course, utc_offset_seconds: 0) else { fatalError("expected a deck", file: file, line: line) }
    return deck
}

@main struct YapSmoke {
    @MainActor static func main() async throws {
        // Shared language copy crosses the real native bridge, including Unicode,
        // optional script qualifiers, and accent keyboard arrays.
        let french = get_language_metadata(language: .French)
        check(french.native_name == "Français" && french.flag == "🇫🇷")
        check(french.i_speak == "Je parle français" && french.lets_go == "Allons-y !")
        check(french.accented_characters.contains("œ") && french.script == nil)
        let simplified = get_language_metadata(language: .ChineseSimplified)
        let traditional = get_language_metadata(language: .ChineseTraditional)
        check(simplified.script == "Hans" && traditional.script == "Hant")
        check(simplified.iso_code != traditional.iso_code && simplified.iso6391 == traditional.iso6391)
        check(traditional.native_name == "繁體中文" && traditional.i_speak == "我說中文")
        check(get_language_metadata(language: .English).character_type == nil)
        try YapHost.initialize()
        do {
            try YapHost.initialize()
            fatalError("runtime replacement should be rejected")
        } catch let error as BridgeError {
            check(error.description.contains("already"))
        }
        let course = Course(native_language: .English, target_language: .French)
        var syncStreams: [String] = []
        var syncKeys: [ListenerKey] = []
        let weapon = try await Weapon.create(user_id: nil) { key, stream in
            check(Thread.isMainThread)
            syncKeys.append(key)
            syncStreams.append(stream)
        }
        let device = weapon.device_id
        check(!device.isEmpty)
        var notifications = 0
        let listener = weapon.subscribe_to_stream(stream_id: "reviews") { [weak weapon] in
            check(Thread.isMainThread)
            check(weapon!.device_id == device) // reenter the real Weapon
            notifications += 1
        }
        weapon.request_reviews()
        check(notifications > 0 && syncStreams.contains("reviews") && !syncKeys.isEmpty)
        let modifier = syncKeys.removeFirst()
        try await weapon.sync(stream_id: "reviews", access_token: nil, attempt_supabase: false, modifier: modifier, upload: false)
        syncKeys.removeAll()
        try await weapon.load_from_local_storage(stream_id: "reviews")
        let beforeInvalidEvents = weapon.num_events
        for (invalidEvent, message) in [("{", "EOF"), ("{}", "missing field")] {
            do {
                try weapon.add_remote_event(device_id: "invalid-fixture", stream_id: "reviews", event: invalidEvent)
                fatalError("expected a JSON error")
            } catch let error as BridgeError { check(error.description.contains(message)) }
        }
        check(weapon.num_events == beforeInvalidEvents)
        do {
            _ = try await weapon.get_deck_state(course: course, utc_offset_seconds: 0)
            fatalError("expected a missing-pack error")
        } catch let error as BridgeError {
            check(error.description.contains("not loaded"))
        }
        let unsupported = Course(native_language: .English, target_language: .English)
        do {
            try await weapon.load_language_pack(course: unsupported, on_progress: nil)
            fatalError("expected unsupported course")
        } catch LanguageDataError.UnsupportedCourse(let actual) { check(actual == unsupported) }
        let noPackInputs = weapon.deck_inputs_key(course: course)
        var progressCalls = 0
        try await weapon.load_language_pack_core(course: course) { message, percent in
            check(!message.isEmpty && percent >= 0 && percent <= 100)
            check(Thread.isMainThread)
            progressCalls += 1
        }
        check(!weapon.is_language_pack_fully_loaded(course: course) && progressCalls > 0)
        check(weapon.deck_inputs_key(course: course) != noPackInputs)
        // A core-only deck is withheld unless the learner would see the placement test.
        check(try await weapon.get_deck_state(course: course, utc_offset_seconds: 0) == nil)
        weapon.request_deck_selection()
        try weapon.add_deck_selection_event(event: .SelectBothLanguages(native: course.native_language, target: course.target_language))
        try weapon.add_deck_selection_event(event: .SetOnboardingSelections(
            selections: OnboardingSelections(starting_fresh: false, motivation: nil, experience_level: nil, study_goal: nil),
            target_language: course.target_language))
        let coreDeck = try await deck(weapon, course)
        var placement = coreDeck.start_placement_session()
        check(!placement.words.isEmpty)
        placement = toggle_placement_word(session: placement, word: placement.words[0].word)
        check(placement.selected_words.contains(placement.words[0].word))
        placement = coreDeck.advance_placement_session(session: placement)
        let knownBeforeFullPack = placement.known_words
        let unknownBeforeFullPack = placement.unknown_words
        let coreInputs = weapon.deck_inputs_key(course: course)
        try await weapon.load_language_pack(course: course, on_progress: nil)
        check(weapon.deck_inputs_key(course: course) != coreInputs)
        check(weapon.is_language_pack_fully_loaded(course: course))
        let original = try await deck(weapon, course)
        placement = original.refresh_placement_session(session: placement) ?? placement
        check(placement.known_words == knownBeforeFullPack && placement.unknown_words == unknownBeforeFullPack)
        for _ in 0..<20 {
            if get_placement_session_info(session: placement).finished { break }
            placement = original.advance_placement_session(session: placement)
        }
        check(get_placement_session_info(session: placement).finished)
        // A stale clip would be evicted by cleanup if cancellation incorrectly fell through.
        let audioDirectory = URL(fileURLWithPath: String(cString: getenv("YAP_DATA_DIR")!)).appendingPathComponent("audio")
        try FileManager.default.createDirectory(at: audioDirectory, withIntermediateDirectories: true)
        let sentinel = audioDirectory.appendingPathComponent("cancel-preserves.mp3")
        try Data([1, 2, 3]).write(to: sentinel)
        try Data("{\"cancel-preserves.mp3\":1}".utf8).write(to: audioDirectory.appendingPathComponent("last_used.json"))
        let aborted = AbortController()
        aborted.abort()
        await original.cache_challenge_audio(banned_challenge_types: [], access_token: nil, abort_signal: aborted.signal())
        let prefetch = Task { @MainActor in
            await original.cache_challenge_audio(banned_challenge_types: [.Text, .Listening, .Speaking], access_token: nil)
        }
        prefetch.cancel()
        await prefetch.value
        let background = Task { @MainActor in
            await original.cache_challenge_audio(banned_challenge_types: [.Text, .Listening, .Speaking], access_token: nil)
        }
        try await Task.sleep(for: .milliseconds(20))
        background.cancel()
        await background.value
        check(FileManager.default.fileExists(atPath: sentinel.path))
        let summary = original.get_today_summary()
        check(summary.reviews == 0 && !summary.day_of_week.isEmpty)
        check(original.get_daily_review_target() == 600)
        let root = URL(fileURLWithPath: CommandLine.arguments[1])
        let event = try String(contentsOf: root.appendingPathComponent("event-0.json"), encoding: .utf8)
        let before = notifications
        try weapon.add_remote_event(device_id: "swift-prototype-fixture", stream_id: "reviews", event: event)
        check(notifications == before + 1)
        let changed = try await deck(weapon, course)
        check(changed.get_daily_review_target() == 1200)
        check(original.get_daily_review_target() == 600) // independent Deck snapshots
        weapon.unsubscribe(key: listener)
        let after = notifications
        let secondEvent = try String(contentsOf: root.appendingPathComponent("event-1.json"), encoding: .utf8)
        try weapon.add_remote_event(device_id: "swift-prototype-fixture", stream_id: "reviews", event: secondEvent)
        check(notifications == after)
        let latest = try await deck(weapon, course)
        check(latest.get_daily_review_target() == 300)
        // The full Weapon impl is exported: Swift creates typed events directly.
        check(weapon.user_id == nil)
        check(weapon.num_events >= 2)
        let reviewCount = weapon.get_stream_num_events(stream_id: "reviews")!
        let goal = DeckEvent.Language(LanguageEvent(target_language: .French, native_language: .English,
            content: .SetDailyReviewTarget(daily_review_target: .Intense)))
        weapon.add_deck_event(event: goal)
        // Exercise an explicitly later timestamp through the same event API.
        let future = Date().timeIntervalSince1970 * 1000 + 10_000
        weapon.add_deck_event_at(event: goal, timestamp_ms: future)
        check(weapon.get_stream_num_events(stream_id: "reviews")! == reviewCount + 2)
        weapon.request_deck_selection()
        try await weapon.load_from_local_storage(stream_id: "deck_selection")
        try weapon.add_deck_selection_event(event: .SelectBothLanguages(native: .English, target: .French))
        let selection = weapon.get_deck_selection_state()!
        check(selection.native_language == .English && selection.target_language == .French)
        // Selection itself marks a course onboarded: hosts must defer it for a
        // new course until the wizard completes (prefetch requires no event).
        check(selection.onboarded_languages.contains(.French))
        let sync = weapon.get_sync_state(target: .Supabase)
        check(sync.remote_clock.isEmpty && sync.last_sync_error == nil)
        check(weapon.num_events_on_remote_as_of_last_sync(target: .Supabase) == 0)
        let earliest = weapon.get_timestamp_of_earliest_unsynced_event(target: .Supabase)!
        check(earliest.timestamp.seconds > 0 && earliest.timestamp.nanoseconds < 1_000_000_000)
        let beforeSyncInputs = weapon.deck_inputs_key(course: course)
        // Offline sync runs the same load/save pipeline against native storage.
        try await weapon.sync(stream_id: "reviews", access_token: nil, attempt_supabase: false, modifier: nil, upload: false)
        try await weapon.sync(stream_id: "deck_selection", access_token: nil, attempt_supabase: false, modifier: nil, upload: false)
        try await weapon.sync_with_supabase(access_token: "unused-offline", modifier: nil, upload: false)
        check(weapon.deck_inputs_key(course: course) == beforeSyncInputs)
        let reopened = try await Weapon.create(user_id: nil) { _, _ in }
        check(reopened.device_id == device)
        reopened.request_reviews()
        reopened.request_deck_selection()
        try await reopened.load_from_local_storage(stream_id: "reviews")
        try await reopened.load_from_local_storage(stream_id: "deck_selection")
        check(reopened.get_stream_num_events(stream_id: "reviews") == reviewCount + 2)
        check(reopened.get_deck_selection_state() == selection)
        // Disable fixture downloads; reopening must use the shared chunk cache.
        let packsOrigin = String(cString: getenv("YAP_PACKS_URL")!)
        var offlineRequest = URLRequest(url: URL(string: "\(packsOrigin)/__offline")!)
        offlineRequest.httpMethod = "POST"
        let (_, response) = try await URLSession.shared.data(for: offlineRequest)
        check((response as? HTTPURLResponse)?.statusCode == 204)
        try await reopened.cache_language_pack(course: course)
        let persisted = try await deck(reopened, course)
        check(persisted.get_daily_review_target() == 1200)
        do {
            _ = try await weapon.get_deck_state(course: course, utc_offset_seconds: 100_000)
            fatalError("expected invalid timezone error")
        } catch let error as BridgeError { check(error.description.contains("timezone")) }
        // Exercise the full Deck interface through generated objects, values, and getters.
        check(persisted.get_target_language() == .French)
        check(persisted.get_all_cards_summary().isEmpty && persisted.locked_count() == 0)
        check(persisted.get_lockup_offer(banned_challenge_types: [], timestamp_ms: future) == nil)
        check(persisted.get_release_offer(timestamp_ms: future) == nil)
        check(persisted.get_current_tier().tier > 0)
        check(persisted.get_current_week_progress().count == 7)
        for query: String? in [nil, "être", "etre", "love"] {
            let expected = persisted.get_gram_dictionary_entries(search_query: query, limit: 400).map(\.frequency_index)
            let first = persisted.get_gram_dictionary_page(search_query: query, offset: 0, limit: 200)
            let second = persisted.get_gram_dictionary_page(search_query: query, offset: 200, limit: 200)
            check((first + second).map(\.frequency_index) == expected)
            check(Set(expected).count == expected.count)
        }
        check(persisted.get_gram_dictionary_page(search_query: nil, offset: UInt64.max, limit: 200).isEmpty)
        check(persisted.get_gram_dictionary_page(search_query: nil, offset: 0, limit: 0).isEmpty)
        check(persisted.get_movie_poster(movie_id: "missing-smoke-movie") == nil)
        let ready = persisted.get_no_cards_ready_info(banned_challenge_types: [.Listening, .Speaking], sentence_list: nil)
        check(ready.smart_add_count > 0 && ready.smart_add_event != nil)
        let add = persisted.get_manual_add_option(card_type: .TargetLanguage, sentence_list: nil)!
        check(add.count > 0 && !add.label.isEmpty)
        reopened.add_deck_event(event: add.event)
        let withCards = try await deck(reopened, course)
        let cards = withCards.get_all_cards_summary()
        check(cards.count == Int(add.count) && !cards[0].card_text.isEmpty)
        check(cards[0].due_timestamp_ms.isFinite && !cards[0].state.isEmpty)
        // Reopening the pack can outlast the earlier goal timestamp. Query
        // just after these cards are due, independent of machine speed. The
        // exported timestamp truncates sub-millisecond Rust precision.
        let reviewTime = cards.map(\.due_timestamp_ms).max()! + 1
        let review = withCards.get_review_info(banned_challenge_types: [.Listening, .Speaking], timestamp_ms: reviewTime)
        check(review.total_count == UInt64(cards.count) && review.due_count > 0)
        var screenInputs = ReviewScreenInputs(banned: [.Listening, .Speaking], sentence_list: nil,
            online: false, is_signed_in: false, needs_display_name: false, display_name_dismissed: false,
            has_access_token: false, starting_fresh: nil, history_known: true,
            dismissed_accomplishment_at_review: withCards.get_total_reviews(), placement: nil, current_challenge: nil, timestamp_ms: reviewTime)
        let screen = withCards.review_screen_view(inputs: screenInputs)
        check(screen.total_count == UInt64(cards.count) && screen.target_language == .French)
        if case .Challenge = screen.step {} else { fatalError("expected a challenge") }
        let reviewed = withCards.review_card(reviewed: cards[0].card_indicator, rating: .Good)
        check(reviewed != nil)
        let beforeReviewInputs = reopened.deck_inputs_key(course: course)
        reopened.add_deck_event(event: reviewed!)
        check(reopened.deck_inputs_key(course: course) != beforeReviewInputs)
        let afterReview = try await deck(reopened, course)
        check(afterReview.get_total_reviews() == withCards.get_total_reviews() + 1)
        reopened.add_deck_event(event: afterReview.complete_placement_test(known_words: placement.known_words, unknown_words: placement.unknown_words))
        let placed = try await deck(reopened, course)
        check(placed.has_taken_placement_test())
        screenInputs.starting_fresh = false
        if case .PlacementTest = placed.review_screen_view(inputs: screenInputs).step { fatalError("placement must not repeat") }
        let onboarding = OnboardingSelections(starting_fresh: false, motivation: .JustForFun, experience_level: .CommonWords, study_goal: .Casual)
        reopened.add_deck_selection_event(event: .SetOnboardingSelections(selections: onboarding, target_language: .French))
        reopened.add_deck_selection_event(event: .SetHeardAbout(heard_about: .Other))
        let onboarded = reopened.get_deck_selection_state()!
        check(onboarded.onboarded_languages.contains(.French) && onboarded.onboarding_selections == onboarding && onboarded.heard_about == .Other)
        if let movie = placed.get_best_movie_sentence_list() {
            reopened.add_deck_event(event: placed.change_sentence_list(sentence_list: movie))
            let movieDeck = try await deck(reopened, course)
            check(movieDeck.get_sentence_list() == movie)
            reopened.add_deck_event(event: movieDeck.change_sentence_list(sentence_list: nil))
            let essentialDeck = try await deck(reopened, course)
            check(essentialDeck.get_sentence_list() == nil)
        }
        // Build a real backlog and verify the exact previews/events rendered by
        // ReviewPlanView, rather than fabricating bridge object handles.
        var backlog = try await deck(reopened, course)
        for _ in 0..<30 {
            if backlog.get_all_cards_summary().count > 20 { break }
            guard let event = backlog.get_manual_add_option(card_type: .TargetLanguage, sentence_list: nil)?.event else { break }
            reopened.add_deck_event(event: event)
            backlog = try await deck(reopened, course)
        }
        let dueAt = (backlog.get_all_cards_summary().map(\.due_timestamp_ms).max() ?? future) + 1
        let plan = backlog.get_lockup_offer(banned_challenge_types: [], timestamp_ms: dueAt)
        check(plan != nil && !plan!.keep_preview.isEmpty)
        check(plan!.keep_preview.allSatisfy { !$0.card_text.isEmpty })
        reopened.add_deck_event(event: plan!.lock_event)
        let locked = try await deck(reopened, course)
        check(locked.locked_count() > 0)
        let release = locked.get_release_offer(timestamp_ms: dueAt)
        check(release != nil && release!.release_count == UInt64(release!.release_preview.count))
        reopened.add_deck_event(event: release!.unlock_event)
        let released = try await deck(reopened, course)
        check(released.locked_count() < locked.locked_count())
        let word = persisted.get_gram_dictionary_page(search_query: "bonjour", offset: 0, limit: 1).first!
        let beforeDictionaryAdd = reopened.num_events
        reopened.add_deck_event(event: persisted.add_gram_by_frequency_index(frequency_index: word.frequency_index)!)
        let dictionaryDeck = try await deck(reopened, course)
        check(reopened.num_events == beforeDictionaryAdd + 1)
        // The word's index is its primary sense's; adding by it adds that sense.
        check(dictionaryDeck.gram_dictionary_entry(frequency_index: word.frequency_index)!.senses
            .first { $0.frequency_index == word.frequency_index }!.is_in_deck)
        print("PASS: bounded dictionary pages, relevance order, empty/overflow offsets, add-word event")
        print("PASS: lockup/release previews and immutable plan events")
        print("PASS: placement survives core/full rebuild; completion, onboarding, referral and sentence-list events fold")
        print("PASS: real Weapon factory, native filesystem, stable device ID, callbacks/reentrancy/unsubscribe, real French pack, returned Deck objects, TodaySummary, typed event inputs, optional counts, sync state/timestamps, and native persistence/reopen")
    }
}
