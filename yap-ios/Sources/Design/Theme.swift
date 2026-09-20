import SwiftUI
import CoreText

extension Color {
    static let yapOnAccent = Color(uiColor: UIColor { traits in
        traits.userInterfaceStyle == .dark ? .black : .white
    })
    // sRGB conversions of the web's OKLCH hue-328 palette. Dynamic for contrast.
    static let yapAccent = Color(uiColor: UIColor { traits in
        traits.userInterfaceStyle == .dark
            ? UIColor(red: 0.9462, green: 0.6264, blue: 0.9292, alpha: 1)
            : UIColor(red: 0.573, green: 0.220, blue: 0.541, alpha: 1)
    })
    static let yapText = Color(uiColor: UIColor { traits in
        traits.userInterfaceStyle == .dark
            ? UIColor(red: 0.9409, green: 0.8507, blue: 0.9331, alpha: 1)
            : UIColor.label
    })
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
