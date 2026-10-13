import { type CSSProperties, useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { flushSync } from "react-dom";
import { useTheme } from "@/components/theme-provider";
import { animatePageTurn, type PageSnapshot } from "./page-turn";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Link, useNavigate, useOutletContext, useParams } from "react-router-dom";
import { ArrowLeft, ChevronRight, List } from "lucide-react";
import {
  books_copy,
  load_books,
  load_book,
  save_book_position,
  type BookCardView,
  type BooksScreenView,
  type BookReaderView,
  type Language,
  type ReaderLayout,
  reader_layout,
  reader_themes,
  type ReaderTheme,
} from "../../../yap-frontend-rs/pkg";
import { useCourseDeck } from "@/review/course-study";
import type { AppContextType } from "@/app/context";
import { TopPageLayout } from "@/components/TopPageLayout";
import { TargetLanguageText } from "@/components/TargetLanguageText";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

/** The book's cover, or for a book without one a stable sea-toned tile from its slug. */
function BookCover({ book }: { book: BookCardView }) {
  const shape = "aspect-[2/3] w-20 shrink-0 rounded-md shadow-sm ring-1 ring-inset ring-black/10";
  if (book.cover) {
    return <img src={book.cover} alt="" className={`${shape} object-cover`} loading="lazy" />;
  }
  let hash = 0;
  for (const c of book.slug) hash = (hash * 31 + c.charCodeAt(0)) | 0;
  const hue = 190 + (Math.abs(hash) % 60);
  return (
    <div
      aria-hidden
      className={`${shape} flex items-end p-2`}
      style={{
        background: `linear-gradient(150deg, hsl(${hue} 55% 42%), hsl(${hue + 25} 60% 20%))`,
      }}
    >
      <span className="font-serif text-2xl leading-none text-white/85">{book.title.charAt(0)}</span>
    </div>
  );
}

function BookCard({ book }: { book: BookCardView }) {
  return (
    <Link
      to={`/books/${encodeURIComponent(book.slug)}`}
      className="group rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      <Card className="gap-4 p-4 transition-colors group-hover:bg-muted/40">
        <div className="flex items-center gap-4">
          <BookCover book={book} />
          <div className="flex min-w-0 flex-1 flex-col gap-1">
            <h2 className="font-serif text-xl font-semibold leading-tight">{book.title}</h2>
            <p className="text-muted-foreground">{book.author}</p>
          </div>
          <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" aria-hidden />
        </div>
        {book.teaser && <p className="text-sm leading-relaxed text-muted-foreground">{book.teaser}</p>}
      </Card>
    </Link>
  );
}

function Catalog({ language, nativeLanguage }: { language: Language; nativeLanguage: Language }) {
  const { userInfo } = useOutletContext<AppContextType>();
  const navigate = useNavigate();
  const [view, setView] = useState<BooksScreenView>();
  const [failed, setFailed] = useState(false);
  useEffect(() => {
    let cancelled = false;
    load_books(language, nativeLanguage)
      .then((v) => !cancelled && setView(v))
      .catch(() => !cancelled && setFailed(true));
    return () => {
      cancelled = true;
    };
  }, [language, nativeLanguage]);
  const copy = view?.copy ?? books_copy();
  return (
    <TopPageLayout
      userInfo={userInfo}
      headerProps={{ backButton: { label: copy.home, onBack: () => navigate("/home") } }}
    >
    <main className="mx-auto flex w-full max-w-2xl flex-col gap-4 p-5" data-page="books">
      <h1 className="text-3xl font-bold">{copy.title}</h1>
      {failed && (
        <p role="alert" className="text-muted-foreground">
          {copy.failed}
        </p>
      )}
      {!view && !failed && <p className="text-muted-foreground">{copy.loading}</p>}
      {view?.books.length === 0 && <p className="text-muted-foreground">{copy.empty}</p>}
      {view?.books.map((book) => <BookCard key={book.slug} book={book} />)}
    </main>
    </TopPageLayout>
  );
}

