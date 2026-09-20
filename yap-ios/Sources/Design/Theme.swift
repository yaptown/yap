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
}

struct StudyCard<Content: View>: View {
    @ViewBuilder var content: Content
    var body: some View {
        VStack(alignment: .leading, spacing: 12) { content }
            .frame(maxWidth: .infinity, alignment: .leading).padding(16)
            .background(Color(uiColor: .secondarySystemGroupedBackground), in: RoundedRectangle(cornerRadius: 20))
            .overlay { RoundedRectangle(cornerRadius: 20).strokeBorder(Color(uiColor: .separator).opacity(0.5)) }
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
