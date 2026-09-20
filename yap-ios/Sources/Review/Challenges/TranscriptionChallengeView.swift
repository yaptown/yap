import SwiftUI

struct TranscriptionChallengeView: View {
    @Environment(AudioPlayer.self) private var audio
    let model: ReviewModel
    let sentence: TranscribeComprehensibleSentence
    @State private var draft = Draft()
    @State private var result: Grade?
    @State private var grading = false
    @State private var hasClip: Bool?
    @State private var gradingTask: Task<Void, Never>?
    @State private var translationRevealed = false
    @FocusState private var focused: Int?
    private struct Draft: Codable, Equatable {
        var inputs: [Int: String] = [:]
        var completedAtMs: Double?
        var result: Data?
    }
    private var storage: PendingReview { PendingReview(kind: "transcription", challenge: sentence, model: model) }
    private var blanks: [Int] { sentence.parts.indices.filter { if case .AskedToTranscribe = sentence.parts[$0] { true } else { false } } }
    private var submission: TranscriptionSubmission {
        prepare_transcription_submission(parts: sentence.parts, inputs: draft.inputs.map { TranscriptionInput(index: UInt64($0.key), text: $0.value) })
    }
    private var perfect: Bool { result.map { transcription_is_perfect(results: $0.results) } ?? false }
    private let gradeOptions: [(String, WordGrade)] = [
        ("Perfect", .Perfect(wrote: nil)), ("Correct with typo", .CorrectWithTypo(wrote: nil)),
        ("Phonetically identical", .PhoneticallyIdenticalButContextuallyIncorrect(wrote: nil)),
        ("Phonetically similar", .PhoneticallySimilarButContextuallyIncorrect(wrote: nil)),
        ("Incorrect", .Incorrect(wrote: nil)), ("Missed", .Missed)
    ]
    var body: some View {
        VStack(spacing: 12) {
            StudyCard {
                if sentence.second_chance { ReviewBadge(text: "Second chance") }
                HStack(alignment: .center, spacing: 8) {
                    AudioButton(request: sentence.audio, session: model.session, reviewCount: model.deck.get_total_reviews(), autoplay: true)
                    SentenceFlow(spacing: 0, alignment: .center) {
                        ForEach(Array(sentence.parts.enumerated()), id: \.offset) { index, part in
                            switch part {
                            case let .Provided(literal): Text(literal.word.text + literal.whitespace).font(.body)
                            case let .AskedToTranscribe(parts):
                                TextField("What did you hear?", text: Binding(get: { draft.inputs[index] ?? "" }, set: { draft.inputs[index] = $0 }))
                                    .textFieldStyle(.roundedBorder).font(.body).frame(width: 150, height: 44).focused($focused, equals: index)
                                    .autocorrectionDisabled().textInputAutocapitalization(index == 0 ? .sentences : .never)
                                    .submitLabel(index == blanks.last ? .done : .next).onSubmit { advance(index) }
                                    .disabled(grading || result != nil)
                                if let whitespace = parts.last?.whitespace, !whitespace.isEmpty { Text(whitespace) }
                            }
                        }
                        }.frame(maxWidth: .infinity)
                }
                VideoClipView(deck: model.deck, language: model.deck.get_target_language(), text: sentence.target_language,
                    session: model.session, reviewCount: model.deck.get_total_reviews(),
                    maskedSentence: result == nil && !grading ? sentence.parts.map { part in
                        switch part { case let .Provided(literal): literal.word.text + literal.whitespace
                        case let .AskedToTranscribe(parts): parts.map { "____" + $0.whitespace }.joined() }
                    }.joined() : nil, available: $hasClip)
                if let result {
                    SentenceVerdictView(submission: draft.inputs.sorted { $0.key < $1.key }.map(\.value).joined(separator: " "),
                        correct: sentence.target_language, perfect: perfect, encouragement: result.encouragement,
                        explanation: result.explanation, error: result.autograding_error, correctLabel: "Correct sentence:", submissionLabel: "Your answer:")
                    wordGrades(result)
                    if !result.compare.isEmpty {
                        Text(result.compare.joined(separator: " · "))
                        AudioButton(request: AudioRequest(request: TtsRequest(text: result.compare.map { $0 + ";" }.joined(separator: " "),
                            language: model.deck.get_target_language(), is_ssml: false, instructions: nil, speed: 0.8, verification_hints: []), provider: .Google),
                            session: model.session, reviewCount: model.deck.get_total_reviews())
                    }
                    DisclosureGroup("Translation", isExpanded: $translationRevealed) { Text(sentence.native_language) }
                    ReviewDefinitionsView(definitions: get_transcription_review_definitions(challenge: sentence, results: result.results))
                } else if grading {
                    Text(sentence.target_language).foregroundStyle(.green)
                    ProgressView("Grading your answer…")
                }
            }
            if result == nil {
                Button { submit() } label: { Text("Check answer").frame(maxWidth: .infinity) }.disabled(grading || !submission.all_blanks_filled)
                    .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
                Button("I can't listen right now") { model.cantListen() }.font(.footnote).foregroundStyle(.secondary).frame(minHeight: 44).disabled(grading)
            } else {
                Button { complete() } label: { Text(perfect ? "Nailed it!" : "Continue").frame(maxWidth: .infinity) }.disabled(model.submitting)
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
            if let saved = storage.load(Draft.self) {
                draft = saved
                if let data = saved.result { result = try? PendingReview.decode(data, as: Grade.self) }
            }
            if result == nil { focused = blanks.first }
            if draft.completedAtMs != nil && result == nil { submit() }
        }
        .onDisappear { gradingTask?.cancel(); grading = false }
        .onChange(of: draft) { _, _ in storage.save(draft) }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in guard DebugHarness.shared.activeTab == .learn else { return }; debugCommand() }
        #endif
    }
    @ViewBuilder private func wordGrades(_ grade: Grade) -> some View {
        ForEach(Array(grade.results.enumerated()), id: \.offset) { partIndex, part in
            if case let .AskedToTranscribe(parts, _) = part {
                ForEach(Array(parts.enumerated()), id: \.offset) { wordIndex, word in
                    HStack(spacing: 4) {
                        Text(word.heard.word.text).fontWeight(.semibold)
                        Spacer()
                        Picker("Grade \(word.heard.word.text)", selection: Binding(get: { gradeIndex(word.grade) }, set: { index in
                            setGrade(partIndex, wordIndex, gradeOptions[index].1)
                        })) {
                            ForEach(gradeOptions.indices, id: \.self) { index in Text(gradeOptions[index].0).tag(index) }
                        }.pickerStyle(.menu).controlSize(.small)
                    }.font(.subheadline)
                }
            }
        }
    }
    private func gradeIndex(_ grade: WordGrade) -> Int {
        switch grade {
        case .Perfect: 0; case .CorrectWithTypo: 1; case .PhoneticallyIdenticalButContextuallyIncorrect: 2
        case .PhoneticallySimilarButContextuallyIncorrect: 3; case .Incorrect: 4; case .Missed: 5
        }
    }
    private func setGrade(_ part: Int, _ word: Int, _ grade: WordGrade) {
        guard var value = result else { return }
        value.results = apply_transcription_grade(results: value.results, part_index: UInt64(part), word_index: UInt64(word), grade: grade)
        setResult(value)
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
    private func setResult(_ value: Grade) {
        result = value; draft.result = try? PendingReview.encode(value); storage.save(draft)
    }
    private func submit() {
        guard !grading, result == nil, submission.all_blanks_filled, let course = model.course else { return }
        grading = true; focused = nil
        if draft.completedAtMs == nil { draft.completedAtMs = ReviewModel.now }
        storage.save(draft)
        let request = submission.request
        gradingTask = Task { @MainActor in
            let grade: Grade
            if model.session.online {
                grade = await autograde_transcription(submission: request, access_token: model.session.accessToken(), course: course, movie_titles: MovieTitles(value: sentence.movie_titles))
            } else { grade = failed_transcription_review(submission: request, course: course) }
            guard !Task.isCancelled else { return }
            setResult(grade); grading = false
            audio.playEffect("ai-done-grading")
            if perfect { audio.playEffect("success-1") }
            #if DEBUG
            DebugHarness.log("transcription graded: perfect=\(perfect) completedAtMs=\(draft.completedAtMs ?? 0)")
            #endif
        }
    }
    private func complete() {
        guard let result, let timestamp = draft.completedAtMs else { return }
        if model.completeTranscription(result.results, completedAtMs: timestamp) { storage.clear(); audio.stop() }
    }
    #if DEBUG
    private func debugCommand() {
        let command = DebugHarness.shared.command
        if command.hasPrefix("type "), result == nil, !grading, let index = focused ?? blanks.first { draft.inputs[index] = String(command.dropFirst(5)) }
        if command == "type-reference", result == nil, !grading {
            for index in blanks { if case let .AskedToTranscribe(parts) = sentence.parts[index] { draft.inputs[index] = gramText(parts) } }
        }
        if command == "submit" { submit() }
        if command == "continue" { complete() }
        if command.hasPrefix("focus "), let index = Int(command.dropFirst(6)), blanks.contains(index) { focused = index }
        if command.hasPrefix("accent ") { insertAccent(String(command.dropFirst(7))) }
        if command.hasPrefix("toggle-word ") {
            let indices = command.dropFirst(12).split(separator: " ").compactMap { Int($0) }
            if indices.count == 2 { setGrade(indices[0], indices[1], .Perfect(wrote: nil)) }
        }
    }
    #endif
}

extension UIResponder {
    @objc func insertReviewAccent(_ sender: Any?) {
        guard let input = self as? UIKeyInput, let text = sender as? String else { return }
        input.insertText(text)
    }
}
