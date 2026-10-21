import CoreText
import SwiftUI

struct BooksScreen: View {
    let language: Language
    let nativeLanguage: Language
    @State private var catalog: BooksScreenView?
    @State private var failed = false
    private var copy: BooksCopy { catalog?.copy ?? books_copy() }
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if failed { Text(copy.failed).foregroundStyle(Color.yapMuted) }
                if let books = catalog?.books {
                    if books.isEmpty { Text(copy.empty).foregroundStyle(Color.yapMuted) }
                    ForEach(books, id: \.slug) { book in
                        NavigationLink(value: CourseRoute.book(book.slug)) {
                            BookCard(book: book)
                        }.buttonStyle(.plain)
                    }
                } else if !failed { ProgressView(copy.loading).frame(maxWidth: .infinity) }
            }.padding(20).frame(maxWidth: .readableColumn).frame(maxWidth: .infinity)
        }.navigationTitle(copy.title)
            .task {
                do { catalog = try await load_books(language: language, native_language: nativeLanguage) } catch { failed = true }
            }
            #if DEBUG
            .onAppear { DebugHarness.shared.activeScreen = .books }
            .onChange(of: DebugHarness.shared.commandID) { _, _ in
                guard DebugHarness.shared.activeScreen == .books, DebugHarness.shared.command == "status" else { return }
                DebugHarness.log("books count=\(catalog?.books.count ?? 0) failed=\(failed)")
            }
            #endif
    }
}

/// The book's cover, or for a book without one a stable sea-toned tile from its slug
/// (the same hash as the web's BookCover).
private struct BookCover: View {
    let book: BookCardView
    private var hue: Double {
        var hash: Int32 = 0
        for unit in book.slug.utf16 { hash = hash &* 31 &+ Int32(unit) }
        return Double(190 + Int(hash.magnitude % 60)) / 360
    }
    var body: some View {
        let shape = RoundedRectangle(cornerRadius: 6, style: .continuous)
        Group {
            if let cover = book.cover, let url = URL(string: cover) {
                AsyncImage(url: url) { image in image.resizable().scaledToFill() } placeholder: { tile }
            } else { tile }
        }
        .frame(width: 80, height: 120)
        .clipShape(shape)
        .overlay(shape.strokeBorder(.black.opacity(0.1)))
        .shadow(color: .black.opacity(0.15), radius: 2, y: 1)
        .accessibilityHidden(true)
    }
    private var tile: some View {
        LinearGradient(
            colors: [Color(hue: hue, saturation: 0.6, brightness: 0.65), Color(hue: hue + 25 / 360, saturation: 0.75, brightness: 0.33)],
            startPoint: .topLeading, endPoint: .bottomTrailing)
            .overlay(alignment: .bottomLeading) {
                Text(String(book.title.prefix(1))).font(.system(.title, design: .serif)).foregroundStyle(.white.opacity(0.85)).padding(8)
            }
    }
}

private struct BookCard: View {
    let book: BookCardView
    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack(spacing: 16) {
                BookCover(book: book)
                VStack(alignment: .leading, spacing: 4) {
                    Text(book.title).font(.system(.title3, design: .serif).weight(.semibold)).foregroundStyle(Color.yapText)
                    Text(book.author).foregroundStyle(Color.yapMuted)
                }
                Spacer(minLength: 0)
                Image(systemName: "chevron.right").font(.footnote).foregroundStyle(Color.yapMuted)
            }
            if let teaser = book.teaser {
                Text(teaser).font(.subheadline).foregroundStyle(Color.yapMuted)
            }
        }
        .padding(16).frame(maxWidth: .infinity, alignment: .leading).quietSurface()
    }
}

/// Where to open a chapter: a paragraph, or its last page when paging backwards.
private enum Opening: Equatable { case paragraph(Int), end }

