#if DEBUG
import Foundation
import Observation

/// Explicit opt-in simulator driver. Never compiled into release builds. Commands
/// exercise the same view actions as taps; only the designated throwaway account
/// may receive test review/add events. No credentials are written to disk or logs.
@Observable @MainActor final class DebugHarness {
    static let shared = DebugHarness()
    var fixture: ChallengeFixture?
    var fixtureName = ""
    var command = ""
    var commandID = 0
    var forceDisplayName = false
    var activeTab: CourseTab = .learn
    private var started = false
    static func log(_ text: String) {
        print("Yap test: \(text)")
        fflush(nil)
        let url = FileManager.default.temporaryDirectory.appendingPathComponent("yap-test.log")
        let prior = (try? String(contentsOf: url, encoding: .utf8)) ?? ""
        try? (prior + text + "\n").write(to: url, atomically: true, encoding: .utf8)
    }
    static func dumpFixture(_ fixture: ChallengeFixture, name: String) {
        guard !name.isEmpty, name.allSatisfy({ $0.isLetter || $0.isNumber || $0 == "-" }) else {
            log("invalid fixture name"); return
        }
        do {
            let directory = FileManager.default.temporaryDirectory.appendingPathComponent("fixtures")
            try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
            let url = directory.appendingPathComponent(name + ".json")
            try challenge_fixture_json(fixture: fixture).write(to: url, atomically: true, encoding: .utf8)
            log("fixture written \(url.path)")
        } catch { log("fixture write failed: \(error)") }
    }
    func start(auth: AuthStore) async {
        guard !started else { return }; started = true
        let args = CommandLine.arguments
        if let i = args.firstIndex(of: "--fixture"), args.count > i + 1 {
            let url = URL(fileURLWithPath: args[i + 1])
            do {
                fixture = try parse_challenge_fixture(json: String(contentsOf: url, encoding: .utf8))
                fixtureName = url.deletingPathExtension().lastPathComponent
            } catch { Self.log("fixture failed: \(error)") }
        }
        if let i = args.firstIndex(of: "--test-credentials"), args.count > i + 2 {
            await auth.signIn(email: args[i + 1], password: args[i + 2])
            Self.log(auth.userId == nil ? "sign-in failed: \(auth.error ?? "unknown")" : "sign-in succeeded")
        }
        guard args.contains("--test-driver") else { return }
        let commandURL = FileManager.default.temporaryDirectory.appendingPathComponent("yap-command")
        while !Task.isCancelled {
            do { try await Task.sleep(for: .milliseconds(250)) } catch { return }
            guard let input = try? String(contentsOf: commandURL, encoding: .utf8) else { continue }
            try? FileManager.default.removeItem(at: commandURL)
            let value = input.trimmingCharacters(in: .whitespacesAndNewlines)
            if value == "signout" { await auth.signOut(); Self.log(auth.userId == nil ? "signed out" : "signout failed"); continue }
            guard auth.session?.user.email == "yap-mcp-test@popovit.ch" else {
                Self.log("Ignoring test action outside throwaway account"); continue
            }
            command = value; commandID += 1
            Self.log("action \(value)")
        }
    }
}
#endif
