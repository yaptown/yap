import SwiftUI
import CoreText

extension DynamicColor {
    var color: Color {
        Color(uiColor: UIColor { traits in
            let rgba = traits.userInterfaceStyle == .dark ? dark : light
            return UIColor(red: rgba.r, green: rgba.g, blue: rgba.b, alpha: rgba.a)
        })
    }
}

@MainActor enum Tokens {
    static let palette = design_palette()
}

extension Color {
    // The web's accent is its primary token: deep plum in light, near-white pink in dark.
    @MainActor static let yapAccent = Tokens.palette.primary.color
    @MainActor static let yapOnAccent = Tokens.palette.primary_foreground.color
    /// Saturated enough to read as "on" for switches, which near-white would not be in dark mode.
    @MainActor static let yapSwitchTint = Tokens.palette.accent_foreground.color
    @MainActor static let yapText = Tokens.palette.foreground.color
    @MainActor static let yapMuted = Tokens.palette.muted_foreground.color
    @MainActor static let yapDestructiveForeground = Tokens.palette.destructive_foreground.color
    @MainActor static let yapPositiveForeground = Tokens.palette.positive_foreground.color
    @MainActor static let yapPositive = Tokens.palette.positive.color
    @MainActor static let yapPositiveSurface = Tokens.palette.positive_surface.color
    @MainActor static let yapPositiveBorder = Tokens.palette.positive_border.color
    @MainActor static let yapPositiveField = Tokens.palette.positive_field.color
    @MainActor static let yapCautionForeground = Tokens.palette.caution_foreground.color
    @MainActor static let yapCaution = Tokens.palette.caution.color
    @MainActor static let yapCautionSurface = Tokens.palette.caution_surface.color
    @MainActor static let yapCautionBorder = Tokens.palette.caution_border.color
    @MainActor static let yapCautionField = Tokens.palette.caution_field.color
    @MainActor static let yapWarningForeground = Tokens.palette.warning_foreground.color
    @MainActor static let yapWarning = Tokens.palette.warning.color
    @MainActor static let yapWarningSurface = Tokens.palette.warning_surface.color
    @MainActor static let yapWarningBorder = Tokens.palette.warning_border.color
    @MainActor static let yapWarningField = Tokens.palette.warning_field.color
    @MainActor static let yapNegativeForeground = Tokens.palette.negative_foreground.color
    @MainActor static let yapNegative = Tokens.palette.negative.color
    @MainActor static let yapNegativeSurface = Tokens.palette.negative_surface.color
    @MainActor static let yapNegativeBorder = Tokens.palette.negative_border.color
    @MainActor static let yapNegativeField = Tokens.palette.negative_field.color
    @MainActor static let yapInfoForeground = Tokens.palette.info_foreground.color
    @MainActor static let yapInfo = Tokens.palette.info.color
    @MainActor static let yapInfoSurface = Tokens.palette.info_surface.color
    @MainActor static let yapInfoBorder = Tokens.palette.info_border.color
    @MainActor static let yapInfoField = Tokens.palette.info_field.color
}

/// The one card surface, shared by study and browse screens in both schemes:
/// Liquid Glass on iOS 26, a translucent material before it.
private struct CardSurface: ViewModifier {
    @Environment(\.accessibilityReduceTransparency) private var reduceTransparency
    @Environment(\.cardGlass) private var glass
    func body(content: Content) -> some View {
        // Only the surface switches, so the card's contents keep their identity.
        content.background { surface }
    }
    @ViewBuilder private var surface: some View {
        let shape = RoundedRectangle(cornerRadius: 20, style: .continuous)
        if #available(iOS 26, *), glass {
            Color.clear.glassEffect(.regular.tint(Tokens.palette.card.color.opacity(0.12)), in: shape)
        } else {
            shape.fill(Tokens.palette.card.color.opacity(0.18))
                .background { shape.fill(.ultraThinMaterial).opacity(reduceTransparency ? 1 : 0.65) }
                .overlay { shape.strokeBorder(Color(uiColor: .separator).opacity(0.5)) }
        }
    }
}

extension EnvironmentValues {
    /// Glass doesn't follow a rotated view, so a card being swiped turns it off.
    @Entry var cardGlass = true
}