struct BookReaderScreen: View {
    let language: Language
    let nativeLanguage: Language
    let slug: String
    @Environment(\.dismiss) private var dismiss
    @Environment(AuthStore.self) private var auth
    /// The tapped sentence's tutor, shared with web through Rust's `book_sentence` reducer.
    @State private var tutor: BookSentenceState?
    @State private var tutorTask: Task<Void, Never>?
    @State private var view: BookReaderView?
    @State private var opening = Opening.paragraph(0)
    /// Bumped on every completed load, so reopening the current chapter starts it afresh.
    @State private var loads = 0
    @State private var failed = false
    @State private var requestID = 0
    @State private var showingContents = false
    @State private var barShown = true
    @State private var showingSettings = false
    @Environment(\.colorScheme) private var colorScheme
    @AppStorage("yap-reader-theme") private var themeID = "system"
    private var themes: [ReaderTheme] { reader_themes(dark: colorScheme == .dark) }
    private var paper: Paper { Paper(theme: themes.first { $0.id == themeID } ?? themes[0]) }
    @AppStorage("yap-reader-text-size") private var textSize: Int?
    @State private var saveTask: Task<Void, Never>?
    private let copy = books_copy()
    var body: some View {
        GeometryReader { geometry in
            let layout = reader_layout(width: geometry.size.width, height: geometry.size.height, text_size: textSize.map(UInt32.init))
            ZStack(alignment: .top) {
                if let view {
                    let highlight = tutor.map { book_sentence_view(state: $0) }.flatMap { tutor in
                        tutor.highlight.map { (paragraph: Int(tutor.paragraph_index), range: $0) }
                    }
                    ChapterPages(view: view, layout: layout, paper: paper, controlsShown: barShown, opening: opening, remember: remember,
                                 highlight: highlight,
                                 tapText: { paragraph, offset in
                                     guard let text = view.paragraphs[paragraph].text else { return }
                                     startTutor(book_sentence_start(language: language, native_language: nativeLanguage,
                                                                    paragraph_index: UInt64(paragraph), paragraph: text, utf16_offset: UInt32(offset)))
                                 },
                                 tapAway: tutor == nil ? nil : { closeTutor() },
                                 turned: { closeTutor(); withAnimation { barShown = false } },
                                 turnPast: { direction in
                                     guard let chapter = direction == 1 ? view.next : view.previous else { return false }
                                     Task { await load(chapter: chapter, opening: direction == 1 ? .paragraph(0) : .end) }
                                     return true
                                 },
                                 tapMiddle: { withAnimation { barShown.toggle() } })
                        .id("\(view.position.chapter)-\(loads)")
                } else {
                    Group {
                        if failed { Text(copy.failed) } else { ProgressView(copy.loading) }
                    }.foregroundStyle(paper.muted).frame(maxWidth: .infinity).padding(.top, 96)
                }
                if barShown || view == nil {
                    ReaderBar(copy: copy, paper: paper, back: { dismiss() }, settings: { showingSettings = true },
                              contents: view == nil ? nil : { showingContents = true })
                        .transition(.opacity)
                }
            }
            .sheet(isPresented: $showingSettings) {
                VStack(alignment: .leading, spacing: 24) {
                    Text(copy.appearance).font(.headline)
                    HStack {
                        Button { layout.smaller.map { textSize = Int($0) } } label: { Text("A").font(.system(size: 18, design: .serif)).frame(maxWidth: .infinity, minHeight: 44) }
                            .disabled(layout.smaller == nil).accessibilityLabel(copy.text_smaller)
                        Text("\(Int(layout.font_size))").font(.caption.monospacedDigit()).foregroundStyle(paper.muted)
                        Button { layout.larger.map { textSize = Int($0) } } label: { Text("A").font(.system(size: 30, design: .serif)).frame(maxWidth: .infinity, minHeight: 44) }
                            .disabled(layout.larger == nil).accessibilityLabel(copy.text_larger)
                    }.background(paper.text.opacity(0.05), in: Capsule())
                    LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                        ForEach(themes, id: \.id) { theme in
                            let option = Paper(theme: theme)
                            Button { themeID = theme.id } label: {
                                VStack(spacing: 4) { Text("Aa").font(.system(size: 30, design: .serif)); Text(theme.label).font(.caption) }
                                    .frame(maxWidth: .infinity).padding(.vertical, 16)
                                    .foregroundStyle(option.text).background(option.background, in: RoundedRectangle(cornerRadius: 16))
                                    .overlay(RoundedRectangle(cornerRadius: 16).stroke(option.text.opacity(themeID == theme.id ? 1 : 0.15), lineWidth: themeID == theme.id ? 2 : 1))
                            }.accessibilityAddTraits(themeID == theme.id ? .isSelected : [])
                        }
                    }
                }.buttonStyle(.plain).padding(24).foregroundStyle(paper.text)
                    .presentationBackground(paper.background).presentationDetents([.height(380)])
                    .presentationDragIndicator(.visible)
            }
        }
        .foregroundStyle(paper.text)
        .background(paper.background.ignoresSafeArea())
        .toolbar(.hidden, for: .navigationBar)
        .statusBarHidden()
        .persistentSystemOverlays(.hidden)
        .sheet(isPresented: $showingContents) {
            if let view {
                BookContents(view: view, copy: copy, paper: paper) { chapter in
                    showingContents = false
                    Task { await load(chapter: chapter, opening: .paragraph(0)) }
                }.presentationDetents([.medium, .large])
            }
        }
        .sheet(isPresented: Binding(get: { tutor != nil }, set: { if !$0 { closeTutor() } })) {
            if let tutor {
                SentenceTutor(view: book_sentence_view(state: tutor), lang: view?.lang ?? "", paper: paper, send: sendTutor)
            }
        }
        .task { await load() }
        #if DEBUG
        .onAppear { DebugHarness.shared.activeScreen = .bookReader }
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeScreen == .bookReader else { return }
            let command = DebugHarness.shared.command
            let words = command.split(separator: " ")
            if words.first == "tap-text", words.count == 3, let paragraph = Int(words[1]), let offset = UInt32(words[2]),
               let text = view?.paragraphs[paragraph].text {
                startTutor(book_sentence_start(language: language, native_language: nativeLanguage,
                                               paragraph_index: UInt64(paragraph), paragraph: text, utf16_offset: offset))
            }
            if command.hasPrefix("ask ") { sendTutor(.DraftChanged(text: String(command.dropFirst(4)))); sendTutor(.Ask) }
            if command == "tutor-close" { closeTutor() }
            if command == "status", let tutor {
                let view = book_sentence_view(state: tutor)
                DebugHarness.log("tutor sentence=\(view.sentence ?? "-") word=\(view.word ?? "-") status=\(view.status ?? "-") messages=\(view.messages.count)")
            }
            if command == "contents" { showingContents.toggle() }
            if command == "bar" { withAnimation { barShown.toggle() } }
            if command.hasPrefix("chapter "), let chapter = UInt64(command.dropFirst(8)) { Task { await load(chapter: chapter, opening: .paragraph(0)) } }
            if command.hasPrefix("text-size "), let step = Int(command.dropFirst(10)) { textSize = step }
        }
        #endif
    }
    /// nil opens at the saved position; a chapter opens at its start or end.
    private func load(chapter: UInt64? = nil, opening requested: Opening = .paragraph(0)) async {
        requestID += 1
        let request = requestID
        do {
            await saveTask?.value
            guard request == requestID, !Task.isCancelled else { return }
            let result = try await load_book(language: language, slug: slug, position: chapter.map { BookPosition(chapter: $0, paragraph: 0) })
            guard request == requestID, !Task.isCancelled else { return }
            closeTutor()
            view = result
            opening = chapter == nil ? .paragraph(Int(result.position.paragraph)) : requested
            loads += 1
            failed = false
        } catch { if request == requestID { failed = true } }
    }
    private func startTutor(_ step: BookSentenceStep) {
        tutorTask?.cancel()
        applyTutor(step)
    }
    private func closeTutor() {
        tutorTask?.cancel()
        tutor = nil
    }
    private func sendTutor(_ event: BookSentenceEvent) {
        guard let tutor else { return }
        applyTutor(book_sentence_transition(state: tutor, event: event))
    }
    private func applyTutor(_ step: BookSentenceStep) {
        tutor = step.state
        let token = auth.accessToken
        for effect in step.effects {
            tutorTask = Task { @MainActor in
                let event: BookSentenceEvent
                switch effect {
                case let .Explain(request):
                    do { event = .Explained(explanation: try await explain_book_sentence(request: request, access_token: token)) }
                    catch { event = .ExplainFailed }
                case let .Chat(request):
                    do { event = .Replied(reply: try await chat_book_sentence(request: request, access_token: token)) }
                    catch { event = .ReplyFailed }
                }
                guard !Task.isCancelled else { return }
                sendTutor(event)
            }
        }
    }
    private func remember(_ paragraph: Int) {
        guard let view else { return }
        let previous = saveTask
        saveTask = Task {
            await previous?.value
            try? await save_book_position(language: language, slug: slug, position: BookPosition(chapter: view.position.chapter, paragraph: UInt64(paragraph)))
        }
    }
}

