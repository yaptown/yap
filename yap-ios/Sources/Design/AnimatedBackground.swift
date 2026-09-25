import SwiftUI
import Metal

/// One clock for the whole app: grading changes its energy, not its identity.
@MainActor @Observable final class BackgroundController {
    private(set) var time = 0.0
    private(set) var speed = 0.09
    private(set) var colors: [Double] = []
    private(set) var settled = false
    private var targetColors: [Double] = []
    private var lastFrame: Date?

    func bump(_ multiplier: Double = 3) {
        speed = 0.03 * multiplier
        if settled { lastFrame = nil }
        settled = false
    }

    func setPalette(_ palette: BackgroundPalette) {
        targetColors = palette.colors
        if colors.isEmpty { colors = targetColors }
        if settled { lastFrame = nil }
        settled = false
    }

    func resetClock() { lastFrame = nil }

    func advance(to date: Date) {
        defer { lastFrame = date }
        guard let lastFrame else { return }
        let delta = max(0, date.timeIntervalSince(lastFrame))
        speed *= exp(-0.5 * delta)
        time += delta * 1000 * speed
        let alpha = 1 - exp(-18 * delta)
        colors = zip(colors, targetColors).map { $0 + alpha * ($1 - $0) }
        // Same stopping rule as the web worker: below this speed the rest of the
        // decay drifts the bands by under a pixel, so stop drawing instead.
        if speed < 0.005 && zip(colors, targetColors).allSatisfy({ abs($0 - $1) <= 0.001 }) {
            speed = 0
            settled = true
        }
    }
}

struct AnimatedBackground: View {
    @Environment(BackgroundController.self) private var controller
    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.scenePhase) private var scenePhase
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.colorSchemeContrast) private var contrast
    @Environment(\.accessibilityReduceTransparency) private var reduceTransparency
    @AppStorage("yap-animated-background") private var enabled = true
    // Missing Metal support/library leaves the shared static color, not black.
    private static let shaderAvailable: Bool = {
        guard let device = MTLCreateSystemDefaultDevice(), let library = device.makeDefaultLibrary() else { return false }
        return library.makeFunction(name: "yapBackground") != nil
    }()
    private static let lightPalette = background_palette(theme: .Light)
    private static let darkPalette = background_palette(theme: .Dark)
    private var dark: Bool { colorScheme == .dark }
    private var palette: BackgroundPalette { dark ? Self.darkPalette : Self.lightPalette }
    private var animated: Bool { enabled && !reduceMotion && contrast != .increased && !reduceTransparency && Self.shaderAvailable }
    private var active: Bool { animated && scenePhase == .active }

    var body: some View {
        let palette = palette
        ZStack {
            Color(.sRGB, red: palette.fallback.r, green: palette.fallback.g, blue: palette.fallback.b, opacity: 1)
            if animated {
                GeometryReader { geometry in
                    // Use the web drawing-buffer dimensions for a 1x offscreen group,
                    // then stretch it to fill. GPU backing allocation is managed by SwiftUI.
                    let scale = min(UIScreen.main.scale, 1.5) * (geometry.size.width < 768 ? 0.35 : 0.75)
                    let size = CGSize(width: max(1, floor(geometry.size.width * scale)), height: max(1, floor(geometry.size.height * scale)))
                    TimelineView(.animation(minimumInterval: 1.0 / 30, paused: !active || controller.settled)) { timeline in
                        Rectangle().fill(.white)
                            .frame(width: size.width, height: size.height)
                            .colorEffect(ShaderLibrary.yapBackground(
                                .float2(size), .float(controller.time), .float2(0.5, 0.4),
                                .float(dark ? 1 : 0), .float(0), .float(Float(palette.num_bands)),
                                .floatArray((controller.colors.isEmpty ? palette.colors : controller.colors).map(Float.init))))
                            .drawingGroup(opaque: true, colorMode: .nonLinear)
                            .environment(\.displayScale, 1)
                            .scaleEffect(x: geometry.size.width / size.width, y: geometry.size.height / size.height, anchor: .topLeading)
                            .frame(width: geometry.size.width, height: geometry.size.height, alignment: .topLeading)
                            .onChange(of: timeline.date) { _, date in if active { controller.advance(to: date) } }
                    }
                    textures
                }
            }
        }
        .ignoresSafeArea()
        .allowsHitTesting(false)
        .accessibilityHidden(true)
        .onChange(of: dark, initial: true) { _, _ in controller.setPalette(palette) }
        .onChange(of: active) { _, _ in controller.resetClock() }
    }

    private var textures: some View {
        ZStack {
            texture(dark ? "BackgroundFog" : "BackgroundNoise2", opacity: dark ? 0.7 : 0.3)
            texture("BackgroundNoise", opacity: 0.2)
        }
    }

    private func texture(_ name: String, opacity: Double) -> some View {
        GeometryReader { geometry in
            Image(name).resizable().scaledToFill()
                .frame(width: geometry.size.width, height: geometry.size.height).clipped()
                .colorInvertIf(dark)
                .blendMode(dark ? .multiply : .screen)
                .opacity(opacity)
        }
    }
}

private extension View {
    @ViewBuilder func colorInvertIf(_ inverted: Bool) -> some View {
        if inverted { colorInvert() } else { self }
    }
}
