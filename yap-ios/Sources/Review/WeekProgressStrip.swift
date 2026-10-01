import SwiftUI

/// Seven cells, Monday to Sunday: cards gained that day over a fill showing
/// time studied against the daily target; today is raised. Mirrors web's
/// `WeekProgressStrip`; Rust supplies every number.
struct WeekProgressStrip: View, Equatable {
    let week: [DayProgress]
    private static let dayLabels = ["M", "T", "W", "T", "F", "S", "S"]
    var body: some View {
        HStack(spacing: 0) {
            ForEach(Array(week.enumerated()), id: \.offset) { index, day in
                DayCell(day: day, label: Self.dayLabels[index % Self.dayLabels.count],
                        shape: UnevenRoundedRectangle(cornerRadii: .init(
                            topLeading: index == 0 ? 10 : 0, bottomLeading: index == 0 ? 10 : 0,
                            bottomTrailing: index == week.count - 1 ? 10 : 0, topTrailing: index == week.count - 1 ? 10 : 0)))
            }
        }
        .frame(maxWidth: .infinity)
    }
}

private struct DayCell: View {
    let day: DayProgress
    let label: String
    let shape: UnevenRoundedRectangle
    private var fill: Double {
        guard !day.is_future, day.target_seconds > 0 else { return 0 }
        return min(1, Double(day.seconds) / Double(day.target_seconds))
    }
    private var content: some View {
        VStack(spacing: 2) {
            Text(day.is_future ? "·" : "+\(day.new_cards + day.learned_cards + day.locked_in_cards)")
                .font(.headline.monospacedDigit())
            Text(label).font(.system(size: 10)).textCase(.uppercase).tracking(0.5).opacity(0.7)
        }
        .frame(maxWidth: .infinity, minHeight: 48)
    }
    var body: some View {
        // Like the web: the time studied fills the cell from the bottom in the
        // foreground color, and a second copy of the text in the background
        // color shows through wherever the fill has reached.
        let cell = day.is_today ? AnyShape(RoundedRectangle(cornerRadius: 10)) : AnyShape(shape)
        content.foregroundStyle(day.is_future ? Color.yapMuted.opacity(0.6) : Color.yapText)
            .overlay {
                if fill > 0 {
                    content.foregroundStyle(Color.yapBackground)
                        .background(Color.yapText.opacity(fill >= 1 ? 1 : 0.9))
                        .mask(alignment: .bottom) {
                            GeometryReader { geometry in
                                Rectangle().frame(height: geometry.size.height * fill).frame(maxHeight: .infinity, alignment: .bottom)
                            }
                        }
                }
            }
            .background {
                if day.is_future { Color.yapMutedSurface.opacity(0.3) }
                else { Color.yapBackground.opacity(0.25).overlay(Color.yapText.opacity(day.seconds > 0 ? 0.1 : 0)) }
            }
            .clipShape(cell)
            .overlay { if day.is_today { cell.stroke(Color.yapBorder) } }
            .shadow(color: .black.opacity(day.is_today ? 0.15 : 0), radius: 8, y: 4)
            .scaleEffect(day.is_today ? 1.1 : 1)
            .zIndex(day.is_today ? 1 : 0)
            .accessibilityElement(children: .combine)
    }
}
