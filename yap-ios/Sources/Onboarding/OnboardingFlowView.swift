import SwiftUI
import Charts

/// Only Rust owns the wizard's answers, itinerary, and transitions. Hosts apply
/// effects using the existing immutable deck-selection events.
struct OnboardingFlowView: View {
    let session: YapSession
    let course: Course
    let onComplete: () -> Void
    @State private var state: OnboardingState

    init(session: YapSession, course: Course, hasHeardAbout: Bool, onComplete: @escaping () -> Void) {
        self.session = session
        self.course = course
        self.onComplete = onComplete
        _state = State(initialValue: onboarding_start(target_language: course.target_language, has_heard_about: hasHeardAbout, offer_notifications: false, purpose: .App))
    }

    private var view: OnboardingView { onboarding_view(state: state) }

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView {
                VStack(alignment: .leading, spacing: 24) {
                    ProgressView(value: view.progress_percent, total: 100).accessibilityLabel(view.progress_label).id("top")
                    Button(view.back_label, systemImage: "chevron.left") { send(.Back) }
                    StudyCard(animated: true) { content }
                    if case let .Ready(_, _, startFreshLabel) = view.content, let startFreshLabel {
                        Button(startFreshLabel) { send(.StartFromScratch) }.controlSize(.large)
                    }
                }.padding(20).frame(maxWidth: 600).frame(maxWidth: .infinity)
            }.bottomBar {
                if let primary = view.primary {
                    Button { send(.Next) } label: {
                        HStack {
                            Text(primary.label)
                            if primary.show_arrow { Image(systemName: "arrow.right") }
                        }.frame(maxWidth: .infinity)
                    }
                        .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large).disabled(!primary.enabled)
                        .padding(20).frame(maxWidth: 600).frame(maxWidth: .infinity)
                }
            }.onChange(of: state.step_index) { _, _ in proxy.scrollTo("top", anchor: .top) }
        }
        .background(.clear)
        .navigationTitle(view.navigation_title)
        .navigationBarTitleDisplayMode(.inline)
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            let command = DebugHarness.shared.command
            if command == "next" { send(.Next) }
            if command == "back" { send(.Back) }
            if command == "start-fresh" { send(.StartFromScratch) }
            if command.hasPrefix("choose "), let index = Int(command.dropFirst(7)),
               case let .Choices(options, _) = view.content, options.indices.contains(index) {
                send(.Choose(choice: options[index].choice))
            }
            DebugHarness.log("onboarding=\(view.step) step=\(view.step_number)/\(view.total_steps)")
        }
        #endif
    }

    @ViewBuilder private var content: some View {
        switch view.content {
        case let .Choices(options, hint):
            title(view.title)
            VStack(spacing: 12) {
                ForEach(options.indices, id: \.self) { index in
                    let option = options[index]
                    Button { send(.Choose(choice: option.choice)) } label: {
                        HStack(spacing: 12) {
                            Text(option.label).multilineTextAlignment(.leading)
                            Spacer()
                            Image(systemName: option.selected ? "checkmark.circle.fill" : "circle")
                        }.padding(12).frame(maxWidth: .infinity, minHeight: 52)
                            .background(Color.yapAccent.opacity(option.selected ? 0.15 : 0.04), in: RoundedRectangle(cornerRadius: 12))
                    }.buttonStyle(.plain).accessibilityAddTraits(option.selected ? .isSelected : [])
                }
            }
            if let hint { Text(hint) }
        case let .Achievements(items):
            title(view.title)
            ForEach(items, id: \.text) { item in
                HStack(spacing: 12) { Text(item.emoji); Text(item.text) }
            }
        case let .Studies(studies, conclusion):
            title(view.title)
            Text(conclusion).font(.title.bold())
            ForEach(studies, id: \.url) { study in
                VStack(alignment: .leading, spacing: 6) {
                    Link(study.title, destination: URL(string: study.url)!).font(.subheadline.weight(.semibold))
                    Text(verbatim: "\(study.authors) (\(study.year)). \(study.journal)").font(.caption).foregroundStyle(.secondary)
                }
            }
        case let .Review(eyebrow, emphasis, demoReviews, reviewLabel, learned, learnedTitle, learnedBody, chart):
            Text(eyebrow).font(.caption.weight(.semibold)).foregroundStyle(Color.yapAccent)
            (Text(view.title) + Text(emphasis).italic().foregroundColor(.yapAccent))
                .font(.title2.bold()).fixedSize(horizontal: false, vertical: true)
            HStack(spacing: 8) {
                ForEach(0..<min(Int(demoReviews), 3), id: \.self) { _ in Circle().fill(Color.yapAccent).frame(width: 12, height: 12) }
                if let reviewLabel { Text(reviewLabel).font(.subheadline).foregroundStyle(.secondary) }
            }.frame(height: 20)
            if learned {
                Label(learnedTitle, systemImage: "checkmark.circle").font(.title2.bold())
                Text(learnedBody)
            } else {
                LearningIllustration(growth: false, visibleCurves: min(Int(demoReviews), 3), copy: chart)
            }
        case let .Growth(chart):
            title(view.title)
            LearningIllustration(growth: true, visibleCurves: 0, copy: chart)
        case let .Ready(flag, body, _):
            Text(flag).font(.system(size: 64))
            title(view.title)
            Text(body)
        case .Notifications:
            // iOS never offers this step until native notification support lands.
            EmptyView()
        }
    }

    private func title(_ text: String) -> some View { Text(text).font(.title2.bold()).fixedSize(horizontal: false, vertical: true) }

    private func send(_ event: OnboardingEvent) {
        let transition = onboarding_reduce(state: state, event: event)
        state = transition.state
        for effect in transition.effects {
            switch effect {
            case let .SaveHeardAbout(value):
                session.addDeckSelectionEvent(.SetHeardAbout(heard_about: value))
            case let .Complete(selections):
                session.addDeckSelectionEvent(.SetOnboardingSelections(selections: selections, target_language: course.target_language))
                // SelectBothLanguages records the course as onboarded, only at completion.
                session.addDeckSelectionEvent(.SelectBothLanguages(native: course.native_language, target: course.target_language))
                onComplete()
            case .Exit:
                session.onboardingCourse = nil
            }
        }
    }
}