function TableOfContents({
  view,
  language,
  select,
}: {
  view: BookReaderView;
  language: Language;
  select: (chapter: number) => void;
}) {
  return (
    <nav className="flex flex-col gap-5">
      {view.parts.map((part, i) => (
        <section key={i} className="flex flex-col gap-1">
          {part.title && (
            <h3 className="px-3 pb-1 text-xs font-semibold uppercase tracking-[0.15em] text-muted-foreground">
              <TargetLanguageText language={language}>{part.title}</TargetLanguageText>
            </h3>
          )}
          {part.chapters.map((chapter) => (
            <button
              key={chapter.position}
              disabled={!chapter.enabled}
              aria-current={
                chapter.position === view.position.chapter ? "page" : undefined
              }
              className="rounded-lg px-3 py-2 text-left font-serif hover:bg-current/5 aria-[current=page]:bg-current/10 aria-[current=page]:font-semibold disabled:opacity-40 disabled:hover:bg-transparent"
              onClick={() => select(chapter.position)}
            >
              {chapter.enabled ? (
                <TargetLanguageText language={language}>{chapter.title}</TargetLanguageText>
              ) : (
                chapter.title
              )}
            </button>
          ))}
        </section>
      ))}
    </nav>
  );
}

const TEXT_SIZE_KEY = "yap-reader-text-size";

/** Where to open a chapter: the saved paragraph, or its last page when paging backwards. */
type Opening = number | "end";
type OpeningTurn = { before: PageSnapshot; direction: 1 | -1; layout: ReaderLayout };

/**
 * CSS columns paginate the live text; a temporary curved sheet animates each turn.
 * The geometry comes from Rust's `reader_layout`, shared with iOS.
 */
