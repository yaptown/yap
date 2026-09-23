import SwiftUI

struct PronunciationChallengeView: View {
    @Environment(AudioPlayer.self) private var audio
    @Environment(\.reviewHost!) private var host
    @Environment(\.reviewActions!) private var actions
    let indicator: CardIndicator_Gram_String_String
    let pattern: String
    let guide: PronunciationGuide
    let cues: [PronunciationCue]
    let isNew: Bool
    let timesSeen: UInt32
    private var view: PronunciationView {
        pronunciation_view(pattern: pattern, guide: guide, cues: cues, is_new: isNew, times_type_seen: timesSeen)
    }
    var body: some View {
        ReviewStepScrollView {
            if let prompt = view.tutorial_prompt { TutorialPromptText(prompt: prompt) }
            StudyCard {
                HStack(alignment: .center, spacing: 8) {
                    Color.clear.frame(width: 44, height: 44)
                    Text(view.positioned_pattern).font(.title2.bold()).multilineTextAlignment(.center).frame(maxWidth: .infinity)
                    Color.clear.frame(width: 44, height: 44)
                }
                if let note = view.position_note {
                    Text(note).font(.caption).foregroundStyle(.secondary).frame(maxWidth: .infinity)
                }
                ForEach(Array(view.examples.enumerated()), id: \.offset) { _, example in
                    PronunciationRow(cue: example.cue, pattern: view.pattern, position: view.position,
                                     context: example.cultural_context)
                }
                if let description = view.description { Text(markdown(description)).font(.subheadline) }
            }
            if let prompt = view.tutorial_grade_prompt {
                Text(prompt).font(.footnote).foregroundStyle(.secondary).multilineTextAlignment(.center)
            }
        } actions: {
            HStack(spacing: 12) {
                Button { rate(.Again) } label: { Text(view.again_label).frame(maxWidth: .infinity) }.tint(Tokens.palette.destructive.color).foregroundStyle(Color.yapDestructiveForeground)
                Button { rate(.Remembered) } label: { Text(view.remembered_label).frame(maxWidth: .infinity) }
            }.buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large).disabled(actions.submitting)
            Button(view.cant_speak_label) { actions.cantSpeak() }.font(.footnote).foregroundStyle(.secondary).frame(minHeight: 44)
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeScreen == .review else { return }
            let command = DebugHarness.shared.command
            if command == "pron-grade" || command == "grade" { rate(.Remembered) }
            if command == "audio", let cue = cues.first {
                Task { try? await audio.play(request: cue.audio, accessToken: host.accessToken) }
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
    @Environment(\.reviewScreen!) private var screen
    let cue: PronunciationCue
    let pattern: String
    let position: PatternPosition
    let context: String?
    @State private var connectorHeard = false
    private var playing: Bool { audio.currentRequest == cue.audio && audio.isPlaying }
    private var firstConnector: Int? { cue.segments.firstIndex { $0.role == .Connector } }
    private func trailingSpace(_ index: Int) -> String {
        guard index + 1 < cue.segments.count else { return "" }
        return cue.segments[index].role == .Pattern && cue.segments[index + 1].role == .Pattern ? "" : " "
    }
    private func spokenWord(_ index: Int) -> AttributedString {
        let segment = cue.segments[index]
        let current = playing ? cue.segments.lastIndex { $0.start_ms.map { Double($0) <= audio.currentTime * 1000 } ?? false } : nil
        let firstExample = cue.segments.firstIndex { $0.role == .Example }
        let lastExample = cue.segments.lastIndex { $0.role == .Example }
        var word = AttributedString(segment.text)
        word.foregroundColor = current == index ? .yapAccent : segment.role == .Connector ? .secondary : .yapText
        word.font = .body.weight(segment.role == .Example ? .semibold : .regular)
        if playing, let start = segment.start_ms, audio.currentTime * 1000 < Double(start) { word.foregroundColor = .secondary.opacity(0.5) }
        if segment.role == .Example {
            let options: String.CompareOptions = position == .End ? [.caseInsensitive, .backwards] : [.caseInsensitive]
            if let range = word.range(of: pattern, options: options),
               position == .Anywhere || (position == .Beginning && index == firstExample && range.lowerBound == word.startIndex)
                || (position == .End && index == lastExample && range.upperBound == word.endIndex) {
                word[range].backgroundColor = .yapCaution.opacity(0.3)
            }
        }
        word += AttributedString(trailingSpace(index))
        return word
    }
    private var connector: some View {
        let indices = cue.segments.indices.filter { cue.segments[$0].role == .Connector }
        let target = indices.reduce(into: AttributedString()) { $0 += spokenWord($1) }
        let native = cue.native_connector + (indices.last.map { trailingSpace($0) } ?? "")
        return ZStack {
            Text(native).foregroundStyle(.secondary).opacity(connectorHeard ? 0 : 1).accessibilityHidden(connectorHeard)
            Text(target).opacity(connectorHeard ? 1 : 0).accessibilityHidden(!connectorHeard)
        }.animation(.easeInOut(duration: 0.3), value: connectorHeard)
    }
    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            AudioButton(request: cue.audio, reviewCount: screen.total_reviews)
            VStack(alignment: .leading, spacing: 4) {
                SentenceFlow(spacing: 0) {
                    ForEach(cue.segments.indices, id: \.self) { index in
                        if cue.segments[index].role == .Connector {
                            if index == firstConnector { connector }
                        } else { Text(spokenWord(index)) }
                    }
                }.font(.body)
                if let context { Text(context).font(.footnote).foregroundStyle(.secondary) }
            }.frame(maxWidth: .infinity, alignment: .leading)
        }.onChange(of: playing) { _, playing in if playing { connectorHeard = true } }
    }
}
