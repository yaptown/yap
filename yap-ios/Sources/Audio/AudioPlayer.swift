import AVFoundation
import Observation

@Observable @MainActor final class AudioPlayer {
    private(set) var isPlaying = false
    private(set) var currentTime: TimeInterval = 0
    private(set) var currentRequest: AudioRequest?
    private(set) var voiceCredit: String?
    private var creditTask: Task<Void, Never>?
    private var player: AVAudioPlayer?
    private var video: AVPlayer?
    private var generation = 0
    private var configured = false
    private var effectTask: Task<Void, Never>?
    private var effectPlayer: AVAudioPlayer?
    private(set) var effectPlaying = false

    private func configure() throws {
        guard !configured else { return }
        try AVAudioSession.sharedInstance().setCategory(.playback)
        try AVAudioSession.sharedInstance().setActive(true)
        configured = true
    }
    func play(request: AudioRequest, accessToken: String?) async throws {
        stop()
        let expected = generation
        currentRequest = request
        do {
            let result = try await get_audio(request: request, access_token: accessToken)
            try Task.checkCancellation()
            guard expected == generation else { return }
            try configure()
            let bytes = try audio_for_native_playback(bytes: result.bytes)
            try Task.checkCancellation()
            guard expected == generation else { return }
            let audio = try AVAudioPlayer(data: Data(bytes))
            if let actor = result.voice_actor { credit(actor) }
            try await play(audio, generation: expected)
        } catch {
            guard expected == generation else { return }
            stop()
            if !(error is CancellationError), !Task.isCancelled {
                do { try await invalidate_audio_cache(request: request) }
                catch { print("Yap audio cache invalidation failed: \(error)") }
            }
            throw error
        }
    }
    private func play(_ audio: AVAudioPlayer, generation expected: Int) async throws {
        player = audio
        let delegate = PlaybackDelegate()
        audio.delegate = delegate
        guard audio.play() else { throw PlaybackError.couldNotPlay }
        isPlaying = true
        print("Yap: AVAudioPlayer started (\(audio.duration)s)")
        #if DEBUG
        DebugHarness.log("audio started duration=\(audio.duration)")
        #endif
        defer {
            if generation == expected { player = nil; isPlaying = false; currentTime = 0; currentRequest = nil }
        }
        while audio.isPlaying, generation == expected {
            do { try await Task.sleep(for: .milliseconds(50)) }
            catch { audio.stop(); throw error }
            guard generation == expected else { return }
            currentTime = audio.currentTime
        }
        if let error = delegate.failure { throw error }
        withExtendedLifetime(delegate) {}
    }
    private func credit(_ actor: VoiceActorInfo) {
        let key = "voice-actor-toast:\(actor.name)"
        let now = Date().timeIntervalSince1970
        if let last = UserDefaults.standard.object(forKey: key) as? Double, now - last < 86_400 { return }
        UserDefaults.standard.set(now, forKey: key)
        voiceCredit = "Audio recorded by @\(actor.name), a \(actor.compensation == .Paid ? "paid" : "volunteer") voice actor"
        #if DEBUG
        DebugHarness.log("voice credit: \(actor.name)")
        #endif
        creditTask?.cancel()
        creditTask = Task { [weak self] in
            do { try await Task.sleep(for: .seconds(5)) } catch { return }
            self?.voiceCredit = nil
        }
    }
    func playVideo(_ video: AVPlayer) throws {
        stop()
        try configure()
        self.video = video
        video.seek(to: .zero)
        video.play()
    }
    func stopVideo(_ video: AVPlayer) {
        video.pause()
        if self.video === video { self.video = nil }
    }
    func stop() {
        video?.pause(); video = nil
        generation += 1
        effectTask?.cancel(); effectTask = nil
        effectPlayer?.stop(); effectPlayer = nil; effectPlaying = false
        player?.stop(); player = nil
        isPlaying = false; currentTime = 0; currentRequest = nil
    }
    /// Effects have their own player so a grading sound never interrupts the
    /// sentence audio that is still playing (the web layers them the same way).
    func playEffect(_ name: String) {
        effectTask?.cancel(); effectPlayer?.stop()
        guard let url = Bundle.main.url(forResource: name, withExtension: "mp3") else { return }
        effectTask = Task { [weak self] in
            guard let self else { return }
            var effect: AVAudioPlayer?
            do {
                try self.configure()
                let player = try AVAudioPlayer(contentsOf: url)
                player.volume = 0.5
                effect = player
                self.effectPlayer = player
                self.effectPlaying = player.play()
                while player.isPlaying, !Task.isCancelled {
                    try await Task.sleep(for: .milliseconds(50))
                }
            } catch { if !Task.isCancelled { print("Yap effect failed: \(error)") } }
            // Only the effect that this task started may clear the flag.
            if let effect, self.effectPlayer === effect { self.effectPlaying = false }
        }
    }
    enum PlaybackError: LocalizedError {
        case couldNotPlay
        var errorDescription: String? { "Couldn't play this audio. Please try again." }
    }
    isolated deinit { video?.pause(); player?.stop(); effectPlayer?.stop(); effectTask?.cancel(); creditTask?.cancel() }
}

/// Delegate entry points are nonisolated; only Sendable error values cross to the
/// main actor. The player itself never leaves the actor or enters the callback task.
@MainActor private final class PlaybackDelegate: NSObject, AVAudioPlayerDelegate {
    var failure: Error?
    nonisolated func audioPlayerDecodeErrorDidOccur(_ player: AVAudioPlayer, error: Error?) {
        Task { @MainActor [weak self] in self?.failure = error ?? AudioPlayer.PlaybackError.couldNotPlay }
    }
    nonisolated func audioPlayerDidFinishPlaying(_ player: AVAudioPlayer, successfully flag: Bool) {
        if !flag { Task { @MainActor [weak self] in self?.failure = AudioPlayer.PlaybackError.couldNotPlay } }
    }
}
