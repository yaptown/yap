import SwiftUI

/// The graded half of a sentence challenge: how it went, the reference
/// answer and the autograder's notes. While grading, placeholder bars hold
/// the headline's and feedback's places so nothing jumps when they arrive;
/// the reference answer shows from the start.
struct SentenceVerdictView: View, Equatable {
    /// Nil while grading.
    let headline: VerdictHeadline?
    let correctLabel: String
    let correct: String
    var feedbackLabel = ""
    var encouragement: String?
    var explanation: String?
    var error: String?
    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            if let headline { VerdictHeadlineView(headline: headline) } else { SkeletonBars(widths: [0.45]) }
            if headline?.tone != .Perfect {
                VStack(alignment: .leading, spacing: 6) {
                    SectionLabel(text: correctLabel)
                    Text(correct).font(.title3).textSelection(.enabled)
                }
            }
            if let headline {
                if error != nil {
                    Text("Your submission could not be graded automatically. Please grade the words manually below.")
                        .font(.footnote).foregroundStyle(Color.yapCautionForeground)
                }
                FeedbackSection(label: feedbackLabel, tone: headline.tone, encouragement: encouragement, explanation: explanation)
            } else {
                SkeletonBars(widths: [0.35, 0.9, 0.65])
            }
        }.frame(maxWidth: .infinity, alignment: .leading)
    }
}

/// A quiet uppercase caption over one block of a challenge card.
struct SectionLabel: View {
    let text: String
    var color = Color.yapMuted
    var body: some View {
        Text(text.uppercased()).font(.caption.weight(.semibold)).tracking(1.2).foregroundStyle(color)
    }
}

@MainActor extension VerdictTone {
    var foreground: Color {
        switch self { case .Perfect: .yapPositiveForeground; case .Almost: .yapCautionForeground; case .Wrong: .yapNegativeForeground }
    }
    var line: Color {
        switch self { case .Perfect: .yapPositive; case .Almost: .yapCaution; case .Wrong: .yapNegative }
    }
    var surface: Color {
        switch self { case .Perfect: .yapPositiveSurface; case .Almost: .yapCautionSurface; case .Wrong: .yapNegativeSurface }
    }
}

/// "Nailed it!" beside a mark in the verdict's tint.
struct VerdictHeadlineView: View {
    let headline: VerdictHeadline
    var body: some View {
        HStack(spacing: 12) {
            mark.font(.subheadline.weight(.heavy)).foregroundStyle(headline.tone.foreground)
                .frame(width: 32, height: 32).background(headline.tone.surface, in: Circle())
                .accessibilityHidden(true)
            Text(headline.text).font(.title2.bold())
        }.fadeIn(duration: 0.25)
    }
    @ViewBuilder private var mark: some View {
        switch headline.tone {
        case .Perfect: Image(systemName: "checkmark")
        case .Almost: Text("~").font(.headline.weight(.heavy))
        case .Wrong: Image(systemName: "xmark")
        }
    }
}

/// The autograder's encouragement and explanation as plain paragraphs.
struct FeedbackSection: View, Equatable {
    let label: String
    let tone: VerdictTone
    let encouragement: String?
    let explanation: String?
    var body: some View {
        let paragraphs = [encouragement, explanation].compactMap { $0 }.filter { !$0.isEmpty }
        if !paragraphs.isEmpty {
            VStack(alignment: .leading, spacing: 6) {
                SectionLabel(text: label, color: tone.foreground)
                VStack(alignment: .leading, spacing: 10) { ForEach(paragraphs, id: \.self) { Text(markdown($0)) } }
            }.frame(maxWidth: .infinity, alignment: .leading).fadeIn(duration: 0.25)
        }
    }
}

/// Pulsing placeholder lines, each a fraction of the available width.
struct SkeletonBars: View {
    let widths: [CGFloat]
    @State private var dim = false
    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            ForEach(Array(widths.enumerated()), id: \.offset) { _, width in
                GeometryReader { proxy in Capsule().fill(Color.yapMuted.opacity(0.2)).frame(width: proxy.size.width * width) }
                    .frame(height: 12)
            }
        }
        .opacity(dim ? 0.5 : 1)
        .animation(.easeInOut(duration: 0.9).repeatForever(), value: dim)
        .onAppear { dim = true }
        .accessibilityHidden(true)
    }
}

/// The web's collapsible grading list: a quiet title row with a Show/Hide toggle.
struct GradeSectionDisclosure<Content: View>: View {
    let title: String
    @Binding var isExpanded: Bool
    @ViewBuilder let content: () -> Content
    var body: some View {
        VStack(spacing: 12) {
            Button { withAnimation(.easeOut(duration: 0.15)) { isExpanded.toggle() } } label: {
                HStack {
                    Text(title).font(.subheadline.weight(.medium))
                    Spacer()
                    Text(isExpanded ? "Hide" : "Show").font(.caption).foregroundStyle(Color.yapMuted)
                }.contentShape(Rectangle())
            }.buttonStyle(.plain).frame(minHeight: 32)
            if isExpanded { content() }
        }
    }
}

func markdown(_ text: String) -> AttributedString {
    let text = text.replacingOccurrences(of: "<word>", with: "**").replacingOccurrences(of: "</word>", with: "**")
    return (try? AttributedString(markdown: text)) ?? AttributedString(text)
}

struct ReviewDefinitionsView: View, Equatable {
    let definitions: [ReviewDefinition]
    var body: some View {
        ForEach(Array(definitions.enumerated()), id: \.offset) { _, entry in
            CompactDefinitionRow(entry: entry).equatable().padding(.vertical, 8)
        }
    }
}

private struct CompactDefinitionRow: View, Equatable {
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
        let colon = Text(":").fontWeight(.semibold)
        return HStack(alignment: .top, spacing: 8) {
            if let parts = entry.breakdown, !parts.isEmpty {
                // Scrolling fills the row, so the colon rides inside it to stay beside the grid.
                if scrolling { MorphemeBreakdownView(parts: parts, trailing: colon) }
                else { MorphemeBreakdownView(parts: parts).grid; colon }
            } else { Text(word).fontWeight(.semibold); colon }
        }
    }
    @ViewBuilder private var definitionBody: some View {
        VStack(alignment: .leading, spacing: 8) {
            ForEach(Array(entry.definition.senses.enumerated()), id: \.offset) { _, sense in
                VStack(alignment: .leading, spacing: 4) {
                    Text(sense.meaning) + Text(sense.note.map { " " + $0 } ?? "").foregroundColor(.yapMuted)
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
        }.font(.footnote).foregroundStyle(Color.yapMuted)
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
