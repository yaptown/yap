import SwiftUI

struct SentenceVerdictView: View {
    let submission: String
    let correct: String
    let perfect: Bool
    let encouragement: String?
    let explanation: String?
    let error: String?
    var correctLabel = "Correct translation:"
    var submissionLabel = "Your translation:"
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            if !perfect {
                VStack(alignment: .leading, spacing: 4) {
                    Text(submissionLabel).font(.footnote.weight(.medium))
                    Text(submission).font(.body.weight(.medium))
                }.frame(maxWidth: .infinity, alignment: .leading).padding(12)
                    .overlay { RoundedRectangle(cornerRadius: 10).strokeBorder(Color(uiColor: .separator)) }
            }
            VStack(alignment: .leading, spacing: 4) {
                Text(correctLabel).font(.footnote.weight(.medium)).foregroundStyle(.green)
                Text(correct).font(.body.weight(.medium)).textSelection(.enabled)
            }.frame(maxWidth: .infinity, alignment: .leading).padding(12)
                .background(.green.opacity(0.1), in: RoundedRectangle(cornerRadius: 10))
                .overlay { RoundedRectangle(cornerRadius: 10).strokeBorder(.green.opacity(0.2)) }
            if error != nil {
                Text("Your submission could not be graded automatically. Please grade the words manually below.").font(.footnote).foregroundStyle(.orange)
            }
            if encouragement?.isEmpty == false || explanation?.isEmpty == false {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Feedback:").font(.footnote.weight(.medium)).foregroundStyle(.blue)
                    if let encouragement, !encouragement.isEmpty {
                        HStack(alignment: .top, spacing: 8) {
                            if Theme.emojiFontAvailable { Text(perfect ? "🎉" : "☀️") }
                            else { Image(systemName: perfect ? "party.popper" : "sun.max") }
                            Text(markdown(encouragement))
                        }.font(.subheadline.weight(.medium)).foregroundStyle(.green).padding(8)
                            .overlay(alignment: .leading) { Rectangle().fill(.green.opacity(0.4)).frame(width: 2) }
                    }
                    if let explanation, !explanation.isEmpty { Text(markdown(explanation)).font(.subheadline) }
                }.frame(maxWidth: .infinity, alignment: .leading).padding(12)
                    .background(.blue.opacity(0.1), in: RoundedRectangle(cornerRadius: 10))
                    .overlay { RoundedRectangle(cornerRadius: 10).strokeBorder(.blue.opacity(0.2)) }
            }
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
            CompactDefinitionRow(entry: entry).padding(.vertical, 8)
        }
    }
}

private struct CompactDefinitionRow: View {
    let entry: ReviewDefinition
    private var word: String { entry.definition.headword }
    var body: some View {
        ViewThatFits(in: .horizontal) {
            HStack(alignment: .top, spacing: 8) {
                heading(scrolling: false).fixedSize(horizontal: true, vertical: false)
                definitionBody.frame(minWidth: 160, alignment: .leading)
            }
            VStack(alignment: .leading, spacing: 8) {
                heading(scrolling: true)
                definitionBody
            }
        }.font(.subheadline)
    }
    private func heading(scrolling: Bool) -> some View {
        HStack(alignment: .top, spacing: 8) {
            if let parts = entry.breakdown, !parts.isEmpty {
                if scrolling { MorphemeBreakdownView(parts: parts) }
                else { MorphemeBreakdownView(parts: parts).grid }
            } else { Text(word).fontWeight(.semibold) }
            Text(":").fontWeight(.semibold)
        }
    }
    @ViewBuilder private var definitionBody: some View {
        VStack(alignment: .leading, spacing: 8) {
            ForEach(Array(entry.definition.senses.enumerated()), id: \.offset) { _, sense in
                VStack(alignment: .leading, spacing: 4) {
                    Text(sense.meaning) + Text(sense.note.map { " " + $0 } ?? "").foregroundColor(.secondary)
                    if let example = sense.example {
                        DefinitionExamples(example: example)
                    }
                }
            }
        }
    }
}

struct DefinitionExamples: View {
    let example: SenseExample
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("\"\(example.target)\"").italic()
            Text("\"\(example.native)\"")
        }.font(.footnote).foregroundStyle(.secondary)
    }
}

/// Natural wrapping for sentence words and inline dictation fields.
struct SentenceFlow: Layout {
    var spacing: CGFloat = 6
    var alignment: HorizontalAlignment = .leading
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
        var sizes: [CGSize] = []
        var rowStart = 0
        // Center each finished row horizontally when asked, and always center its
        // items vertically so an inline field sits level with the words beside it.
        func finishRow() {
            let offset = alignment == .center ? max(0, (width - max(0, x - spacing)) / 2) : 0
            for index in rowStart..<points.count {
                points[index].x += offset
                points[index].y += (row - sizes[index].height) / 2
            }
        }
        for view in subviews {
            let ideal = view.sizeThatFits(.unspecified)
            let size = view.sizeThatFits(ProposedViewSize(width: min(width, ideal.width), height: nil))
            if x > 0 && x + size.width > width { finishRow(); rowStart = points.count; x = 0; y += row + spacing; row = 0 }
            points.append(CGPoint(x: x, y: y)); sizes.append(size); x += size.width + spacing; row = max(row, size.height)
        }
        finishRow()
        return (CGSize(width: width, height: y + row), points)
    }
}
