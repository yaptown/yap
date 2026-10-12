import CoreText
import SwiftUI

struct BooksScreen: View {
    let language: Language
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
                        }.disabled(!book.enabled).buttonStyle(.plain)
                    }
                } else if !failed { ProgressView(copy.loading).frame(maxWidth: .infinity) }
            }.padding(20).frame(maxWidth: .readableColumn).frame(maxWidth: .infinity)
        }.navigationTitle(copy.title)
            .task {
                do { catalog = try await load_books(language: language) } catch { failed = true }
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
        HStack(spacing: 16) {
            BookCover(book: book)
            VStack(alignment: .leading, spacing: 4) {
                Text(book.title).font(.system(.title3, design: .serif).weight(.semibold)).foregroundStyle(Color.yapText)
                Text(book.author).foregroundStyle(Color.yapMuted)
                Text(book.availability).font(.subheadline).foregroundStyle(Color.yapMuted)
            }
            Spacer(minLength: 0)
            if book.enabled { Image(systemName: "chevron.right").font(.footnote).foregroundStyle(Color.yapMuted) }
        }
        .padding(16).frame(maxWidth: .infinity, alignment: .leading).quietSurface()
        .opacity(book.enabled ? 1 : 0.6)
    }
}

/// Where to open a chapter: a paragraph, or its last page when paging backwards.
private enum Opening: Equatable { case paragraph(Int), end }

