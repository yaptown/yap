import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
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
  const card = (
    <Card className="flex-row items-center gap-4 p-4 transition-colors group-hover:bg-muted/40">
      <BookCover book={book} />
      <div className="flex min-w-0 flex-1 flex-col gap-1">
        <h2 className="font-serif text-xl font-semibold leading-tight">
          {book.title}
        </h2>
        <p className="text-muted-foreground">{book.author}</p>
        <p className="text-sm text-muted-foreground">{book.availability}</p>
      </div>
      {book.enabled && (
        <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" aria-hidden />
      )}
    </Card>
  );
  if (!book.enabled) return <div className="opacity-60">{card}</div>;
  return (
    <Link
      to={`/books/${encodeURIComponent(book.slug)}`}
      className="group rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      {card}
    </Link>
  );
}

function Catalog({ language }: { language: Language }) {
  const { userInfo } = useOutletContext<AppContextType>();
  const navigate = useNavigate();
  const [view, setView] = useState<BooksScreenView>();
  const [failed, setFailed] = useState(false);
  useEffect(() => {
    let cancelled = false;
    load_books(language)
      .then((v) => !cancelled && setView(v))
      .catch(() => !cancelled && setFailed(true));
    return () => {
      cancelled = true;
    };
  }, [language]);
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
              className="rounded-lg px-3 py-2 text-left font-serif hover:bg-muted/60 aria-[current=page]:bg-muted aria-[current=page]:font-semibold disabled:text-muted-foreground disabled:hover:bg-transparent"
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

/**
 * Lays a chapter out as pages with CSS columns and turns them by sliding the column
 * strip. The geometry comes from Rust's `reader_layout`, shared with iOS.
 */
function ChapterPages({
  view,
  layout,
  opening,
  paused,
  remember,
  turnPast,
  turned,
  tapMiddle,
}: {
  view: BookReaderView;
  layout: ReaderLayout;
  opening: Opening;
  paused: boolean;
  remember: (paragraph: number) => void;
  /** Turning beyond the first or last page moves to the neighbouring chapter. */
  turnPast: (direction: 1 | -1) => void;
  /** Called on every page turn the reader makes. */
  turned: () => void;
  tapMiddle: () => void;
}) {
  const flow = useRef<HTMLDivElement>(null);
  const [page, setPage] = useState(0);
  const [pages, setPages] = useState(1);
  const [animate, setAnimate] = useState(false);
  // The paragraph to keep in view when the layout changes (resize, text size).
  const anchor = useRef<Opening>(opening);
  // Set when the layout picked the page, so the page it lands on doesn't move the anchor:
  // a paragraph straddling two pages would otherwise pull every relayout back a page.
  const placed = useRef(false);

  const width = layout.page_width * layout.columns + layout.gap * (layout.columns - 1);
  const stride = width + layout.gap;
  const blocks = () => [...flow.current!.querySelectorAll<HTMLElement>("[data-paragraph]")];
  const pageOf = (rect: DOMRect) =>
    Math.floor((rect.left - flow.current!.getBoundingClientRect().left + 1) / stride);

  useLayoutEffect(() => {
    const place = () => {
      const total = Math.max(1, Math.round((flow.current!.scrollWidth + layout.gap) / stride));
      const target = anchor.current;
      const block = target === "end" ? undefined : blocks()[target];
      setAnimate(false);
      placed.current = true;
      setPages(total);
      setPage(block ? Math.min(pageOf(block.getClientRects()[0]), total - 1) : target === "end" ? total - 1 : 0);
    };
    place();
    document.fonts.ready.then(place);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- relayout only when the geometry or text changes
  }, [layout, view]);

  // The first paragraph showing on this page is the reading position.
  useEffect(() => {
    const first = blocks().findIndex((block) =>
      [...block.getClientRects()].some((rect) => pageOf(rect) >= page),
    );
    if (first === -1) return;
    if (!placed.current) anchor.current = first;
    placed.current = false;
    remember(first);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- recorded per page turn
  }, [page, pages]);

  const turn = (direction: 1 | -1) => {
    turned();
    const next = page + direction;
    if (next < 0 || next >= pages) return turnPast(direction);
    placed.current = false;
    setAnimate(true);
    setPage(next);
  };
  const turnRef = useRef(turn);
  turnRef.current = turn;
  useEffect(() => {
    if (paused) return;
    const onKey = (event: KeyboardEvent) => {
      if (["ArrowRight", "PageDown", " "].includes(event.key)) turnRef.current(1);
      else if (["ArrowLeft", "PageUp"].includes(event.key)) turnRef.current(-1);
      else return;
      event.preventDefault();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [paused]);

  // Tap the outer thirds to turn, the middle to show the controls; or swipe.
  const touch = useRef({ x: 0, swiped: false });
  return (
    <div
      className="relative flex-1 select-none overflow-hidden"
      onTouchStart={(e) => (touch.current = { x: e.touches[0].clientX, swiped: false })}
      onTouchEnd={(e) => {
        const dx = e.changedTouches[0].clientX - touch.current.x;
        if (Math.abs(dx) < 40) return;
        touch.current.swiped = true;
        turn(dx < 0 ? 1 : -1);
      }}
      onClick={(e) => {
        if (touch.current.swiped) return;
        const x = e.clientX / window.innerWidth;
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
            transition: animate ? "transform 300ms ease" : undefined,
          }}
        >
          <header className="mb-10 break-inside-avoid text-left">
            <p className="text-[0.7em] text-neutral-500">{view.byline}</p>
            {view.chapter_title && (
              <h1 className="mt-3 font-serif text-[1.9em] font-semibold leading-tight">{view.chapter_title}</h1>
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
                {block.text}
              </p>
            ),
          )}
          {view.copyright && view.next == null && (
            <p className="mt-10 whitespace-pre-line text-center text-[0.6em] text-neutral-500">{view.copyright}</p>
          )}
        </div>
      </div>
      <p className="absolute inset-x-0 bottom-5 text-center text-xs tabular-nums text-neutral-400">
        {page + 1} / {pages}
      </p>
    </div>
  );
}

/** The reader's controls, shown when the book opens and on tapping the page's middle. */
function ReaderBar({
  shown,
  title,
  copy,
  layout,
  setTextSize,
  onContents,
}: {
  shown: boolean;
  title?: string;
  copy: ReturnType<typeof books_copy>;
  layout?: ReaderLayout;
  setTextSize: (step: number) => void;
  onContents?: () => void;
}) {
  const navigate = useNavigate();
  const smaller = layout?.smaller;
  const larger = layout?.larger;
  return (
    <div
      className={`absolute inset-x-0 top-0 z-10 border-b border-neutral-200 bg-white transition-transform duration-300 dark:border-neutral-800 dark:bg-black ${shown ? "" : "-translate-y-full"}`}
    >
      <div className="mx-auto flex h-14 max-w-5xl items-center gap-1 px-3">
        <Button variant="ghost" size="icon" onClick={() => navigate("/books")} aria-label={copy.title}>
          <ArrowLeft aria-hidden />
        </Button>
        <p className="min-w-0 flex-1 truncate text-center text-sm font-medium">{title}</p>
        {layout && (
          <>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => smaller != null && setTextSize(smaller)}
              disabled={smaller == null}
              aria-label={copy.text_smaller}
            >
              <span aria-hidden className="font-serif text-sm">
                A
              </span>
            </Button>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => larger != null && setTextSize(larger)}
              disabled={larger == null}
              aria-label={copy.text_larger}
            >
              <span aria-hidden className="font-serif text-xl">
                A
              </span>
            </Button>
          </>
        )}
        <Button variant="ghost" size="icon" onClick={onContents} disabled={!onContents} aria-label={copy.contents}>
          <List aria-hidden />
        </Button>
      </div>
    </div>
  );
}

