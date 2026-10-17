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
import { useRef, useState } from "react";
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
  const card =
    "absolute inset-x-0 top-0 overflow-hidden rounded-xl border border-white/10 bg-card shadow-2xl transition-transform duration-500";
  const still = "aspect-[2/1] w-full object-cover";
  return (
    <div className="group relative mt-6 mb-14 aspect-[16/9] w-full max-w-[16rem] sm:max-w-sm">
      {/* A projector beam from the top of the page: haze in the air that
          fades as it reaches the cards, where Lit takes over. */}
      <div
        aria-hidden
        className="projector-beam pointer-events-none absolute bottom-[45%] left-1/2 z-10 h-[34rem] w-[140%] -translate-x-1/2 mix-blend-screen blur-[12px]"
        style={{
          clipPath: "polygon(48% 0, 52% 0, 100% 100%, 0 100%)",
          background:
            "radial-gradient(ellipse 45% 100% at 50% 0, #fff4dc40, #fff4dc14 70%, transparent)",
          maskImage: "linear-gradient(black 55%, transparent)",
        }}
      >
        <div className="absolute inset-0 bg-[url(/noise.webp)] bg-cover opacity-50 mix-blend-overlay" />
      </div>
      <div
        className={`${card} -rotate-[10deg] -translate-x-[30%] translate-y-[2%] scale-90 group-hover:-rotate-[13deg] group-hover:-translate-x-[36%]`}
      >
        <img src="/anki/portrait.avif" alt="" className={still} />
        <Lit at="80% 0" />
      </div>
      <div
        className={`${card} rotate-[10deg] translate-x-[30%] translate-y-[2%] scale-90 group-hover:rotate-[13deg] group-hover:translate-x-[36%]`}
      >
        <img src="/anki/400-blows.avif" alt="" className={still} />
        <Lit at="20% 0" />
      </div>
      <div className={`${card} translate-y-[6%] group-hover:translate-y-[3%]`}>
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
          <Lit at="50% 0" />
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
  );
}

// Where the beam lands on a card. Overlay blending lifts the frame's bright
// parts and leaves its shadows dark, as real light on a screen would.
function Lit({ at }: { at: string }) {
  return (
    <span
      aria-hidden
      className="projector-beam pointer-events-none absolute inset-0 mix-blend-overlay"
      style={{
        background: `radial-gradient(ellipse 70% 110% at ${at}, #fff4dcb0, #fff4dc30 55%, transparent 80%)`,
      }}
    />
  );
}
