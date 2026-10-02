import SwiftUI

func gramText(_ gram: [Literal_String]) -> String {
    gram.map { $0.word.text + $0.whitespace }.joined().trimmingCharacters(in: .whitespacesAndNewlines)
}

struct FlashcardChallengeView: View {
    @Environment(BackgroundController.self) private var background
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
    /// Per-meaning grades on a written card with several meanings; the grade
    /// buttons then apply to whichever meanings are left unmarked.
    @State private var marks: [Int: Rating] = [:]
    private var view: FlashcardView {
        flashcard_view(flashcard: flashcard, is_new: isNew, total_card_count: screen.total_count,
                       times_type_seen: timesTypeSeen, target_language: screen.target_language,
                       native_language: screen.native_language)
    }
    private var canGrade: Bool { hasOpened || revealed || !view.require_answer_reveal }
    private var listening: Bool { if case .Listening = flashcard.content { true } else { false } }
    private var meanings: [FlashcardMeaning] { if case let .Gram(_, _, _, meanings) = flashcard.content { meanings } else { [] } }
    private var allMarked: Bool { meanings.count > 1 && marks.count == meanings.count }
    private var againLabel: String { (allMarked ? view.continue_label : marks.isEmpty ? nil : view.again_rest_label) ?? view.again_label }
    private var rememberedLabel: String { (allMarked ? view.continue_label : marks.isEmpty ? nil : view.remembered_rest_label) ?? view.remembered_label }
    var body: some View {
        ReviewStepScrollView {
            if !revealed, let prompt = view.tutorial_prompt {
                TutorialHint(prompt: prompt).fadeIn()
            }
            StudyCard(animated: true) {
                // A listening card leads with the big visualizer; a written one puts
                // the word first with its audio top-right, like the sentence challenges.
                if listening {
                    if let request = flashcard.audio {
                        AudioButton(request: request, reviewCount: screen.total_reviews, autoplay: true, kind: .visualizer)
                            .frame(maxWidth: .infinity)
                    }
                } else if case let .Gram(gram, prefix, _, _) = flashcard.content {
                    HStack(alignment: .center, spacing: 12) {
                        (Text(prefix.map { $0.prefix + $0.separator } ?? "").foregroundStyle(Color.yapMuted.opacity(0.6))
                            + Text(gramText(gram)))
                            .font(.system(size: 34, weight: .bold, design: .rounded)).textSelection(.enabled)
                            .frame(maxWidth: .infinity, alignment: .leading)
                        if let request = flashcard.audio {
                            AudioButton(request: request, reviewCount: screen.total_reviews, autoplay: revealed, kind: .prominent)
                        }
                    }
                }
                if let subtitle = view.subtitle {
                    Text(subtitle).font(.footnote).foregroundStyle(Color.yapMuted).frame(maxWidth: .infinity, alignment: listening ? .center : .leading)
                }
                Divider()
                if revealed {
                    VStack(alignment: .leading, spacing: 12) {
                        if meanings.count > 1 {
                            ForEach(Array(meanings.enumerated()), id: \.offset) { index, meaning in
                                MeaningRow(meaning: meaning, newLabel: view.new_label, mark: Binding(
                                    get: { marks[index] }, set: { marks[index] = $0 }))
                            }
                        } else {
                            FlashcardAnswer(content: flashcard.content, view: view).equatable()
                        }
                    }.fadeIn(duration: 0.2)
                } else {
                    Label(view.reveal_label, systemImage: "chevron.down")
                        .font(.subheadline.weight(view.require_answer_reveal ? .bold : .regular))
                        .foregroundStyle(view.require_answer_reveal ? Color.yapText : Color.yapMuted)
                        .frame(maxWidth: .infinity, minHeight: 44)
                }
            }
            .contentShape(Rectangle())
            .onTapGesture { toggle() }
            .accessibilityAction(named: revealed ? "Hide answer" : "Reveal answer") { toggle() }
            .swipeToGrade(enabled: canGrade && !actions.submitting, againLabel: againLabel,
                          rememberedLabel: rememberedLabel, rate: rate)
            // Like the web, the breakdown sits under the card rather than inside it.
            if revealed, case let .Gram(_, _, breakdown, _) = flashcard.content, let breakdown, !breakdown.isEmpty {
                MorphemeBreakdownView(parts: breakdown, alignment: .center, revealDelay: 1.5).padding(.top, 12)
            }
            if !revealed, let hint = view.tutorial_hidden_hint {
                TutorialHint(text: hint, pointing: .up).fadeIn(duration: 0.3, delay: 1.5)
            }
        } actions: {
            if revealed, let hint = view.tutorial_revealed_hint {
                TutorialHint(text: hint, pointing: .down, arrowSize: 96).fadeIn(duration: 0.3, delay: 1.5)
            }
            if !revealed, let label = view.cant_listen_label {
                CantListenButton(label: label) { actions.cantListen() }
            }
            if canGrade {
                Group {
                    if allMarked {
                        Button { rate(.Remembered) } label: { Text(rememberedLabel).frame(maxWidth: .infinity) }
                            .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
                            .keyboardShortcut(.rightArrow, modifiers: [])
                    } else {
                        GradeButtons(againLabel: againLabel, rememberedLabel: rememberedLabel, rate: rate)
                    }
                }
                .disabled(actions.submitting).fadeIn(duration: 0.2)
            }
        } footer: {
            ReportIssueLink(subject: .Flashcard(flashcard.content))
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
    /// `rest` grades every meaning the learner hasn't marked individually.
    private func rate(_ rest: Rating) {
        guard canGrade, !actions.submitting else { return }
        background.bump(30)
        audio.stop()
        let reviews: [CardReview]
        let overall: Rating
        if meanings.isEmpty {
            reviews = [CardReview(card: indicator, rating: rest)]
            overall = rest
        } else {
            let ratings = meanings.indices.map { marks[$0] ?? rest }
            reviews = zip(meanings, ratings).flatMap { meaning, rating in meaning.cards.map { CardReview(card: $0, rating: rating) } }
            overall = ratings.allSatisfy { $0 == .Again } ? .Again : .Remembered
        }
        actions.rate(reviews)
        if overall != .Again { audio.playEffect("success-\(Int.random(in: 1...3))") }
    }
}

/// One meaning of a written card with several: what it is, whether it's new,
/// and the learner's own ✗ / ✓ for it, tinted once marked.
private struct MeaningRow: View {
    let meaning: FlashcardMeaning
    let newLabel: String?
    @Binding var mark: Rating?
    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 8) {
                if let label = meaning.label { MeaningLabel(text: label) }
                if meaning.is_new, let newLabel {
                    Text(newLabel).font(.caption.weight(.medium)).foregroundStyle(Color.yapAccent)
                        .padding(.horizontal, 8).padding(.vertical, 2)
                        .background(Color.yapAccent.opacity(0.15), in: Capsule())
                }
                Spacer(minLength: 0)
                toggle(.Again, systemImage: "xmark", tint: .yapNegative)
                toggle(.Remembered, systemImage: "checkmark", tint: .yapPositive)
            }
            ForEach(Array(meaning.definition.senses.enumerated()), id: \.offset) { _, sense in
                SenseView(sense: sense, morphology: meaning.definition.morphology_label)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading).padding(14)
        .background(tint.map { $0.opacity(0.15) } ?? Color(uiColor: .systemBackground).opacity(0.35),
                    in: RoundedRectangle(cornerRadius: 12, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .strokeBorder(tint.map { $0.opacity(0.5) } ?? Color(uiColor: .separator).opacity(0.4))
        }
        .animation(.easeOut(duration: 0.2), value: mark)
    }
    private var tint: Color? { mark.map { $0 == .Again ? .yapNegative : .yapPositive } }
    /// Tapping the active mark clears it again.
    private func toggle(_ rating: Rating, systemImage: String, tint: Color) -> some View {
        let active = mark == rating
        return Button { mark = active ? nil : rating } label: {
            Image(systemName: systemImage).font(.system(size: 14, weight: .bold))
                .foregroundStyle(active ? .white : Color.yapMuted)
                .frame(width: 36, height: 36)
                .background(active ? tint : Color(uiColor: .systemBackground).opacity(0.6), in: Circle())
                .overlay { if !active { Circle().strokeBorder(Color(uiColor: .separator).opacity(0.6)) } }
        }
        .buttonStyle(.plain)
        .frame(minWidth: 44, minHeight: 44)
        .accessibilityLabel(rating == .Again ? "Forgot" : "Remembered")
        .accessibilityAddTraits(active ? .isSelected : [])
    }
}

private struct MeaningLabel: View {
    let text: String
    var body: some View {
        Text(text.uppercased()).font(.caption.weight(.medium)).tracking(1).foregroundStyle(Color.yapMuted)
    }
}

/// One sense: the gloss with its morphology trailing, then the example quoted
/// with a small play button beside it.
private struct SenseView: View {
    @Environment(\.reviewHost!) private var host
    let sense: DefinitionSense
    let morphology: String
    var showsNote = false
    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(sense.meaning).font(.title3.weight(.medium))
                Spacer(minLength: 0)
                if !morphology.isEmpty {
                    Text(morphology).font(.caption).italic().foregroundStyle(Color.yapMuted).multilineTextAlignment(.trailing)
                }
            }
            if showsNote, let note = sense.note { Text(note).font(.footnote).foregroundStyle(Color.yapMuted) }
            if let example = sense.example {
                HStack(alignment: .top, spacing: 4) {
                    AudioButton(request: AudioRequest(request: TtsRequest(text: example.target, language: host.deck.get_target_language(), is_ssml: false,
                        instructions: nil, speed: 1, verification_hints: []), provider: .ElevenLabs),
                        reviewCount: host.deck.get_total_reviews())
                    DefinitionExamples(example: example).frame(minHeight: 44, alignment: .center)
                }
            }
        }
    }
}

