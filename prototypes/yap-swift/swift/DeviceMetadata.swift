import Foundation
import SwiftUI

/// This small bootstrap runs the linked iOS library's generator on the phone.
@main struct DeviceMetadataApp: App {
    @State private var status = "Generating Yap’s bindings…"

    var body: some Scene {
        WindowGroup {
            Text(status).padding().task { generate() }
        }
    }

    @MainActor private func generate() {
        guard let index = CommandLine.arguments.firstIndex(of: "--generation-id"),
              CommandLine.arguments.indices.contains(index + 1) else { return }
        let output = URL.documentsDirectory.appendingPathComponent("Bindings-" + CommandLine.arguments[index + 1])
        let path = Array(output.path.utf8)
        let result = path.withUnsafeBufferPointer {
            bridgerton_generate_v1($0.baseAddress, $0.count)
        }
        defer { bridgerton_buffer_free(result.data) }
        if result.status == 0 {
            status = "Bindings generated on this iPhone. Return to your Mac to finish installation."
            print("PASS: Yap binding generation executed on the physical iPhone")
        } else {
            let bytes = result.data.data.map { Data(bytes: $0, count: result.data.len) } ?? Data()
            status = "Generation failed: \(String(decoding: bytes, as: UTF8.self))"
            try? Data(status.utf8).write(to: URL.documentsDirectory.appendingPathComponent("generation-error.txt"))
        }
    }
}
