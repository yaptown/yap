import SwiftUI
import Charts

/// Like the web, abandoned wizard answers are ephemeral; only referral is saved
/// immediately. The final selections are one immutable deck-selection event.
struct OnboardingFlowView: View {
    let session: YapSession
    let course: Course
    let hasHeardAbout: Bool
    let onComplete: () -> Void
    @State private var step = 0
    @State private var heard: HeardAbout?
    @State private var motivation: Motivation?
    @State private var experience: ExperienceLevel?
    @State private var goal: DailyReviewTarget?
    @State private var reviews = 0
    private var screens: [String] {
        (hasHeardAbout ? [] : ["heard-about"]) + ["motivation", "experience", "achievements", "srs-teaser", "srs-intro", "srs-conclusion", "study-goal", "ready"]
    }
    private var screen: String { screens[step] }
    private var language: String { get_language_metadata(language: course.target_language).common_name }
    private let heardOptions: [(HeardAbout, String)] = [(.FriendsOrFamily, "Friends or family"), (.Reddit, "Reddit"), (.TikTok, "TikTok"), (.GoogleSearch, "Google Search"), (.YouTube, "YouTube"), (.Other, "Other")]
    private let motivations: [(Motivation, String)] = [(.SpendTimeProductively, "Spend time productively"), (.SupportMyEducation, "Support my education"), (.ConnectWithPeople, "Connect with people"), (.BoostMyCareer, "Boost my career"), (.PrepareForTravel, "Prepare for travel"), (.JustForFun, "Just for fun"), (.Other, "Other")]
    private var experiences: [(ExperienceLevel, String)] { [(.New, "I'm new to \(language)"), (.CommonWords, "I know some common words"), (.BasicConversations, "I can have basic conversations"), (.VariousTopics, "I can talk about various topics"), (.MostTopics, "I can discuss most topics in detail")] }
    private var canContinue: Bool {
        switch screen { case "heard-about": heard != nil; case "motivation": motivation != nil; case "experience": experience != nil; case "study-goal": goal != nil; default: true }
    }
    var body: some View {
        ScrollViewReader { proxy in
            ScrollView {
                VStack(alignment: .leading, spacing: 24) {
                    ProgressView(value: Double(step + 1), total: Double(screens.count)).accessibilityLabel("Step \(step + 1) of \(screens.count)").id("top")
                    Button("Back", systemImage: "chevron.left", action: back)
                    StudyCard { content }
                    Button(buttonTitle, action: next)
                        .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
                        .frame(maxWidth: .infinity).disabled(!canContinue)
                    if screen == "ready", experience != .New {
                        Button("Start from scratch") { finish(startingFresh: true) }.controlSize(.large)
                    }
                }.padding(20).frame(maxWidth: 600)
            }.onChange(of: step) { _, _ in proxy.scrollTo("top", anchor: .top) }
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle(get_language_metadata(language: course.target_language).yaptown_name)
        .navigationBarTitleDisplayMode(.inline)
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            let command = DebugHarness.shared.command
            if command == "next" { next() }
            if command == "back" { back() }
            if command == "start-fresh", screen == "ready" { finish(startingFresh: true) }
            if command.hasPrefix("choose "), let index = Int(command.dropFirst(7)) { choose(index) }
            DebugHarness.log("onboarding=\(screen) step=\(step + 1)/\(screens.count)")
        }
        #endif
    }
    @ViewBuilder private var content: some View {
        switch screen {
        case "heard-about":
            title("How did you hear about Yap?")
            options(heardOptions, selection: $heard)
        case "motivation":
            title("Why are you learning \(language)?")
            options(motivations, selection: $motivation)
        case "experience":
            title("How much \(language) do you know?")
            options(experiences, selection: $experience)
        case "achievements":
            title("Here's what you can achieve")
            Label("Converse with confidence", systemImage: "bubble.left.and.bubble.right")
            Label("Build a large vocabulary", systemImage: "books.vertical")
            Label("Develop a lasting learning habit", systemImage: "repeat")
        case "srs-teaser":
            title("Yap is based on one scientifically proven idea:")
            Text("Spaced repetition.").font(.title.bold())
            ForEach(Self.studies, id: \.url) { study in
                VStack(alignment: .leading, spacing: 6) {
                    Link(study.title, destination: URL(string: study.url)!).font(.subheadline.weight(.semibold))
                    Text(study.citation).font(.caption).foregroundStyle(.secondary)
                }
            }
        case "srs-intro":
            title("Every time you review a word, you'll remember it for longer.")
            if reviews > 3 {
                Label("Word learned", systemImage: "checkmark.circle").font(.title2.bold())
                Text("That word is now in long-term memory!")
            } else {
                LearningIllustration(growth: false, reviews: reviews)
                Text("\(reviews) reviews").foregroundStyle(.secondary)
            }
        case "srs-conclusion":
            title("That's why if you study a little bit every day, you'll learn a lot.")
            LearningIllustration(growth: true, reviews: 3)
        case "study-goal":
            title("Set a daily study goal")
            options(get_daily_goal_options().map { ($0.value, "\($0.minutes) min/day — \(String(describing: $0.value))") }, selection: $goal)
            if let selected = get_daily_goal_options().first(where: { $0.value == goal }) {
                Text("That's ~\(selected.estimated_first_week_words) words in your first week!")
            }
        default:
            title(experience == .New ? "Let's start from the beginning!" : "Now let's find the best place to start")
            Text(experience == .New ? "We'll build your \(language) foundation step by step." : "Since you already know some \(language), we can skip ahead to where you belong.")
        }
    }
    private func title(_ text: String) -> some View { Text(text).font(.title2.bold()).fixedSize(horizontal: false, vertical: true) }
    private func options<T: Hashable>(_ values: [(T, String)], selection: Binding<T?>) -> some View {
        VStack(spacing: 12) {
            ForEach(values.indices, id: \.self) { index in
                Button { selection.wrappedValue = values[index].0 } label: {
                    HStack(spacing: 12) {
                        Text(values[index].1).multilineTextAlignment(.leading)
                        Spacer()
                        Image(systemName: selection.wrappedValue == values[index].0 ? "checkmark.circle.fill" : "circle")
                    }.padding(12).frame(maxWidth: .infinity, minHeight: 52)
                        .background(Color.yapAccent.opacity(selection.wrappedValue == values[index].0 ? 0.15 : 0.04), in: RoundedRectangle(cornerRadius: 12))
                }.buttonStyle(.plain).accessibilityAddTraits(selection.wrappedValue == values[index].0 ? .isSelected : [])
            }
        }
    }
    private var buttonTitle: String {
        if screen == "srs-intro", reviews <= 3 { return "Review" }
        if screen == "srs-conclusion" { return "Set a goal" }
        if screen == "ready" { return experience == .New ? get_language_metadata(language: course.target_language).lets_go : "Find my level" }
        return "Continue"
    }
    private func back() {
        if step == 0 { session.onboardingCourse = nil } else { step -= 1 }
    }
    private func next() {
        guard canContinue else { return }
        if screen == "srs-intro", reviews <= 3 { reviews += 1; return }
        if screen == "ready" { finish(startingFresh: experience == .New); return }
        if screen == "heard-about", let heard { session.addDeckSelectionEvent(.SetHeardAbout(heard_about: heard)) }
        step = min(step + 1, screens.count - 1)
    }
    private func finish(startingFresh: Bool) {
        session.addDeckSelectionEvent(.SetOnboardingSelections(selections: OnboardingSelections(starting_fresh: startingFresh, motivation: motivation, experience_level: experience, study_goal: goal), target_language: course.target_language))
        // SelectBothLanguages also records the course as onboarded. Like the
        // web, emit it only after the wizard, never when prefetching starts.
        session.addDeckSelectionEvent(.SelectBothLanguages(native: course.native_language, target: course.target_language))
        onComplete()
    }
    private func choose(_ index: Int) {
        guard index >= 0 else { return }
        switch screen {
        case "heard-about": if heardOptions.indices.contains(index) { heard = heardOptions[index].0 }
        case "motivation": if motivations.indices.contains(index) { motivation = motivations[index].0 }
        case "experience": if experiences.indices.contains(index) { experience = experiences[index].0 }
        case "study-goal": let goals = get_daily_goal_options(); if goals.indices.contains(index) { goal = goals[index].value }
        default: break
        }
    }
    private struct Study { let title: String; let citation: String; let url: String }
    private static let studies = [
        Study(title: "Memory: A Contribution to Experimental Psychology", citation: "Ebbinghaus, H. (1885). Teachers College, Columbia University.", url: "https://psychclassics.yorku.ca/Ebbinghaus/index.htm"),
        Study(title: "Distributed practice in verbal recall tasks: A review and quantitative synthesis", citation: "Cepeda, N.J., Pashler, H., Vul, E., Wixted, J.T., & Rohrer, D. (2006). Psychological Bulletin, 132(3), 354–380.", url: "https://doi.org/10.1037/0033-2909.132.3.354"),
        Study(title: "The Critical Importance of Retrieval for Learning", citation: "Karpicke, J.D. & Roediger, H.L. (2008). Science, 319(5865), 966–968.", url: "https://doi.org/10.1126/science.1152408"),
        Study(title: "A Stochastic Shortest Path Algorithm for Optimizing Spaced Repetition Scheduling", citation: "Ye, J.J., Su, J., & Cao, Y. (2022). KDD ’22, 4381–4390.", url: "https://dl.acm.org/doi/10.1145/3534678.3539081?cid=99660547150")
    ]
}

