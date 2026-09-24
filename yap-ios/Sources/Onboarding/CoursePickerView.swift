import SwiftUI

struct CoursePickerView: View {
    @Environment(AuthStore.self) private var auth
    @Environment(AuthSheet.self) private var authSheet
    let session: YapSession
    var onSelected: () -> Void = {}
    @State private var native: Language = .English
    private var courses: [Course] { get_available_courses() }
    private var natives: [Language] { Array(Set(courses.map(\.native_language))).sorted { String(describing: $0) < String(describing: $1) } }
    private var onboarded: [Language] {
        switch session.deckSelection {
        case let .noLanguageSelected(languages, _), let .languageSelected(_, _, languages, _): languages
        case .loading: []
        }
    }
    private var targets: [Course] {
        courses.filter { $0.native_language == native }.sorted {
            get_language_metadata(language: $0.target_language).english_name < get_language_metadata(language: $1.target_language).english_name
        }
    }
    var body: some View {
        Group {
            if let course = session.onboardingCourse {
                OnboardingFlowView(session: session, course: course, hasHeardAbout: session.onboardingHasHeardAbout) {
                    session.onboardingCourse = nil; onSelected()
                }.id(course)
            } else {
                List {
                    Section("I speak") {
                        Picker("Native language", selection: $native) {
                            ForEach(natives, id: \.self) { language in Text(get_language_metadata(language: language).native_name).tag(language) }
                        }
                    }
                    Section("Resume") {
                        ForEach(targets.filter { onboarded.contains($0.target_language) }, id: \.self) { course in courseButton(course, resume: true) }
                    }
                    ForEach([CourseMaturity.Stable, .Beta, .Alpha], id: \.self) { maturity in
                        Section(maturity == .Stable ? "What language will you speak next?" : "\(String(describing: maturity)) Languages") {
                            ForEach(targets.filter { !onboarded.contains($0.target_language) && get_language_metadata(language: $0.target_language).status == maturity }, id: \.self) { course in courseButton(course, resume: false) }
                        }
                    }
                }.navigationTitle("Choose a course")
            }
        }
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                if auth.userId == nil && !session.choosingCourse {
                    Button(account_copy().sign_in_action) { authSheet.present(tab: .signIn) }
                }
            }
        }
        .onAppear {
            let code = Locale.current.language.languageCode?.identifier
            native = natives.first { get_language_metadata(language: $0).iso6391 == code } ?? natives.first ?? .English
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            let command = DebugHarness.shared.command
            guard session.onboardingCourse == nil else { return }
            if command == "status" { DebugHarness.log("courses: " + targets.enumerated().map { "\($0.offset)=\($0.element.target_language)\(onboarded.contains($0.element.target_language) ? " (Resume)" : "")" }.joined(separator: ", ")) }
            if command.hasPrefix("choose "), let i = Int(command.dropFirst(7)), targets.indices.contains(i) { select(targets[i]) }
        }
        #endif
    }
    private func courseButton(_ course: Course, resume: Bool) -> some View {
        Button { select(course) } label: {
            HStack(spacing: 12) {
                let metadata = get_language_metadata(language: course.target_language)
                if Theme.emojiFontAvailable { Text(metadata.flag) }
                Text((resume ? "Resume " : "") + metadata.english_name)
                Spacer(); Image(systemName: "chevron.right")
            }.frame(minHeight: 52)
        }
    }
    private func select(_ course: Course) {
        if !onboarded.contains(course.target_language) {
            switch session.deckSelection {
            case let .noLanguageSelected(_, heard), let .languageSelected(_, _, _, heard): session.onboardingHasHeardAbout = heard
            case .loading: session.onboardingHasHeardAbout = false
            }
            session.onboardingCourse = course
            session.prefetchPack(course)
        } else {
            session.addDeckSelectionEvent(.SelectBothLanguages(native: course.native_language, target: course.target_language))
            onSelected()
        }
    }
}
