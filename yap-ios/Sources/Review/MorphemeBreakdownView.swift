import SwiftUI

struct MorphemeBreakdownView: View {
    let parts: [BridgeTriple<String, String?, String?>]
    var alignment: HorizontalAlignment = .leading
    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            grid.frame(minWidth: 0, maxWidth: .infinity, alignment: alignment == .center ? .center : .leading)
        }.defaultScrollAnchor(alignment == .center ? .center : .leading, for: .alignment)
    }
    var grid: some View {
        Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 2) {
            GridRow {
                ForEach(parts.indices, id: \.self) { index in
                    cell(parts[index].first).fontWeight(.medium)
                }
            }
            if parts.contains(where: { $0.second != nil }) {
                GridRow {
                    ForEach(parts.indices, id: \.self) { index in
                        cell(parts[index].second ?? "").italic().foregroundStyle(.secondary)
                    }
                }
            }
            GridRow {
                ForEach(parts.indices, id: \.self) { index in
                    cell(parts[index].third ?? "").foregroundStyle(.secondary)
                }
            }
        }.font(.subheadline)
    }
    private func cell(_ text: String) -> some View {
        Text(text).lineLimit(3).frame(maxWidth: 192, alignment: .leading)
    }
}
