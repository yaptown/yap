import { t } from "@/lib/i18n";
import {
  ArrowRight,
  Clapperboard,
  Gauge,
  Gift,
  Headphones,
  Volume2,
  VolumeX,
} from "lucide-react";
import { useEffect, useRef, useState, type RefObject } from "react";
import { useNavigate, useOutletContext } from "react-router-dom";
import type { AppContextType } from "@/app/context";
import { TargetLanguageText } from "@/components/TargetLanguageText";
import { TopPageLayout } from "@/components/TopPageLayout";
import { Button } from "@/components/ui/button";

// What a visitor sees at /anki before they've picked a course: the pitch,
// then the course picker, which brings them back here to build the deck.
export function AnkiIntro() {
  const { userInfo } = useOutletContext<AppContextType>();
  const navigate = useNavigate();
  const choose = () =>
    navigate(`/select-language?next=${encodeURIComponent("/anki")}`);
  const features = [
    {
      Icon: Clapperboard,
      title: t("anki-intro-clips-title"),
      body: t("anki-intro-clips-body"),
    },
    {
      Icon: Gauge,
      title: t("anki-intro-level-title"),
      body: t("anki-intro-level-body"),
    },
    {
      Icon: Headphones,
      title: t("anki-intro-cards-title"),
      body: t("anki-intro-cards-body"),
    },
    {
      Icon: Gift,
      title: t("anki-intro-free-title"),
      body: t("anki-intro-free-body"),
    },
  ];
  return (
    <TopPageLayout
      userInfo={userInfo}
      headerProps={{
        title: t("anki-page-anki-decks"),
        backButton: { label: "Yap.Town", onBack: () => navigate("/") },
      }}
    >
      <main className="mx-auto flex w-full max-w-2xl flex-col gap-14 px-5 py-10 sm:py-14">
        <section className="flex flex-col items-center gap-6 text-center">
          <h1
            className="text-4xl font-bold leading-[1.1] tracking-tight sm:text-5xl"
            style={{ textWrap: "balance" }}
          >
            {t("anki-intro-title")}
          </h1>
          <p className="max-w-xl text-lg text-muted-foreground text-pretty">
            {t("anki-intro-lede")}
          </p>
          <CardFan />
          <Button
            size="lg"
            className="group h-12 px-6 text-base font-medium shadow-lg shadow-primary/25"
            onClick={choose}
          >
            {t("anki-intro-make-my-deck")}
            <ArrowRight className="transition-transform group-hover:translate-x-0.5" />
          </Button>
        </section>
        <ul className="grid gap-4 sm:grid-cols-2">
          {features.map(({ Icon, title, body }) => (
            <li
              key={title}
              className="flex gap-4 rounded-xl border bg-card/80 p-5"
            >
              <span className="flex size-10 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
                <Icon className="size-5" aria-hidden />
              </span>
              <div className="flex flex-col gap-1">
                <h2 className="font-semibold">{title}</h2>
                <p className="text-sm text-muted-foreground text-pretty">
                  {body}
                </p>
              </div>
            </li>
          ))}
        </ul>
        <Button
          size="lg"
          variant="outline"
          className="h-12 self-center px-8 text-base font-medium"
          onClick={choose}
        >
          {t("anki-intro-make-my-deck")}
        </Button>
      </main>
    </TopPageLayout>
  );
}

// Real cards from the French deck, fanned out like a hand: stills from
// Portrait de la jeune fille en feu and Les Quatre Cents Coups behind, and
// Amélie in front, looping silently until it's tapped for sound.
function CardFan() {
  const video = useRef<HTMLVideoElement>(null);
  const [muted, setMuted] = useState(true);
  const face =
    "overflow-hidden rounded-xl border border-white/10 bg-card shadow-2xl";
  const card = `absolute inset-x-0 top-0 transition-transform duration-500 ${face}`;
  const still = "aspect-[2/1] w-full object-cover";
  return (
    <div className="group relative mt-6 mb-14 aspect-[16/9] w-full max-w-[16rem] sm:max-w-sm">
      <div
        className={`${card} -rotate-[10deg] -translate-x-[30%] translate-y-[2%] scale-90 group-hover:-rotate-[13deg] group-hover:-translate-x-[36%]`}
      >
        <img src="/anki/portrait.avif" alt="" className={still} />
      </div>
      <div
        className={`${card} rotate-[10deg] translate-x-[30%] translate-y-[2%] scale-90 group-hover:rotate-[13deg] group-hover:translate-x-[36%]`}
      >
        <img src="/anki/400-blows.avif" alt="" className={still} />
      </div>
      {/* The beam rides with the front card so it always lands on its top
          edge, even when the card lifts on hover. It moves by `top`, not a
          transform: a transform would isolate the beam's screen blend from
          the page behind it. */}
      <div className="absolute inset-x-0 top-[7%] transition-[top] duration-500 group-hover:top-[3%]">
        <ProjectorBeam video={video} />
        <div className={face}>
          <button
            type="button"
            className="relative block w-full"
            aria-label={t("anki-intro-play-with-sound")}
            onClick={() => {
              const element = video.current;
              if (!element) return;
              element.muted = !muted;
              if (muted) element.currentTime = 0;
              void element.play();
              setMuted(!muted);
            }}
          >
            <video
              ref={video}
              src="/anki/amelie.mp4"
              poster="/anki/amelie.avif"
              className={still}
              autoPlay
              muted
              loop
              playsInline
            />
            <Lit />
            <span className="absolute top-2 right-2 flex size-8 items-center justify-center rounded-full bg-black/40 ring-1 ring-white/25 backdrop-blur-md">
              {muted ? (
                <VolumeX className="size-4 text-white" />
              ) : (
                <Volume2 className="size-4 text-white" />
              )}
            </span>
            <p className="absolute inset-x-0 bottom-[8%] text-center text-base font-medium text-white [text-shadow:0_1px_3px_#000] sm:text-lg">
              <TargetLanguageText language="French">
                Vous croyez aux miracles ?
              </TargetLanguageText>
            </p>
          </button>
          <p className="border-t border-dashed px-4 py-3 text-center text-sm text-muted-foreground">
            {t("anki-intro-sample-translation")}
          </p>
        </div>
      </div>
    </div>
  );
}

