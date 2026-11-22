import { t } from "@/lib/i18n";
import { LanguageIcon } from "@/components/LanguageIcon";
import { useEffect, useMemo, useRef, useState, type RefObject } from "react";
import { Link, useNavigate, useOutletContext } from "react-router-dom";
import {
  account_copy,
  get_showcase_data,
  type CourseShowcase,
  type Language,
} from "../../../yap-frontend-rs/pkg";
import { ShaderCanvas, ShaderTexture } from "@/components/shader-canvas";
import { DotGridCanvas } from "@/components/dot-grid-canvas";
import { useShaderTheme } from "@/components/use-shader-theme";
import { useTheme } from "@/components/theme-provider";
import { shaderAvailable } from "@/lib/shader-background";
import { getShaderBackgroundCss } from "@/lib/shader-colors";
import { Card } from "@/components/ui/card";
import { LANGUAGES, detectBrowserLanguage } from "@/lib/languages";
import { AnimatePresence, motion } from "framer-motion";
import { Moon, MoreVertical, Volume2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { SiteFooter } from "@/components/site-footer";
import { APP_STORE_URL, IS_IOS } from "@/lib/ios-app";
import { mountGlobe, type Landmark } from "@/lib/landing-globe";
import type { AppContextType, UserInfo } from "@/app/context";
import { useDeckSelection } from "@/core/useDeck";
import { useAuthDialog } from "@/auth/auth-dialog-provider";
import { ModeToggle } from "@/browse/mode-toggle";
import { GITHUB_REPO_URL, GitHubIcon } from "@/components/GitHubIcon";
import "./landing.css";

// ---------------------------------------------------------------------------
// Building blocks

/** The one grid every section shares: a label column, then content. */
const GRID = "grid gap-3 md:grid-cols-[220px_minmax(0,1fr)] md:gap-10";
/** Horizontal page margins. */
const GUTTER = "px-5 sm:px-10 xl:px-[120px]";

/** The bare cutouts from the icon set, by icon id. */
const CUTOUTS = Object.fromEntries(
  Object.entries(
    import.meta.glob<string>("../assets/language-icons/*-bare.svg", {
      eager: true,
      query: "?url",
      import: "default",
    }),
  ).map(([path, url]) => [path.slice(path.lastIndexOf("/") + 1, -9), url]),
);

/** Where each course's landmark stands, as [lat, lon] in degrees. */
const LANDMARKS: Landmark[] = Object.entries({
  "deu.neuschwanstein": [47.56, 10.75],
  "eng.liberty": [40.69, -74.04],
  "fra.eiffel": [48.86, 2.29],
  "hin.tajmahal": [27.17, 78.04],
  "ita.pisa": [43.72, 10.4],
  "jpn.torii": [34.3, 132.32],
  "kor.gyeongbokgung": [37.58, 126.98],
  "por-pt.tram": [38.72, -9.14],
  "por.cristo": [-22.95, -43.21],
  "rus.basil": [55.75, 37.62],
  "spa-es.sagrada": [41.4, 2.17],
  "spa.calavera": [19.43, -99.13],
  "tha.elephant": [13.75, 100.5],
  "zho-hans.panda": [30.66, 104.06],
  "zho-hant.boba": [25.03, 121.56],
}).map(([id, [lat, lon]]) => ({ url: CUTOUTS[id], lat, lon }));

/** Full-bleed navigation aligned to the landing page's own gutter. */
function LandingHeader({ userInfo }: { userInfo: UserInfo | undefined }) {
  const { openSignIn } = useAuthDialog();
  const copy = account_copy();

  return (
    <header
      className={cn(
        "landing-themed absolute inset-x-0 top-0 z-20 flex h-20 items-center justify-between gap-4 bg-transparent!",
        GUTTER,
      )}
    >
      <div className="flex items-center gap-8">
        <Link
          to="/"
          className="font-heading text-[22px] leading-none font-bold tracking-[-0.03em]"
        >
          Yap<span className="text-(--pink)">.</span>Town
        </Link>
        <nav className="hidden items-center gap-6 text-[14.5px] text-(--fg-dim) md:flex">
          <a href="/d/">{t("common-dictionary")}</a>
          <Link to="/anki">{t("landing-anki-decks")}</Link>
          <Link to="/about">{t("common-about")}</Link>
        </nav>
      </div>
      <div className="flex items-center gap-2 sm:gap-3">
        <a
          href={GITHUB_REPO_URL}
          className="landing-nav-github"
          aria-label="GitHub"
        >
          <GitHubIcon className="size-4" />
          <span className="hidden sm:inline">GitHub</span>
        </a>
        {userInfo ? (
          <Link
            to="/settings"
            className="px-2 py-1 text-[14px] text-(--fg-dim) transition-colors hover:text-(--fg)"
            title={t("common-settings")}
          >
            {userInfo.displayName || userInfo.email}
          </Link>
        ) : (
          <button
            type="button"
            onClick={openSignIn}
            className="cursor-pointer px-2.5 py-1.5 text-[14px] font-medium text-(--fg-soft) transition-colors hover:text-(--fg)"
          >
            {copy.sign_in_action}
          </button>
        )}
        <ModeToggle />
      </div>
    </header>
  );
}

/** The hero's little spinning world. The canvas covers the hero; the world
 *  itself sits in `slot`. */
function WorldCanvas({
  slot,
  className,
}: {
  slot: RefObject<HTMLElement | null>;
  className?: string;
}) {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const canvas = ref.current;
    if (!canvas || !slot.current) return;
    return mountGlobe(canvas, slot.current, LANDMARKS);
  }, [slot]);
  return (
    <canvas
      ref={ref}
      aria-hidden
      className={cn(
        "block touch-pan-y opacity-0 transition-opacity duration-700 data-ready:opacity-100",
        className,
      )}
    />
  );
}

