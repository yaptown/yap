import { useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { toast } from "sonner";
import {
  get_ai_server_url,
  anki_options_copy,
  mint_anki_deck,
  refresh_clip_manifest,
  type AnkiCardTypes,
  type AnkiDeckPlan,
  type AnkiNote,
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
import { CoursePill } from "@/components/CoursePill";
import { TopPageLayout } from "@/components/TopPageLayout";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { Progress } from "@/components/ui/progress";
import { useDeckSelection } from "@/core/useDeck";
import { useWeapon } from "@/core/weapon";
import { PlacementTest } from "@/review/ladder/PlacementTest";
import { Backstory } from "./Backstory";
import { ExportResult } from "./ExportResult";
import { saveFile } from "./save-file";
import { DeckBuilding } from "./DeckBuilding";
import { addNotes } from "./deck-build";
import { claimExports, startExport, useAnkiExport } from "./export-store";

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
// exactly one copy, so the seam never shows. While the clip manifest loads,
// blank tiles hold its place so the page doesn't jump when posters arrive.
function PosterStrip({ films, deck, loading }: { films: MovieMetadataBasic[]; deck: Deck; loading: boolean }) {
  if (films.length === 0 && !loading) return null;
  return (
    <div className="poster-strip -mx-5 overflow-hidden py-1 [mask-image:linear-gradient(to_right,transparent,black_3rem,black_calc(100%-3rem),transparent)]">
      <div className={`flex w-max ${films.length ? "poster-strip-track" : ""}`}>
        {films.length === 0 && Array.from({ length: 8 }, (_, index) => (
          <div key={index} className="shrink-0 pr-3" aria-hidden>
            <div className="h-36 w-24 animate-pulse rounded-md border border-border/50 bg-muted/40" />
          </div>
        ))}
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
  const [wordCards, setWordCards] = useState(true);
  const cardTypes: AnkiCardTypes | undefined = reading && listening ? "Both" : reading ? "Reading" : listening ? "Listening" : undefined;
  const copy = useMemo(() => anki_options_copy(reading, listening, wordCards), [reading, listening, wordCards]);
  const [manifest, setManifest] = useState<"loading" | "ready" | "error">("loading");
  const [retry, setRetry] = useState(0);
  const deckSelection = useDeckSelection();
  const startingFresh = deckSelection?.type === "languageSelected" ? deckSelection.startingFresh : undefined;
  const view = useMemo(() => ({ ...deck.anki_export_view(startingFresh), manifest }), [deck, manifest, startingFresh]);
  const { owner, phase, progress, build, run, choosing, finishMessage, summary, downloadLink, file } = useAnkiExport(view.course_code, userInfo?.id);
  // A deck finished while signed out and now seen signed in means they just
  // created an account (the page remounts when that happens): show the
  // signed-in ending in a dialog so it isn't missed below the fold.
  const [welcome, setWelcome] = useState(false);
  useEffect(() => {
    if (userInfo && owner === undefined && summary) setWelcome(true);
    claimExports(userInfo?.id);
  }, [userInfo, owner, summary]);
  const busy = phase !== undefined;
  const status = useRef<HTMLDivElement>(null);

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

  // Writes go to the export store, not component state, so the build carries
  // on (and its result stays) if the page remounts under it.
  async function download(cardTypes: AnkiCardTypes) {
    const update = startExport(view.course_code, userInfo?.id);
    update((current) => ({ ...current, phase: "Preparing deck…" }));
    // On a phone the status sits below the fold; bring it (and the backstory) up.
    requestAnimationFrame(() => status.current?.scrollIntoView({ behavior: "smooth", block: "start" }));
    try {
      const options = { card_types: cardTypes, word_cards: wordCards };
      const minted = await mint_anki_deck(options, accessToken);
      update((current) => ({ ...current, phase: "Choosing sentences…", choosing: true }));
      await new Promise((resolve) => setTimeout(resolve, 0));
      const planner = deck.start_anki_deck_plan(options, minted.token, Date.now());
      let plan: AnkiDeckPlan;
      try {
        let done = false;
        while (!done) {
          const deadline = performance.now() + 12;
          const added: AnkiNote[] = [];
          let step;
          do {
            step = planner.step();
            added.push(...step.notes);
          } while (!step.done && performance.now() < deadline);
          const progress = { done: step.sentences_chosen, total: step.target_size };
          update((current) => ({ ...current, progress, build: addNotes(current.build, added) }));
          done = step.done;
          // A macrotask, not a resolved Promise: input and paint get a turn.
          await new Promise((resolve) => setTimeout(resolve, 0));
        }
        plan = planner.finish();
      } finally {
        planner.free();
        update((current) => ({ ...current, choosing: false }));
      }
      const { buildApkg } = await import("./apkg-client");
      update((current) => ({ ...current, finishMessage: plan.finish_message, progress: undefined, phase: "Fetching media…" }));
      const blob = await buildApkg(plan, (source) => deck.anki_bundled_media(source), (progress) => {
        update((current) => ({ ...current, progress, phase: progress.done === progress.total ? "Writing Anki package…" : current.phase }));
      });
      update((current) => ({ ...current, phase: "Uploading deck…" }));
      let downloadLink: string | undefined;
      try {
        downloadLink = await uploadPackage(minted, plan.course_code, blob, accessToken);
      } catch (error) {
        toast.error(`Could not create a download link. Your deck will still download. ${String(error)}`);
      }
      const file = { blob, name: `yap-${plan.course_code}.apkg` };
      saveFile(file);
      const notes = `${plan.stats.sentence_count.toLocaleString()} sentence notes${plan.stats.word_count ? ` + ${plan.stats.word_count.toLocaleString()} word notes` : ""}`;
      const summary = `${notes} · ${(blob.size / 1024 / 1024).toFixed(1)} MB`;
      update((current) => ({ ...current, downloadLink, file, summary }));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      update((current) => ({ ...current, phase: undefined, progress: undefined }));
    }
  }

  return (
    <TopPageLayout userInfo={userInfo} headerProps={{ title: "Anki Decks", backButton: { label: "Yap.Town", onBack: () => navigate("/") } }}>
      <main className="mx-auto flex w-full max-w-2xl flex-col gap-6 px-5 py-8">
        <div className="flex flex-col gap-4">
          <CoursePill flag={view.course_flag} label={view.course_label} onClick={() => navigate(`/select-language?next=${encodeURIComponent("/anki")}`)} />
          <h1 className="text-2xl font-semibold" style={{ textWrap: "balance" }}>{view.title}</h1>
        </div>
        <PosterStrip films={view.films} deck={deck} loading={view.manifest === "loading"} />
        {view.needs_placement ? (
          <div className="flex flex-col gap-4">
            <p className="text-sm text-muted-foreground">{view.placement_intro}</p>
            <AnkiPlacement deck={deck} targetLanguage={targetLanguage} completeLabel={view.placement_complete_label} />
          </div>
        ) : (
          <form className="flex flex-col gap-6" onSubmit={(event) => { event.preventDefault(); if (cardTypes) void download(cardTypes); }}>
            <p className="text-muted-foreground">{copy.intro}</p>
            {/* Same bar as the Essential tab on the goals screen. */}
            <div className="flex flex-col gap-3 border-y py-4">
              <h2 className="font-semibold">{view.progress_label}</h2>
              <Progress className="h-6" value={view.percent_known} showPercentage aria-label={view.progress_label} />
              {view.too_advanced_message && <p className="text-sm text-muted-foreground">{view.too_advanced_message}</p>}
            </div>
            <fieldset className="flex flex-col gap-3" disabled={busy}>
              <legend className="mb-3 font-semibold">{view.card_types_label}</legend>
              <div className="grid gap-3">
                <CardTypeOption label={view.reading_label} description={copy.reading_description} checked={reading} onChange={setReading} />
                <CardTypeOption label={view.listening_label} description={copy.listening_description} checked={listening} onChange={setListening} />
                <CardTypeOption label={view.word_cards_label} description={copy.word_cards_description} checked={wordCards} onChange={setWordCards} />
              </div>
              {!cardTypes && <p className="text-sm text-muted-foreground">Pick at least one card type.</p>}
            </fieldset>
            <p className="text-sm text-muted-foreground">Re-downloading updates matching notes. If you remove a card type, use Tools → Empty Cards in Anki to remove the old cards.</p>
            {view.manifest === "error" ? (
              <Button type="button" variant="outline" onClick={() => { setManifest("loading"); setRetry((value) => value + 1); }}>Retry loading movie clips</Button>
            ) : (
              <Button type="submit" size="lg" className="h-12 text-base font-medium" disabled={busy || !cardTypes || view.manifest !== "ready" || view.clip_sentence_count === 0}>{view.download_label}</Button>
            )}
            <div ref={status} className="flex scroll-mt-4 flex-col gap-2 text-sm text-muted-foreground" role="status" aria-live="polite">
              {view.manifest === "loading" && <p>Loading movie clips…</p>}
              {view.manifest === "ready" && view.clip_sentence_count === 0 && <p>No movie clips are available for this course.</p>}
              {(busy || summary) && (
                <div className="flex flex-col gap-4 pt-2 text-base text-foreground">
                  <DeckBuilding
                    key={run}
                    build={build}
                    deck={deck}
                    language={targetLanguage}
                    choosing={choosing}
                    animate={busy}
                    status={phase && (!choosing && progress && progress.done < progress.total ? `${phase} ${progress.done.toLocaleString()} of ${progress.total.toLocaleString()}` : phase)}
                    progress={busy && progress ? (progress.total ? 100 * progress.done / progress.total : 100) : undefined}
                    target={choosing ? progress?.total : undefined}
                    finishMessage={finishMessage}
                  />
                  {busy && <Backstory text={view.backstory} signature={view.backstory_signature} />}
                </div>
              )}
              {summary && <p>Downloaded {summary}</p>}
            </div>
            {summary && (
              <Card className="p-5">
                <ExportResult view={view} signedIn={!!userInfo} file={file} downloadLink={downloadLink} onSignUp={openSignUp} onGoToYap={() => navigate("/learn")} />
              </Card>
            )}
            <Dialog open={welcome} onOpenChange={setWelcome}>
              <DialogContent className="max-h-[90dvh] overflow-y-auto sm:max-w-lg">
                <ExportResult view={view} signedIn file={file} downloadLink={downloadLink} onSignUp={openSignUp} onGoToYap={() => navigate("/learn")} Title={DialogTitle} Body={DialogDescription} />
              </DialogContent>
            </Dialog>
          </form>
        )}
      </main>
    </TopPageLayout>
  );
}
