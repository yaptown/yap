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
    static let yapOnAccent = Color(uiColor: UIColor { traits in
        traits.userInterfaceStyle == .dark ? .black : .white
    })
    @MainActor static let yapAccent = Tokens.palette.accent_foreground.color
    @MainActor static let yapText = Tokens.palette.foreground.color
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

/// The one card surface, shared by study and browse screens in both schemes.
private struct CardSurface: ViewModifier {
    @Environment(\.accessibilityReduceTransparency) private var reduceTransparency
    func body(content: Content) -> some View {
        content
            .background(Tokens.palette.card.color.opacity(0.18), in: RoundedRectangle(cornerRadius: 20))
            .background {
                RoundedRectangle(cornerRadius: 20).fill(.ultraThinMaterial)
                    .opacity(reduceTransparency ? 1 : 0.65)
            }
            .overlay { RoundedRectangle(cornerRadius: 20).strokeBorder(Color(uiColor: .separator).opacity(0.5)) }
    }
}

extension View {
    func cardSurface() -> some View { modifier(CardSurface()) }
}

struct StudyCard<Content: View>: View {
    var alignment: HorizontalAlignment = .leading
    var spacing: CGFloat = 12
    @ViewBuilder var content: Content
    var body: some View {
        VStack(alignment: alignment, spacing: spacing) { content }
            .frame(maxWidth: .infinity, alignment: Alignment(horizontal: alignment, vertical: .center)).padding(16)
            .cardSurface()
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