function ChapterPages({
  view,
  layout,
  opening,
  openingTurn,
  controlsShown,
  paused,
  language,
  remember,
  turnPast,
  turned,
  tapMiddle,
}: {
  view: BookReaderView;
  layout: ReaderLayout;
  opening: Opening;
  openingTurn?: OpeningTurn;
  controlsShown: boolean;
  paused: boolean;
  language: Language;
  remember: (paragraph: number) => void;
  /** Turning beyond the first or last page moves to the neighbouring chapter. */
  turnPast: (direction: 1 | -1, before: PageSnapshot) => void;
  /** Called on every page turn the reader makes. */
  turned: () => void;
  tapMiddle: () => void;
}) {
  const host = useRef<HTMLDivElement>(null);
  const flow = useRef<HTMLDivElement>(null);
  const cancelTurn = useRef<(() => void) | undefined>(undefined);
  const [page, setPage] = useState(0);
  const [pages, setPages] = useState(1);
  const [leafCount, setLeafCount] = useState(1);
  // The paragraph to keep in view when the layout changes (resize, text size).
  const anchor = useRef<Opening>(opening);
  const arrival = useRef(openingTurn);

  const width = layout.page_width * layout.columns + layout.gap * (layout.columns - 1);
  const stride = width + layout.gap;
  const blocks = useCallback(() => [...flow.current!.querySelectorAll<HTMLElement>("[data-paragraph]")], []);
  const pageOf = useCallback((rect: DOMRect) =>
    Math.floor((rect.left - flow.current!.getBoundingClientRect().left + 1) / stride), [stride]);
  const rememberRef = useRef(remember);
  useLayoutEffect(() => { rememberRef.current = remember; });

  useLayoutEffect(() => {
    let cancelled = false;
    const place = (animateArrival = false) => {
      if (cancelled || !flow.current) return;
      cancelTurn.current?.();
      cancelTurn.current = undefined;
      const rects = [...flow.current.lastElementChild!.getClientRects()];
      const right = Math.max(...rects.map((rect) => rect.right)) - flow.current.getBoundingClientRect().left;
      const leaves = Math.max(1, Math.ceil((right - 1) / (layout.page_width + layout.gap)));
      const total = Math.ceil(leaves / layout.columns);
      setLeafCount(leaves);
      const target = anchor.current;
      const block = target === "end" ? undefined : blocks()[target];
      setPages(total);
      const next = block ? Math.min(pageOf(block.getClientRects()[0]), total - 1) : target === "end" ? total - 1 : 0;
      setPage(next);
      const first = blocks().findIndex((block) => [...block.getClientRects()].some((rect) => pageOf(rect) >= next));
      if (first >= 0) rememberRef.current(first);
      const transition = arrival.current;
      if (animateArrival && transition) {
        arrival.current = undefined;
        if (JSON.stringify(transition.layout) === JSON.stringify(layout) && !matchMedia("(prefers-reduced-motion: reduce)").matches) {
          cancelTurn.current = animatePageTurn(host.current!, flow.current, layout, transition.before, { flow: flow.current, page: next }, transition.direction, () => { cancelTurn.current = undefined; });
        }
      }
    };
    place();
    document.fonts.ready.then(() => place(true));
    return () => { cancelled = true; cancelTurn.current?.(); cancelTurn.current = undefined; };
  }, [layout, view, blocks, pageOf]);

  const turn = (direction: 1 | -1) => {
    if (paused || cancelTurn.current) return;
    turned();
    const next = page + direction;
    if (next < 0 || next >= pages) return turnPast(direction, { flow: flow.current!.cloneNode(true) as HTMLElement, page });
    const commit = () => {
      const first = blocks().findIndex((block) => [...block.getClientRects()].some((rect) => pageOf(rect) >= next));
      if (first >= 0) { anchor.current = first; rememberRef.current(first); }
      setPage(next);
    };
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      commit();
    } else {
      cancelTurn.current = animatePageTurn(host.current!, flow.current!, layout, { flow: flow.current!, page }, { flow: flow.current!, page: next }, direction, () => {
        flushSync(commit);
        cancelTurn.current = undefined;
      });
    }
  };
  const turnRef = useRef(turn);
  useLayoutEffect(() => { turnRef.current = turn; });
  useEffect(() => {
    if (paused) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.target instanceof HTMLElement && event.target.closest("input, select, textarea, [contenteditable=true]")) return;
      if (event.key === " " && event.target instanceof HTMLElement && event.target.closest("button")) return;
      if (["ArrowRight", "PageDown", " "].includes(event.key)) turnRef.current(1);
      else if (["ArrowLeft", "PageUp"].includes(event.key)) turnRef.current(-1);
      else return;
      event.preventDefault();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [paused]);

  // Tap the outer thirds to turn, the middle to show the controls; or swipe.
  const touch = useRef({ x: 0, y: 0, swiped: false });
  return (
    <div
      ref={host}
      className="relative flex-1 select-none overflow-hidden"
      onTouchStart={(e) => (touch.current = { x: e.touches[0].clientX, y: e.touches[0].clientY, swiped: false })}
      onTouchEnd={(e) => {
        const dx = e.changedTouches[0].clientX - touch.current.x;
        const dy = e.changedTouches[0].clientY - touch.current.y;
        if (Math.abs(dx) < 40 || Math.abs(dx) < Math.abs(dy)) return;
        touch.current.swiped = true;
        turn(dx < 0 ? 1 : -1);
      }}
      onClick={(e) => {
        if (touch.current.swiped) { touch.current.swiped = false; return; }
        if (paused) return;
        const rect = e.currentTarget.getBoundingClientRect();
        const x = (e.clientX - rect.left) / rect.width;
        if (x < 0.3) turn(-1);
        else if (x > 0.7) turn(1);
        else tapMiddle();
      }}
    >
      <div
        className="absolute left-1/2 -translate-x-1/2 overflow-hidden"
        style={{ top: layout.top, width, height: layout.page_height }}
      >
        <div
          ref={flow}
          lang={view.lang}
          className={`font-serif hyphens-auto ${layout.justified ? "text-justify" : "text-left"}`}
          style={{
            width,
            height: layout.page_height,
            columnWidth: layout.page_width,
            columnGap: layout.gap,
            columnFill: "auto",
            fontSize: layout.font_size,
            lineHeight: layout.line_height,
            transform: `translateX(${-page * stride}px)`,

          }}
        >
          <header className="flex break-inside-avoid flex-col items-end gap-5 pb-[3em] pt-[2em] text-right">
            <p className="max-w-[85%] text-[0.65em] text-[var(--reader-muted)]">{view.byline}</p>
            {view.chapter_title && (
              <h1 className="max-w-[90%] font-serif text-[1.7em] font-semibold leading-tight"><TargetLanguageText language={language}>{view.chapter_title}</TargetLanguageText></h1>
            )}
          </header>
          {view.paragraphs.map((block, i) =>
            block.text == null ? (
              <div
                key={i}
                data-paragraph={i}
                role="separator"
                className="break-inside-avoid py-[0.5em] text-center text-neutral-400"
              >
                ⁂
              </div>
            ) : (
              <p
                key={i}
                data-paragraph={i}
                className={i > 0 && view.paragraphs[i - 1].text != null ? "indent-[1.5em]" : ""}
              >
                <TargetLanguageText language={language}>{block.text}</TargetLanguageText>
              </p>
            ),
          )}
          {view.copyright && view.next == null && (
            <p className="mt-10 whitespace-pre-line text-center text-[0.6em] text-neutral-500">{view.copyright}</p>
          )}
        </div>
      </div>
      <div className={`pointer-events-none absolute inset-x-0 top-7 flex text-center text-xs text-[var(--reader-muted)] transition-opacity ${controlsShown ? "opacity-0" : ""}`} aria-hidden>
        <p className="flex-1 truncate px-16"><TargetLanguageText language={language}>{view.title}</TargetLanguageText></p>
        {layout.columns === 2 && <p className="flex-1 truncate px-16"><TargetLanguageText language={language}>{view.chapter_title}</TargetLanguageText></p>}
      </div>
      <div className="pointer-events-none absolute inset-x-0 bottom-6 flex text-center text-xs tabular-nums text-[var(--reader-muted)]" aria-live="polite" aria-atomic="true">
        {Array.from({ length: layout.columns }, (_, column) => {
          const number = page * layout.columns + column + 1;
          return <p key={column} className="flex-1">{number <= leafCount ? number : ""}</p>;
        })}
      </div>
    </div>
  );
}

