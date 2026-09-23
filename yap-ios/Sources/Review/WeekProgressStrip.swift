import SwiftUI

/// Seven cells, Monday to Sunday: cards gained that day over a fill showing
/// time studied against the daily target; today is raised. Mirrors web's
/// `WeekProgressStrip`; Rust supplies every number.
struct WeekProgressStrip: View {
    let week: [DayProgress]
    private static let dayLabels = ["M", "T", "W", "T", "F", "S", "S"]
    var body: some View {
        HStack(spacing: 2) {
            ForEach(Array(week.enumerated()), id: \.offset) { index, day in
                DayCell(day: day, label: Self.dayLabels[index % Self.dayLabels.count])
            }
        }
        .clipShape(RoundedRectangle(cornerRadius: 10))
        .frame(maxWidth: .infinity)
    }
}

private struct DayCell: View {
    let day: DayProgress
    let label: String
    private var fill: Double {
        guard !day.is_future, day.target_seconds > 0 else { return 0 }
        return min(1, Double(day.seconds) / Double(day.target_seconds))
    }
    var body: some View {
        VStack(spacing: 2) {
            Text(day.is_future ? "·" : "+\(day.new_cards + day.learned_cards + day.locked_in_cards)")
                .font(.headline.monospacedDigit())
            Text(label).font(.system(size: 10)).textCase(.uppercase).opacity(0.7)
        }
        .frame(maxWidth: .infinity, minHeight: 48)
        .background(alignment: .bottom) {
            GeometryReader { geometry in
                Color.yapPositiveSurface
                    .frame(height: geometry.size.height * fill)
                    .frame(maxHeight: .infinity, alignment: .bottom)
            }
        }
        .background(day.is_future ? Color.secondary.opacity(0.08) : Color.primary.opacity(day.seconds > 0 ? 0.08 : 0.04))
        .overlay {
            if day.is_today {
                RoundedRectangle(cornerRadius: 8).stroke(Color.secondary.opacity(0.4))
            }
        }
        .scaleEffect(day.is_today ? 1.08 : 1)
        .zIndex(day.is_today ? 1 : 0)
        .accessibilityElement(children: .combine)
    }
}
