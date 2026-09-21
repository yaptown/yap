import SwiftUI
import AVKit
import CryptoKit

/// Bytes/cache and manifest policy stay in Rust. This view owns only the native
/// player and its temporary playable file. Like the web, replay plays the whole
/// thought from zero; critical_start_ms is only the initial poster frame.
struct VideoClipView: View {
    @Environment(AudioPlayer.self) private var audio
    let language: Language
    let text: String
    let media: ReviewMedia
    let reviewCount: UInt64
    var autoplay = false
    var maskedSentence: String?
    @Binding var available: Bool?
    @State private var movie: MovieMetadataBasic?
    @State private var player: AVPlayer?
    @State private var cues: [ClipSubtitleCue] = []
    @State private var timeMs = 0.0
    @State private var playing = false
    @State private var playbackFailed = false
    private var cue: ClipSubtitleCue? { cues.first { timeMs >= Double($0.at_ms) && timeMs < Double($0.until_ms) } }
    var body: some View {
        VStack(spacing: 0) {
            if let player {
                VStack(spacing: 8) {
                    if let movie {
                        HStack(spacing: 10) {
                            MoviePoster(poster: media.moviePoster, id: movie.id, title: movie.title)
                            Text(movie.title).font(.subheadline.weight(.semibold))
                            if let year = movie.year { Text(String(year)).font(.caption).foregroundStyle(.secondary) }
                        }.frame(maxWidth: .infinity, alignment: .leading)
                    }
                    ClipSurface(player: player)
                        .overlay {
                            Button {
                                if playing { player.pause() } else { play(player) }
                            } label: {
                                ZStack {
                                    Color.clear
                                    if !playing { Image(systemName: "play.circle.fill").font(.largeTitle).foregroundStyle(.white) }
                                }.contentShape(Rectangle())
                            }.buttonStyle(.plain).accessibilityLabel(playing ? "Pause clip" : "Play clip")
                        }
                        .aspectRatio(16 / 9, contentMode: .fit)
                        .overlay(alignment: .bottom) {
                            if playing, let cue {
                                Text(cue.role == "sentence" ? maskedSentence ?? cue.text : cue.text)
                                    .font(.callout.weight(.medium))
                                    .foregroundStyle(cue.role == "sentence" ? .yellow : .white)
                                    .padding(6).background(.black.opacity(0.7), in: RoundedRectangle(cornerRadius: 6))
                                    .padding(.bottom, 32).allowsHitTesting(false)
                            }
                        }.clipShape(RoundedRectangle(cornerRadius: 12))
                    Button("Replay clip", systemImage: "play.fill") { play(player) }
                }
            }
        }
        .task(id: text) { await watch() }
        .task(id: autoplay && player != nil) {
            guard autoplay, let player else { return }
            // Let grading sounds finish, just as on the web.
            while audio.isPlaying || audio.effectPlaying {
                do { try await Task.sleep(for: .milliseconds(50)) } catch { return }
            }
            guard !Task.isCancelled, media.claimAutoplay(reviewCount) else { return }
            play(player)
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            if DebugHarness.shared.command == "clip-play", let player { play(player) }
            if DebugHarness.shared.command == "clip-fail" { playbackFailed = true }
            if DebugHarness.shared.command == "status" { DebugHarness.log("clip available=\(available == true) playing=\(playing) subtitle=\(cue?.text ?? "none")") }
        }
        #endif
    }
    private func play(_ player: AVPlayer) {
        do { try audio.playVideo(player) }
        catch { playbackFailed = true }
    }
    private func watch() async {
        var file: URL?
        defer {
            if let player { audio.stopVideo(player); player.replaceCurrentItem(with: nil) }
            player = nil; playing = false
            if let file { try? FileManager.default.removeItem(at: file) }
        }
        var version: UInt32?
        while !Task.isCancelled {
            let latest = get_clip_manifest_version()
            if player == nil && version != latest {
                version = latest
                do {
                    let result = try await get_clip(language: language, text: text, access_token: media.accessToken)
                    try Task.checkCancellation()
                    #if DEBUG
                    DebugHarness.log("get_clip returned \(result == nil ? "none" : "clip") for \(text)")
                    #endif
                    if let result {
                        let hash = SHA256.hash(data: Data((result.movie_id + "\n" + text).utf8)).map { String(format: "%02x", $0) }.joined()
                        let url = FileManager.default.temporaryDirectory.appendingPathComponent("yap-clip-\(hash).mp4")
                        file = url
                        if !FileManager.default.fileExists(atPath: url.path) { try Data(result.bytes).write(to: url, options: .atomic) }
                        movie = media.movieMetadata(result.movie_id)
                        cues = result.subtitles
                        let next = AVPlayer(url: url)
                        await next.seek(to: CMTime(seconds: Double(result.critical_start_ms) / 1000, preferredTimescale: 1000))
                        try Task.checkCancellation()
                        player = next; available = true
                    } else { available = false }
                } catch {
                    if Task.isCancelled { return }
                    available = false
                    print("Yap clip lookup failed: \(error)")
                }
            }
            if let player {
                if playbackFailed || player.status == .failed || player.currentItem?.status == .failed {
                    audio.stopVideo(player)
                    self.player = nil; available = false
                    // Release the shared autoplay claim so the TTS fallback runs.
                    if autoplay { media.releaseAutoplay() }
                    do { try await invalidate_clip_cache(language: language, text: text) }
                    catch { print("Yap clip invalidation failed: \(error)") }
                    return
                }
                timeMs = player.currentTime().seconds * 1000
                playing = player.rate > 0
            }
            do { try await Task.sleep(for: .milliseconds(player == nil ? 2000 : 50)) } catch { return }
        }
    }
}

/// No independent AVKit transport controls: every play must interrupt TTS via
/// the shared playback registry, including taps on the video itself.
private struct ClipSurface: UIViewRepresentable {
    let player: AVPlayer
    final class Surface: UIView {
        override class var layerClass: AnyClass { AVPlayerLayer.self }
        var videoLayer: AVPlayerLayer { layer as! AVPlayerLayer }
    }
    func makeUIView(context: Context) -> Surface {
        let view = Surface()
        view.videoLayer.videoGravity = .resizeAspect
        view.backgroundColor = .black
        return view
    }
    func updateUIView(_ view: Surface, context: Context) { view.videoLayer.player = player }
    static func dismantleUIView(_ view: Surface, coordinator: ()) { view.videoLayer.player = nil }
}