/// Shared Rust palette, resolved once for this reader's selected appearance.
private struct Paper {
    let theme: ReaderTheme
    var background: Color { Color(uiColor: Self.color(theme.background)) }
    var text: Color { Color(uiColor: ink) }
    var muted: Color { Color(uiColor: mutedInk) }
    var ink: UIColor { Self.color(theme.ink) }
    var mutedInk: UIColor { Self.color(theme.muted) }
    var highlight: UIColor { Self.color(theme.highlight) }
    /// `#rrggbb`, or `#rrggbbaa` with alpha.
    private static func color(_ hex: String) -> UIColor {
        let digits = hex.dropFirst()
        let value = UInt32(digits, radix: 16) ?? 0
        let rgba = digits.count == 8 ? value : value << 8 | 255
        func channel(_ shift: UInt32) -> CGFloat { CGFloat((rgba >> shift) & 255) / 255 }
        return UIColor(red: channel(24), green: channel(16), blue: channel(8), alpha: channel(0))
    }
}

/// Floating controls do not change the page's geometry when revealed.
private struct ReaderBar: View {
    let copy: BooksCopy
    let paper: Paper
    let back: () -> Void
    let settings: () -> Void
    let contents: (() -> Void)?
    var body: some View {
        HStack(alignment: .top) {
            button("chevron.left", copy.title, back)
                .background(paper.background.opacity(0.95), in: Capsule())
                .overlay(Capsule().stroke(paper.text.opacity(0.1)))
            Spacer()
            HStack(spacing: 4) {
                button("list.bullet", copy.contents, contents ?? {}).disabled(contents == nil)
                Button(action: settings) { Text("Aa").font(.system(size: 18)).frame(width: 44, height: 44) }
                    .disabled(contents == nil).accessibilityLabel(copy.appearance)
            }.padding(4).background(paper.background.opacity(0.95), in: Capsule())
                .overlay(Capsule().stroke(paper.text.opacity(0.1)))
        }.buttonStyle(.plain).shadow(color: .black.opacity(0.1), radius: 10, y: 4)
            .padding(.horizontal, 20).padding(.top, 16)
    }
    private func button(_ symbol: String, _ label: String, _ action: @escaping () -> Void) -> some View {
        Button(action: action) { Image(systemName: symbol).frame(width: 44, height: 44).contentShape(Rectangle()) }
            .accessibilityLabel(label)
    }
}