/** Floating controls leave the page geometry untouched when revealed. */
function ReaderBar({
  shown, copy, layout, setTextSize, onContents, theme, themes, setTheme, settingsOpen, setSettingsOpen,
}: {
  shown: boolean;
  copy: ReturnType<typeof books_copy>;
  layout?: ReaderLayout;
  setTextSize: (step: number) => void;
  onContents?: () => void;
  theme: ReaderTheme;
  themes: ReaderTheme[];
  setTheme: (id: string) => void;
  settingsOpen: boolean;
  setSettingsOpen: (open: boolean) => void;
}) {
  const navigate = useNavigate();
  const surface = { background: theme.background, color: theme.ink };
  const pill = "rounded-full border border-current/10 shadow-lg backdrop-blur-xl";
  return (
    <div inert={!shown} className={`pointer-events-none absolute inset-x-0 top-4 z-10 flex items-start justify-between gap-4 px-5 transition-opacity duration-200 motion-reduce:transition-none ${shown ? "" : "opacity-0"}`}>
      <Button className={`pointer-events-auto h-11 w-11 ${pill}`} style={surface} variant="ghost" size="icon" onClick={() => navigate("/books")} aria-label={copy.title}>
        <ArrowLeft aria-hidden />
      </Button>
      <div className={`pointer-events-auto flex items-center gap-1 p-1 ${pill}`} style={surface}>
        <Button variant="ghost" className="rounded-full hover:bg-current/5 hover:text-inherit" size="icon" onClick={onContents} disabled={!onContents} aria-label={copy.contents}><List aria-hidden /></Button>
        <Popover open={settingsOpen} onOpenChange={setSettingsOpen}>
          <PopoverTrigger asChild>
            <Button variant="ghost" className="rounded-full hover:bg-current/5 hover:text-inherit" size="icon" disabled={!layout} aria-label={copy.appearance}><span className="text-lg">Aa</span></Button>
          </PopoverTrigger>
          <PopoverContent align="end" sideOffset={12} className="flex w-80 flex-col gap-6 rounded-3xl border-current/10 p-5 shadow-xl" style={surface}>
            <h2 className="text-sm font-semibold">{copy.appearance}</h2>
            <div className="flex items-center justify-between gap-4 rounded-full bg-current/5 p-1">
              <Button variant="ghost" className="h-11 flex-1 rounded-full hover:bg-current/5 hover:text-inherit font-serif text-lg" disabled={layout?.smaller == null} onClick={() => layout?.smaller != null && setTextSize(layout.smaller)} aria-label={copy.text_smaller}>A</Button>
              <span className="text-xs tabular-nums" style={{ color: theme.muted }}>{layout?.font_size}</span>
              <Button variant="ghost" className="h-11 flex-1 rounded-full hover:bg-current/5 hover:text-inherit font-serif text-3xl" disabled={layout?.larger == null} onClick={() => layout?.larger != null && setTextSize(layout.larger)} aria-label={copy.text_larger}>A</Button>
            </div>
            <div className="grid grid-cols-2 gap-3">
              {themes.map((option) => <button key={option.id} aria-pressed={theme.id === option.id} onClick={() => setTheme(option.id)} style={{ background: option.background, color: option.ink }} className="flex flex-col items-center gap-1 rounded-2xl border border-current/15 px-2 py-4 outline-offset-4 aria-pressed:ring-2 aria-pressed:ring-current">
                <span className="font-serif text-3xl" aria-hidden>Aa</span><span className="text-xs">{option.label}</span>
              </button>)}
            </div>
          </PopoverContent>
        </Popover>
      </div>
    </div>
  );
}

