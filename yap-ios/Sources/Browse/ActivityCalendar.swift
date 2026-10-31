import SwiftUI

/// The web's ActivityCalendar: a year of days, shaded by how much the learner reviewed.
struct ActivityCalendar: View {
    let view: ActivityView
    private let cell: CGFloat = 11
    private let gap: CGFloat = 3
    var body: some View {
        StudyCard(spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text(view.title).font(.title2.bold()).foregroundStyle(Color.yapText)
                Text(view.summary).font(.subheadline).foregroundStyle(Color.yapMuted)
            }
            HStack(alignment: .top, spacing: 4) {
                VStack(alignment: .leading, spacing: gap) {
                    Color.clear.frame(height: cell)
                    ForEach(Array(view.weekdays.enumerated()), id: \.offset) { _, label in
                        Text(label).frame(height: cell)
                    }
                }.accessibilityHidden(true)
                // Narrow screens scroll; start at the most recent weeks.
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(alignment: .top, spacing: gap) {
                        ForEach(Array(view.weeks.enumerated()), id: \.offset) { _, week in
                            VStack(spacing: gap) {
                                Text(week.month_label ?? "").fixedSize()
                                    .frame(width: cell, height: cell, alignment: .leading)
                                    .accessibilityHidden(true)
                                ForEach(Array(week.days.enumerated()), id: \.offset) { _, day in
                                    if let day {
                                        square(level: day.level).accessibilityLabel(day.label)
                                    } else {
                                        Color.clear.frame(width: cell, height: cell)
                                    }
                                }
                            }
                        }
                    }.padding(.trailing, gap)
                }.defaultScrollAnchor(.trailing)
            }.font(.system(size: 10)).foregroundStyle(Color.yapMuted)
            HStack(spacing: gap) {
                Text(view.less_label).padding(.trailing, 4)
                ForEach(0..<5) { level in square(level: UInt8(level)) }
                Text(view.more_label).padding(.leading, 4)
            }.font(.caption).foregroundStyle(Color.yapMuted)
                .frame(maxWidth: .infinity, alignment: .trailing).accessibilityHidden(true)
        }
    }
    private func square(level: UInt8) -> some View {
        // Level 0 is a faint neutral; 1–4 deepen the chart color, like GitHub's greens.
        let color = level == 0 ? Color.yapMuted.opacity(0.15)
            : Tokens.palette.chart_1.color.opacity([0.35, 0.55, 0.78, 1][Int(level) - 1])
        return RoundedRectangle(cornerRadius: 2).fill(color).frame(width: cell, height: cell)
    }
}
