import Foundation

/// Runs only when the device harness launches the app with --device-check.
@MainActor func runDeviceChecks(startupError: String?) async {
    let report = URL.documentsDirectory.appendingPathComponent("device-check.json")
    let index = CommandLine.arguments.firstIndex(of: "--device-check-id")
    let runID = index.flatMap { CommandLine.arguments.indices.contains($0 + 1) ? CommandLine.arguments[$0 + 1] : nil } ?? "manual"
    do {
        if let startupError { throw NSError(domain: "YapDeviceCheck", code: 1, userInfo: [NSLocalizedDescriptionKey: startupError]) }
        let course = Course(native_language: .English, target_language: .French)
        let weapon = try await Weapon.create(user_id: nil) { _, _ in }
        var notifications = 0
        let listener = weapon.subscribe_to_stream(stream_id: "reviews") { notifications += 1 }
        defer { weapon.unsubscribe(key: listener) }
        weapon.request_reviews()
        try await weapon.load_from_local_storage(stream_id: "reviews")
        try await weapon.load_language_pack(course: course, on_progress: nil)
        let deck = try await weapon.get_deck_state(course: course, utc_offset_seconds: 0)
        guard !deck.get_today_summary().day_of_week.isEmpty else { throw CheckFailure("empty summary") }
        guard get_language_metadata(language: .French).native_name == "Français" else { throw CheckFailure("Unicode metadata") }
        let before = notifications
        weapon.add_deck_event(event: .Language(LanguageEvent(target_language: .French, native_language: .English,
            content: .SetDailyReviewTarget(daily_review_target: .Intense))))
        guard notifications > before else { throw CheckFailure("event callback") }
        try await weapon.sync(stream_id: "reviews", access_token: nil, attempt_supabase: false, modifier: nil, upload: false)
        let reopened = try await Weapon.create(user_id: nil) { _, _ in }
        reopened.request_reviews()
        try await reopened.load_from_local_storage(stream_id: "reviews")
        try await reopened.load_language_pack(course: course, on_progress: nil)
        let persisted = try await reopened.get_deck_state(course: course, utc_offset_seconds: 0)
        guard reopened.device_id == weapon.device_id, persisted.get_daily_review_target() == 1200 else {
            throw CheckFailure("filesystem persistence")
        }
        let data: [String: Any] = ["passed": true, "run_id": runID, "platform": "physical iPhone", "checks": [
            "Rust async factory", "HTTPS language pack", "Unicode values", "opaque Deck objects",
            "callbacks", "event append", "filesystem save and reopen", "stable device ID",
        ]]
        try JSONSerialization.data(withJSONObject: data, options: .prettyPrinted).write(to: report)
        print("PASS: physical iPhone Yap integration")
    } catch {
        let data: [String: Any] = ["passed": false, "run_id": runID, "error": String(describing: error)]
        try? JSONSerialization.data(withJSONObject: data, options: .prettyPrinted).write(to: report)
        print("FAIL: physical iPhone Yap integration: \(error)")
    }
}

private struct CheckFailure: Error {
    let message: String
    init(_ message: String) { self.message = message }
}
