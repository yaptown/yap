import SwiftUI

struct HomeScreen: View {
    @Environment(AuthStore.self) private var auth
    @Environment(AuthSheet.self) private var authSheet
    @Environment(\.reviewActions!) private var actions
    let review: ReviewModel
    let navigate: (CourseRoute) -> Void
    let searchTransition: Namespace.ID
    var view: HomeScreenView? = nil
    var isVisible = true
    @State private var snapshot = Snapshot()
    // While a route covers Home, reuse the last view instead of asking Rust again; keeping
    // the subtree mounted preserves scroll position for the way back.
    private final class Snapshot { var view: HomeScreenView? }
    private func screenView() -> HomeScreenView {
        if let view { return view }
        if isVisible || snapshot.view == nil {
            let deck = review.deck
            snapshot.view = deck.home_screen_view(inputs: review.inputs(
                sentenceList: review.session.curriculumDraft.map(\.selection) ?? deck.get_sentence_list()))
        }
        return snapshot.view!
    }
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
            let view = screenView()
            GeometryReader { geometry in
                ScrollView {
                    VStack(alignment: .leading, spacing: 16) {
                        Button(action: actions.switchCourse) {
                            // The quietest thing on the page: bare text, no surface.
                            HStack(spacing: 6) {
                                if Theme.emojiFontAvailable { Text(view.course_flag) }
                                Text(view.course_label)
                                Image(systemName: "chevron.down").font(.caption2.weight(.semibold))
                            }
                            .font(.subheadline).foregroundStyle(.secondary)
                            .padding(.vertical, 6).contentShape(Rectangle())
                        }.buttonStyle(.plain)
                        // A tagline, not a headline: Up Next's word is the page's one headline.
                        Text(view.greeting).font(.title2.weight(.medium)).foregroundStyle(Color.yapText)
                            .fixedSize(horizontal: false, vertical: true)
                        if let idle = view.up_next.idle {
                            IdleScreen(view: idle)
                                .environment(\.reviewActions, addingGoesToReview)
                                .environment(\.embeddedInHome, true)
                        } else {
                            // The whole card starts the review, not just its button.
                            Button { navigate(.review) } label: { UpNextCard(upNext: view.up_next) }
                                .buttonStyle(.plain)
                        }
                        // Everything below Up Next is one quiet panel of progress, so
                        // the next thing to study stays the only card that stands out.
                        VStack(alignment: .leading, spacing: 0) {
                            if let goal = view.goal {
                                Button { navigate(.goals) } label: {
                                    VStack(alignment: .leading, spacing: 12) { GoalProgress(goal: goal) }
                                        .padding(16).contentShape(Rectangle())
                                }.buttonStyle(.plain)
                                Divider()
                            }
                            VStack(spacing: 16) {
                                HStack(alignment: .firstTextBaseline) {
                                    Text(view.week.title).font(.headline).foregroundStyle(Color.yapText)
                                    Spacer()
                                    Text(view.week.today_label).font(.subheadline.monospacedDigit()).foregroundStyle(.secondary)
                                }
                                WeekProgressStrip(week: view.week.days)
                            }.padding(16)
                            Divider()
                            Button { navigate(.stats) } label: {
                                HStack(alignment: .top, spacing: 16) {
                                    HomeStat(stat: view.xp, systemImage: "bolt")
                                    HomeStat(stat: view.cards, systemImage: "book")
                                }.padding(16).contentShape(Rectangle())
                            }.buttonStyle(.plain)
                        }
                        .quietSurface()
                        // Not a real field: it zooms into the dictionary, whose
                        // search bar takes the focus and shows live results.
                        Button { navigate(.dictionary(searching: true)) } label: {
                            DictionarySearchBar(placeholder: view.dictionary.search_placeholder)
                        }.buttonStyle(.plain).accessibilityLabel(view.dictionary.title)
                            .matchedTransitionSource(id: DictionarySearchBar.id, in: searchTransition)
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
            }.background(.clear)
                #if DEBUG
                .onChange(of: DebugHarness.shared.commandID) { _, _ in
                    let command = DebugHarness.shared.command
                    guard isVisible, DebugHarness.shared.activeScreen == .home, command.hasPrefix("dump-fixture ") else { return }
                    DebugHarness.dumpFixture(.Home(view), name: String(command.dropFirst(13)))
                }
                #endif
        }
        // The wordmark sits in the bar, level with the trailing buttons, like the web
        // header on a phone; pushed screens inherit the inline title mode.
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .topBarLeading) {
                Text("Yap").font(.title2.bold()).foregroundStyle(Color.yapText).fixedSize()
                    .accessibilityAddTraits(.isHeader)
            }.hidingSharedBackground()
            ToolbarItem(placement: .topBarTrailing) {
                if auth.userId == nil {
                    Button(account_copy().sign_in_action) { authSheet.present(tab: .signIn) }
                }
            }
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
        HStack(spacing: 10) {
            Text(goal.name).font(.headline).foregroundStyle(Color.yapText)
            Text(goal.level_label).font(.caption).foregroundStyle(.secondary).lineLimit(1)
                .padding(.horizontal, 8).padding(.vertical, 3)
                .overlay { Capsule().strokeBorder(Color(uiColor: .separator)) }
            Spacer(minLength: 0)
            Text(goal.percent_label).font(.headline.monospacedDigit()).foregroundStyle(Color.yapText)
        }
        ProgressView(value: goal.percent, total: 100)
            .accessibilityLabel(goal.title).accessibilityValue(goal.percent_label)
        Text(goal.subtitle).font(.subheadline).foregroundStyle(.secondary)
    }
}

