import SwiftUI

/// Again and Remembered as one split bar, joined like the web's pair and drawn
/// with the web button fills: translucent destructive and primary over a blur.
struct GradeButtons: View {
    @Environment(\.colorScheme) private var scheme
    let againLabel: String
    let rememberedLabel: String
    let rate: (Rating) -> Void
    var body: some View {
        HStack(spacing: 0) {
            Button(againLabel) { rate(.Again) }
                .buttonStyle(GradeHalfStyle(fill: Tokens.palette.destructive.color.opacity(scheme == .dark ? 0.6 : 0.85),
                                            text: .yapDestructiveForeground))
                .keyboardShortcut(.leftArrow, modifiers: [])
            Button(rememberedLabel) { rate(.Remembered) }
                .buttonStyle(GradeHalfStyle(fill: Tokens.palette.primary.color.opacity(0.85),
                                            text: Tokens.palette.primary_foreground.color))
                .keyboardShortcut(.rightArrow, modifiers: [])
        }
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
    }
}

private struct GradeHalfStyle: ButtonStyle {
    let fill: Color
    let text: Color
    @Environment(\.isEnabled) private var isEnabled
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.headline).foregroundStyle(text)
            .frame(maxWidth: .infinity, minHeight: 56)
            .background(fill)
            .brightness(configuration.isPressed ? 0.08 : 0)
            .opacity(isEnabled ? 1 : 0.5)
    }
}
