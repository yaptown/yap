import SwiftUI

struct MorphemeBreakdownView: View {
    let parts: [BridgeTriple<String, String?, String?>]
    var alignment: HorizontalAlignment = .leading
    /// Fades the columns in one by one from this delay, as the web does under a revealed card.
    var revealDelay: Double?
    /// Scrolls with the grid, for punctuation that must stay beside it.
    var trailing: Text?
    @State private var revealed = false
    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(alignment: .top, spacing: 8) { grid; trailing }
                .frame(minWidth: 0, maxWidth: .infinity, alignment: alignment == .center ? .center : .leading)
        }.defaultScrollAnchor(alignment == .center ? .center : .leading, for: .alignment)
    }
    var grid: some View {
        Grid(alignment: .topLeading, horizontalSpacing: 16, verticalSpacing: 2) {
            GridRow {
                ForEach(parts.indices, id: \.self) { index in
                    cell(parts[index].first, index).fontWeight(.medium)
                }
            }
            if parts.contains(where: { $0.second != nil }) {
                GridRow {
                    ForEach(parts.indices, id: \.self) { index in
                        cell(parts[index].second ?? "", index).italic().foregroundStyle(Color.yapMuted)
                    }
                }
            }
            GridRow {
                ForEach(parts.indices, id: \.self) { index in
                    cell(parts[index].third ?? "", index).foregroundStyle(Color.yapMuted)
                }
            }
        }
        .onAppear { revealed = true }
    }
    private func cell(_ text: String, _ index: Int) -> some View {
        WrapWidth(max: 192) { Text(text).lineLimit(3) }
            .opacity(revealDelay == nil || revealed ? 1 : 0)
            .animation(revealDelay.map { .easeOut(duration: 0.4).delay($0 + Double(index) * 0.2) }, value: revealed)
    }
}

/// The horizontal scroll view proposes no width, so text would lay out on one line and
/// `frame(maxWidth:)` would only truncate it. Proposing the capped width makes it wrap.
private struct WrapWidth: Layout {
    let max: CGFloat
    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let width = min(subviews[0].sizeThatFits(.unspecified).width, proposal.width ?? max, max)
        return subviews[0].sizeThatFits(ProposedViewSize(width: width, height: nil))
    }
    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        subviews[0].place(at: bounds.origin, proposal: ProposedViewSize(bounds.size))
    }
}
