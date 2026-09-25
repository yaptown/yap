import SwiftUI

/// A first-review guide line, flanked by the web's hand-drawn arrows pointing
/// at what it describes: down at the card or the grade buttons, up at the card
/// from below it.
struct TutorialHint: View {
    let text: String
    let pointing: PlayfulArrow.Direction
    var arrowSize: CGFloat = 70
    var body: some View {
        HStack(alignment: pointing == .down ? .top : .bottom, spacing: 4) {
            PlayfulArrow(direction: pointing, flipStart: pointing == .down, size: arrowSize)
            Text(text).font(.title2.weight(.semibold)).multilineTextAlignment(.center)
            PlayfulArrow(direction: pointing, flipStart: pointing == .up, size: arrowSize)
        }
        .foregroundStyle(Tokens.palette.muted_foreground.color)
        .frame(maxWidth: .infinity)
    }
}

extension TutorialHint {
    init(prompt: TutorialPrompt) {
        self.init(text: prompt.before + (prompt.target ?? "") + prompt.after, pointing: .down)
    }
}

/// The web's `PlayfulArrow`: a loop and then a head, drawn left to right in a
/// 168×81 box and turned to point up or down.
struct PlayfulArrow: View {
    enum Direction { case up, down }
    let direction: Direction
    var flipStart = false
    var size: CGFloat = 70
    var body: some View {
        let length = size * 0.75
        let thickness = length * 81 / 168
        ArrowPath()
            .stroke(style: StrokeStyle(lineWidth: 10 * length / 168, lineCap: .round, lineJoin: .round))
            .frame(width: length, height: thickness)
            .scaleEffect(x: 1, y: flipStart ? -1 : 1)
            .rotationEffect(.degrees(direction == .up ? -90 : 90))
            .frame(width: length, height: size)
            .opacity(0.5)
            .accessibilityHidden(true)
    }
}

private struct ArrowPath: Shape {
    func path(in rect: CGRect) -> Path {
        let sx = rect.width / 168, sy = rect.height / 81
        func p(_ x: CGFloat, _ y: CGFloat) -> CGPoint { CGPoint(x: rect.minX + x * sx, y: rect.minY + y * sy) }
        var path = Path()
        path.move(to: p(0, 81))
        path.addCurve(to: p(95, 36), control1: p(0, -35), control2: p(85, 2))
        path.addCurve(to: p(56, 53), control1: p(102, 71), control2: p(58, 72))
        path.addCurve(to: p(155, 32), control1: p(51, 8), control2: p(110, 34))
        path.move(to: p(137, 14))
        path.addLine(to: p(160, 32))
        path.addLine(to: p(137, 50))
        return path
    }
}
