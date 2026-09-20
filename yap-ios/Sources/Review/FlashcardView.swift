import SwiftUI

func gramText(_ gram: [Literal_String]) -> String {
    gram.map { $0.word.text + $0.whitespace }.joined().trimmingCharacters(in: .whitespacesAndNewlines)
}

struct FlashcardView: View {
    @Environment(AudioPlayer.self) private var audio
    let model: ReviewModel
    let indicator: CardIndicator_Gram_String_String
    let flashcard: FlashCard
    let isNew: Bool
    let timesTypeSeen: UInt32
    @State private var revealed = false
    @State private var hasOpened = false
    private var disclosure: FlashcardDisclosure {
        get_flashcard_disclosure(total_card_count: model.reviewInfo.total_count, times_type_seen: timesTypeSeen)
    }
    private var canGrade: Bool { hasOpened || revealed || !disclosure.require_answer_reveal }
    private var listening: Bool { if case .Listening = flashcard.content { true } else { false } }
    private var exampleAudio: ExampleAudio {
        ExampleAudio(session: model.session, reviewCount: model.deck.get_total_reviews(), language: model.deck.get_target_language())
    }
    var body: some View {
        VStack(spacing: 12) {
            StudyCard {
                if isNew { ReviewBadge(text: "New word") }
                // Like the web card: audio at the leading edge, the word centered, the menu trailing.
                HStack(alignment: .center, spacing: 8) {
                    if let request = flashcard.audio {
                        AudioButton(request: request, session: model.session, reviewCount: model.deck.get_total_reviews(), autoplay: listening || revealed)
                    } else { Color.clear.frame(width: 44, height: 44) }
                    Group {
                        switch flashcard.content {
                        case let .Gram(gram, _, prefix, _):
                            Text((prefix.map { $0.prefix + $0.separator } ?? "") + gramText(gram))
                                .font(.system(size: 28, weight: .semibold, design: .rounded)).textSelection(.enabled)
                        case .Listening:
                            Text("What do you hear?").font(.title2.bold())
                        }
                    }.multilineTextAlignment(.center).frame(maxWidth: .infinity)
                    // The web's main row is always Again/Remembered; Hard/Good/Easy live in its menu.
                    Menu {
                        Button("Hard") { rate(.Hard) }
                        Button("Good") { rate(.Good) }
                        Button("Easy") { rate(.Easy) }
                    } label: {
                        Image(systemName: "ellipsis").frame(width: 44, height: 44).contentShape(Rectangle())
                    }.disabled(!canGrade || model.submitting).accessibilityLabel("More grades")
                }
                if disclosure.show_tutorial && should_show_challenge_tutorial(times_type_seen: timesTypeSeen) {
                    Text(listening ? "Listen, then guess what's being said." : "Guess the meaning, then reveal the answer.")
                        .font(.footnote).foregroundStyle(.secondary).frame(maxWidth: .infinity)
                }
                Divider()
                if revealed {
                    answer
                } else {
                    Label("Tap to reveal answer", systemImage: "chevron.down")
                        .font(.subheadline.weight(disclosure.require_answer_reveal ? .bold : .regular))
                        .foregroundStyle(disclosure.require_answer_reveal ? .primary : .secondary)
                        .frame(maxWidth: .infinity, minHeight: 44)
                }
            }
            .contentShape(Rectangle())
            .onTapGesture { toggle() }
            .accessibilityAction(named: revealed ? "Hide answer" : "Reveal answer") { toggle() }
            if canGrade {
                HStack(spacing: 12) {
                    Button { rate(.Again) } label: { Text(isNew ? "Didn't know" : "Forgot").frame(maxWidth: .infinity) }
                        .tint(.red).keyboardShortcut(.leftArrow, modifiers: [])
                    Button { rate(.Remembered) } label: { Text(isNew ? "Already knew" : "Remembered").frame(maxWidth: .infinity) }
                        .keyboardShortcut(.rightArrow, modifiers: [])
                }.buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large).disabled(model.submitting)
            }
            if listening {
                Button("I can't listen right now") { model.cantListen() }.font(.footnote).foregroundStyle(.secondary).frame(minHeight: 44)
            }
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeTab == .learn else { return }
            switch DebugHarness.shared.command {
            case "reveal": revealed = true; hasOpened = true
            case "grade": rate(.Remembered)
            case "grade-again": rate(.Again)
            case "audio":
                if let request = flashcard.audio {
                    Task {
                        do { try await audio.play(request: request, accessToken: model.session.accessToken()); DebugHarness.log("audio completed") }
                        catch { DebugHarness.log("audio failed: \(error)") }
                    }
                }
            default: break
            }
        }
        #endif
    }
    private func toggle() { revealed.toggle(); hasOpened = true }
    @ViewBuilder private var answer: some View {
        switch flashcard.content {
        case let .Gram(_, definition, _, breakdown):
            DefinitionView(definition: definition, exampleAudio: exampleAudio)
            if let breakdown, !breakdown.isEmpty { MorphemeBreakdownView(parts: breakdown, alignment: .center) }
        case let .Listening(possible):
            if possible.count > 1 { Text("It could have been any of these words:").font(.footnote).foregroundStyle(.secondary) }
            ForEach(Array(possible.enumerated()), id: \.offset) { _, entry in
                VStack(alignment: .leading, spacing: 8) {
                    HStack(alignment: .firstTextBaseline, spacing: 8) {
                        Text(gramText(entry.second)).font(.title3.weight(.medium))
                        if entry.first { Text("(known)").font(.footnote).foregroundStyle(.green) }
                    }
                    ForEach(Array(entry.third.enumerated()), id: \.offset) { _, definition in
                        DefinitionView(definition: definition, exampleAudio: exampleAudio)
                    }
                }
            }
        }
    }
    private func rate(_ rating: Rating) {
        guard canGrade, !model.submitting else { return }
        audio.stop()
        model.rate(indicator, rating)
        if rating != .Again { audio.playEffect("success-\(Int.random(in: 1...3))") }
    }
}

