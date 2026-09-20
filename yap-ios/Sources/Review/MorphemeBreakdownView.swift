import SwiftUI

struct MorphemeBreakdownView: View {
    let parts: [BridgeTriple<String, String?, String?>]
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            if !parts.isEmpty { Text("Word breakdown").font(.headline) }
            ForEach(Array(parts.enumerated()), id: \.offset) { _, part in
                HStack(alignment: .firstTextBaseline, spacing: 12) {
                    Text(part.first).fontWeight(.semibold)
                    VStack(alignment: .leading, spacing: 4) {
                        if let meaning = part.second { Text(meaning) }
                        if let note = part.third { Text(note).font(.caption).foregroundStyle(.secondary) }
                    }
                }
            }
        }
    }
}
