import SwiftUI

struct SentenceVerdictView: View {
    let submission: String
    let correct: String
    let perfect: Bool
    let encouragement: String?
    let explanation: String?
    let error: String?
    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            if perfect { Label("Perfect!", systemImage: "checkmark.circle.fill").font(.title.bold()).foregroundStyle(.green) }
            if !perfect { VStack(alignment: .leading, spacing: 4) { Text("Your answer").font(.caption); Text(submission) } }
            VStack(alignment: .leading, spacing: 4) {
                Text("Correct answer").font(.caption)
                Text(correct).font(.title3.weight(.semibold)).textSelection(.enabled)
            }.foregroundStyle(.green)
            if error != nil {
                Text("Your answer couldn't be graded automatically. Please grade the words below.").foregroundStyle(.orange)
            }
            if let encouragement { Text(markdown(encouragement)) }
            if let explanation { Text(markdown(explanation)).foregroundStyle(.secondary) }
        }
    }
}

func markdown(_ text: String) -> AttributedString {
    let text = text.replacingOccurrences(of: "<word>", with: "**").replacingOccurrences(of: "</word>", with: "**")
    return (try? AttributedString(markdown: text)) ?? AttributedString(text)
}

struct ReviewDefinitionsView: View {
    let definitions: [ReviewDefinition]
    var body: some View {
        ForEach(Array(definitions.enumerated()), id: \.offset) { _, entry in
            DefinitionView(definition: entry.definition)
            if let breakdown = entry.breakdown { MorphemeBreakdownView(parts: breakdown) }
        }
    }
}

/// Natural wrapping for sentence words and inline dictation fields.
struct SentenceFlow: Layout {
    var spacing: CGFloat = 6
    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        arrange(width: proposal.width ?? 320, subviews: subviews).size
    }
    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let layout = arrange(width: bounds.width, subviews: subviews)
        for (index, point) in layout.points.enumerated() {
            subviews[index].place(at: CGPoint(x: bounds.minX + point.x, y: bounds.minY + point.y), anchor: .topLeading,
                                 proposal: ProposedViewSize(width: min(bounds.width, subviews[index].sizeThatFits(.unspecified).width), height: nil))
        }
    }
    private func arrange(width: CGFloat, subviews: Subviews) -> (size: CGSize, points: [CGPoint]) {
        var x: CGFloat = 0, y: CGFloat = 0, row: CGFloat = 0
        var points: [CGPoint] = []
        for view in subviews {
            let ideal = view.sizeThatFits(.unspecified)
            let size = view.sizeThatFits(ProposedViewSize(width: min(width, ideal.width), height: nil))
            if x > 0 && x + size.width > width { x = 0; y += row + spacing; row = 0 }
            points.append(CGPoint(x: x, y: y)); x += size.width + spacing; row = max(row, size.height)
        }
        return (CGSize(width: width, height: y + row), points)
    }
}
