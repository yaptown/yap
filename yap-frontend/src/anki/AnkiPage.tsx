import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { toast } from "sonner";
import {
  get_ai_server_url,
  mint_anki_deck,
  refresh_clip_manifest,
  type AnkiCardTypes,
  type Deck,
  type Language,
  type MintedAnkiDeck,
  type MovieMetadataBasic,
  type PlacementSession,
} from "../../../yap-frontend-rs/pkg";
import type { AppContextType } from "@/app/context";
import { DeckPage } from "@/app/DeckPage";
import { useAuthDialog } from "@/auth/auth-dialog-provider";
import { Poster } from "@/browse/Poster";
import { TopPageLayout } from "@/components/TopPageLayout";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Progress } from "@/components/ui/progress";
import { useWeapon } from "@/core/weapon";
import { PlacementTest } from "@/review/ladder/PlacementTest";
import type { MediaProgress } from "./apkg";

// Stores the built package so it can be fetched by link (AnkiMobile's
// "Download link"). The backend keeps it for 8 days.
async function uploadPackage(minted: MintedAnkiDeck, languageCode: string, blob: Blob, accessToken?: string): Promise<string> {
  const query = new URLSearchParams({ d: minted.token, code: languageCode });
  const response = await fetch(`${get_ai_server_url()}/anki/deck/${encodeURIComponent(minted.deck_id)}/package?${query}`, {
    method: "PUT",
    headers: { Authorization: `Bearer ${accessToken ?? "anonymous"}`, "Content-Type": "application/octet-stream" },
    body: blob,
  });
  if (!response.ok) throw new Error(`${response.status} ${await response.text()}`);
  const { url } = (await response.json()) as { url: string };
  return url;
}

export function AnkiPage() {
  return <DeckPage prefetchAudio={false}>{(props) => <AnkiScreen key={props.targetLanguage} {...props} />}</DeckPage>;
}

function AnkiPlacement({ deck, targetLanguage, completeLabel }: { deck: Deck; targetLanguage: Language; completeLabel: string }) {
  const weapon = useWeapon();
  const [session, setSession] = useState<PlacementSession>(() => deck.start_placement_session());
  return <PlacementTest
    deck={deck}
    targetLanguage={targetLanguage}
    session={session}
    setSession={setSession}
    completeLabel={completeLabel}
    onComplete={({ known_words, unknown_words }) => {
      weapon.add_deck_event(deck.complete_placement_test(known_words, unknown_words));
    }}
  />;
}