/// Conceptual illustrations, not forecasts or measured learner data.
private struct LearningIllustration: View {
    @Environment(\.colorScheme) private var scheme
    let growth: Bool
    let visibleCurves: Int
    let copy: OnboardingChart
    private var ink: Color { scheme == .dark ? Color(red: 192 / 255, green: 112 / 255, blue: 186 / 255) : .yapAccent }
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Chart {
                if growth {
                    ForEach(0..<50, id: \.self) { i in
                        let x = Double(i) / 49
                        LineMark(x: .value(copy.x_label, x), y: .value(copy.y_label, x * x)).foregroundStyle(ink).lineStyle(StrokeStyle(lineWidth: 2))
                    }
                } else {
                    ForEach(0..<visibleCurves, id: \.self) { segment in
                        ForEach(0..<30, id: \.self) { i in
                            let widths = [0.22, 0.30, 0.48]
                            let ends = [0.28, 0.38, 0.55]
                            let t = Double(i) / 29
                            let x = widths.prefix(segment).reduce(0, +) + t * widths[segment]
                            LineMark(x: .value(copy.x_label, x), y: .value(copy.y_label, pow(ends[segment], t)), series: .value("", segment)).foregroundStyle(ink).lineStyle(StrokeStyle(lineWidth: 2))
                        }
                    }
                }
            }.chartXScale(domain: 0...1).chartYScale(domain: 0...1)
                .chartXAxis(.hidden).chartYAxis(.hidden).frame(height: 180)
                .accessibilityElement(children: .ignore).accessibilityLabel(copy.accessibility_label)
            HStack {
                if !copy.y_label.isEmpty { Text(copy.y_label) }
                Spacer()
                Text(copy.x_label)
            }.font(.caption).foregroundStyle(.secondary)
        }
    }
}
