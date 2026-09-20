import SwiftUI

struct AudioButton: View {
    @Environment(AudioPlayer.self) private var audio
    @Environment(AuthStore.self) private var auth
    let request: AudioRequest
    let session: YapSession
    let reviewCount: UInt64
    var autoplay = false
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
            Group {
                if loading { ProgressView().controlSize(.small) }
                else if error != nil { Image(systemName: "speaker.slash.fill").foregroundStyle(.red) }
                else {
                    Image(systemName: "speaker.wave.2.fill")
                        .symbolEffect(.variableColor, isActive: audio.isPlaying && audio.currentRequest == request)
                }
            }.frame(width: 44, height: 44).contentShape(Rectangle())
        }.buttonStyle(.plain).foregroundStyle(Color.yapAccent).disabled(loading)
            .accessibilityLabel(error.map { "Play audio. \($0)" } ?? "Play audio")
        .task(id: autoplay) {
            guard autoplay, session.lastAutoPlayReviewCount != reviewCount else { return }
            session.lastAutoPlayReviewCount = reviewCount
            await play()
        }
        .onDisappear { playback?.cancel(); if audio.currentRequest == request { audio.stop() } }
    }
    private func play() async {
        error = nil; loading = true
        defer { loading = false }
        do { try await audio.play(request: request, accessToken: auth.accessToken) }
        catch { if !Task.isCancelled { self.error = error.localizedDescription } }
    }
}
