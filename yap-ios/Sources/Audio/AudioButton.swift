import SwiftUI

struct AudioButton: View {
    @Environment(AudioPlayer.self) private var audio
    let request: AudioRequest
    @Environment(\.reviewHost!) private var host
    let reviewCount: UInt64
    var autoplay = false
    /// The web's dictation card leads with a large ringed speaker; everywhere
    /// else the button is a 44pt icon inline with the text it plays.
    var hero = false
    @State private var error: String?
    @State private var playback: Task<Void, Never>?
    @State private var loading = false
    var body: some View {
        // Icon-only like the web; a failure turns the icon into a red slash and
        // tapping retries, so the caption never widens the card's header row.
        Button {
            playback?.cancel()
            playback = Task { await play() }
        } label: {
            // `play` doesn't return until playback ends, so the spinner has to
            // stop at the moment the player reports this request is playing.
            let playing = audio.isPlaying && audio.currentRequest == request
            Group {
                if loading && !playing { ProgressView().controlSize(hero ? .regular : .small) }
                else if error != nil { Image(systemName: "speaker.slash.fill").foregroundStyle(Color.yapNegativeForeground) }
                else {
                    Image(systemName: "speaker.wave.2.fill")
                        .symbolEffect(.variableColor, isActive: playing)
                }
            }
            .font(hero ? .title : .body)
            .frame(width: hero ? 72 : 44, height: hero ? 72 : 44)
            .background { if hero { Circle().strokeBorder(Color.yapAccent.opacity(0.6), lineWidth: 2) } }
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
