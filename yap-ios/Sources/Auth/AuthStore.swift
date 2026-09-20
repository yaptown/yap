import Foundation
import Observation
import Supabase

@Observable @MainActor final class AuthStore {
    private(set) var session: Session? { didSet { Telemetry.user(userId) } }
    private(set) var restoring = true
    private(set) var busy = false
    var displayName: String?
    var needsDisplayName = false
    var error: String?
    var userId: String? { session?.user.id.uuidString.lowercased() }
    var accessToken: String? { session?.accessToken }
    let client: SupabaseClient
    private var observation: Task<Void, Never>?

    init() {
        #if DEBUG
        if CommandLine.arguments.contains("--test-offline") { URLProtocol.registerClass(OfflineTestProtocol.self) }
        #endif
        let config = supabase_config()
        client = SupabaseClient(supabaseURL: URL(string: config.supabase_url)!, supabaseKey: config.supabase_anon_key)
        let auth = client.auth
        observation = Task { [weak self] in
            // currentSession is the persisted Keychain value and works offline.
            self?.session = auth.currentSession
            self?.restoring = false
            for await (_, session) in auth.authStateChanges {
                guard !Task.isCancelled else { break }
                self?.session = session
                await self?.loadProfile()
            }
        }
    }

    private func loadProfile() async {
        guard let id = userId else { displayName = nil; needsDisplayName = false; return }
        struct Profile: Decodable { let display_name: String? }
        do {
            let profiles: [Profile] = try await client.from("profiles").select("display_name").eq("id", value: id).execute().value
            guard userId == id else { return }
            displayName = profiles.first?.display_name
            needsDisplayName = displayName == nil
        } catch { print("Yap profile lookup failed: \(error)") }
    }

    func signIn(email: String, password: String) async {
        busy = true; error = nil
        defer { busy = false }
        do { session = try await client.auth.signIn(email: email, password: password) }
        catch { self.error = error.localizedDescription }
    }

    func signUp(email: String, password: String) async {
        busy = true; error = nil
        defer { busy = false }
        do {
            _ = try await client.auth.signUp(email: email, password: password)
            session = try await client.auth.signIn(email: email, password: password)
        } catch { self.error = error.localizedDescription }
    }

    func signOut() async {
        busy = true; error = nil
        defer { busy = false }
        do { try await client.auth.signOut(); session = nil; displayName = nil }
        catch { self.error = error.localizedDescription }
    }
    isolated deinit { observation?.cancel() }
}

#if DEBUG
/// Fail SDK HTTP requests, rather than relying on the simulator's Wi-Fi icon.
private final class OfflineTestProtocol: URLProtocol {
    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }
    override func startLoading() { client?.urlProtocol(self, didFailWithError: URLError(.notConnectedToInternet)) }
    override func stopLoading() {}
}
#endif