function GitHubSourceButton({ className }: { className?: string }) {
  return (
    <a
      href={GITHUB_REPO_URL}
      className={cn("landing-btn landing-btn-outline", className)}
    >
      <GitHubIcon className="size-4 shrink-0" />
      <span>GitHub</span>
    </a>
  );
}

/** The call to action crossfades into the app: the browser keeps a snapshot
 *  of this page and landing.css blurs and fades it out over the next one. On
 *  iOS the App Store comes first, with the browser as a quiet fallback. */
function CallToAction({
  className,
  includeGitHub = false,
}: {
  className?: string;
  includeGitHub?: boolean;
}) {
  if (!IS_IOS) {
    return (
      <div className={cn("flex flex-wrap items-center gap-3", className)}>
        <Link to="/select-language" viewTransition className="landing-btn">
          {t("idle-start-learning")}
        </Link>
        {includeGitHub && <GitHubSourceButton />}
      </div>
    );
  }
  return (
    <div className={cn("flex flex-wrap items-center gap-3", className)}>
      <a href={APP_STORE_URL} className="landing-btn">
        {t("ios-app-download")}
      </a>
      <Link
        to="/select-language"
        viewTransition
        className="landing-btn landing-btn-outline"
      >
        {t("ios-app-get-started-in-browser")}
      </Link>
      {includeGitHub && <GitHubSourceButton />}
    </div>
  );
}

/** The section's own ground at the given opacity, for fades over photos. */
function veil(percent: number) {
  return `color-mix(in srgb, var(--bg) ${percent}%, transparent)`;
}

type Corpus = ReturnType<typeof useCorpus>;

/** "Learn how X people actually talk", with X taken from the courses on
 *  offer, or from every language we know of until the data is in. */
function peopleOf(targets: Language[] | undefined): string[] {
  const langs =
    targets && targets.length > 0
      ? targets
      : (Object.keys(LANGUAGES) as Language[]).filter((l) => l !== "English");
  return [...new Set(langs.map((l) => LANGUAGES[l].people))];
}

/** One word at a time from the list, a few seconds each, fading in. The
 *  word sits on its own line so its length never reflows the rest. */
function RotatingWord({ words }: { words: string[] }) {
  const [i, setI] = useState(0);
  useEffect(() => {
    const id = setInterval(() => setI((n) => (n + 1) % words.length), 2600);
    return () => clearInterval(id);
  }, [words.length]);
  const word = words[i % words.length];
  return (
    <span key={word} className="landing-word text-(--pink)">
      {word}
    </span>
  );
}

