import SwiftUI

struct TranslationChallengeView: View {
    @Environment(AudioPlayer.self) private var audio
    let model: ReviewModel
    let sentence: TranslateComprehensibleSentence
    @State private var state: TranslationState
    @State private var view: TranslationView
    @State private var hasClip: Bool?
    @State private var gradingTask: Task<Void, Never>?
    @State private var focused = false
    @State private var gradesExpanded = false

    init(model: ReviewModel, sentence: TranslateComprehensibleSentence, course: Course) {
        self.model = model
        self.sentence = sentence
        let state = translation_start(sentence: sentence, course: course)
        _state = State(initialValue: state)
        _view = State(initialValue: translation_view(state: state))
    }
    private var storage: PendingReview? {
        #if DEBUG
        if DebugHarness.shared.fixture != nil { return nil }
        #endif
        return PendingReview(kind: "translation", challenge: sentence, model: model)
    }
    private var editing: Bool { if case .Editing = state.phase { true } else { false } }
    var body: some View {
        VStack(spacing: 12) {
            StudyCard {
                if let badge = view.badge { ReviewBadge(text: badge) }
                HStack(alignment: .center, spacing: 8) {
                    AudioButton(request: sentence.audio, session: model.session, reviewCount: model.deck.get_total_reviews(), autoplay: !editing && hasClip == false)
                    SentenceFlow(spacing: 0, alignment: .center) {
                        ForEach(Array(view.words.enumerated()), id: \.offset) { index, word in
                            let text = Text(word.text + word.whitespace)
                                .underline(word.tappable, pattern: .dot)
                                .font(.title2.weight(.semibold)).foregroundStyle(tint(word.tint))
                            if word.tappable {
                                Button { send(.WordTapped(index: UInt64(index))) } label: { text.frame(minHeight: 44) }.buttonStyle(.plain)
                            } else { text }
                        }
                    }.frame(maxWidth: .infinity)
                }
                if let verdict = view.verdict {
                    SentenceVerdictView(submission: verdict.submission, correct: verdict.correct_translation,
                        perfect: verdict.perfect, encouragement: verdict.encouragement, explanation: verdict.explanation,
                        error: verdict.autograding_error, correctLabel: verdict.correct_label, submissionLabel: verdict.submission_label)
                    if let section = view.grade_section {
                        DisclosureGroup(section.title, isExpanded: $gradesExpanded) {
                            VStack(spacing: 12) {
                                Text(section.subtitle).font(.caption).foregroundStyle(.secondary)
                                ForEach(Array(section.items.enumerated()), id: \.offset) { index, item in gradeRow(item, index: index) }
                            }
                        }
                    }
                } else if view.is_grading {
                    Text(state.text)
                    Text(view.correct_translation ?? "").foregroundStyle(.green)
                    ProgressView(view.submit_label)
                } else {
                    SubmissionTextView(text: Binding(get: { state.text }, set: { send(.TextChanged(text: $0)) }), focused: $focused, onSubmit: submit)
                        .overlay(alignment: .topLeading) {
                            if state.text.isEmpty { Text(view.placeholder).foregroundStyle(.secondary).padding(12).allowsHitTesting(false) }
                        }
                    ForEach(Array(view.proper_nouns.enumerated()), id: \.offset) { _, entry in
                        Text("\(entry.first): \(entry.second.learner_native_language_translation)").font(.subheadline)
                        if let description = entry.second.description { Text(description).font(.caption) }
                    }
                }
                VideoClipView(deck: model.deck, language: model.deck.get_target_language(), text: sentence.target_language,
                    session: model.session, reviewCount: model.deck.get_total_reviews(), autoplay: !editing, available: $hasClip)
                ReviewDefinitionsView(definitions: view.definitions)
            }
            if view.verdict != nil {
                Button { send(.Continue) } label: { Text(view.continue_label).frame(maxWidth: .infinity) }
                    .disabled(!view.can_continue || model.submitting)
                    .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
            } else {
                HStack {
                    Button { submit() } label: { Text(view.submit_label).frame(maxWidth: .infinity) }.disabled(!view.can_submit)
                        .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
                    if view.is_grading { Button("Cancel") { send(.CancelGrading) } }
                }
            }
        }
        .onAppear {
            #if DEBUG
            if let saved = DebugHarness.shared.challengeFixture?.translation { state = saved }
            #endif
            if let data = storage?.load(Data.self), let saved = try? PendingReview.decode(data, as: TranslationState.self) { state = saved }
            focused = editing
            apply(translation_resume(state: state))
        }
        .onDisappear { gradingTask?.cancel() }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in guard DebugHarness.shared.activeTab == .learn else { return }; debugCommand() }
        #endif
    }
    private func tint(_ tint: TranslationWordTint) -> Color {
        switch tint {
        case .Neutral: .yapText
        case .Perfect, .Remembered: .green
        case .Tapped: .yellow
        case .Forgot: .red
        }
    }
    private func gradeRow(_ item: TranslationGradeItemView, index: Int) -> some View {
        let display = item.label
        let status = item.grade.map { $0 == .Remembered }
        return VStack(alignment: .leading, spacing: 4) {
            Text(display).fontWeight(.semibold)
            Picker("Grade \(display)", selection: Binding<Int?>(get: { status.map { $0 ? 1 : 0 } }, set: { value in
                if let value { send(.ItemGraded(item_index: UInt64(index), grade: value == 1 ? .Remembered : .Forgot)) }
            })) {
                Text("Forgot").tag(Int?.some(0))
                Text("Remembered").tag(Int?.some(1))
            }.pickerStyle(.segmented).controlSize(.small)
        }.font(.subheadline)
    }
    private func send(_ event: TranslationEvent) {
        if case .CancelGrading = event { gradingTask?.cancel() }
        apply(translation_transition(state: state, event: event))
    }
    private func apply(_ step: TranslationStep) {
        let hadGradeSection = view.grade_section != nil
        state = step.state
        view = translation_view(state: state)
        if !hadGradeSection { gradesExpanded = view.grade_section?.open_by_default ?? false }
        if let data = try? PendingReview.encode(state) { storage?.save(data) }
        for effect in step.effects {
            switch effect {
            case let .Autograde(submission):
                focused = false
                gradingTask?.cancel()
                let course = state.course
                gradingTask = Task { @MainActor in
                    if model.session.online {
                        let response = await autograde_translation(challenge_sentence: sentence.target_language, user_sentence: submission,
                            native_translations: sentence.native_translations, literals: sentence.target_language_literals,
                            phrases: sentence.unique_target_language_phrases, access_token: model.session.accessToken(), course: course,
                            gram_definitions: GramDefinitions(value: sentence.gram_definitions_for_lookup), literal_gram_indices: sentence.literal_gram_indices,
                            phrase_definitions: GramDefinitions(value: sentence.phrase_definitions), primary_expression: sentence.primary_expression,
                            movie_titles: MovieTitles(value: sentence.movie_titles))
                        guard !Task.isCancelled else { return }
                        send(.Graded(response: response))
                    } else {
                        guard !Task.isCancelled else { return }
                        send(.GradingFailed(message: "You're offline"))
                    }
                    #if DEBUG
                    DebugHarness.log("translation graded: perfect=\(view.verdict?.perfect ?? false)")
                    #endif
                }
            case let .PlaySound(sound):
                switch sound {
                case .AiDoneGrading: audio.playEffect("ai-done-grading")
                case .Success: audio.playEffect("success-1")
                }
            case let .Complete(outcome, tapped, submission, completedAtMs):
                let completed: Bool
                switch outcome {
                case .Perfect: completed = model.completeTranslationPerfect(sentence.target_language, tapped: tapped, completedAtMs: completedAtMs)
                case let .Manual(grade): completed = model.completeTranslationWrong(sentence.target_language, submission: submission, grade: grade, tapped: tapped, completedAtMs: completedAtMs)
                }
                if completed { storage?.clear(); audio.stop() }
            }
        }
    }
    private func submit() { send(.Submit(now_ms: ReviewModel.now)) }
    #if DEBUG
    private func debugCommand() {
        let command = DebugHarness.shared.command
        if command.hasPrefix("dump-fixture ") {
            DebugHarness.dumpFixture(.Challenge(ChallengeFixture(challenge: .TranslateComprehensibleSentence(sentence), transcription: nil, translation: state)), name: String(command.dropFirst(13)))
        }
        if command.hasPrefix("type ") { send(.TextChanged(text: String(command.dropFirst(5)))) }
        if command == "type-reference" { send(.TextChanged(text: sentence.native_translations.first ?? "")) }
        if command == "submit" { submit() }
        if command == "continue" { send(.Continue) }
        if command == "cancel" { send(.CancelGrading) }
        let parts = command.split(separator: " ")
        if parts.count == 2, ["tap", "tap-word"].contains(parts[0]), let index = UInt64(parts[1]) { send(.WordTapped(index: index)) }
        if parts.count == 3, parts[0] == "grade", let index = UInt64(parts[1]), ["forgot", "remembered"].contains(parts[2]) {
            send(.ItemGraded(item_index: index, grade: parts[2] == "remembered" ? .Remembered : .Forgot))
        }
        if command.hasPrefix("toggle-phrase "), let index = Int(command.dropFirst(14)), let items = view.grade_section?.items, items.indices.contains(index) {
            send(.ItemGraded(item_index: UInt64(index), grade: items[index].grade == .Remembered ? .Forgot : .Remembered))
        }
    }
    #endif
}