extension View {
    func cardSurface() -> some View { modifier(CardSurface()) }

    /// A lighter surface than a card, for secondary panels that shouldn't compete
    /// with the card beside them (the web's `light` card variant).
    func quietSurface(cornerRadius: CGFloat = 20) -> some View {
        let shape = RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
        return background { shape.fill(.ultraThinMaterial).opacity(0.5) }
            .overlay { shape.strokeBorder(Color(uiColor: .separator).opacity(0.5)) }
    }

    /// A box inside a card (a sense, a word tile): a translucent wash with a
    /// hairline, so the glass behind it still shows through.
    func insetSurface(cornerRadius: CGFloat = 12, fill: Color = Color(uiColor: .systemBackground).opacity(0.35)) -> some View {
        let shape = RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
        return background(fill, in: shape)
            .overlay { shape.strokeBorder(Color(uiColor: .separator).opacity(0.4)) }
    }

    /// Pins controls below a scroll view without a hard-edged backdrop, so the
    /// animated background runs unbroken to the bottom of the screen. On iOS 26
    /// the system's soft scroll-edge effect fades content scrolling beneath them.
    @ViewBuilder func bottomBar<Bar: View>(@ViewBuilder _ bar: () -> Bar) -> some View {
        if #available(iOS 26, *) { safeAreaBar(edge: .bottom, spacing: 0, content: bar) }
        else { safeAreaInset(edge: .bottom, spacing: 0, content: bar) }
    }
}

struct StudyCard<Content: View>: View {
    var alignment: HorizontalAlignment = .leading
    var spacing: CGFloat = 12
    /// The web's `<Card animate>`: study-step cards fade and grow in as they appear.
    var animated = false
    @ViewBuilder var content: Content
    var body: some View {
        VStack(alignment: alignment, spacing: spacing) { content }
            .frame(maxWidth: .infinity, alignment: Alignment(horizontal: alignment, vertical: .center)).padding(16)
            .cardSurface()
            .modifier(Entrance(scale: animated ? 0.95 : 1, rise: 0, duration: animated ? 0.3 : 0))
    }
}

extension View {
    /// The web's `fade-in` family: fade in while rising 10pt.
    func fadeIn(duration: Double = 0.6, delay: Double = 0) -> some View {
        modifier(Entrance(scale: 1, rise: 10, duration: duration, delay: delay))
    }
}

/// Plays once when the view appears; Reduce Motion keeps only the fade.
private struct Entrance: ViewModifier {
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    let scale: CGFloat
    let rise: CGFloat
    let duration: Double
    var delay: Double = 0
    @State private var shown = false
    func body(content: Content) -> some View {
        let settled = shown || duration == 0
        content
            .opacity(settled ? 1 : 0)
            .scaleEffect(settled || reduceMotion ? 1 : scale)
            .offset(y: settled || reduceMotion ? 0 : rise)
            .onAppear {
                guard duration > 0 else { return }
                withAnimation(.easeOut(duration: duration).delay(delay)) { shown = true }
            }
    }
}

@MainActor enum Theme {
    // UIFont can return a descriptor even when the simulator's on-demand emoji
    // asset is missing. Avoid displaying replacement glyphs in that case.
    static let emojiFontAvailable: Bool = {
        let font = CTFontCreateWithName("AppleColorEmoji" as CFString, 28, nil)
        guard let url = CTFontCopyAttribute(font, kCTFontURLAttribute) as? URL else { return false }
        return FileManager.default.fileExists(atPath: url.path)
    }()
}

struct ReviewBadge: View {
    let text: String
    var body: some View {
        Text(text).font(.caption.weight(.semibold))
            .foregroundStyle(Color.yapAccent).padding(.horizontal, 8).padding(.vertical, 4)
            .background(Color.yapAccent.opacity(0.12), in: Capsule())
    }
}

extension ToolbarItem {
    /// Plain content in the bar (a wordmark, not a control) without iOS 26's glass capsule.
    @ToolbarContentBuilder func hidingSharedBackground() -> some ToolbarContent {
        if #available(iOS 26, *) { sharedBackgroundVisibility(.hidden) } else { self }
    }
}
