import SwiftUI

struct AudioButton: View {
    @Environment(AudioPlayer.self) private var audio
    let request: AudioRequest
    @Environment(\.reviewHost!) private var host
    let reviewCount: UInt64
    var autoplay = false
    /// The web's listening cards lead with a large speaker in a ring that wobbles
    /// with the voice, beside a waveform of what has played; everywhere else the
    /// button is a 44pt icon inline with the text it plays.
    var visualizer = false
    @State private var error: String?
    @State private var playback: Task<Void, Never>?
    @State private var loading = false
    var body: some View {
        if visualizer {
            HStack(spacing: 12) {
                button.background { SpeechBlob(active: playing, level: audio.level).frame(width: 128, height: 128) }
                    .frame(width: 128, height: 128)
                Waveform(playing: playing, progress: audio.duration > 0 ? audio.currentTime / audio.duration : 0, level: audio.level)
            }
        } else {
            button
        }
    }
    private var playing: Bool { audio.isPlaying && audio.currentRequest == request }
    private var button: some View {
        // Icon-only like the web; a failure turns the icon into a red slash and
        // tapping retries, so the caption never widens the card's header row.
        Button {
            playback?.cancel()
            playback = Task { await play() }
        } label: {
            // `play` doesn't return until playback ends, so the spinner has to
            // stop at the moment the player reports this request is playing.
            Group {
                if loading && !playing { ProgressView().controlSize(visualizer ? .regular : .small) }
                else if error != nil { Image(systemName: "speaker.slash.fill").foregroundStyle(Color.yapNegativeForeground) }
                else {
                    Image(systemName: "speaker.wave.2.fill")
                        .symbolEffect(.variableColor, isActive: playing)
                }
            }
            .font(visualizer ? .title : .body)
            .frame(width: visualizer ? 72 : 44, height: visualizer ? 72 : 44)
            .contentShape(Rectangle())
        }.buttonStyle(.plain).foregroundStyle(Color.yapAccent).disabled(loading)
            .accessibilityLabel(error.map { "Play audio. \($0)" } ?? "Play audio")
        .task(id: autoplay) {
            guard autoplay, host.autoplay.reviewCount != reviewCount else { return }
            host.autoplay.reviewCount = reviewCount
            await play()
        }
        .onDisappear { playback?.cancel(); if audio.currentRequest == request { audio.stop() } }
    }
    private func play() async {
        error = nil; loading = true
        defer { loading = false }
        do { try await audio.play(request: request, accessToken: host.accessToken) }
        catch { if !Task.isCancelled { self.error = error.localizedDescription } }
    }
}

/// Two wobbly rings around the speaker, still when silent and moving with the
/// voice's loudness while it plays (the web's blob, same constants).
private struct SpeechBlob: View {
    let active: Bool
    let level: Double
    var body: some View {
        TimelineView(.animation(paused: !active)) { context in
            let t = context.date.timeIntervalSinceReferenceDate * 1000
            let intensity = active ? level : 0
            Canvas { graphics, size in
                let scale = min(size.width, size.height) / 120
                graphics.scaleBy(x: scale, y: scale)
                graphics.stroke(Self.path(t: t, intensity: intensity, base: 37, amplitude: 0.85, phase: 1.2, speed: 1.35),
                                with: .color(.yapAccent.opacity(0.35)), lineWidth: 2)
                graphics.stroke(Self.path(t: t, intensity: intensity, base: 34, amplitude: 1, phase: 0, speed: 1),
                                with: .color(.yapAccent.opacity(0.7)), style: StrokeStyle(lineWidth: 2.5, lineJoin: .round))
            }
        }.allowsHitTesting(false).accessibilityHidden(true)
    }
    private static func path(t: Double, intensity: Double, base: Double, amplitude: Double, phase: Double, speed s: Double) -> Path {
        let count = 14
        let points = (0..<count).map { i -> CGPoint in
            let i = Double(i)
            let angle = i / Double(count) * 2 * .pi + phase
            let wobble = sin(t * 0.006 * s + i * 1.7 + phase) * 0.55 + sin(-t * 0.011 * s + i * 2.9 + phase) * 0.35
                + sin(t * 0.017 * s + i * 3.7 + phase) * 0.2 + sin(-t * 0.028 * s + i * 5.3 + phase) * 0.1
            let r = base + wobble * amplitude * intensity
            return CGPoint(x: 60 + cos(angle) * r, y: 60 + sin(angle) * r)
        }
        func mid(_ a: CGPoint, _ b: CGPoint) -> CGPoint { CGPoint(x: (a.x + b.x) / 2, y: (a.y + b.y) / 2) }
        return Path { path in
            path.move(to: mid(points[0], points[1]))
            for i in 1...count { path.addQuadCurve(to: mid(points[i % count], points[(i + 1) % count]), control: points[i % count]) }
            path.closeSubpath()
        }
    }
}

/// A fixed strip of bars, one per slice of the clip: each records the loudest
/// moment heard in its slice, and bars the playhead has reached light up.
private struct Waveform: View {
    static let bars = 36
    let playing: Bool
    let progress: Double
    let level: Double
    @State private var peaks = Array(repeating: 0.0, count: bars)
    @State private var lastIndex = -1
    private var playhead: Int? { playing ? min(Self.bars - 1, Int(progress * Double(Self.bars))) : nil }
    var body: some View {
        HStack(spacing: 2) {
            ForEach(0..<Self.bars, id: \.self) { index in
                Capsule().fill(Color.yapAccent).frame(width: 3, height: max(3, peaks[index] * 40))
                    .opacity(playhead.map { index <= $0 } ?? false ? 1 : 0.2)
                    .animation(.easeOut(duration: 0.5), value: playhead.map { index <= $0 } ?? false)
            }
        }
        .frame(height: 48).accessibilityHidden(true)
        .onChange(of: level) { _, level in
            guard let index = playhead else { lastIndex = -1; return }
            // The player samples every 50ms, so a short clip can skip slices; a
            // new sample fills those too, replacing what an earlier play left there.
            if index != lastIndex {
                for slice in (index > lastIndex ? lastIndex + 1 : index)...index { peaks[slice] = level }
                lastIndex = index
            } else { peaks[index] = max(peaks[index], level) }
        }
    }
}
