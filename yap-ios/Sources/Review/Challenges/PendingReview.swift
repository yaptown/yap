import Foundation
import CryptoKit

/// Disposable UI state, not event storage. The bridge codec avoids a second
/// model of Rust's grades. Version, full challenge, account, course and review
/// count all scope the snapshot; a changed schema simply discards the draft.
@MainActor struct PendingReview {
    /// One slot per kind, account and course: a draft for a different challenge
    /// (or app version, or review count) is discarded rather than left behind.
    let key: String
    let identity: String
    private struct Slot: Codable { let identity: String; let payload: Data }
    init<T: BridgeValue>(kind: String, challenge: T, model: ReviewModel) {
        let encoded = (try? Self.encode(challenge)) ?? Data()
        let digest = SHA256.hash(data: encoded).map { String(format: "%02x", $0) }.joined()
        key = "yap-pending-\(kind)-\(model.session.userId)-\(String(describing: model.course))"
        identity = "v1-\(get_app_version())-\(model.deck.get_total_reviews())-\(digest)"
    }
    static func encode<T: BridgeValue>(_ value: T) throws -> Data {
        var writer = BridgeWriter()
        try value.bridgeWrite(&writer)
        return Data(writer.bytes)
    }
    static func decode<T: BridgeValue>(_ data: Data, as type: T.Type) throws -> T {
        guard data.count <= bridgeMaxBytes else { throw BridgeError(description: "Oversized pending review") }
        var reader = BridgeReader(data: data)
        let value: T = try reader.read()
        guard reader.offset == data.count else { throw BridgeError(description: "Trailing pending review data") }
        return value
    }
    func load<T: Decodable>(_ type: T.Type) -> T? {
        guard let data = UserDefaults.standard.data(forKey: key) else { return nil }
        do {
            let slot = try JSONDecoder().decode(Slot.self, from: data)
            guard slot.identity == identity else { clear(); return nil }
            return try JSONDecoder().decode(type, from: slot.payload)
        } catch { clear(); return nil }
    }
    func save<T: Encodable>(_ value: T) {
        do {
            let slot = Slot(identity: identity, payload: try JSONEncoder().encode(value))
            UserDefaults.standard.set(try JSONEncoder().encode(slot), forKey: key)
        } catch { print("Yap pending review: \(error)") }
    }
    func clear() { UserDefaults.standard.removeObject(forKey: key) }
}