/// Conceptual illustrations, not forecasts or measured learner data. One series
/// uses one color; the text alternative explains it without relying on the plot.
private struct LearningIllustration: View {
    @Environment(\.colorScheme) private var scheme
    let growth: Bool
    let reviews: Int
    private var ink: Color { scheme == .dark ? Color(red: 192 / 255, green: 112 / 255, blue: 186 / 255) : .yapAccent }
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Chart {
                if growth {
                    ForEach(0..<50, id: \.self) { i in
                        let x = Double(i) / 49
                        LineMark(x: .value("Days", x), y: .value("Words learned", x * x)).foregroundStyle(ink).lineStyle(StrokeStyle(lineWidth: 2))
                    }
                } else {
                    ForEach(0..<min(reviews + 1, 3), id: \.self) { segment in
                        ForEach(0..<30, id: \.self) { i in
                            let widths = [0.22, 0.30, 0.48]
                            let ends = [0.28, 0.38, 0.55]
                            let t = Double(i) / 29
                            let x = widths.prefix(segment).reduce(0, +) + t * widths[segment]
                            LineMark(x: .value("Time", x), y: .value("Memory", pow(ends[segment], t)), series: .value("Review", segment)).foregroundStyle(ink).lineStyle(StrokeStyle(lineWidth: 2))
                        }
                    }
                }
            }.chartXScale(domain: 0...1).chartYScale(domain: 0...1)
                .chartXAxis(.hidden).chartYAxis(.hidden).frame(height: 180)
                .accessibilityHidden(true)
            Text(growth ? "Days → Words learned" : "Time → Memory between reviews").font(.caption)
            Text(growth ? "Illustration: consistent daily practice builds vocabulary over time." : "Illustration: memory fades more slowly after each review.").font(.caption).foregroundStyle(.secondary)
        }
    }
}