struct DictionarySearchBar: View {
    static let id = "dictionary-search"
    let placeholder: String
    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: "magnifyingglass")
            Text(placeholder).lineLimit(1)
            Spacer(minLength: 0)
        }
        .foregroundStyle(.secondary)
        .padding(.horizontal, 14).frame(height: 44)
        .quietSurface(cornerRadius: 14)
    }
}

/// Up Next: the page's one headline and its one saturated control. It sits on the
/// same surface as the panel below, so it stands out by content, not by glow.
private struct UpNextCard: View {
    let upNext: UpNextView
    private var symbol: String {
        switch upNext.kind {
        case .Flashcard: "text.bubble"
        case .Listening: "headphones"
        case .Pronunciation: "mic"
        case .Translation: "character.bubble"
        case .Transcription: "keyboard"
        case .Other: "sparkles"
        }
    }
    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Label(upNext.eyebrow.uppercased(), systemImage: symbol)
                .font(.caption.weight(.semibold)).tracking(1.5).foregroundStyle(.secondary)
            Text(upNext.headline).font(.system(size: 34, weight: .bold)).foregroundStyle(Color.yapText)
                .minimumScaleFactor(0.6).fixedSize(horizontal: false, vertical: true)
                .padding(.top, 6)
            HStack {
                Text(upNext.action_label)
                Spacer(minLength: 8)
                Image(systemName: "arrow.right")
            }
            .font(.headline).foregroundStyle(Color.yapOnAccent)
            .padding(.horizontal, 18).frame(height: 50)
            .background(Color.yapAccent, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
            .padding(.top, 20)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(20).quietSurface().contentShape(Rectangle())
    }
}

private struct HomeStat: View {
    let stat: HomeStatView
    let systemImage: String
    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(spacing: 8) {
                Image(systemName: systemImage).font(.title3).foregroundStyle(.secondary)
                Text(stat.value).font(.title2.bold().monospacedDigit()).foregroundStyle(Color.yapText)
            }
            Text(stat.caption).font(.subheadline).foregroundStyle(.secondary)
            if let note = stat.note { Text(note).font(.caption).foregroundStyle(.secondary).padding(.top, 2) }
        }
        .frame(maxWidth: .infinity, alignment: .topLeading)
    }
}
