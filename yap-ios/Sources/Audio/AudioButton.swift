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
        VStack(spacing: 8) {
            Button {
                playback?.cancel()
                playback = Task { await play() }
            } label: {
                Label(loading ? "Loading audio…" : audio.isPlaying && audio.currentRequest == request ? "Playing…" : "Play audio",
                      systemImage: "speaker.wave.2.fill").frame(minHeight: 44)
            }.buttonStyle(.bordered).disabled(loading)
            if let error { Text(error).font(.caption).foregroundStyle(.red) }
        }
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
