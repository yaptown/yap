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
    private var current: Course? {
        if case let .languageSelected(course, _, _, _) = session.deckSelection, session.choosingCourse { course } else { nil }
    }
    private var targets: [Course] { courses.filter { $0.native_language == native } }
    var body: some View {
        Group {
            if let course = session.onboardingCourse {
                OnboardingFlowView(session: session, course: course, hasHeardAbout: session.onboardingHasHeardAbout) {
                    session.onboardingCourse = nil; onSelected()
                }.id(course)
            } else {
                ScrollView {
                    VStack(spacing: 32) {
                        Text("What language will you speak next?").font(.largeTitle.bold()).multilineTextAlignment(.center)
                        if let current { resumeCard(current); divider("Or choose a different language") }
                        ForEach([CourseMaturity.Stable, .Beta, .Alpha], id: \.self) { maturity in
                            let section = targets.filter { get_language_metadata(language: $0.target_language).status == maturity }
                            if !section.isEmpty {
                                if maturity != .Stable { divider("\(String(describing: maturity)) Languages") }
                                LazyVGrid(columns: [GridItem(.flexible(), spacing: 16, alignment: .top), GridItem(.flexible(), spacing: 16, alignment: .top)], spacing: 32) {
                                    ForEach(section, id: \.self, content: tile)
                                }
                            }
                        }
                        HStack {
                            Text("Native language:").foregroundStyle(Color.yapMuted)
                            Picker("Native language", selection: $native) {
                                ForEach(natives, id: \.self) { language in Text(get_language_metadata(language: language).native_name).tag(language) }
                            }.pickerStyle(.menu)
                        }
                        Text("(Yap.Town is great for beginner and intermediate students.)")
                            .font(.subheadline).foregroundStyle(Color.yapMuted).multilineTextAlignment(.center)
                    }.padding(20).frame(maxWidth: 600).frame(maxWidth: .infinity)
                }.navigationTitle("Choose a course").navigationBarTitleDisplayMode(.inline)
            }
        }
        .containerBackground(.clear, for: .navigation)
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
    private func tile(_ course: Course) -> some View {
        let name = get_language_name(language: course.target_language, reader: native)
        return Button { select(course) } label: {
            VStack(spacing: 2) {
                icon(course, size: 144)
                Text(name.name).font(.title2.bold()).foregroundStyle(Color.yapText)
                if let variant = name.variant { Text(variant).font(.subheadline).foregroundStyle(Color.yapMuted) }
            }.frame(maxWidth: .infinity).contentShape(Rectangle())
        }.buttonStyle(.plain)
    }
    private func resumeCard(_ course: Course) -> some View {
        let name = get_language_name(language: course.target_language, reader: native)
        return Button { select(course) } label: {
            HStack(spacing: 16) {
                icon(course, size: 56)
                VStack(alignment: .leading, spacing: 2) {
                    Text("Resume " + name.full).font(.title3.bold()).foregroundStyle(Color.yapText)
                    Text("Continue where you left off").font(.subheadline).foregroundStyle(Color.yapMuted)
                }
                Spacer(minLength: 0)
                Image(systemName: "arrow.right").foregroundStyle(Color.yapText)
            }.padding(20).frame(maxWidth: .infinity).cardSurface()
        }.buttonStyle(.plain)
    }
    private func icon(_ course: Course, size: CGFloat) -> some View {
        Image(get_language_metadata(language: course.target_language).icon).renderingMode(.original).resizable().scaledToFit()
            .frame(width: size, height: size).accessibilityHidden(true)
    }
    private func divider(_ label: String) -> some View {
        HStack(spacing: 12) {
            VStack { Divider() }
            Text(label.uppercased()).font(.caption.weight(.semibold)).tracking(1).foregroundStyle(Color.yapMuted).fixedSize()
            VStack { Divider() }
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
