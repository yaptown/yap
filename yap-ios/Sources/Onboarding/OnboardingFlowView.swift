import SwiftUI

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
                    content
                        .frame(maxWidth: .infinity)
                        .id(view.step)
                        .transition(.asymmetric(insertion: .offset(x: 40).combined(with: .opacity), removal: .opacity))
                }.padding(20).frame(maxWidth: 600).frame(maxWidth: .infinity)
            }.bottomBar {
                VStack(spacing: 12) {
                    if case let .Ready(_, _, startFreshLabel) = view.content, let startFreshLabel {
                        Button { send(.StartFromScratch) } label: { Text(startFreshLabel).frame(maxWidth: .infinity) }
                            .buttonStyle(.bordered).controlSize(.large)
                    }
                    if let primary = view.primary {
                        Button { send(.Next) } label: {
                            HStack {
                                Text(primary.label)
                                if primary.show_arrow { Image(systemName: "arrow.right") }
                            }.frame(maxWidth: .infinity)
                        }
                            .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large).disabled(!primary.enabled)
                    }
                }.padding(20).frame(maxWidth: 600).frame(maxWidth: .infinity)
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
               case let .Choices(options) = view.content, options.indices.contains(index) {
                send(.Choose(choice: options[index].choice))
            }
            DebugHarness.log("onboarding=\(view.step) step=\(view.step_number)/\(view.total_steps)")
        }
        #endif
    }

    @ViewBuilder private var content: some View {
        switch view.content {
        case let .Choices(options):
            ChoicesStep(title: view.title, options: options) { send(.Choose(choice: $0)) }
        case let .Achievements(items):
            VStack(spacing: 24) {
                title(view.title)
                VStack(spacing: 16) {
                    ForEach(Array(items.enumerated()), id: \.element.text) { index, item in
                        HStack(spacing: 16) {
                            Text(item.emoji).font(.system(size: 36))
                            Text(item.text).font(.title3.weight(.medium)).multilineTextAlignment(.leading)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }.padding(20).cardSurface()
                            .modifier(SlideIn(delay: 0.15 + Double(index) * 0.15))
                    }
                }
            }
        case let .Studies(studies, conclusion):
            VStack(spacing: 8) {
                title(view.title)
                PaperStack(studies: studies).padding(.top, 16)
                Text(conclusion).font(.largeTitle.bold()).italic().foregroundStyle(Color.yapAccent)
                    .fadeIn(duration: 0.6, delay: 0.6 + Double(studies.count) * 0.5)
            }
        case let .Review(eyebrow, emphasis, curves, caption, reviewLabel, learned, learnedTitle, learnedBody, chart):
            StudyCard(animated: true) {
                Text(eyebrow.uppercased()).font(.subheadline.weight(.semibold)).tracking(1).foregroundStyle(Color.yapAccent)
                (Text(view.title) + Text(emphasis).italic().foregroundColor(.yapAccent))
                    .font(.title2.bold()).fixedSize(horizontal: false, vertical: true)
                if learned {
                    LearnedBadge(title: learnedTitle, message: learnedBody).transition(.scale(scale: 0.9).combined(with: .opacity))
                } else {
                    VStack(alignment: .leading, spacing: 12) {
                        HStack(spacing: 8) {
                            ForEach(1..<curves.count, id: \.self) { index in
                                Circle().fill(curveColor(index)).frame(width: 12, height: 12).transition(.scale)
                            }
                            if let reviewLabel { Text(verbatim: "\(curves.count - 1) \(reviewLabel)").font(.subheadline).foregroundStyle(Color.yapMuted) }
                        }.frame(height: 20)
                        ForgettingCurveChart(curves: curves, copy: chart)
                        Text(caption).foregroundStyle(Color.yapMuted).multilineTextAlignment(.center)
                            .frame(maxWidth: .infinity, minHeight: 48).id(caption).transition(.opacity)
                    }.transition(.opacity)
                }
            }
        case let .Remember(words):
            VStack(spacing: 24) {
                title(view.title)
                WordsIntoMemory(words: words)
            }
        case let .Ready(icon, body, _):
            ReadyCelebration(icon: icon, title: view.title, message: body)
        case .Notifications:
            // iOS never offers this step until native notification support lands.
            EmptyView()
        }
    }

    private func title(_ text: String) -> some View {
        Text(text).font(.title.bold()).multilineTextAlignment(.center).fixedSize(horizontal: false, vertical: true)
            .frame(maxWidth: .infinity)
    }

    private func send(_ event: OnboardingEvent) {
        let transition = onboarding_reduce(state: state, event: event)
        withAnimation(.easeOut(duration: 0.3)) { state = transition.state }
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

/// The web's chart tokens, one per curve, so each review has its own color.
@MainActor private func curveColor(_ index: Int) -> Color {
    let palette = Tokens.palette
    return [palette.chart_1, palette.chart_2, palette.chart_3, palette.chart_5][index % 4].color
}

/// Choosing advances; the pick shows for a beat first so the tap registers.
private struct ChoicesStep: View {
    let title: String
    let options: [OnboardingOption]
    let choose: @MainActor (OnboardingChoice) -> Void
    @State private var picked: OnboardingChoice?
    var body: some View {
        VStack(spacing: 24) {
            Text(title).font(.title.bold()).multilineTextAlignment(.center).fixedSize(horizontal: false, vertical: true)
            VStack(spacing: 12) {
                ForEach(Array(options.enumerated()), id: \.offset) { index, option in
                    let selected = picked.map { $0 == option.choice } ?? option.selected
                    Button {
                        if picked == nil { picked = option.choice }
                    } label: {
                        HStack(spacing: 14) {
                            choiceIcon(option.choice).font(.title3).foregroundStyle(Color.yapMuted).frame(width: 28)
                            VStack(alignment: .leading, spacing: 2) {
                                Text(option.label).font(.body.weight(.medium)).foregroundStyle(Color.yapText)
                                if let detail = option.detail { Text(detail).font(.subheadline).foregroundStyle(Color.yapMuted) }
                            }.multilineTextAlignment(.leading)
                            Spacer(minLength: 0)
                            if selected { Image(systemName: "checkmark.circle.fill").foregroundStyle(Color.yapAccent).transition(.scale) }
                        }
                        .padding(.horizontal, 16).padding(.vertical, 12).frame(maxWidth: .infinity, minHeight: 60)
                        .background(Color.yapAccent.opacity(selected ? 0.15 : 0.04), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
                        .overlay {
                            RoundedRectangle(cornerRadius: 14, style: .continuous)
                                .strokeBorder(selected ? Color.yapAccent : Color.yapBorder, lineWidth: selected ? 2 : 1)
                        }
                        .contentShape(Rectangle())
                        .animation(.easeOut(duration: 0.15), value: selected)
                    }
                    .buttonStyle(PressableButtonStyle())
                    .accessibilityAddTraits(selected ? .isSelected : [])
                    .fadeIn(duration: 0.25, delay: 0.05 * Double(index))
                }
            }
        }
        // Leaving the step (Back) cancels a pick that hasn't landed yet.
        .task(id: picked) {
            guard let picked, (try? await Task.sleep(for: .milliseconds(220))) != nil else { return }
            choose(picked)
        }
    }

    /// Icons are platform presentation; the choices, order and labels come from Rust.
    private func choiceIcon(_ choice: OnboardingChoice) -> Image {
        switch choice {
        case let .HeardAbout(value):
            let symbols: [HeardAbout: String] = [
                .FriendsOrFamily: "person.2", .Reddit: "globe", .TikTok: "video", .GoogleSearch: "magnifyingglass",
                .YouTube: "play.rectangle", .TwitterX: "globe", .Other: "questionmark.circle",
            ]
            return Image(systemName: symbols[value]!)
        case let .Motivation(value):
            let symbols: [Motivation: String] = [
                .SpendTimeProductively: "clock", .SupportMyEducation: "graduationcap", .ConnectWithPeople: "heart",
                .BoostMyCareer: "briefcase", .PrepareForTravel: "airplane", .JustForFun: "sparkles", .Other: "questionmark.circle",
            ]
            return Image(systemName: symbols[value]!)
        case let .Experience(value):
            let levels: [ExperienceLevel] = [.New, .CommonWords, .BasicConversations, .VariousTopics, .MostTopics]
            return Image(systemName: "cellularbars", variableValue: Double(levels.firstIndex(of: value)!) / 4)
        case let .StudyGoal(value):
            let goals: [DailyReviewTarget] = [.Casual, .Regular, .Serious, .Intense]
            return Image(systemName: "cellularbars", variableValue: Double(goals.firstIndex(of: value)! + 1) / 4)
        }
    }
}

private struct PressableButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label.scaleEffect(configuration.isPressed ? 0.98 : 1)
            .animation(.easeOut(duration: 0.1), value: configuration.isPressed)
    }
}

