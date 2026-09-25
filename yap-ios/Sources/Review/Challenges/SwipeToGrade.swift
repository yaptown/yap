import SwiftUI

extension View {
    /// The web's card swipe: drag right for Remembered, left for Again. The card
    /// tilts with the drag under a red or green wash and a stamped label; past
    /// 100pt it flies off and grades, otherwise it springs back.
    func swipeToGrade(enabled: Bool, againLabel: String, rememberedLabel: String, rate: @escaping (Rating) -> Void) -> some View {
        modifier(SwipeToGrade(enabled: enabled, againLabel: againLabel, rememberedLabel: rememberedLabel, rate: rate))
    }
}

private struct SwipeToGrade: ViewModifier {
    let enabled: Bool
    let againLabel: String
    let rememberedLabel: String
    let rate: (Rating) -> Void
    @State private var offset: CGFloat = 0
    /// Set once a drag proves horizontal, so vertical scrolling keeps working.
    @State private var horizontal: Bool?

    func body(content: Content) -> some View {
        let progress = max(-1, min(1, offset / 200))
        content
            .environment(\.cardGlass, offset == 0)
            .overlay {
                ZStack {
                    Color.yapNegative.opacity(0.2 * max(0, -progress))
                    Color.yapPositive.opacity(0.2 * max(0, progress))
                    stamp(againLabel, color: .yapNegativeForeground, angle: -30, alignment: .topLeading).opacity(max(0, -progress))
                    stamp(rememberedLabel, color: .yapPositiveForeground, angle: 30, alignment: .topTrailing).opacity(max(0, progress))
                }
                .clipShape(RoundedRectangle(cornerRadius: 20, style: .continuous))
                .allowsHitTesting(false)
            }
            .offset(x: offset)
            .rotationEffect(.degrees(30 * progress))
            // If the same card comes straight back, swiping re-enables on this
            // view once grading finishes; bring the card back to rest then.
            .onChange(of: enabled) { _, enabled in if enabled { offset = 0 } }
            .simultaneousGesture(
                DragGesture(minimumDistance: 12)
                    .onChanged { value in
                        if horizontal == nil { horizontal = abs(value.translation.width) > abs(value.translation.height) }
                        if horizontal == true { offset = value.translation.width }
                    }
                    .onEnded { value in
                        defer { horizontal = nil }
                        guard horizontal == true else { return }
                        let dx = value.translation.width, vx = value.velocity.width
                        if enabled, dx > 100, vx > 0 { fling(to: 300, rating: .Remembered) }
                        else if enabled, dx < -100, vx < 0 { fling(to: -300, rating: .Again) }
                        else { withAnimation(.spring(response: 0.35, dampingFraction: 0.6)) { offset = 0 } }
                    }
            )
    }

    private func fling(to target: CGFloat, rating: Rating) {
        // Stay off screen: grading replaces this view with the next card.
        withAnimation(.easeIn(duration: 0.2)) { offset = target } completion: { rate(rating) }
    }

    private func stamp(_ label: String, color: Color, angle: Double, alignment: Alignment) -> some View {
        Text(label.uppercased()).font(.title2.bold()).foregroundStyle(color)
            .rotationEffect(.degrees(angle)).padding(32)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: alignment)
    }
}
