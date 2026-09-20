import SwiftUI

struct TranscriptionChallengeView: View {
    @Environment(AudioPlayer.self) private var audio
    let model: ReviewModel
    let sentence: TranscribeComprehensibleSentence
    @State private var state: TranscriptionState
    // Recomputed once per step rather than on every access, since each call crosses the bridge.
    @State private var view: TranscriptionView
    @State private var hasClip: Bool?
    @State private var gradingTask: Task<Void, Never>?
    @FocusState private var focused: Int?
    init(model: ReviewModel, sentence: TranscribeComprehensibleSentence) {
        self.model = model
        self.sentence = sentence
        let state = transcription_start(parts: sentence.parts)
        _state = State(initialValue: state)
        _view = State(initialValue: transcription_view(state: state))
    }
    private var storage: PendingReview? {
        #if DEBUG
        if DebugHarness.shared.fixture != nil { return nil }
        #endif
        return PendingReview(kind: "transcription", challenge: sentence, model: model)
    }
    private var blanks: [Int] { view.blanks.map { Int($0.index) } }
    private var editing: Bool { if case .Editing = state.phase { true } else { false } }
    var body: some View {
        VStack(spacing: 12) {
            StudyCard {
                if sentence.second_chance { ReviewBadge(text: "Second chance") }
                // Like the web: a big speaker on top, then the sentence with its blanks inline.
                VStack(spacing: 4) {
                    AudioButton(request: sentence.audio, session: model.session, reviewCount: model.deck.get_total_reviews(), autoplay: true, hero: true)
                    Text(view.instructions).font(.footnote).foregroundStyle(.secondary)
                }.frame(maxWidth: .infinity)
                SentenceFlow(spacing: 0, alignment: .center) {
                    ForEach(Array(sentence.parts.enumerated()), id: \.offset) { index, part in
                        switch part {
                        case let .Provided(literal): Text(literal.word.text + literal.whitespace).font(sentenceFont)
                        case let .AskedToTranscribe(parts):
                            blank(index)
                            if let whitespace = parts.last?.whitespace, !whitespace.isEmpty { Text(whitespace).font(sentenceFont) }
                        }
                    }
                }.frame(maxWidth: .infinity).padding(.top, 4)
                VideoClipView(deck: model.deck, language: model.deck.get_target_language(), text: sentence.target_language,
                    session: model.session, reviewCount: model.deck.get_total_reviews(),
                    maskedSentence: editing ? sentence.parts.map { part in
                        switch part { case let .Provided(literal): literal.word.text + literal.whitespace
                        case let .AskedToTranscribe(parts): parts.map { "____" + $0.whitespace }.joined() }
                    }.joined() : nil, available: $hasClip)
                if let verdict = view.verdict {
                    SentenceVerdictView(submission: verdict.submission_text,
                        correct: sentence.target_language, perfect: verdict.perfect, encouragement: verdict.encouragement,
                        explanation: verdict.explanation, error: verdict.autograding_error,
                        correctLabel: verdict.correct_label, submissionLabel: verdict.submission_label)
                    wordGrades(verdict)
                    if !verdict.compare.isEmpty {
                        Text(verdict.compare.joined(separator: " · "))
                        AudioButton(request: AudioRequest(request: TtsRequest(text: verdict.compare.map { $0 + ";" }.joined(separator: " "),
                            language: model.deck.get_target_language(), is_ssml: false, instructions: nil, speed: 0.8, verification_hints: []), provider: .Google),
                            session: model.session, reviewCount: model.deck.get_total_reviews())
                    }
                    DisclosureGroup("Translation", isExpanded: Binding(get: { verdict.translation_revealed }, set: { _ in send(.TranslationToggled) })) { Text(sentence.native_language) }
                    if case let .Graded(_, grade, _) = state.phase {
                        ReviewDefinitionsView(definitions: get_transcription_review_definitions(challenge: sentence, results: grade.results))
                    }
                } else if view.is_grading {
                    Text(sentence.target_language).foregroundStyle(.green)
                    ProgressView("Grading your answer…")
                }
            }
            if view.verdict == nil {
                Button { submit() } label: { Text(view.submit_label).frame(maxWidth: .infinity) }.disabled(!view.can_submit)
                    .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
                Button("I can't listen right now") { model.cantListen() }.font(.footnote).foregroundStyle(.secondary).frame(minHeight: 44).disabled(view.is_grading)
            } else {
                Button { complete() } label: { Text(view.verdict?.continue_label ?? "").frame(maxWidth: .infinity) }.disabled(model.submitting)
                    .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
            }
        }
        .toolbar {
            ToolbarItemGroup(placement: .keyboard) {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 16) {
                        ForEach(get_language_metadata(language: model.deck.get_target_language()).accented_characters, id: \.self) { character in
                            Button(character) { insertAccent(character) }
                        }
                    }
                }
                Button("Done") { focused = nil }
            }
        }
        .onAppear {
            #if DEBUG
            if let saved = DebugHarness.shared.challengeFixture?.transcription { state = saved }
            #endif
            if let data = storage?.load(Data.self), let saved = try? PendingReview.decode(data, as: TranscriptionState.self) {
                state = saved
            }
            if editing { focused = blanks.first }
            apply(transcription_resume(state: state))
        }
        .onDisappear { gradingTask?.cancel() }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in guard DebugHarness.shared.activeTab == .learn else { return }; debugCommand() }
        #endif
    }
    private let sentenceFont = Font.title2.weight(.semibold)
    /// A field that grows with what's typed (a hidden twin of the text sets the
    /// width) and is underlined with dots, tinted by its grade once checked.
    /// The placeholder is drawn behind the field rather than inside it: an
    /// empty centered field with its own prompt shows no caret on iOS.
    private func blank(_ index: Int) -> some View {
        let blank = view.blanks.first { $0.index == UInt64(index) }!
        let text = blank.text
        return Text(text.isEmpty ? view.placeholder : text).font(sentenceFont)
            .foregroundStyle(.secondary).opacity(text.isEmpty && focused != index ? 1 : 0)
            .overlay {
                TextField("", text: Binding(get: { text }, set: { send(.InputChanged(index: UInt64(index), text: $0)) }))
                    .textFieldStyle(.plain).font(sentenceFont).multilineTextAlignment(.center).focused($focused, equals: index)
                    .autocorrectionDisabled().textInputAutocapitalization(index == 0 ? .sentences : .never)
                    .submitLabel(index == blanks.last ? .done : .next).onSubmit { advance(index) }
                    .disabled(!blank.editable)
            }
            .padding(.horizontal, 6)
            .background(alignment: .bottom) {
                DottedUnderline().stroke(tint(blank.tint, focused: focused == index), style: StrokeStyle(lineWidth: 3, lineCap: .round, dash: [0, 6])).frame(height: 3)
                    .animation(.easeOut(duration: 0.15), value: focused)
            }
            .padding(.horizontal, 2)
    }
    private func tint(_ tint: BlankTint, focused: Bool) -> Color {
        switch tint {
        case .Neutral: focused ? Color.yapAccent : .secondary.opacity(0.4)
        case .Perfect: .green
        case .PhoneticallyIdentical: .yellow
        case .PhoneticallySimilar: .orange
        case .Wrong: .red
        }
    }
    @ViewBuilder private func wordGrades(_ verdict: VerdictView) -> some View {
        ForEach(Array(verdict.word_grades.enumerated()), id: \.offset) { _, word in
            HStack(spacing: 4) {
                Text(word.heard).fontWeight(.semibold)
                Spacer()
                Picker("Grade \(word.heard)", selection: Binding(get: { Int(word.selected) }, set: { index in
                    send(.WordGradeChanged(part_index: word.part_index, word_index: word.word_index, grade: view.grade_options[index].grade))
                })) {
                    ForEach(view.grade_options.indices, id: \.self) { index in Text(view.grade_options[index].label).tag(index) }
                }.pickerStyle(.menu).controlSize(.small)
            }.font(.subheadline)
        }
    }
    private func insertAccent(_ character: String) {
        // UIKit inserts at the current selection, including replacing selected
        // text, while the SwiftUI binding continues to own the field contents.
        guard focused != nil else { return }
        UIApplication.shared.sendAction(#selector(UIResponder.insertReviewAccent(_:)), to: nil, from: character, for: nil)
    }
    private func advance(_ index: Int) {
        if let position = blanks.firstIndex(of: index), position + 1 < blanks.count { focused = blanks[position + 1] }
        else { submit() }
    }
    private func send(_ event: TranscriptionEvent) {
        apply(transcription_transition(state: state, event: event))
    }
    private func apply(_ step: TranscriptionStep) {
        state = step.state
        view = transcription_view(state: state)
        if let data = try? PendingReview.encode(state) { storage?.save(data) }
        for effect in step.effects {
            switch effect {
            case let .Autograde(submission):
                guard let course = model.course else { continue }
                focused = nil
                gradingTask?.cancel()
                gradingTask = Task { @MainActor in
                    let grade: Grade
                    if model.session.online {
                        grade = await autograde_transcription(submission: submission, access_token: model.session.accessToken(), course: course, movie_titles: MovieTitles(value: sentence.movie_titles))
                    } else { grade = failed_transcription_review(submission: submission, course: course) }
                    guard !Task.isCancelled else { return }
                    send(.Graded(grade: grade))
                    #if DEBUG
                    DebugHarness.log("transcription graded: perfect=\(view.verdict?.perfect ?? false)")
                    #endif
                }
            case let .PlaySound(sound):
                switch sound {
                case .AiDoneGrading: audio.playEffect("ai-done-grading")
                case .Success: audio.playEffect("success-1")
                }
            case let .Complete(results, completedAtMs):
                if model.completeTranscription(results, completedAtMs: completedAtMs) { storage?.clear(); audio.stop() }
            }
        }
    }
    private func submit() { send(.Submit(now_ms: ReviewModel.now)) }
    private func complete() { send(.Continue) }
    #if DEBUG
    private func debugCommand() {
        let command = DebugHarness.shared.command
        if command.hasPrefix("dump-fixture ") {
            DebugHarness.dumpFixture(.Challenge(ChallengeFixture(challenge: .TranscribeComprehensibleSentence(sentence), transcription: state, translation: nil)), name: String(command.dropFirst(13)))
        }
        if command.hasPrefix("type "), editing, let index = focused ?? blanks.first { send(.InputChanged(index: UInt64(index), text: String(command.dropFirst(5)))) }
        if command == "type-reference", editing {
            for index in blanks { if case let .AskedToTranscribe(parts) = sentence.parts[index] { send(.InputChanged(index: UInt64(index), text: gramText(parts))) } }
        }
        if command == "submit" { submit() }
        if command == "continue" { complete() }
        if command.hasPrefix("focus "), let index = Int(command.dropFirst(6)), blanks.contains(index) { focused = index }
        if command.hasPrefix("accent ") { insertAccent(String(command.dropFirst(7))) }
        if command.hasPrefix("toggle-word ") {
            let indices = command.dropFirst(12).split(separator: " ").compactMap { Int($0) }
            if indices.count == 2, indices.allSatisfy({ $0 >= 0 }) { send(.WordGradeChanged(part_index: UInt64(indices[0]), word_index: UInt64(indices[1]), grade: .Perfect(wrote: nil))) }
        }
    }
    #endif
}

private struct DottedUnderline: Shape {
    func path(in rect: CGRect) -> Path {
        Path { $0.move(to: CGPoint(x: rect.minX, y: rect.midY)); $0.addLine(to: CGPoint(x: rect.maxX, y: rect.midY)) }
    }
}

extension UIResponder {
    @objc func insertReviewAccent(_ sender: Any?) {
        guard let input = self as? UIKeyInput, let text = sender as? String else { return }
        input.insertText(text)
    }
}