/// Slides in from the right, like the web's staggered achievement cards.
private struct SlideIn: ViewModifier {
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    let delay: Double
    @State private var shown = false
    func body(content: Content) -> some View {
        content.opacity(shown ? 1 : 0).offset(x: shown || reduceMotion ? 0 : 24)
            .onAppear { withAnimation(.easeOut(duration: 0.35).delay(delay)) { shown = true } }
    }
}

/// The studies arrive one by one as a fanned stack of papers.
private struct PaperStack: View {
    let studies: [OnboardingStudy]
    var body: some View {
        ZStack(alignment: .top) {
            ForEach(Array(studies.enumerated()), id: \.element.url) { index, study in
                Paper(study: study, index: index)
                    .rotationEffect(.degrees((Double(index) - 1.5) * 2))
                    .offset(y: CGFloat(index) * 8)
                    .modifier(PaperDrop(delay: 0.3 + Double(index) * 0.5))
            }
        }.frame(maxWidth: .infinity).frame(height: 210, alignment: .top)
    }
}

private struct Paper: View {
    let study: OnboardingStudy
    let index: Int
    var body: some View {
        Link(destination: URL(string: study.url)!) {
            VStack(alignment: .leading, spacing: 4) {
                Text(study.title).font(.system(size: 11, weight: .semibold)).lineLimit(1)
                Text(verbatim: "\(study.authors) (\(study.year)). \(study.journal)").font(.system(size: 9)).foregroundStyle(.gray).lineLimit(1)
                VStack(alignment: .leading, spacing: 6) {
                    ForEach(0..<6, id: \.self) { line in
                        let fraction = line == 5 ? 0.4 + Double(index * 10 % 30) / 100 : 0.75 + Double((index + line) * 7 % 25) / 100
                        Capsule().fill(Color(white: 0.88)).frame(height: 6)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .scaleEffect(x: fraction, anchor: .leading)
                    }
                }.padding(.top, 8)
                Spacer(minLength: 0)
            }
            .foregroundStyle(.black).padding(.horizontal, 18).padding(.vertical, 14)
            .frame(maxWidth: .infinity, alignment: .leading).frame(height: 170)
            .background(.white, in: RoundedRectangle(cornerRadius: 4))
            .shadow(color: .black.opacity(0.18), radius: 6, y: 3)
        }.buttonStyle(.plain).padding(.horizontal, 12)
    }
}