export function BookReader({ language, slug }: { language: Language; slug: string }) {
  const copy = books_copy();
  const [view, setView] = useState<BookReaderView>();
  const [failed, setFailed] = useState(false);
  const [contentsOpen, setContentsOpen] = useState(false);
  const [barShown, setBarShown] = useState(true);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const { theme: appTheme } = useTheme();
  const [systemDark, setSystemDark] = useState(() => matchMedia("(prefers-color-scheme: dark)").matches);
  useEffect(() => {
    const media = matchMedia("(prefers-color-scheme: dark)");
    const changed = () => setSystemDark(media.matches);
    media.addEventListener("change", changed);
    return () => media.removeEventListener("change", changed);
  }, []);
  const themes = reader_themes(appTheme === "system" ? systemDark : appTheme !== "light");
  const [themeId, setThemeId] = useState(() => localStorage.getItem("yap-reader-theme") ?? "system");
  const theme = themes.find((option) => option.id === themeId) ?? themes[0];
  const saveTheme = (id: string) => { localStorage.setItem("yap-reader-theme", id); setThemeId(id); };
  const [textSize, setTextSize] = useState(() => {
    const saved = localStorage.getItem(TEXT_SIZE_KEY);
    return saved === null ? undefined : Number(saved);
  });
  const saveTextSize = (step: number) => {
    localStorage.setItem(TEXT_SIZE_KEY, String(step));
    setTextSize(step);
  };
  // Books are read on plain paper, filling the screen; a hidden page measures 0×0.
  const screen = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState<{ width: number; height: number }>();
  useEffect(() => {
    const observer = new ResizeObserver(([{ contentRect }]) => {
      if (contentRect.width > 0) setSize({ width: contentRect.width, height: contentRect.height });
    });
    observer.observe(screen.current!);
    return () => observer.disconnect();
  }, []);
  const layout = useMemo(
    () => (size ? reader_layout(size.width, size.height, textSize) : undefined),
    [size, textSize],
  );
  // undefined opens at the saved position; otherwise a chapter's start or end.
  const [request, setRequest] = useState<{ chapter: number; opening: Opening; revision: number; turn?: OpeningTurn }>();
  const [opened, setOpened] = useState<{ opening: Opening; revision: number; turn?: OpeningTurn }>();
  useEffect(() => {
    let cancelled = false;
    load_book(language, slug, request === undefined ? undefined : { chapter: request.chapter, paragraph: 0 })
      .then((v) => {
        if (cancelled) return;
        setView(v);
        setOpened({ opening: request?.opening ?? v.position.paragraph, revision: request?.revision ?? 0, turn: request?.turn });
      })
      .catch(() => !cancelled && setFailed(true));
    return () => {
      cancelled = true;
    };
  }, [language, slug, request]);
  // Serialize writes so an older anchor cannot overwrite a newer one; a failed write
  // mustn't stop later ones.
  const saves = useRef(Promise.resolve());
  const remember = (chapter: number) => (paragraph: number) => {
    saves.current = saves.current
      .then(() => save_book_position(language, slug, { chapter, paragraph }))
      .catch(() => {});
  };
  const open = (chapter: number, opening: Opening, turn?: OpeningTurn) => {
    setContentsOpen(false);
    setRequest((previous) => ({ chapter, opening, revision: (previous?.revision ?? 0) + 1, turn }));
  };

  return (
    <div
      ref={screen}
      className="fixed inset-0 z-10 flex flex-col bg-[var(--reader-paper)] text-[var(--reader-ink)]"
      style={{ "--reader-paper": theme.background, "--reader-ink": theme.ink, "--reader-muted": theme.muted } as CSSProperties}
      data-page="book-reader"
    >
      <ReaderBar
        shown={barShown || !view}
        copy={copy}
        theme={theme}
        themes={themes}
        setTheme={saveTheme}
        settingsOpen={settingsOpen}
        setSettingsOpen={setSettingsOpen}
        layout={view && layout}
        setTextSize={saveTextSize}
        onContents={view && (() => setContentsOpen(true))}
      />
      {view && opened && layout ? (
        <ChapterPages
          key={`${view.position.chapter}-${opened.revision}`}
          view={view}
          layout={layout}
          opening={opened.opening}
          openingTurn={opened.turn}
          controlsShown={barShown}
          paused={contentsOpen || settingsOpen}
          language={language}
          remember={remember(view.position.chapter)}
          turned={() => setBarShown(false)}
          turnPast={(direction, before) => {
            const chapter = direction === 1 ? view.next : view.previous;
            if (chapter != null) open(chapter, direction === 1 ? 0 : "end", { before, direction, layout });
          }}
          tapMiddle={() => setBarShown((shown) => !shown)}
        />
      ) : (
        <p className="pt-24 text-center text-neutral-500">
          {failed ? <span role="alert">{copy.failed}</span> : copy.loading}
        </p>
      )}
      {view && (
        <Dialog open={contentsOpen} onOpenChange={setContentsOpen}>
          <DialogContent className="max-h-[85dvh] overflow-y-auto rounded-3xl border-current/15" style={{ background: theme.background, color: theme.ink }}>
            <DialogHeader>
              <DialogTitle className="font-serif text-2xl"><TargetLanguageText language={language}>{view.title}</TargetLanguageText></DialogTitle>
              <DialogDescription style={{ color: theme.muted }}>
                {view.author}
              </DialogDescription>
            </DialogHeader>
            <TableOfContents view={view} language={language} select={(chapter) => open(chapter, 0)} />
          </DialogContent>
        </Dialog>
      )}
    </div>
  );
}

export function BooksPage() {
  const deck = useCourseDeck();
  const navigate = useNavigate();
  const { slug } = useParams();
  const phase = deck.view.phase;
  useEffect(() => {
    if (phase.type === "NoLanguageSelected") navigate("/", { replace: true });
  }, [phase.type, navigate]);
  // Books only need the course language, so they don't wait for the deck to load.
  if (!deck.course) return null;
  const language = deck.course.targetLanguage;
  return slug ? (
    <BookReader key={`${language}-${slug}`} language={language} slug={slug} />
  ) : (
    <Catalog key={language} language={language} nativeLanguage={deck.course.nativeLanguage} />
  );
}