function Hero({ corpus }: { corpus: Corpus }) {
  const people = useMemo(() => peopleOf(corpus?.targets), [corpus]);
  const slot = useRef<HTMLDivElement>(null);
  return (
    <div className="landing-hero landing-themed relative flex min-h-svh flex-col overflow-hidden">
      <WorldCanvas slot={slot} className="absolute inset-0 h-full w-full" />
      <div className="landing-grain" />
      {/* Where the world sits: under the copy on narrow screens, beside it
          on wide ones. */}
      <div
        ref={slot}
        className="pointer-events-none aspect-square w-full md:absolute md:top-[54%] md:left-[75%] md:w-[min(46vw,86svh)] md:-translate-1/2"
      />

      <div
        className={cn(
          "relative z-[1] flex flex-col items-start gap-7 pt-28 max-md:-order-1 md:pointer-events-none md:flex-1 md:justify-center md:gap-8 md:py-24 md:*:pointer-events-auto",
          GUTTER,
        )}
      >
        <h1 className="landing-disp m-0 text-[clamp(34px,5vw,76px)] leading-[1.0]">
          {t("landing-hero.before")}
          <br />
          <RotatingWord words={people} />
          <br />
          {t("landing-hero.after")}
        </h1>
        <p
          className="m-0 max-w-[520px] text-[18px] leading-[1.5] text-(--fg-soft) max-md:max-w-[85%] md:text-[21px]"
          style={{ textWrap: "pretty" }}
        >
          {t("landing-yap-teaches-words-in-real-contexts-every-sentence-comes")}
        </p>
        <CallToAction includeGitHub />
      </div>
    </div>
  );
}

/** The app's own review card, framed upright beside the three things worth
 *  knowing about it, color-keyed to the highlights inside the card. */
function Product() {
  const { animatedBackground } = useTheme();
  const theme = useShaderTheme();
  const shader = useMemo(
    () => shaderAvailable(animatedBackground),
    [animatedBackground],
  );
  const points = [
    {
      dot: "bg-(--pink)",
      title: t("landing-real-sentences"),
      description: t(
        "landing-learn-words-in-context-from-real-films-books-and",
      ),
    },
    {
      dot: "bg-amber-500 dark:bg-amber-300",
      title: t("landing-phrase-aware"),
      description: t("landing-cards-target-precise-meanings-not-just-words"),
    },
    {
      dot: "bg-sky-600 dark:bg-sky-300",
      title: t("landing-smarter-review"),
      description: t(
        "landing-fsrs-brings-each-word-back-just-before-you-would",
      ),
    },
  ];
  return (
    <section
      className={cn(
        "landing-themed relative border-t border-(--rule) py-24 md:py-32",
        GUTTER,
      )}
    >
      <div className="grid items-center gap-12 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.05fr)] lg:gap-16">
        <div className="flex flex-col">
          <h2 className="landing-disp m-0 text-[clamp(36px,4vw,52px)] leading-[1.04]">
            {t("landing-every-card-is-a-sentence")}
          </h2>
          <p
            className="m-0 mt-6 max-w-[560px] text-[18px] leading-[1.55] text-(--fg-soft)"
            style={{ textWrap: "pretty" }}
          >
            {t("landing-hear-it-translate-it-speak-it-yap-analyzes-your")}
          </p>
          <dl className="mt-10 divide-y divide-(--rule) border-y border-(--rule)">
            {points.map((point) => (
              <div
                key={point.title}
                className="grid grid-cols-[12px_minmax(0,1fr)] items-baseline gap-x-3.5 py-4"
              >
                <span
                  aria-hidden
                  className={cn("size-2.5 translate-y-[-1px] rounded-full", point.dot)}
                />
                <div>
                  <dt className="text-[15.5px] font-semibold text-(--fg)">
                    {point.title}
                  </dt>
                  <dd className="m-0 mt-0.5 text-[14.5px] leading-relaxed text-(--fg-dim)">
                    {point.description}
                  </dd>
                </div>
              </div>
            ))}
          </dl>
        </div>

        <div
          className="relative overflow-hidden rounded-[16px] border border-(--rule) p-3.5 sm:p-10 [corner-shape:round]"
          style={{ backgroundColor: getShaderBackgroundCss(theme) }}
        >
          {shader && (
            <div className="pointer-events-none absolute inset-0">
              <ShaderCanvas className="absolute inset-0" />
              <ShaderTexture className="absolute inset-0" />
            </div>
          )}
          <div className="relative flex justify-center">
            <MockFlashcard />
          </div>
        </div>
      </div>
    </section>
  );
}