/// What an example sentence needs to be played aloud from a definition box.
struct ExampleAudio {
    let session: YapSession
    let reviewCount: UInt64
    let language: Language
}

/// The web's CardBack: each sense in its own muted box, the morphology trailing
/// the meaning, the example quoted with a small play button beside it.
struct DefinitionView: View {
    private enum Content {
        case dictionary([TargetToNativeWord], [Morphology])
        case phrase(String, String, String, String)
    }
    private let content: Content
    private let exampleAudio: ExampleAudio?
    init(definition: GramDefinition, exampleAudio: ExampleAudio? = nil) {
        self.exampleAudio = exampleAudio
        switch definition {
        case let .Dictionary(entry): content = .dictionary(entry.definitions, entry.morphology)
        case let .Phrasebook(entry): content = .phrase(entry.meaning, entry.additional_notes, entry.target_language_example, entry.native_language_example)
        }
    }
    init(entry: GramDictionaryEntry, exampleAudio: ExampleAudio? = nil) {
        self.exampleAudio = exampleAudio
        switch entry.definition {
        case let .Dictionary(definitions): content = .dictionary(definitions, entry.morphology.map { [$0] } ?? [])
        case let .Phrasebook(meaning, target, native): content = .phrase(meaning, "", target ?? "", native ?? "")
        }
    }
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            switch content {
            case let .dictionary(definitions, morphologies):
                let morphology = morphologies.first.map(morphologyText).flatMap { $0.isEmpty ? nil : $0 }
                ForEach(Array(definitions.enumerated()), id: \.offset) { _, item in
                    box(meaning: item.native, trailing: morphology, note: item.note ?? "",
                        target: item.example_sentence_target_language, native: item.example_sentence_native_language)
                }
            case let .phrase(meaning, notes, target, native):
                box(meaning: meaning, trailing: nil, note: notes, target: target, native: native)
            }
        }
    }
    private func box(meaning: String, trailing: String?, note: String, target: String, native: String) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(meaning).font(.title3.weight(.medium))
                Spacer(minLength: 0)
                if let trailing { Text(trailing).font(.caption).italic().foregroundStyle(.secondary).multilineTextAlignment(.trailing) }
            }
            if !note.isEmpty { Text(note).font(.footnote).foregroundStyle(.secondary) }
            if !target.isEmpty {
                HStack(alignment: .top, spacing: 4) {
                    if let exampleAudio {
                        AudioButton(request: AudioRequest(request: TtsRequest(text: target, language: exampleAudio.language, is_ssml: false,
                            instructions: nil, speed: 1, verification_hints: []), provider: .ElevenLabs),
                            session: exampleAudio.session, reviewCount: exampleAudio.reviewCount)
                    }
                    DefinitionExamples(target: target, native: native).frame(minHeight: exampleAudio == nil ? 0 : 44, alignment: .center)
                }
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading).padding(12)
        .background(Color(uiColor: .tertiarySystemGroupedBackground), in: RoundedRectangle(cornerRadius: 10))
    }
    private func morphologyText(_ value: Morphology) -> String {
        [value.gender.map { String(describing: $0) }, value.number.map { String(describing: $0) },
         value.tense.map { String(describing: $0) }, value.person.map { String(describing: $0) },
         value.case.map { String(describing: $0) }, value.mood.map { String(describing: $0) },
         value.aspect.map { String(describing: $0) }, value.politeness.map { String(describing: $0) }]
            .compactMap { $0 }.joined(separator: " · ")
    }
}
