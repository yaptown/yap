import Foundation

/// Native presentation copy uses the same Fluent catalog as the shared views.
@MainActor func L(_ id: String, _ arguments: [String: Any] = [:]) -> String {
    let json = try! JSONSerialization.data(withJSONObject: arguments)
    return YapHost.translate(id: id, arguments: String(data: json, encoding: .utf8)!)
}