private struct PaperDrop: ViewModifier {
    let delay: Double
    @State private var shown = false
    func body(content: Content) -> some View {
        content.opacity(shown ? 1 : 0).offset(y: shown ? 0 : 60)
            .onAppear { withAnimation(.easeOut(duration: 0.5).delay(delay)) { shown = true } }
    }
}

/// Conceptual illustration, not a forecast: each review restarts the curve,
/// and each restart decays more slowly. Geometry comes from Rust.
private struct ForgettingCurveChart: View {
    let curves: [OnboardingCurve]
    let copy: OnboardingChart
    var body: some View {
        HStack(alignment: .top, spacing: 6) {
            Text(verbatim: "\(copy.y_label) →").font(.caption.weight(.medium)).foregroundStyle(Color.yapMuted)
                .fixedSize().rotationEffect(.degrees(-90)).frame(width: 16, height: 70, alignment: .center)
            VStack(alignment: .trailing, spacing: 4) {
                GeometryReader { geometry in
                    let size = geometry.size
                    ZStack(alignment: .topLeading) {
                        ForEach([0.25, 0.5, 0.75], id: \.self) { memory in
                            Path { path in
                                path.move(to: CGPoint(x: 0, y: (1 - memory) * size.height))
                                path.addLine(to: CGPoint(x: size.width, y: (1 - memory) * size.height))
                            }.stroke(Color.yapText.opacity(0.1), style: StrokeStyle(lineWidth: 1, dash: [4, 4]))
                        }
                        ForEach(Array(curves.enumerated()), id: \.offset) { index, curve in
                            CurveSegment(curve: curve, index: index, size: size)
                        }
                        Path { path in
                            path.move(to: CGPoint(x: 0, y: -6))
                            path.addLine(to: CGPoint(x: 0, y: size.height))
                            path.addLine(to: CGPoint(x: size.width, y: size.height))
                        }.stroke(Color.yapText.opacity(0.35), lineWidth: 1.5)
                    }
                }.frame(height: 160)
                Text(verbatim: "\(copy.x_label) →").font(.caption.weight(.medium)).foregroundStyle(Color.yapMuted)
            }
        }
        .padding(.top, 8)
        .accessibilityElement(children: .ignore).accessibilityLabel(copy.accessibility_label)
    }
}