function MockFlashcard() {
  const wordClass = "border-b-2 border-dotted border-(--pink) pb-0.5";
  return (
    <Card className="w-full max-w-lg gap-4 rounded-[14px] p-4 text-left text-foreground sm:gap-5 sm:p-5 [corner-shape:round]">
      <div className="flex items-center justify-between px-1">
        <div className="flex items-center gap-2">
          <LanguageIcon icon="fra.eiffel" variant="bare" className="size-6" />
          <span className="text-base font-bold">Yap</span>
        </div>
        <div className="flex items-center gap-3 text-sm text-muted-foreground">
          <span className="rounded-[6px] border border-border/60 px-2 py-0.5 text-xs font-medium tabular-nums [corner-shape:round]">
            25
          </span>
          <span>André</span>
          <Moon className="h-4 w-4" />
        </div>
      </div>

      <Card className="gap-4 rounded-[12px] p-4 sm:p-6 [corner-shape:round]">
        <div className="flex items-center gap-3">
          <Volume2 className="h-5 w-5 shrink-0 text-muted-foreground" />
          <p
            lang="fr"
            className="flex flex-1 flex-wrap gap-x-2 gap-y-1 text-2xl font-semibold tracking-tight sm:text-[28px]"
          >
            <span className={wordClass}>Il</span>
            <span className={wordClass}>m'en</span>
            <span className={wordClass}>faut</span>
            <span className="inline-flex gap-x-2 text-amber-600 dark:text-amber-300">
              <span className={wordClass}>un</span>
              <span className={wordClass}>autre.</span>
            </span>
          </p>
          <MoreVertical className="h-5 w-5 shrink-0 text-muted-foreground" />
        </div>
        <div className="rounded-[8px] border border-border/60 bg-background/40 px-3.5 py-2.5 text-base text-muted-foreground [corner-shape:round]">
          {t("translation-translation")}
        </div>
      </Card>
      <p className="px-1 text-center text-sm font-medium text-sky-700 dark:text-sky-300">
        {t("landing-reviewing-6-words")}
      </p>
    </Card>
  );
}

function HighlightPhrase({ text, phrase }: { text: string; phrase: string }) {
  const idx = text.toLowerCase().indexOf(phrase.toLowerCase());
  if (idx === -1) return <>{text}</>;
  return (
    <>
      {text.slice(0, idx)}
      <mark className="rounded-xs bg-(--pink)/22 px-1 text-(--fg) [corner-shape:round]">
        {text.slice(idx, idx + phrase.length)}
      </mark>
      {text.slice(idx + phrase.length)}
    </>
  );
}

/** The corpus, live: pick a language and watch its words go by, each with
 *  the definition and three of the sentences it was found in, on a grid of
 *  dots the pointer can stir. */
