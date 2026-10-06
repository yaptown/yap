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
        TimelineView(.every(10)) { _ in
            let view = screenView()
            GeometryReader { geometry in
                ScrollView {
                    VStack(alignment: .leading, spacing: 16) {
                        // A tagline, not a headline: Up Next's word is the page's one headline.
                        // The course inside it is the course switcher.
                        Button(action: actions.switchCourse) { HomeGreeting(view: view) }
                            .buttonStyle(.plain).padding(.top, 8)
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
                            if let curriculum = view.curriculum {
                                HomeCurriculum(view: curriculum, inert: self.view != nil)
                                Divider()
                            }
                            VStack(spacing: 16) {
                                HStack(alignment: .firstTextBaseline) {
                                    Text(view.week.title).font(.headline).foregroundStyle(Color.yapText)
                                    Spacer()
                                    GoalMenu(week: view.week) { if self.view == nil { actions.addEvent($0) } }
                                }
                                WeekProgressStrip(week: view.week.days).equatable()
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
                        }.font(.caption).foregroundStyle(Color.yapMuted).frame(maxWidth: .infinity)
                    }.padding(20).frame(maxWidth: 600)
                        .frame(maxWidth: .infinity, minHeight: geometry.size.height)
                }
            }.background(.clear)
                #if DEBUG
                .onChange(of: DebugHarness.shared.commandID) { _, _ in
                    let command = DebugHarness.shared.command
                    guard isVisible, DebugHarness.shared.activeScreen == .home else { return }
                    if command == "sign-in" { authSheet.present(tab: .signIn) }
                    guard command.hasPrefix("dump-fixture ") else { return }
                    DebugHarness.dumpFixture(.Home(view), name: String(command.dropFirst(13)))
                }
                #endif
        }
        // The wordmark sits in the bar, level with the trailing buttons, like the web
        // header on a phone; pushed screens inherit the inline title mode.
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .topBarLeading) {
                HStack(spacing: 8) {
                    Text("Yap").font(.title2.bold()).foregroundStyle(Color.yapText).fixedSize()
                        .accessibilityAddTraits(.isHeader)
                    SyncDot { navigate(.settings) }
                }
            }.hidingSharedBackground()
            ToolbarItem(placement: .topBarTrailing) {
                if auth.userId == nil {
                    Button(account_copy().sign_in_action) { authSheet.present(tab: .signIn) }
                }
            }
            ToolbarItem(placement: .topBarTrailing) {
                // Icon-only bar buttons draw the inherited tint in its light-scheme
                // color, invisible on a dark bar; an explicit style follows the scheme.
                Button("Settings", systemImage: "gearshape") { navigate(.settings) }
                    .labelStyle(.iconOnly).foregroundStyle(Color.yapAccent)
            }
        }
        #if DEBUG
        .onAppear { if isVisible { DebugHarness.shared.activeScreen = .home } }
        .onChange(of: isVisible) { _, visible in if visible { DebugHarness.shared.activeScreen = .home } }
        #endif
    }
}

/// Web's header dot: shown while something isn't synced, colored by how much it
/// matters. Settings holds the sync details, so that's where a tap goes.
private struct SyncDot: View {
    @Environment(\.reviewHost!) private var host
    let open: () -> Void
    var body: some View {
        TimelineView(.every(1)) { context in
            if let sync = host.weapon?.sync_status(online: host.online, now_ms: context.date.timeIntervalSince1970 * 1000,
                                                   manual_sync_in_flight: false, host_sync_error: host.syncError),
               let indicator = sync.indicator {
                Button(action: open) {
                    Circle().fill(color(indicator)).frame(width: 8, height: 8).frame(width: 24, height: 24).contentShape(Rectangle())
                }.buttonStyle(.plain).accessibilityLabel(sync.label)
            }
        }
    }
    private func color(_ severity: SyncSeverity) -> Color {
        switch severity {
        case .Neutral: .yapMuted
        case .Caution: .yapCaution
        case .Negative: .yapNegative
        }
    }
}