/// The web's CardBack: each sense in its own muted box, the morphology trailing
/// the meaning, the example quoted with a small play button beside it.
struct DefinitionBoxesView: View {
    let definition: DefinitionView
    /// Flashcards leave the note out, like the web.
    var showsNotes = true

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            ForEach(Array(definition.senses.enumerated()), id: \.offset) { _, sense in
                SenseView(sense: sense, morphology: definition.morphology_label, showsNote: showsNotes)
                    .frame(maxWidth: .infinity, alignment: .leading).padding(12)
                    .insetSurface()
            }
        }
    }
}

/// No callbacks here: grading and reveal state remain in the challenge host.
private struct FlashcardAnswer: View, Equatable {
    let content: CardContent
    let view: FlashcardView
    @ViewBuilder var body: some View {
        switch content {
        case let .Gram(_, _, _, meanings):
            ForEach(Array(meanings.enumerated()), id: \.offset) { _, meaning in
                VStack(alignment: .leading, spacing: 8) {
                    if let label = meaning.label { MeaningLabel(text: label) }
                    DefinitionBoxesView(definition: meaning.definition, showsNotes: false)
                }
            }
        case let .Listening(possible):
            if let header = view.listening_header { Text(header).font(.footnote).foregroundStyle(Color.yapMuted) }
            ForEach(Array(possible.enumerated()), id: \.offset) { _, entry in
                VStack(alignment: .leading, spacing: 8) {
                    HStack(alignment: .firstTextBaseline, spacing: 8) {
                        Text(gramText(entry.second)).font(.title3.weight(.medium))
                        if possible.count > 1 && entry.first { Text(view.known_label).font(.footnote).foregroundStyle(Color.yapPositiveForeground) }
                    }
                    ForEach(Array(entry.third.enumerated()), id: \.offset) { _, definition in
                        DefinitionBoxesView(definition: definition, showsNotes: false)
                    }
                }
            }
        }
    }
}