/// A chapter laid out into pages with TextKit and turned with a native curl. The geometry
/// comes from Rust's `reader_layout`, shared with web. Pages -1 and `spreads` are blank
/// sentinels: landing on one turns into the neighbouring chapter.
private struct ChapterPages: View {
    let view: BookReaderView
    let layout: ReaderLayout
    let paper: Paper
    let controlsShown: Bool
    let opening: Opening
    let remember: (Int) -> Void
    /// The tutor's sentence, as a UTF-16 range within its paragraph.
    let highlight: (paragraph: Int, range: BookSentenceHighlight)?
    /// A tap on a paragraph's text, at a UTF-16 offset.
    let tapText: (Int, Int) -> Void
    /// While the tutor is open, a tap off the text closes it instead of turning or showing the bar.
    let tapAway: (() -> Void)?
    /// Called on every page turn the reader makes.
    let turned: () -> Void
    /// Turning beyond the first or last page; false at the start or end of the book.
    let turnPast: (Int) -> Bool
    let tapMiddle: () -> Void
    @State private var pagination: Pagination?
    @State private var spread = 0
    // The paragraph to keep in view when the layout changes (rotation, text size).
    @State private var anchor: Opening?
    // Set when the layout picked the spread, so the spread it lands on doesn't move the anchor:
    // a paragraph straddling two pages would otherwise pull every relayout back a page.
    @State private var placed = false
    var body: some View {
        let spreads = pagination?.spreads(layout) ?? 1
        CurlPages(pagination: pagination, layout: layout, paper: paper, spread: $spread,
                  hasPrevious: view.previous != nil, hasNext: view.next != nil,
                  tapText: tapText, tapAway: tapAway, tapMiddle: tapMiddle)
        .id(layout.columns)
        .accessibilityScrollAction { direction in
            if direction == .trailing { turn(1) }
            if direction == .leading { turn(-1) }
        }
        .overlay(alignment: .bottom) {
            HStack(spacing: 0) {
                ForEach(0..<Int(layout.columns), id: \.self) { column in
                    let page = min(max(spread, 0), spreads - 1) * Int(layout.columns) + column + 1
                    Text(page <= (pagination?.containers.count ?? 0) ? "\(page)" : "")
                        .frame(maxWidth: .infinity)
                }
            }.font(.caption.monospacedDigit()).foregroundStyle(paper.muted).padding(.bottom, 24)
        }
        .overlay(alignment: .top) {
            HStack(spacing: 0) {
                Text(view.title).lineLimit(1).padding(.horizontal, 64).frame(maxWidth: .infinity)
                if layout.columns == 2 { Text(view.chapter_title ?? "").lineLimit(1).padding(.horizontal, 64).frame(maxWidth: .infinity) }
            }.font(.caption).foregroundStyle(paper.muted).padding(.top, 28).opacity(controlsShown ? 0 : 1).allowsHitTesting(false)
        }
        .onChange(of: paper.theme) { _, _ in place(layout) }
        .onChange(of: highlight?.paragraph, initial: true) { _, _ in pagination?.highlight(highlight, color: paper.highlight) }
        .onChange(of: highlight?.range) { _, _ in pagination?.highlight(highlight, color: paper.highlight) }
        .onChange(of: layout, initial: true) { _, layout in place(layout) }
        .onChange(of: spread) { old, new in
            if new < 0 || new >= spreads {
                turned()
                if !turnPast(new < 0 ? -1 : 1) { withAnimation { spread = old } }
                return
            }
            guard let pagination else { return }
            let first = pagination.firstParagraph(onPage: new * Int(layout.columns))
            if placed { placed = false } else { anchor = .paragraph(first); turned() }
            remember(first)
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeScreen == .bookReader else { return }
            let command = DebugHarness.shared.command
            if command == "next" { turn(1) }
            let words = command.split(separator: " ").compactMap { Double($0) }
            if command.hasPrefix("tap "), words.count == 2 { NotificationCenter.default.post(name: .debugReaderTap, object: CGPoint(x: words[0], y: words[1])) }
            if command == "previous" { turn(-1) }
            if command.hasPrefix("page "), let page = Int(command.dropFirst(5)) { withAnimation { spread = page - 1 } }
            if command == "status" { DebugHarness.log("book chapter=\(view.position.chapter) page=\(spread + 1)/\(spreads) anchor=\(String(describing: anchor))") }
        }
        #endif
    }
    private func turn(_ direction: Int) {
        let next = spread + direction
        let count = pagination?.spreads(layout) ?? 1
        if next < 0 || next >= count { turned(); _ = turnPast(direction) }
        else { spread = next }
    }
    private func place(_ layout: ReaderLayout) {
        let pagination = Pagination(view: view, layout: layout, paper: paper)
        pagination.highlight(highlight, color: paper.highlight)
        self.pagination = pagination
        let spreads = pagination.spreads(layout)
        let target = switch anchor ?? opening {
        case let .paragraph(paragraph): pagination.page(ofParagraph: paragraph) / Int(layout.columns)
        case .end: spreads - 1
        }
        let next = min(target, spreads - 1)
        if next != spread { placed = true; spread = next }
        if anchor == nil {
            let first = pagination.firstParagraph(onPage: next * Int(layout.columns))
            anchor = opening == .end ? .paragraph(first) : opening
            remember(first)
        }
    }
}