// A projector beam from the top of the page, cast by the clip itself: each
// frame is shrunk to a strip of columns, and each column throws a ray from
// the lamp down to the same spot on the front card's top edge, so bright,
// warm parts of the scene light the haze above them. Lit carries the light
// onto the card itself.
const BEAM_COLUMNS = 24;

function ProjectorBeam({
  video,
}: {
  video: RefObject<HTMLVideoElement | null>;
}) {
  const canvas = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const element = video.current!;
    const context = canvas.current!.getContext("2d")!;
    const sample = document
      .createElement("canvas")
      .getContext("2d", { willReadFrequently: true })!;
    sample.canvas.width = BEAM_COLUMNS;
    sample.canvas.height = 6;
    // Eased toward each frame's levels so a cut doesn't strobe.
    const levels = new Float32Array(BEAM_COLUMNS * 3);
    const paint = (ease: number) => {
      sample.drawImage(element, 0, 0, BEAM_COLUMNS, 6);
      const { data } = sample.getImageData(0, 0, BEAM_COLUMNS, 6);
      const { width, height } = context.canvas;
      context.clearRect(0, 0, width, height);
      context.globalCompositeOperation = "lighter";
      const column = width / BEAM_COLUMNS;
      for (let x = 0; x < BEAM_COLUMNS; x++) {
        const color = [0, 0, 0];
        for (let y = 0; y < 6; y++)
          for (let c = 0; c < 3; c++)
            color[c] += data[(y * BEAM_COLUMNS + x) * 4 + c] / (6 * 255);
        for (let c = 0; c < 3; c++)
          levels[x * 3 + c] += (color[c] - levels[x * 3 + c]) * ease;
        const [r, g, b] = levels.subarray(x * 3, x * 3 + 3);
        const luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        const alpha = 0.03 + 1.2 * Math.max(0, luminance - 0.12);
        // The frame's hue at full brightness, warmed toward the lamp.
        const peak = Math.max(r, g, b, 0.01);
        const tint = [r, g, b]
          .map((v, c) => Math.round(127 * (v / peak) + [128, 122, 110][c]))
          .join(",");
        const gradient = context.createLinearGradient(0, 0, 0, height);
        gradient.addColorStop(0, `rgba(${tint},${alpha * 0.35})`);
        gradient.addColorStop(1, `rgba(${tint},${alpha})`);
        context.fillStyle = gradient;
        context.beginPath();
        context.moveTo(width / 2, 0);
        context.lineTo(x * column, height);
        context.lineTo((x + 1) * column, height);
        context.fill();
      }
    };
    // The first frame lights the beam even if autoplay is blocked (say, in
    // Low Power Mode); after that it follows playback.
    const first = () => paint(1);
    if (element.readyState >= HTMLMediaElement.HAVE_CURRENT_DATA) first();
    element.addEventListener("loadeddata", first);
    let handle = 0;
    const loop = () => {
      paint(0.3);
      handle = element.requestVideoFrameCallback(loop);
    };
    // Firefox before 132 lacks frame callbacks and keeps the first frame's
    // beam.
    if ("requestVideoFrameCallback" in element) {
      // Scheduling a frame callback reads the video; it doesn't mutate it.
      // eslint-disable-next-line react-hooks/immutability
      handle = element.requestVideoFrameCallback(loop);
    }
    return () => {
      element.removeEventListener("loadeddata", first);
      if (handle) element.cancelVideoFrameCallback(handle);
    };
  }, [video]);
  return (
    <div
      aria-hidden
      // Reaches a corner radius onto the card, with its edges pulled in by
      // about the same, so it lands on the card rather than beside its
      // rounded corners.
      className="projector-beam pointer-events-none absolute bottom-[calc(100%-1rem)] left-0 z-10 h-[28rem] w-full mix-blend-screen blur-[6px]"
      style={{
        clipPath: "polygon(48% 0, 52% 0, 97% 100%, 3% 100%)",
        maskImage: "linear-gradient(black calc(100% - 0.6rem), transparent)",
      }}
    >
      <canvas ref={canvas} width={300} height={448} className="size-full" />
      <div className="absolute inset-0 bg-[url(/noise.webp)] bg-cover opacity-50 mix-blend-overlay" />
    </div>
  );
}

// Where the beam lands on the card. Overlay blending lifts the frame's bright
// parts and leaves its shadows dark, as real light on a screen would.
function Lit() {
  return (
    <span
      aria-hidden
      className="projector-beam pointer-events-none absolute inset-0 mix-blend-overlay"
      style={{
        background:
          "radial-gradient(ellipse 70% 110% at 50% 0, #fff4dcb0, #fff4dc30 55%, transparent 80%)",
      }}
    />
  );
}