private struct CurveSegment: View {
    let curve: OnboardingCurve
    let index: Int
    let size: CGSize
    @State private var drawn = false
    private let inset: CGFloat = 10
    private func point(_ t: Double) -> CGPoint {
        let time = curve.start + t * (curve.end - curve.start)
        return CGPoint(x: inset + time * (size.width - inset), y: (1 - pow(curve.retained, t)) * size.height)
    }
    private var line: Path {
        Path { path in
            path.move(to: point(0))
            for step in 1...60 { path.addLine(to: point(Double(step) / 60)) }
        }
    }
    var body: some View {
        let color = curveColor(index)
        let start = point(0)
        ZStack(alignment: .topLeading) {
            if index > 0 {
                Path { path in
                    path.move(to: CGPoint(x: start.x, y: size.height))
                    path.addLine(to: start)
                }.stroke(color, style: StrokeStyle(lineWidth: 1.5, dash: [4, 3]))
            }
            Path { path in
                path.addPath(line)
                path.addLine(to: CGPoint(x: point(1).x, y: size.height))
                path.addLine(to: CGPoint(x: start.x, y: size.height))
                path.closeSubpath()
            }.fill(color.opacity(0.15)).opacity(drawn ? 1 : 0)
            line.trim(from: 0, to: drawn ? 1 : 0).stroke(color, style: StrokeStyle(lineWidth: 2.5, lineCap: .round))
            Circle().fill(Color.yapBackground).overlay { Circle().strokeBorder(color, lineWidth: 2.5) }
                .frame(width: 12, height: 12).scaleEffect(drawn ? 1 : 0.01)
                .position(start)
        }
        .onAppear { withAnimation(.easeOut(duration: 0.8)) { drawn = true } }
    }
}

private struct LearnedBadge: View {
    let title: String
    let message: String
    @State private var shown = false
    var body: some View {
        VStack(spacing: 12) {
            Image(systemName: "checkmark").font(.system(size: 36, weight: .bold)).foregroundStyle(Color.yapAccent)
                .frame(width: 76, height: 76).background(Color.yapAccent.opacity(0.15), in: Circle())
                .scaleEffect(shown ? 1 : 0.01).rotationEffect(.degrees(shown ? 0 : -30))
            Text(title).font(.largeTitle.bold())
            Text(message).font(.title3).foregroundStyle(Color.yapMuted).multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity).padding(.vertical, 32)
        .onAppear { withAnimation(.spring(response: 0.45, dampingFraction: 0.55).delay(0.1)) { shown = true } }
    }
}

