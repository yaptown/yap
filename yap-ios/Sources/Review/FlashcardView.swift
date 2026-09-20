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
    var body: some View {
        VStack(spacing: 24) {
            StudyCard {
                HStack {
                    Text(isNew ? "NEW WORD" : "FLASHCARD").font(.caption.weight(.semibold)).foregroundStyle(.secondary)
                    Spacer()
                    // The web's main row is always Again/Remembered; Hard/Good/Easy live in its menu.
                    Menu("More", systemImage: "ellipsis") {
                        Button("Hard") { rate(.Hard) }
                        Button("Good") { rate(.Good) }
                        Button("Easy") { rate(.Easy) }
                    }.disabled(!canGrade || model.submitting)
                }
                if disclosure.show_tutorial && should_show_challenge_tutorial(times_type_seen: timesTypeSeen) {
                    Text(listening ? "Listen, then guess what's being said." : "Guess the meaning, then reveal the answer.").foregroundStyle(.secondary)
                }
                switch flashcard.content {
                case let .Gram(gram, _, prefix, _):
                    Text((prefix.map { $0.prefix + $0.separator } ?? "") + gramText(gram))
                        .font(.system(size: 36, weight: .semibold, design: .rounded)).textSelection(.enabled)
                case .Listening:
                    Text("What do you hear?").font(.title.bold())
                }
                if let request = flashcard.audio {
                    AudioButton(request: request, session: model.session, reviewCount: model.deck.get_total_reviews(), autoplay: listening || revealed)
                }
                Button(revealed ? "Hide answer" : "Reveal answer") {
                    revealed.toggle(); hasOpened = true
                }.buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large).keyboardShortcut(.space, modifiers: [])
                if revealed {
                    Divider()
                    answer
                }
            }
            if canGrade {
                HStack(spacing: 12) {
                    Button(isNew ? "Didn't know" : "Forgot") { rate(.Again) }
                        .tint(.red).keyboardShortcut(.leftArrow, modifiers: [])
                    Button(isNew ? "Already knew" : "Remembered") { rate(.Remembered) }
                        .keyboardShortcut(.rightArrow, modifiers: [])
                }.buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large).disabled(model.submitting)
            }
            if listening { Button("I can't listen right now") { model.cantListen() }.frame(minHeight: 44) }
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
    @ViewBuilder private var answer: some View {
        switch flashcard.content {
        case let .Gram(_, definition, _, breakdown):
            DefinitionView(definition: definition)
            if let breakdown { MorphemeBreakdownView(parts: breakdown) }
        case let .Listening(possible):
            ForEach(Array(possible.enumerated()), id: \.offset) { _, entry in
                VStack(alignment: .leading, spacing: 12) {
                    Text(gramText(entry.second)).font(.title2.bold())
                    ForEach(Array(entry.third.enumerated()), id: \.offset) { _, definition in DefinitionView(definition: definition) }
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

struct DefinitionView: View {
    private enum Content {
        case dictionary([TargetToNativeWord], [Morphology])
        case phrase(String, String, String, String)
    }
    private let content: Content
    init(definition: GramDefinition) {
        switch definition {
        case let .Dictionary(entry): content = .dictionary(entry.definitions, entry.morphology)
        case let .Phrasebook(entry): content = .phrase(entry.meaning, entry.additional_notes, entry.target_language_example, entry.native_language_example)
        }
    }
    init(entry: GramDictionaryEntry) {
        switch entry.definition {
        case let .Dictionary(definitions): content = .dictionary(definitions, entry.morphology.map { [$0] } ?? [])
        case let .Phrasebook(meaning, target, native): content = .phrase(meaning, "", target ?? "", native ?? "")
        }
    }
    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            switch content {
            case let .dictionary(definitions, morphologies):
                ForEach(Array(definitions.enumerated()), id: \.offset) { _, item in
                    VStack(alignment: .leading, spacing: 8) {
                        Text(item.native).font(.title3.weight(.semibold))
                        if let note = item.note, !note.isEmpty { Text(note).font(.subheadline).foregroundStyle(.secondary) }
                        if !item.example_sentence_target_language.isEmpty {
                            Text(item.example_sentence_target_language).italic()
                            Text(item.example_sentence_native_language).foregroundStyle(.secondary)
                        }
                    }
                }
                if let morphology = morphologies.first {
                    let text = morphologyText(morphology)
                    if !text.isEmpty { Text(text).font(.caption).foregroundStyle(.secondary) }
                }
            case let .phrase(meaning, notes, target, native):
                Text(meaning).font(.title3.weight(.semibold))
                if !notes.isEmpty { Text(notes).foregroundStyle(.secondary) }
                if !target.isEmpty { Text(target).italic() }
                if !native.isEmpty { Text(native).foregroundStyle(.secondary) }
            }
        }
    }
    private func morphologyText(_ value: Morphology) -> String {
        [value.gender.map { String(describing: $0) }, value.number.map { String(describing: $0) },
         value.tense.map { String(describing: $0) }, value.person.map { String(describing: $0) },
         value.case.map { String(describing: $0) }, value.mood.map { String(describing: $0) },
         value.aspect.map { String(describing: $0) }, value.politeness.map { String(describing: $0) }]
            .compactMap { $0 }.joined(separator: " · ")
    }
}
