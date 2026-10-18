import SwiftUI

/// `text` with readings set over it. Rust sends no segments when there are none
/// to show, and their bases spell exactly the text they replace. Font, tint and
/// underline come from the environment like a plain `Text`; the readings use
/// `rubyFont` in the muted color.
struct RubyText: View {
    let text: String
    let segments: [RubySegment]
    let rubyFont: Font
    /// For one word of a sentence: keep an empty reading line over plain text,
    /// so it sits level with ruby on the words beside it, and don't wrap.
    /// Otherwise the text wraps like a plain `Text`.
    var reserve = false
    var body: some View {
        if segments.isEmpty && !reserve {
            Text(text)
        } else if !reserve {
            RubyFlow(segments: segments, rubyFont: rubyFont)
        } else {
            HStack(alignment: .lastTextBaseline, spacing: 0) {
                ForEach(Array((segments.isEmpty ? [RubySegment(base: text, ruby: nil)] : segments).enumerated()), id: \.offset) { _, segment in
                    RubyCell(segment: segment, rubyFont: rubyFont)
                }
            }
        }
    }
}

/// A sentence with readings that wraps like text: between segments, and
/// anywhere inside plain runs (these are Japanese and Mandarin, which break
/// between characters).
struct RubyFlow: View {
    let segments: [RubySegment]
    let rubyFont: Font
    var body: some View {
        let cells = segments.flatMap { segment in
            segment.ruby == nil ? segment.base.map { RubySegment(base: String($0), ruby: nil) } : [segment]
        }
        SentenceFlow(spacing: 0) {
            ForEach(Array(cells.enumerated()), id: \.offset) { _, segment in
                RubyCell(segment: segment, rubyFont: rubyFont)
            }
        }
    }
}

private struct RubyCell: View {
    let segment: RubySegment
    let rubyFont: Font
    var body: some View {
        VStack(spacing: 0) {
            Text(segment.ruby ?? " ").font(rubyFont).foregroundStyle(Color.yapMuted).underline(false)
                .opacity(segment.ruby == nil ? 0 : 1)
            Text(segment.base)
        }.fixedSize()
    }
}
