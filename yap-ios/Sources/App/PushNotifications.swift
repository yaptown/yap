import OneSignalFramework
import UIKit
import UserNotifications

/// Reminders reach the phone through OneSignal, the same as on the web: the deck
/// uploads its schedule (`submit_push_notifications`), and a Supabase cron job
/// sends each one to the account's OneSignal external id. This links the device
/// to that id, and tracks whether the learner allows notifications and when they
/// were last asked, which Rust turns into the review screen's prompt.
@Observable @MainActor final class PushNotifications {
    static let shared = PushNotifications()
    /// The production OneSignal app, shared with yap.town so one send reaches
    /// every device on the account.
    private static let appId = "31e87860-a9d1-435b-9838-cb7643bb5cdf"
    private static let askedAtKey = "yap-notifications-asked-at-ms"
    private static let askedAtReviewKey = "yap-notifications-asked-at-review"

    /// `nil` until the system has answered.
    private(set) var permission: NotificationPermission?
    private(set) var lastAsked: NotificationAsk?

    private init() {
        OneSignal.initialize(Self.appId, withLaunchOptions: nil)
        let defaults = UserDefaults.standard
        if let at = defaults.object(forKey: Self.askedAtKey) as? Double {
            lastAsked = NotificationAsk(at_ms: at, at_review: UInt64(defaults.integer(forKey: Self.askedAtReviewKey)))
        }
        Task { await refresh() }
    }

    var promptInputs: NotificationPromptInputs? {
        permission.map { NotificationPromptInputs(permission: $0, last_asked: lastAsked) }
    }

    func refresh() async {
        let status = await UNUserNotificationCenter.current().notificationSettings().authorizationStatus
        permission = switch status {
        case .notDetermined: .NotDetermined
        case .denied: .Denied
        default: .Granted
        }
    }

    /// Sends the account's reminders to this device; signed out, it gets none.
    func link(userId: String?) {
        if let userId { OneSignal.login(userId) } else { OneSignal.logout() }
    }

    /// Shows the system prompt while it can still be shown; after that, only
    /// Settings can turn notifications on, so it opens them.
    func enable(atReview review: UInt64) async {
        recordAsk(atReview: review)
        if permission == .NotDetermined {
            _ = await withCheckedContinuation { done in
                OneSignal.Notifications.requestPermission({ done.resume(returning: $0) }, fallbackToSettings: false)
            }
        } else if let url = URL(string: UIApplication.openNotificationSettingsURLString) {
            _ = await UIApplication.shared.open(url)
        }
        await refresh()
    }

    func recordAsk(atReview review: UInt64) {
        let ask = NotificationAsk(at_ms: Date().timeIntervalSince1970 * 1000, at_review: review)
        UserDefaults.standard.set(ask.at_ms, forKey: Self.askedAtKey)
        UserDefaults.standard.set(Int(ask.at_review), forKey: Self.askedAtReviewKey)
        lastAsked = ask
    }
}
