import SwiftUI

struct ProperNounGroupsView: View {
    let groups: [ProperNounGroup]

    var body: some View {
        if !groups.isEmpty {
            VStack(alignment: .leading, spacing: 4) {
                ForEach(Array(groups.enumerated()), id: \.offset) { _, group in
                    Text(text(group))
                        .font(.subheadline)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(12)
                        .insetSurface(cornerRadius: 8)
                }
            }
        }
    }

    private func text(_ group: ProperNounGroup) -> AttributedString {
        var result = AttributedString()
        for span in group.spans {
            var run = AttributedString(span.text)
            run.foregroundColor = span.target_language ? .primary : .secondary
            if span.target_language { run.font = .subheadline.weight(.semibold) }
            result += run
        }
        return result
    }
}