struct BookReaderScreen: View {
    let language: Language
    let slug: String
    @Environment(\.dismiss) private var dismiss
    @State private var view: BookReaderView?
    @State private var opening = Opening.paragraph(0)
    /// Bumped on every completed load, so reopening the current chapter starts it afresh.
    @State private var loads = 0
    @State private var failed = false
    @State private var requestID = 0
    @State private var showingContents = false
    @State private var barShown = true
    @AppStorage("yap-reader-text-size") private var textSize: Int?
    @State private var saveTask: Task<Void, Never>?
    private let copy = books_copy()
    var body: some View {
        GeometryReader { geometry in
            let layout = reader_layout(width: geometry.size.width, height: geometry.size.height, text_size: textSize.map(UInt32.init))
            ZStack(alignment: .top) {
                if let view {
                    ChapterPages(view: view, layout: layout, width: geometry.size.width, opening: opening, remember: remember,
                                 turned: { withAnimation { barShown = false } },
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
                    }.foregroundStyle(Paper.muted).frame(maxWidth: .infinity).padding(.top, 96)
                }
                if barShown || view == nil {
                    ReaderBar(title: view?.card.title, copy: copy, layout: view == nil ? nil : layout,
                              setTextSize: { textSize = Int($0) }, back: { dismiss() },
                              contents: view == nil ? nil : { showingContents = true })
                        .transition(.move(edge: .top))
                }
            }
        }
        .foregroundStyle(Paper.text)
        .background(Paper.background.ignoresSafeArea())
        .toolbar(.hidden, for: .navigationBar)
        .statusBarHidden()
        .persistentSystemOverlays(.hidden)
        .sheet(isPresented: $showingContents) {
            if let view {
                BookContents(view: view, copy: copy) { chapter in
                    showingContents = false
                    Task { await load(chapter: chapter, opening: .paragraph(0)) }
                }.presentationDetents([.medium, .large])
            }
        }
        .task { await load() }
        #if DEBUG
        .onAppear { DebugHarness.shared.activeScreen = .bookReader }
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeScreen == .bookReader else { return }
            let command = DebugHarness.shared.command
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
            view = result
            opening = chapter == nil ? .paragraph(Int(result.position.paragraph)) : requested
            loads += 1
            failed = false
        } catch { if request == requestID { failed = true } }
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

/// Books are read on plain paper: white in light mode, black in dark.
private enum Paper {
    static let background = Color(uiColor: ui(light: 1, dark: 0))
    static let text = Color(uiColor: ink)
    static let muted = Color(uiColor: mutedInk)
    static let faint = Color(uiColor: faintInk)
    static let rule = Color(uiColor: ui(light: 0.898, dark: 0.149))
    // Tailwind's neutral-900/-200, -500 and -400, as on web.
    static let ink = ui(light: 0.090, dark: 0.898)
    static let mutedInk = UIColor(white: 0.451, alpha: 1)
    static let faintInk = UIColor(white: 0.639, alpha: 1)
    private static func ui(light: CGFloat, dark: CGFloat) -> UIColor {
        UIColor { $0.userInterfaceStyle == .dark ? UIColor(white: dark, alpha: 1) : UIColor(white: light, alpha: 1) }
    }
}

/// The reader's controls, shown when the book opens and on tapping the page's middle.
private struct ReaderBar: View {
    let title: String?
    let copy: BooksCopy
    let layout: ReaderLayout?
    let setTextSize: (UInt32) -> Void
    let back: () -> Void
    let contents: (() -> Void)?
    var body: some View {
        HStack(spacing: 4) {
            button("chevron.left", copy.title, back)
            Text(title ?? "").font(.subheadline.weight(.medium)).lineLimit(1).frame(maxWidth: .infinity)
            if let layout {
                Button { layout.smaller.map(setTextSize) } label: { Text("A").font(.system(size: 14, design: .serif)).frame(width: 40, height: 40) }
                    .disabled(layout.smaller == nil).accessibilityLabel(copy.text_smaller)
                Button { layout.larger.map(setTextSize) } label: { Text("A").font(.system(size: 21, design: .serif)).frame(width: 40, height: 40) }
                    .disabled(layout.larger == nil).accessibilityLabel(copy.text_larger)
            }
            button("list.bullet", copy.contents, contents ?? {}).disabled(contents == nil)
        }
        .buttonStyle(.plain).padding(.horizontal, 8).frame(height: 56).frame(maxWidth: 1024)
        .frame(maxWidth: .infinity)
        .background(Paper.background.ignoresSafeArea(edges: .top))
        .overlay(alignment: .bottom) { Paper.rule.frame(height: 1) }
    }
    private func button(_ symbol: String, _ label: String, _ action: @escaping () -> Void) -> some View {
        Button(action: action) { Image(systemName: symbol).frame(width: 40, height: 40).contentShape(Rectangle()) }
            .accessibilityLabel(label)
    }
}

/// A chapter laid out into pages with TextKit, turned with a paged TabView. The geometry
/// comes from Rust's `reader_layout`, shared with web. Pages -1 and `spreads` are blank
/// sentinels: landing on one turns into the neighbouring chapter.
private struct ChapterPages: View {
    let view: BookReaderView
    let layout: ReaderLayout
    /// The screen's width, for telling taps on its outer thirds from the middle.
    let width: CGFloat
    let opening: Opening
    let remember: (Int) -> Void
    /// Called on every page turn the reader makes.
    let turned: () -> Void
    /// Turning beyond the first or last page; false at the start or end of the book.
    let turnPast: (Int) -> Bool
    let tapMiddle: () -> Void
    @Environment(\.colorScheme) private var colorScheme
    @State private var pagination: Pagination?
    @State private var spread = 0
    // The paragraph to keep in view when the layout changes (rotation, text size).
    @State private var anchor: Opening?
    // Set when the layout picked the spread, so the spread it lands on doesn't move the anchor:
    // a paragraph straddling two pages would otherwise pull every relayout back a page.
    @State private var placed = false
    var body: some View {
        let spreads = pagination?.spreads(layout) ?? 1
        TabView(selection: $spread) {
            ForEach(-1...spreads, id: \.self) { index in
                Group {
                    if let pagination, index >= 0, index < spreads {
                        HStack(alignment: .top, spacing: layout.gap) {
                            ForEach(0..<Int(layout.columns), id: \.self) { column in
                                PageCanvas(pagination: pagination, page: index * Int(layout.columns) + column, colorScheme: colorScheme)
                                    .frame(width: layout.page_width, height: layout.page_height)
                            }
                        }
                        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
                        .padding(.top, layout.top)
                    } else { Color.clear }
                }
                .contentShape(Rectangle())
                .onTapGesture { location in
                    let x = location.x / width
                    if x < 0.3 { turn(-1) } else if x > 0.7 { turn(1) } else { tapMiddle() }
                }
                .tag(index)
            }
        }
        .tabViewStyle(.page(indexDisplayMode: .never))
        .overlay(alignment: .bottom) {
            Text("\(min(max(spread, 0), spreads - 1) + 1) / \(spreads)").font(.caption.monospacedDigit())
                .foregroundStyle(Paper.faint).padding(.bottom, 20)
        }
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
            if command == "previous" { turn(-1) }
            if command.hasPrefix("page "), let page = Int(command.dropFirst(5)) { withAnimation { spread = page - 1 } }
            if command == "status" { DebugHarness.log("book chapter=\(view.position.chapter) page=\(spread + 1)/\(spreads) anchor=\(String(describing: anchor))") }
        }
        #endif
    }
    private func turn(_ direction: Int) { withAnimation { spread += direction } }
    private func place(_ layout: ReaderLayout) {
        let pagination = Pagination(view: view, layout: layout)
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

/// The chapter set as one attributed string and split into page-sized text containers.
@MainActor private final class Pagination {
    let layoutManager = NSLayoutManager()
    private let storage: NSTextStorage
    private(set) var containers: [NSTextContainer] = []
    /// Where each paragraph starts in the text.
    private let starts: [Int]
    let lang: String

    init(view: BookReaderView, layout: ReaderLayout) {
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
        append(view.byline + "\n", [.font: Self.serif(size * 0.7), .foregroundColor: Paper.mutedInk,
                                    .paragraphStyle: style { $0.paragraphSpacing = size * 0.4 }])
        if let title = view.chapter_title {
            append(title + "\n", [.font: Self.serif(size * 1.9, weight: .semibold), .foregroundColor: Paper.ink,
                                  .paragraphStyle: style { $0.lineHeightMultiple = 1.05; $0.paragraphSpacing = 40 }])
        }
        // Line height is set exactly, with the glyphs centred in the line as CSS does.
        let body: [NSAttributedString.Key: Any] = [
            .font: font, .foregroundColor: Paper.ink,
            .baselineOffset: (lineHeight - font.lineHeight) / 2,
            NSAttributedString.Key(kCTLanguageAttributeName as String): view.lang,
        ]
        var starts: [Int] = []
        for (index, paragraph) in view.paragraphs.enumerated() {
            starts.append(text.length)
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
                append("⁂" + (last ? "" : "\n"), body.merging([.foregroundColor: Paper.faintInk, .paragraphStyle: style {
                    $0.minimumLineHeight = lineHeight; $0.maximumLineHeight = lineHeight
                    $0.alignment = .center
                    $0.paragraphSpacingBefore = lineHeight / 2; $0.paragraphSpacing = lineHeight / 2
                }]) { $1 })
            }
        }
        if let copyright = view.copyright, view.next == nil {
            append("\n" + copyright, [.font: Self.serif(size * 0.6), .foregroundColor: Paper.mutedInk,
                                      .paragraphStyle: style { $0.alignment = .center; $0.paragraphSpacingBefore = 40 }])
        }
        self.starts = starts
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

/// Draws one page's glyphs from the shared layout manager.
private struct PageCanvas: UIViewRepresentable {
    let pagination: Pagination
    let page: Int
    /// Redraws when the appearance flips, so the dynamic colours resolve again.
    let colorScheme: ColorScheme
    func makeUIView(context: Context) -> Canvas { Canvas() }
    func updateUIView(_ canvas: Canvas, context: Context) {
        canvas.pagination = pagination
        canvas.page = page
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
        override func draw(_ rect: CGRect) {
            guard let pagination, pagination.containers.indices.contains(page) else { return }
            let glyphs = pagination.layoutManager.glyphRange(for: pagination.containers[page])
            pagination.layoutManager.drawGlyphs(forGlyphRange: glyphs, at: .zero)
        }
    }
}

private struct BookContents: View {
    let view: BookReaderView
    let copy: BooksCopy
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
                                    .foregroundStyle(chapter.enabled ? Color.yapText : Color.yapMuted)
                            }.disabled(!chapter.enabled)
                        }
                    } header: {
                        if let title = part.title { Text(title) }
                    }
                }
            }
            .readableListMargins()
            .navigationTitle(copy.contents)
            .navigationBarTitleDisplayMode(.inline)
            .safeAreaInset(edge: .top) {
                Text("\(view.card.author) · \(view.card.availability)").font(.footnote).foregroundStyle(Color.yapMuted).padding(.bottom, 4)
            }
        }
    }
}