function Showcase({ corpus }: { corpus: Corpus }) {
  const { animatedBackground } = useTheme();
  const dots = useMemo(
    () => shaderAvailable(animatedBackground),
    [animatedBackground],
  );
  const [selected, setSelected] = useState(0);
  const [phraseIdx, setPhraseIdx] = useState(0);
  useEffect(() => {
    const id = setInterval(() => setPhraseIdx((i) => i + 1), 5000);
    return () => clearInterval(id);
  }, [selected]);
  if (!corpus) return null;
  const course = corpus.courses[selected] ?? corpus.courses[0];
  if (!course || course.phrases.length === 0) return null;
  const phrase = course.phrases[phraseIdx % course.phrases.length];
  const name = LANGUAGES[course.targetLanguage].englishName;

  return (
    <section
      className={cn(
        "landing-themed relative flex min-h-svh flex-col items-center justify-center overflow-hidden border-t border-(--rule) py-24",
        GUTTER,
      )}
    >
      {dots ? (
        <DotGridCanvas className="absolute inset-0" />
      ) : (
        <div className="landing-dots absolute inset-0" />
      )}
      <div className="relative flex w-full max-w-3xl flex-col items-center gap-10">
        <div className="flex flex-col items-center gap-3 text-center">
          <h2
            className="landing-disp m-0 text-[clamp(36px,4.2vw,54px)] leading-none"
            style={{ textWrap: "balance" }}
          >
            {t("landing-sentence-count", {
              count: String(corpus.sentences.toLocaleString()),
            })}
          </h2>
          <p
            className="m-0 max-w-[540px] text-[18px] leading-[1.5] text-(--fg-soft)"
            style={{ textWrap: "balance" }}
          >
            {t("landing-language-count", {
              count: corpus.courses.length,
            })}
          </p>
        </div>

        <div
          role="tablist"
          aria-label={t("landing-languages-2")}
          className="flex flex-wrap justify-center gap-1.5"
        >
          {corpus.courses.map((c, i) => (
            <button
              key={c.targetLanguage}
              type="button"
              role="tab"
              aria-selected={i === selected}
              className="landing-chip"
              onClick={() => {
                setSelected(i);
                setPhraseIdx(0);
              }}
            >
              {t("language-name.title-full", {
                language: LANGUAGES[c.targetLanguage].isoCode,
              })}
            </button>
          ))}
        </div>

        <div className="landing-specimen w-full max-w-2xl text-left">
          <AnimatePresence mode="wait">
            <motion.div
              key={`${selected}-${phraseIdx % course.phrases.length}`}
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ duration: 0.22 }}
            >
              <div className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1 border-b border-(--rule) px-6 py-4">
                <div className="flex flex-wrap items-baseline gap-x-3 gap-y-0.5">
                  <p className="font-heading m-0 text-[26px] leading-tight font-bold">
                    {phrase.displayText}
                  </p>
                  <p className="m-0 text-[15px] text-(--fg-dim)">
                    {phrase.definition}
                  </p>
                </div>
                <p className="m-0 text-[13px] text-(--fg-dim) tabular-nums">
                  {t("landing-course-sentences", {
                    count: String(course.sentenceCount.toLocaleString()),
                    language: String(name),
                  })}
                </p>
              </div>
              <div className="divide-y divide-(--rule) px-6">
                {phrase.examples.map((ex, i) => (
                  <div key={i} className="py-4">
                    <p className="m-0 text-[17px] leading-snug font-medium">
                      <HighlightPhrase
                        text={ex.target}
                        phrase={phrase.displayText}
                      />
                    </p>
                    <p className="m-0 mt-1 text-[14.5px] leading-snug text-(--fg-dim)">
                      {ex.native}
                    </p>
                  </div>
                ))}
              </div>
            </motion.div>
          </AnimatePresence>
        </div>

        <div className="flex flex-wrap items-center justify-center gap-x-6 gap-y-2 text-[15px] text-(--fg-soft)">
          <a href="/d/" className="landing-ul">
            {t("landing-look-up-any-word-in-our-dictionaries")}
          </a>
          <Link to="/anki" className="landing-ul">
            {t("landing-or-take-these-sentences-with-you-as-an-anki")}
          </Link>
        </div>
      </div>
    </section>
  );
}

/** One word's recall over forty days, as FSRS models it: each review lands
 *  as recall reaches the 90% the scheduler aims for, and each hold lengthens
 *  the next gap. Days of the reviews, and the gap each one earned. */
const REVIEWS = [
  { day: 1, gap: 3 },
  { day: 4, gap: 8 },
  { day: 12, gap: 21 },
  { day: 33, gap: 55 },
];
const HORIZON = 40;
/** Recall after `t` days at stability `s`: FSRS's forgetting curve, which
 *  reaches 0.9 exactly at t = s. */
const recall = (t: number, s: number) => (1 + ((19 / 81) * t) / s) ** -0.5;