/// What Home is studying from. The name is a menu of categories; a movie or
/// lesson opens the browser for the rest of its category. Both commit on the spot.
private struct HomeCurriculum: View {
    @Environment(\.reviewHost!) private var host
    @Environment(\.reviewActions!) private var actions
    @AppStorage("yap-pimsleur-acknowledged") private var pimsleurAcknowledged = false
    let view: HomeCurriculumView
    let inert: Bool
    @State private var browsing: SentenceListCategory?
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(spacing: 10) {
                Menu {
                    ForEach(view.options, id: \.category) { option in
                        Button { choose(option.category) } label: {
                            if option.category == view.category { Label(option.label, systemImage: "checkmark") } else { Text(option.label) }
                        }
                    }
                } label: {
                    HStack(spacing: 4) {
                        Text(view.menu_label).font(.headline)
                        Image(systemName: "chevron.down").font(.caption.weight(.semibold)).foregroundStyle(Color.yapMuted)
                    }.foregroundStyle(Color.yapText).contentShape(Rectangle())
                }
                if let level = view.level_label {
                    Text(level).font(.caption).foregroundStyle(Color.yapMuted).lineLimit(1)
                        .padding(.horizontal, 8).padding(.vertical, 3)
                        .overlay { Capsule().strokeBorder(Color(uiColor: .separator)) }
                }
                Spacer(minLength: 0)
            }
            if let item = view.item {
                HStack(spacing: 14) {
                    Group {
                        if let id = item.movie_id { PosterImage(id: id) }
                        else { Color.yapMutedSurface.overlay { Image(systemName: "headphones").foregroundStyle(Color.yapMuted) } }
                    }
                    .frame(width: 44, height: 66).clipShape(RoundedRectangle(cornerRadius: 6))
                    VStack(alignment: .leading, spacing: 8) {
                        HStack(spacing: 8) {
                            Text(item.title).font(.body.weight(.medium)).foregroundStyle(Color.yapText).lineLimit(1)
                            Spacer(minLength: 0)
                            Button(item.browse_label) { if !inert { browsing = view.category } }
                                .buttonStyle(.bordered).controlSize(.small).fixedSize()
                        }
                        progress(label: item.title)
                    }
                }
            } else {
                progress(label: view.menu_label)
            }
            if let subtitle = view.subtitle { Text(subtitle).font(.subheadline).foregroundStyle(Color.yapMuted) }
        }
        .padding(16)
        .sheet(item: $browsing) { category in CurriculumBrowser(tab: category) }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeScreen == .home else { return }
            let command = DebugHarness.shared.command
            if command.hasPrefix("curriculum "), let index = Int(command.dropFirst(11)), view.options.indices.contains(index) {
                choose(view.options[index].category)
            }
            if command == "browse" { browsing = view.category }
        }
        #endif
    }
    private func progress(label: String) -> some View {
        HStack(spacing: 10) {
            ProgressView(value: view.percent, total: 100)
            Text(view.percent_label).font(.subheadline.weight(.semibold).monospacedDigit()).foregroundStyle(Color.yapText)
        }
        .accessibilityElement(children: .ignore).accessibilityLabel(label).accessibilityValue(view.percent_label)
    }
    private func choose(_ category: SentenceListCategory) {
        guard !inert, category != view.category else { return }
        // The disclaimer lives in the browser, so an unacknowledged Pimsleur goes there first.
        if category == .Pimsleur && !pimsleurAcknowledged { browsing = category }
        else { actions.commitSentenceList(host.deck.choose_curriculum_category(category: category)) }
    }
}

extension SentenceListCategory: Identifiable {
    public var id: Self { self }
}

/// "0 / [5 min ⌄] today": the goal is a menu of the daily goals.
private struct GoalMenu: View {
    let week: WeekCardView
    let choose: (DeckEvent) -> Void
    var body: some View {
        HStack(spacing: 0) {
            Text(week.today.before)
            Menu {
                ForEach(Array(week.goal_options.enumerated()), id: \.offset) { _, option in
                    Button { if option.target != week.daily_goal { choose(option.event) } } label: {
                        if option.target == week.daily_goal { Label(option.minutes_label, systemImage: "checkmark") } else { Text(option.minutes_label) }
                        Text(option.label)
                    }
                }
            } label: {
                HStack(spacing: 2) {
                    Text(week.today.emphasis).fontWeight(.semibold).foregroundStyle(Color.yapText)
                    Image(systemName: "chevron.down").font(.caption2.weight(.semibold))
                }.contentShape(Rectangle())
            }
            Text(week.today.after)
        }
        .font(.subheadline.monospacedDigit()).foregroundStyle(Color.yapMuted)
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            let command = DebugHarness.shared.command
            guard DebugHarness.shared.activeScreen == .home, command.hasPrefix("goal "),
                  let index = Int(command.dropFirst(5)), week.goal_options.indices.contains(index) else { return }
            choose(week.goal_options[index].event)
        }
        #endif
    }
}

/// The greeting is one run of text, so the artwork and words wrap like a sentence.
private struct HomeGreeting: View {
    @Environment(\.colorScheme) private var colorScheme
    @ScaledMetric(relativeTo: .title2) private var fontSize = 22.0
    let view: HomeScreenView

    private var courseIcon: Text {
        let traits = UITraitCollection(userInterfaceStyle: colorScheme == .dark ? .dark : .light)
        guard let source = UIImage(named: "\(view.course_icon)-bare", in: .main, compatibleWith: traits) else { return Text("") }
        let font = UIFont.systemFont(ofSize: fontSize, weight: .medium)
        let side = font.pointSize * 1.1
        let size = CGSize(width: side, height: side)
        var rendered = UIImage()
        traits.performAsCurrent {
            rendered = UIGraphicsImageRenderer(size: size).image { _ in
                source.draw(in: CGRect(origin: .zero, size: size))
            }
        }
        let image = rendered.withBaselineOffset(fromBottom: (side - font.capHeight) / 2)
        return Text(Image(uiImage: image).renderingMode(.original)) + Text(" ")
    }

    var body: some View {
        (Text("\(view.greeting_lead) ") + courseIcon + Text(view.course_name).fontWeight(.semibold)
            + Text(" \(Image(systemName: "chevron.down"))").font(.headline).foregroundStyle(Color.yapMuted)
            + Text(view.greeting_tail))
            .font(.title2.weight(.medium)).foregroundStyle(Color.yapText)
            .fixedSize(horizontal: false, vertical: true)
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
        .foregroundStyle(Color.yapMuted)
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
                .font(.caption.weight(.semibold)).tracking(1.5).foregroundStyle(Color.yapMuted)
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
                Image(systemName: systemImage).font(.title3).foregroundStyle(Color.yapMuted)
                Text(stat.value).font(.title2.bold().monospacedDigit()).foregroundStyle(Color.yapText)
            }
            Text(stat.caption).font(.subheadline).foregroundStyle(Color.yapMuted)
            if let note = stat.note { Text(note).font(.caption).foregroundStyle(Color.yapMuted).padding(.top, 2) }
        }
        .frame(maxWidth: .infinity, alignment: .topLeading)
    }
}
