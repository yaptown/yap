import SwiftUI

struct HomeScreen: View {
    @Environment(\.reviewActions!) private var actions
    let review: ReviewModel
    let navigate: (CourseRoute) -> Void
    var view: HomeScreenView? = nil
    var isVisible = true
    @State private var query = ""
    // Adding cards from Home means "study these now" — after the add fires, jump
    // to Review so the user lands on the cards they just committed to (mirrors
    // web). In fixture mode `navigate` is a no-op and hit-testing is off.
    private var addingGoesToReview: ReviewActions {
        var wrapped = actions
        let addEvent = actions.addEvent
        wrapped.addEvent = { event in
            addEvent(event)
            navigate(.review)
        }
        return wrapped
    }
    var body: some View {
        TimelineView(.periodic(from: .now, by: 10)) { _ in
            let deck = review.deck
            let view = self.view ?? deck.home_screen_view(inputs: HomeScreenInputs(
                banned: review.banned, sentence_list: review.session.curriculumDraft.map(\.selection) ?? deck.get_sentence_list(),
                online: review.session.online, is_signed_in: review.auth.session != nil,
                timestamp_ms: ReviewModel.now))
            GeometryReader { geometry in
                ScrollView {
                    VStack(alignment: .leading, spacing: 16) {
                        Button(action: actions.switchCourse) {
                            StudyCard {
                                HStack {
                                    Text(view.course_label).font(.headline).foregroundStyle(Color.yapText)
                                    Spacer()
                                    Image(systemName: "chevron.right").foregroundStyle(.secondary)
                                }
                            }
                        }.buttonStyle(.plain)
                        if let idle = view.up_next.idle {
                            NoCardsReadyView(view: idle)
                                .environment(\.reviewActions, addingGoesToReview)
                        } else {
                            StudyCard {
                                Text(view.up_next.title).font(.subheadline).foregroundStyle(.secondary)
                                Button { navigate(.review) } label: {
                                    VStack(alignment: .leading, spacing: 8) {
                                        Text(view.up_next.headline).font(.title2.bold()).foregroundStyle(Color.yapText)
                                        Text(view.up_next.kind_label).font(.subheadline).foregroundStyle(.secondary)
                                    }.frame(maxWidth: .infinity, alignment: .leading)
                                }.buttonStyle(.plain)
                                Button("Review") { navigate(.review) }
                                    .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent)
                            }
                            Button("\(view.up_next.ready_label) →") { navigate(.due) }
                                .font(.subheadline).frame(maxWidth: .infinity)
                        }
                        Button { navigate(.goals) } label: {
                            StudyCard { GoalProgress(goal: view.goal) }
                        }.buttonStyle(.plain)
                        HStack(alignment: .top, spacing: 16) {
                            StudyCard {
                                Text(view.streak.title).font(.subheadline).foregroundStyle(.secondary)
                                Text(view.streak.days_label).font(.title2.bold())
                                Text(view.streak.today_label).font(.subheadline).foregroundStyle(.secondary)
                            }
                            Button { navigate(.stats) } label: {
                                StudyCard {
                                    Text(view.stats.title).font(.subheadline).foregroundStyle(.secondary)
                                    Text(view.stats.cards_label).font(.title2.bold())
                                    Text(view.stats.percent_known_label).font(.subheadline).foregroundStyle(.secondary)
                                }
                            }.buttonStyle(.plain)
                        }
                        StudyCard {
                            Button(view.dictionary.title) { navigate(.dictionary()) }.font(.headline)
                            HStack {
                                TextField(view.dictionary.search_placeholder, text: $query)
                                    .textFieldStyle(.roundedBorder).submitLabel(.search)
                                    .onSubmit { navigate(.dictionary(query: query)) }
                                Button { navigate(.dictionary(query: query)) } label: { Image(systemName: "arrow.right") }
                                    .buttonStyle(.bordered).accessibilityLabel(view.dictionary.title)
                            }
                        }
                        Spacer(minLength: 0)
                        VStack(spacing: 8) {
                            Text("yap.town is created by [André Popovitch](https://twitter.com/chadnauseam).")
                            HStack(spacing: 12) {
                                Link("GitHub", destination: URL(string: "https://github.com/yaptown/yap")!)
                                Link("Discord", destination: URL(string: "https://discord.gg/mpgqfsH")!)
                                Link("About", destination: URL(string: "https://yap.town/about")!)
                                Link("Privacy", destination: URL(string: "https://yap.town/privacy")!)
                                Link("Terms", destination: URL(string: "https://yap.town/terms")!)
                            }
                        }.font(.caption).foregroundStyle(.secondary).frame(maxWidth: .infinity)
                    }.padding(20).frame(maxWidth: 600)
                        .frame(maxWidth: .infinity, minHeight: geometry.size.height)
                }
            }.background(Color(uiColor: .systemGroupedBackground))
                .navigationTitle("Yap.Town")
                #if DEBUG
                .onChange(of: DebugHarness.shared.commandID) { _, _ in
                    let command = DebugHarness.shared.command
                    guard isVisible, DebugHarness.shared.activeScreen == .home, command.hasPrefix("dump-fixture ") else { return }
                    DebugHarness.dumpFixture(.Home(view), name: String(command.dropFirst(13)))
                }
                #endif
        }
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                Button("Settings", systemImage: "gearshape") { navigate(.settings) }
                    .labelStyle(.iconOnly)
            }
        }
        #if DEBUG
        .onAppear { if isVisible { DebugHarness.shared.activeScreen = .home } }
        .onChange(of: isVisible) { _, visible in if visible { DebugHarness.shared.activeScreen = .home } }
        #endif
    }
}

struct GoalProgress: View {
    let goal: GoalCardView
    var body: some View {
        HStack {
            Text(goal.title).font(.headline)
            Spacer()
            Text(goal.percent_label).font(.subheadline).foregroundStyle(.secondary)
        }
        ProgressView(value: goal.percent, total: 100)
            .accessibilityLabel(goal.title).accessibilityValue(goal.percent_label)
        Text(goal.subtitle).font(.subheadline).foregroundStyle(.secondary)
    }
}
