import SwiftUI

struct PronunciationChallengeView: View {
    @Environment(AudioPlayer.self) private var audio
    let screen: ReviewScreenView
    let actions: ReviewActions
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
        VStack(spacing: 12) {
            StudyCard {
                HStack(alignment: .center, spacing: 8) {
                    Color.clear.frame(width: 44, height: 44)
                    Text(positionedPattern).font(.title2.bold()).multilineTextAlignment(.center).frame(maxWidth: .infinity)
                    Color.clear.frame(width: 44, height: 44)
                }
                if should_show_challenge_tutorial(times_type_seen: timesSeen) { Text("Listen, then practice saying the sound aloud.").font(.footnote).foregroundStyle(.secondary).multilineTextAlignment(.center).frame(maxWidth: .infinity) }
                ForEach(Array(cues.prefix(3).enumerated()), id: \.offset) { index, cue in
                    PronunciationRow(screen: screen, actions: actions, cue: cue, pattern: pattern, position: guide.position,
                                     context: guide.example_words.indices.contains(index) ? guide.example_words[index].cultural_context : nil)
                }
                Text(markdown(guide.description)).font(.subheadline)
            }
            HStack(spacing: 12) {
                Button { rate(.Again) } label: { Text(isNew ? "Didn't know" : "Forgot").frame(maxWidth: .infinity) }.tint(.red)
                Button { rate(.Remembered) } label: { Text(isNew ? "Already knew" : "Remembered").frame(maxWidth: .infinity) }
            }.buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large).disabled(actions.submitting)
            Button("I can't speak right now") { actions.cantSpeak() }.font(.footnote).foregroundStyle(.secondary).frame(minHeight: 44)
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeTab == .learn else { return }
            let command = DebugHarness.shared.command
            if command == "pron-grade" || command == "grade" { rate(.Remembered) }
            if command == "audio", let cue = cues.first {
                Task { try? await audio.play(request: cue.audio, accessToken: actions.media.accessToken) }
            }
        }
        #endif
    }
    private func rate(_ rating: Rating) {
        guard !actions.submitting else { return }
        audio.stop(); actions.rate(indicator, rating)
        if rating != .Again { audio.playEffect("success-2") }
    }
}

private struct PronunciationRow: View {
    @Environment(AudioPlayer.self) private var audio
    let screen: ReviewScreenView
    let actions: ReviewActions
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
        HStack(alignment: .top, spacing: 8) {
            AudioButton(request: cue.audio, media: actions.media, reviewCount: screen.total_reviews)
            VStack(alignment: .leading, spacing: 4) {
                Text(words).font(.body)
                if let context { Text(context).font(.footnote).foregroundStyle(.secondary) }
            }.frame(maxWidth: .infinity, alignment: .leading)
        }.onChange(of: playing) { _, playing in if playing { connectorHeard = true } }
    }
}