function RecallChart() {
  const x = (day: number) => ((day - 1) / (HORIZON - 1)) * 100;
  // The vertical axis runs from full recall down to the review line and is
  // square-rooted between them: FSRS's curve is nearly straight across that
  // narrow band, and the root gives the drop its steep start and long tail.
  const y = (r: number) => Math.sqrt((1 - r) / 0.1) * 100;
  const segments = REVIEWS.map(({ day, gap }) => {
    const end = Math.min(day + gap, HORIZON);
    const pts: string[] = [];
    // Quadratic parameter spacing puts more vertices near t = day where the
    // square-root transform curves most steeply.
    for (let i = 0; i <= 48; i++) {
      const u = i / 48;
      const t = day + (end - day) * u * u;
      pts.push(`${x(t)},${y(recall(t - day, gap))}`);
    }
    return { day, gap, end, pts };
  });
  return (
    <div className="mt-10 md:mt-14">
      <div className="landing-label flex items-baseline justify-between text-(--fg-dim)">
        <span>
          <span className="text-(--fg)">100%</span>
          {t("landing-recalled")}
        </span>
        <span className="landing-tag tracking-normal normal-case">
          {t("landing-next-gap-55-days-then-longer")}
        </span>
      </div>
      <div className="relative mt-2 h-[200px] md:h-[260px]">
        <svg
          className="absolute inset-0 h-full w-full overflow-visible"
          viewBox="0 0 100 100"
          preserveAspectRatio="none"
          aria-hidden
        >
          {segments.map((sg) => (
            <g key={sg.day}>
              <polygon
                points={`${sg.pts.join(" ")} ${x(sg.end)},100 ${x(sg.day)},100`}
                fill="var(--fg)"
                fillOpacity="0.14"
              />
              <line
                x1={x(sg.day)}
                y1="0"
                x2={x(sg.day)}
                y2="100"
                stroke="var(--pink)"
                strokeOpacity="0.7"
                vectorEffect="non-scaling-stroke"
              />
              <polyline
                points={sg.pts.join(" ")}
                fill="none"
                stroke="var(--fg)"
                strokeWidth="2"
                vectorEffect="non-scaling-stroke"
              />
            </g>
          ))}
          <line
            x1="0"
            y1="0"
            x2="100"
            y2="0"
            stroke="var(--fg)"
            strokeOpacity="0.3"
            vectorEffect="non-scaling-stroke"
          />
          <line
            x1="0"
            y1="100"
            x2="100"
            y2="100"
            stroke="var(--fg)"
            strokeOpacity="0.3"
            strokeDasharray="2 3"
            vectorEffect="non-scaling-stroke"
          />
        </svg>
        {REVIEWS.map((r) => (
          <div
            key={r.day}
            className="absolute top-0 h-2.5 w-2.5 -translate-x-1/2 -translate-y-1/2 rounded-full bg-(--pink)"
            style={{ left: `${x(r.day)}%` }}
          />
        ))}
      </div>
      <div className="landing-label mt-2 text-right text-(--fg-dim)">
        <span className="text-(--fg)">90%</span>
        {t("landing-about-to-forget-the-review-lands-here")}
      </div>
      {/* Day labels, on two rows where one row cannot hold them. */}
      <div className="relative mt-4 h-12 border-t border-(--rule-strong) xl:h-6">
        {REVIEWS.map((r, i) => (
          <div
            key={r.day}
            className={cn(
              "absolute top-2 text-[14.5px] leading-none whitespace-nowrap text-(--pink)",
              i > 0 && "-translate-x-1/2",
              i % 2 === 1 && "max-xl:top-7",
            )}
            style={{ left: `${x(r.day)}%` }}
          >
            {t("landing-day", { day: r.day })}
            {i > 0 && (
              <span className="text-(--fg-dim)">
                {" "}
                · +{r.day - REVIEWS[i - 1].day}
              </span>
            )}
          </div>
        ))}
        <div className="absolute top-2 right-0 text-[14.5px] leading-none text-(--fg-dim)">
          {t("landing-day", { day: HORIZON })}
        </div>
      </div>
    </div>
  );
}

/** The scheduler, over the sky (a starry night of pink-lit clouds, or the
 *  same clouds by day in light mode): the forgetting curve, the call to
 *  action, and the footer along the bottom of the same frame. */
