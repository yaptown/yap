import { t } from "@/lib/i18n";
import { LanguageIcon } from "@/components/LanguageIcon";
import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
  type RefObject,
} from "react";
import { Link, useNavigate, useOutletContext } from "react-router-dom";
import {
  get_showcase_data,
  type CourseShowcase,
  type Language,
} from "../../../yap-frontend-rs/pkg";
import { Header } from "@/components/header";
import { ShaderCanvas, ShaderTexture } from "@/components/shader-canvas";
import { DotGridCanvas } from "@/components/dot-grid-canvas";
import { useShaderTheme } from "@/components/use-shader-theme";
import { useTheme } from "@/components/theme-provider";
import { shaderAvailable } from "@/lib/shader-background";
import { getShaderBackgroundCss } from "@/lib/shader-colors";
import { Card } from "@/components/ui/card";
import { LANGUAGES, detectBrowserLanguage } from "@/lib/languages";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { AnimatePresence, motion } from "framer-motion";
import { Moon, MoreVertical, Volume2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { SiteFooter } from "@/components/site-footer";
import { APP_STORE_URL, IS_IOS } from "@/lib/ios-app";
import { mountGlobe, type Landmark } from "@/lib/landing-globe";
import type { AppContextType } from "@/app/context";
import { useDeckSelection } from "@/core/useDeck";
import "./landing.css";

// ---------------------------------------------------------------------------
// Content. The copy is a placeholder from the design; the sentences are real
// corpus lines.

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

/** The call to action crossfades into the app: the browser keeps a snapshot
 *  of this page and landing.css blurs and fades it out over the next one. On
 *  iOS the App Store comes first, with the browser as a quiet fallback. */
function CallToAction({ className }: { className?: string }) {
  if (!IS_IOS) {
    return (
      <Link
        to="/select-language"
        viewTransition
        className={cn("landing-btn", className)}
      >
        {t("idle-start-learning")}
      </Link>
    );
  }
  return (
    <div className={cn("grid w-fit gap-3 sm:grid-flow-col", className)}>
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
    <span key={word} className="landing-word">
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
        className="pointer-events-none aspect-square w-full md:absolute md:top-[54%] md:left-[77%] md:w-[min(48vw,91svh)] md:-translate-1/2"
      />

      <div
        className={cn(
          "relative z-[1] flex flex-col items-start gap-7 pt-24 max-md:-order-1 md:pointer-events-none md:flex-1 md:justify-center md:gap-9 md:py-24 md:*:pointer-events-auto",
          GUTTER,
        )}
      >
        <h1 className="landing-disp m-0 text-[clamp(38px,5.2vw,80px)] leading-[0.98]">
          {t("landing-hero.before")}
          <br />
          <RotatingWord words={people} />
          <br />
          {t("landing-hero.after")}
        </h1>
        <p
          className="m-0 max-w-[560px] text-[19px] leading-[1.4] text-(--fg-soft) max-md:max-w-[82%] md:text-[24px]"
          style={{ textWrap: "pretty" }}
        >
          {t("landing-yap-teaches-words-in-real-contexts-every-sentence-comes")}
        </p>
        <CallToAction />
      </div>
    </div>
  );
}

/** The app's own review card, as it looks in the app and on the app's own
 *  background, with the three things worth knowing about it pinned around
 *  the edges. The pins use CSS anchor positioning and stack underneath where
 *  that is missing. */
function Product() {
  const { animatedBackground } = useTheme();
  const theme = useShaderTheme();
  const shader = useMemo(
    () => shaderAvailable(animatedBackground),
    [animatedBackground],
  );
  return (
    <div className="landing-themed relative flex min-h-svh flex-col justify-center overflow-hidden">
      {shader && (
        <div
          className="absolute inset-0"
          style={{ backgroundColor: getShaderBackgroundCss(theme) }}
        >
          <ShaderCanvas className="absolute inset-0" />
          <ShaderTexture className="absolute inset-0" />
        </div>
      )}
      <div
        className={cn(
          "relative flex flex-col pt-16 pb-24 md:pt-24 md:pb-36",
          GUTTER,
        )}
      >
        <h2 className="landing-disp m-0 text-[clamp(40px,4.5vw,56px)] leading-none">
          {t("landing-every-card-is-a-sentence")}
        </h2>
        <p
          className="m-0 mt-8 max-w-[640px] text-[20px] leading-[1.5] text-(--fg-soft)"
          style={{ textWrap: "pretty" }}
        >
          {t("landing-hear-it-translate-it-speak-it-yap-analyzes-your")}
        </p>
        <CallToAction className="mt-8 self-start" />
        <div className="mt-16 flex flex-col items-center gap-4 lg:gap-0 lg:pb-40">
          <MockFlashcard />
          <Pin anchor="--sentence" area="left">
            <FeatureCard
              title={t("landing-real-sentences")}
              description={t(
                "landing-learn-words-in-context-from-real-films-books-and",
              )}
              titleClassName="text-(--pink)"
            />
          </Pin>
          <Pin anchor="--phrase" area="right">
            <FeatureCard
              title={t("landing-phrase-aware")}
              description={t(
                "landing-cards-target-precise-meanings-not-just-words",
              )}
              titleClassName="text-yellow-400"
            />
          </Pin>
          <Pin anchor="--review" area="bottom">
            <FeatureCard
              title={t("landing-smarter-review")}
              description={t(
                "landing-fsrs-brings-each-word-back-just-before-you-would",
              )}
              titleClassName="text-sky-600 dark:text-sky-100"
            />
          </Pin>
        </div>
      </div>
    </div>
  );
}

function Pin({
  anchor,
  area,
  children,
}: {
  anchor: string;
  area: "left" | "right" | "bottom";
  children: ReactNode;
}) {
  return (
    <div
      className="w-full max-w-sm lg:absolute lg:w-64 lg:max-w-none"
      style={
        { positionAnchor: anchor, positionArea: area } as React.CSSProperties
      }
    >
      {children}
    </div>
  );
}

function FeatureCard({
  title,
  description,
  titleClassName,
}: {
  title: string;
  description: string;
  titleClassName?: string;
}) {
  return (
    <Card
      variant="light"
      className="-rotate-[3deg] flex-row items-start gap-3 p-4 text-left text-foreground"
    >
      <div>
        <h3 className={cn("text-base font-semibold", titleClassName)}>
          {title}
        </h3>
        <p className="mt-0.5 text-sm leading-snug">{description}</p>
      </div>
    </Card>
  );
}

function MockFlashcard() {
  const wordClass = "border-b-2 border-dotted border-(--pink) pb-0.5";
  return (
    <Card className="w-fit max-w-xl rotate-[5deg] gap-5 p-5 text-left text-foreground">
      <div className="flex items-center justify-between px-2">
        <div className="flex items-center gap-2">
          <LanguageIcon icon="fra.eiffel" variant="bare" className="size-6" />
          <span className="text-lg font-bold">Yap</span>
        </div>
        <div className="flex items-center gap-3 text-base text-muted-foreground">
          <span className="rounded-md border border-border/60 px-2 py-0.5 text-sm font-medium">
            25
          </span>
          <span>André</span>
          <Moon className="h-5 w-5" />
        </div>
      </div>

      <Card className="gap-4 p-6">
        <div
          className="flex items-center gap-3"
          style={{ anchorName: "--sentence" } as React.CSSProperties}
        >
          <Volume2 className="h-6 w-6 shrink-0 text-muted-foreground" />
          <p
            lang="fr"
            className="flex flex-1 flex-wrap gap-x-2 gap-y-1 text-2xl font-semibold tracking-tight md:text-3xl"
          >
            <span className={wordClass}>Il</span>
            <span className={wordClass}>m'en</span>
            <span className={wordClass}>faut</span>
            <span
              className="inline-flex gap-x-2 text-yellow-400"
              style={{ anchorName: "--phrase" } as React.CSSProperties}
            >
              <span className={wordClass}>un</span>
              <span className={wordClass}>autre.</span>
            </span>
          </p>
          <MoreVertical className="h-5 w-5 shrink-0 text-muted-foreground" />
        </div>
        <p className="text-lg text-muted-foreground">
          {t("translation-translation")}
        </p>
      </Card>
      <p
        className="px-2 text-center text-sm font-medium text-sky-600 dark:text-sky-100"
        style={{ anchorName: "--review" } as React.CSSProperties}
      >
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
      <mark className="rounded-sm bg-(--pink)/25 px-0.5 text-(--fg)">
        {text.slice(idx, idx + phrase.length)}
      </mark>
      {text.slice(idx + phrase.length)}
    </>
  );
}

/** The corpus, live: pick a language and watch its words go by, each with
 *  the definition and three of the sentences it was found in. */
/** The corpus, live: one word at a time from the reader's likely course,
 *  on a grid of dots the pointer can stir. */
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
    <div
      className={cn(
        "landing-themed relative flex min-h-svh flex-col items-center justify-center overflow-hidden py-24 text-center",
        GUTTER,
      )}
    >
      {dots ? (
        <DotGridCanvas className="absolute inset-0" />
      ) : (
        <div className="landing-dots absolute inset-0" />
      )}
      <div className="relative flex flex-col items-center gap-12">
        <div className="flex flex-col items-center gap-4">
          <h2
            className="landing-disp m-0 text-[clamp(40px,4.5vw,56px)] leading-none"
            style={{ textWrap: "balance" }}
          >
            {t("landing-sentence-count", {
              count: String(corpus.sentences.toLocaleString()),
            })}
          </h2>
          <p
            className="m-0 max-w-[560px] text-[20px] leading-[1.5] text-(--fg-soft)"
            style={{ textWrap: "balance" }}
          >
            {t("landing-language-count", {
              count: corpus.courses.length,
            })}
          </p>
        </div>

        <Tabs
          value={String(selected)}
          onValueChange={(v) => {
            setSelected(Number(v));
            setPhraseIdx(0);
          }}
        >
          <TabsList className="h-auto flex-wrap gap-1">
            {corpus.courses.map((c, i) => (
              <TabsTrigger key={c.targetLanguage} value={String(i)}>
                {t("language-name.title-full", {
                  language: LANGUAGES[c.targetLanguage].isoCode,
                })}
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>

        <AnimatePresence mode="wait">
          <motion.div
            key={`${selected}-${phraseIdx % course.phrases.length}`}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.25 }}
            className="flex w-full max-w-lg flex-col items-center gap-5"
          >
            <div className="flex flex-col items-center gap-1">
              <p className="m-0 text-[28px] leading-none font-semibold">
                {phrase.displayText}
              </p>
              <p className="m-0 text-[15px] text-(--fg-dim)">
                {phrase.definition}
              </p>
            </div>
            <div className="flex w-full flex-col gap-4">
              {phrase.examples.map((ex, i) => (
                <div key={i}>
                  <p className="m-0 text-[17px] leading-snug font-medium">
                    <HighlightPhrase
                      text={ex.target}
                      phrase={phrase.displayText}
                    />
                  </p>
                  <p className="m-0 mt-1 text-[15px] leading-snug text-(--fg-dim)">
                    {ex.native}
                  </p>
                </div>
              ))}
            </div>
            <p className="m-0 text-[15px] text-(--fg-dim)">
              {t("landing-course-sentences", {
                count: String(course.sentenceCount.toLocaleString()),
                language: String(name),
              })}
            </p>
          </motion.div>
        </AnimatePresence>

        <div className="flex flex-col items-center gap-2 text-[15px] text-(--fg-dim)">
          <a href="/d/" className="landing-ul">
            {t("landing-look-up-any-word-in-our-dictionaries")}
          </a>
          <Link to="/anki" className="landing-ul">
            {t("landing-or-take-these-sentences-with-you-as-an-anki")}
          </Link>
        </div>
      </div>
    </div>
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
    for (let i = 0; i <= 24; i++) {
      const t = day + ((end - day) * i) / 24;
      pts.push(`${x(t)},${y(recall(t - day, gap))}`);
    }
    return { day, gap, end, pts };
  });
  return (
    <div className="mt-14 md:mt-20">
      <div className="landing-label flex items-baseline justify-between text-(--fg-dim)">
        <span>
          <span className="text-(--fg)">100%</span>
          {t("landing-recalled")}
        </span>
        <span className="landing-tag normal-case tracking-normal">
          {t("landing-next-gap-55-days-then-longer")}
        </span>
      </div>
      <div className="relative mt-2 h-[220px] md:h-[300px]">
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
                fillOpacity="0.16"
              />
              <line
                x1={x(sg.day)}
                y1="0"
                x2={x(sg.day)}
                y2="100"
                stroke="var(--pink)"
                strokeOpacity="0.75"
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
            strokeOpacity="0.35"
            vectorEffect="non-scaling-stroke"
          />
          <line
            x1="0"
            y1="100"
            x2="100"
            y2="100"
            stroke="var(--fg)"
            strokeOpacity="0.35"
            strokeDasharray="2 3"
            vectorEffect="non-scaling-stroke"
          />
        </svg>
        {REVIEWS.map((r) => (
          <div
            key={r.day}
            className="absolute top-0 h-3 w-3 -translate-x-1/2 -translate-y-1/2 rounded-full bg-(--pink)"
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
              "absolute top-2 text-[15px] leading-none whitespace-nowrap text-(--pink)",
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
        <div className="absolute top-2 right-0 text-[15px] leading-none text-(--fg-dim)">
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
    <div className="landing-themed relative flex min-h-svh flex-col overflow-hidden">
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
          "relative flex flex-1 flex-col justify-end gap-16 pt-32 md:gap-20 md:pt-40",
          GUTTER,
        )}
      >
        <div>
          <div className={GRID}>
            <div className="flex flex-col gap-3">
              <div className="text-[22px] leading-none text-(--pink)">
                se souvenir
              </div>
              <div className="landing-tag leading-snug text-(--fg-soft)">
                Un gosse, ça met du temps à se souvenir.
              </div>
              <div className="text-[15px] leading-snug text-(--fg-dim)">
                A kid takes a while to remember.
              </div>
            </div>
            <div>
              <h2
                className="landing-disp m-0 text-[clamp(40px,4.5vw,56px)] leading-none"
                style={{ textWrap: "balance" }}
              >
                {t("landing-the-scheduler")}
              </h2>
              <p
                className="m-0 mt-8 max-w-[640px] text-[20px] leading-[1.5] text-(--fg-soft)"
                style={{ textWrap: "pretty" }}
              >
                {t(
                  "landing-fsrs-spaced-repetition-models-your-memory-by-understanding-what",
                )}
              </p>
            </div>
          </div>
          <RecallChart />
          <div className="mt-6 flex flex-col gap-6 sm:flex-row sm:items-end sm:justify-between sm:gap-12">
            <div className="landing-tag max-w-[560px] text-(--fg-dim)">
              {t("landing-recall-of-one-word-over-forty-days-each-dot")}
            </div>
            <CallToAction className="shrink-0 self-start sm:self-auto" />
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
  // with the app's usual header floated over the top of the hero.
  return (
    <div className="landing landing-night relative z-10 -mb-4 ml-[calc(50%-50vw)] w-screen overflow-hidden">
      <div className="absolute inset-x-0 top-0 z-20 px-2 text-foreground">
        <div className="mx-auto max-w-2xl py-2">
          <Header userInfo={userInfo} showSignupNag={false} />
        </div>
      </div>
      <Hero corpus={corpus} />
      <Product />
      <Showcase corpus={corpus} />
      <Coda />
    </div>
  );
}
