import SwiftUI

struct TranslationChallengeView: View {
    @Environment(AudioPlayer.self) private var audio
    let model: ReviewModel
    let sentence: TranslateComprehensibleSentence
    @State private var draft = Draft()
    @State private var result: TranslationReviewResult?
    @State private var grading = false
    @State private var hasClip: Bool?
    @State private var gradingTask: Task<Void, Never>?
    @State private var focused = false
    private struct Draft: Codable, Equatable {
        var text = ""
        var tapped: [UInt64] = []
        var completedAtMs: Double?
        var correct = ""
        var result: Data?
    }
    private var storage: PendingReview { PendingReview(kind: "translation", challenge: sentence, model: model) }
    private var manual: ManualTranslationGrade? { if case let .Manual(grade) = result { grade } else { nil } }
    private var perfect: Bool { if case .Perfect = result { true } else { false } }
    private var feedback: TranslationReviewFeedback {
        get_translation_review_feedback(sentence: sentence, grade: manual, is_perfect: perfect,
            tapped_words: draft.tapped, language: model.deck.get_target_language())
    }
    var body: some View {
        VStack(spacing: 24) {
            StudyCard {
                Text(sentence.second_chance ? "TRANSLATION · SECOND CHANCE" : "TRANSLATION").font(.caption).foregroundStyle(.secondary)
                SentenceFlow(spacing: 0) {
                    ForEach(Array(sentence.target_language_literals.enumerated()), id: \.offset) { index, literal in
                        let word = Text(literal.word.text + literal.whitespace).font(.title2.weight(.semibold)).foregroundStyle(wordColor(index))
                        if result == nil && !grading {
                            Button { tap(index) } label: { word }.buttonStyle(.plain)
                        } else { word }
                    }
                }
                AudioButton(request: sentence.audio, session: model.session, reviewCount: model.deck.get_total_reviews(), autoplay: (grading || result != nil) && hasClip == false)
                if let result {
                    verdict(result)
                    if manual != nil {
                        ForEach(Array(feedback.grade_items.enumerated()), id: \.offset) { index, item in
                            gradeRow(item, index: index)
                        }
                    }
                } else if grading {
                    Text(draft.text)
                    Text(draft.correct).foregroundStyle(.green)
                    ProgressView("Grading your answer…")
                } else {
                    SubmissionTextView(text: $draft.text, focused: $focused, onSubmit: submit)
                        .overlay(alignment: .topLeading) {
                            if draft.text.isEmpty { Text("Translation…").foregroundStyle(.secondary).padding(12).allowsHitTesting(false) }
                        }
                    ForEach(Array(sentence.proper_noun_definitions.enumerated()), id: \.offset) { _, entry in
                        Text("\(entry.first): \(entry.second.learner_native_language_translation)").font(.subheadline)
                        if let description = entry.second.description { Text(description).font(.caption) }
                    }
                }
                VideoClipView(deck: model.deck, language: model.deck.get_target_language(), text: sentence.target_language,
                    session: model.session, reviewCount: model.deck.get_total_reviews(), autoplay: grading || result != nil, available: $hasClip)
                ReviewDefinitionsView(definitions: feedback.definitions)
            }
            if result != nil {
                Button(perfect ? "Nailed it!" : "Continue") { complete() }
                    .disabled(!feedback.can_continue || model.submitting)
                    .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
            } else {
                Button("Check answer") { submit() }.disabled(grading || draft.text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                    .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
            }
        }
        .onAppear {
            if let saved = storage.load(Draft.self) {
                draft = saved
                if let data = saved.result { result = try? PendingReview.decode(data, as: TranslationReviewResult.self) }
            }
            focused = result == nil
            // An interrupted request resumes with the original submit timestamp.
            if draft.completedAtMs != nil && result == nil { submit() }
        }
        .onDisappear { gradingTask?.cancel(); grading = false }
        .onChange(of: draft) { _, _ in storage.save(draft) }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in guard DebugHarness.shared.activeTab == .learn else { return }; debugCommand() }
        #endif
    }
    private func wordColor(_ index: Int) -> Color {
        if perfect { return .green }
        if draft.tapped.contains(UInt64(index)) || (sentence.literal_gram_indices.indices.contains(index) && feedback.tapped_gram_groups.contains(sentence.literal_gram_indices[index])) { return .orange }
        if let grades = manual?.literal_grades, grades.indices.contains(index), let grade = grades[index] { return grade == .Remembered ? .green : .red }
        return .yapText
    }
    @ViewBuilder private func verdict(_ result: TranslationReviewResult) -> some View {
        switch result {
        case let .Perfect(encouragement, explanation):
            SentenceVerdictView(submission: draft.text, correct: draft.correct, perfect: true, encouragement: encouragement, explanation: explanation, error: nil)
        case let .Manual(grade):
            SentenceVerdictView(submission: draft.text, correct: draft.correct, perfect: false, encouragement: grade.encouragement, explanation: grade.explanation, error: grade.autograding_error)
        }
    }
    private func gradeRow(_ item: TranslationGradeItem, index: Int) -> some View {
        let display: String, status: Bool?
        switch item { case let .Literal(_, text, value), let .Phrase(_, text, value): display = text; status = value }
        return VStack(alignment: .leading, spacing: 8) {
            Text(display).fontWeight(.semibold)
            Picker("Grade \(display)", selection: Binding<Int>(get: { status.map { $0 ? 1 : 0 } ?? -1 }, set: { setGrade(index, remembered: $0 == 1) })) {
                if status == nil { Text("Choose").tag(-1) }
                Text("Forgot").tag(0)
                Text("Remembered").tag(1)
            }.pickerStyle(.segmented)
        }
    }
    private func tap(_ index: Int) {
        guard !grading, result == nil, sentence.target_language_literals.indices.contains(index), !draft.tapped.contains(UInt64(index)) else { return }
        guard case .Heteronym = sentence.target_language_literals[index].word.word_type else { return }
        draft.tapped.append(UInt64(index))
    }
    private func setGrade(_ index: Int, remembered: Bool) {
        guard let grade = manual, feedback.grade_items.indices.contains(index) else { return }
        setResult(.Manual(grade: apply_translation_grade(grade: grade, item: feedback.grade_items[index], remembered: remembered,
                                                       literal_count: UInt64(sentence.target_language_literals.count))))
    }
    private func setResult(_ value: TranslationReviewResult) {
        result = value; draft.result = try? PendingReview.encode(value); storage.save(draft)
    }
    private func submit() {
        guard !grading, result == nil, !draft.text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty, let course = model.course else { return }
        grading = true; focused = false
        if draft.completedAtMs == nil { draft.completedAtMs = ReviewModel.now }
        draft.correct = find_closest_translation(user_translation: draft.text, candidates: sentence.native_translations, language: course.native_language) ?? sentence.native_translations.first ?? ""
        storage.save(draft)
        gradingTask = Task { @MainActor in
            let value: TranslationReviewResult
            if model.session.online {
                let response = await autograde_translation(challenge_sentence: sentence.target_language, user_sentence: draft.text,
                    native_translations: sentence.native_translations, literals: sentence.target_language_literals,
                    phrases: sentence.unique_target_language_phrases, access_token: model.session.accessToken(), course: course,
                    gram_definitions: GramDefinitions(value: sentence.gram_definitions_for_lookup), literal_gram_indices: sentence.literal_gram_indices,
                    phrase_definitions: GramDefinitions(value: sentence.phrase_definitions), primary_expression: sentence.primary_expression,
                    movie_titles: MovieTitles(value: sentence.movie_titles))
                if let error = response.autograding_error {
                    value = .Manual(grade: failed_translation_review(literal_count: UInt64(sentence.target_language_literals.count), error: error))
                } else { value = prepare_translation_review(literals: sentence.target_language_literals, response: response) }
            } else {
                value = .Manual(grade: failed_translation_review(literal_count: UInt64(sentence.target_language_literals.count), error: "You're offline"))
            }
            guard !Task.isCancelled else { return }
            setResult(value); grading = false
            audio.playEffect("ai-done-grading")
            if perfect { audio.playEffect("success-1") }
            #if DEBUG
            DebugHarness.log("translation graded: perfect=\(perfect) completedAtMs=\(draft.completedAtMs ?? 0)")
            #endif
        }
    }
    private func complete() {
        guard let result, feedback.can_continue, let timestamp = draft.completedAtMs else { return }
        let completed: Bool
        switch result {
        case .Perfect: completed = model.completeTranslationPerfect(sentence.target_language, tapped: feedback.heteronyms_tapped, completedAtMs: timestamp)
        case let .Manual(grade): completed = model.completeTranslationWrong(sentence.target_language, submission: draft.text, grade: grade, tapped: feedback.heteronyms_tapped, completedAtMs: timestamp)
        }
        if completed { storage.clear(); audio.stop() }
    }
    #if DEBUG
    private func debugCommand() {
        let command = DebugHarness.shared.command
        if command.hasPrefix("type "), result == nil, !grading { draft.text = String(command.dropFirst(5)) }
        if command == "type-reference", result == nil, !grading { draft.text = sentence.native_translations.first ?? "" }
        if command == "submit" { submit() }
        if command == "continue" { complete() }
        if command.hasPrefix("tap-word "), let index = Int(command.dropFirst(9)) { tap(index) }
        if command.hasPrefix("toggle-phrase "), let index = Int(command.dropFirst(14)), feedback.grade_items.indices.contains(index) {
            let status: Bool?
            switch feedback.grade_items[index] { case let .Literal(_, _, value), let .Phrase(_, _, value): status = value }
            setGrade(index, remembered: status != true)
        }
    }
    #endif
}