function Reader({ language, slug }: { language: Language; slug: string }) {
  const copy = books_copy();
  const [view, setView] = useState<BookReaderView>();
  const [failed, setFailed] = useState(false);
  const [contentsOpen, setContentsOpen] = useState(false);
  const [barShown, setBarShown] = useState(true);
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
  const [request, setRequest] = useState<{ chapter: number; opening: Opening; revision: number }>();
  const [opened, setOpened] = useState<{ opening: Opening; revision: number }>();
  useEffect(() => {
    let cancelled = false;
    load_book(language, slug, request === undefined ? undefined : { chapter: request.chapter, paragraph: 0 })
      .then((v) => {
        if (cancelled) return;
        setView(v);
        setOpened({ opening: request?.opening ?? v.position.paragraph, revision: request?.revision ?? 0 });
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
  const open = (chapter: number, opening: Opening) => {
    setContentsOpen(false);
    setRequest((previous) => ({ chapter, opening, revision: (previous?.revision ?? 0) + 1 }));
  };

  return (
    <div
      ref={screen}
      className="fixed inset-0 z-10 flex flex-col bg-white text-neutral-900 dark:bg-black dark:text-neutral-200"
      data-page="book-reader"
    >
      <ReaderBar
        shown={barShown || !view}
        title={view?.card.title}
        copy={copy}
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
          paused={contentsOpen}
          remember={remember(view.position.chapter)}
          turned={() => setBarShown(false)}
          turnPast={(direction) => {
            const chapter = direction === 1 ? view.next : view.previous;
            if (chapter != null) open(chapter, direction === 1 ? 0 : "end");
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
          <DialogContent className="max-h-[85dvh] overflow-y-auto">
            <DialogHeader>
              <DialogTitle className="font-serif text-2xl">{view.card.title}</DialogTitle>
              <DialogDescription>
                {view.card.author} · {view.card.availability}
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
    <Reader key={`${language}-${slug}`} language={language} slug={slug} />
  ) : (
    <Catalog key={language} language={language} />
  );
}