/// Words drift into memory and stay there.
private struct WordsIntoMemory: View {
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    let words: [String]
    private let cycle = 4.0
    @State private var start = Date()
    var body: some View {
        TimelineView(.animation(paused: reduceMotion)) { context in
            let elapsed = context.date.timeIntervalSince(start)
            ZStack {
                Circle().fill(Color.yapAccent.opacity(0.2)).frame(width: 144, height: 144).blur(radius: 24)
                    .scaleEffect(1 + 0.12 * sin(elapsed * 2 * .pi * Double(words.count) / cycle))
                Image(systemName: "brain").font(.system(size: 56, weight: .light)).foregroundStyle(Color.yapAccent)
                    .frame(width: 112, height: 112).background(Color.yapAccent.opacity(0.1), in: Circle())
                    .overlay { Circle().strokeBorder(Color.yapAccent.opacity(0.2)) }
                ForEach(Array(words.enumerated()), id: \.offset) { index, word in
                    let phase = (elapsed / cycle + Double(index) / Double(words.count)).truncatingRemainder(dividingBy: 1)
                    let angle = Double(index) / Double(words.count) * 2 * .pi + 0.4
                    // Ease in: words drift, then get pulled in.
                    let pull = phase < 0.35 ? phase / 0.35 * 0.2 : 0.2 + pow((phase - 0.35) / 0.65, 2) * 0.8
                    let opacity = phase < 0.15 ? phase / 0.15 : phase > 0.85 ? (1 - phase) / 0.15 : 1
                    Text(word).font(.body.weight(.medium)).foregroundStyle(Color.yapText)
                        .padding(.horizontal, 12).padding(.vertical, 5)
                        .background(.ultraThinMaterial, in: Capsule())
                        .overlay { Capsule().strokeBorder(Color.yapBorder) }
                        .fixedSize()
                        .scaleEffect(1 - 0.7 * max(0, (phase - 0.35) / 0.65))
                        .opacity(reduceMotion ? 1 : opacity)
                        .offset(x: cos(angle) * 130 * (1 - pull), y: sin(angle) * 80 * (1 - pull))
                }
            }.frame(maxWidth: .infinity).frame(height: 260)
        }.accessibilityHidden(true)
    }
}

/// The finish line: the course icon lands with a burst of confetti.
private struct ReadyCelebration: View {
    let icon: String
    let title: String
    let message: String
    @State private var landed = false
    @State private var burst = false
    @State private var glow = false
    var body: some View {
        VStack(spacing: 20) {
            ZStack {
                Circle().fill(Color.yapAccent.opacity(0.25)).frame(width: 120, height: 120).blur(radius: 28)
                    .scaleEffect(glow ? 1.2 : 1)
                ForEach(0..<14, id: \.self) { index in
                    let angle = Double(index) / 14 * 2 * .pi
                    let distance = 70 + Double(index % 3) * 14
                    Group {
                        if index.isMultiple(of: 2) { RoundedRectangle(cornerRadius: 1.5).frame(width: 6, height: 12) }
                        else { Circle().frame(width: 8, height: 8) }
                    }
                    .foregroundStyle(curveColor(index))
                    .rotationEffect(.degrees(burst ? Double(index) * 40 : 0))
                    .offset(x: burst ? cos(angle) * distance : 0, y: burst ? sin(angle) * distance : 0)
                    .scaleEffect(burst ? 0.7 : 0.01)
                    .opacity(burst ? 0 : 1)
                }
                Image(icon).renderingMode(.original).resizable().scaledToFit().frame(width: 96, height: 96)
                    .scaleEffect(landed ? 1 : 0.01).rotationEffect(.degrees(landed ? 0 : -20))
            }.frame(height: 170).accessibilityHidden(true)
            Text(title).font(.largeTitle.bold()).multilineTextAlignment(.center).fixedSize(horizontal: false, vertical: true)
                .fadeIn(duration: 0.4, delay: 0.35)
            Text(message).font(.title3).foregroundStyle(Color.yapMuted).multilineTextAlignment(.center)
                .fadeIn(duration: 0.4, delay: 0.55)
        }
        .frame(maxWidth: .infinity).padding(.top, 16)
        .onAppear {
            withAnimation(.spring(response: 0.5, dampingFraction: 0.5)) { landed = true }
            withAnimation(.easeOut(duration: 1.2).delay(0.25)) { burst = true }
            withAnimation(.easeInOut(duration: 1.2).repeatForever()) { glow = true }
        }
    }
}
