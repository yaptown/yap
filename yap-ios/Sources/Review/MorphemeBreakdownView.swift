import SwiftUI

struct MorphemeBreakdownView: View {
    let parts: [BridgeTriple<String, String?, String?>]
    var alignment: HorizontalAlignment = .leading
    /// Fades the columns in one by one from this delay, as the web does under a revealed card.
    var revealDelay: Double?
    @State private var revealed = false
    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            grid.frame(minWidth: 0, maxWidth: .infinity, alignment: alignment == .center ? .center : .leading)
        }.defaultScrollAnchor(alignment == .center ? .center : .leading, for: .alignment)
    }
    var grid: some View {
        Grid(alignment: .leading, horizontalSpacing: 16, verticalSpacing: 2) {
            GridRow {
                ForEach(parts.indices, id: \.self) { index in
                    cell(parts[index].first, index).fontWeight(.medium)
                }
            }
            if parts.contains(where: { $0.second != nil }) {
                GridRow {
                    ForEach(parts.indices, id: \.self) { index in
                        cell(parts[index].second ?? "", index).italic().foregroundStyle(.secondary)
                    }
                }
            }
            GridRow {
                ForEach(parts.indices, id: \.self) { index in
                    cell(parts[index].third ?? "", index).foregroundStyle(.secondary)
                }
            }
        }
        .onAppear { revealed = true }
    }
    private func cell(_ text: String, _ index: Int) -> some View {
        Text(text).lineLimit(3).frame(maxWidth: 192, alignment: .leading)
            .opacity(revealDelay == nil || revealed ? 1 : 0)
            .animation(revealDelay.map { .easeOut(duration: 0.4).delay($0 + Double(index) * 0.2) }, value: revealed)
    }
}
