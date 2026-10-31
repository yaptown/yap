import SwiftUI
import Charts

struct ReviewTotalChart: View {
    let view: ReviewTotalView
    @State private var day: Int?
    private var selected: ReviewTotalPoint? {
        guard let day else { return nil }
        // The step holds each total until the next study day.
        return view.points.last { Int($0.day) <= day } ?? view.points.first
    }
    var body: some View {
        StudyCard {
            Text(view.title).font(.title2.bold()).foregroundStyle(Color.yapText)
            Chart {
                ForEach(view.points, id: \.day) { point in
                    LineMark(x: .value("day", Int(point.day)), y: .value("total", Int(point.total)))
                        .interpolationMethod(.stepEnd)
                        .foregroundStyle(Tokens.palette.chart_1.color).lineStyle(StrokeStyle(lineWidth: 2))
                }
                if let selected {
                    PointMark(x: .value("day", Int(selected.day)), y: .value("total", Int(selected.total)))
                        .foregroundStyle(Tokens.palette.chart_1.color)
                }
            }
                .chartXScale(domain: 0...Int(view.points.last?.day ?? 0))
                .chartXAxis {
                    AxisMarks(values: view.ticks.map { Int($0.day) }) { value in
                        AxisValueLabel {
                            if let day = value.as(Int.self), let tick = view.ticks.first(where: { Int($0.day) == day }) { Text(tick.label).fixedSize() }
                        }
                    }
                }
                .chartYAxis { AxisMarks(position: .leading) { _ in AxisGridLine(stroke: StrokeStyle(lineWidth: 0.5)).foregroundStyle(.secondary.opacity(0.25)); AxisValueLabel() } }
                .chartXSelection(value: $day).frame(height: 200)
            if let selected { Text(selected.label).font(.caption).foregroundStyle(Color.yapMuted) }
        }
    }
}
