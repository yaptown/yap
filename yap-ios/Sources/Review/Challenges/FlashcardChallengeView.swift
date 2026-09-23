import SwiftUI

func gramText(_ gram: [Literal_String]) -> String {
    gram.map { $0.word.text + $0.whitespace }.joined().trimmingCharacters(in: .whitespacesAndNewlines)
}

struct FlashcardChallengeView: View {
    @Environment(AudioPlayer.self) private var audio
    @Environment(\.reviewScreen!) private var screen
    @Environment(\.reviewHost!) private var host
    @Environment(\.reviewActions!) private var actions
    let indicator: CardIndicator_Gram_String_String
    let flashcard: FlashCard
    let isNew: Bool
    let timesTypeSeen: UInt32
    @State private var revealed = false
    @State private var hasOpened = false
    private var view: FlashcardView {
        flashcard_view(flashcard: flashcard, is_new: isNew, total_card_count: screen.total_count,
                       times_type_seen: timesTypeSeen, target_language: screen.target_language,
                       native_language: screen.native_language)
    }
    private var canGrade: Bool { hasOpened || revealed || !view.require_answer_reveal }
    private var listening: Bool { if case .Listening = flashcard.content { true } else { false } }
    var body: some View {
        ReviewStepScrollView {
            if !revealed, let prompt = view.tutorial_prompt {
                TutorialPromptText(prompt: prompt)
            }
            StudyCard {
                // Like the web card: audio at the leading edge, the word centered, the menu trailing.
                HStack(alignment: .center, spacing: 8) {
                    if let request = flashcard.audio {
                        AudioButton(request: request, reviewCount: screen.total_reviews, autoplay: listening || revealed)
                            .frame(maxWidth: listening ? .infinity : nil)
                    } else { Color.clear.frame(width: 44, height: 44) }
                    Group {
                        switch flashcard.content {
                        case let .Gram(gram, _, prefix, _):
                            Text((prefix.map { $0.prefix + $0.separator } ?? "") + gramText(gram))
                                .font(.system(size: 28, weight: .semibold, design: .rounded)).textSelection(.enabled)
                        case .Listening:
                            EmptyView()
                        }
                    }.multilineTextAlignment(.center).frame(maxWidth: listening ? nil : .infinity)
                    // The web's main row is always Again/Remembered; Hard/Good/Easy live in its menu.
                    Menu {
                        ForEach(Array(view.menu_grades.enumerated()), id: \.offset) { _, grade in
                            Button(grade.label) { rate(grade.rating) }
                        }
                    } label: {
                        Image(systemName: "ellipsis").frame(width: 44, height: 44).contentShape(Rectangle())
                    }.disabled(!canGrade || actions.submitting).accessibilityLabel("More grades")
                }
                if let subtitle = view.subtitle {
                    Text(subtitle).font(.footnote).foregroundStyle(.secondary).frame(maxWidth: .infinity)
                }
                Divider()
                if revealed {
                    answer
                } else {
                    Label(view.reveal_label, systemImage: "chevron.down")
                        .font(.subheadline.weight(view.require_answer_reveal ? .bold : .regular))
                        .foregroundStyle(view.require_answer_reveal ? .primary : .secondary)
                        .frame(maxWidth: .infinity, minHeight: 44)
                }
            }
            .contentShape(Rectangle())
            .onTapGesture { toggle() }
            .accessibilityAction(named: revealed ? "Hide answer" : "Reveal answer") { toggle() }
            if !revealed, let hint = view.tutorial_hidden_hint {
                Text(hint).font(.footnote).foregroundStyle(.secondary).multilineTextAlignment(.center)
            }
        } actions: {
            if revealed, let hint = view.tutorial_revealed_hint {
                Text(hint).font(.footnote).foregroundStyle(.secondary).multilineTextAlignment(.center)
            }
            if !revealed, let label = view.cant_listen_label {
                Button(label) { actions.cantListen() }.font(.footnote).foregroundStyle(.secondary).frame(minHeight: 44)
            }
            if canGrade {
                HStack(spacing: 12) {
                    Button { rate(.Again) } label: { Text(view.again_label).frame(maxWidth: .infinity) }
                        .tint(Tokens.palette.destructive.color).foregroundStyle(Color.yapDestructiveForeground).keyboardShortcut(.leftArrow, modifiers: [])
                    Button { rate(.Remembered) } label: { Text(view.remembered_label).frame(maxWidth: .infinity) }
                        .keyboardShortcut(.rightArrow, modifiers: [])
                }.buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large).disabled(actions.submitting)
            }
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeScreen == .review else { return }
            switch DebugHarness.shared.command {
            case "reveal": revealed = true; hasOpened = true
            case "grade": rate(.Remembered)
            case "grade-again": rate(.Again)
            case "audio":
                if let request = flashcard.audio {
                    Task {
                        do { try await audio.play(request: request, accessToken: host.accessToken); DebugHarness.log("audio completed") }
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
            DefinitionBoxesView(definition: definition)
            if let breakdown, !breakdown.isEmpty { MorphemeBreakdownView(parts: breakdown, alignment: .center) }
        case let .Listening(possible):
            if let header = view.listening_header { Text(header).font(.footnote).foregroundStyle(.secondary) }
            ForEach(Array(possible.enumerated()), id: \.offset) { _, entry in
                VStack(alignment: .leading, spacing: 8) {
                    HStack(alignment: .firstTextBaseline, spacing: 8) {
                        Text(gramText(entry.second)).font(.title3.weight(.medium))
                        if possible.count > 1 && entry.first { Text(view.known_label).font(.footnote).foregroundStyle(Color.yapPositiveForeground) }
                    }
                    ForEach(Array(entry.third.enumerated()), id: \.offset) { _, definition in
                        DefinitionBoxesView(definition: definition)
                    }
                }
            }
        }
    }
    private func rate(_ rating: Rating) {
        guard canGrade, !actions.submitting else { return }
        audio.stop()
        actions.rate(indicator, rating)
        if rating != .Again { audio.playEffect("success-\(Int.random(in: 1...3))") }
    }
}

struct TutorialPromptText: View {
    let prompt: TutorialPrompt

    var body: some View {
        Text(prompt.before + (prompt.target ?? "") + prompt.after)
            .font(.footnote).foregroundStyle(.secondary).multilineTextAlignment(.center)
            .frame(maxWidth: .infinity)
    }
}

/// The web's CardBack: each sense in its own muted box, the morphology trailing
/// the meaning, the example quoted with a small play button beside it.
struct DefinitionBoxesView: View {
    @Environment(\.reviewHost!) private var host
    let definition: DefinitionView

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            ForEach(Array(definition.senses.enumerated()), id: \.offset) { _, sense in
                box(sense: sense, trailing: definition.morphology_label.isEmpty ? nil : definition.morphology_label)
            }
        }
    }
    private func box(sense: DefinitionSense, trailing: String?) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(sense.meaning).font(.title3.weight(.medium))
                Spacer(minLength: 0)
                if let trailing { Text(trailing).font(.caption).italic().foregroundStyle(.secondary).multilineTextAlignment(.trailing) }
            }
            if let note = sense.note { Text(note).font(.footnote).foregroundStyle(.secondary) }
            if let example = sense.example {
                HStack(alignment: .top, spacing: 4) {
                    AudioButton(request: AudioRequest(request: TtsRequest(text: example.target, language: host.deck.get_target_language(), is_ssml: false,
                        instructions: nil, speed: 1, verification_hints: []), provider: .ElevenLabs),
                        reviewCount: host.deck.get_total_reviews())
                    DefinitionExamples(example: example).frame(minHeight: 44, alignment: .center)
                }
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading).padding(12)
        .background(Color(uiColor: .tertiarySystemGroupedBackground), in: RoundedRectangle(cornerRadius: 10))
    }
}
