import Foundation
import SwiftUI
import Observation

@Observable @MainActor final class YapModel {
    var status = "Opening Yap…"
    var summary: TodaySummary?
    var targetSeconds: UInt32 = 600
    var notificationCount = 0
    var didTryGoal = false
    var progress: Double = 0
    var error: String?
    private var weapon: Weapon?
    private var listener: ListenerKey?
    private let course = Course(native_language: .English, target_language: .French)

    func start() async {
        guard weapon == nil else { return }
        do {
            let weapon = try await Weapon.create(user_id: nil) { _, _ in }
            self.weapon = weapon
            listener = weapon.subscribe_to_stream(stream_id: "reviews") { [weak self] in
                guard let self else { return }
                self.notificationCount += 1
                guard self.summary != nil else { return }
                Task { @MainActor [weak self] in await self?.refresh() }
            }
            weapon.request_reviews()
            try await weapon.load_from_local_storage(stream_id: "reviews")
            status = "Loading the French language pack…"
            try await weapon.load_language_pack(course: course) { [weak self] _, percent in
                self?.progress = Double(percent) / 100
            }
            await refresh()
        } catch { self.error = String(describing: error); status = "Couldn’t open Yap" }
    }
    func refresh() async {
        guard let weapon else { return }
        do {
            let offset = Int32(TimeZone.current.secondsFromGMT())
            let deck = try await weapon.get_deck_state(course: course, utc_offset_seconds: offset)
            summary = deck.get_today_summary()
            targetSeconds = deck.get_daily_review_target()
            status = "French · for English speakers"
        } catch { self.error = String(describing: error) }
    }
    func tryGoal() {
        guard let weapon, !didTryGoal else { return }
        let event = DeckEvent.Language(LanguageEvent(target_language: .French, native_language: .English,
            content: .SetDailyReviewTarget(daily_review_target: .Intense)))
        weapon.add_deck_event(event: event)
        didTryGoal = true
        Task { @MainActor in
            do {
                try await weapon.sync(stream_id: "reviews", access_token: nil, attempt_supabase: false, modifier: nil, upload: false)
                await refresh()
            } catch { self.error = String(describing: error) }
        }
    }
    isolated deinit {
        if let weapon, let listener { weapon.unsubscribe(key: listener) }
    }
}

struct YapView: View {
    @State private var model = YapModel()
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                VStack(alignment: .leading, spacing: 6) {
                    Text("Yap").font(.system(size: 38, weight: .bold, design: .rounded))
                    Text(model.status).foregroundStyle(.secondary)
                }
                if let summary = model.summary {
                    Text(summary.day_of_week).font(.title2.weight(.semibold))
                    HStack(spacing: 24) {
                        metric("Reviews", "\(summary.reviews)")
                        metric("Learned", "\(summary.new_cards.count)")
                        metric("Daily goal", "\(model.targetSeconds / 60) min")
                    }
                    Divider()
                    Text("Ready for a fresh start.").font(.headline)
                    Text("Learn French, one day at a time.")
                        .foregroundStyle(.secondary)
                    Button(model.didTryGoal ? "Goal updated" : "Try a 20-minute goal") { model.tryGoal() }
                        .buttonStyle(.borderedProminent).disabled(model.didTryGoal)
                    Text("Local prototype · no account connected")
                        .font(.caption).foregroundStyle(.secondary)
                } else if model.error == nil {
                    ProgressView(value: model.progress)
                }
                if let error = model.error { Text(error).foregroundStyle(.red).textSelection(.enabled) }
            }
            .padding(24).frame(maxWidth: 560, alignment: .leading)
        }
        #if os(macOS)
        .frame(width: 560, height: 440)
        #endif
        .task {
            await model.start()
            #if os(iOS)
            if CommandLine.arguments.contains("--device-check") {
                await runDeviceChecks(startupError: model.error)
            }
            #endif
        }
    }
    private func metric(_ label: String, _ value: String) -> some View {
        VStack(alignment: .leading, spacing: 5) {
            Text(value).font(.system(size: 28, weight: .semibold, design: .rounded))
            Text(label).font(.subheadline).foregroundStyle(.secondary)
        }
    }
}

@main struct YapPrototypeApp: App {
    init() {
        #if os(iOS)
        let root = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("YapPrototype")
        // Automated checks use separate storage from the app's interactive session.
        let data = CommandLine.arguments.contains("--device-check")
            ? FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
            : root
        setenv("YAP_DATA_DIR", data.path, 1)
        #else
        let temporary = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        setenv("YAP_DATA_DIR", temporary.path, 1)
        #endif
        try! YapHost.initialize()
    }
    var body: some Scene {
        WindowGroup { YapView() }
        #if os(macOS)
        .windowResizability(.contentSize)
        #endif
    }
}
