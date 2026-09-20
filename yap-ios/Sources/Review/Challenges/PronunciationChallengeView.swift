import SwiftUI

struct PronunciationChallengeView: View {
    @Environment(AudioPlayer.self) private var audio
    let model: ReviewModel
    let indicator: CardIndicator_Gram_String_String
    let pattern: String
    let guide: PronunciationGuide
    let cues: [PronunciationCue]
    let isNew: Bool
    let timesSeen: UInt32
    private var positionedPattern: String {
        switch guide.position { case .Beginning: pattern + "___"; case .End: "___" + pattern; case .Anywhere: pattern }
    }
    var body: some View {
        VStack(spacing: 24) {
            StudyCard {
                Text("PRONUNCIATION").font(.caption).foregroundStyle(.secondary)
                Text(positionedPattern).font(.largeTitle.bold())
                if should_show_challenge_tutorial(times_type_seen: timesSeen) { Text("Listen, then practice saying the sound aloud.").foregroundStyle(.secondary) }
                ForEach(Array(cues.prefix(3).enumerated()), id: \.offset) { index, cue in
                    PronunciationRow(model: model, cue: cue, pattern: pattern, position: guide.position,
                                     context: guide.example_words.indices.contains(index) ? guide.example_words[index].cultural_context : nil)
                }
                Text(markdown(guide.description))
            }
            HStack(spacing: 12) {
                Button(isNew ? "Didn't know" : "Forgot") { rate(.Again) }.tint(.red)
                Button(isNew ? "Already knew" : "Remembered") { rate(.Remembered) }
            }.buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large).disabled(model.submitting)
            Button("I can't speak right now") { model.cantSpeak() }
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeTab == .learn else { return }
            let command = DebugHarness.shared.command
            if command == "pron-grade" || command == "grade" { rate(.Remembered) }
            if command == "audio", let cue = cues.first {
                Task { try? await audio.play(request: cue.audio, accessToken: model.session.accessToken()) }
            }
        }
        #endif
    }
    private func rate(_ rating: Rating) {
        guard !model.submitting else { return }
        audio.stop(); model.rate(indicator, rating)
        if rating != .Again { audio.playEffect("success-2") }
    }
}

private struct PronunciationRow: View {
    @Environment(AudioPlayer.self) private var audio
    let model: ReviewModel
    let cue: PronunciationCue
    let pattern: String
    let position: PatternPosition
    let context: String?
    @State private var connectorHeard = false
    private var playing: Bool { audio.currentRequest == cue.audio && audio.isPlaying }
    private var words: AttributedString {
        let current = playing ? cue.segments.lastIndex { $0.start_ms.map { Double($0) <= audio.currentTime * 1000 } ?? false } : nil
        let firstExample = cue.segments.firstIndex { $0.role == .Example }
        let lastExample = cue.segments.lastIndex { $0.role == .Example }
        let firstConnector = cue.segments.firstIndex { $0.role == .Connector }
        var result = AttributedString()
        for (index, segment) in cue.segments.enumerated() {
            if segment.role == .Connector && !connectorHeard && index != firstConnector { continue }
            if index > 0 && !(segment.role == .Pattern && cue.segments[index - 1].role == .Pattern) { result += AttributedString(" ") }
            var word = AttributedString(segment.role == .Connector && !connectorHeard ? cue.native_connector : segment.text)
            word.foregroundColor = current == index ? .yapAccent : segment.role == .Connector ? .secondary : .yapText
            word.font = .body.weight(segment.role == .Example ? .semibold : .regular)
            if playing, let start = segment.start_ms, audio.currentTime * 1000 < Double(start) { word.foregroundColor = .secondary.opacity(0.5) }
            if segment.role == .Example {
                let options: String.CompareOptions = position == .End ? [.caseInsensitive, .backwards] : [.caseInsensitive]
                if let range = word.range(of: pattern, options: options),
                   position == .Anywhere || (position == .Beginning && index == firstExample && range.lowerBound == word.startIndex)
                    || (position == .End && index == lastExample && range.upperBound == word.endIndex) {
                    word[range].backgroundColor = .yellow.opacity(0.3)
                }
            }
            result += word
        }
        return result
    }
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(words).font(.title3)
            if let context { Text(context).font(.caption).foregroundStyle(.secondary) }
            AudioButton(request: cue.audio, session: model.session, reviewCount: model.deck.get_total_reviews())
        }.onChange(of: playing) { _, playing in if playing { connectorHeard = true } }
    }
}