function Coda() {
  return (
    <div className="landing-themed relative flex min-h-svh flex-col overflow-hidden border-t border-(--rule)">
      {(["sky-day", "sky"] as const).map((name) => (
        <img
          key={name}
          className={cn(
            "landing-breathe absolute top-0 left-0 h-full w-[120%] object-cover",
            name === "sky" ? "hidden dark:block" : "dark:hidden",
          )}
          src={`/landing/${name}.webp`}
          alt=""
          // Far below the fold; lazy also keeps the other theme's sky, which
          // is display: none, from downloading at all.
          loading="lazy"
          style={{ objectPosition: "50% 35%" }}
        />
      ))}
      <div
        className="absolute inset-0"
        style={{
          background: `linear-gradient(180deg, var(--bg) 0%, ${veil(72)} 22%, ${veil(42)} 50%, ${veil(32)} 75%, ${veil(70)} 100%)`,
        }}
      />
      <div
        className={cn(
          "relative flex flex-1 flex-col justify-end gap-12 pt-20 md:gap-16 md:pt-24",
          GUTTER,
        )}
      >
        <div>
          <div className={GRID}>
            <div className="flex flex-col gap-2.5">
              <div className="text-[20px] leading-none font-semibold text-(--pink)">
                se souvenir
              </div>
              <div className="landing-tag leading-snug text-(--fg-soft)">
                Un gosse, ça met du temps à se souvenir.
              </div>
              <div className="text-[14.5px] leading-snug text-(--fg-dim)">
                A kid takes a while to remember.
              </div>
            </div>
            <div>
              <h2
                className="landing-disp m-0 text-[clamp(36px,4.2vw,52px)] leading-none"
                style={{ textWrap: "balance" }}
              >
                {t("landing-the-scheduler")}
              </h2>
              <p
                className="m-0 mt-6 max-w-[640px] text-[18px] leading-[1.55] text-(--fg-soft)"
                style={{ textWrap: "pretty" }}
              >
                {t(
                  "landing-fsrs-spaced-repetition-models-your-memory-by-understanding-what",
                )}
              </p>
            </div>
          </div>
          <RecallChart />
          <div className="mt-6 flex flex-col gap-5 sm:flex-row sm:items-end sm:justify-between sm:gap-10">
            <div className="landing-tag max-w-[520px] text-(--fg-dim)">
              {t("landing-recall-of-one-word-over-forty-days-each-dot")}
            </div>
            <CallToAction
              includeGitHub
              className="shrink-0 self-start sm:self-auto"
            />
          </div>
        </div>
        <SiteFooter className="[--border:var(--rule)] [--muted-foreground:var(--fg-dim)]" />
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------

/** The courses on offer for the visitor's language (English's if we have
 *  none for it), with the total sentence count and the languages taught. */
function useCorpus() {
  return useMemo(() => {
    let all: CourseShowcase[];
    try {
      all = get_showcase_data();
    } catch {
      all = [];
    }
    const native = detectBrowserLanguage() ?? "English";
    let courses = all.filter((c) => c.nativeLanguage === native);
    if (courses.length === 0)
      courses = all.filter((c) => c.nativeLanguage === "English");
    if (courses.length === 0) return null;
    const total = courses.reduce((sum, c) => sum + c.sentenceCount, 0);
    return {
      courses,
      sentences: Math.floor(total / 10_000) * 10_000,
      targets: courses.map((c) => c.targetLanguage),
    };
  }, []);
}

export function LandingPage() {
  const { userInfo } = useOutletContext<AppContextType>();
  const deckSelection = useDeckSelection();
  const navigate = useNavigate();
  const corpus = useCorpus();

  useEffect(() => {
    if (deckSelection?.type === "languageSelected") {
      navigate("/learn", { replace: true });
    }
  }, [deckSelection, navigate]);

  // `null` means the deck_selection stream hasn't been read from OPFS yet.
  // Rendering the hero in that window flashes it at signed-in users before
  // the redirect above fires, so wait until we know.
  if (deckSelection === null || deckSelection.type === "languageSelected") {
    return null;
  }

  // The page breaks out of the shell's narrow column and runs full-bleed,
  // with a full-width header aligned to the page's gutter.
  return (
    <div className="landing landing-night relative z-10 -mb-4 ml-[calc(50%-50vw)] w-screen overflow-hidden">
      <LandingHeader userInfo={userInfo} />
      <Hero corpus={corpus} />
      <Product />
      <Showcase corpus={corpus} />
      <Coda />
    </div>
  );
}