/// UIKit supplies a flexible sheet on iPhone and a centre-spined spread on iPad.
/// TextKit remains responsible for pagination and reading anchors.
private struct CurlPages: UIViewControllerRepresentable {
    let pagination: Pagination?
    let layout: ReaderLayout
    let paper: Paper
    @Binding var spread: Int
    let hasPrevious: Bool
    let hasNext: Bool
    let tapText: (Int, Int) -> Void
    let tapAway: (() -> Void)?
    let tapMiddle: () -> Void
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    func makeCoordinator() -> Coordinator { Coordinator(self) }
    func makeUIViewController(context: Context) -> UIPageViewController {
        let controller = UIPageViewController(transitionStyle: .pageCurl, navigationOrientation: .horizontal,
            options: [.spineLocation: NSNumber(value: layout.columns == 2 ? UIPageViewController.SpineLocation.mid.rawValue : UIPageViewController.SpineLocation.min.rawValue)])
        controller.isDoubleSided = layout.columns == 2
        controller.dataSource = context.coordinator
        controller.delegate = context.coordinator
        for recognizer in controller.gestureRecognizers where recognizer is UITapGestureRecognizer { recognizer.isEnabled = false }
        let tap = UITapGestureRecognizer(target: context.coordinator, action: #selector(Coordinator.tap(_:)))
        tap.delegate = context.coordinator
        controller.view.addGestureRecognizer(tap)
        #if DEBUG
        NotificationCenter.default.addObserver(forName: .debugReaderTap, object: nil, queue: .main) { [weak coordinator = context.coordinator, weak controller] note in
            let point = note.object as! CGPoint
            MainActor.assumeIsolated { if let controller { coordinator?.tap(at: controller.view.convert(point, from: nil), in: controller.view) } }
        }
        #endif
        for direction: UISwipeGestureRecognizer.Direction in [.left, .right] {
            let swipe = UISwipeGestureRecognizer(target: context.coordinator, action: #selector(Coordinator.swipe(_:)))
            swipe.direction = direction
            controller.view.addGestureRecognizer(swipe)
            context.coordinator.swipes.append(swipe)
        }
        return controller
    }
    func updateUIViewController(_ controller: UIPageViewController, context: Context) {
        let coordinator = context.coordinator
        coordinator.parent = self
        for recognizer in controller.gestureRecognizers where !(recognizer is UITapGestureRecognizer) { recognizer.isEnabled = !reduceMotion }
        coordinator.swipes.forEach { $0.isEnabled = reduceMotion }
        controller.view.backgroundColor = UIColor(paper.background)
        coordinator.render(controller)
    }

    final class Leaf: UIHostingController<AnyView> {
        let index: Int
        init(index: Int, content: AnyView) { self.index = index; super.init(rootView: content) }
        @MainActor required dynamic init?(coder: NSCoder) { fatalError() }
    }
    final class Coordinator: NSObject, UIPageViewControllerDataSource, UIPageViewControllerDelegate, UIGestureRecognizerDelegate {
        var parent: CurlPages
        var pagination: Pagination?
        var turning = false
        var swipes: [UISwipeGestureRecognizer] = []
        init(_ parent: CurlPages) { self.parent = parent }
        func render(_ controller: UIPageViewController) {
            guard let nextPagination = parent.pagination, !turning else { return }
            let changed = pagination !== nextPagination
            let first = parent.spread * Int(parent.layout.columns)
            let current = (controller.viewControllers?.first as? Leaf)?.index
            guard changed || current != first else { return }
            pagination = nextPagination
            let leaves = (0..<Int(parent.layout.columns)).map { leaf(first + $0) }
            turning = true
            controller.setViewControllers(leaves, direction: first >= (current ?? 0) ? .forward : .reverse,
                                          animated: !changed && current != nil && !parent.reduceMotion) { [weak self, weak controller] _ in
                guard let self, let controller else { return }
                turning = false
                // A resize or appearance change during the curl must still reach UIKit.
                render(controller)
            }
        }
        func leaf(_ index: Int) -> Leaf {
            let content = AnyView(Group {
                if let pagination, pagination.containers.indices.contains(index) {
                    PageCanvas(pagination: pagination, page: index)
                        .frame(width: parent.layout.page_width, height: parent.layout.page_height)
                } else { Color.clear.frame(width: parent.layout.page_width, height: parent.layout.page_height) }
            }.frame(maxWidth: .infinity, alignment: parent.layout.columns == 1 ? .center : (index.isMultiple(of: 2) ? .trailing : .leading))
                .padding(.horizontal, parent.layout.columns == 2 ? parent.layout.gap / 2 : 0)
                .padding(.top, parent.layout.top)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
                .background(parent.paper.background))
            let leaf = Leaf(index: index, content: content)
            leaf.view.backgroundColor = UIColor(parent.paper.background)
            return leaf
        }
        func pageViewController(_ pageViewController: UIPageViewController, viewControllerBefore viewController: UIViewController) -> UIViewController? {
            guard let index = (viewController as? Leaf)?.index else { return nil }
            let minimum = parent.hasPrevious ? -Int(parent.layout.columns) : 0
            return index > minimum ? leaf(index - 1) : nil
        }
        func pageViewController(_ pageViewController: UIPageViewController, viewControllerAfter viewController: UIViewController) -> UIViewController? {
            guard let index = (viewController as? Leaf)?.index, let pagination else { return nil }
            let limit = (pagination.spreads(parent.layout) + (parent.hasNext ? 1 : 0)) * Int(parent.layout.columns)
            return index + 1 < limit ? leaf(index + 1) : nil
        }
        func pageViewController(_ pageViewController: UIPageViewController, willTransitionTo pendingViewControllers: [UIViewController]) { turning = true }
        func pageViewController(_ pageViewController: UIPageViewController, didFinishAnimating finished: Bool, previousViewControllers: [UIViewController], transitionCompleted completed: Bool) {
            turning = false
            guard pagination === parent.pagination else { render(pageViewController); return }
            guard completed, let index = (pageViewController.viewControllers?.first as? Leaf)?.index else { return }
            parent.spread = Int(floor(Double(index) / Double(parent.layout.columns)))
        }
        func gestureRecognizer(_ gestureRecognizer: UIGestureRecognizer, shouldRecognizeSimultaneouslyWith otherGestureRecognizer: UIGestureRecognizer) -> Bool { true }
        private func step(_ direction: Int) {
            guard !turning, let pagination else { return }
            let next = parent.spread + direction
            if next < 0 && !parent.hasPrevious { return }
            if next >= pagination.spreads(parent.layout) && !parent.hasNext { return }
            parent.spread = next
        }
        @objc func tap(_ recognizer: UITapGestureRecognizer) {
            guard let view = recognizer.view else { return }
            tap(at: recognizer.location(in: view), in: view)
        }
        func tap(at point: CGPoint, in view: UIView) {
            guard !turning else { return }
            if let canvas = view.hitTest(point, with: nil) as? PageCanvas.Canvas,
               let hit = canvas.character(at: view.convert(point, to: canvas)) {
                return parent.tapText(hit.paragraph, hit.offset)
            }
            if let tapAway = parent.tapAway { return tapAway() }
            let x = point.x / view.bounds.width
            if x < 0.3 { step(-1) } else if x > 0.7 { step(1) } else { parent.tapMiddle() }
        }
        @objc func swipe(_ recognizer: UISwipeGestureRecognizer) { step(recognizer.direction == .left ? 1 : -1) }
    }
}

/// The chapter set as one attributed string and split into page-sized text containers.
@MainActor private final class Pagination {
    let layoutManager = NSLayoutManager()
    private let storage: NSTextStorage
    private(set) var containers: [NSTextContainer] = []
    /// Where each paragraph starts in the text.
    private let starts: [Int]
    /// Each paragraph's text, nil for scene breaks.
    private let texts: [NSRange?]
    /// The canvases drawing these pages, to redraw when the highlight moves.
    let canvases = NSHashTable<PageCanvas.Canvas>.weakObjects()
    let lang: String

    init(view: BookReaderView, layout: ReaderLayout, paper: Paper) {
        let size = layout.font_size
        let font = Self.serif(size)
        let lineHeight = size * layout.line_height
        let text = NSMutableAttributedString()
        func style(_ build: (NSMutableParagraphStyle) -> Void) -> NSParagraphStyle {
            let style = NSMutableParagraphStyle()
            build(style)
            return style
        }
        func append(_ string: String, _ attributes: [NSAttributedString.Key: Any]) {
            text.append(NSAttributedString(string: string, attributes: attributes))
        }
        append(view.byline + "\n", [.font: Self.serif(size * 0.65), .foregroundColor: paper.mutedInk,
                                    .paragraphStyle: style { $0.alignment = .right; $0.paragraphSpacingBefore = size * 2; $0.paragraphSpacing = view.chapter_title == nil ? size * 3 : size }])
        if let title = view.chapter_title {
            append(title + "\n", [.font: Self.serif(size * 1.7, weight: .semibold), .foregroundColor: paper.ink,
                                  .paragraphStyle: style { $0.alignment = .right; $0.lineHeightMultiple = 1.05; $0.paragraphSpacing = size * 3 }])
        }
        // Line height is set exactly, with the glyphs centred in the line as CSS does.
        let body: [NSAttributedString.Key: Any] = [
            .font: font, .foregroundColor: paper.ink,
            .baselineOffset: (lineHeight - font.lineHeight) / 2,
            NSAttributedString.Key(kCTLanguageAttributeName as String): view.lang,
        ]
        var starts: [Int] = []
        var texts: [NSRange?] = []
        for (index, paragraph) in view.paragraphs.enumerated() {
            starts.append(text.length)
            texts.append(paragraph.text.map { NSRange(location: text.length, length: ($0 as NSString).length) })
            let last = index == view.paragraphs.count - 1
            if let string = paragraph.text {
                let indented = index > 0 && view.paragraphs[index - 1].text != nil
                append(string + (last ? "" : "\n"), body.merging([.paragraphStyle: style {
                    $0.minimumLineHeight = lineHeight; $0.maximumLineHeight = lineHeight
                    $0.alignment = layout.justified ? .justified : .natural
                    $0.hyphenationFactor = 1
                    $0.firstLineHeadIndent = indented ? size * 1.5 : 0
                }]) { $1 })
            } else {
                append("⁂" + (last ? "" : "\n"), body.merging([.foregroundColor: paper.mutedInk, .paragraphStyle: style {
                    $0.minimumLineHeight = lineHeight; $0.maximumLineHeight = lineHeight
                    $0.alignment = .center
                    $0.paragraphSpacingBefore = lineHeight / 2; $0.paragraphSpacing = lineHeight / 2
                }]) { $1 })
            }
        }
        if let copyright = view.copyright, view.next == nil {
            append("\n" + copyright, [.font: Self.serif(size * 0.6), .foregroundColor: paper.mutedInk,
                                      .paragraphStyle: style { $0.alignment = .center; $0.paragraphSpacingBefore = 40 }])
        }
        self.starts = starts
        self.texts = texts
        lang = view.lang
        storage = NSTextStorage(attributedString: text)
        storage.addLayoutManager(layoutManager)
        repeat {
            let container = NSTextContainer(size: CGSize(width: layout.page_width, height: layout.page_height))
            container.lineFragmentPadding = 0
            layoutManager.addTextContainer(container)
            containers.append(container)
        } while NSMaxRange(layoutManager.glyphRange(for: containers.last!)) < layoutManager.numberOfGlyphs
    }

    func spreads(_ layout: ReaderLayout) -> Int {
        (containers.count + Int(layout.columns) - 1) / Int(layout.columns)
    }
    func page(ofParagraph paragraph: Int) -> Int {
        guard starts.indices.contains(paragraph) else { return 0 }
        let glyph = layoutManager.glyphIndexForCharacter(at: starts[paragraph])
        return layoutManager.textContainer(forGlyphAt: glyph, effectiveRange: nil).flatMap { containers.firstIndex(of: $0) } ?? 0
    }
    /// The first paragraph with any text on this page: the reading position.
    func firstParagraph(onPage page: Int) -> Int {
        guard containers.indices.contains(page) else { return starts.count - 1 }
        let start = layoutManager.characterRange(forGlyphRange: layoutManager.glyphRange(for: containers[page]), actualGlyphRange: nil).location
        return max(0, (starts.lastIndex { $0 <= start }) ?? 0)
    }

    /// The paragraph and UTF-16 offset of the character drawn under a point on a page.
    func character(at point: CGPoint, onPage page: Int) -> (paragraph: Int, offset: Int)? {
        guard containers.indices.contains(page) else { return nil }
        let container = containers[page]
        let glyph = layoutManager.glyphIndex(for: point, in: container)
        guard layoutManager.boundingRect(forGlyphRange: NSRange(location: glyph, length: 1), in: container).contains(point) else { return nil }
        let character = layoutManager.characterIndexForGlyph(at: glyph)
        guard let paragraph = texts.firstIndex(where: { $0.map { NSLocationInRange(character, $0) } ?? false }) else { return nil }
        return (paragraph, character - starts[paragraph])
    }

    func highlight(_ highlight: (paragraph: Int, range: BookSentenceHighlight)?, color: UIColor) {
        // A drawing-only attribute: pages keep their line breaks.
        storage.beginEditing()
        storage.removeAttribute(.backgroundColor, range: NSRange(location: 0, length: storage.length))
        if let highlight, starts.indices.contains(highlight.paragraph) {
            let start = starts[highlight.paragraph] + Int(highlight.range.start)
            storage.addAttribute(.backgroundColor, value: color,
                                 range: NSRange(location: start, length: Int(highlight.range.end - highlight.range.start)))
        }
        storage.endEditing()
        canvases.allObjects.forEach { $0.setNeedsDisplay() }
    }

    /// A page's text, for VoiceOver, since pages are drawn as glyphs.
    func text(onPage page: Int) -> String {
        guard containers.indices.contains(page) else { return "" }
        let range = layoutManager.characterRange(forGlyphRange: layoutManager.glyphRange(for: containers[page]), actualGlyphRange: nil)
        return storage.attributedSubstring(from: range).string
    }

    private static func serif(_ size: CGFloat, weight: UIFont.Weight = .regular) -> UIFont {
        let font = UIFont.systemFont(ofSize: size, weight: weight)
        return font.fontDescriptor.withDesign(.serif).map { UIFont(descriptor: $0, size: size) } ?? font
    }
}

#if DEBUG
private extension Notification.Name { static let debugReaderTap = Notification.Name("yap-debug-reader-tap") }
#endif

/// Draws one page's glyphs from the shared layout manager.
private struct PageCanvas: UIViewRepresentable {
    let pagination: Pagination
    let page: Int
    func makeUIView(context: Context) -> Canvas { Canvas() }
    func updateUIView(_ canvas: Canvas, context: Context) {
        canvas.pagination = pagination
        canvas.page = page
        pagination.canvases.add(canvas)
        canvas.accessibilityLabel = pagination.text(onPage: page)
        canvas.accessibilityLanguage = pagination.lang
        canvas.setNeedsDisplay()
    }
    final class Canvas: UIView {
        var pagination: Pagination?
        var page = 0
        override init(frame: CGRect) {
            super.init(frame: frame)
            isOpaque = false
            backgroundColor = .clear
            isAccessibilityElement = true
            accessibilityTraits = .staticText
        }
        required init?(coder: NSCoder) { fatalError() }
        func character(at point: CGPoint) -> (paragraph: Int, offset: Int)? { pagination?.character(at: point, onPage: page) }
        override func draw(_ rect: CGRect) {
            guard let pagination, pagination.containers.indices.contains(page) else { return }
            let glyphs = pagination.layoutManager.glyphRange(for: pagination.containers[page])
            pagination.layoutManager.drawBackground(forGlyphRange: glyphs, at: .zero)
            pagination.layoutManager.drawGlyphs(forGlyphRange: glyphs, at: .zero)
        }
    }
}

/// The tapped sentence, its translation and the tapped word, then a short chat about it.
/// Pages stay tappable behind the sheet, so tapping another word moves the tutor there.
private struct SentenceTutor: View {
    let view: BookSentenceView
    /// The book's BCP 47 language, for the target-language sentence.
    let lang: String
    let paper: Paper
    let send: (BookSentenceEvent) -> Void
    var body: some View {
        VStack(spacing: 0) {
            ScrollViewReader { scroll in
                ScrollView {
                    VStack(alignment: .leading, spacing: 20) {
                        if let sentence = view.sentence {
                            VStack(alignment: .leading, spacing: 8) {
                                Text(sentence).font(.system(.title3, design: .serif))
                                    .typesettingLanguage(Locale.Language(identifier: lang)).textSelection(.enabled)
                                if let translation = view.translation { Text(translation).foregroundStyle(paper.muted) }
                            }
                        }
                        if let word = view.word, let explanation = view.word_explanation {
                            VStack(alignment: .leading, spacing: 4) {
                                Text(word).font(.system(.headline, design: .serif)).typesettingLanguage(Locale.Language(identifier: lang))
                                Text(explanation)
                            }.padding(16).frame(maxWidth: .infinity, alignment: .leading)
                                .background(paper.text.opacity(0.05), in: RoundedRectangle(cornerRadius: 16, style: .continuous))
                        }
                        ForEach(Array(view.messages.enumerated()), id: \.offset) { _, message in
                            if message.from_learner {
                                Text(message.text).padding(.horizontal, 14).padding(.vertical, 10)
                                    .background(paper.text.opacity(0.08), in: RoundedRectangle(cornerRadius: 18, style: .continuous))
                                    .frame(maxWidth: .infinity, alignment: .trailing).padding(.leading, 48)
                            } else {
                                Text(message.text).textSelection(.enabled).frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }
                        if let status = view.status {
                            HStack(spacing: 10) {
                                if !view.status_is_error { ProgressView().tint(paper.muted) }
                                Text(status).foregroundStyle(paper.muted)
                                if let retry = view.retry_label {
                                    Button(retry) { send(.Retry) }.fontWeight(.semibold)
                                }
                            }
                        }
                        Color.clear.frame(height: 0).id("end")
                    }.padding(.horizontal, 24).padding(.top, 28).padding(.bottom, 12)
                }
                .onChange(of: view.messages.count) { _, _ in withAnimation { scroll.scrollTo("end", anchor: .bottom) } }
            }
            HStack(spacing: 8) {
                TextField(view.placeholder, text: Binding(get: { view.draft }, set: { send(.DraftChanged(text: $0)) }))
                    .submitLabel(.send).onSubmit { send(.Ask) }
                Button { send(.Ask) } label: { Image(systemName: "arrow.up.circle.fill").font(.title2) }
                    .disabled(!view.can_send).opacity(view.can_send ? 1 : 0.3).accessibilityLabel(view.send_label)
            }.padding(.leading, 18).padding(.trailing, 6).frame(minHeight: 48)
                .background(paper.text.opacity(0.06), in: Capsule())
                .padding(.horizontal, 16).padding(.bottom, 12)
        }
        .buttonStyle(.plain)
        .foregroundStyle(paper.text)
        .presentationDetents([.medium, .large])
        .presentationBackgroundInteraction(.enabled(upThrough: .medium))
        .presentationBackground(paper.background)
        .presentationDragIndicator(.visible)
    }
}

private struct BookContents: View {
    let view: BookReaderView
    let copy: BooksCopy
    let paper: Paper
    let select: (UInt64) -> Void
    var body: some View {
        NavigationStack {
            List {
                ForEach(Array(view.parts.enumerated()), id: \.offset) { _, part in
                    Section {
                        ForEach(part.chapters, id: \.position) { chapter in
                            Button { select(chapter.position) } label: {
                                Text(chapter.title).font(.system(.body, design: .serif))
                                    .fontWeight(chapter.position == view.position.chapter ? .semibold : .regular)
                                    .foregroundStyle(chapter.enabled ? paper.text : paper.muted)
                            }.disabled(!chapter.enabled).listRowBackground(paper.background)
                        }
                    } header: {
                        if let title = part.title { Text(title) }
                    }
                }
            }
            .scrollContentBackground(.hidden)
            .background(paper.background)
            .foregroundStyle(paper.text)
            .readableListMargins()
            .navigationTitle(copy.contents)
            .navigationBarTitleDisplayMode(.inline)
            .safeAreaInset(edge: .top) {
                Text(view.author).font(.footnote).foregroundStyle(paper.muted).padding(.bottom, 4)
            }
        }
    }
}