// Drifts sideways forever: the list is drawn twice and the track slides by
// exactly one copy, so the seam never shows.
function PosterStrip({ films, deck }: { films: MovieMetadataBasic[]; deck: Deck }) {
  if (films.length === 0) return null;
  return (
    <div className="poster-strip -mx-5 overflow-hidden py-1 [mask-image:linear-gradient(to_right,transparent,black_3rem,black_calc(100%-3rem),transparent)]">
      <div className="poster-strip-track flex w-max">
        {[...films, ...films].map((film, index) => (
          <div key={index} className="shrink-0 pr-3" aria-hidden={index >= films.length} title={film.year ? `${film.title} (${film.year})` : film.title}>
            <div className="h-36 w-24 overflow-hidden rounded-md border border-border/50 bg-muted shadow-sm">
              <Poster movieId={film.id} deck={deck} alt={film.title} />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function CardTypeOption({ label, description, checked, onChange }: { label: string; description: string; checked: boolean; onChange: (checked: boolean) => void }) {
  return (
    <label className="flex cursor-pointer gap-3 rounded-lg border p-4 transition-colors has-[:checked]:border-primary/50 has-[:checked]:bg-primary/5 has-[:disabled]:cursor-default">
      <input type="checkbox" className="mt-1 size-4 shrink-0 accent-primary" checked={checked} onChange={(event) => onChange(event.target.checked)} />
      <span className="flex flex-col gap-1">
        <span className="font-medium">{label}</span>
        <span className="text-sm text-muted-foreground">{description}</span>
      </span>
    </label>
  );
}

function AnkiScreen({ deck, targetLanguage, userInfo, accessToken }: AppContextType & { deck: Deck; targetLanguage: Language }) {
  const navigate = useNavigate();
  const { openSignUp } = useAuthDialog();
  const [reading, setReading] = useState(true);
  const [listening, setListening] = useState(true);
  const cardTypes: AnkiCardTypes | undefined = reading && listening ? "Both" : reading ? "Reading" : listening ? "Listening" : undefined;
  const [manifest, setManifest] = useState<"loading" | "ready" | "error">("loading");
  const [retry, setRetry] = useState(0);
  const [phase, setPhase] = useState<string>();
  const [progress, setProgress] = useState<MediaProgress>();
  const [result, setResult] = useState<string>();
  const [downloadLink, setDownloadLink] = useState<string>();
  const busy = phase !== undefined;
  const view = useMemo(() => ({ ...deck.anki_export_view(), manifest }), [deck, manifest]);

  useEffect(() => {
    let active = true;
    refresh_clip_manifest(targetLanguage, accessToken).then(() => {
      if (active) setManifest("ready");
    }).catch((error: unknown) => {
      if (active) {
        setManifest("error");
        toast.error(String(error));
      }
    });
    return () => { active = false; };
  }, [targetLanguage, accessToken, retry]);

  async function download(cardTypes: AnkiCardTypes) {
    setPhase("Preparing deck…");
    setProgress(undefined);
    setResult(undefined);
    setDownloadLink(undefined);
    try {
      const options = { card_types: cardTypes };
      const minted = await mint_anki_deck(options, accessToken);
      // Let the preparation status paint before entering the synchronous WASM planner.
      await new Promise((resolve) => setTimeout(resolve, 0));
      const plan = deck.anki_deck_plan(options, minted.token, Date.now());
      const { buildApkg } = await import("./apkg");
      setPhase("Fetching media…");
      const blob = await buildApkg(plan, (source) => deck.anki_bundled_media(source), (next) => {
        setProgress(next);
        if (next.done === next.total) setPhase("Writing Anki package…");
      });
      setPhase("Uploading deck…");
      try {
        setDownloadLink(await uploadPackage(minted, plan.course_code, blob, accessToken));
      } catch (error) {
        toast.error(`Could not create a download link. Your deck will still download. ${String(error)}`);
      }
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = `yap-${plan.course_code}.apkg`;
      document.body.append(anchor);
      anchor.click();
      anchor.remove();
      setTimeout(() => URL.revokeObjectURL(url), 60_000);
      setResult(`${plan.stats.sentence_count.toLocaleString()} sentence notes + ${plan.stats.word_count.toLocaleString()} word notes · ${(blob.size / 1024 / 1024).toFixed(1)} MB`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setPhase(undefined);
      setProgress(undefined);
    }
  }

  return (
    <TopPageLayout userInfo={userInfo} headerProps={{ title: "Anki Decks", backButton: { label: "Yap.Town", onBack: () => navigate("/") } }}>
      <main className="mx-auto flex w-full max-w-2xl flex-col gap-6 px-5 py-8">
        <div className="flex flex-col gap-2">
          <p className="font-mono text-xs text-muted-foreground">Anki · {view.language_name}</p>
          <h1 className="text-2xl font-semibold" style={{ textWrap: "balance" }}>{view.title}</h1>
        </div>
        <PosterStrip films={view.films} deck={deck} />
        {view.needs_placement ? (
          <div className="flex flex-col gap-4">
            <p className="text-sm text-muted-foreground">{view.placement_intro}</p>
            <AnkiPlacement deck={deck} targetLanguage={targetLanguage} completeLabel={view.placement_complete_label} />
          </div>
        ) : (
          <form className="flex flex-col gap-6" onSubmit={(event) => { event.preventDefault(); if (cardTypes) void download(cardTypes); }}>
            <p className="text-muted-foreground">{view.intro}</p>
            {/* Same bar as the Essential tab on the goals screen. */}
            <div className="flex flex-col gap-3 border-y py-4">
              <h2 className="font-semibold">{view.progress_label}</h2>
              <Progress className="h-6" value={view.percent_known} showPercentage aria-label={view.progress_label} />
              {view.too_advanced_message && <p className="text-sm text-muted-foreground">{view.too_advanced_message}</p>}
            </div>
            <fieldset className="flex flex-col gap-3" disabled={busy}>
              <legend className="mb-3 font-semibold">{view.card_types_label}</legend>
              <div className="grid gap-3 sm:grid-cols-2">
                <CardTypeOption label={view.reading_label} description={view.reading_description} checked={reading} onChange={setReading} />
                <CardTypeOption label={view.listening_label} description={view.listening_description} checked={listening} onChange={setListening} />
              </div>
              {!cardTypes && <p className="text-sm text-muted-foreground">Pick at least one card type.</p>}
            </fieldset>
            <p className="text-sm text-muted-foreground">Re-downloading updates matching notes. If you remove a card type, use Tools → Empty Cards in Anki to remove the old cards.</p>
            {view.manifest === "error" ? (
              <Button type="button" variant="outline" onClick={() => { setManifest("loading"); setRetry((value) => value + 1); }}>Retry loading movie clips</Button>
            ) : (
              <Button type="submit" size="lg" className="h-12 text-base font-medium" disabled={busy || !cardTypes || view.manifest !== "ready" || view.clip_sentence_count === 0}>{view.download_label}</Button>
            )}
            <div className="flex flex-col gap-2 text-sm text-muted-foreground" role="status" aria-live="polite">
              {view.manifest === "loading" && <p>Loading movie clips…</p>}
              {view.manifest === "ready" && view.clip_sentence_count === 0 && <p>No movie clips are available for this course.</p>}
              {busy && <p>{progress && progress.done < progress.total ? `Fetching media ${progress.done.toLocaleString()} of ${progress.total.toLocaleString()}` : phase}</p>}
              {busy && progress && <Progress value={progress.total ? 100 * progress.done / progress.total : 100} />}
              {busy && <p className="border-l-2 pl-4 text-foreground">{view.backstory}</p>}
              {downloadLink && <div className="flex flex-col gap-2">
                <Label htmlFor="anki-download-link">Download link</Label>
                <div className="flex gap-2">
                  <Input id="anki-download-link" readOnly value={downloadLink} className="min-w-0 font-mono" />
                  <Button type="button" variant="outline" onClick={async () => {
                    try {
                      await navigator.clipboard.writeText(downloadLink);
                      toast.success("Link copied");
                    } catch {
                      toast.error("Could not copy the link. Select and copy it manually.");
                    }
                  }}>Copy</Button>
                </div>
                <p>In AnkiMobile: Decks → Add → Download link. On AnkiDroid or desktop, open the downloaded file instead. The link works for 8 days.</p>
              </div>}
              {result && <p>Downloaded {result}</p>}
            </div>
            {result && (
              <Card className="gap-3 p-5">
                <h2 className="text-lg font-semibold">{view.keep_going_heading}</h2>
                <p className="text-sm text-muted-foreground">{view.keep_going_body}</p>
                <div className="flex flex-wrap gap-2">
                  <Button type="button" onClick={() => navigate("/learn")}>{view.keep_going_label}</Button>
                  {!userInfo && <Button type="button" variant="outline" onClick={openSignUp}>{view.sign_up_label}</Button>}
                </div>
              </Card>
            )}
          </form>
        )}
      </main>
    </TopPageLayout>
  );
}
