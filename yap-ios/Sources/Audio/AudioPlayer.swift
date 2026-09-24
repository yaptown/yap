import AVFoundation
import Observation

@Observable @MainActor final class AudioPlayer {
    private(set) var isPlaying = false
    private(set) var currentTime: TimeInterval = 0
    private(set) var currentRequest: AudioRequest?
    private(set) var voiceCredit: String?
    private(set) var needsAccount = false
    private var accountPromptTask: Task<Void, Never>?
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
            let bytes: [UInt8]
            do { bytes = try audio_for_native_playback(bytes: result.bytes) }
            catch { throw PlaybackError.unreadable(String(describing: error), bytes: result.bytes.count) }
            try Task.checkCancellation()
            guard expected == generation else { return }
            let container = Self.container(of: bytes)
            let audio: AVAudioPlayer
            do { audio = try AVAudioPlayer(data: Data(bytes), fileTypeHint: container.hint) }
            catch { throw PlaybackError.unreadable(String(describing: error), bytes: bytes.count) }
            if let actor = result.voice_actor { credit(actor) }
            Telemetry.breadcrumb("audio", "Playing \(container.name), \(bytes.count) bytes, \(String(format: "%.2f", audio.duration))s")
            try await play(audio, generation: expected)
        } catch {
            guard expected == generation else { return }
            stop()
            if !(error is CancellationError), !Task.isCancelled {
                if accessToken == nil, String(describing: error).contains("400") {
                    needsAccount = true
                    accountPromptTask?.cancel()
                    accountPromptTask = Task { [weak self] in
                        do { try await Task.sleep(for: .seconds(5)) } catch { return }
                        self?.needsAccount = false
                    }
                }
                Telemetry.breadcrumb("audio", "Playback failed: \(error)", failed: true)
            }
            // Only audio the decoder rejected is worth re-downloading; a refused
            // start or an interruption says nothing about the cached bytes.
            if case let playback as PlaybackError = error, playback.corrupt {
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
        guard audio.play() else {
            let session = AVAudioSession.sharedInstance()
            throw PlaybackError.refused(duration: audio.duration, category: session.category.rawValue, otherAudio: session.isOtherAudioPlaying)
        }
        isPlaying = true
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
        // A player that stops well short of its duration without a delegate error
        // was interrupted (a phone call, another app taking the session).
        if generation == expected, audio.currentTime > 0, audio.currentTime < audio.duration - 0.5 {
            throw PlaybackError.interrupted(at: audio.currentTime, duration: audio.duration)
        }
        withExtendedLifetime(delegate) {}
    }
    /// The cache stores WAV, MP3 and (remuxed) CAF under one extension; the
    /// hint spares CoreAudio from guessing the parser from the bytes.
    private static func container(of bytes: [UInt8]) -> (name: String, hint: String?) {
        if bytes.starts(with: Array("RIFF".utf8)) { return ("wav", AVFileType.wav.rawValue) }
        if bytes.starts(with: Array("caff".utf8)) { return ("caf", AVFileType.caf.rawValue) }
        if bytes.starts(with: Array("ID3".utf8)) || bytes.first == 0xFF { return ("mp3", AVFileType.mp3.rawValue) }
        return ("unknown", nil)
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
    /// Interrupt speech/video on playback changes or navigation, but let a chime finish.
    func stop() {
        video?.pause(); video = nil
        generation += 1
        player?.stop(); player = nil
        isPlaying = false; currentTime = 0; currentRequest = nil
    }
    /// Full teardown when the playback owner or signed-in session goes away.
    func stopAll() {
        stop()
        accountPromptTask?.cancel(); accountPromptTask = nil; needsAccount = false
        effectTask?.cancel(); effectTask = nil
        effectPlayer?.stop(); effectPlayer = nil; effectPlaying = false
    }
    /// Effects and sentence audio have independent players: neither channel
    /// interrupts the other (the web keeps them separate too).
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
    /// One generic message for the learner; the case and its payload are what
    /// the breadcrumb records so a failure can be diagnosed afterwards.
    enum PlaybackError: LocalizedError, CustomStringConvertible {
        case unreadable(String, bytes: Int)
        case refused(duration: TimeInterval, category: String, otherAudio: Bool)
        case decode(String)
        case stoppedEarly
        case interrupted(at: TimeInterval, duration: TimeInterval)
        var errorDescription: String? { "Couldn't play this audio. Please try again." }
        var corrupt: Bool { switch self { case .unreadable, .decode: true; default: false } }
        var description: String {
            switch self {
            case let .unreadable(reason, bytes): "AVAudioPlayer rejected \(bytes) bytes: \(reason)"
            case let .refused(duration, category, otherAudio): "play() returned false (duration \(duration)s, session \(category), other audio playing: \(otherAudio))"
            case let .decode(reason): "decode error: \(reason)"
            case .stoppedEarly: "finished with successfully=false"
            case let .interrupted(at, duration): "stopped at \(at)s of \(duration)s without a delegate error"
            }
        }
    }
    isolated deinit { stopAll(); creditTask?.cancel() }
}

/// Delegate entry points are nonisolated; only Sendable error values cross to the
/// main actor. The player itself never leaves the actor or enters the callback task.
@MainActor private final class PlaybackDelegate: NSObject, AVAudioPlayerDelegate {
    var failure: Error?
    nonisolated func audioPlayerDecodeErrorDidOccur(_ player: AVAudioPlayer, error: Error?) {
        let reason = error.map { String(describing: $0) } ?? "unknown"
        Task { @MainActor [weak self] in self?.failure = AudioPlayer.PlaybackError.decode(reason) }
    }
    nonisolated func audioPlayerDidFinishPlaying(_ player: AVAudioPlayer, successfully flag: Bool) {
        if !flag { Task { @MainActor [weak self] in self?.failure = AudioPlayer.PlaybackError.stoppedEarly } }
    }
}
