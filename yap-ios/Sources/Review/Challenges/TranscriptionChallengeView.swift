import SwiftUI

struct TranscriptionChallengeView: View {
    @Environment(BackgroundController.self) private var background
    @Environment(AudioPlayer.self) private var audio
    @Environment(\.reviewScreen!) private var screen
    @Environment(\.reviewHost!) private var host
    @Environment(\.reviewActions!) private var actions
    let sentence: TranscribeComprehensibleSentence
    @State private var state: TranscriptionState
    // Recomputed once per step rather than on every access, since each call crosses the bridge.
    @State private var view: TranscriptionView
    @State private var hasClip: Bool?
    @State private var clipMovieId: String?
    @State private var gradesExpanded = false
    @AppStorage("mobile-keyboard-tip-dismissed") private var keyboardTipDismissed = false
    @State private var gradingTask: Task<Void, Never>?
    @FocusState private var focused: Int?
    init(sentence: TranscribeComprehensibleSentence, initialState: TranscriptionState?) {
        self.sentence = sentence
        let state = initialState ?? transcription_start(parts: sentence.parts, proper_noun_definitions: sentence.proper_noun_definitions)
        _state = State(initialValue: state)
        let view = transcription_view(state: state)
        _view = State(initialValue: view)
        _gradesExpanded = State(initialValue: view.verdict?.word_grades_open_by_default ?? false)
    }
    private var storage: PendingReview? {
        guard let scope = actions.pendingReviewKey else { return nil }
        let slot = transcription_pending_slot(parts: sentence.parts, scope: scope, app_version: get_app_version(), review_count: screen.total_reviews)
        return PendingReview(key: slot.key, identity: slot.identity)
    }
    private var blanks: [Int] { view.blanks.map { Int($0.index) } }
    private var editing: Bool { if case .Editing = state.phase { true } else { false } }
    var body: some View {
        ReviewStepScrollView {
            StudyCard(spacing: 20, animated: true) {
                if sentence.second_chance { ReviewBadge(text: "Second chance") }
                // Like the web: a big speaker on top, then the sentence with its blanks inline.
                VStack(spacing: 4) {
                    AudioButton(request: sentence.audio, reviewCount: screen.total_reviews, autoplay: true, kind: .visualizer)
                    Text(view.instructions).font(.footnote).foregroundStyle(Color.yapMuted)
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
                if editing { ProperNounGroupsView(groups: view.proper_nouns).equatable() }
                VideoClipView( language: screen.target_language, text: sentence.target_language,
                    reviewCount: screen.total_reviews,
                    maskedSentence: editing ? sentence.parts.map { part in
                        switch part { case let .Provided(literal): literal.word.text + literal.whitespace
                        case let .AskedToTranscribe(parts): parts.map { "____" + $0.whitespace }.joined() }
                    }.joined() : nil, available: $hasClip, movieId: $clipMovieId)
                if let verdict = view.verdict {
                    // The typed answer is already inline in the sentence above, so
                    // only the grading list repeats it.
                    SentenceVerdictView(headline: verdict.headline, correctLabel: view.correct_label, correct: sentence.target_language,
                        feedbackLabel: verdict.feedback_label, encouragement: verdict.encouragement,
                        explanation: verdict.explanation, error: verdict.autograding_error).equatable()
                    if !verdict.compare.isEmpty { compareRow(verdict) }
                    Button { send(.TranslationToggled) } label: {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(verdict.translation_label).font(.footnote.weight(.medium))
                            Text(sentence.native_language).font(.body.weight(.medium))
                                .blur(radius: verdict.translation_revealed ? 0 : 5)
                                .animation(.easeOut(duration: 0.1), value: verdict.translation_revealed)
                        }.frame(maxWidth: .infinity, alignment: .leading).padding(12)
                            .overlay { RoundedRectangle(cornerRadius: 10).strokeBorder(Color(uiColor: .separator)) }
                            .contentShape(Rectangle())
                    }.buttonStyle(.plain)
                    if !verdict.word_grades.isEmpty {
                        GradeSectionDisclosure(title: verdict.word_grades_title, isExpanded: $gradesExpanded) { wordGrades(verdict) }
                    }
                    if case let .Graded(_, grade, _, _) = state.phase {
                        ReviewDefinitionsView(definitions: get_transcription_review_definitions(challenge: sentence, results: grade.results)).equatable()
                    }
                } else if view.is_grading {
                    SentenceVerdictView(headline: nil, correctLabel: view.correct_label, correct: sentence.target_language).equatable()
                }
            }
            ReportIssueLink(subject: .Transcription(sentence))
            if editing, let tip = screen.keyboard_tip, !keyboardTipDismissed {
                HStack(spacing: 8) {
                    (Text("Tip:").fontWeight(.medium) + Text(" " + tip)).font(.subheadline).foregroundStyle(Color.yapMuted)
                        .frame(maxWidth: .infinity, alignment: .leading)
                    Button("Dismiss tip", systemImage: "xmark") { keyboardTipDismissed = true }
                        .labelStyle(.iconOnly).font(.footnote).buttonStyle(.plain).frame(width: 32, height: 32)
                }.padding(12)
                    .background(Color.yapMutedSurface.opacity(0.3), in: RoundedRectangle(cornerRadius: 10))
                    .overlay { RoundedRectangle(cornerRadius: 10).strokeBorder(Color(uiColor: .separator)) }
            }
            if editing {
                MoviePosterGrid(movies: host.deck.sentence_posters(movie_ids: sentence.movie_titles.map { $0.first }, shown_in_clip: clipMovieId)).equatable()
            }
        } actions: {
            if view.verdict == nil {
                Button(view.cant_listen_label) { actions.cantListen() }.font(.footnote).foregroundStyle(Color.yapMuted).frame(minHeight: 44).disabled(view.is_grading)
                HStack {
                    Button { submit() } label: { Text(view.submit_label).frame(maxWidth: .infinity) }.disabled(!view.can_submit)
                        .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
                    if view.is_grading { Button("Cancel") { send(.CancelGrading) } }
                }
            } else {
                Button { complete() } label: { Text(view.verdict?.continue_label ?? "").frame(maxWidth: .infinity) }.disabled(!view.can_continue || actions.submitting)
                    .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
            }
        }
        .onAppear {
            if let data = storage?.load(Data.self), let saved = try? PendingReview.decode(data, as: TranscriptionState.self) {
                state = saved
            }
            if editing { focused = blanks.first }
            apply(transcription_resume(state: state))
        }
        .onDisappear { gradingTask?.cancel() }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in guard DebugHarness.shared.activeScreen == .review else { return }; debugCommand() }
        #endif
    }
    private let sentenceFont = Font.title2.weight(.semibold)
    private let fieldFont = Font.body.weight(.semibold)
    /// The web's inline input: a bordered chip at least as wide as a word (a
    /// lone blank gets more room) that grows with what's typed (a hidden twin
    /// of the text sets the width), underlined with dots and tinted by its
    /// grade once checked. The placeholder is drawn behind the field rather
    /// than inside it: an empty centered field with its own prompt shows no
    /// caret on iOS.
    private func blank(_ index: Int) -> some View {
        let blank = view.blanks.first { $0.index == UInt64(index) }!
        let text = blank.text
        let shape = RoundedRectangle(cornerRadius: 6)
        return Text(text.isEmpty ? view.placeholder : text).font(fieldFont)
            .foregroundStyle(Color.yapMuted).opacity(text.isEmpty ? 1 : 0)
            .overlay {
                TextField("", text: Binding(get: { text }, set: { send(.InputChanged(index: UInt64(index), text: $0)) }))
                    .textFieldStyle(.plain).font(fieldFont).multilineTextAlignment(.center).focused($focused, equals: index)
                    .autocorrectionDisabled().textInputAutocapitalization(index == 0 ? .sentences : .never)
                    .submitLabel(index == blanks.last ? .done : .next).onSubmit { advance(index) }
                    .disabled(!blank.editable)
            }
            .padding(.horizontal, 12).padding(.vertical, 4)
            .frame(minWidth: blanks.count == 1 ? 256 : 128)
            .background(field(blank.tint), in: shape)
            .overlay { shape.strokeBorder(focused == index ? Color.yapAccent.opacity(0.5) : Color.yapInput, lineWidth: 1) }
            .overlay(alignment: .bottom) {
                DottedUnderline().stroke(tint(blank.tint), style: StrokeStyle(lineWidth: 3, lineCap: .round, dash: [0, 6])).frame(height: 3)
                    .padding(.horizontal, 3)
            }
            .opacity(blank.editable ? 1 : 0.5)
            .animation(.easeOut(duration: 0.15), value: focused)
            .padding(.horizontal, 4).padding(.vertical, 2)
    }
    private func tint(_ tint: BlankTint) -> Color {
        switch tint {
        case .Neutral: Color.yapMuted.opacity(0.3)
        case .Perfect: .yapPositive
        case .Typo, .PhoneticallyIdentical: .yapCaution
        case .PhoneticallySimilar: .yapWarning
        case .Wrong: .yapNegative
        }
    }
    private func field(_ tint: BlankTint) -> Color {
        switch tint {
        case .Neutral: Color.yapInput.opacity(0.3)
        case .Perfect: .yapPositiveField
        case .Typo, .PhoneticallyIdentical: .yapCautionField
        case .PhoneticallySimilar: .yapWarningField
        case .Wrong: .yapNegativeField
        }
    }
    @ViewBuilder private func wordGrades(_ verdict: VerdictView) -> some View {
        (Text(verdict.submission_label + " ") + Text(verdict.submission_text).foregroundStyle(Color.yapText))
            .font(.subheadline).foregroundStyle(Color.yapMuted).frame(maxWidth: .infinity, alignment: .leading)
        ForEach(Array(verdict.word_grades.enumerated()), id: \.offset) { _, word in
            HStack(spacing: 4) {
                Text(word.heard).fontWeight(.medium)
                Spacer()
                Picker("Grade \(word.heard)", selection: Binding(get: { Int(word.selected) }, set: { index in
                    send(.WordGradeChanged(part_index: word.part_index, word_index: word.word_index, grade: view.grade_options[index].grade))
                })) {
                    ForEach(view.grade_options.indices, id: \.self) { index in Text(view.grade_options[index].label).tag(index) }
                }.pickerStyle(.menu).controlSize(.small)
            }.font(.subheadline).padding(.horizontal, 8).padding(.vertical, 4)
                .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8))
        }
    }
    /// The words the learner confused, read slowly one after another.
    private func compareRow(_ verdict: VerdictView) -> some View {
        HStack(spacing: 12) {
            Text(verdict.compare_label).font(.subheadline.weight(.medium))
            AudioButton(request: AudioRequest(request: TtsRequest(text: verdict.compare.map { $0 + ";" }.joined(separator: " "),
                language: screen.target_language, is_ssml: false, instructions: nil, speed: 0.8, verification_hints: []), provider: .Google),
                reviewCount: screen.total_reviews)
            Text(verdict.compare.joined(separator: ",  ")).fontWeight(.medium).frame(maxWidth: .infinity)
        }.padding(.horizontal, 12).padding(.vertical, 4)
            .overlay { RoundedRectangle(cornerRadius: 10).strokeBorder(Color(uiColor: .separator)) }
    }
    private func advance(_ index: Int) {
        if let position = blanks.firstIndex(of: index), position + 1 < blanks.count { focused = blanks[position + 1] }
        else { submit() }
    }
    private func send(_ event: TranscriptionEvent) {
        if case .CancelGrading = event { gradingTask?.cancel() }
        apply(transcription_transition(state: state, event: event))
    }
    private func apply(_ step: TranscriptionStep) {
        let hadVerdict = view.verdict != nil
        state = step.state
        view = transcription_view(state: state)
        if !hadVerdict { gradesExpanded = view.verdict?.word_grades_open_by_default ?? false }
        if let data = try? PendingReview.encode(state) { storage?.save(data) }
        for effect in step.effects {
            switch effect {
            case let .Autograde(submission):
                background.bump(30)
                let course = Course(native_language: screen.native_language, target_language: screen.target_language)
                focused = nil
                gradingTask?.cancel()
                gradingTask = Task { @MainActor in
                    let grade = await autograde_transcription(submission: submission, access_token: host.accessToken, course: course, movie_titles: MovieTitles(value: sentence.movie_titles))
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
                if actions.completeTranscription(results, completedAtMs) { background.bump(30); storage?.clear(); audio.stop() }
            }
        }
    }
    private func submit() { send(.Submit(now_ms: Date().timeIntervalSince1970 * 1000)) }
    private func complete() { send(.Continue) }
    #if DEBUG
    private func debugCommand() {
        let command = DebugHarness.shared.command
        if command.hasPrefix("dump-fixture ") {
            var capture = screen
            capture.step = .Challenge(ChallengeView(challenge: .TranscribeComprehensibleSentence(sentence), transcription: state, translation: nil))
            DebugHarness.dumpFixture(.Review(capture), name: String(command.dropFirst(13)))
        }
        if command.hasPrefix("type "), editing, let index = focused ?? blanks.first { send(.InputChanged(index: UInt64(index), text: String(command.dropFirst(5)))) }
        if command == "type-reference", editing {
            for index in blanks { if case let .AskedToTranscribe(parts) = sentence.parts[index] { send(.InputChanged(index: UInt64(index), text: gramText(parts))) } }
        }
        if command == "submit" { submit() }
        if command == "continue" { complete() }
        if command == "cancel" { send(.CancelGrading) }
        if command.hasPrefix("focus "), let index = Int(command.dropFirst(6)), blanks.contains(index) { focused = index }
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
