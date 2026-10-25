import { type CSSProperties, useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { flushSync } from "react-dom";
import { useTheme } from "@/components/theme-provider";
import { startPageTurn, type PageSnapshot, type PageTurn } from "./page-turn";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Link, useNavigate, useOutletContext, useParams } from "react-router-dom";
import { ArrowLeft, ArrowUp, ChevronRight, List, LoaderCircle, X } from "lucide-react";
import {
  book_sentence_start,
  book_sentence_transition,
  book_sentence_view,
  books_copy,
  chat_book_sentence,
  explain_book_sentence,
  load_books,
  load_book,
  save_book_position,
  type BookCardView,
  type BookSentenceEvent,
  type BookSentenceHighlight,
  type BookSentenceState,
  type BookSentenceStep,
  type BookSentenceView,
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

/** The paragraph and UTF-16 offset of the character drawn under a point, if any. */
function textAt(x: number, y: number): { paragraph: number; offset: number } | undefined {
  const caret = document.caretPositionFromPoint?.(x, y);
  const legacy = caret ? undefined : document.caretRangeFromPoint?.(x, y);
  const node = caret?.offsetNode ?? legacy?.startContainer;
  const offset = caret?.offset ?? legacy?.startOffset ?? 0;
  const paragraph = node?.parentElement?.closest<HTMLElement>("p[data-paragraph]");
  if (!(node instanceof Text) || !paragraph) return;
  // The caret falls between characters: take whichever neighbour is under the point.
  for (const at of [offset, offset - 1]) {
    if (at < 0 || at >= node.length) continue;
    const range = document.createRange();
    range.setStart(node, at);
    range.setEnd(node, at + 1);
    if ([...range.getClientRects()].some((r) => x >= r.left && x <= r.right && y >= r.top && y <= r.bottom)) {
      range.setStart(paragraph, 0);
      return { paragraph: Number(paragraph.dataset.paragraph), offset: range.toString().length - 1 };
    }
  }
}

/** A turn under way: the sheet, if one is drawn, and what finishing it does. */
type Turning = { sheet?: PageTurn; commit: () => void };
/** A finger on the page. Velocity is smoothed, in px/ms. */
type Swipe = { x: number; y: number; direction: 1 | -1; turning?: Turning; lastX: number; lastTime: number; velocity: number };

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
  highlight,
  tapText,
  tapAway,
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
  /** The tutor's sentence, as a UTF-16 range within its paragraph. */
  highlight?: { paragraph: number; range: BookSentenceHighlight };
  /** A tap on a paragraph's text, at a UTF-16 offset. */
  tapText: (paragraph: number, offset: number) => void;
  /** While the tutor is open, a tap off the text closes it instead of turning or showing the bar. */
  tapAway?: () => void;
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
          const sheet = startPageTurn(host.current!, flow.current, layout, transition.before, { flow: flow.current, page: next }, transition.direction);
          cancelTurn.current = sheet.cancel;
          sheet.settle(true, () => { cancelTurn.current = undefined; });
        }
      }
    };
    place();
    document.fonts.ready.then(() => place(true));
    return () => { cancelled = true; cancelTurn.current?.(); cancelTurn.current = undefined; };
  }, [layout, view, blocks, pageOf]);

  /** Lifts the page toward `direction`, or nothing while another turn is under way.
   * Past the first or last page there is no sheet, and finishing changes chapter. */
  const begin = (direction: 1 | -1): Turning | undefined => {
    if (paused || cancelTurn.current) return;
    turned();
    const next = page + direction;
    if (next < 0 || next >= pages) {
      const before = { flow: flow.current!.cloneNode(true) as HTMLElement, page };
      return { commit: () => turnPast(direction, before) };
    }
    const commit = () => {
      const first = blocks().findIndex((block) => [...block.getClientRects()].some((rect) => pageOf(rect) >= next));
      if (first >= 0) { anchor.current = first; rememberRef.current(first); }
      setPage(next);
    };
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return { commit };
    const sheet = startPageTurn(host.current!, flow.current!, layout, { flow: flow.current!, page }, { flow: flow.current!, page: next }, direction);
    cancelTurn.current = sheet.cancel;
    return { sheet, commit };
  };
  const settle = ({ sheet, commit }: Turning, complete: boolean) => {
    if (!sheet) return complete && commit();
    sheet.settle(complete, () => {
      cancelTurn.current = undefined;
      if (complete) flushSync(commit);
    });
  };
  const turn = (direction: 1 | -1) => {
    const turning = begin(direction);
    if (turning) settle(turning, true);
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

  // Tap text for the tutor; elsewhere the outer thirds turn and the middle shows
  // the controls. Or drag the page: it follows the finger, and on release the
  // finger's last motion decides whether it goes over or falls back.
  const swipe = useRef<Swipe | undefined>(undefined);
  const swiped = useRef(false);
  const release = (complete: (swipe: Swipe) => boolean) => {
    const current = swipe.current;
    swipe.current = undefined;
    if (current?.turning) settle(current.turning, complete(current));
  };
  return (
    <div
      ref={host}
      className="relative flex-1 touch-pan-y touch-pinch-zoom select-none overflow-hidden"
      onTouchStart={(e) => {
        // A second finger is a pinch, not a turn.
        if (e.touches.length > 1) return release(() => false);
        const { clientX: x, clientY: y } = e.touches[0];
        swiped.current = false;
        swipe.current = { x, y, direction: 1, lastX: x, lastTime: e.timeStamp, velocity: 0 };
      }}
      onTouchMove={(e) => {
        const current = swipe.current;
        if (!current) return;
        const { clientX: x, clientY: y } = e.touches[0];
        const dx = x - current.x;
        if (!current.turning) {
          if (Math.max(Math.abs(dx), Math.abs(y - current.y)) < 10) return;
          current.direction = dx < 0 ? 1 : -1;
          current.turning = Math.abs(dx) > Math.abs(y - current.y) ? begin(current.direction) : undefined;
          if (!current.turning) { swipe.current = undefined; return; }
          swiped.current = true;
        }
        current.turning.sheet?.drag(-current.direction * dx / e.currentTarget.clientWidth);
        const elapsed = e.timeStamp - current.lastTime;
        if (elapsed > 0) {
          current.velocity = 0.6 * ((x - current.lastX) / elapsed) + 0.4 * current.velocity;
          current.lastX = x;
          current.lastTime = e.timeStamp;
        }
      }}
      onTouchEnd={(e) => release(({ x, direction, lastX, lastTime, velocity }) => {
        // A finger that came to rest before lifting decides by how far the page went.
        const toward = e.timeStamp - lastTime < 100 ? -direction * velocity : 0;
        return Math.abs(toward) > 0.15 ? toward > 0 : -direction * (lastX - x) > e.currentTarget.clientWidth / 2;
      })}
      onTouchCancel={() => release(() => false)}
      onClick={(e) => {
        if (swiped.current) { swiped.current = false; return; }
        if (paused) return;
        const hit = textAt(e.clientX, e.clientY);
        if (hit) return tapText(hit.paragraph, hit.offset);
        if (tapAway) return tapAway();
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
                <TargetLanguageText language={language}>
                  {highlight?.paragraph === i ? (
                    <>
                      {block.text.slice(0, highlight.range.start)}
                      <mark className="rounded-[0.2em] bg-[var(--reader-highlight)] text-inherit [box-decoration-break:clone]">
                        {block.text.slice(highlight.range.start, highlight.range.end)}
                      </mark>
                      {block.text.slice(highlight.range.end)}
                    </>
                  ) : block.text}
                </TargetLanguageText>
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

/**
 * The tapped sentence, its translation and the tapped word, then a short chat about it.
 * Pages stay clickable around the panel, so tapping another word moves the tutor there.
 */
function SentenceTutor({
  view, language, send, close,
}: {
  view: BookSentenceView;
  language: Language;
  send: (event: BookSentenceEvent) => void;
  close: () => void;
}) {
  const end = useRef<HTMLDivElement>(null);
  useEffect(() => { end.current?.scrollIntoView({ behavior: "smooth", block: "end" }); }, [view.messages.length]);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => event.key === "Escape" && close();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [close]);
  return (
    <section
      aria-live="polite"
      className="absolute inset-x-0 bottom-0 z-20 mx-auto flex max-h-[55dvh] w-full max-w-xl flex-col rounded-t-3xl border border-current/10 bg-[var(--reader-paper)] shadow-[0_-8px_40px_rgb(0_0_0/0.15)] sm:bottom-4 sm:rounded-3xl"
    >
      <Button variant="ghost" size="icon" className="absolute right-3 top-3 z-10 rounded-full bg-[var(--reader-paper)] hover:bg-[var(--reader-paper)] hover:text-inherit" onClick={close} aria-label={books_copy().close}>
        <X aria-hidden />
      </Button>
      <div className="flex flex-col gap-5 overflow-y-auto px-6 pb-3 pt-7">
        {view.sentence && (
          <div className="flex flex-col gap-2 pr-8">
            <p className="font-serif leading-snug text-[var(--reader-muted)]"><TargetLanguageText language={language}>{view.sentence}</TargetLanguageText></p>
            {view.translation && <p className="text-xl font-medium leading-snug">{view.translation}</p>}
          </div>
        )}
        {view.word && view.gloss && (
          <div className="flex flex-col gap-1 rounded-2xl bg-current/5 p-4">
            <p>
              <span className="font-serif text-lg font-semibold"><TargetLanguageText language={language}>{view.word}</TargetLanguageText></span>
              <span className="text-[var(--reader-muted)]"> — </span>
              {view.gloss}
            </p>
            {view.word_explanation && <p>{view.word_explanation}</p>}
          </div>
        )}
        {view.messages.map((message, i) =>
          message.from_learner ? (
            <p key={i} className="ml-12 self-end rounded-[1.1rem] bg-current/8 px-3.5 py-2.5">{message.text}</p>
          ) : (
            <p key={i}>{message.text}</p>
          ),
        )}
        {view.status && (
          <p className="flex items-center gap-2.5 text-[var(--reader-muted)]" role={view.status_is_error ? "alert" : undefined}>
            {!view.status_is_error && <LoaderCircle className="h-4 w-4 animate-spin" aria-hidden />}
            {view.status}
            {view.retry_label && (
              <button className="font-semibold text-[var(--reader-ink)] underline-offset-4 hover:underline" onClick={() => send({ type: "Retry" })}>
                {view.retry_label}
              </button>
            )}
          </p>
        )}
        <div ref={end} />
      </div>
      <form
        className="m-4 mt-1 flex min-h-12 items-center gap-2 rounded-full bg-current/6 pl-5 pr-1.5"
        onSubmit={(e) => { e.preventDefault(); send({ type: "Ask" }); }}
      >
        <input
          className="min-w-0 flex-1 bg-transparent py-3 outline-none placeholder:text-[var(--reader-muted)]"
          placeholder={view.placeholder}
          value={view.draft}
          onChange={(e) => send({ type: "DraftChanged", text: e.target.value })}
        />
        <Button type="submit" size="icon" className="h-9 w-9 shrink-0 rounded-full bg-[var(--reader-ink)] text-[var(--reader-paper)] hover:bg-[var(--reader-ink)]/85 disabled:opacity-30" disabled={!view.can_send} aria-label={view.send_label}>
          <ArrowUp aria-hidden />
        </Button>
      </form>
    </section>
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

export function BookReader({ language, nativeLanguage, slug }: { language: Language; nativeLanguage: Language; slug: string }) {
  const copy = books_copy();
  const { accessToken } = useOutletContext<AppContextType>();
  // The tapped sentence's tutor, shared with iOS through Rust's `book_sentence` reducer.
  const [tutor, setTutor] = useState<BookSentenceState>();
  const tutorRef = useRef(tutor);
  const tutorRun = useRef(0);
  const applyTutor = (step: BookSentenceStep) => {
    tutorRef.current = step.state;
    setTutor(step.state);
    const run = tutorRun.current;
    for (const effect of step.effects) {
      const reply: Promise<BookSentenceEvent> = effect.type === "Explain"
        ? explain_book_sentence(effect.request, accessToken).then(
            (explanation) => ({ type: "Explained", explanation }),
            () => ({ type: "ExplainFailed" }),
          )
        : chat_book_sentence(effect.request, accessToken).then(
            (reply) => ({ type: "Replied", reply }),
            () => ({ type: "ReplyFailed" }),
          );
      void reply.then((event) => run === tutorRun.current && sendTutor(event));
    }
  };
  const sendTutor = (event: BookSentenceEvent) => tutorRef.current && applyTutor(book_sentence_transition(tutorRef.current, event));
  const closeTutor = () => {
    tutorRun.current++;
    tutorRef.current = undefined;
    setTutor(undefined);
  };
  const tutorView = useMemo(() => tutor && book_sentence_view(tutor), [tutor]);
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
    closeTutor();
    setContentsOpen(false);
    setRequest((previous) => ({ chapter, opening, revision: (previous?.revision ?? 0) + 1, turn }));
  };

  return (
    <div
      ref={screen}
      className="fixed inset-0 z-10 flex flex-col bg-[var(--reader-paper)] text-[var(--reader-ink)]"
      style={{ "--reader-paper": theme.background, "--reader-ink": theme.ink, "--reader-muted": theme.muted, "--reader-highlight": theme.highlight } as CSSProperties}
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
          turned={() => { closeTutor(); setBarShown(false); }}
          highlight={tutorView?.highlight ? { paragraph: tutorView.paragraph_index, range: tutorView.highlight } : undefined}
          tapText={(paragraph, offset) => {
            tutorRun.current++;
            applyTutor(book_sentence_start(view, language, nativeLanguage, paragraph, offset));
          }}
          tapAway={tutor && closeTutor}
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
      {tutorView && <SentenceTutor view={tutorView} language={language} send={sendTutor} close={closeTutor} />}
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
    <BookReader key={`${language}-${slug}`} language={language} nativeLanguage={deck.course.nativeLanguage} slug={slug} />
  ) : (
    <Catalog key={language} language={language} nativeLanguage={deck.course.nativeLanguage} />
  );
}
