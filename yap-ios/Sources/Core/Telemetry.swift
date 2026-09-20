import Foundation
import OSLog
import Sentry

@MainActor enum Telemetry {
    static func start() {
        guard let dsn = Bundle.main.object(forInfoDictionaryKey: "SentryDSN") as? String,
              !dsn.isEmpty, !dsn.contains("$(") else { return }
        let version = get_app_version()
        SentrySDK.start { options in
            options.dsn = dsn
            options.releaseName = version
            #if DEBUG
            options.environment = "development"
            #else
            options.environment = "production"
            #endif
            options.tracesSampleRate = 0.2
            options.sendDefaultPii = true
            // Session replay is deliberately not enabled on iOS.
        }
    }
    static func user(_ id: String?) {
        SentrySDK.setUser(id.map { User(userId: $0) })
    }
    /// Every breadcrumb also lands in the unified log, so a failure on a phone
    /// without a Sentry DSN can still be read in Console.app (subsystem town.yap.ios).
    static func breadcrumb(_ category: String, _ message: String, failed: Bool = false) {
        let crumb = Breadcrumb(level: failed ? .error : .info, category: category)
        crumb.message = message
        SentrySDK.addBreadcrumb(crumb)
        let logger = Logger(subsystem: "town.yap.ios", category: category)
        if failed { logger.error("\(message, privacy: .public)") } else { logger.notice("\(message, privacy: .public)") }
    }
}
